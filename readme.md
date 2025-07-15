# Nostr Long-Form Content Publisher

## Execute

```sh
cargo run publish -f fixtures/post.md -a test-article-01 -i "https://i.sstatic.net/jaiMD.jpg?s=256"
```

### To delete a previous publication
```sh
cargo run delete -a test-article-01
```



A command-line tool written in Rust for publishing and deleting long-form content events (NIP-23) on the Nostr protocol. It supports validation of content, custom tags for articles, and interaction with multiple relays configured via a TOML file.

## Features

- Publish Markdown articles as Nostr long-form text notes with optional metadata (title, image, summary, publication timestamp).
- Delete published articles by identifier.
- Content validation to ensure no HTML tags or hard line breaks.
- Uses environment variable for secure key handling.
- Asynchronous relay connections using the `nostr-sdk` crate.

## Prerequisites

- Rust (version 1.70 or later recommended) and Cargo installed. See [rustup.rs](https://rustup.rs/) for installation.
- A Nostr secret key (in Bech32 format, e.g., `nsec1...`) for signing events.

## Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   cd nostr-publish
   ```

2. Build the binary:
   ```
   cargo build --release
   ```
   The executable will be available at `target/release/<binary-name>` (binary name is derived from your Cargo.toml, e.g., `nostr-publish`).

Alternatively, install directly via Cargo if published to crates.io (adjust if applicable):
```
cargo install nostr-publish
```

## Configuration

### Relays Configuration
Create a `relays.toml` file in the working directory (or specify a custom path with `-c/--config`):
```toml
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    # Add more relays as needed
]
```

### Environment Variable
Set your Nostr secret key:
```
export NOSTR_SEC_KEY="nsec1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```
**Warning:** Never commit or share your secret key. Use environment variables or secure secret management tools.

## Usage

Run the tool with subcommands `publish` or `delete`. Use `--help` for detailed options.

### General Flags
- `-c, --config <FILE>`: Path to custom relays TOML file (default: `relays.toml`).

### Subcommand: Publish
Publish a new long-form content event from a Markdown file.

```
./target/release/nostr-publish publish -f <FILE_NAME> -a <ARTICLE_IDENTIFIER> [OPTIONS]
```

Options:
- `-f, --file-name <FILE_NAME>`: Path to the Markdown content file (required).
- `-a, --article-identifier <ARTICLE_IDENTIFIER>`: Unique identifier for the article (required).
- `-t, --title <TITLE>`: Article title (optional).
- `-i, --image <URL>`: URL to an image for the article (optional).
- `-s, --summary <SUMMARY>`: Article summary (optional, added as a hashtag tag).
- `-p, --published-at <TIMESTAMP>`: Unix timestamp for publication date (optional, defaults to current time).

Content Rules:
- The file must contain plain Markdown text.
- No HTML tags allowed.
- No hard line breaks (e.g., arbitrary breaks at 80 columns); use soft wraps.

Output: Prints the Bech32 public key and the generated Event ID on success.

### Subcommand: Delete
Delete an existing long-form content event by identifier.

```
./target/release/nostr-publish delete -a <ARTICLE_IDENTIFIER>
```

Options:
- `-a, --article-identifier <ARTICLE_IDENTIFIER>`: Identifier of the article to delete (required).

Behavior:
- Searches for matching events on connected relays (timeout: 10 seconds).
- If found, broadcasts a deletion event (NIP-09) with reason "Deleted by user request".
- Output: Prints the deletion Event ID or a message if no events are found.

## Examples

### Publishing an Article
Assume `article.md` contains your Markdown content and `relays.toml` is set up.

```
export NOSTR_SEC_KEY="nsec1..."
./target/release/nostr-publish publish -f article.md -a my-article-id -t "My First Article" -i https://example.com/image.jpg -s "A summary here" -p 1721059200
```

Expected Output:
```
Bech32 PubKey: npub1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Generated EventId: <event-id-hex>
```

### Deleting an Article
```
./target/release/nostr-publish delete -a my-article-id
```

Expected Output:
```
Bech32 PubKey: npub1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Deletion EventId: <deletion-event-id-hex>
```

Or if no events found:
```
Bech32 PubKey: npub1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Not found events to delete.
```

### Using Custom Config
```
./target/release/nostr-publish publish -f article.md -a my-article-id -c custom-relays.toml
```

## Dependencies

This tool relies on the following crates:
- `clap`: Command-line argument parsing.
- `nostr-sdk`: Nostr protocol implementation.
- `anyhow`: Error handling.
- `serde` and `toml`: Configuration parsing.
- `regex`: Content validation.
- `tokio`: Asynchronous runtime.

See `Cargo.toml` for full details and versions.

## Troubleshooting

- **Relay Connection Issues:** Ensure relays in `relays.toml` are valid and reachable. Check Nostr relay status if needed.
- **Key Parsing Errors:** Verify `NOSTR_SEC_KEY` is a valid Bech32 `nsec` string.
- **Content Validation Fails:** Remove any HTML or manual line breaks from your Markdown file.
- **Timeout on Delete:** Increase the timeout in code if searching large histories (default: 10s).

## Contributing

Contributions are welcome! Open issues or pull requests on the repository.

## License

MIT License. See [LICENSE](LICENSE) for details.