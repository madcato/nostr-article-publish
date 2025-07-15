# Publish long form text note on nostr

Read the [project description](doc/description.md) to know how it was made.

## Execute

```sh
cargo run publish -f fixtures/post.md -a test-article-01 -i "https://i.sstatic.net/jaiMD.jpg?s=256"
```

### To delete a previous publication
```sh
cargo run delete -a test-article-01
```