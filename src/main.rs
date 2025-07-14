use std::env;
use clap::Parser;
use nostr_sdk::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// Generate Nostr pub/sec keys randomly or with BIP39 mnemonic code.
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Publish an new long-form content event on configured nostr relays.
    Publish {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load nosrt sec key to sign the message
    let bech32_sec_key = env::var("NOSTR_SEC_KEY")?;
    let keys = Keys::parse(&bech32_sec_key)?;

    // Show bech32 public key
    let bech32_pubkey: String = keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client with custom options
    let client = Client::builder().signer(keys.clone()).build();
    
    // Add relays
    client.add_relay("ws://micro-atx:18080").await?;
    
    // Connect to relays
    client.connect().await;

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