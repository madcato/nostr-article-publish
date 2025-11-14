use crate::config::{Relays, PublishedRegistry};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use toml;
use serde_json;

pub async fn init_blog_structure() -> Result<()> {
    println!("Initializing blog structure...");
    
    // Helper to create a directory and an empty .keep file inside it
    fn create_dir_with_keep(dir: &str) -> Result<()> {
        // Create the directory (including parents)
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory `{}`", dir))?;

        // Create the .keep file if it does not already exist
        let keep_path = Path::new(dir).join(".keep");
        if !keep_path.exists() {
            File::create(&keep_path)
                .with_context(|| format!("Failed to create .keep in `{}`", dir))?;
        }
        println!("Created: {}/", dir);
        Ok(())
    }

    // Use the helper function to create directories with .keep files
    create_dir_with_keep("articles")?;
    create_dir_with_keep("images")?;
    create_dir_with_keep("articles/_deleted")?;
    create_dir_with_keep(".nostr")?;
    create_dir_with_keep(".nostr/keys")?;

    // Create nostr.toml with default configuration
    let nostr_toml_path = "nostr.toml";
    if !Path::new(nostr_toml_path).exists() {
        let default_config = Relays {
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nostr.band".to_string(),
            ],
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
            articles: HashMap::new(),
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
"#;

        fs::write(example_article_path, example_content)
            .with_context(|| "Failed to write example article")?;

        println!("Created: articles/example-article.md");
    } else {
        println!("Skipped: articles/example-article.md (already exists)");
    }

    // Initialize git repository if not already initialized
    let git_path = Path::new(".git");
    if !git_path.exists() {
        println!("Initializing git repository...");
        match git2::Repository::init(".") {
            Ok(_) => println!("Initialized git repository"),
            Err(e) => eprintln!("Warning: Failed to initialize git repository: {}", e),
        }
    } else {
        println!("Skipped: git repository (already initialized)");
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
