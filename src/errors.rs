use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrPublishError {
    #[error("Invalid Nostr secret key format")]
    InvalidSecretKey,
    
    #[error("Content contains HTML tags")]
    HtmlTagsFound { tags: Vec<String> },
    
    #[error("Content has hard line breaks at positions: {positions:?}")]
    HardLineBreaks { positions: Vec<usize> },
    
    #[error("Relay connection failed: {relay} - {reason}")]
    RelayConnectionFailed { relay: String, reason: String },
    
    #[error("Article identifier '{id}' already exists")]
    DuplicateArticleId { id: String },
    
    #[error("Missing required environment variable: {var}")]
    MissingEnvVar { var: String },
    
    #[error("Invalid file format: {file}")]
    InvalidFileFormat { file: String },
    
    #[error("Network timeout after {seconds}s")]
    NetworkTimeout { seconds: u64 },

    #[error("Content MUST NOT hard line-break paragraphs of text, such as arbitrary line breaks at 80 column boundaries.")]
    HardLineBreak,
}