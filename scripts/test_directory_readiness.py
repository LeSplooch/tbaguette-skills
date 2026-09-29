"""The whole plugin folder stays inside Anthropic's plugin directory rules.

Claude Desktop, claude.ai and Cowork install plugins from the directory,
and the directory reads this repository at every commit on the branch it
tracks. Each commit is validated and scanned again, so a rule broken by one
careless push does not wait for the next submission: it blocks or holds the
version that push produced, and every version after it until someone
notices. The listing keeps serving the last published version meanwhile,
which makes the failure quiet -- nothing breaks for anyone, the updates
simply stop arriving.

The rules are the directory's own, from its pre-submission checklist
(https://claude.com/docs/plugins/pre-submission-checklist), limited to the
ones a repository can check without the portal. Two outcomes matter and
the tests say which one each rule produces:

* Blocks, or stops validation: the version cannot be submitted or listed.
* Held for a reviewer: the version waits for a person before it goes live.
  The first listing is always read by a person anyway; what a hold costs is
  every later update, one reviewer at a time.

Two files already had a guard of their own before this suite existed
(graph.json in test_generate, UPDATES.md in test_archive_updates). This one
reads every file, because the next file to cross a limit is the one nobody
wrote a guard for.

The plugin folder is the repository root: .claude-plugin/plugin.json sits
there, and the directory reads what the commit contains, which is what git
tracks rather than whatever happens to be on disk.
"""
from __future__ import annotations

import json
import re
import subprocess
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
KIB = 1024
MIB = 1024 * KIB

# Held for a reviewer past these.
MAX_TEXT_FILE = 256 * KIB
MAX_FILES = 512
# Validation stops past these.
MAX_ANY_FILE = 5 * MIB
MAX_ARCHIVE = 50 * MIB
MAX_ENTRIES = 10_000

# Images and fonts are exempt from the 256 KiB rule, and are the only
# binaries the validator reads; anything else binary is held.
EXEMPT_BINARY = re.compile(r"\.(png|jpe?g|gif|webp|svg|woff2?|ttf|otf)$", re.I)

# Symbolic links are a warning where the plugin does not load them and a
# block where it does. This one points AGENTS.md at CLAUDE.md for other
# agents working on this repository; no Claude surface loads either file
# from a plugin.
ALLOWED_SYMLINKS = {"AGENTS.md"}

RESERVED_WHOLE_NAMES = {"claude", "anthropic", "official", "plugin", "mcp", "test"}
SYSTEM_FILES = {".DS_Store", "Thumbs.db", "desktop.ini"}
WINDOWS_DEVICES = {"con", "prn", "aux", "nul"} | {f"com{i}" for i in range(1, 10)} | {f"lpt{i}" for i in range(1, 10)}
PACKAGE_LAUNCHERS = re.compile(r"\b(npx|bunx|pnpm\s+dlx|yarn\s+dlx|uvx|pipx\s+run|uv\s+run)\b")
PACKAGE_SOURCE_CONFIGS = {".npmrc", "bunfig.toml", "uv.toml", ".yarnrc", ".yarnrc.yml", "pip.conf"}
HOOK_EVENTS = {
    "SessionStart", "SessionEnd", "UserPromptSubmit", "PreToolUse", "PostToolUse",
    "PostToolUseFailure", "Notification", "Stop", "SubagentStart", "SubagentStop",
    "PreCompact", "PermissionRequest",
}


def _tracked() -> list[tuple[str, str]]:
    """(mode, path) for every file the commit will contain."""
    out = subprocess.run(["git", "ls-files", "-s"], cwd=REPO_ROOT, check=True,
                         capture_output=True, text=True).stdout
    entries = []
    for line in out.splitlines():
        meta, path = line.split("\t", 1)
        entries.append((meta.split()[0], path))
    return entries


def _regular(entries):
    return [path for mode, path in entries if mode not in ("120000", "160000")]


class TestLayoutThatStopsValidation(unittest.TestCase):
    def setUp(self):
        self.entries = _tracked()
        self.paths = [path for _, path in self.entries]

    def test_the_manifest_is_at_the_plugin_root(self):
        self.assertTrue((REPO_ROOT / ".claude-plugin" / "plugin.json").is_file())

    def test_no_file_reaches_five_mib(self):
        big = [p for p in _regular(self.entries) if (REPO_ROOT / p).stat().st_size >= MAX_ANY_FILE]
        self.assertEqual(big, [])

    def test_fewer_than_ten_thousand_entries(self):
        folders = {str(Path(p).parent) for p in self.paths}
        self.assertLess(len(self.paths) + len(folders), MAX_ENTRIES)

    def test_the_archive_github_serves_is_under_fifty_mib(self):
        size = len(subprocess.run(["git", "archive", "--format=tar.gz", "HEAD"], cwd=REPO_ROOT,
                                  check=True, capture_output=True).stdout)
        self.assertLess(size, MAX_ARCHIVE)

    def test_names_are_valid_on_windows_and_macos(self):
        bad = []
        for path in self.paths:
            for part in path.split("/"):
                stem = part.split(".")[0].lower()
                if (re.search(r'[:<>"|?*\\]', part) or part.endswith((".", " "))
                        or stem in WINDOWS_DEVICES):
                    bad.append(path)
        self.assertEqual(bad, [])

    def test_no_two_names_differ_only_by_case(self):
        seen: dict[str, str] = {}
        clashes = []
        for path in self.paths:
            key = path.lower()
            if key in seen:
                clashes.append((seen[key], path))
            seen[key] = path
        self.assertEqual(clashes, [])

    def test_gitattributes_never_rewrites_or_drops_content(self):
        found = []
        for path in self.paths:
            if Path(path).name == ".gitattributes":
                text = (REPO_ROOT / path).read_text(encoding="utf-8")
                for line in text.splitlines():
                    if line.lstrip().startswith("#"):
                        continue
                    if re.search(r"\bexport-ignore\b|\bexport-subst\b|\bfilter\b", line):
                        found.append(f"{path}: {line}")
        self.assertEqual(found, [])


