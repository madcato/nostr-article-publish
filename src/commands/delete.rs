use crate::config::PublishedRegistry;
use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::select;

pub async fn delete_article(article_identifier: String, client: Client, public_key: PublicKey) -> Result<()> {
    let coordinate = Coordinate { kind: Kind::LongFormTextNote, public_key: public_key, identifier: article_identifier.clone() };

    // Create subscription to find the article to delete
    let subscription = Filter::new()
        .author(public_key)
        .kind(Kind::LongFormTextNote)
        .coordinate(&coordinate);
    let Output { val: sub_id_1, .. } = client.subscribe(subscription, None).await?;

    let mut event_ids: HashSet<EventId> = HashSet::new();
    let mut eose_received = false;

    let timeout_duration = Duration::from_secs(10);
    let start = Instant::now();

    let mut notifications = client.notifications();

    loop {
        if eose_received || start.elapsed() > timeout_duration {
            break;
        }

        select! {
            Ok(notification) = notifications.recv() => {
                match notification {
                    RelayPoolNotification::Event { event, subscription_id: sid, .. } if sid == sub_id_1 => {
                        event_ids.insert(event.id);
                    }
                    RelayPoolNotification::Message { message, .. } => {
                        if let RelayMessage::EndOfStoredEvents(sid) = message {
                            if sid.into_owned() == sub_id_1 {
                                eose_received = true;
                            }
                        }
                    }
                    _ => {},
                }
            }
            _ = tokio::time::sleep(timeout_duration - start.elapsed()) => {
                break;
            }
        }
    }

    client.unsubscribe(&sub_id_1).await;

    if event_ids.is_empty() {
        println!("Not found events to delete.");
        return Ok(());
    }

    // Create deletion event
    let request = EventDeletionRequest::new()
                                            .coordinate(coordinate)
                                            .reason("Deleted by user request");
    let k_tag = Tag::from_standardized(TagStandard::Kind { kind: Kind::LongFormTextNote  , uppercase: false });
    let mut builder = EventBuilder::delete(request)
                                    .tag(k_tag);

    for id in &event_ids {
        builder = builder.tag(Tag::event(*id));
    }
    let event = client.send_event_builder(builder).await?;
    println!("Deletion EventId: {}", event.to_hex());

    // Move the file from ./articles to ./articles/_deleted
    move_deleted_article_file(&article_identifier)?;
    
    Ok(())
}

fn move_deleted_article_file(article_identifier: &str) -> Result<()> {
    // Create the filename with .md extension
    let filename = format!("{}.md", article_identifier);
    let source_path = Path::new("./articles").join(&filename);
    
    // Check if the file exists in the articles directory
    if !source_path.exists() {
        println!("Article file not found: {}", source_path.display());
        // Still try to remove from registry even if file doesn't exist
        remove_from_published_registry(&filename)?;
        return Ok(());
    }
    
    // Create the _deleted directory if it doesn't exist
    let deleted_dir = Path::new("./articles/_deleted");
    fs::create_dir_all(deleted_dir)
        .with_context(|| format!("Failed to create directory: {}", deleted_dir.display()))?;
    
    // Move the file to the _deleted directory
    let destination_path = deleted_dir.join(&filename);
    fs::rename(&source_path, &destination_path)
        .with_context(|| format!("Failed to move file from {} to {}", source_path.display(), destination_path.display()))?;
    
    println!("Moved deleted article from {} to {}", source_path.display(), destination_path.display());
    
    // Remove the entry from the published registry
    remove_from_published_registry(&filename)?;
    
    Ok(())
}

fn remove_from_published_registry(filename: &str) -> Result<()> {
    // Ensure .nostr directory exists
    fs::create_dir_all(".nostr")?;
    
    let registry_path = ".nostr/published.json";
    
    // Load existing registry
    let mut registry = match PublishedRegistry::load_from_path(registry_path) {
        Ok(registry) => registry,
        Err(_) => {
            println!("Published registry not found or invalid: {}", registry_path);
            return Ok(());
        }
    };
    
    // Remove the article from the registry
    match registry.articles.remove(filename) {
        Some(_) => {
            // Save the updated registry
            if let Err(e) = registry.save_to_path(registry_path) {
                println!("Failed to save updated registry: {}", e);
            } else {
                println!("Removed article '{}' from published registry", filename);
            }
        }
        None => {
            println!("Article '{}' not found in published registry", filename);
        }
    }
    
    Ok(())
}