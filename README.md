# clubs-cli

Command-line interface for composing, sealing, and inspecting Gordian Club editions.

This crate is under active development.

## Single-Publisher Workflow (Conceptual)

The forthcoming CLI will weave together existing Gordian tools. The script below mirrors the expected sequence with correct command syntax; long UR payloads are replaced with semantic placeholders for readability.

```bash
# 1. Generate the publisher's signing material and XID document.
#    `envelope generate prvkeys` yields a PrivateKeyBase; piping it into
#    `envelope xid new` embeds the key (and salt) inside a ur:xid envelope.
PUBLISHER_PRVKEYS=$(envelope generate prvkeys --seed ur:seed/PublisherSeed)
PUBLISHER_XID=$(envelope xid new $PUBLISHER_PRVKEYS)
printf '%s\n' "$PUBLISHER_XID" > publisher.xid.ur
# -> ur:xid/Publisher (includes signing keys + metadata)

# 2. Prepare recipient descriptors.
#    These may be existing documents or newly generated identities.
printf 'ur:xid/Alice\n' > alice.xid.ur
printf 'ur:crypto-pubkeys/Bob\n' > bob.pubkeys.ur

# 3. Assemble the club content envelope.
#    The subject and assertions use standard envelope subcommands.
CONTENT_ENVELOPE=$( \
  envelope subject type string "Welcome to the club!" | \
  envelope assertion add pred-obj string "title" string "Genesis Edition" \
)
printf '%s\n' "$CONTENT_ENVELOPE" > content.env.ur
# -> ur:envelope/Content

# 4. Establish a provenance mark chain and capture the genesis mark UR.
#    The chain directory holds generator.json plus mark-*.json artifacts.
provenance new publisher-chain --seed "Base64PublisherSeed==" --comment "Genesis edition"
GENESIS_MARK=$(provenance print publisher-chain --start 0 --end 0 \
  | awk '/^#### ur:provenance/{print $2; exit}')
printf '%s\n' "$GENESIS_MARK" > genesis.prov.ur
# -> ur:provenance/Mark0

# 5. Compose the first edition (clubs-cli wiring in progress).
clubs init \
  --publisher @publisher.xid.ur \
  --provenance @genesis.prov.ur \
  --content @content.env.ur \
  --permit @alice.xid.ur \
  --permit @bob.pubkeys.ur \
  --sskr 2of3
# -> ur:envelope/Edition (signed) + optional ur:sskr/Share*

# 6. Inspect the assembled edition.
clubs edition inspect --edition ur:envelope/Edition --summary

# 7. Publish subsequent editions by advancing the provenance chain.
provenance next publisher-chain --comment "Follow-up release"
NEXT_MARK=$(provenance print publisher-chain --start 1 --end 1 \
  | awk '/^#### ur:provenance/{print $2; exit}')
clubs edition compose \
  --publisher @publisher.xid.ur \
  --provenance "$NEXT_MARK" \
  --content ur:envelope/ContentV2 \
  --previous ur:envelope/Edition \
  --permit @alice.xid.ur \
  --permit @bob.pubkeys.ur
```

Every placeholder UR (for example `ur:xid/Alice` or `ur:provenance/Mark0`) represents a concrete object emitted by its respective tool. The intent for `clubs-cli` is to bind these building blocks into a cohesive single-publisher workflow while staying interoperable with the broader Gordian ecosystem.
