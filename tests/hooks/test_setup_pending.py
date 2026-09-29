import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HOOK = Path(__file__).resolve().parents[2] / ".claude" / "hooks" / "setup-pending.py"


def run_hook(project: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(HOOK)],
        input="{}",
        capture_output=True,
        text=True,
        env={**os.environ, "CLAUDE_PROJECT_DIR": str(project)},
    )


class SetupPendingHookTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.project = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q", str(self.project)], check=True)
        self.skill = self.project / ".claude" / "skills" / "setup-template" / "SKILL.md"
        self.skill.parent.mkdir(parents=True)
        self.skill.write_text("# Set up\n")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def set_origin(self, url: str) -> None:
        subprocess.run(["git", "-C", str(self.project), "remote", "add", "origin", url], check=True)

    def test_suggests_setup_in_a_project_created_from_the_template(self) -> None:
        self.set_origin("https://github.com/example/my-project.git")

        result = run_hook(self.project)

        self.assertEqual(result.returncode, 0)
        self.assertIn("/setup-template", result.stdout)

    def test_suggests_setup_when_the_project_has_no_remote(self) -> None:
        result = run_hook(self.project)

        self.assertEqual(result.returncode, 0)
        self.assertIn("/setup-template", result.stdout)

    def test_stays_silent_in_the_template_repository_itself(self) -> None:
        self.set_origin("git@github.com:ffuffix/template.rs.git")

        result = run_hook(self.project)

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")

    def test_stays_silent_once_setup_has_removed_the_skill(self) -> None:
        self.set_origin("https://github.com/example/my-project.git")
        self.skill.unlink()

        result = run_hook(self.project)

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
