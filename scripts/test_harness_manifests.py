"""Validates every harness manifest this repo ships is well-formed and
internally consistent with .claude-plugin/plugin.json, the source of
truth for the plugin's name/version. Proportionate to "compatibility
layer" scope -- this is not a port of superpowers' own per-harness
tests/ suite, just enough to catch a malformed JSON file or a stale
version number before it ships.
"""
from __future__ import annotations

import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

JSON_MANIFESTS = [
    ".claude-plugin/plugin.json",
    ".claude-plugin/marketplace.json",
    ".agents/plugins/marketplace.json",
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
    "hooks/hooks.json",
    "hooks/hooks-cursor.json",
    "package.json",
]

VERSIONED_MANIFESTS = [
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
]


def _load_json(rel_path: str):
    with (REPO_ROOT / rel_path).open() as f:
        return json.load(f)


class TestHarnessManifests(unittest.TestCase):
    def test_all_manifests_are_valid_json(self):
        for rel_path in JSON_MANIFESTS:
            path = REPO_ROOT / rel_path
            with self.subTest(manifest=rel_path):
                self.assertTrue(path.is_file(), f"{rel_path} missing")
                _load_json(rel_path)  # raises on malformed JSON

    def test_versions_match_plugin_json(self):
        expected_version = _load_json(".claude-plugin/plugin.json")["version"]
        for rel_path in VERSIONED_MANIFESTS:
            data = _load_json(rel_path)
            with self.subTest(manifest=rel_path):
                self.assertEqual(data["version"], expected_version)
        marketplace = _load_json(".claude-plugin/marketplace.json")
        with self.subTest(manifest=".claude-plugin/marketplace.json (nested)"):
            self.assertEqual(marketplace["plugins"][0]["version"], expected_version)

    def test_agents_md_symlinks_to_claude_md(self):
        agents_md = REPO_ROOT / "AGENTS.md"
        self.assertTrue(agents_md.is_symlink(), "AGENTS.md must be a symlink")
        self.assertEqual(agents_md.resolve(), (REPO_ROOT / "CLAUDE.md").resolve())

    def test_package_json_points_at_real_files(self):
        package = _load_json("package.json")
        main_path = REPO_ROOT / package["main"]
        self.assertTrue(main_path.is_file(), f"package.json main={package['main']!r} does not exist")
        for ext_path in package["pi"]["extensions"]:
            resolved = REPO_ROOT / ext_path
            with self.subTest(extension=ext_path):
                self.assertTrue(resolved.is_file(), f"pi extension {ext_path!r} does not exist")


if __name__ == "__main__":
    unittest.main()
