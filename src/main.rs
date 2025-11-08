use std::env;
use anyhow::{Context, bail, Result};
use serde::{Serialize, Deserialize};
use std::fs;
use toml;
use regex::Regex;
use clap::Parser;
use nostr_sdk::prelude::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::select;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// Generate Nostr pub/sec keys randomly or with BIP39 mnemonic code.
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Create base blog files structure.
    Init,
    /// Show what is published.
    Status,
    /// Publish an new long-form content event on configured nostr relays.
    Publish {
        /// File name of the content to publish.
        #[arg(short, long)]
        file_name: String,
        /// Identifier of the article.
        #[arg(short, long)]
        article_identifier: String,
        /// Title of the article.
        #[arg(short, long)]
        title: Option<String>,
        /// URL pointing to an image to be shown along with the title
        #[arg(short, long)]
        image: Option<Url>,
        /// Article summary
        #[arg(short, long)]
        summary: Option<String>,
        /// Timestamp in unix seconds (stringified) of the first time the article 
        #[arg(short, long)]
        published_at: Option<u64>,
    },
    /// Delete an event from configured nostr relays.
    Delete {
        /// Identifier of the event to delete. Must be the same used in the publish.
        #[arg(short, long)]
        article_identifier: String,
    },
    /// Force complete resynchronization.
    Sync,
    /// List all articles published by the sec key owner.
    List {
        /// Timestamp in unix seconds (stringified) of the first time the article to list, this is not the "published_at" tag, but the event time.
        #[arg(short, long)]
        since_published: Option<u64>,
        /// Timestamp in unix seconds (stringified) of the last time the article to list, this is not the "published_at" tag, but the event time.
        #[arg(short, long)]
        until_published: Option<u64>,
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Relays {
    relays: Vec<String>
}

fn validate_content(content: &String) -> Result<()> {
    if content.contains("\\n") {
        bail!("Content MUST NOT hard line-break paragraphs of text, such as arbitrary line breaks at 80 column boundaries.");
    }

    let re = Regex::new(r"<[^>]+>").unwrap();
    if re.is_match(content) {
        bail!("Content MUST NOT have HTML.");
    }

    Ok(())
}

async fn publish_article(file_name: String, article_identifier: String, title: Option<String>, image: Option<Url>, summary: Option<String>, published_at: Option<u64>, client: Client, public_key: PublicKey) -> Result<()> {
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
    // let event: Event = client.sign_event_builder(builder).await?;
    // println!("{:?}", event.tags);
    // println!("{:?}", event.coordinate());
    Ok(())
}

async fn delete_article(article_identifier: String, client: Client, public_key: PublicKey) -> Result<()> {
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
    // let event: Event = client.sign_event_builder(builder).await?;
    // println!("{:?}",event.tags);
    Ok(())
}

async fn list_articles(since_published: Option<u64>, until_published: Option<u64>, client: Client, public_key: PublicKey) -> Result<()> {
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

    loop {
        if eose_received || start.elapsed() > timeout_duration {
            break;
        }

        select! {
            Ok(notification) = notifications.recv() => {
                match notification {
                    RelayPoolNotification::Event { relay_url, event, subscription_id: sid, .. } if sid == sub_id_1 => {
                        println!("Article id: {:?} on relay: {}", event.tags, relay_url);
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

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load nostr sec key to sign the message
    let bech32_sec_key = env::var("NOSTR_SEC_KEY").with_context(|| format!("To launch this command, define the enviroment variable NOSTR_SEC_KEY with the signing key"))?;
    let keys = Keys::parse(&bech32_sec_key)?;

    let config_file = args.config.unwrap_or(String::from("relays.toml"));

    // Load relays from relays.toml
    let relays_str = fs::read_to_string(config_file).with_context(|| format!("Configuration file 'relays.toml' could not be read."))?;
    let relays: Relays = toml::from_str(&relays_str).with_context(|| format!("Error deserializing 'relays.toml'."))?;

    // Show bech32 public key
    let bech32_pubkey: String = keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client with custom options
    let client = Client::builder().signer(keys.clone()).build();
    
    // Add relays
    for relay in relays.relays {
        client.add_relay(relay).await?;
    }
    
    // Connect to relays
    client.connect().await;

    match args.command {
        Commands::Init { } => { },  // TODO: Implement
        Commands::Status { } => { },  // TODO: Implement
        Commands::Publish { file_name, article_identifier, title, image, summary, published_at } => { publish_article(file_name, article_identifier, title, image, summary, published_at, client, keys.public_key()).await? },
        Commands::Delete { article_identifier } => { delete_article(article_identifier, client, keys.public_key()).await? },
        Commands::List { since_published, until_published } => { list_articles(since_published, until_published, client, keys.public_key()).await? },
        Commands::Sync { } => {}  // TODO: Implement
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests for validate_content function
    #[test]
    fn test_validate_content_valid() {
        let content = "This is valid Markdown.\n\n# Heading\nParagraph.".to_string();
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_empty() {
        let content = String::new();
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_normal_newlines() {
        let content = "Line 1\nLine 2\n\nParagraph 2".to_string();
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_markdown_valid() {
        let content = "# Title\n\n**Bold text** and *italic text*.\n\n- List item 1\n- List item 2\n\n[Link](https://example.com)".to_string();
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_html_invalid() {
        let content = "This has <p>HTML</p>".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MUST NOT have HTML"));
    }

    #[test]
    fn test_validate_content_various_html_tags() {
        let test_cases = vec![
            "<div>content</div>",
            "<span>text</span>",
            "<img src='test.jpg'>",
            "<a href='link'>text</a>",
            "<br>",
            "<h1>heading</h1>",
        ];

        for html_content in test_cases {
            let content = html_content.to_string();
            let result = validate_content(&content);
            assert!(result.is_err(), "HTML content should be invalid: {}", html_content);
        }
    }

    #[test]
    fn test_validate_content_backslash_n_invalid() {
        let content = "This has \\n".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MUST NOT hard line-break"));
    }

    #[test]
    fn test_validate_content_multiple_backslash_n() {
        let content = "Line 1\\nLine 2\\nLine 3".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_backslash_n_in_middle() {
        let content = "Beginning of text\\nand more text here".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_html_and_backslash_n() {
        let content = "<div>HTML content</div>\\nwith backslash n".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
        // Should fail on backslash n first (since that check comes first)
        assert!(result.unwrap_err().to_string().contains("MUST NOT hard line-break"));
    }

    // Tests for Relays struct
    #[test]
    fn test_relays_deserialization_valid() {
        let toml_content = r#"
            relays = [
                "wss://relay1.example.com",
                "wss://relay2.example.com",
                "wss://relay3.example.com"
            ]
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 3);
        assert_eq!(relays.relays[0], "wss://relay1.example.com");
        assert_eq!(relays.relays[1], "wss://relay2.example.com");
        assert_eq!(relays.relays[2], "wss://relay3.example.com");
    }

    #[test]
    fn test_relays_deserialization_empty() {
        let toml_content = r#"
            relays = []
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 0);
    }

    #[test]
    fn test_relays_deserialization_single() {
        let toml_content = r#"
            relays = ["wss://single-relay.example.com"]
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 1);
        assert_eq!(relays.relays[0], "wss://single-relay.example.com");
    }

    #[test]
    fn test_relays_serialization() {
        let relays = Relays {
            relays: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
            ]
        };
        
        let serialized = toml::to_string(&relays);
        assert!(serialized.is_ok());
        
        let serialized = serialized.unwrap();
        assert!(serialized.contains("relay1.example.com"));
        assert!(serialized.contains("relay2.example.com"));
    }

    #[test]
    fn test_relays_deserialization_invalid_toml() {
        let invalid_toml = r#"
            invalid_key = "value"
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(invalid_toml);
        assert!(relays.is_err());
    }

    #[test]
    fn test_relays_deserialization_wrong_type() {
        let invalid_toml = r#"
            relays = "should_be_array"
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(invalid_toml);
        assert!(relays.is_err());
    }

    // Test round-trip serialization/deserialization
    #[test]
    fn test_relays_round_trip() {
        let original = Relays {
            relays: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
                "wss://relay3.example.com".to_string(),
            ]
        };
        
        let serialized = toml::to_string(&original).unwrap();
        let deserialized: Relays = toml::from_str(&serialized).unwrap();
        
        assert_eq!(original.relays.len(), deserialized.relays.len());
        for (orig, deser) in original.relays.iter().zip(deserialized.relays.iter()) {
            assert_eq!(orig, deser);
        }
    }

    // Test edge cases for content validation
    #[test]
    fn test_validate_content_unicode() {
        let content = "Unicode content: 🎉 测试 العربية ñoño".to_string();
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_very_long() {
        let content = "a".repeat(10000);
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_html_with_attributes() {
        let content = r#"<div class="test" id="element">Content</div>"#.to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_self_closing_html() {
        let content = "<br /> and <img src='test.jpg' />".to_string();
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_angle_brackets_not_html() {
        let content = "Math: 5 less than 10 greater than 3".to_string();
        // This should be valid as it has no angle brackets that could be mistaken for HTML
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_mathematical_comparison() {
        let content = "Compare values: a < b, where 5 < 10".to_string();
        // This should be valid as these are just comparison operators without closing >
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_angle_brackets_html_like() {
        let content = "Math: a<b>c where <b> looks like HTML".to_string();
        // This should be invalid as <b> looks like an HTML tag
        let result = validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_html_in_code_block() {
        let content = "Here's some code: `<div>html</div>`".to_string();
        // The regex will still catch this as it looks for any <tag> pattern
        let result = validate_content(&content);
        assert!(result.is_err());
    }
}