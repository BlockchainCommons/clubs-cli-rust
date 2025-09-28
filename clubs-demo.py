#!/usr/bin/env python3
"""Run the clubs demo and print each step with command and output in Markdown."""

from __future__ import annotations

import os
import uuid
import time
import locale
import threading
import selectors
import subprocess
import shlex
import sys
import textwrap
from pathlib import Path
from typing import Optional, Tuple

BOX_PREFIX = "│ "


class PersistentShell:
    """
    Persistent Bash shell that preserves state across commands and returns
    combined stdout+stderr with accurate exit codes.
    """

    _CTRL_FD = 9
    _RS = b"\x1e"  # ASCII Record Separator to minimize collision in user output

    def __init__(
        self,
        cwd: Optional[str] = None,
        env: Optional[dict] = None,
        *,
        bash_path: str = "bash",
        login: bool = True,
        encoding: Optional[str] = None,
        read_chunk: int = 65536,
        debug: bool = False
    ):
        """Initialize the persistent shell."""
        if os.name != "posix":
            raise OSError("PersistentShell requires a POSIX system.")

        self._encoding = encoding or locale.getpreferredencoding(False)
        self._read_chunk = int(read_chunk)
        self._lock = threading.RLock()
        self._residual = bytearray()

        # Create a dedicated pipe for control input (FD 9 on the child side)
        ctrl_r, ctrl_w = os.pipe()
        self._ctrl_r = ctrl_r
        self._ctrl_w = ctrl_w

        # Build the bootstrap script
        bootstrap = f"""\
# PersistentShell bootstrap (executed via: bash -lc '<this script>')
# stdin is already /dev/null from the parent; do not touch FD 0 here.

# ── Debug channel on FD 200 (default: silent) ──────────────────────────────────
if [[ -n "${{PSH_DEBUG_FILE:-}}" ]]; then
  exec 200>>"${{PSH_DEBUG_FILE}}" || {{ echo "PSH: cannot open ${{PSH_DEBUG_FILE}}" >&2; exit 95; }}
elif [[ -n "${{PSH_DEBUG:-}}" ]]; then
  exec 200>/dev/stderr
else
  exec 200>/dev/null
fi

# ── Control FD: duplicate the inherited FD to 9 and close the original ─────────
exec 9<&{ctrl_r} || {{ echo "PSH: dup {ctrl_r} -> 9 failed" >&200; exit 97; }}
exec {ctrl_r}<&- || true

# Sanitize prompts/hooks; keep normal bash semantics (no `set -e`)
PS1=; PS2=; PROMPT_COMMAND=

# Optional xtrace to FD 200
if [[ -n "${{PSH_DEBUG:-}}" ]]; then
  export BASH_XTRACEFD=200
  set -x
fi

# Helpful traps to see unexpected exits/signals in debug mode
trap 'rc=$?; echo "PSH: bootstrap exiting rc=$rc" >&200' EXIT
trap 'echo "PSH: got signal" >&200' HUP INT TERM

# ── Main loop: read two NUL‑terminated fields (token, command) from FD 9 ───────
while IFS= read -r -d $'\\0' -u 9 __psh_token; do
  if ! IFS= read -r -d $'\\0' -u 9 __psh_cmd; then
    # Partial frame: return a special exit code (98) for this token
    printf '\\x1ePSHEXIT:%s:%d\\x1e\\n' "$__psh_token" 98
    continue
  fi

  # Execute in the *current* shell so env, cwd, aliases, functions persist
  builtin eval -- "$__psh_cmd"
  __psh_status=$?

  # Emit sentinel to FD 1 (stderr already merged by parent)
  builtin printf '\\x1ePSHEXIT:%s:%d\\x1e\\n' "$__psh_token" "$__psh_status"
done

# Clean EOF on control FD
exit 0
"""

        # Compose bash argv
        argv = [bash_path]
        if login:
            argv.append("-l")
        argv += ["-c", bootstrap]

        # Set up environment with debug options if requested
        shell_env = env.copy() if env else os.environ.copy()
        if debug:
            shell_env["PSH_DEBUG"] = "1"

        # Start Bash
        self._proc = subprocess.Popen(
            argv,
            stdin=subprocess.DEVNULL,  # Important: don't use stdin for bootstrap
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            cwd=cwd,
            env=shell_env,
            bufsize=0,
            close_fds=True,
            pass_fds=(self._ctrl_r,),
            text=False
        )

        # Prepare a selector to read from the combined output pipe efficiently
        self._sel = selectors.DefaultSelector()
        if self._proc.stdout is None:
            raise RuntimeError("Failed to create pipes for persistent shell.")
        self._sel.register(self._proc.stdout, selectors.EVENT_READ)

        # Don't write bootstrap to stdin - it's handled by -c argument
        self._ctrl_wf = os.fdopen(self._ctrl_w, "wb", buffering=0)

    def _assert_alive(self):
        if self._proc.poll() is not None:
            raise RuntimeError(f"Persistent shell has exited with code {self._proc.returncode}.")

    def _write_frame(self, token: str, command: str):
        try:
            self._ctrl_wf.write(token.encode("utf-8") + b"\x00" +
                                command.encode("utf-8") + b"\x00")
            self._ctrl_wf.flush()
        except BrokenPipeError:
            raise RuntimeError("Persistent shell control channel closed.")

    def _read_until_sentinel(self, token: str, timeout: Optional[float]) -> Tuple[bytes, int]:
        """Read combined output until we see the sentinel for this token."""
        self._assert_alive()

        token_b = token.encode("utf-8")
        prefix = self._RS + b"PSHEXIT:" + token_b + b":"
        suffix = self._RS + b"\n"

        buf = bytearray()
        if self._residual:
            buf += self._residual
            self._residual = bytearray()

        end_time = (time.monotonic() + timeout) if timeout else None

        def time_left():
            if end_time is None:
                return None
            return max(0.0, end_time - time.monotonic())

        while True:
            # Check if sentinel is already in buffer
            idx = buf.find(prefix)
            if idx != -1:
                after = buf[idx + len(prefix):]
                j = after.find(suffix)
                if j != -1:
                    exit_bytes = after[:j]
                    try:
                        exit_code = int(exit_bytes.decode("ascii", "strict"))
                    except Exception:
                        raise RuntimeError("Malformed sentinel from persistent shell.")
                    before = bytes(buf[:idx])
                    remaining = bytes(after[j + len(suffix):])
                    self._residual.extend(remaining)
                    return before, exit_code

            # Need to read more
            self._assert_alive()
            timeout_this = time_left()
            events = self._sel.select(timeout_this)
            if not events:
                if end_time is not None and time.monotonic() >= end_time:
                    raise TimeoutError("Timed out waiting for command to complete.")
                continue

            for key, _ in events:
                if self._proc.stdout:
                    chunk = self._proc.stdout.read(self._read_chunk)
                    if chunk is None:
                        continue
                    if chunk == b"":
                        raise RuntimeError("Shell terminated unexpectedly while reading output.")
                    buf.extend(chunk)

    def run_command(self, command: str, *, timeout: Optional[float] = None) -> Tuple[str, int]:
        """Execute a command in the persistent shell and return (combined_output, exit_code)."""
        if "\x00" in command:
            raise ValueError("Command may not contain NUL characters.")

        with self._lock:
            self._assert_alive()
            token = uuid.uuid4().hex
            self._write_frame(token, command)
            out_bytes, exit_code = self._read_until_sentinel(token, timeout)
            output = out_bytes.decode(self._encoding, errors="replace")
            return output, exit_code

    def close(self):
        """Cleanly shut down the shell process."""
        with self._lock:
            try:
                if hasattr(self, "_ctrl_wf") and self._ctrl_wf:
                    self._ctrl_wf.close()
            except Exception:
                pass
            try:
                if self._proc.poll() is None:
                    try:
                        self._proc.wait(timeout=2.0)
                    except subprocess.TimeoutExpired:
                        self._proc.terminate()
                        try:
                            self._proc.wait(timeout=2.0)
                        except subprocess.TimeoutExpired:
                            self._proc.kill()
            finally:
                try:
                    if self._proc.stdout:
                        self._sel.unregister(self._proc.stdout)
                except Exception:
                    pass
                try:
                    if self._proc.stdout:
                        self._proc.stdout.close()
                except Exception:
                    pass
                try:
                    os.close(self._ctrl_r)
                except Exception:
                    pass

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        self.close()


