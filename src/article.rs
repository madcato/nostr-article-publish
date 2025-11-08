struct Article {
    path: PathBuf,
    title: String,
    slug: String,
    published_at: Option<DateTime<Utc>>,
    tags: Vec<String>,
    summary: Option<String>,
    content: String,
    event_id: Option<String>,  // ID del evento Nostr
    deleted: bool,
}