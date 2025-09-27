#!/usr/bin/env python3
"""Run the clubs demo and print each step with command and output in Markdown."""

from __future__ import annotations

import os
import shlex
import subprocess
import sys
import textwrap
from pathlib import Path

BOX_PREFIX = "│ "


def run_step(title: str, command: str, commentary: str | None = None) -> None:
    """Execute *command* (a bash snippet) and render the result in Markdown."""

    dedented = textwrap.dedent(command).strip()
    display_command = sanitize_command(dedented)

    print(f"## {title}\n")
    if commentary:
        print(f"{commentary}\n")

    print("```")
    print(display_command)

    try:
        result = subprocess.run(
            ["bash", "-lc", dedented],
            capture_output=True,
            text=True,
            check=True,
            cwd=SCRIPT_DIR,
            env=ENV,
        )
        output = (result.stdout + result.stderr).rstrip("\n")
    except subprocess.CalledProcessError as error:
        output = ((error.stdout or "") + (error.stderr or "")).rstrip("\n")
        if output:
            print("")
            for line in output.splitlines():
                print(f"{BOX_PREFIX}{line}")
        print("```")
        print("")
        raise SystemExit(error.returncode) from error

    if output:
        print("")
        for line in output.splitlines():
            print(f"{BOX_PREFIX}{line}")

    print("```")
    print("")


def qp(path: Path) -> str:
    """Shell-quote a filesystem path."""

    return shlex.quote(rel(path))


def rel(path: Path) -> str:
    """Return *path* relative to the script directory when possible."""

    try:
        return str(path.relative_to(SCRIPT_DIR))
    except ValueError:
        return str(path)


def sanitize_command(command: str) -> str:
    display = command
    for abs_path, rel_path in PATH_REPLACEMENTS:
        display = display.replace(abs_path, rel_path)
    return display