def run_step(
    shell: PersistentShell,
    title: str,
    commands: list[str] | tuple[str, ...] | str,
    commentary: str | None = None,
    *,
    stop_on_success: bool = False,
) -> list[str]:
    """Execute commands using persistent shell and render the result in Markdown."""

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
            output, exit_code = shell.run_command(command)
            if exit_code != 0:
                raise subprocess.CalledProcessError(exit_code, command, output=output)
            success = True
        except subprocess.CalledProcessError as error:
            output = error.output if hasattr(error, 'output') else ""
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
        except Exception as error:
            output = str(error)
            last_error = subprocess.CalledProcessError(1, command, output=output)
            if not stop_on_success:
                if output:
                    print("")
                    for line in output.splitlines():
                        print(f"{BOX_PREFIX}{line}")
                print("```")
                print("")
                raise SystemExit(1) from error
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
    # Create persistent shell instance for efficient execution
    with PersistentShell(cwd=str(SCRIPT_DIR), env=ENV, debug=False) as shell:

        run_step(shell,
            "Checking prerequisites",
            "for cmd in seedtool envelope provenance cargo; do command -v \"$cmd\"; done",
        )

        run_step(shell,
            "Preparing demo workspace",
            f"rm -rf {qp(DEMO_DIR)} && mkdir -p {qp(DEMO_DIR)}",
        )

        run_step(shell,
            "Generating deterministic publisher seed",
            [
                "PUBLISHER_SEED=$(seedtool --deterministic=CLUBS-DEMO --out seed)",
                "echo $PUBLISHER_SEED"
            ],
        )

        run_step(shell,
            "Deriving publisher signing material",
            [
                'PUBLISHER_PRVKEYS=$(envelope generate prvkeys --seed "$PUBLISHER_SEED")',
                'echo $PUBLISHER_PRVKEYS',
                'PUBLISHER_XID=$(envelope xid new "$PUBLISHER_PRVKEYS")',
                'echo $PUBLISHER_XID',
                'envelope format "$PUBLISHER_XID"',  # Show the formatted output
            ],
        )

        for name, seed_tag in PARTICIPANTS:
            upper = name.upper()
            run_step(
                shell,
                f"Creating XID document for {upper}",
                [
                    f'{upper}_SEED=$(seedtool --deterministic={seed_tag} --out seed)',
                    f'echo "${upper}_SEED=${{{upper}_SEED}}"',
                    f'{upper}_PRVKEYS=$(envelope generate prvkeys --seed "${upper}_SEED")',
                    f'echo "${upper}_PRVKEYS=${{{upper}_PRVKEYS}}"',
                    f'{upper}_PUBKEYS=$(envelope generate pubkeys "${upper}_PRVKEYS")',
                    f'echo "${upper}_PUBKEYS=${{{upper}_PUBKEYS}}"',
                    f'{upper}_XID=$(envelope xid new "${upper}_PRVKEYS")',
                    f'echo "${upper}_XID=${{{upper}_XID}}"',
                    # Show the formatted output
                    f'envelope format "${upper}_XID"',
                ],
            )

        run_step(shell,
            "Assembling edition content envelope",
            [
                'CONTENT_SUBJECT=$(envelope subject type string "Welcome to the Gordian Club!")',
                'CONTENT_CLEAR=$(echo "$CONTENT_SUBJECT" | envelope assertion add pred-obj string "title" string "Genesis Edition")',
                'CONTENT_WRAPPED=$(envelope subject type wrapped "$CONTENT_CLEAR")',
                'envelope format "$CONTENT_WRAPPED"',
            ],
        )

        run_step(shell,
            "Deriving deterministic provenance seed",
            "PROVENANCE_SEED=$(seedtool --deterministic=PROVENANCE-DEMO --count 32 --out seed)",
        )

        register_path(PROV_DIR / "generator.json")
        register_path(PROV_DIR / "marks")
        register_path(PROV_DIR / "marks/mark-0.json")

        run_step(shell,
            "Starting provenance mark chain",
            [
                f'provenance new {rel(PROV_DIR)} --seed "$PROVENANCE_SEED" --comment "Genesis edition"',
                f'provenance print {rel(PROV_DIR)} --start 0 --end 0 --format markdown',
                f'GENESIS_MARK=$(provenance print {rel(PROV_DIR)} --start 0 --end 0 --format ur)',
                f'echo "$GENESIS_MARK"',  # Show the UR we captured
            ],
        )

        edition_file = register_path(DEMO_DIR / "edition.ur")
        sskr_shares = [
            register_path(DEMO_DIR / "sskr-share-g1-1.ur"),
            register_path(DEMO_DIR / "sskr-share-g1-2.ur"),
            register_path(DEMO_DIR / "sskr-share-g1-3.ur"),
        ]

        run_step(shell,
            "Composing genesis edition",
            f"""
            RUSTFLAGS='-C debug-assertions=no' cargo run -p clubs-cli -- init \
              --publisher "$PUBLISHER_XID" \
              --content "$CONTENT_WRAPPED" \
              --provenance "$GENESIS_MARK" \
              --permit "$ALICE_XID" \
              --permit "$BOB_PUBKEYS" \
              --sskr 2of3 \
              --summary \
              --out-dir {qp(DEMO_DIR)}
            """,
        )

        inspect_output = run_step(
            shell,
            "Inspecting composed edition",
            [
                (
                    "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                    f"edition inspect "
                    f"--edition \"@{rel(edition_file)}\" "
                    f"--publisher \"$PUBLISHER_XID\" "
                    f"--summary "
                    f"--emit-permits"
                ),
            ],
        )
        process_inspection_output(inspect_output[-1] if inspect_output else "")

        sskr_output = run_step(
            shell,
            "Decrypting content via SSKR shares",
            [
                (
                    "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                    f"content decrypt "
                    f"--edition \"@{rel(edition_file)}\" "
                    f"--publisher \"$PUBLISHER_XID\" "
                    f"--sskr \"@{rel(sskr_shares[0])}\" "
                    f"--sskr \"@{rel(sskr_shares[1])}\" "
                    f"--emit-ur"
                ),
            ],
        )
        content_from_sskr = register_path(DEMO_DIR / "content.from-sskr.ur")
        process_sskr_output(sskr_output[-1] if sskr_output else "", content_from_sskr)

        run_step(shell,
            "Formatting SSKR-recovered content",
            [
                f'envelope format "$(cat {rel(content_from_sskr)})"',
            ],
        )

        content_from_permit = register_path(DEMO_DIR / "content.from-permit.ur")
        permit_commands = build_permit_commands(edition_file)
        permit_outputs = run_step(
            shell,
            "Decrypting content with Alice's permit",
            permit_commands,
            stop_on_success=True,
        )
        process_permit_output(permit_outputs, content_from_permit)

        run_step(shell,
            "Formatting permit-recovered content",
            [
                f'envelope format "$(cat {rel(content_from_permit)})"',
            ],
        )


