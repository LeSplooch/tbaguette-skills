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
    ".github/plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
    "hooks/hooks.json",
    "hooks/hooks-copilot.json",
    "hooks/hooks-cursor.json",
    "package.json",
]

# .claude-plugin/plugin.json is absent on purpose: it is the source of truth
# every entry here is compared against, not a follower.
#
# package.json is here because it is a harness manifest like the rest -- the
# declared entry point for two of the eight reference integrations (OpenCode
# reads "main", Pi reads the "pi" field; see PORTING.md). It was left out
# originally with no reason recorded, and drifted to 0.9.0 against a plugin at
# 0.14.2 -- five minor versions, silently, because nothing compared them. Its
# npm-ness is incidental: this repo publishes nothing to npm and has no release
# automation at all, so the version field has exactly one job here, the same one
# it has in the other six.
VERSIONED_MANIFESTS = [
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".github/plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
    "package.json",
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

    def test_copilot_manifest_declares_its_own_hook_file(self):
        """Copilot CLI resolves a plugin's hooks by searching for hooks.json or
        hooks/hooks.json when the manifest doesn't name one -- and this repo has
        a hooks/hooks.json in Claude Code's incompatible schema. So the Copilot
        manifest naming its own hook file is not tidiness, it is the thing
        keeping Copilot off a config it cannot parse."""
        manifest = _load_json(".github/plugin/plugin.json")
        hooks_rel = manifest.get("hooks")
        self.assertEqual(hooks_rel, "./hooks/hooks-copilot.json")
        self.assertTrue((REPO_ROOT / "hooks/hooks-copilot.json").is_file())
        self.assertEqual(manifest.get("skills"), "./skills/")

    def test_copilot_manifest_is_not_moved_to_the_conventional_name(self):
        """Every other integration here lives in .<harness>-plugin/, so this one
        looks like the odd one out and reads as something to tidy up. It is not.
        Copilot CLI searches a fixed list -- .plugin/plugin.json, plugin.json,
        .github/plugin/plugin.json, .claude-plugin/plugin.json -- and
        .copilot-plugin/ is not on it. Renaming for consistency would leave a
        manifest that is never read, and nothing else in this suite would
        notice, because every other assertion about it would still pass.

        Guarding the absence rather than the presence is the point: that the
        file exists is already covered above, and this is the failure mode that
        would otherwise ship silently."""
        self.assertFalse(
            (REPO_ROOT / ".copilot-plugin").exists(),
            ".copilot-plugin/ is not a location Copilot CLI searches -- the "
            "Copilot manifest belongs at .github/plugin/plugin.json",
        )

    def test_published_copilot_install_command_names_a_real_marketplace(self):
        """`copilot plugin install TBaguette@tbaguette-dev` is printed on the
        live site. Both halves of that spec come from marketplace.json, and
        neither is derived at build time -- renaming either field would leave
        the site publishing an install command for a marketplace and a plugin
        that no longer exist, with every other suite still green."""
        marketplace = _load_json(".claude-plugin/marketplace.json")
        plugin_name = marketplace["plugins"][0]["name"]
        spec = f"{plugin_name}@{marketplace['name']}"
        self.assertEqual(spec, "TBaguette@tbaguette-dev")

        porting = (REPO_ROOT / "PORTING.md").read_text(encoding="utf-8")
        self.assertIn(f"copilot plugin install {spec}", porting)

        # Copilot resolves that entry's source to the repo root and searches
        # there for a manifest -- so the source has to point at a directory
        # this repo actually gives it one in.
        source = marketplace["plugins"][0]["source"]
        self.assertEqual(source, "./")
        self.assertTrue((REPO_ROOT / source / ".github/plugin/plugin.json").is_file())

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
