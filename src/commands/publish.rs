use crate::config::{PublishedEvent, PublishedRegistry};
use crate::validation::ContentValidator;
use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    let content = fs::read_to_string(&file_name).with_context(|| format!("Content file could not be read."))?;
    ContentValidator::validate_content(&content)?;
    
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

    let coordinate = Coordinate { kind: Kind::LongFormTextNote, public_key: public_key, identifier: article_identifier.clone() };
    tags.push(Tag::from_standardized(TagStandard::Coordinate { coordinate: coordinate, relay_url: None, uppercase: false }));
    
    // Publish a text note
    let builder = EventBuilder::long_form_text_note(content).tags(tags);
    let event = client.send_event_builder(builder).await?;
    println!("Generated EventId: {}", event.to_hex());
    
    // Save to published registry
    save_to_published_registry(&file_name, &article_identifier, &event)?;
    
    Ok(())
}

fn save_to_published_registry(file_name: &str, article_identifier: &str, event: &EventId) -> Result<()> {
    // Ensure .nostr directory exists
    fs::create_dir_all(".nostr")?;
    
    let registry_path = ".nostr/published.json";
    
    // Load existing registry or create new one
    let mut registry = match PublishedRegistry::load_from_path(registry_path) {
        Ok(registry) => registry,
        Err(_) => PublishedRegistry {
            articles: HashMap::new(),
        },
    };
    
    // Create filename from the original file path
    let filename = Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(article_identifier)
        .to_string();
    
    // Create published event entry
    let published_event = PublishedEvent {
        event_id: event.to_hex(),
        published_at: Timestamp::now().to_string(),
        kind: Kind::LongFormTextNote.as_u16(),
    };
    
    // Add to registry
    registry.articles.insert(filename, published_event);
    
    // Save registry
    if let Err(e) = registry.save_to_path(registry_path) {
        println!("{}", e);
    }

    println!("Article saved to published registry: {}", registry_path);
    Ok(())
}