def main() -> None:
    run_step(
        "Checking prerequisites",
        "for cmd in seedtool envelope provenance cargo; do command -v \"$cmd\"; done",
    )

    run_step(
        "Preparing demo workspace",
        f"rm -rf {qp(DEMO_DIR)} && mkdir -p {qp(DEMO_DIR)}",
    )

    run_step(
        "Generating deterministic publisher seed",
        f"seedtool --deterministic=CLUBS-DEMO --out seed | tee {qp(PUBLISHER_SEED)}",
    )

    run_step(
        "Deriving publisher signing material",
        f"""
        envelope generate prvkeys --seed "$(cat {qp(PUBLISHER_SEED)})" | tee {qp(PUBLISHER_PRVKEYS)} && \
        envelope xid new "$(cat {qp(PUBLISHER_PRVKEYS)})" | tee {qp(PUBLISHER_XID)} && \
        envelope format "$(cat {qp(PUBLISHER_XID)})" | tee {qp(PUBLISHER_XID_FORMAT)}
        """,
    )

    for name, seed_tag in PARTICIPANTS:
        upper = name.upper()
        run_step(
            f"Creating XID document for {upper}",
            f"""
            seedtool --deterministic={seed_tag} --out seed | tee {qp(SEED_FILES[name])} && \
            envelope generate prvkeys --seed "$(cat {qp(SEED_FILES[name])})" | tee {qp(PRVKEY_FILES[name])} && \
            envelope generate pubkeys "$(cat {qp(PRVKEY_FILES[name])})" | tee {qp(PUBKEY_FILES[name])} && \
            envelope xid new "$(cat {qp(PRVKEY_FILES[name])})" | tee {qp(XID_FILES[name])} && \
            envelope format "$(cat {qp(XID_FILES[name])})" | tee {qp(XID_FORMAT_FILES[name])}
            """,
        )

    run_step(
        "Assembling edition content envelope",
        f"""
        CONTENT_CLEAR_UR=$( \
          envelope subject type string "Welcome to the Gordian Club!" | \
            envelope assertion add pred-obj string "title" string "Genesis Edition" \
        ) && \
        printf '%s\n' "$CONTENT_CLEAR_UR" | tee {qp(CONTENT_CLEAR)} && \
        CONTENT_UR=$(envelope subject type wrapped "$CONTENT_CLEAR_UR") && \
        printf '%s\n' "$CONTENT_UR" | tee {qp(CONTENT_WRAPPED)} && \
        envelope format "$CONTENT_CLEAR_UR" | tee {qp(CONTENT_CLEAR_FORMAT)} && \
        envelope format "$CONTENT_UR" | tee {qp(CONTENT_FORMAT)}
        """,
    )

    run_step(
        "Deriving deterministic provenance seed",
        f"seedtool --deterministic=PROVENANCE-DEMO --count 32 --out seed | tee {qp(PROVENANCE_SEED)}",
    )

    run_step(
        "Starting provenance mark chain",
        f"""
        provenance new {qp(PROV_DIR)} --seed "$(cat {qp(PROVENANCE_SEED)})" --comment "Genesis edition" | tee {qp(PROVENANCE_NEW_LOG)} && \
        provenance print {qp(PROV_DIR)} --start 0 --end 0 --format markdown | tee {qp(PROVENANCE_GENESIS)} && \
        provenance print {qp(PROV_DIR)} --start 0 --end 0 --format ur | tee {qp(GENESIS_MARK)}
        """,
    )

    run_step(
        "Composing genesis edition",
        f"""
        RUSTFLAGS='-C debug-assertions=no' cargo run -p clubs-cli -- init \
          --publisher "@{PUBLISHER_XID}" \
          --content "@{CONTENT_WRAPPED}" \
          --provenance "@{GENESIS_MARK}" \
          --permit "@{XID_FILES['alice']}" \
          --permit "@{PUBKEY_FILES['bob']}" \
          --sskr 2of3 \
          --summary \
          --out-dir {qp(DEMO_DIR)} \
          2>&1 | tee {qp(COMPOSE_LOG)}
        """,
    )

    run_step(
        "Inspecting composed edition",
        f"""
        INSPECT_STDOUT=$( \
          RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
            edition inspect \
            --edition "@{EDITION_FILE}" \
            --publisher "@{PUBLISHER_XID}" \
            --summary \
            --emit-permits \
            2> >(tee {qp(EDITION_INSPECT_LOG)} >&2) \
        ) && \
        printf '%s\n' "$INSPECT_STDOUT" | tee {qp(EDITION_INSPECT_OUT)} && \
        if [ -s {qp(EDITION_INSPECT_OUT)} ]; then \
          head -n1 {qp(EDITION_INSPECT_OUT)} > {qp(EDITION_NORMALIZED)}; \
          tail -n +2 {qp(EDITION_INSPECT_OUT)} | awk -v dir={qp(DEMO_DIR)} 'NF {{ printf "%s\\n", $0 > sprintf("%s/permit-%d.ur", dir, NR) }}'; \
        fi
        """,
    )

    run_step(
        "Decrypting content via SSKR shares",
        f"""
        SSKR_STDOUT=$( \
          RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
            content decrypt \
            --edition "@{EDITION_FILE}" \
            --publisher "@{PUBLISHER_XID}" \
            --sskr "@{SSKR_SHARES[0]}" \
            --sskr "@{SSKR_SHARES[1]}" \
            --emit-ur \
            2> >(tee {qp(CONTENT_SSKR_LOG)} >&2) \
        ) && \
        printf '%s\n' "$SSKR_STDOUT" | tee {qp(CONTENT_FROM_SSKR)} && \
        envelope format "$SSKR_STDOUT" | tee {qp(CONTENT_FROM_SSKR_FORMAT)}
        """,
    )

    run_step(
        "Decrypting content with Alice's permit",
        f"""
        PERMIT_STDOUT="" && \
        for permit in {qp(DEMO_DIR)}/permit-*.ur; do \
          [ -f "$permit" ] || continue; \
          if PERMIT_STDOUT=$( \
              RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- \
                content decrypt \
                --edition "@{EDITION_FILE}" \
                --publisher "@{PUBLISHER_XID}" \
                --permit "@$permit" \
                --identity "@{PRVKEY_FILES['alice']}" \
                --emit-ur \
                2>&1 | tee {qp(CONTENT_PERMIT_LOG)} \
            ); then \
            printf '%s\n' "$PERMIT_STDOUT" | tee {qp(CONTENT_FROM_PERMIT)} && \
            envelope format "$PERMIT_STDOUT" | tee {qp(CONTENT_FROM_PERMIT_FORMAT)} && \
            break; \
          fi; \
        done && \
        if [ -z "$PERMIT_STDOUT" ]; then \
          echo "Failed to decrypt content with Alice's permit" >&2; \
          exit 1; \
        fi
        """,
    )


SCRIPT_DIR = Path(__file__).resolve().parent
DEMO_DIR = SCRIPT_DIR / "clubs-demo"
PROV_DIR = DEMO_DIR / "provenance-chain"

PUBLISHER_SEED = DEMO_DIR / "publisher.seed.ur"
PUBLISHER_PRVKEYS = DEMO_DIR / "publisher.prvkeys.ur"
PUBLISHER_XID = DEMO_DIR / "publisher.xid.ur"
PUBLISHER_XID_FORMAT = DEMO_DIR / "publisher.xid.format.txt"

