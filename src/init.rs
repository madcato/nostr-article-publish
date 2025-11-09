use crate::config::{Relays, PublishedRegistry};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml;

pub async fn init_blog_structure() -> Result<()> {
    println!("Initializing blog structure...");
    
    // Create articles directory
    fs::create_dir_all("articles")
        .with_context(|| "Failed to create articles directory")?;
    println!("Created: articles/");
    
    // Create articles/_deleted directory
    fs::create_dir_all("articles/_deleted")
        .with_context(|| "Failed to create articles/_deleted directory")?;
    println!("Created: articles/_deleted/");
    
    // Create .nostr directory
    fs::create_dir_all(".nostr")
        .with_context(|| "Failed to create .nostr directory")?;
    println!("Created: .nostr/");
    
    // Create .nostr/keys directory
    fs::create_dir_all(".nostr/keys")
        .with_context(|| "Failed to create .nostr/keys directory")?;
    println!("Created: .nostr/keys/");
    
    // Create nostr.toml with default configuration
    let nostr_toml_path = "nostr.toml";
    if !Path::new(nostr_toml_path).exists() {
        let default_config = Relays {
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nostr.band".to_string(),
            ]
        };
        
        let toml_content = toml::to_string_pretty(&default_config)
            .with_context(|| "Failed to serialize default configuration")?;
        
        fs::write(nostr_toml_path, toml_content)
            .with_context(|| "Failed to write nostr.toml")?;
        
        println!("Created: nostr.toml");
    } else {
        println!("Skipped: nostr.toml (already exists)");
    }
    
    // Create .nostr/published.json with empty registry
    let published_json_path = ".nostr/published.json";
    if !Path::new(published_json_path).exists() {
        let empty_registry = PublishedRegistry {
            articles: HashMap::new()
        };
        
        let json_content = serde_json::to_string_pretty(&empty_registry)
            .with_context(|| "Failed to serialize published registry")?;
        
        fs::write(published_json_path, json_content)
            .with_context(|| "Failed to write .nostr/published.json")?;
        
        println!("Created: .nostr/published.json");
    } else {
        println!("Skipped: .nostr/published.json (already exists)");
    }
    
    // Create example article
    let example_article_path = "articles/example-article.md";
    if !Path::new(example_article_path).exists() {
        let example_content = r#"---
title: My First Post
date: 2025-01-01
tags: [rust, nostr]
summary: An example article to get you started
slug: my-first-post
---

# My First Post

Welcome to your Nostr blog! This is an example article.

## Getting Started

Edit this file or create new `.md` files in the `articles/` directory.

## Publishing

Use the following command to publish your articles:

```bash
nostr-publish publish -f articles/example-article.md -a my-first-post
```

**Remember**: Set your `NOSTR_SEC_KEY` environment variable before publishing!
"#;
        
        fs::write(example_article_path, example_content)
            .with_context(|| "Failed to write example article")?;
        
        println!("Created: articles/example-article.md");
    } else {
        println!("Skipped: articles/example-article.md (already exists)");
    }
    
    println!();
    println!("Blog structure initialized successfully!");
    println!();
    println!("Next steps:");
    println!("1. Set your Nostr secret key: export NOSTR_SEC_KEY='nsec1...'");
    println!("2. Edit articles/example-article.md or create new articles");
    println!("3. Publish with: nostr-publish publish -f articles/your-article.md -a article-id");
    
    Ok(())
}