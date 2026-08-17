# Harness Compatibility Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make TBaguette discoverable and loadable on every harness the `superpowers` plugin (v6.3.0) supports — Codex, Cursor, Devin, Gemini CLI, Hermes, Kimi, OpenCode, Pi, plus the generic `.agents` marketplace convention — without touching `skills/` content.

**Architecture:** Add one small manifest/adapter per harness at the repo root, adapted from superpowers 6.3.0's own copies (cached at `/home/thisfuck/.claude/plugins/cache/claude-plugins-official/superpowers/6.3.0/`): rename `superpowers`→`TBaguette`, point `skills` paths at `./skills/`, point bootstrap references at `using-tbaguette` instead of `using-superpowers`, and drop everything that's superpowers' own dev/release tooling rather than harness discovery.

**Tech Stack:** JSON manifests, one Node.js adapter (OpenCode), one TypeScript adapter (Pi), one Python adapter (Hermes), bash (existing hooks, unchanged).

## Global Constraints

- Scope is discovery/compatibility only. Do NOT port superpowers' `.pre-commit-config.yaml`, `scripts/bump-version.sh`, per-harness `tests/`, `RELEASE-NOTES.md`, `CODE_OF_CONDUCT.md`, or `.github/ISSUE_TEMPLATE/` — TBaguette already covers that ground differently via `run_tests.py` and this repo's own ship pipeline.
- TBaguette's `docs/` directory is the **generated static site**, fully owned by `scripts/generate.py` — never place hand-authored harness docs there. Superpowers' `docs/porting-to-a-new-harness.md`, `docs/README.kimi.md`, `docs/README.opencode.md` become `PORTING.md`, `README.kimi.md`, `README.opencode.md` at the repo root instead, as peers of `README.md`.
- Reuse the existing `docs/assets/favicon.svg` for any manifest field wanting a small icon reference. Do not create new raster artwork (no `app-icon.png` equivalent).
- Do not touch `.claude/worktrees/website-i18n` (branch `fr-using-tbaguette`) — unrelated in-progress work in a sibling worktree.
- Every new/modified file must leave `python3 scripts/run_tests.py` fully green before any commit.
- Source reference for every task: superpowers 6.3.0 files under `/home/thisfuck/.claude/plugins/cache/claude-plugins-official/superpowers/6.3.0/` (read-only reference — never edit anything under that path).
- Repo's own commit convention: real message explaining *why*, no `--no-verify`, new commits only (never amend), push to `origin/master` only. Before any push, `git fetch origin master` and check for divergence first — this repo has other agents/worktrees pushing to it concurrently (confirmed during the design phase: a `website-i18n` worktree merged 27 commits mid-session). If `origin/master` has moved, `git merge origin/master`, resolve any conflicts confined to the generated `docs/` tree by regenerating from merged source (`git checkout --ours -- docs/ && python3 scripts/generate.py --base-path /tbaguette-skills`), rerun the full test suite, then push.

---

### Execution note (found during Task 1, not in the original scan)

The design spec's harness table didn't surface this: OpenCode's and Pi's
discovery both depend on a repo-root `package.json` (`main` for OpenCode,
the `pi.extensions`/`pi.skills` fields for Pi — confirmed by reading
superpowers' own `package.json` and its `docs/porting-to-a-new-harness.md`
Part 5 Step 2). Without it, `.opencode/plugins/*.js` and
`.pi/extensions/*.ts` are just files nobody's told to load — the hard
requirement from that guide's Part 2 (auto-discovery, no per-session
opt-in) silently fails. Added `package.json` to Task 1's file list to close
this gap before Tasks 2-3 produce adapter code that would otherwise be
inert.

### Task 1: Manifests and docs (mechanical)

**Files:**
- Create: `.claude-plugin/marketplace.json`
- Create: `.agents/plugins/marketplace.json`
- Create: `.codex-plugin/plugin.json`
- Create: `.cursor-plugin/plugin.json`
- Create: `.devin-plugin/plugin.json`
- Create: `.kimi-plugin/plugin.json`
- Create: `gemini-extension.json`
- Create: `GEMINI.md`
- Create: `AGENTS.md` (symlink to `CLAUDE.md`)
- Create: `hooks/hooks-cursor.json`
- Create: `PORTING.md`
- Create: `README.kimi.md`
- Create: `README.opencode.md`
- Test: `scripts/test_harness_manifests.py`