SEED_FILES = {
    "alice": DEMO_DIR / "alice.seed.ur",
    "bob": DEMO_DIR / "bob.seed.ur",
}
PRVKEY_FILES = {
    "alice": DEMO_DIR / "alice.prvkeys.ur",
    "bob": DEMO_DIR / "bob.prvkeys.ur",
}
PUBKEY_FILES = {
    "alice": DEMO_DIR / "alice.pubkeys.ur",
    "bob": DEMO_DIR / "bob.pubkeys.ur",
}
XID_FILES = {
    "alice": DEMO_DIR / "alice.xid.ur",
    "bob": DEMO_DIR / "bob.xid.ur",
}
XID_FORMAT_FILES = {
    "alice": DEMO_DIR / "alice.xid.format.txt",
    "bob": DEMO_DIR / "bob.xid.format.txt",
}

CONTENT_CLEAR = DEMO_DIR / "content.clear.env.ur"
CONTENT_CLEAR_FORMAT = DEMO_DIR / "content.clear.format.txt"
CONTENT_WRAPPED = DEMO_DIR / "content.env.ur"
CONTENT_FORMAT = DEMO_DIR / "content.format.txt"

PROVENANCE_SEED = DEMO_DIR / "provenance-seed.ur"
PROVENANCE_NEW_LOG = DEMO_DIR / "provenance-new.log"
PROVENANCE_GENESIS = DEMO_DIR / "provenance-genesis.txt"
GENESIS_MARK = DEMO_DIR / "genesis-mark.ur"

COMPOSE_LOG = DEMO_DIR / "clubs-edition-init.log"
EDITION_FILE = DEMO_DIR / "edition.ur"
EDITION_INSPECT_LOG = DEMO_DIR / "clubs-edition-inspect.log"
EDITION_INSPECT_OUT = DEMO_DIR / "clubs-edition-inspect.out"
EDITION_NORMALIZED = DEMO_DIR / "edition.normalized.ur"

SSKR_SHARES = [
    DEMO_DIR / "sskr-share-g1-1.ur",
    DEMO_DIR / "sskr-share-g1-2.ur",
]

CONTENT_SSKR_LOG = DEMO_DIR / "clubs-content-sskr.log"
CONTENT_FROM_SSKR = DEMO_DIR / "content.from-sskr.ur"
CONTENT_FROM_SSKR_FORMAT = DEMO_DIR / "content.from-sskr.format.txt"
CONTENT_PERMIT_LOG = DEMO_DIR / "clubs-content-permit.log"
CONTENT_FROM_PERMIT = DEMO_DIR / "content.from-permit.ur"
CONTENT_FROM_PERMIT_FORMAT = DEMO_DIR / "content.from-permit.format.txt"

PARTICIPANTS = (
    ("alice", "ALICE-DEMO"),
    ("bob", "BOB-DEMO"),
)

PATH_OBJECTS = {
    DEMO_DIR,
    PROV_DIR,
    PUBLISHER_SEED,
    PUBLISHER_PRVKEYS,
    PUBLISHER_XID,
    PUBLISHER_XID_FORMAT,
    *SEED_FILES.values(),
    *PRVKEY_FILES.values(),
    *PUBKEY_FILES.values(),
    *XID_FILES.values(),
    *XID_FORMAT_FILES.values(),
    CONTENT_CLEAR,
    CONTENT_CLEAR_FORMAT,
    CONTENT_WRAPPED,
    CONTENT_FORMAT,
    PROVENANCE_SEED,
    PROVENANCE_NEW_LOG,
    PROVENANCE_GENESIS,
    GENESIS_MARK,
    COMPOSE_LOG,
    EDITION_FILE,
    EDITION_INSPECT_LOG,
    EDITION_INSPECT_OUT,
    EDITION_NORMALIZED,
    *SSKR_SHARES,
    CONTENT_SSKR_LOG,
    CONTENT_FROM_SSKR,
    CONTENT_FROM_SSKR_FORMAT,
    CONTENT_PERMIT_LOG,
    CONTENT_FROM_PERMIT,
    CONTENT_FROM_PERMIT_FORMAT,
}

PATH_REPLACEMENTS = []
for path in PATH_OBJECTS:
    abs_path = str(path.resolve())
    rel_path = rel(path)
    PATH_REPLACEMENTS.append((abs_path, rel_path))
    PATH_REPLACEMENTS.append((shlex.quote(abs_path), rel_path))
    PATH_REPLACEMENTS.append((f"@{abs_path}", f"@{rel_path}"))
    PATH_REPLACEMENTS.append((f"@{shlex.quote(abs_path)}", f"@{rel_path}"))

ENV = os.environ.copy()


if __name__ == "__main__":
    try:
        main()
    except SystemExit as exc:
        sys.exit(exc.code)
