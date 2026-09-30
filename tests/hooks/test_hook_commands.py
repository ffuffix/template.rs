import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPOSITORY = Path(__file__).resolve().parents[2]
SETTINGS = REPOSITORY / ".claude" / "settings.json"


def hook_commands() -> list[str]:
    settings = json.loads(SETTINGS.read_text())
    return [
        hook["command"]
        for matchers in settings["hooks"].values()
        for matcher in matchers
        for hook in matcher["hooks"]
    ]


class HookCommandTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.project = Path(self.temporary.name) / "my project"
        shutil.copytree(REPOSITORY / ".claude" / "hooks", self.project / ".claude" / "hooks")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def assert_hook_commands_run(self, shell: list[str]) -> None:
        for template in hook_commands():
            command = template.replace("${CLAUDE_PROJECT_DIR}", str(self.project))
            with self.subTest(command=template):
                result = subprocess.run(
                    [*shell, command],
                    input="{}",
                    capture_output=True,
                    text=True,
                    env={**os.environ, "CLAUDE_PROJECT_DIR": str(self.project)},
                )

                self.assertEqual(result.returncode, 0, result.stderr)

    def test_runs_through_sh_from_a_path_with_spaces(self) -> None:
        self.assert_hook_commands_run(["sh", "-c"])

    @unittest.skipUnless(shutil.which("pwsh"), "PowerShell is not installed")
    def test_runs_through_powershell_from_a_path_with_spaces(self) -> None:
        self.assert_hook_commands_run(["pwsh", "-NoProfile", "-NonInteractive", "-Command"])


if __name__ == "__main__":
    unittest.main()
