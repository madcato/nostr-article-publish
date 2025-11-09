use crate::cli::Args;
use crate::client::init_nostr_client;
use crate::nostr_operations::{publish_article, delete_article, list_articles};
use anyhow::Result;
use clap::Parser;
use nostr_sdk::Url;

pub async fn action_publish(
    file_name: String, 
    article_identifier: String, 
    title: Option<String>, 
    image: Option<Url>, 
    summary: Option<String>, 
    published_at: Option<u64>
) -> Result<()> {
    let args = Args::parse();
    let (keys, client) = init_nostr_client(args).await?;
    publish_article(file_name, article_identifier, title, image, summary, published_at, client, keys.public_key()).await
}

pub async fn action_delete(article_identifier: String) -> Result<()> {
    let args = Args::parse();
    let (keys, client) = init_nostr_client(args).await?;
    delete_article(article_identifier, client, keys.public_key()).await
}

pub async fn action_list(since_published: Option<u64>, until_published: Option<u64>) -> Result<()> {
    let args = Args::parse();
    let (keys, client) = init_nostr_client(args).await?;
    list_articles(since_published, until_published, client, keys.public_key()).await
}