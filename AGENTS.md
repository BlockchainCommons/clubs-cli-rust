# Overview

For minimal context, read these files:

- ../clubs/docs/XanaduToReality.md
- ../clubs/docs/PublicKeyPermits.md
- ../clubs/docs/FrostProvenanceMarks.md
- ../clubs/docs/dcbor-draft.md
- ../clubs/docs/envelope-links.md

Workspace crates that provide key APIs:

- ../dcbor/
- ../bc-envelope/
- ../bc-xid/
- ../bc-components/
- ../bc-ur/
- ../known-values/
- ../provenance-mark/

## Current Task

Implement the new `clubs-cli` tool. Focus first on single-publisher clubs and reuse functionality already available in sibling CLIs:

- XID documents are created through `envelope-cli`.
- `provenance-mark-cli` manages provenance chains; do not duplicate.
- `envelope-cli` is responsible for preparing `content` envelopes.
- `clubs-cli` composes new `Edition`s by optionally encrypting content, adding permits (public-key and SSKR), and signing with the publisher’s keys from the XID document.
- All inputs and outputs must be URs; leverage the UR-aware types across the workspace.

## `clubs-cli` Development Plan

### Objectives
- Support single-publisher clubs end-to-end: compose, sign, and distribute editions without re-implementing XID, provenance mark, or content authoring flows covered by existing tooling.
- Consume and emit `UR` strings exclusively for interoperability with Gordian tools, including `envelope-cli`, `provenance-mark-cli`, and `provenance-mark`-aware viewers.
- Establish a modular command surface that can grow toward multi-publisher/FROST workflows without refactoring the initial release.

### Core Workflows (Single Publisher)
- **Genesis publication**: accept the publisher’s XID document (with private keys), a genesis `ProvenanceMark`, and a content `Envelope`; seal without permits or with explicitly supplied permits; output the signed edition envelope UR plus any generated SSKR share URs.
- **Subsequent edition**: same flow, but require the caller to provide the previous edition’s UR (or at least its provenance mark) so the CLI can confirm `precedes` ordering before sealing.
- **Verification & inspection**: given an edition UR, verify signature against the embedded club XID, list permit recipients, and optionally decrypt content when provided the correct secrets (content key via UR, SSKR joins, or permit decryption).

### Command Surface for v0
- `clubs-cli init` – convenience wrapper for creating the first edition. Validates that the supplied provenance mark is genesis, refuses `--previous` input, and delegates to the generic composer.
- `clubs-cli edition compose` – core command that loads inputs, applies requested permits, handles optional encryption (defaulting to encrypt when any permit or SSKR spec is present), and signs with the publisher’s XID private key. Emits:
  - edition UR (first line)
  - zero or more `ur:sskr` shards (subsequent lines, sorted deterministically)
- `clubs-cli edition inspect` – parses an edition UR, verifies the signature, outputs a human-readable summary (to stderr) and optionally re-emits normalized URs for the edition and attached permits.
- `clubs-cli permits derive` – helper that converts recipient material (XID document or public-keys UR) into a `PublicKeyPermit` UR for reuse across editions.
- `clubs-cli content decrypt` – optional helper that, given an edition UR plus either `ur:sealedmessage` permits, SSKR shard URs, or a raw symmetric key UR, recovers the plaintext content envelope for auditing.

### Data Handling & I/O
- **UR parsing**: centralize in an `io` module that accepts inline UR strings, `@file` indirections, or STDIN. Use `bc_ur::UR` to detect type labels (`envelope`, `sealedmessage`, `sskr`, `xiddoc`, etc.) and dispatch to the right converter (`Envelope::from_ur_string`, `XIDDocument::from_ur_string`, `PublicKeys::from_ur_string`).
- **Permit inputs**: allow `--recipient` to accept either an XID document UR (extracting the current public keys and optional XID) or a raw `PublicKeys` UR. Support optional `--label` to annotate holder XIDs when they differ from the supplied material.
- **SSKR spec**: parse `--sskr` arguments such as `2of3`, `2of3,3of5` (multiple groups). Map this to `SSKRSpec::new` and emit validation errors with actionable hints.
- **Output formatting**: default stdout to bare UR strings (edition first, then shards). Provide `--out-dir` to drop shard URs into files and `--summary` to print structured metadata to stderr without breaking UR-only stdout pipes.
- **Key management**: require the publisher XID UR to include private keys; fail fast if only public material is present.

### Implementation Roadmap
1. ✅ **Scaffolding**: add the `clubs-cli` crate to the workspace with Clap 4, anyhow, and dependencies on `clubs`, `bc-components`, `bc-envelope`, `bc-ur`, `bc-xid`, and `known-values`.
2. **Infrastructure**: implement `io` (UR parsing), `serialization` (UR + envelope helpers), and shared error/reporting utilities. Register Gordian tags at startup.
3. **Permit helpers**: create converters for recipient descriptors → `PublicKeyPermit::Encode`, including detection of XID vs public-keys URs and holder annotations.
4. **Edition composer**: wire `Edition::new`, `Edition::seal_with_permits`, and SSKR handling into the `edition compose` command. Ensure digest-based AAD binding and optional cleartext path work as in `clubs` tests.
5. **Genesis command**: wrap composer with genesis-only validation (must pass `is_genesis`, must not include `--previous`).
6. **Inspection tooling**: build `edition inspect`, returning verification status, provenance chain details (using `ProvenanceMarkProvider::precedes` when `--previous` supplied), and enumerated permits. Support optional re-encoding to canonical URs.
7. **Decryption helper**: allow decrypt command to accept one of: decrypted symmetric key UR, permit UR + private key, or SSKR shards. Reuse `PublicKeyPermit::Decode` and `Envelope::sskr_join` where possible.
8. **Documentation & examples**: draft README usage examples that chain together existing CLI tools (`envelope-cli`, `provenance-mark-cli`) with `clubs-cli` workflows. Include worked example mirroring `../clubs/tests/basic_scenario.rs`.

### Testing Strategy
- Unit-test UR parsing and permit derivation against fixtures using the rubric in `../clubs/docs/expected-text-output-rubric.md` for deterministic text expectations.
- Add integration tests with `assert_cmd` that replicate the single-publisher scenario: create deterministic keys, feed UR fixtures through the CLI, and compare resulting URs / formatted summaries to frozen outputs.
- Validate failure cases: missing private key material, mismatched provenance ordering, malformed SSKR spec, repeated permit labels.
- Include round-trip tests ensuring CLI-produced editions can be consumed by the existing `clubs` library APIs (`Edition::try_from`, `Edition::unseal`).

### Future Extensions (Post-v0 Priorities)
- Multi-publisher support: introduce FROST signing flows (coordinator + participants) reusing `clubs::frost` modules once the single-publisher skeleton is stable.
- Capability permits and adaptor signatures once the underlying crates expose production APIs.
- Provenance automation: optional command to call `provenance-mark-cli` under the hood or to manage VRF-based chains using `clubs::frost::pm`.
- Serialization profiles beyond raw URs (e.g., QR, Bytewords) when integration with mobile apps warrants it.
