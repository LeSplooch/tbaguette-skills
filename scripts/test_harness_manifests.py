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
    "plugin.json",
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "com.github.copilot/hooks/hooks.json",
    "gemini-extension.json",
    "hooks/hooks.json",
    "hooks/hooks-codex.json",
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
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
    "package.json",
    "plugin.json",
]


# .hermes-plugin/plugin.yaml is the one manifest here that is not JSON, and it
# is why this list exists separately rather than the YAML being skipped: it had
# drifted to 0.6.0 against a plugin at 1.0.28 -- silently, for the same reason
# package.json once drifted five minor versions, which is that nothing compared
# them. Parsed by hand rather than with PyYAML because this repository has no
# third-party dependencies and the file is flat `key: value` lines.
YAML_MANIFESTS = [".hermes-plugin/plugin.yaml"]


def _load_yaml_scalars(rel_path: str) -> dict:
    values = {}
    for line in (REPO_ROOT / rel_path).read_text(encoding="utf-8").splitlines():
        if line.startswith((" ", "-", "#")) or ":" not in line:
            continue
        key, _, value = line.partition(":")
        values[key.strip()] = value.strip()
    return values


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
        for rel_path in YAML_MANIFESTS:
            with self.subTest(manifest=rel_path):
                self.assertEqual(_load_yaml_scalars(rel_path)["version"], expected_version)

    def test_agents_md_symlinks_to_claude_md(self):
        agents_md = REPO_ROOT / "AGENTS.md"
        self.assertTrue(agents_md.is_symlink(), "AGENTS.md must be a symlink")
        self.assertEqual(agents_md.resolve(), (REPO_ROOT / "CLAUDE.md").resolve())

    def test_copilot_manifest_declares_its_own_hook_file(self):
        """Copilot CLI resolves a plugin's hooks by searching for hooks.json or
        hooks/hooks.json when the manifest doesn't name one -- and this repo has
        a hooks/hooks.json in Claude Code's incompatible schema. So the manifest
        naming its own hook file is not tidiness, it is the thing keeping the
        CLI off a config it cannot parse."""
        manifest = _load_json("plugin.json")
        self.assertEqual(manifest.get("hooks"), "./hooks/hooks-copilot.json")
        self.assertTrue((REPO_ROOT / "hooks/hooks-copilot.json").is_file())
        self.assertEqual(manifest.get("skills"), "./skills/")

    def test_copilot_manifest_is_at_the_repo_root(self):
        """Every other integration here lives in .<harness>-plugin/, so a root
        plugin.json looks like the odd one out and reads as something to tidy
        away. It is not. Copilot CLI searches a fixed list -- .plugin/,
        plugin.json, .github/plugin/, .claude-plugin/ -- and VS Code searches a
        different one that does NOT include .github/plugin/ at all. The repo
        root is the only location on both lists, which is why one manifest can
        serve both. Moving it anywhere more conventional loses a surface
        silently, with every other assertion in this suite still passing."""
        self.assertTrue((REPO_ROOT / "plugin.json").is_file())
        for tidier_looking in (".copilot-plugin", ".github/plugin"):
            self.assertFalse(
                (REPO_ROOT / tidier_looking).exists(),
                f"{tidier_looking}/ is not read by both Copilot surfaces -- the "
                "manifest belongs at the repo root",
            )

    def test_agent_plugins_schema_is_what_routes_vscode_to_its_hooks(self):
        """VS Code ignores a manifest's hooks field entirely and derives the
        path from the detected plugin format: Agent Plugins 1.0 resolves to
        com.github.copilot/hooks/hooks.json, the other formats resolve
        elsewhere. So the $schema line is not decoration -- drop it and VS Code
        stops finding the only hook file written for it, while the CLI (which
        does honor the hooks field) carries on working and hides the loss."""
        manifest = _load_json("plugin.json")
        self.assertEqual(
            manifest.get("$schema"),
            "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
        )
        self.assertTrue((REPO_ROOT / "com.github.copilot/hooks/hooks.json").is_file())

    def test_the_two_copilot_hook_files_do_not_swap_event_casing(self):
        """Two hook files, two casings, and they are not interchangeable: the
        CLI's file is reached through the manifest and uses camelCase; VS Code's
        is reached by format-derived path and uses the PascalCase names its docs
        publish. Swapping them leaves both files present, both valid JSON, and
        at least one surface with no bootstrap."""
        cli = _load_json("hooks/hooks-copilot.json")["hooks"]
        vscode = _load_json("com.github.copilot/hooks/hooks.json")["hooks"]
        self.assertEqual(set(cli), {"sessionStart", "userPromptSubmitted"})
        self.assertEqual(set(vscode), {"SessionStart", "UserPromptSubmit"})

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
        # The coding agent installs the same plugin by the same spec, but
        # declaratively -- so a rename would have to be chased into a second
        # published snippet that no other assertion here would catch.
        self.assertIn(f'"{spec}": true', porting)

        # Copilot resolves that entry's source to the repo root and searches
        # there for a manifest -- so the source has to point at a directory
        # this repo actually gives it one in.
        source = marketplace["plugins"][0]["source"]
        self.assertEqual(source, "./")
        self.assertTrue((REPO_ROOT / source / "plugin.json").is_file())

    def test_codex_manifest_no_longer_disables_its_own_hooks(self):
        """This shipped as `"hooks": {}` -- deliberately empty, to stop Codex
        default-discovering Claude Code's hooks/hooks.json. It worked, and the
        cost was the whole bootstrap: Codex had skills on disk and nothing that
        ever put the rule in front of the model.

        Codex's hook config shape, its stdout shape, and even its
        CLAUDE_PLUGIN_ROOT variable are all Claude Code's, so the fix was a
        hook file of its own rather than no hooks at all. An empty object here
        again would restore the original defect silently."""
        hooks = _load_json(".codex-plugin/plugin.json").get("hooks")
        self.assertEqual(hooks, "./hooks/hooks-codex.json")
        self.assertTrue((REPO_ROOT / "hooks/hooks-codex.json").is_file())

    def test_every_harness_hook_file_reaches_a_real_script(self):
        """Five hook configs now, in four schemas, and the one thing they all
        have to get right is naming a script that exists. A rename in hooks/
        that missed one would leave that harness exiting non-zero on every
        session start, which no other assertion here would notice."""
        configs = {
            "hooks/hooks.json": ("SessionStart", "UserPromptSubmit"),
            "hooks/hooks-codex.json": ("SessionStart", "UserPromptSubmit"),
            "hooks/hooks-copilot.json": ("sessionStart", "userPromptSubmitted"),
            "com.github.copilot/hooks/hooks.json": ("SessionStart", "UserPromptSubmit"),
            "hooks/hooks-cursor.json": ("sessionStart", "postToolUse"),
        }
        for rel_path, events in configs.items():
            data = _load_json(rel_path)
            with self.subTest(config=rel_path):
                self.assertEqual(set(data["hooks"]), set(events))
                commands = json.dumps(data["hooks"])
                # Every config, whatever its schema, routes through the one
                # cross-platform launcher.
                self.assertIn("run-hook.cmd", commands)
                for script in ("session-start", "user-prompt-submit"):
                    if script in commands:
                        self.assertTrue(
                            (REPO_ROOT / "hooks" / script).is_file(),
                            f"{rel_path} names hooks/{script}, which does not exist",
                        )

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
