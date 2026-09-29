#!/usr/bin/env python3
"""PostToolUse hook: run `cargo fmt` on the package that owns an edited Rust file.

Resolves the package from the edited file's path rather than CLAUDE_PROJECT_DIR,
because CLAUDE_PROJECT_DIR keeps pointing at the main checkout after Claude
enters a worktree. Exit 2 feeds rustfmt errors back to Claude.

A `mod` declaration whose file does not exist yet is expected while Claude
writes a module tree one file at a time, so that error is not reported.
"""
import json
import subprocess
import sys
from pathlib import Path

MAX_ERROR_LINES = 20
CARGO_TIMEOUT_SECONDS = 50
UNWRITTEN_MODULE_ERROR = "failed to resolve mod"


def find_manifest(start: Path) -> Path | None:
    for directory in (start, *start.parents):
        manifest = directory / "Cargo.toml"
        if manifest.is_file():
            return manifest
    return None


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    raw_path = (payload.get("tool_input") or {}).get("file_path")
    if not raw_path:
        return 0

    path = Path(payload.get("cwd") or ".", raw_path)
    if path.suffix != ".rs":
        return 0

    manifest = find_manifest(path.parent)
    if manifest is None:
        return 0

    try:
        result = subprocess.run(
            ["cargo", "fmt", "--manifest-path", str(manifest)],
            capture_output=True,
            text=True,
            timeout=CARGO_TIMEOUT_SECONDS,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return 0

    errors = result.stderr or result.stdout
    if result.returncode == 0 or UNWRITTEN_MODULE_ERROR in errors:
        return 0

    output = errors.strip().splitlines()
    print(f"cargo fmt failed for {path.name}:", file=sys.stderr)
    for line in output[:MAX_ERROR_LINES]:
        print(f"  {line}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
