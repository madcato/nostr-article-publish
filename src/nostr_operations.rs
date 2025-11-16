use crate::config::{PublishedEvent, PublishedRegistry};
use crate::validation::validate_content;
use crate::blog_header::BlogHeader;
use chrono::{DateTime, NaiveDate};
use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};
use tokio::select;
use std::collections::HashMap;
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
        let event = article.0;
        let slug = generate_slug_from_event(&event);
        let filename = format!("{}.md", slug);
        let file_path = Path::new("./articles").join(filename.clone());

        // Create the directory if it doesn't exist
        fs::create_dir_all(file_path.parent().unwrap())?;

        // Try to parse existing BlogHeader if content has frontmatter
        let blog_header = if event.content.starts_with("---") {
            // Content already has frontmatter, try to parse it
            match parse_event_with_frontmatter(&event) {
                Ok(header) => header,
                Err(_) => create_blog_header_from_event(&event, &slug),
            }
        } else {
            // No frontmatter, create new header from event data
            create_blog_header_from_event(&event, &slug)
        };

        // Save the blog header to file
        if let Err(e) = blog_header.save(&file_path) {
            eprintln!("Failed to save article {}: {}", filename, e);
            continue;
        }

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

fn generate_slug_from_event(event: &Event) -> String {
    // Try to extract title from content for slug generation
    let first_line = event.content.lines().next().unwrap_or("");
    
    if first_line.starts_with("# ") {
        // Extract title from markdown header
        let title = first_line.trim_start_matches("# ").trim();
        slug_from_title(title)
    } else {
        // Fallback to event ID
        event.id.to_string()[..8].to_string() // Use first 8 chars of event ID
    }
}

fn slug_from_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn parse_event_with_frontmatter(event: &Event) -> Result<BlogHeader, Box<dyn std::error::Error>> {
    // Create temporary file to use BlogHeader::new()
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(event.content.as_bytes())?;
    
    let mut header = BlogHeader::new(temp_file.path())?;
    
    // Update with event metadata if not present
    if header.published_at.is_none() {
        header.published_at = convert_timestamp_to_date(event.created_at);
    }
    
    // Extract image URL from event tags if not already present
    if header.image.is_none() {
        header.image = extract_image_url_from_event(event);
    }
    
    Ok(header)
}

fn create_blog_header_from_event(event: &Event, slug: &str) -> BlogHeader {
    let content = event.content.clone();
    
    // Try to extract title from first line
    let title = extract_title_from_content(&content);
    
    // Convert timestamp to date
    let published_at = convert_timestamp_to_date(event.created_at);
    
    // Extract summary from hashtag tag
    let summary = event.tags.iter()
        .find_map(|tag| {
            tag.content();
             if tag.kind() == TagKind::t() {
                tag.content().map(|s| s.to_string())
            } else {
                None
            }
        });
    
    // Extract image URL from event tags
    let image = extract_image_url_from_event(event);
    
    BlogHeader {
        title,
        published_at,
        image,
        summary,
        slug: slug.to_string(),
        content,
    }

}

fn extract_title_from_content(content: &str) -> Option<String> {
    let first_line = content.lines().next().unwrap_or("").trim();
    
    if first_line.starts_with("# ") {
        Some(first_line.trim_start_matches("# ").trim().to_string())
    } else {
        None
    }
}

// fn generate_summary(content: &str) -> Option<&str> {
//     // Take first paragraph that's not a header as summary
//     for line in content.lines() {
//         let trimmed = line.trim();
//         if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.len() > 20 {
//             // Truncate to reasonable summary length
//             if trimmed.len() > 150 {
//                 return Some(&trimmed[..147]);
//             } else {
//                 return Some(trimmed);
//             }
//         }
//     }
//     None
// }

fn convert_timestamp_to_date(timestamp: Timestamp) -> Option<NaiveDate> {
    // Assuming timestamp is Unix timestamp
    match DateTime::from_timestamp(timestamp.as_secs() as i64, 0) {
        Some(datetime) => Some(datetime.naive_utc().date()),
        None => None,
    }
}

fn extract_image_url_from_event(event: &Event) -> Option<String> {
    // Look for image tags in the event
    event.tags.iter()
        .find_map(|tag| {
            if tag.kind() == TagKind::Image {
                // Try to extract URL from imeta or image tag content
                tag.content().map(|s| s.to_string())
            } else {
                None
            }
        })
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
