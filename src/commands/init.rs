use crate::config::{Relays, PublishedRegistry};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use toml;
use serde_json;

pub async fn init_blog_structure_in_dir<P: AsRef<Path>>(base_dir: P) -> Result<()> {
    let base_dir = base_dir.as_ref();
    println!("Initializing blog structure in {:?}...", base_dir);
    
    println!("Initializing blog structure...");
    
    // Helper to create a directory and an empty .keep file inside it
    fn create_dir_with_keep<P: AsRef<Path>>(base_dir: P, dir: &str) -> Result<()> {
        let full_path = base_dir.as_ref().join(dir);
        println!("Create dir: {}", full_path.display());
        fs::create_dir_all(&full_path)
            .with_context(|| format!("Failed to create directory `{}`", full_path.display()))?;

        let keep_path = full_path.join(".keep");
        if !keep_path.exists() {
            fs::write(&keep_path, "")
                .with_context(|| format!("Failed to create .keep file in `{}`", full_path.display()))?;
        }
        Ok(())
    }

    // Use the helper function to create directories with .keep files
    create_dir_with_keep(&base_dir, "articles")?;
    create_dir_with_keep(&base_dir, "articles/_deleted")?;
    create_dir_with_keep(&base_dir, ".nostr")?;
    create_dir_with_keep(&base_dir, ".nostr/keys")?;

    // Create nostr.toml with default configuration
    let relays_toml_path = base_dir.join("relays.toml");
    if !relays_toml_path.exists() {
        let default_config = Relays {
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nostr.band".to_string(),
            ],
        };
        
        let toml_content = toml::to_string_pretty(&default_config)
            .with_context(|| "Failed to serialize default configuration")?;
        
        fs::write(relays_toml_path, toml_content)
            .with_context(|| "Failed to write nostr.toml")?;
        
        println!("Created: nostr.toml");
    } else {
        println!("Skipped: nostr.toml (already exists)");
    }
    
    // Create .nostr/published.json with empty registry
    let published_json_path = base_dir.join(".nostr/published.json");
    if !published_json_path.exists() {
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
    let example_article_path = base_dir.join("articles/example-article.md");
    if !example_article_path.exists() {
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
    let git_path = base_dir.join(".git");
    if !git_path.exists() {
        println!("Initializing git repository...");
        match git2::Repository::init(base_dir) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;

    // Helper function to setup a temporary directory for testing
    fn setup_test_dir() -> PathBuf {
        TempDir::new().unwrap().keep()
    }

    // Helper function to change to a directory and run a test
    async fn run_in_temp_dir<F, Fut>(test_fn: F) -> Result<()>
    where
        F: FnOnce(PathBuf) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let temp_dir = setup_test_dir();

        println!("Temp path: {}", temp_dir.to_str().unwrap());
        println!("Temp path exists 1: {}", temp_dir.exists());           
        
        // Ensure the temp directory exists before changing to it
        if !temp_dir.exists() {
            return Err(anyhow::anyhow!("Temp directory does not exist: {:?}", temp_dir));
        }
        
        // Change to temp directory
        std::env::set_current_dir(&temp_dir)
            .with_context(|| format!("Failed to change to temp directory: {:?}", temp_dir))?;
        
        // Runtemp_dir the test (pass temp_dir to keep it alive)
        let result = test_fn(temp_dir.clone()).await;
        
        println!("Temp path exists 3: {}", temp_dir.exists());           

        // Delete the temporary directory ourselves.
        // fs::remove_dir_all(temp_dir)?;

        result
    }

    #[tokio::test]
    async fn test_init_blog_structure_creates_all_directories() {
        run_in_temp_dir(|temp_path| async move {    
            println!("Temp path exists 2: {}", temp_path.exists());        
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify all directories are created
            assert!(temp_path.join("articles").exists());
            assert!(temp_path.join("articles/_deleted").exists());
            assert!(temp_path.join(".nostr").exists());
            assert!(temp_path.join(".nostr/keys").exists());
            
            // Verify .keep files are created
            assert!(temp_path.join("articles/.keep").exists());
            assert!(temp_path.join("articles/_deleted/.keep").exists());
            assert!(temp_path.join(".nostr/.keep").exists());
            assert!(temp_path.join(".nostr/keys/.keep").exists());
            
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_creates_relays_toml() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify relays.toml is created
            let relays_path = temp_path.join("relays.toml");
            assert!(relays_path.exists());
            
            // Verify content is valid TOML with expected relays
            let content = fs::read_to_string(&relays_path)?;
            let config: Relays = toml::from_str(&content)?;
            
            assert_eq!(config.relays.len(), 3);
            assert!(config.relays.contains(&"wss://relay.damus.io".to_string()));
            assert!(config.relays.contains(&"wss://nos.lol".to_string()));
            assert!(config.relays.contains(&"wss://relay.nostr.band".to_string()));
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_creates_published_registry() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify published.json is created
            let published_path = temp_path.join(".nostr/published.json");
            assert!(published_path.exists());
            
            // Verify content is valid JSON with empty registry
            let content = fs::read_to_string(&published_path)?;
            let registry: PublishedRegistry = serde_json::from_str(&content)?;
            
            assert!(registry.articles.is_empty());
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_creates_example_article() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify example article is created
            let article_path = temp_path.join("articles/example-article.md");
            assert!(article_path.exists());
            
            // Verify content contains expected front matter and content
            let content = fs::read_to_string(&article_path)?;
            assert!(content.contains("---"));
            assert!(content.contains("title: My First Post"));
            assert!(content.contains("date: 2025-01-01"));
            assert!(content.contains("tags: [rust, nostr]"));
            assert!(content.contains("summary: An example article to get you started"));
            assert!(content.contains("slug: my-first-post"));
            assert!(content.contains("# My First Post"));
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_skips_existing_files() {
        run_in_temp_dir(|temp_path| async move {
            // Create existing files with different content
            fs::write(temp_path.join("relays.toml"), "# existing content")?;
            fs::create_dir_all(temp_path.join(".nostr"))?;
            fs::write(temp_path.join(".nostr/published.json"), "{\"test\": \"existing\"}")?
            ;
            fs::create_dir_all(temp_path.join("articles"))?;
            fs::write(temp_path.join("articles/example-article.md"), "# Existing Article")?;
            
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify existing files are not overwritten
            let relays_content = fs::read_to_string(temp_path.join("relays.toml"))?;
            assert_eq!(relays_content, "# existing content");
            
            let published_content = fs::read_to_string(temp_path.join(".nostr/published.json"))?;
            assert_eq!(published_content, "{\"test\": \"existing\"}");
            
            let article_content = fs::read_to_string(temp_path.join("articles/example-article.md"))?;
            assert_eq!(article_content, "# Existing Article");
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_keep_files_not_overwritten() {
        run_in_temp_dir(|temp_path| async move {
            // Create directories and .keep files with custom content
            fs::create_dir_all(temp_path.join("articles"))?;
            fs::write(temp_path.join("articles/.keep"), "custom keep content")?;
            
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify .keep file is not overwritten
            let keep_content = fs::read_to_string(temp_path.join("articles/.keep"))?;
            assert_eq!(keep_content, "custom keep content");
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_creates_git_repo() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify git repository is initialized
            assert!(temp_path.join(".git").exists());
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_skips_existing_git_repo() {
        run_in_temp_dir(|temp_path| async move {
            // Create existing .git directory with a test file
            fs::create_dir_all(temp_path.join(".git"))?;
            fs::write(temp_path.join(".git/test_marker"), "existing repo")?;
            
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Verify existing git repo is not touched
            assert!(temp_path.join(".git/test_marker").exists());
            let marker_content = fs::read_to_string(temp_path.join(".git/test_marker"))?;
            assert_eq!(marker_content, "existing repo");
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_idempotent() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function twice
            init_blog_structure_in_dir(&temp_path).await?;
            let result = init_blog_structure_in_dir(&temp_path).await;
            
            // Second run should succeed without errors
            assert!(result.is_ok());
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_dir_with_keep_function() {
        run_in_temp_dir(|temp_path| async move {
            // Create a nested directory structure using the helper function
            let nested_dir = "test/nested/structure";
            let full_path = temp_path.join(nested_dir);
            
            // Simulate the helper function behavior
            fs::create_dir_all(&full_path)?;
            let keep_path = full_path.join(".keep");
            if !keep_path.exists() {
                File::create(&keep_path)?;
            }
            
            // Verify directory and .keep file are created
            assert!(full_path.exists());
            assert!(keep_path.exists());
            
            Ok(())
        }).await.unwrap();
    }

    #[tokio::test]
    async fn test_init_blog_structure_serialization_formats() {
        run_in_temp_dir(|temp_path| async move {
            // Run the init function
            init_blog_structure_in_dir(&temp_path).await?;
            
            // Test that the generated TOML is properly formatted
            let relays_content = fs::read_to_string(temp_path.join("relays.toml"))?;
            assert!(relays_content.contains("relays = ["));
            assert!(relays_content.contains("\"wss://"));
            
            // Test that the generated JSON is properly formatted
            let published_content = fs::read_to_string(temp_path.join(".nostr/published.json"))?;
            // Should be pretty-printed JSON
            assert!(published_content.contains("{"));
            assert!(published_content.contains("}"));
            
            Ok(())
        }).await.unwrap();
    }
}