**Interfaces:**
- Consumes: `.claude-plugin/plugin.json` (existing — name, description, version fields to mirror)
- Produces: nothing consumed by later tasks in this plan — Task 4 only needs these files to exist and be valid, not their contents

- [ ] **Step 1: Read the existing plugin identity**

Read `.claude-plugin/plugin.json` (name: `TBaguette`, version: `0.6.0`, description mentions "Seventy-four... skills"). Every manifest below reuses this name/description, adapted to each harness's schema — read the matching superpowers 6.3.0 file first in each case (paths given per file below) to see the exact schema shape, then substitute TBaguette's identity.

- [ ] **Step 2: `.claude-plugin/marketplace.json`**

Read `.../superpowers/6.3.0/.claude-plugin/marketplace.json` for shape. Write:

```json
{
  "name": "tbaguette-dev",
  "description": "Development marketplace for the TBaguette skills library",
  "owner": {
    "name": "verderosa2",
    "email": "verderosa2@gmail.com"
  },
  "plugins": [
    {
      "name": "TBaguette",
      "description": "Seventy-four project-, stack-, and language-agnostic skills: judgment, code comprehension, change discipline, testing, debugging, systems design, defensive security, communication, tooling — plus formidable, design craft for every UI stack.",
      "version": "0.6.0",
      "source": "./",
      "author": {
        "name": "verderosa2",
        "email": "verderosa2@gmail.com"
      }
    }
  ]
}
```

- [ ] **Step 3: `.agents/plugins/marketplace.json`**

Read `.../superpowers/6.3.0/.agents/plugins/marketplace.json` for shape. Write:

```json
{
  "name": "tbaguette-dev",
  "interface": {
    "displayName": "TBaguette Dev"
  },
  "plugins": [
    {
      "name": "TBaguette",
      "source": {
        "source": "url",
        "url": "./"
      },
      "policy": {
        "installation": "AVAILABLE",
        "authentication": "ON_INSTALL"
      },
      "category": "Developer Tools"
    }
  ]
}
```

- [ ] **Step 4: `.codex-plugin/plugin.json`**

Read `.../superpowers/6.3.0/.codex-plugin/plugin.json` for shape (includes an `interface` block with `defaultPrompt`, `brandColor`, icon paths). Write, dropping `composerIcon`/`logo`/`screenshots` (no raster art — see Global Constraints) and `interface.brandColor` (no established brand color):

```json
{
  "name": "TBaguette",
  "version": "0.6.0",
  "description": "Seventy-four project-, stack-, and language-agnostic skills: judgment, code comprehension, change discipline, testing, debugging, systems design, defensive security, communication, tooling.",
  "author": {
    "name": "verderosa2",
    "email": "verderosa2@gmail.com"
  },
  "homepage": "https://github.com/LeSplooch/tbaguette-skills",
  "repository": "https://github.com/LeSplooch/tbaguette-skills",
  "license": "MIT",
  "keywords": [
    "skills",
    "code-review",
    "debugging",
    "testing",
    "systems-design",
    "security",
    "naming",
    "workflow"
  ],
  "skills": "./skills/",
  "hooks": {},
  "interface": {
    "displayName": "TBaguette",
    "shortDescription": "Judgment calls, code comprehension, and change discipline for coding agents",
    "longDescription": "Use TBaguette for the craft between the ticket and the commit: reading unfamiliar code, naming, testing strategy, debugging, systems design, defensive security, and communicating the result.",
    "developerName": "verderosa2",
    "category": "Developer Tools",
    "capabilities": [
      "Interactive",
      "Read",
      "Write"
    ],
    "websiteURL": "https://lesplooch.github.io/tbaguette-skills/"
  }
}
```

Check the repo's actual license identifier before writing `"license"` — read `LICENSE` at the repo root and use whatever SPDX id matches (do not assume MIT without checking).

- [ ] **Step 5: `.cursor-plugin/plugin.json`**

Read `.../superpowers/6.3.0/.cursor-plugin/plugin.json` for shape. Write, pointing `hooks` at the new Cursor hook manifest from Step 9:

```json
{
  "name": "TBaguette",
  "displayName": "TBaguette",
  "description": "Seventy-four project-, stack-, and language-agnostic skills for judgment, code comprehension, change discipline, testing, debugging, systems design, defensive security, communication, and tooling.",
  "version": "0.6.0",
  "author": {
    "name": "verderosa2",
    "email": "verderosa2@gmail.com"
  },
  "homepage": "https://github.com/LeSplooch/tbaguette-skills",
  "repository": "https://github.com/LeSplooch/tbaguette-skills",
  "license": "MIT",
  "keywords": [
    "skills",
    "code-review",
    "debugging",
    "testing",
    "systems-design",
    "security"
  ],
  "skills": "./skills/",
  "hooks": "./hooks/hooks-cursor.json"
}
```

