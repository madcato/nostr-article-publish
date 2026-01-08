use crate::errors::NostrPublishError;
use regex::Regex;
use anyhow::Result;

pub struct ContentValidator;

impl ContentValidator {
    pub fn validate_content(content: &str) -> Result<(), NostrPublishError> {
        Self::check_html_tags(content)?;
        Self::check_hard_line_breaks(content)?;
        Ok(())
    }
    
    fn check_html_tags(content: &str) -> Result<(), NostrPublishError> {
        let html_regex = Regex::new(r"<[^>]+>").unwrap();
        let found_tags: Vec<String> = html_regex
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();
            
        if !found_tags.is_empty() {
            return Err(NostrPublishError::HtmlTagsFound { tags: found_tags });
        }
        Ok(())
    }

    fn check_hard_line_breaks(content: &str) -> Result<(), NostrPublishError> {
        if content.contains("\\n") {
            return Err(NostrPublishError::HardLineBreak);
        }
        Ok(())
    }
    
    fn validate_url(url: &str) -> Result<(), NostrPublishError> {
        // Validar URLs de imágenes
        url::Url::parse(url)
            .map_err(|_| NostrPublishError::InvalidFileFormat { 
                file: url.to_string() 
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests for validate_content function
    #[test]
    fn test_validate_content_valid() {
        let content = "This is valid Markdown.\n\n# Heading\nParagraph.";
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_empty() {
        let content = String::new();
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_normal_newlines() {
        let content = "Line 1\nLine 2\n\nParagraph 2".to_string();
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_markdown_valid() {
        let content = "# Title\n\n**Bold text** and *italic text*.\n\n- List item 1\n- List item 2\n\n[Link](https://example.com)".to_string();
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_html_invalid() {
        let content = "This has <p>HTML</p>".to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Content contains HTML tags"));
    }

    #[test]
    fn test_validate_content_various_html_tags() {
        let test_cases = vec![
            "<div>content</div>",
            "<span>text</span>",
            "<img src='test.jpg'>",
            "<a href='link'>text</a>",
            "<br>",
            "<h1>heading</h1>",
        ];

        for html_content in test_cases {
            let content = html_content.to_string();
            let result = ContentValidator::validate_content(&content);
            assert!(result.is_err(), "HTML content should be invalid: {}", html_content);
        }
    }

    #[test]
    fn test_validate_content_backslash_n_invalid() {
        let content = "This has \\n".to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MUST NOT hard line-break"));
    }

    #[test]
    fn test_validate_content_multiple_backslash_n() {
        let content = "Line 1\\nLine 2\\nLine 3".to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_backslash_n_in_middle() {
        let content = "Beginning of text\\nand more text here".to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }

    // Test edge cases for content validation
    #[test]
    fn test_validate_content_unicode() {
        let content = "Unicode content: 🎉 测试 العربية ñoño".to_string();
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_very_long() {
        let content = "a".repeat(10000);
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_html_with_attributes() {
        let content = r#"<div class="test" id="element">Content</div>"#.to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_self_closing_html() {
        let content = "<br /> and <img src='test.jpg' />".to_string();
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_angle_brackets_not_html() {
        let content = "Math: 5 less than 10 greater than 3".to_string();
        // This should be valid as it has no angle brackets that could be mistaken for HTML
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_mathematical_comparison() {
        let content = "Compare values: a < b, where 5 < 10".to_string();
        // This should be valid as these are just comparison operators without closing >
        assert!(ContentValidator::validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_angle_brackets_html_like() {
        let content = "Math: a<b>c where <b> looks like HTML".to_string();
        // This should be invalid as <b> looks like an HTML tag
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_html_in_code_block() {
        let content = "Here's some code: `<div>html</div>`".to_string();
        // The regex will still catch this as it looks for any <tag> pattern
        let result = ContentValidator::validate_content(&content);
        assert!(result.is_err());
    }
}