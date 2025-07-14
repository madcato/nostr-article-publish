# Publish long form text note on nostr

Read the [project description](doc/description.md) to know how it was made.

## Execute

```sh
cargo run publish -f fixtures/post.md -i "https://primaldata.s3.us-east-005.backblazeb2.com/cache/7/8e/6e/78e6e2a5bfd3066ea8ceec980ad88e2e07a239d4708a74ad8904b45b4a33b2a7.jpg"
```

### To delete a previous publication
```sh
cargo run delete -i 69700e2f32faee33e53098d9c38148926fdd98d8513ec2b566cf17c9af2a2048
```