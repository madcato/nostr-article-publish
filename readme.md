# Publish long form text note on nostr

Read the [project description](doc/description.md) to know how it was made.

## Execute

```sh
cargo run publish -f fixtures/post.md -a test-article-01 -i "https://primaldata.s3.us-east-005.backblazeb2.com/cache/7/8e/6e/78e6e2a5bfd3066ea8ceec980ad88e2e07a239d4708a74ad8904b45b4a33b2a7.jpg"
```

### To delete a previous publication
```sh
cargo run delete -a test-article-01
```