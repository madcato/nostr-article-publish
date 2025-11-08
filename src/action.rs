enum Action {
    Publish { article: Article, event_id: Option<String> },
    Replace { article: Article, old_event_id: String },
    Delete { path: PathBuf, event_id: String },
    NoOp,
}