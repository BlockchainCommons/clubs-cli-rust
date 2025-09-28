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

- Make sure you understand `queries.rs` and examine the tests in the `bc-envelope` crate for many functional examples.
- Make sure you understand traits like `URDecodable`, `UREncodable`, `URCodable`, `EnvelopeEncodable`, `EnvelopeDecodable`, `CBOREncodable`, `CBORDecodable`, `CBORCodable`, `CBORTaggedEncodable`, `CBORTaggedDecodable`, `CBORTaggedCodable`, and the blanket implementations for these types, which are frequently linked to `From`/`TryFrom` implementations.

## Current Task

Implement the new `clubs-cli` tool. Focus first on single-publisher clubs and reuse functionality already available in sibling CLIs:

- XID documents are created through `envelope-cli`.
- `provenance-mark-cli` manages provenance chains; do not duplicate.
- `envelope-cli` is responsible for preparing `content` envelopes.
- `clubs-cli` composes new `Edition`s by optionally encrypting content, adding permits (public-key and SSKR), and signing with the publisher’s keys from the XID document.
- All inputs and outputs must be URs; leverage the UR-aware types across the workspace.

The command line tools `provenance`, `envelope`, and others from this workspace are installed and can be run to generate test vectors and understand expected workflows.

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

## Progress Update (2025-09-27)

### Workflow implementation status
- Implemented `clubs-cli edition compose` and `clubs-cli init`, wiring publisher XID documents (including private keys) into the composer, enforcing genesis/previous-mark rules, and emitting edition URs plus optional SSKR shards.
- Added `clubs-cli/clubs-demo.py`, a reproducible walkthrough that regenerates deterministic seeds/XID docs, assembles content, advances provenance, and exercises the new compose pipeline. The script now lives beside the binary, writes outputs into `clubs-cli/demo/`, and demonstrates the current limitations (inspection/decrypt commands still pending).
- Composer summary output documents the recipients, SSKR spec, provenance sequence, and where artifacts are written; stdout stays machine-friendly (UR per line) for downstream tooling.

### Envelope digest expectations
- Envelopes carry a stable digest that survives obscuring operations (encryption, compression, elision) because the digest is committed into the obscured representation and AAD.
- Wrapping *does* change the digest; therefore, edition content must arrive as an already wrapped envelope with **no additional assertions** so the digest we bind into permits and provenance remains stable after encryption.
- The composer currently wraps cleartext content on ingest if necessary, but upcoming validation will reject content that carries extra assertions or is unwrapped to align with the stability requirement. CLI now enforces this precondition and surfaces an actionable error if assertions remain.

### Provenance tooling integration
- Extended `provenance` (new/next) with shared `--info-hex`/`--info-ur[ --info-ur-tag ]` parsing so marks can embed dCBOR payloads (e.g., the content digest). Known UR types automatically derive their registered CBOR tags; unknown types require an explicit tag override.
- These improvements let us feed the wrapped-content digest (as `ur:digest/...` or hex CBOR) into the provenance mark `info` field without building ad hoc plumbing inside `clubs-cli`.
- JSON mark snapshots continue to store the `info` payload in base64, while the CLI accepts both base64 and hex inputs for convenience.

## Progress Update (2025-09-28)

### CLI coverage
- Implemented `clubs edition inspect` with optional `--publisher/--verifier` input. The command now verifies signatures, enforces XID alignment, emits structured summaries, and can re-print normalized edition and permit URs via `--emit-permits`.
- Added `clubs content decrypt`, supporting three recovery paths: direct symmetric key, public-key permits (when paired with `--identity` private material), and SSKR shares. The command reuses the verifier flow from `inspect`, cross-checks results when multiple inputs are provided, and reports the recovery sources plus content digest.
- Completed `clubs permits derive`; it emits a `PublicKeyPermit` descriptor envelope (`type: "PublicKeyPermit"`, subject `PublicKeys`, optional `holder` assertion) that `edition compose` now accepts alongside raw XID docs or public keys.
- Fixed public-key permit decryption: `content decrypt` now parses tagged symmetric keys correctly and the demo script confirms recovery via Alice's permit.

### I/O helpers
- `io::RecipientDescriptor` can now retain an annotated member XID (even without a full XID document) and shares it via `member_xid()`.
- Added parsing for permit envelopes, private-key URs/XID documents, and bare XID strings (UR form or `XID(...)`). These are reused across `compose`, `inspect`, `content decrypt`, and `permits derive`.
- Brought in `dcbor` to decode symmetric keys carried inside sealed messages when unwrapping public-key permits.

### Demo workflow
- `clubs-demo.py` now wraps the content envelope before composition, runs `edition inspect` to harvest permit URs, and exercises both SSKR- and permit-based decryptions. Resulting URs plus formatted envelopes are saved under `demo/` for reference.
