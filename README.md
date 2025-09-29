# `clubs-cli` (`clubs`)

`clubs-cli` is a command-line interface for working with Blockchain Commons’ Gordian Clubs. It orchestrates the underlying [`clubs`](../clubs) crate to compose editions, attach permits, verify signatures, check provenance continuity, and demonstrate decryption paths.

> **Status:** the tool is not published on crates.io. Build it from the git workspace. The crate expects the `clubs` library source to be available via a sibling path because Cargo manifests currently reference `../clubs` directly.

## Usage overview

The CLI currently focuses on single-publisher workflows and provides the following subcommands:

- `clubs init` – convenience wrapper for producing the first edition of a club.
- `clubs edition compose` – general-purpose edition composer for subsequent releases.
- `clubs edition verify` – signature and provenance checks for a single edition.
- `clubs edition permits` – extract sealed member permits from an edition.
- `clubs edition sequence` – prove that a set of editions belong to the same club and form a contiguous provenance chain.
- `clubs content decrypt` – recover plaintext content using a permit, SSKR shards, or symmetric key.

Run `clubs --help` or `clubs <command> --help` for full flag listings.

## Getting started

Start by examining the `demo-log.md` file produced by the demo script. It illustrates a complete end-to-end scenario using the CLI.

## Demonstration script

The `clubs-cli/clubs-demo.py` script drives the installed CLI together with the other Gordian tools (notably `envelope` and `provenance`). It produces a Markdown transcript (`demo-log.md`) that documents a reproducible end-to-end scenario:

- Generate publisher and member key material.
- Author initial club content, compute its digest, and bind it to a genesis provenance mark.
- Compose and verify the genesis edition with both permit styles.
- Advance the provenance chain, compose a subsequent edition, and validate the continuity using `clubs edition sequence`.

Running the script requires the `provenance` and `envelope` tools to be on `PATH`:

```bash
cd clubs-cli
./clubs-demo.py > demo-log.md
```
