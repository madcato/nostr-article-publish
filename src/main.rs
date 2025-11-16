mod cli;
mod config;
mod validation;
mod client;
mod nostr_operations;
mod init;
mod actions;
mod blog_header;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Commands};
use init::init_blog_structure;
use actions::{action_publish, action_delete, action_list, action_sync};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Init { } => { init_blog_structure().await? },
        Commands::Publish { file_name, article_identifier, title, image, summary, published_at } => { 
            action_publish(file_name, article_identifier, title, image, summary, published_at).await? 
        },
        Commands::Delete { article_identifier } => { action_delete(article_identifier).await? },
        Commands::List { since_published, until_published } => { action_list(since_published, until_published).await? },
        Commands::Sync { } => { action_sync().await? },  
    }

    Ok(())
}