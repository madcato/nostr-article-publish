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
        published_at: Option<String>,
    },
    /// Delete an event from configured nostr relays.
    Delete {
        /// Identifier of the event to delete.
        #[arg(short, long)]
        identifier: String,
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
async fn publish_long_content_event(file_name: String, title: Option<String>, image: Option<Url>, summary: Option<String>, published_at: Option<String>, client: Client) -> Result<()> {
    let content = fs::read_to_string(file_name).with_context(|| format!("Content file could not be read."))?;
    validate_content(&content)?;
    let event_id = EventId::from_hex("b3e392b11f5d4f28321dedd09303a748acfd0487aea5a7450b3481c60b6e4f87").unwrap();
    let content: &str = "# Title\n## Section 1\nMy first text note from rust-nostr!\n## Section 2\nWhat!?\n";
    let image_url = Url::parse("https://primaldata.s3.us-east-005.backblazeb2.com/cache/7/8e/6e/78e6e2a5bfd3066ea8ceec980ad88e2e07a239d4708a74ad8904b45b4a33b2a7.jpg")?;
    let dimensions = ImageDimensions::new(200, 200);
    let tags = [
        Tag::identifier("long-form-content-2".to_string()),
        Tag::from_standardized(TagStandard::Title("Long-form Content".to_string())),
        Tag::from_standardized(TagStandard::PublishedAt(Timestamp::from(1296962229))),
        Tag::image(image_url, Some(dimensions)),
        Tag::hashtag("placeholder".to_string()),
        Tag::event(event_id),
    ];
     
    // Publish a text note
    let builder = EventBuilder::long_form_text_note(content).tags(tags);

    client.send_event_builder(builder).await?;

    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    // Load nostr sec key to sign the message
    let bech32_sec_key = env::var("NOSTR_SEC_KEY").with_context(|| format!("To launch this command, define the enviroment variable NOSTR_SEC_KEY with the signing key"))?;
    let keys = Keys::parse(&bech32_sec_key)?;

    // Load relays from relays.toml
    let relays_str = fs::read_to_string("relays.toml").with_context(|| format!("Configuration file 'relays.toml' could not be read."))?;
    let relays: Relays = toml::from_str(&relays_str).with_context(|| format!("Error deserializing 'relays.toml'."))?;

    let args = Args::parse();

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
        Commands::Publish { file_name, title, image, summary, published_at } => { publish_long_content_event(file_name, title, image, summary, published_at, client).await? },
        Commands::Delete { identifier } => {}
    }

    Ok(())
}
