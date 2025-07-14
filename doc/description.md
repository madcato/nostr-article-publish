# Project description

This project publish event `kind:30023` on nostr.
The `.content` of these events should be a string text in Markdown syntax.

### Metadata fields
- "title", for the article title
- "image", for a URL pointing to an image to be shown along with the title
- "summary", for the article summary (placeholder)
- "published_at", for the timestamp in unix seconds (stringified) of the first time the article was published

## Info
- [nips-23: Long-form Content](https://github.com/nostr-protocol/nips/blob/master/23.md)

## ToDo
- [ ] Load parameters from command line.
- [ ] Load sec key from environment. Check if exists, show error: "To launch this command, first define the enviroment variable NOSTR_SEC_KEY with the signing key.
- [ ] Read relays from a YAML file.
- [ ] Check if the content has forced hard line-breaks. Show an error: "MUST NOT hard line-break paragraphs of text, such as arbitrary line breaks at 80 column boundaries.".
- [ ] Check if content has HTML tags. Show an error: "MUST NOT support adding HTML to Markdown."
- [ ] Read from command arguments this fields, all optional: _title_, _image_, _summary_, and _published_at_.
- [ ] Analyze what `d` identifier is.
- [ ] Check every relay if allows `d`. Do not publish to that relay. Show error: "This relay does not allow to edit the content".
- [ ] Analyze [bip-19](https://github.com/nostr-protocol/nips/blob/master/19.md).
- [ ] Show identifier of the created event.
- [ ] Impliment **modify** command.
- [ ] Implement **delete** command.
- [ ] Document in readme.md how to use this project.

