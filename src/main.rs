use std::env;
use anyhow::{Context, bail, Result};
use serde::{Serialize, Deserialize};
use std::fs;
use toml;
use regex::Regex;
use clap::Parser;
use nostr_sdk::prelude::*;

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
async fn publish_article(file_name: String, article_identifier: String, title: Option<String>, image: Option<Url>, summary: Option<String>, published_at: Option<u64>, client: Client) -> Result<()> {
    let content = fs::read_to_string(file_name).with_context(|| format!("Content file could not be read."))?;
    validate_content(&content)?;
    
    let mut tags = Vec::from([
        Tag::identifier(article_identifier),
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

    let timestamp = published_at.unwrap_or_else(|| Timestamp::now().as_u64());
    tags.push(Tag::from_standardized(TagStandard::PublishedAt(Timestamp::from(timestamp))));
     
    // Publish a text note
    let builder = EventBuilder::long_form_text_note(content).tags(tags);
    let event = client.send_event_builder(builder).await?;
    println!("Generated EventId: {}", event.to_hex());

    Ok(())
}

async fn delete_article(article_identifier: String, client: Client, public_key: PublicKey) -> Result<()> {
    let coordinate = Coordinate { kind: Kind::LongFormTextNote, public_key: public_key, identifier: article_identifier };
    let request = EventDeletionRequest::new().coordinate(coordinate);
    let k_tag = Tag::from_standardized(TagStandard::Kind { kind: Kind::LongFormTextNote  , uppercase: false });
    let builder = EventBuilder::delete(request).tag(k_tag);
    client.send_event_builder(builder).await?;
    // let event: Event = client.sign_event_builder(builder).await?;
    // println!("{:?}",event.tags);
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
        Commands::Publish { file_name, article_identifier, title, image, summary, published_at } => { publish_article(file_name, article_identifier, title, image, summary, published_at, client).await? },
        Commands::Delete { article_identifier } => { delete_article(article_identifier, client, keys.public_key()).await? }
    }

    Ok(())
}