class TestWhatBlocksASubmission(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((REPO_ROOT / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))
        self.paths = [path for _, path in _tracked()]

    def test_the_name_is_lowercase_ascii_kebab_case(self):
        self.assertRegex(self.manifest["name"], r"^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$")

    def test_the_name_is_not_a_reserved_word(self):
        self.assertNotIn(self.manifest["name"], RESERVED_WHOLE_NAMES)

    def test_display_and_author_names_are_plain_ascii(self):
        for field in (self.manifest.get("displayName", ""), self.manifest["author"]["name"]):
            with self.subTest(field=field):
                self.assertTrue(field.isascii() and field.isprintable())

    def test_description_author_and_version_are_set(self):
        for key in ("description", "author", "version"):
            with self.subTest(key=key):
                self.assertTrue(self.manifest.get(key))

    def test_a_licence_ships(self):
        self.assertTrue(self.manifest.get("license") or (REPO_ROOT / "LICENSE").is_file())

    def test_the_readme_has_forty_words_outside_code(self):
        prose = re.sub(r"```.*?```", "", (REPO_ROOT / "README.md").read_text(encoding="utf-8"), flags=re.S)
        self.assertGreaterEqual(len(re.findall(r"\w+", prose)), 40)

    def test_no_operating_system_litter(self):
        litter = [p for p in self.paths
                  if Path(p).name in SYSTEM_FILES or "__MACOSX" in p.split("/")]
        self.assertEqual(litter, [])

    def test_hooks_json_uses_only_known_events(self):
        hooks = json.loads((REPO_ROOT / "hooks" / "hooks.json").read_text(encoding="utf-8"))
        self.assertIsInstance(hooks.get("hooks"), dict)
        self.assertLessEqual(set(hooks["hooks"]), HOOK_EVENTS)

    def test_hook_scripts_run_no_package_launcher(self):
        hooks = json.loads((REPO_ROOT / "hooks" / "hooks.json").read_text(encoding="utf-8"))
        commands = [h["command"] for groups in hooks["hooks"].values()
                    for group in groups for h in group.get("hooks", []) if "command" in h]
        scripts = {REPO_ROOT / "hooks" / name for name in ("run-hook.cmd", "session-start", "user-prompt-submit")}
        texts = commands + [s.read_text(encoding="utf-8") for s in scripts if s.is_file()]
        self.assertEqual([t for t in texts if PACKAGE_LAUNCHERS.search(t)], [])

    def test_no_file_redirects_package_installs(self):
        self.assertEqual([p for p in self.paths if Path(p).name in PACKAGE_SOURCE_CONFIGS], [])


class TestWhatHoldsEveryUpdateForAReviewer(unittest.TestCase):
    def setUp(self):
        self.entries = _tracked()

    def test_every_text_file_is_under_256_kib(self):
        big = [(p, (REPO_ROOT / p).stat().st_size) for p in _regular(self.entries)
               if not EXEMPT_BINARY.search(p) and (REPO_ROOT / p).stat().st_size >= MAX_TEXT_FILE]
        self.assertEqual(big, [])

    def test_no_more_than_512_files(self):
        self.assertLessEqual(len(self.entries), MAX_FILES,
                             f"{len(self.entries)} files; past {MAX_FILES} every version waits for a reviewer")

    def test_the_only_binaries_are_images_and_fonts(self):
        binaries = []
        for path in _regular(self.entries):
            if EXEMPT_BINARY.search(path):
                continue
            if b"\0" in (REPO_ROOT / path).read_bytes()[:8192]:
                binaries.append(path)
        self.assertEqual(binaries, [])

    def test_symlinks_only_where_nothing_loads_them(self):
        links = {path for mode, path in self.entries if mode == "120000"}
        self.assertLessEqual(links, ALLOWED_SYMLINKS)
        self.assertEqual([path for mode, path in self.entries if mode == "160000"], [], "git submodules")

    def test_no_git_lfs_pointers(self):
        pointers = [p for p in _regular(self.entries)
                    if (REPO_ROOT / p).read_bytes()[:40].startswith(b"version https://git-lfs")]
        self.assertEqual(pointers, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
