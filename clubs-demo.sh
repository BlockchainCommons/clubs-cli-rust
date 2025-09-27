#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_DIR="$SCRIPT_DIR/clubs-demo"
PROV_DIR="$DEMO_DIR/provenance-chain"

step() {
  printf '\n▶ %s\n' "$1"
}

for cmd in seedtool envelope provenance cargo; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
done

step "Preparing demo workspace at $DEMO_DIR"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"

step "Generating deterministic publisher seed with seedtool"
seedtool --deterministic=CLUBS-DEMO --out seed | tee "$DEMO_DIR/publisher.seed.ur"

PUBLISHER_SEED="$(cat "$DEMO_DIR/publisher.seed.ur")"

step "Deriving publisher signing material via envelope generate"
envelope generate prvkeys --seed "$PUBLISHER_SEED" | tee "$DEMO_DIR/publisher.prvkeys.ur"
envelope xid new "$(cat "$DEMO_DIR/publisher.prvkeys.ur")" | tee "$DEMO_DIR/publisher.xid.ur"
envelope format "$(cat "$DEMO_DIR/publisher.xid.ur")" | tee "$DEMO_DIR/publisher.xid.format.txt"

declare -a PARTICIPANTS=(
  "alice:ALICE-DEMO"
  "bob:BOB-DEMO"
)

for entry in "${PARTICIPANTS[@]}"; do
  IFS=":" read -r name seed_tag <<<"$entry"
  upper_name=$(printf '%s' "$name" | tr '[:lower:]' '[:upper:]')
  step "Creating XID document for $upper_name"
  seedtool --deterministic="$seed_tag" --out seed | tee "$DEMO_DIR/$name.seed.ur"
  envelope generate prvkeys --seed "$(cat "$DEMO_DIR/$name.seed.ur")" | tee "$DEMO_DIR/$name.prvkeys.ur"
  envelope generate pubkeys "$(cat "$DEMO_DIR/$name.prvkeys.ur")" | tee "$DEMO_DIR/$name.pubkeys.ur"
  envelope xid new "$(cat "$DEMO_DIR/$name.prvkeys.ur")" | tee "$DEMO_DIR/$name.xid.ur"
  envelope format "$(cat "$DEMO_DIR/$name.xid.ur")" | tee "$DEMO_DIR/$name.xid.format.txt"
done

step "Assembling edition content envelope"
CONTENT_CLEAR_UR="$(
  envelope subject type string "Welcome to the Gordian Club!" |
    envelope assertion add pred-obj string "title" string "Genesis Edition"
)"
printf '%s\n' "$CONTENT_CLEAR_UR" | tee "$DEMO_DIR/content.clear.env.ur"
CONTENT_UR="$(envelope subject type wrapped "$CONTENT_CLEAR_UR")"
printf '%s\n' "$CONTENT_UR" | tee "$DEMO_DIR/content.env.ur"
envelope format "$CONTENT_CLEAR_UR" | tee "$DEMO_DIR/content.clear.format.txt"
envelope format "$CONTENT_UR" | tee "$DEMO_DIR/content.format.txt"

step "Deriving deterministic provenance seed for the publisher"
seedtool --deterministic=PROVENANCE-DEMO --count 32 --out seed | tee "$DEMO_DIR/provenance-seed.ur"
PROV_SEED_UR="$(cat "$DEMO_DIR/provenance-seed.ur")"

step "Starting provenance mark chain"
provenance new "$PROV_DIR" --seed "$PROV_SEED_UR" --comment "Genesis edition" | tee "$DEMO_DIR/provenance-new.log"
provenance print "$PROV_DIR" --start 0 --end 0 --format markdown | tee "$DEMO_DIR/provenance-genesis.txt"
provenance print "$PROV_DIR" --start 0 --end 0 --format ur | tee "$DEMO_DIR/genesis-mark.ur"

step "Composing genesis edition with clubs-cli"
COMPOSE_LOG="$DEMO_DIR/clubs-edition-init.log"
if RUSTFLAGS='-C debug-assertions=no' cargo run -p clubs-cli -- init \
  --publisher "@$DEMO_DIR/publisher.xid.ur" \
  --content "@$DEMO_DIR/content.env.ur" \
  --provenance "@$DEMO_DIR/genesis-mark.ur" \
  --permit "@$DEMO_DIR/alice.xid.ur" \
  --permit "@$DEMO_DIR/bob.pubkeys.ur" \
  --sskr 2of3 \
  --summary \
  --out-dir "$DEMO_DIR" \
  2>&1 | tee "$COMPOSE_LOG"; then
  step "Edition composed successfully. URs written to $DEMO_DIR"

  EDITION_FILE="$DEMO_DIR/edition.ur"
  EDITION_UR="$(cat "$EDITION_FILE")"

  step "Inspecting composed edition"
  INSPECT_STDOUT=$(
    RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
      edition inspect \
      --edition "@$EDITION_FILE" \
      --publisher "@$DEMO_DIR/publisher.xid.ur" \
      --summary \
      --emit-permits \
      2> >(tee "$DEMO_DIR/clubs-edition-inspect.log" >&2)
  )
  INSPECT_FILE="$DEMO_DIR/clubs-edition-inspect.out"
  printf '%s\n' "$INSPECT_STDOUT" | tee "$INSPECT_FILE"
  if [[ -s "$INSPECT_FILE" ]]; then
    head -n1 "$INSPECT_FILE" >"$DEMO_DIR/edition.normalized.ur"
    tail -n +2 "$INSPECT_FILE" | awk -v dir="$DEMO_DIR" 'NF { printf "%s\n", $0 > sprintf("%s/permit-%d.ur", dir, NR) }'
  fi

  step "Decrypting content via SSKR shares"
  SSKR_STDOUT=$(
    RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
      content decrypt \
      --edition "@$EDITION_FILE" \
      --publisher "@$DEMO_DIR/publisher.xid.ur" \
      --sskr "@$DEMO_DIR/sskr-share-g1-1.ur" \
      --sskr "@$DEMO_DIR/sskr-share-g1-2.ur" \
      --emit-ur \
      2> >(tee "$DEMO_DIR/clubs-content-sskr.log" >&2)
  )
  printf '%s\n' "$SSKR_STDOUT" | tee "$DEMO_DIR/content.from-sskr.ur"
  envelope format "$SSKR_STDOUT" | tee "$DEMO_DIR/content.from-sskr.format.txt"

  step "Decrypting content with Alice's permit"
  PERMIT_STDOUT=""
  for permit in "$DEMO_DIR"/permit-*.ur; do
    [[ -f "$permit" ]] || continue
    if PERMIT_STDOUT=$(
      RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
        content decrypt \
        --edition "@$EDITION_FILE" \
        --publisher "@$DEMO_DIR/publisher.xid.ur" \
        --permit "@${permit}" \
        --identity "@$DEMO_DIR/alice.prvkeys.ur" \
        --emit-ur \
        2> >(tee "$DEMO_DIR/clubs-content-permit.log" >&2)
    ); then
      printf '%s\n' "$PERMIT_STDOUT" | tee "$DEMO_DIR/content.from-permit.ur"
      envelope format "$PERMIT_STDOUT" | tee "$DEMO_DIR/content.from-permit.format.txt"
      break
    fi
  done
  if [[ -z "$PERMIT_STDOUT" ]]; then
    echo "Failed to decrypt content with Alice's permit" >&2
  fi
else
  step "clubs-cli is not yet capable of composing editions"
  echo "Inspect $COMPOSE_LOG for the error output." >&2
fi