(Same license-id check as Step 4.)

- [ ] **Step 6: `.devin-plugin/plugin.json`**

Read `.../superpowers/6.3.0/.devin-plugin/plugin.json` for shape (no `skills`/`hooks` keys — Devin discovers `skills/` by convention). Write the same identity fields as Step 4 minus the `interface`/`skills`/`hooks` keys, matching that source file's shape exactly.

- [ ] **Step 7: `.kimi-plugin/plugin.json`**

Read `.../superpowers/6.3.0/.kimi-plugin/plugin.json` in full, including its `skillInstructions` string. Write the same identity fields as Step 4, plus:

```json
  "sessionStart": {
    "skill": "using-tbaguette"
  },
```

For `skillInstructions`: grep `skills/` for the Claude-Code-specific tool names TBaguette skills actually reference — run `grep -rlo -E '\b(Skill tool|Task tool|TodoWrite|AskUserQuestion)\b' skills/` from the repo root and read each hit in context. Write a `skillInstructions` string mapping only the tool names that search actually surfaces to Kimi Code's equivalents (per superpowers' own mapping: `Skill` tool stays `Skill`, `Task tool (general-purpose)` → Kimi's `Agent` tool, `TodoWrite` → `TodoList`, ask-the-user language → Kimi's `AskUserQuestion`). Do not invent mappings for tool names that don't actually appear — copying superpowers' full block verbatim would reference things TBaguette skills don't say.

- [ ] **Step 8: `gemini-extension.json` and `GEMINI.md`**

Read `.../superpowers/6.3.0/gemini-extension.json` and `.../superpowers/6.3.0/GEMINI.md`. Write:

`gemini-extension.json`:
```json
{
  "name": "TBaguette",
  "description": "Seventy-four project-, stack-, and language-agnostic skills for judgment, code comprehension, change discipline, testing, debugging, systems design, defensive security, communication, and tooling.",
  "version": "0.6.0",
  "contextFileName": "GEMINI.md"
}
```

`GEMINI.md` (mirrors superpowers' one-line pointer file — read its exact content first, adapt the plugin name and skill-tool-invocation phrasing to TBaguette's, keep it this short).

- [ ] **Step 9: `AGENTS.md` symlink**

```bash
cd /home/thisfuck/Code/tbaguette-skills
ln -s CLAUDE.md AGENTS.md
```

Verify: `ls -la AGENTS.md` shows it pointing at `CLAUDE.md`, and `cat AGENTS.md` prints `CLAUDE.md`'s actual content.

- [ ] **Step 10: `hooks/hooks-cursor.json`**

Read `.../superpowers/6.3.0/hooks/hooks-cursor.json` and TBaguette's existing `hooks/hooks.json` (Claude Code format) side by side. Write, keeping the same `run-hook.cmd session-start` invocation TBaguette's Claude Code hook already uses:

```json
{
  "version": 1,
  "hooks": {
    "sessionStart": [
      {
        "command": "./hooks/run-hook.cmd session-start"
      }
    ]
  }
}
```

- [ ] **Step 11: `PORTING.md`, `README.kimi.md`, `README.opencode.md`**

Read `.../superpowers/6.3.0/docs/porting-to-a-new-harness.md`, `.../docs/README.kimi.md`, `.../docs/README.opencode.md`. Write adapted versions at the TBaguette repo root (not under `docs/` — see Global Constraints), substituting: superpowers→TBaguette, `using-superpowers`→`using-tbaguette`, the repo URL, and the skill count (74). Drop any section that references superpowers' own dev/release tooling (version-bump script, pre-commit config) since TBaguette doesn't have those and isn't adding them.

- [ ] **Step 12: Write the manifest validity test**

Create `scripts/test_harness_manifests.py`:

```python
"""Validates every harness manifest this repo ships is well-formed and
internally consistent with .claude-plugin/plugin.json, the source of
truth for the plugin's name/version. Proportionate to "compatibility
layer" scope -- this is not a port of superpowers' own per-harness
tests/ suite, just enough to catch a malformed JSON file or a stale
version number before it ships.
"""
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
]


class TestHarnessManifests(unittest.TestCase):
    def test_all_manifests_are_valid_json(self):
        for rel_path in JSON_MANIFESTS:
            path = REPO_ROOT / rel_path
            with self.subTest(manifest=rel_path):
                self.assertTrue(path.is_file(), f"{rel_path} missing")
                with path.open() as f:
                    json.load(f)  # raises on malformed JSON

    def test_versions_match_plugin_json(self):
        plugin = json.load(open(REPO_ROOT / ".claude-plugin/plugin.json"))
        expected_version = plugin["version"]
        for rel_path in (
            ".codex-plugin/plugin.json",
            ".cursor-plugin/plugin.json",
            ".devin-plugin/plugin.json",
            ".kimi-plugin/plugin.json",
            "gemini-extension.json",
        ):
            data = json.load(open(REPO_ROOT / rel_path))
            with self.subTest(manifest=rel_path):
                self.assertEqual(data["version"], expected_version)

    def test_agents_md_symlinks_to_claude_md(self):
        agents_md = REPO_ROOT / "AGENTS.md"
        self.assertTrue(agents_md.is_symlink(), "AGENTS.md must be a symlink")
        self.assertEqual(agents_md.resolve(), (REPO_ROOT / "CLAUDE.md").resolve())


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 13: Run the new test standalone (not yet wired in)**

The test file includes `test_package_json_points_at_real_files`, which
checks that `package.json`'s `main` and `pi.extensions` fields point at
real files — true only after Task 2 (OpenCode) and Task 3 (Pi) create
those files. Since this task runs before those, wiring the suite into
`run_tests.py` now would make the full suite red for a reason that isn't a
real defect. Run it standalone instead:

Run: `cd scripts && python3 test_harness_manifests.py -v`
Expected: `test_all_manifests_are_valid_json`, `test_versions_match_plugin_json`,
and `test_agents_md_symlinks_to_claude_md` pass; `test_package_json_points_at_real_files`
fails (expected — the files it checks for don't exist until Tasks 2-3 run).

- [ ] **Step 14: Run the full suite, confirming nothing else broke**

Run: `python3 scripts/run_tests.py`
Expected: all *existing* suites still pass (this new test file isn't wired
in yet, so it doesn't run here and can't fail the suite).

- [ ] **Step 16: Commit and push**

```bash
git add .claude-plugin/marketplace.json .agents/plugins/marketplace.json \
  .codex-plugin/plugin.json .cursor-plugin/plugin.json .devin-plugin/plugin.json \
  .kimi-plugin/plugin.json gemini-extension.json GEMINI.md AGENTS.md \
  hooks/hooks-cursor.json PORTING.md README.kimi.md README.opencode.md \
  package.json scripts/test_harness_manifests.py
git commit -m "$(cat <<'EOF'
Add multi-harness discovery manifests (Codex, Cursor, Devin, Gemini,
Kimi, generic Agents marketplace)