SCRIPT_DIR = Path(__file__).resolve().parent

PATH_OBJECTS: set[Path] = set()
PATH_REPLACEMENTS: list[tuple[str, str]] = []


def register_path(path: Path) -> Path:
    """Record *path* for later sanitization and return it unchanged."""

    normalized = path if path.is_absolute() else (SCRIPT_DIR / path).resolve()
    normalized = normalized.resolve()
    if normalized in PATH_OBJECTS:
        return normalized

    PATH_OBJECTS.add(normalized)
    abs_path = str(normalized)
    rel_path = rel(normalized)
    PATH_REPLACEMENTS.append((abs_path, rel_path))
    PATH_REPLACEMENTS.append((shlex.quote(abs_path), rel_path))
    PATH_REPLACEMENTS.append((f"@{abs_path}", f"@{rel_path}"))
    PATH_REPLACEMENTS.append((f"@{shlex.quote(abs_path)}", f"@{rel_path}"))
    return normalized


DEMO_DIR = register_path(SCRIPT_DIR / "demo")
# Preserve the provenance chain directory so the markdown log can point at the
# genesis artifacts generated by `provenance`.
PROV_DIR = register_path(DEMO_DIR / "provenance-chain")

PARTICIPANTS = (
    ("alice", "ALICE-DEMO"),
    ("bob", "BOB-DEMO"),
)

