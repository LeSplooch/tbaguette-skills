"""The hook has to survive a fresh clone, and git will not help.

`core.hooksPath` is local to a clone and cannot be committed, so
`.githooks/pre-commit` ships inert. This suite covers githooks.ensure_wired,
which is what the repo's own entry points call to close that gap, and the
shape of the hook itself.

The absolute-path case has its own test because the first version of
ensure_wired compared the configured value to the literal string ".githooks"
and therefore reported this very repo -- which stores an absolute path -- as
having a foreign hooksPath, and offered to leave it "alone".
"""
from __future__ import annotations

import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

from githooks import HOOKS_DIRNAME, ensure_wired  # noqa: E402


def _git(root, *args):
    return subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=False)


def _throwaway_repo(tmp: Path) -> Path:
    _git(tmp, "init", "--quiet")
    (tmp / HOOKS_DIRNAME).mkdir()
    (tmp / HOOKS_DIRNAME / "pre-commit").write_text("#!/bin/sh\nexit 0\n")
    return tmp


class TestTheRealHook(unittest.TestCase):
    def test_hook_exists_and_is_executable(self):
        hook = REPO_ROOT / HOOKS_DIRNAME / "pre-commit"
        self.assertTrue(hook.is_file())
        self.assertTrue(hook.stat().st_mode & stat.S_IXUSR, "pre-commit is not executable")

    def test_hook_is_committed_with_its_executable_bit(self):
        """A hook that loses its +x in the index is inert on every fresh clone,
        and the symptom is silence rather than an error."""
        out = _git(REPO_ROOT, "ls-files", "-s", f"{HOOKS_DIRNAME}/pre-commit").stdout
        self.assertTrue(out.startswith("100755"), f"expected mode 100755, got {out.split()[0:1]}")

    def test_hook_regenerates_the_site(self):
        body = (REPO_ROOT / HOOKS_DIRNAME / "pre-commit").read_text(encoding="utf-8")
        self.assertIn("scripts/generate.py", body)
        self.assertIn("git add docs/", body)


class TestEnsureWired(unittest.TestCase):
    def test_wires_an_unwired_clone_then_is_idempotent(self):
        with tempfile.TemporaryDirectory() as d:
            root = _throwaway_repo(Path(d))
            self.assertEqual(ensure_wired(root, announce=lambda *_: None), "wired")
            self.assertEqual(
                _git(root, "config", "--local", "--get", "core.hooksPath").stdout.strip(),
                HOOKS_DIRNAME,
            )
            self.assertEqual(ensure_wired(root, announce=lambda *_: None), "already-wired")

    def test_an_absolute_path_to_the_same_dir_counts_as_wired(self):
        with tempfile.TemporaryDirectory() as d:
            root = _throwaway_repo(Path(d))
            _git(root, "config", "--local", "core.hooksPath", str(root / HOOKS_DIRNAME))
            self.assertEqual(ensure_wired(root, announce=lambda *_: None), "already-wired")

    def test_a_foreign_hooks_path_is_reported_and_never_overwritten(self):
        with tempfile.TemporaryDirectory() as d:
            root = _throwaway_repo(Path(d))
            (root / "otherhooks").mkdir()
            _git(root, "config", "--local", "core.hooksPath", "otherhooks")
            said = []
            self.assertEqual(ensure_wired(root, announce=said.append), "custom")
            self.assertEqual(
                _git(root, "config", "--local", "--get", "core.hooksPath").stdout.strip(),
                "otherhooks",
                "ensure_wired overwrote a deliberately-set hooksPath",
            )
            self.assertTrue(said, "a foreign hooksPath must be announced, not swallowed")

    def test_a_directory_that_is_not_a_git_checkout_does_not_raise(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / HOOKS_DIRNAME).mkdir()
            self.assertEqual(ensure_wired(root, announce=lambda *_: None), "not-a-git-checkout")

    def test_no_hooks_directory_is_a_no_op(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(ensure_wired(Path(d), announce=lambda *_: None), "no-hooks-dir")


if __name__ == "__main__":
    unittest.main(verbosity=2)
