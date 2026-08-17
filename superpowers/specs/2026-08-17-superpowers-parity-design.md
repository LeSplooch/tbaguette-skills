# Superpowers parity — design

## Goal

Two independent outcomes, shipped separately:

1. **Harness parity.** TBaguette becomes discoverable and loadable on every
   harness the `superpowers` plugin (v6.3.0) supports, not just Claude Code.
2. **Content parity.** The 13 skills `CATALOG.md` currently marks `†` as
   hand-offs to `superpowers` (or loose `~/.claude/skills/` copies) become
   real, TBaguette-native, enhanced skills — same names, so they're true
   drop-in replacements (`TBaguette:brainstorming`,
   `TBaguette:systematic-debugging`, etc.).

`using-superpowers` is the 14th superpowers core skill but is intentionally
**not** duplicated — `using-tbaguette` already fills that role and predates
this project.

## Why this is two projects, not one

They touch disjoint files (plugin/harness manifests vs. `skills/` content),
have different risk profiles (config addition vs. new judgment content that
needs to read as TBaguette's own voice), and can ship on independent
timelines. Bundling them into one commit would make either one harder to
review or revert without the other.

## Part 1 — Content parity: the 13 skills

Every row below currently exists in `CATALOG.md` as a `†` entry. The
category doesn't change — only the dagger drops and the description becomes
real. Four currently-combined rows split apart:
`requesting-code-review † · receiving-code-review †`,
`writing-plans † · executing-plans †`, and
`dispatching-parallel-agents † · subagent-driven-development †` each become
two individual rows, matching every other entry in the table. The fourth,
`writing-skills † · using-superpowers †`, collapses to a single
`writing-skills` row instead of two — `using-superpowers` doesn't get a new
row of its own here because `using-tbaguette` already covers that ground
under its own existing (non-dagger) entry in "Judgment and meta"; repeating
it in this row would be redundant, not a second hand-off.

| Skill | Category | Source (superpowers 6.3.0) | Adaptation |
|---|---|---|---|
| `brainstorming` | Judgment and meta | `SKILL.md` ~230 lines + `visual-companion.md` + `spec-document-reviewer-prompt.md` | Keep the dialogue-driven design process end to end (explore → clarify one-at-a-time → propose approaches → present in sections → write spec → self-review → user review → hand off to writing-plans). Drop the bundled Node.js visual-companion server — no TBaguette skill ships executable tooling. |
| `verification-before-completion` | Judgment and meta | 120 lines | Direct adaptation. |
| `using-git-worktrees` | Landing changes | 167 lines | Direct adaptation. |
| `finishing-a-development-branch` | Landing changes | 225 lines | Direct adaptation. |
| `test-driven-development` | Testing | 320 lines + `writing-good-tests.md` | Cross-reference `designing-test-data`, `choosing-test-scope`, `property-based-testing`, `characterization-testing`, `grounding-test-doubles` — TBaguette already owns the specialized testing judgment calls; this skill owns the red-green-refactor loop itself. |
| `systematic-debugging` | Debugging and performance | 283 lines + 10 reference files | Heaviest single skill. Keep the judgment content (root-cause-tracing, condition-based-waiting, defense-in-depth). Drop `find-polluter.sh` (executable) and the eval/test-pressure fixture files (meta, not content). Cross-reference `reading-stack-traces`, `bisecting-failures`, `flaky-test-triage`, `debugging-concurrency`. |
| `requesting-code-review` | Communicating | 95 lines + `code-reviewer.md` | Cross-reference `reviewing-code-deeply`. |
| `receiving-code-review` | Communicating | 205 lines | Cross-reference `reviewing-code-deeply`, `karen-and-the-manager`. |
| `writing-plans` | Communicating | 171 lines + `plan-document-reviewer-prompt.md` | Cross-reference `writing-adrs`, `estimating-effort`. |
| `executing-plans` | Communicating | 64 lines | Smallest; direct adaptation. |
| `dispatching-parallel-agents` | Environment and tooling | 167 lines | Direct adaptation. |
| `subagent-driven-development` | Environment and tooling | 568 lines + 6 reference/script files | Second-heaviest. Keep the delegation and review-loop judgment; drop the bundled `sdd-workspace`/`review-package`/`task-brief` scripts. |
| `writing-skills` | Environment and tooling | 679 lines + 6 reference files | Heaviest skill in the batch. Becomes the authoritative "how a TBaguette skill gets written" reference — grounded in this repo's actual conventions (frontmatter register, `CATALOG.md` placement, the ship pipeline in `tending-tbaguette`), not just superpowers' generic advice. Drop `render-graphs.js` and `graphviz-conventions.dot` (tooling-specific). |

**Format constraints** (matching all 74 existing skills, no exceptions):
prose-only `SKILL.md` plus adapted `reference/*.md` where the source has
real judgment content worth keeping; `name:` + `description:` frontmatter in
the existing "Use when A, B, or C. Covers D, E, F." register; no bundled
scripts, servers, or executable tooling.

### Execution mechanism

1. One shared brief — TBaguette's voice (terse, aphoristic openers,
   `## Overview` / `## When to use` shape, the "Not for: X (see
   other-skill)" cross-reference convention seen throughout the existing 74
   — plus, per skill, the exact superpowers 6.3.0 source paths to read and
   the target category/cross-references from the table above.