ENV = os.environ.copy()


def process_inspection_output(output: str) -> None:
    ur_lines = [line for line in output.splitlines() if line.startswith("ur:")]

    for permit in sorted(DEMO_DIR.glob("permit-*.ur")):
        permit.unlink()

    if len(ur_lines) <= 1:
        return

    for index, line in enumerate(ur_lines[1:], start=1):
        # Persist each permit UR so the later decryption step can pull it from disk.
        permit_path = register_path(DEMO_DIR / f"permit-{index}.ur")
        permit_path.write_text(line + "\n")


def process_sskr_output(output: str, target_path: Path) -> None:
    ur_lines = [line for line in output.splitlines() if line.startswith("ur:")]
    if not ur_lines:
        raise RuntimeError("SSKR decryption did not produce a UR")
    # Keep the recovered content UR to prove the SSKR path produces the edition payload.
    register_path(target_path).write_text("\n".join(ur_lines) + "\n")


def build_permit_commands(edition_file: Path) -> list[str]:
    commands: list[str] = []
    for permit in sorted(DEMO_DIR.glob("permit-*.ur")):
        commands.append(
            (
                "RUSTFLAGS='-C debug-assertions=no' cargo run -q -p clubs-cli -- "
                f"content decrypt "
                f"--edition \"@{rel(edition_file)}\" "
                f"--publisher \"$PUBLISHER_XID\" "
                f"--permit \"@{rel(permit)}\" "
                f"--identity \"$ALICE_PRVKEYS\" "
                f"--emit-ur"
            )
        )
    if not commands:
        raise RuntimeError("No permit files available for decryption")
    return commands


def process_permit_output(outputs: list[str], target_path: Path) -> None:
    if not outputs:
        raise RuntimeError("Permit decryption produced no output")

    last_output = outputs[-1]
    ur_lines = [line for line in last_output.splitlines() if line.startswith("ur:")]
    if not ur_lines:
        raise RuntimeError("Permit decryption did not produce a UR")
    # Keep the permit-based recovery output so we can diff it against the SSKR result.
    register_path(target_path).write_text("\n".join(ur_lines) + "\n")

if __name__ == "__main__":
    try:
        main()
    except SystemExit as exc:
        sys.exit(exc.code)