Matches what superpowers 6.3.0 ships for the same harnesses, adapted
to TBaguette's identity and skills path. Compatibility-only -- none of
superpowers' own dev/release tooling is included; TBaguette already
covers that ground differently.
EOF
)"
git fetch origin master
# If origin/master has moved: git merge origin/master, resolve any
# docs/-confined conflicts per Global Constraints, rerun the suite.
git push origin master
```

---

### Task 2: OpenCode adapter

**Files:**
- Create: `.opencode/plugins/tbaguette.js`
- Create: `.opencode/INSTALL.md`

**Interfaces:**
- Consumes: nothing from Task 1
- Produces: nothing consumed by later tasks — independent of Task 3 and Task 4

- [ ] **Step 1: Read the source adapter**

Read `.../superpowers/6.3.0/.opencode/plugins/superpowers.js` in full (139 lines) and `.../superpowers/6.3.0/.opencode/INSTALL.md`. Note what it actually does: frontmatter extraction, path normalization, auto-registering the skills directory via a config hook, injecting bootstrap context (the `using-superpowers` skill's content) via a message transform.

- [ ] **Step 2: Port `.opencode/plugins/tbaguette.js`**

Copy the source file's structure and logic unchanged (frontmatter parsing and path normalization are generic, not superpowers-specific). Change only:
- File header comment: "Superpowers plugin" → "TBaguette plugin"
- Any hardcoded `superpowers` string used as the plugin/directory name → `TBaguette`
- The bootstrap skill it reads and injects: `skills/using-superpowers/SKILL.md` → `skills/using-tbaguette/SKILL.md`
- Any bootstrap message text that says "You have superpowers" / references `superpowers:<skill>` invocation syntax → "You have TBaguette" / `TBaguette:<skill>`, matching the phrasing TBaguette's own `hooks/session-start` already uses for the Claude Code hook (read that file for the exact wording to reuse)

- [ ] **Step 3: Port `.opencode/INSTALL.md`**

Same substitutions (superpowers→TBaguette, repo URL, skill count).

- [ ] **Step 4: Syntax-check**

Run: `node --check .opencode/plugins/tbaguette.js`
Expected: no output, exit code 0 (syntax valid — this environment has no OpenCode runtime to execute it against, so syntax validity plus manual diff-review against the source is the available verification).

---

### Task 3: Pi adapter

**Files:**
- Create: `.pi/extensions/tbaguette.ts`

**Interfaces:**
- Consumes: nothing from Task 1 or Task 2
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Read the source adapter**

Read `.../superpowers/6.3.0/.pi/extensions/superpowers.ts` in full (121 lines).

- [ ] **Step 2: Port `.pi/extensions/tbaguette.ts`**

Same substitution rules as Task 2 Step 2 (structure/logic unchanged, only naming and the bootstrap-skill pointer change).

- [ ] **Step 3: Syntax-check**

Run: `npx tsc --noEmit .pi/extensions/tbaguette.ts 2>&1 || true`
If `tsc` isn't available in this environment, fall back to `node --check` after stripping TypeScript-only syntax is not reliable — instead do a manual side-by-side diff against the source file confirming only the intended substitutions changed, and note in the commit message that this file is unexecuted/untested in this environment (matches this repo's own precedent: `README.md` already documents the PowerShell install command as "verified by careful construction," not machine-tested, when no runtime exists to test it).

---

### Task 4: Hermes adapter

**Files:**
- Create: `.hermes-plugin/__init__.py`
- Create: `.hermes-plugin/plugin.yaml`

**Interfaces:**
- Consumes: nothing from Tasks 1-3
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Read the source adapter**

Read `.../superpowers/6.3.0/.hermes-plugin/__init__.py` (104 lines) and `.../superpowers/6.3.0/.hermes-plugin/plugin.yaml`.

- [ ] **Step 2: Port `.hermes-plugin/plugin.yaml`**

```yaml
name: TBaguette
version: 0.6.0
description: TBaguette skills and workflow bootstrap for Hermes Agent
author: verderosa2
provides_hooks:
  - pre_llm_call
