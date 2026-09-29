#!/usr/bin/env python3
"""SessionStart hook: tell Claude when a project created from template.rs has not been set up.

Prints nothing in the template.rs repository itself, or once the setup-template skill
has deleted itself, so it costs nothing outside a fresh project. Claude Code adds a
SessionStart hook's stdout to Claude's context.
"""
import os
import subprocess
import sys
from pathlib import Path

TEMPLATE_REPOSITORY = "ffuffix/template.rs"
MESSAGE = (
    "This repository was created from the template.rs template and has not been set up yet. "
    "Before other work, suggest running /setup-template, which asks how to personalize the project."
)


def origin_url(project: Path) -> str:
    try:
        result = subprocess.run(
            ["git", "-C", str(project), "remote", "get-url", "origin"],
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return ""
    return result.stdout.strip() if result.returncode == 0 else ""


def main() -> int:
    project = Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    if not (project / ".claude" / "skills" / "setup-template" / "SKILL.md").is_file():
        return 0
    if TEMPLATE_REPOSITORY in origin_url(project):
        return 0
    print(MESSAGE)
    return 0


if __name__ == "__main__":
    sys.exit(main())
