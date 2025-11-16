use crate::config::{PublishedEvent, PublishedRegistry};
use crate::blog_header::BlogHeader;
use chrono::{DateTime, NaiveDate};
use anyhow::Result;
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::select;

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