import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HOOK = Path(__file__).resolve().parents[2] / ".claude" / "hooks" / "cargo-fmt.py"
UNFORMATTED = "pub fn answer( )->u32{42}\n"
FORMATTED = "pub fn answer() -> u32 {\n    42\n}\n"
MANIFEST = '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2024"\n'


def run_hook(payload: object, environment: dict[str, str] | None = None) -> subprocess.CompletedProcess:
    stdin = payload if isinstance(payload, str) else json.dumps(payload)
    return subprocess.run(
        [sys.executable, str(HOOK)],
        input=stdin,
        capture_output=True,
        text=True,
        env={**os.environ, **(environment or {})},
    )


class CargoFmtHookTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.package = self.root / "sample"
        (self.package / "src").mkdir(parents=True)
        (self.package / "Cargo.toml").write_text(MANIFEST)
        self.library = self.package / "src" / "lib.rs"
        self.library.write_text(UNFORMATTED)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_formats_owning_package_even_when_project_dir_is_elsewhere(self) -> None:
        result = run_hook(
            {"cwd": str(self.root), "tool_input": {"file_path": str(self.library)}},
            {"CLAUDE_PROJECT_DIR": str(self.root / "main-checkout")},
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.library.read_text(), FORMATTED)

    def test_resolves_relative_file_path_against_cwd(self) -> None:
        result = run_hook({"cwd": str(self.package), "tool_input": {"file_path": "src/lib.rs"}})

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.library.read_text(), FORMATTED)

    def test_ignores_non_rust_files(self) -> None:
        readme = self.package / "README.md"
        readme.write_text("# sample\n")

        result = run_hook({"cwd": str(self.root), "tool_input": {"file_path": str(readme)}})

        self.assertEqual(result.returncode, 0)
        self.assertEqual(self.library.read_text(), UNFORMATTED)

    def test_ignores_rust_file_outside_any_cargo_package(self) -> None:
        loose = self.root / "loose.rs"
        loose.write_text(UNFORMATTED)

        result = run_hook({"cwd": str(self.root), "tool_input": {"file_path": str(loose)}})

        self.assertEqual(result.returncode, 0)
        self.assertEqual(loose.read_text(), UNFORMATTED)

    def test_reports_syntax_error_to_claude_with_exit_two(self) -> None:
        self.library.write_text("pub fn broken( {\n")

        result = run_hook({"cwd": str(self.root), "tool_input": {"file_path": str(self.library)}})

        self.assertEqual(result.returncode, 2)
        self.assertIn("cargo fmt failed for lib.rs", result.stderr)

    def test_stays_silent_while_a_declared_module_is_not_written_yet(self) -> None:
        self.library.write_text("mod parser;\n")

        result = run_hook({"cwd": str(self.root), "tool_input": {"file_path": str(self.library)}})

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stderr, "")

    def test_ignores_malformed_input(self) -> None:
        result = run_hook("not json")

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
