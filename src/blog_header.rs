use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::fs;
use std::path::Path;
use chrono::NaiveDate;

// Custom serialization/deserialization for NaiveDate
fn serialize_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(d) => serializer.serialize_str(&d.format("%Y-%m-%d").to_string()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlogHeader {
    pub title: Option<String>,
    #[serde(serialize_with = "serialize_date", deserialize_with = "deserialize_date")]
    pub published_at: Option<NaiveDate>,
    pub image: Option<String>,
    pub summary: Option<String>,
    pub slug: String,  // Required field
    #[serde(skip)]  // Don't include content in YAML serialization
    pub content: String,  // The markdown content after frontmatter
}

impl BlogHeader {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        
        // Check if file starts with frontmatter delimiter
        if !content.starts_with("---") {
            return Err("File must start with YAML frontmatter delimiter '---'".into());
        }

        // Find the end of frontmatter
        let lines: Vec<&str> = content.lines().collect();
        let mut frontmatter_end = 0;
        let mut found_end = false;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                frontmatter_end = i;
                found_end = true;
                break;
            }
        }
        
        if !found_end {
            return Err("Could not find end of frontmatter delimiter '---'".into());
        }

        // Extract frontmatter and content
        let frontmatter = lines[1..frontmatter_end].join("\n");
        let markdown_content = lines[(frontmatter_end + 1)..].join("\n");

        // Parse frontmatter as YAML
        let mut header: BlogHeader = serde_yaml::from_str(&frontmatter)?;
        
        // Validate required fields
        if header.slug.is_empty() {
            return Err("Slug field is required and cannot be empty".into());
        }

        // Set the content
        header.content = markdown_content.trim().to_string();

        Ok(header)
    }

    pub fn save<P: AsRef<Path>>(&self, file_path: P) -> Result<(), Box<dyn std::error::Error>> {
        // Validate required fields before saving
        if self.slug.is_empty() {
            return Err("Cannot save: slug field is required and cannot be empty".into());
        }

        // Serialize frontmatter to YAML
        let yaml_frontmatter = serde_yaml::to_string(self)?;
        
        // Construct the full markdown file content
        let full_content = format!(
            "---\n{yaml}---\n\n{content}",
            yaml = yaml_frontmatter,
            content = self.content
        );

        // Write to file
        fs::write(file_path, full_content)?;
        
        Ok(())
    }

    pub fn is_published(&self) -> bool {
        self.published_at.is_some()
    }

    pub fn get_title(&self) -> &str {
        self.title.as_deref().unwrap_or("Untitled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, tempdir};
    use std::io::Write;

    #[test]
    fn test_blog_header_parsing() {
        let content = r#"---
title: My first post
published_at: 2025-04-01
image: ./images/header-01.jpg
summary: A short summary
slug: my-first-post
---

# This is my first blog post

Some content here with **markdown** formatting.
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let header = BlogHeader::new(temp_file.path()).unwrap();
        
        assert_eq!(header.title, Some("My first post".to_string()));
        assert_eq!(header.slug, "my-first-post");
        assert!(header.published_at.is_some());
        assert!(header.is_published());
        assert_eq!(header.get_title(), "My first post");
        assert!(header.content.contains("# This is my first blog post"));
    }

    #[test]
    fn test_save_and_reload() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_post.md");

        // Create a blog header
        let header = BlogHeader {
            title: Some("Test Post".to_string()),
            published_at: Some(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            image: Some("./test-image.jpg".to_string()),
            summary: Some("Test summary".to_string()),
            slug: "test-post".to_string(),
            content: "# Test Content\n\nThis is a test post.".to_string(),
        };

        // Save the file
        header.save(&file_path).unwrap();

        // Reload and verify
        let reloaded_header = BlogHeader::new(&file_path).unwrap();
        
        assert_eq!(reloaded_header.title, header.title);
        assert_eq!(reloaded_header.published_at, header.published_at);
        assert_eq!(reloaded_header.image, header.image);
        assert_eq!(reloaded_header.summary, header.summary);
        assert_eq!(reloaded_header.slug, header.slug);
        assert_eq!(reloaded_header.content, header.content);
    }

    #[test]
    fn test_save_minimal_header() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("minimal_post.md");

        let header = BlogHeader {
            title: None,
            published_at: None,
            image: None,
            summary: None,
            slug: "minimal-post".to_string(),
            content: "Just minimal content.".to_string(),
        };

        // Save and reload
        header.save(&file_path).unwrap();
        let reloaded_header = BlogHeader::new(&file_path).unwrap();
        
        assert_eq!(reloaded_header.slug, "minimal-post");
        assert_eq!(reloaded_header.content, "Just minimal content.");
        assert_eq!(reloaded_header.title, None);
    }

    #[test]
    fn test_save_with_empty_slug_fails() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid_post.md");

        let header = BlogHeader {
            title: Some("Invalid Post".to_string()),
            published_at: None,
            image: None,
            summary: None,
            slug: "".to_string(),  // Empty slug should fail
            content: "Content".to_string(),
        };

        let result = header.save(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("slug field is required"));
    }

    #[test]
    fn test_modify_and_save() {
        let dir = tempdir().unwrap();
        let original_path = dir.path().join("original.md");
        let modified_path = dir.path().join("modified.md");

        // Create original file
        let original_content = r#"---
title: Original Title
slug: original-post
---

Original content here.
"#;
        fs::write(&original_path, original_content).unwrap();

        // Load, modify, and save
        let mut header = BlogHeader::new(&original_path).unwrap();
        header.title = Some("Modified Title".to_string());
        header.published_at = Some(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap());
        header.content = "Modified content here.".to_string();

        header.save(&modified_path).unwrap();

        // Verify modifications
        let reloaded = BlogHeader::new(&modified_path).unwrap();
        assert_eq!(reloaded.title, Some("Modified Title".to_string()));
        assert_eq!(reloaded.published_at, Some(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()));
        assert_eq!(reloaded.content, "Modified content here.");
        assert_eq!(reloaded.slug, "original-post"); // Should remain unchanged
    }
}