use clap::Parser;
use nostr_sdk::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// Generate Nostr pub/sec keys randomly or with BIP39 mnemonic code.
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,
}

#[derive(Parser, Debug)]
pub enum Commands {
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
    /// Force complete resynchronization. Publish pending articles, delete marked for deletion, update eixting ones, and download not existing ones.
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