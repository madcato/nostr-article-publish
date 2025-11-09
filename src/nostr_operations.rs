use crate::config::{PublishedEvent, PublishedRegistry};
use crate::validation::validate_content;
use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};
use tokio::select;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::Write;

pub async fn publish_article(
    file_name: String, 
    article_identifier: String, 
    title: Option<String>, 
    image: Option<Url>, 
    summary: Option<String>, 
    published_at: Option<u64>, 
    client: Client, 
    public_key: PublicKey
) -> Result<()> {
    let content = fs::read_to_string(file_name).with_context(|| format!("Content file could not be read."))?;
    validate_content(&content)?;
    
    let mut tags = Vec::from([
        Tag::identifier(article_identifier.clone()),
    ]);

    if let Some(title) = title {
        tags.push(Tag::from_standardized(TagStandard::Title(title)));
    }

    if let Some(summary) = summary {
        tags.push(Tag::hashtag(summary));
    }

    if let Some(image) = image {
        let dimensions = ImageDimensions::new(200, 200);
        tags.push(Tag::image(image, Some(dimensions)));
    }

    let timestamp = published_at.unwrap_or_else(|| Timestamp::now().as_secs());
    tags.push(Tag::from_standardized(TagStandard::PublishedAt(Timestamp::from(timestamp))));

    let coordinate = Coordinate { kind: Kind::LongFormTextNote, public_key: public_key, identifier: article_identifier };
    tags.push(Tag::from_standardized(TagStandard::Coordinate { coordinate: coordinate, relay_url: None, uppercase: false }));
    
    // Publish a text note
    let builder = EventBuilder::long_form_text_note(content).tags(tags);
    let event = client.send_event_builder(builder).await?;
    println!("Generated EventId: {}", event.to_hex());
    Ok(())
}

pub async fn delete_article(article_identifier: String, client: Client, public_key: PublicKey) -> Result<()> {
    let coordinate = Coordinate { kind: Kind::LongFormTextNote, public_key: public_key, identifier: article_identifier };

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
    Ok(())
}

pub async fn list_articles_from_relays(since_published: Option<u64>, until_published: Option<u64>, client: Client, public_key: PublicKey) -> Result<Vec<(Box<nostr_sdk::Event>,nostr_sdk::RelayUrl)>> {
    let mut filter = Filter::new()
                            .author(public_key)
                            .kind(Kind::LongFormTextNote); // 30023
    
    if let Some(since_published) = since_published {
        filter = filter.since(Timestamp::from_secs(since_published));
    }

    if let Some(until_published) = until_published {
        filter = filter.until(Timestamp::from_secs(until_published));
    }
                        // .since(Timestamp::now());
    let Output { val: sub_id_1, .. } = client.subscribe(filter, None).await?;

    let mut eose_received = false;

    let timeout_duration = Duration::from_secs(10);
    let start = Instant::now();

    let mut notifications = client.notifications();

    let mut article_events: Vec<(Box<nostr_sdk::Event>,nostr_sdk::RelayUrl)> = Vec::new();

    loop {
        if eose_received || start.elapsed() > timeout_duration {
            break;
        }

        select! {
            Ok(notification) = notifications.recv() => {
                match notification {
                    RelayPoolNotification::Event { relay_url, event, subscription_id: sid, .. } if sid == sub_id_1 => {
                        article_events.push((event, relay_url));
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

    Ok(article_events)
}

pub async fn list_articles(since_published: Option<u64>, until_published: Option<u64>, client: Client, public_key: PublicKey) -> Result<()> {
    let articles = list_articles_from_relays(since_published, until_published, client, public_key).await?;

    for article in articles {
        let event = article.0;
        let relay_url = article.1;
        println!("Article id: {:?} on relay: {}", event.tags, relay_url);
    }

    Ok(())
}

pub async fn sync_articles(client: Client, public_key: PublicKey) -> Result<()> {
    let articles = list_articles_from_relays(None, None, client, public_key).await?;

    let mut published_events: HashMap<String, PublishedEvent> = HashMap::new();

    for article in articles {
        // Assuming `event.id` is a unique identifier for the article
        let event = article.0;
        let filename = format!("{}.md", event.id);
        let file_path = Path::new("./articles").join(filename.clone());

        // Create the directory if it doesn't exist
        fs::create_dir_all(file_path.parent().unwrap())?;

        // Write the content of the event to a file
        let mut file = File::create(&file_path)?;
        file.write_all(event.content.as_bytes())?;

        let published_event = PublishedEvent {
            event_id: event.id.to_string(),
            published_at: event.created_at.to_string(),
            kind: event.kind.as_u16(),
        };

        published_events.insert(filename, published_event);
    }

    let registry = PublishedRegistry {
        articles: published_events,
    };
    
    fs::create_dir_all(".nostr/")?;

    if let Err(e) = registry.save_to_path(".nostr/published.json") {
        println!("{}", e);
    }

    Ok(())
}
