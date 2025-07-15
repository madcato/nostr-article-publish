# Project description

This project publish event `kind:30023` on nostr.
The `.content` of these events should be a string text in Markdown syntax.

### Metadata fields
- "title", for the article title
- "image", for a URL pointing to an image to be shown along with the title
- "summary", for the article summary (placeholder)
- "published_at", for the timestamp in unix seconds (stringified) of the first time the article was published

### Addresable events

- for kind `n` such that `30000 <= n < 40000`, events are **addressable** by their `kind`, `pubkey` and `d` tag value -- which means that, for each combination of `kind`, `pubkey` and the `d` tag value, only the latest event MUST be stored by relays, older versions MAY be discarded.

## Info
- [nip-23: Long-form Content](https://github.com/nostr-protocol/nips/blob/master/23.md)
- [nip-09: Event deletion](https://github.com/nostr-protocol/nips/blob/master/09.md)

## ToDo
- [x] Load parameters from command line.
- [x] Load sec key from environment. Check if exists, show error: "To launch this command, first define the enviroment variable NOSTR_SEC_KEY with the signing key".
- [x] Read relays from a TOML file.
- [x] Check if the content has forced hard line-breaks. Show an error: "MUST NOT hard line-break paragraphs of text, such as arbitrary line breaks at 80 column boundaries.".
- [x] Check if content has HTML tags. Show an error: "MUST NOT support adding HTML to Markdown."
- [x] Read from command arguments this fields, all optional: _title_, _image_, _summary_, and _published_at_.
- [x] Read config file from arguments.
- [x] Create and show identifier of the created event.
- [x] Ask user for an article identifier, mandatory.
- [x] Implement **delete** command.
- [x] Analyze how `d` identifier works.
- [x] Adapt delete command to use addresable events, `d` tag.
- [x] ~~Check every relay if allows `d`. Do not publish to that relay. Show error: "This relay does not allow to edit the content".~~
- [x] ~~Analyze [bip-19](https://github.com/nostr-protocol/nips/blob/master/19.md).~~
- [x] For deletion command: create a REQ so find the articles, and use `e` tag to remove all.
- [ ] Document in readme.md how to use this project.
- [ ] Add first version Changelog