2. 13 parallel `general-purpose` agent dispatches, one per skill, each
   producing a draft `SKILL.md` (+ reference files where warranted) in the
   scratchpad. Each agent reads its own source directly rather than being
   handed pre-pasted content, to keep the brief itself small.
3. I personally read and integrate all 13 drafts into `skills/<name>/` —
   this pass is where voice consistency and real (not templated)
   cross-references get locked in, since I'm the one constant across 13
   independently-drafted files.
4. `CATALOG.md` updated: all 14 `†` marks tied to these skills removed (13
   skills gain real rows; `using-superpowers`'s mention is dropped as
   redundant with the existing `using-tbaguette` row, not replaced), and the
   four combined rows become seven — three split into two rows each, the
   `writing-skills`/`using-superpowers` row collapses to one.
5. One consolidated `TBaguette:karen-and-the-manager` pass over the whole
   batch — not 13 separate passes.
6. `EXPECTED_SKILL_COUNT` in `scripts/generate.py`: 74 → 87. Grep sweep for
   every other place "74" is written — already confirmed to include
   `README.md`, `CATALOG.md`'s own header, `.claude-plugin/plugin.json`'s
   description, `scripts/content_pipeline.py` (docstring prose),
   `scripts/test_content_pipeline.py`, and `scripts/test_generate.py` — per
   `tending-tbaguette`'s explicit instruction. The list is illustrative, not
   exhaustive; the grep at implementation time is the actual check.
7. `.claude-plugin/plugin.json` version bump: `0.6.1` → `0.7.0` (see
   versioning section below), description updated from "Seventy-four" to
   "Eighty-seven."
8. Translation: one dedicated Sonnet agent call translating all 13
   descriptions into the 12 existing locales (`ar`, `de`, `es`, `fr`, `hi`,
   `it`, `ja`, `ko`, `pt`, `ru`, `tr`, `zh`), writing
   `i18n/<locale>/descriptions.json` updates. No new category, so
   `categories.json` files are untouched. Scope discipline: only these 13
   keys — not a backfill of any locale's pre-existing gaps.
9. `python3 scripts/run_tests.py` — must be fully green.
10. `python3 scripts/generate.py --base-path /tbaguette-skills` — confirm
    via `git status`/`git diff --stat` on `docs/` that only the expected
    pages changed (13 new skill pages + `docs/index.html`).
11. One commit for the whole batch (matches "ship as a batch"). Push.
12. Verify the live Pages deploy with a no-cache fetch. Pull the update into
    `~/.claude/skills/TBaguette`.

## Part 2 — Harness-compatibility layer

Scope: **compatibility only** — whatever makes another harness actually
*discover and load* TBaguette. Explicitly not in scope: superpowers' own
internal dev/release tooling (`.pre-commit-config.yaml`,
`scripts/bump-version.sh`, per-harness `tests/`, `RELEASE-NOTES.md`,
`CODE_OF_CONDUCT.md`, `.github/ISSUE_TEMPLATE/`) — TBaguette already covers
that ground differently, via `run_tests.py` and this repo's own ship
pipeline.

