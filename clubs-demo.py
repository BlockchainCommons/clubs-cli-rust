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


def run_step(
    title: str,
    commands: list[str] | tuple[str, ...] | str,
    commentary: str | None = None,
    *,
    stop_on_success: bool = False,
) -> list[str]:
    """Execute commands and render the result in Markdown."""

    if isinstance(commands, str):
        command_list = [textwrap.dedent(commands).strip()]
    else:
        command_list = [textwrap.dedent(cmd).strip() for cmd in commands]

    outputs: list[str] = []
    aggregated_lines: list[str] = []
    success = False
    last_error: subprocess.CalledProcessError | None = None
    failure_output: str = ""

    print(f"## {title}\n")
    if commentary:
        print(f"{commentary}\n")

    print("```")
    for index, command in enumerate(command_list):
        if not command:
            continue

        display_command = sanitize_command(command)
        print(display_command)

        try:
            result = subprocess.run(
                ["bash", "-lc", command],
                capture_output=True,
                text=True,
                check=True,
                cwd=SCRIPT_DIR,
                env=ENV,
            )
            output = (result.stdout + result.stderr).rstrip("\n")
            success = True
        except subprocess.CalledProcessError as error:
            output = ((error.stdout or "") + (error.stderr or "")).rstrip("\n")
            last_error = error
            if not stop_on_success:
                if output:
                    print("")
                    for line in output.splitlines():
                        print(f"{BOX_PREFIX}{line}")
                print("```")
                print("")
                raise SystemExit(error.returncode) from error
            failure_output = output
            continue
        outputs.append(output)
        if output:
            aggregated_lines.extend(output.splitlines())

        if stop_on_success and success:
            break

    if stop_on_success and not success and last_error is not None:
        if failure_output:
            print("")
            for line in failure_output.splitlines():
                print(f"{BOX_PREFIX}{line}")
        print("```")
        print("")
        raise SystemExit(last_error.returncode) from last_error

    if aggregated_lines:
        print("")
        print("\n".join(f"{BOX_PREFIX}{line}" for line in aggregated_lines))
    print("```")
    print("")

    return outputs


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
        [
            f'envelope generate prvkeys --seed "$(cat {rel(PUBLISHER_SEED)})" | tee {rel(PUBLISHER_PRVKEYS)}',
            f'envelope xid new "$(cat {rel(PUBLISHER_PRVKEYS)})" | tee {rel(PUBLISHER_XID)}',
            f'envelope format "$(cat {rel(PUBLISHER_XID)})" | tee {rel(PUBLISHER_XID_FORMAT)}',
        ],
    )

    for name, seed_tag in PARTICIPANTS:
        upper = name.upper()
        run_step(
            f"Creating XID document for {upper}",
            [
                f'seedtool --deterministic={seed_tag} --out seed | tee {rel(SEED_FILES[name])}',
                f'envelope generate prvkeys --seed "$(cat {rel(SEED_FILES[name])})" | tee {rel(PRVKEY_FILES[name])}',
                f'envelope generate pubkeys "$(cat {rel(PRVKEY_FILES[name])})" | tee {rel(PUBKEY_FILES[name])}',
                f'envelope xid new "$(cat {rel(PRVKEY_FILES[name])})" | tee {rel(XID_FILES[name])}',
                f'envelope format "$(cat {rel(XID_FILES[name])})" | tee {rel(XID_FORMAT_FILES[name])}',
            ],
        )

    run_step(
        "Assembling edition content envelope",
        [
            f'envelope subject type string "Welcome to the Gordian Club!" | tee {rel(CONTENT_SUBJECT_TMP)}',
            f'cat {rel(CONTENT_SUBJECT_TMP)} | envelope assertion add pred-obj string "title" string "Genesis Edition" | tee {rel(CONTENT_CLEAR)}',
            f'rm {rel(CONTENT_SUBJECT_TMP)}',
            f'envelope subject type wrapped "$(cat {rel(CONTENT_CLEAR)})" | tee {rel(CONTENT_WRAPPED)}',
            f'envelope format "$(cat {rel(CONTENT_CLEAR)})" | tee {rel(CONTENT_CLEAR_FORMAT)}',
            f'envelope format "$(cat {rel(CONTENT_WRAPPED)})" | tee {rel(CONTENT_FORMAT)}',
        ],
    )

    run_step(
        "Deriving deterministic provenance seed",
        f"seedtool --deterministic=PROVENANCE-DEMO --count 32 --out seed | tee {qp(PROVENANCE_SEED)}",
    )

    run_step(
        "Starting provenance mark chain",
        [
            f'provenance new {rel(PROV_DIR)} --seed "$(cat {rel(PROVENANCE_SEED)})" --comment "Genesis edition" | tee {rel(PROVENANCE_NEW_LOG)}',
            f'provenance print {rel(PROV_DIR)} --start 0 --end 0 --format markdown | tee {rel(PROVENANCE_GENESIS)}',
            f'provenance print {rel(PROV_DIR)} --start 0 --end 0 --format ur | tee {rel(GENESIS_MARK)}',
        ],
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

    inspect_output = run_step(
        "Inspecting composed edition",
        [
            (
                "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                f"edition inspect "
                f"--edition \"@{rel(EDITION_FILE)}\" "
                f"--publisher \"@{rel(PUBLISHER_XID)}\" "
                f"--summary "
                f"--emit-permits"
            ),
        ],
    )
    process_inspection_output(inspect_output[-1] if inspect_output else "")

    sskr_output = run_step(
        "Decrypting content via SSKR shares",
        [
            (
                "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                f"content decrypt "
                f"--edition \"@{rel(EDITION_FILE)}\" "
                f"--publisher \"@{rel(PUBLISHER_XID)}\" "
                f"--sskr \"@{rel(SSKR_SHARES[0])}\" "
                f"--sskr \"@{rel(SSKR_SHARES[1])}\" "
                f"--emit-ur"
            ),
        ],
    )
    process_sskr_output(sskr_output[-1] if sskr_output else "")

    run_step(
        "Formatting SSKR-recovered content",
        [
            f'envelope format "$(cat {rel(CONTENT_FROM_SSKR)})" | tee {rel(CONTENT_FROM_SSKR_FORMAT)}',
        ],
    )

    permit_commands = build_permit_commands()
    permit_outputs = run_step(
        "Decrypting content with Alice's permit",
        permit_commands,
        stop_on_success=True,
    )
    process_permit_output(permit_outputs)

    run_step(
        "Formatting permit-recovered content",
        [
            f'envelope format "$(cat {rel(CONTENT_FROM_PERMIT)})" | tee {rel(CONTENT_FROM_PERMIT_FORMAT)}',
        ],
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

CONTENT_SUBJECT_TMP = DEMO_DIR / "content.subject.tmp"
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
    CONTENT_SUBJECT_TMP,
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
    CONTENT_SSKR_LOG,
    CONTENT_FROM_SSKR,
    CONTENT_FROM_SSKR_FORMAT,
    CONTENT_PERMIT_LOG,
    CONTENT_FROM_PERMIT,
    CONTENT_FROM_PERMIT_FORMAT,
}

PATH_OBJECTS.update(SEED_FILES.values())
PATH_OBJECTS.update(PRVKEY_FILES.values())
PATH_OBJECTS.update(PUBKEY_FILES.values())
PATH_OBJECTS.update(XID_FILES.values())
PATH_OBJECTS.update(XID_FORMAT_FILES.values())
PATH_OBJECTS.update(SSKR_SHARES)

PATH_REPLACEMENTS = []
for path in PATH_OBJECTS:
    abs_path = str(path.resolve())
    rel_path = rel(path)
    PATH_REPLACEMENTS.append((abs_path, rel_path))
    PATH_REPLACEMENTS.append((shlex.quote(abs_path), rel_path))
    PATH_REPLACEMENTS.append((f"@{abs_path}", f"@{rel_path}"))
    PATH_REPLACEMENTS.append((f"@{shlex.quote(abs_path)}", f"@{rel_path}"))

ENV = os.environ.copy()


def process_inspection_output(output: str) -> None:
    all_lines = [line for line in output.splitlines() if line]
    EDITION_INSPECT_LOG.write_text(output + ("\n" if output else ""))
    EDITION_INSPECT_OUT.write_text("\n".join(all_lines) + ("\n" if all_lines else ""))

    ur_lines = [line for line in all_lines if line.startswith("ur:")]

    for permit in sorted(DEMO_DIR.glob("permit-*.ur")):
        permit.unlink()

    if not ur_lines:
        return

    EDITION_NORMALIZED.write_text(ur_lines[0] + "\n")
    for index, line in enumerate(ur_lines[1:], start=1):
        (DEMO_DIR / f"permit-{index}.ur").write_text(line + "\n")


def process_sskr_output(output: str) -> None:
    CONTENT_SSKR_LOG.write_text(output + ("\n" if output else ""))
    ur_lines = [line for line in output.splitlines() if line.startswith("ur:")]
    if not ur_lines:
        raise RuntimeError("SSKR decryption did not produce a UR")
    CONTENT_FROM_SSKR.write_text("\n".join(ur_lines) + "\n")


def build_permit_commands() -> list[str]:
    commands: list[str] = []
    for permit in sorted(DEMO_DIR.glob("permit-*.ur")):
        commands.append(
            (
                "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                f"content decrypt "
                f"--edition \"@{rel(EDITION_FILE)}\" "
                f"--publisher \"@{rel(PUBLISHER_XID)}\" "
                f"--permit \"@{rel(permit)}\" "
                f"--identity \"@{rel(PRVKEY_FILES['alice'])}\" "
                f"--emit-ur"
            )
        )
    if not commands:
        raise RuntimeError("No permit files available for decryption")
    return commands


def process_permit_output(outputs: list[str]) -> None:
    if not outputs:
        raise RuntimeError("Permit decryption produced no output")

    last_output = outputs[-1]
    CONTENT_PERMIT_LOG.write_text(last_output + ("\n" if last_output else ""))
    ur_lines = [line for line in last_output.splitlines() if line.startswith("ur:")]
    if not ur_lines:
        raise RuntimeError("Permit decryption did not produce a UR")
    CONTENT_FROM_PERMIT.write_text("\n".join(ur_lines) + "\n")

if __name__ == "__main__":
    try:
        main()
    except SystemExit as exc:
        sys.exit(exc.code)