```

- [ ] **Step 3: Port `.hermes-plugin/__init__.py`**

Same substitution rules as Task 2 Step 2.

- [ ] **Step 4: Syntax-check**

Run: `python3 -m py_compile .hermes-plugin/__init__.py`
Expected: no output, exit code 0.

---

### Execution note (found during Task 4, addressed before Task 5)

The Hermes implementer's own report flagged a second gap the original scan
missed: `.hermes-plugin/__init__.py`'s `_build_bootstrap()` reads
`skills/using-tbaguette/references/hermes-tools.md` directly (a hard
runtime dependency, not optional) — mirroring `using-superpowers`'
`references/hermes-tools.md`, which `using-superpowers/SKILL.md` itself
points at via a "Platform Adaptation" section this repo's `using-tbaguette`
didn't yet have. Without the file, the Hermes plugin would raise
`FileNotFoundError` on first load. Fixed before Task 5: ported
`skills/using-tbaguette/references/hermes-tools.md` from superpowers'
source (tool-name mapping, instructions-file location, skill-invocation
and fallback-read syntax, all adapted to TBaguette's naming), and added a
matching "Platform adaptation" section to `using-tbaguette/SKILL.md`,
scoped honestly to only the one harness that actually has a reference file
today (Codex/Pi/Kimi don't need one — their mapping is either inline or
unnecessary, per `PORTING.md`'s reference table).

Separately: **Plan A and Plan B's parallel dispatches share the same
`skills/` tree**, and `scripts/content_pipeline.py` hard-gates on every
on-disk skill directory being registered in its own `CATEGORIES` constant
(a hand-maintained structure this plan's original scan never surfaced,
since Plan A doesn't touch it — that's `content_pipeline.py`'s equivalent
of `CATALOG.md`, and updating it is Plan B Task 16's job, not this one's).
While Plan B's 13 agents are still landing skills, `python3
scripts/run_tests.py` will show `content_pipeline.py` and `generate.py
integration` red for a reason entirely outside this plan — not a defect in
anything Task 1-4 produced. Verified this directly: temporarily relocated
Plan B's in-progress skill directories out of `skills/` to a scratch
location, confirmed all 8 suites pass clean in isolation (498 checks),
then restored them before proceeding. Task 5 below commits only this
plan's own files — no `skills/<new-superpowers-skill>/` directory is ever
staged from here, so this plan's commit itself is not affected by the
other plan's in-progress state, even though a live `run_tests.py` invoked
in between will show red until Plan B also finishes and integrates.

### Task 5: Integrate, test, version bump, commit, push, verify

**Files:**
- Modify: `.claude-plugin/plugin.json` (version bump)
- Modify: `.codex-plugin/plugin.json`, `.cursor-plugin/plugin.json`, `.devin-plugin/plugin.json`, `.kimi-plugin/plugin.json`, `gemini-extension.json`, `.claude-plugin/marketplace.json` (version field, to match)

**Interfaces:**
- Consumes: Task 1 (manifests, already committed), Task 2/3/4 (adapter files, staged but not committed)
- Produces: the completed, pushed harness-compatibility layer

- [ ] **Step 1: Verify Tasks 2-4's outputs are present and untracked**

Run: `git status --short .opencode/ .pi/ .hermes-plugin/`
Expected: three new untracked paths (`.opencode/`, `.pi/`, `.hermes-plugin/`).

- [ ] **Step 2: Bump the version**

Read the current `"version"` in `.claude-plugin/plugin.json` (may no longer be `0.6.0` if other work landed since Task 1 — this repo has concurrent activity, see Global Constraints). Bump the patch component by 1 (e.g. `0.6.0` → `0.6.1`). Apply the same new version string to the `"version"` field in every manifest from Task 1 that carries one: `.codex-plugin/plugin.json`, `.cursor-plugin/plugin.json`, `.devin-plugin/plugin.json`, `.kimi-plugin/plugin.json`, `gemini-extension.json`, and `.claude-plugin/marketplace.json`'s nested `plugins[0].version`.

- [ ] **Step 3: Wire `test_harness_manifests` into `run_tests.py`**

All four of its tests are true now that Tasks 2-4 created the OpenCode and
Pi adapter files. Read `scripts/run_tests.py`'s `SUITES` list and add an
entry following the existing pattern (label, then a `[sys.executable,
"test_harness_manifests.py"]` command — matches the plain-`unittest.main()`
style already used by other entries, whose "Ran N tests in" output the
runner's own regex already parses).

- [ ] **Step 4: Run the full test suite**

Run: `python3 scripts/run_tests.py`
Expected: all suites pass, including `test_harness_manifests`'s
version-match check against the bumped version and its
package.json-points-at-real-files check (now true).

- [ ] **Step 5: Commit**

```bash
git add .opencode/ .pi/ .hermes-plugin/ .claude-plugin/plugin.json \
  .codex-plugin/plugin.json .cursor-plugin/plugin.json .devin-plugin/plugin.json \
  .kimi-plugin/plugin.json gemini-extension.json .claude-plugin/marketplace.json \
  scripts/run_tests.py
git commit -m "$(cat <<'EOF'
Add OpenCode, Pi, and Hermes harness adapters; bump to <new-version>

Ports superpowers 6.3.0's own per-harness bootstrap adapters (skill
frontmatter parsing, directory auto-registration, context injection),
renamed and repointed at TBaguette's identity and using-tbaguette.
Kept as its own commit, separate from the pure-manifest additions,
since this is behavioral code rather than configuration.
EOF
)"
```

- [ ] **Step 6: Fetch, integrate if needed, push**

```bash
git fetch origin master
```

If `origin/master` has moved since Task 1's push: `git merge origin/master`, resolve any conflicts confined to `docs/` by regenerating (`git checkout --ours -- docs/ && python3 scripts/generate.py --base-path /tbaguette-skills`), rerun `python3 scripts/run_tests.py`, commit the merge, then:

```bash
git push origin master
```

- [ ] **Step 7: Verify the live Pages deploy**

```bash
gh api repos/LeSplooch/tbaguette-skills/pages/builds/latest
```

If the new commit SHA hasn't triggered a build within ~30s: `gh api repos/LeSplooch/tbaguette-skills/pages/builds -X POST`. Once built, confirm with a no-cache fetch that this doesn't affect the live site's rendered content (this plan touches no `docs/`-visible content — the check here is only that the deploy pipeline itself stayed healthy, not that anything changed on the page).

- [ ] **Step 8: Update the local install**

```bash
git -C ~/.claude/skills/TBaguette pull
```

If that directory doesn't exist on this machine, skip this step.