| Piece | Current state | Action |
|---|---|---|
| `hooks/hooks.json`, `hooks/run-hook.cmd`, `hooks/session-start` | Already ported — structurally matches superpowers' own polyglot Windows/Unix wrapper | None |
| `hooks/hooks-cursor.json` | Missing | Add: same `run-hook.cmd session-start` invocation, Cursor's hook-manifest shape |
| `.codex-plugin/plugin.json` | Missing | Add, adapted (name, description, `skills: "./skills/"`, no Codex-specific `interface` branding beyond what TBaguette already has assets for) |
| `.cursor-plugin/plugin.json` | Missing | Add, adapted, points `hooks` at `./hooks/hooks-cursor.json` |
| `.devin-plugin/plugin.json` | Missing | Add, adapted |
| `.kimi-plugin/plugin.json` | Missing | Add, adapted, including a `skillInstructions` block mapping the (few) Claude-Code-specific tool references in TBaguette skills to Kimi's tool names |
| `gemini-extension.json` + `GEMINI.md` | Missing | Add |
| `AGENTS.md` | Missing | Add as a symlink to `CLAUDE.md`, matching superpowers exactly |
| `.agents/plugins/marketplace.json`, `.claude-plugin/marketplace.json` | Missing | Add — enables `claude plugin marketplace add` as an alternate install path; a real discoverability gain, not dev tooling |
| `.opencode/plugins/*.js` | Missing | Port and adapt (~139 lines): frontmatter parsing, skill-dir auto-registration, context injection via message transform. Rename superpowers→TBaguette, point at `using-tbaguette` instead of `using-superpowers`. |
| `.pi/extensions/*.ts` | Missing | Port and adapt (~121 lines), same treatment |
| `.hermes-plugin/{__init__.py,plugin.yaml}` | Missing | Port and adapt (~104 lines) |
| `docs/porting-to-a-new-harness.md`, `README.kimi.md`, `README.opencode.md` | N/A | **Naming collision, resolved**: superpowers keeps these under its own hand-authored `docs/`; TBaguette's `docs/` is the *generated site*, fully owned by `generate.py`. These land at repo root instead — `PORTING.md`, `README.kimi.md`, `README.opencode.md` — as peers of `README.md`. |
| Icon/brand assets | Partial | Reuse/adapt the existing `docs/assets/favicon.svg` for harness manifests that want a small icon reference; skip anything needing new raster artwork (no `app-icon.png` equivalent) |

### Execution mechanism

Sequential, not parallel — these files are small in count (~15) and mostly
depend on the same handful of facts (plugin name, description, skills path,
repo URL), so a single pass is both faster and more consistent than
dispatching agents per file.

1. Manifests + docs (mechanical: JSON files, `AGENTS.md` symlink,
   `PORTING.md`, the two harness READMEs, marketplace files,
   `hooks-cursor.json`).
2. Harness adapter runtime code (`opencode/plugins/*.js`,
   `.pi/extensions/*.ts`, `.hermes-plugin/`) — kept as a separate step and a
   separate commit since it's behavioral, not just config, and worth being
   independently revertable.
3. `python3 scripts/run_tests.py`.
4. Two commits (manifests+docs, then adapter code), pushed in sequence.

## Versioning

- Harness layer: `0.6.0` → `0.6.1` (patch — doesn't change skill count or
  count-derived files).
- Content batch: `0.6.1` → `0.7.0` (minor — 74 → 87 skills).

These ship as separate commits/pushes in that order (harness layer first,
since it's the lower-risk, purely-additive change).

## Explicitly out of scope

- Backfilling any locale's pre-existing translation gaps unrelated to these
  13 new skills (`tending-tbaguette`'s own scope-discipline rule).
- Any bundled executable tooling from the source skills (visual-companion
  server, `sdd-workspace` scripts, `render-graphs.js`) — no TBaguette skill
  ships code today; not introducing the first one here.
- The `.claude/worktrees/website-i18n` worktree (branch `fr-using-tbaguette`)
  — unrelated in-progress work, left untouched.
- Superpowers' own internal dev/release tooling (pre-commit config,
  version-bump script, per-harness test suites, issue templates,
  `RELEASE-NOTES.md`, `CODE_OF_CONDUCT.md`) — TBaguette already has
  different-but-equivalent mechanisms.

## Testing

- Existing suite (`run_tests.py`) already validates frontmatter format and
  skill count against `EXPECTED_SKILL_COUNT` for every skill in `skills/` —
  the 13 new skills get this coverage for free once the count is bumped.
- New: a light JSON-validity check for the new harness manifest files
  (`.codex-plugin/plugin.json`, `.cursor-plugin/plugin.json`, etc.) —
  proportionate to "compatibility layer," not a full port of superpowers'
  per-harness `tests/` suite.
- `i18n` key-parity test already catches unknown slugs; per
  `tending-tbaguette`'s documented caveat it does *not* catch a missing
  translation, so the translation step above is the actual enforcement.

## Sequencing

Harness-compatibility layer ships first (Part 2, two commits) — it's
additive-only, zero risk to existing content, and doesn't block on the
content work. Content parity (Part 1) ships second, as one batch commit,
once all 13 skills are drafted, integrated, gated through
`karen-and-the-manager`, and green on `run_tests.py`.
