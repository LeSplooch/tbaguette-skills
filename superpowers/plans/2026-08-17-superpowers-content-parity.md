# Superpowers Content Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Tasks 1-13 are mutually independent and safe to dispatch in parallel — each writes only to its own new `skills/<name>/` directory. Tasks 14-18 are strictly sequential and each depends on all of Tasks 1-13 being complete.

**Goal:** Build TBaguette-native, enhanced replacements for the 13 skills `CATALOG.md` currently marks `†` as hand-offs to the `superpowers` plugin — same names, so they're true drop-in replacements — and ship them as one batch.

**Architecture:** Each of the 13 skills is drafted independently (Tasks 1-13, parallelizable), then integrated, gated through `karen-and-the-manager`, translated, tested, and shipped as a single commit (Tasks 14-18).

**Tech Stack:** Markdown (`SKILL.md` + `reference/*.md`), Python (`scripts/generate.py`/`run_tests.py`, unchanged logic, just new content), JSON (`i18n/*/descriptions.json`).

## Global Constraints

- **Format** (matching all 74 existing skills, no exceptions): prose-only `SKILL.md` plus adapted `reference/*.md` where the source has real judgment content worth keeping. No bundled scripts, servers, or executable tooling — not even adapted ones. An inline fenced code block *within* prose (e.g. a `dot` process diagram, a short shell snippet as an example) is fine; a separate script *file* is not.
- **Frontmatter**: exactly `name:` and `description:`, nothing else. `description:` in the existing register: "Use when A, B, or C. Covers D, E, F." — restate the source skill's actual trigger conditions in TBaguette's own words; do not translate the source sentence literally.
- **Small reviewer-prompt source files fold inline.** Several source skills ship a separate `*-reviewer-prompt.md` (self-review checklists for specs/plans). TBaguette's convention is to inline this kind of checklist directly into the main `SKILL.md`'s own process section rather than as a separate file — matching how `writing-plans`' own "Self-Review" section (which you just read, to load this skill) is already inline, and how this session's own brainstorming spec self-review ran inline rather than as a separate document.
- **Substantial reference content stays as `reference/*.md`**, mirroring the `formidable` skill's pattern (`skills/formidable/reference/*.md`) — used only when a source skill's supporting files carry real, non-duplicated judgment content too large to fold into the main file.
- **Do not modify** `scripts/generate.py`, `CATALOG.md`, or run `python3 scripts/run_tests.py` from within Tasks 1-13. The skill count constant and the catalog are shared state — touching them from 13 parallel tasks would race. That integration work is Task 16 only.
- **`using-superpowers` is not duplicated** — `using-tbaguette` already covers that role. Nothing in this plan creates a `skills/using-superpowers/` directory.
- **Cross-references** use the exact form already established throughout the existing 74 skills: a `## When to use` line reading `Not for: <case> (see <skill-name>)`, and inline backtick-quoted skill names elsewhere in prose where genuinely relevant — not a bolted-on "See also" list at the bottom.
- **Locales for translation** (Task 17): exactly the 12 in `scripts/locales.py`'s `LOCALES` — `ar`, `de`, `es`, `fr`, `hi`, `it`, `ja`, `ko`, `pt`, `ru`, `tr`, `zh`. Skip `en` (the source language). Do not backfill any locale's pre-existing gaps from before this project.
- **Superpowers 6.3.0 source** is read-only reference at `/home/thisfuck/.claude/plugins/cache/claude-plugins-official/superpowers/6.3.0/skills/<name>/` — never edit anything under that path.
- **Concurrent writers**: this repo has other agents/worktrees pushing to `origin/master` (confirmed mid-session: a `website-i18n` worktree merged 27 commits during this project's design phase). Before Task 18's push, `git fetch origin master` and check for divergence; if it moved, `git merge origin/master`, resolve any conflicts confined to the generated `docs/` tree by regenerating (`git checkout --ours -- docs/ && python3 scripts/generate.py --base-path /tbaguette-skills`), rerun the full suite, then push. Do not touch `.claude/worktrees/website-i18n` itself.

---

## Task Pattern (Tasks 1-13 each instantiate this)

Every one of Tasks 1-13 follows the same five steps; only the parameters (source path, category, cross-references, keep/drop list) differ, and each task below states its own parameters in full.

1. **Read the source.** The exact superpowers 6.3.0 file(s) listed in the task.
2. **Read TBaguette voice exemplars.** Always `skills/orienting-in-unfamiliar-code/SKILL.md` and `skills/karen-and-the-manager/SKILL.md` (baseline register), plus the task's category-specific exemplar(s).
3. **Write `skills/<name>/SKILL.md`**: frontmatter, `## Overview`, `## When to use` (including the task's named cross-references in "Not for: X (see Y)" form), then the adapted body — judgment content restated in TBaguette's own words, not copied.
4. **Write `reference/*.md`** only for the files the task marks "keep as reference" — fold everything marked "fold inline" into step 3's `SKILL.md` instead; skip everything marked "drop."
5. **Self-check**: frontmatter is exactly `name:`/`description:`; no bundled scripts/servers survived; every named cross-reference actually appears in the text; nothing from the "drop" list leaked in.

---

### Task 1: `brainstorming`

**Files:** Create `skills/brainstorming/SKILL.md`
**Category:** Judgment and meta
**Interfaces:** Consumes nothing. Produces `skills/brainstorming/SKILL.md`, consumed by Task 14.

- Source: `.../skills/brainstorming/SKILL.md` (~230 lines), `.../skills/brainstorming/visual-companion.md`, `.../skills/brainstorming/spec-document-reviewer-prompt.md`
- Category exemplar (read in addition to the universal two): `skills/reading-specifications/SKILL.md` (closest existing TBaguette analog — turning ambiguous input into concrete requirements)
- Keep: the full dialogue-driven process (explore context → clarify one question at a time → propose 2-3 approaches → present design in sections with per-section approval → write spec doc → self-review → user reviews spec → hand off to writing-plans)
- Fold inline: `spec-document-reviewer-prompt.md`'s checklist, as the `SKILL.md`'s own self-review section (see Global Constraints)
- Drop entirely: `visual-companion.md` and any reference to the bundled Node.js visual-companion server — no TBaguette skill ships executable tooling. If the source's "offer the visual companion" concept has genuine judgment worth keeping (deciding *when* a question is visual vs textual), restate that judgment call in prose without the tool-specific mechanics (server, `--open` flag, port, etc.)
- Cross-references: `Not for: turning an already-clear requirement into tasks (see writing-plans)`; mention `reading-specifications` for the ambiguous-input-parsing overlap and how brainstorming differs (exploring what to build at all, vs. reading down a spec that already exists)
- Spec doc location note: state that this skill's own default spec-output location is `superpowers/specs/YYYY-MM-DD-<topic>-design.md` at the repo root (matching this very session's convention — the design doc for this project lives there), while noting user preference overrides it

---

### Task 2: `verification-before-completion`

**Files:** Create `skills/verification-before-completion/SKILL.md`
**Category:** Judgment and meta
**Interfaces:** Consumes nothing. Produces `skills/verification-before-completion/SKILL.md`, consumed by Task 14.

- Source: `.../skills/verification-before-completion/SKILL.md` (120 lines)
- Category exemplar: `skills/calibrating-confidence/SKILL.md` (closest existing analog — verified vs. inferred vs. assumed)
- Keep: the full "evidence before assertions" content — running verification commands and reading actual output before claiming success, the specific anti-patterns (assuming a fix worked, trusting a partial test run, claiming "done" without a fresh check)
- Cross-references: `Not for: judging whether a decision was actually correct after the fact (see revalidating-decisions)`; mention `calibrating-confidence` for the adjacent-but-distinct concern (marking uncertainty vs. actually verifying before claiming completion)

---

### Task 3: `using-git-worktrees`

**Files:** Create `skills/using-git-worktrees/SKILL.md`
**Category:** Landing changes
**Interfaces:** Consumes nothing. Produces `skills/using-git-worktrees/SKILL.md`, consumed by Task 14.

- Source: `.../skills/using-git-worktrees/SKILL.md` (167 lines)
- Category exemplar: `skills/atomic-commits/SKILL.md`
- Keep: the full content — when isolation is warranted, native-tool vs. `git worktree` fallback decision, cleanup discipline
- Cross-references: mention `using-superpowers`'s absence is intentional (nothing to cross-reference there); `Not for: deciding whether to merge or discard finished work (see finishing-a-development-branch)`

---

### Task 4: `finishing-a-development-branch`

**Files:** Create `skills/finishing-a-development-branch/SKILL.md`
**Category:** Landing changes
**Interfaces:** Consumes nothing. Produces `skills/finishing-a-development-branch/SKILL.md`, consumed by Task 14.

- Source: `.../skills/finishing-a-development-branch/SKILL.md` (225 lines)
- Category exemplar: `skills/refactoring-safely/SKILL.md`, `skills/atomic-commits/SKILL.md`
- Keep: the full integration-decision content (merge vs. rebase vs. squash, when a branch is actually done, cleanup)
- Cross-references: `Not for: isolating work before it starts (see using-git-worktrees)`; mention `atomic-commits` for how the branch's commit history should already look by the time this skill applies

---

### Task 5: `test-driven-development`

**Files:** Create `skills/test-driven-development/SKILL.md`, and `skills/test-driven-development/reference/writing-good-tests.md` **only if** step 5 below finds non-duplicated content
**Category:** Testing
**Interfaces:** Consumes nothing. Produces `skills/test-driven-development/SKILL.md`, consumed by Task 14.

- Source: `.../skills/test-driven-development/SKILL.md` (320 lines), `.../skills/test-driven-development/writing-good-tests.md`
- Category exemplars: `skills/designing-test-data/SKILL.md`, `skills/choosing-test-scope/SKILL.md`, `skills/characterization-testing/SKILL.md`
- Keep: the red-green-refactor cycle itself — writing a failing test first, minimal implementation, refactor, the discipline of never writing production code without a failing test demanding it
- Judgment call before writing `reference/writing-good-tests.md`: read it against TBaguette's existing `designing-test-data`, `property-based-testing`, `grounding-test-doubles` — these already own builders-over-fixtures, generators/shrinking, and fixture provenance respectively. Only keep source content that isn't already better covered by one of those; fold what little remains inline rather than creating a thin reference file
- Cross-references: `Not for: which test level catches a given bug (see choosing-test-scope)`, `Not for: what makes a generated or fixture value trustworthy (see designing-test-data, property-based-testing)`; this skill owns *the loop*, not test design — say so explicitly

---

### Task 6: `systematic-debugging`

**Files:** Create `skills/systematic-debugging/SKILL.md`, `skills/systematic-debugging/reference/root-cause-tracing.md`, `skills/systematic-debugging/reference/condition-based-waiting.md`, `skills/systematic-debugging/reference/defense-in-depth.md`
**Category:** Debugging and performance
**Interfaces:** Consumes nothing. Produces `skills/systematic-debugging/SKILL.md` + 3 reference files, consumed by Task 14.

- Source: `.../skills/systematic-debugging/SKILL.md` (283 lines) plus the three reference files named above
- Category exemplars: `skills/bisecting-failures/SKILL.md`, `skills/reading-stack-traces/SKILL.md`, `skills/debugging-concurrency/SKILL.md`
- Keep: the hypothesis-driven loop itself (reproduce → hypothesize → test the hypothesis → fix → verify), plus the three reference files' content, adapted
- Drop entirely: `find-polluter.sh` (executable), `CREATION-LOG.md` (meta, about the source skill's own authoring history, not content), `test-academic.md`, `test-pressure-1.md`, `test-pressure-2.md`, `test-pressure-3.md` (eval fixtures for testing the *source* skill, not skill content), `condition-based-waiting-example.ts` (code example — restate its lesson in prose or a short inline fenced snippet if genuinely needed, not as a separate file)
- Cross-references: this is the heaviest cross-reference set of the batch — `Not for: which specific technique to reach for once you're in the loop (see bisecting-failures, reading-stack-traces, debugging-concurrency, finding-resource-leaks, flaky-test-triage)`; be explicit that this skill owns the general loop and the specialized ones own specific terrain within it

---

### Task 7: `requesting-code-review`

**Files:** Create `skills/requesting-code-review/SKILL.md`
**Category:** Communicating
**Interfaces:** Consumes nothing. Produces `skills/requesting-code-review/SKILL.md`, consumed by Task 14.

- Source: `.../skills/requesting-code-review/SKILL.md` (95 lines), `.../skills/requesting-code-review/code-reviewer.md`
- Category exemplar: `skills/reviewing-code-deeply/SKILL.md`, `skills/explaining-technical-work/SKILL.md`
- Keep: what makes a review request actually reviewable (context a reviewer needs, what to flag proactively, when to request one at all)
- Fold inline: `code-reviewer.md`'s brief-a-reviewer content, as guidance within the main `SKILL.md` on what to hand a reviewer (human or subagent) rather than a separate prompt-template file
- Cross-references: `Not for: how to actually review — that's the reviewer's job (see reviewing-code-deeply)`; mention `receiving-code-review` as the other half of this pair

---

### Task 8: `receiving-code-review`

**Files:** Create `skills/receiving-code-review/SKILL.md`
**Category:** Communicating
**Interfaces:** Consumes nothing. Produces `skills/receiving-code-review/SKILL.md`, consumed by Task 14.

- Source: `.../skills/receiving-code-review/SKILL.md` (205 lines)
- Category exemplar: `skills/reviewing-code-deeply/SKILL.md`, `skills/karen-and-the-manager/SKILL.md` (for the "technical rigor over performative agreement" register)
- Keep: the core stance — verify feedback technically before implementing it, push back on a suggestion that's actually wrong rather than complying performatively, distinguish "I disagree and here's why" from "I don't understand"
- Cross-references: `Not for: producing the review in the first place (see reviewing-code-deeply, requesting-code-review)`; mention `karen-and-the-manager` for the adjacent but distinct case of a *self*-review pass finding nothing (this skill is about receiving feedback from someone else)

---

### Task 9: `writing-plans`

**Files:** Create `skills/writing-plans/SKILL.md`
**Category:** Communicating
**Interfaces:** Consumes nothing. Produces `skills/writing-plans/SKILL.md`, consumed by Task 14.

- Source: `.../skills/writing-plans/SKILL.md` (171 lines), `.../skills/writing-plans/plan-document-reviewer-prompt.md`
- Category exemplar: `skills/writing-adrs/SKILL.md`, `skills/estimating-effort/SKILL.md`
- Keep: the full task-decomposition discipline — file structure before tasks, bite-sized steps, no placeholders, the plan document header/task structure conventions. (You are executing a plan built from exactly this skill's own logic right now — use this plan document itself, and the harness-compatibility-layer plan alongside it, as a live example of the target shape when deciding how concrete to make the adapted content.)
- Fold inline: `plan-document-reviewer-prompt.md`'s checklist, as the `SKILL.md`'s own self-review section
- Cross-references: `Not for: whether a plan's effort estimate is realistic (see estimating-effort)`, `Not for: recording why a decision was made a particular way (see writing-adrs)`; mention `executing-plans` as the other half of this pair

---

### Task 10: `executing-plans`

**Files:** Create `skills/executing-plans/SKILL.md`
**Category:** Communicating
**Interfaces:** Consumes nothing. Produces `skills/executing-plans/SKILL.md`, consumed by Task 14.

- Source: `.../skills/executing-plans/SKILL.md` (64 lines — smallest in the batch)
- Category exemplar: `skills/writing-adrs/SKILL.md` (for brevity calibration — this should stay short, matching its source's own size)
- Keep: the checkpoint/review-batch execution model for working through an existing plan in the current session
- Cross-references: `Not for: authoring the plan (see writing-plans)`, `Not for: per-task fresh-subagent execution (see subagent-driven-development)` — this skill is specifically the *inline, same-session* execution mode

---

### Task 11: `dispatching-parallel-agents`

**Files:** Create `skills/dispatching-parallel-agents/SKILL.md`
**Category:** Environment and tooling
**Interfaces:** Consumes nothing. Produces `skills/dispatching-parallel-agents/SKILL.md`, consumed by Task 14.

- Source: `.../skills/dispatching-parallel-agents/SKILL.md` (167 lines)
- Category exemplar: none in TBaguette owns delegation patterns yet — use the universal baseline only, plus `skills/managing-scope-drift/SKILL.md` for the adjacent judgment-call register
- Keep: identifying genuinely independent work, avoiding shared-state collisions between parallel dispatches, batching results
- Cross-references: `Not for: single-task delegation with a review loop (see subagent-driven-development)`; this skill itself is a live example of its own subject in this session — Tasks 1-13 of this plan are exactly the pattern it describes

---

### Task 12: `subagent-driven-development`

**Files:** Create `skills/subagent-driven-development/SKILL.md`, `skills/subagent-driven-development/reference/implementer-prompt.md`, `skills/subagent-driven-development/reference/reviewer-prompt.md`
**Category:** Environment and tooling
**Interfaces:** Consumes nothing. Produces `skills/subagent-driven-development/SKILL.md` + 2 reference files, consumed by Task 14.

- Source: `.../skills/subagent-driven-development/SKILL.md` (568 lines — second-heaviest), `.../skills/subagent-driven-development/implementer-prompt.md`, `.../skills/subagent-driven-development/re-review-prompt.md`, `.../skills/subagent-driven-development/task-reviewer-prompt.md`
- Category exemplar: universal baseline only, plus `skills/dispatching-parallel-agents` (Task 11's own output — read it if already drafted; if executing in parallel and it isn't done yet, use the source superpowers version instead for the boundary-drawing) for how the two skills divide the delegation-pattern space
- Keep: the fresh-subagent-per-task loop, the two-stage review model (implementer, then reviewer), how a task brief gets handed to an implementer subagent with zero prior context
- Reference files: read `implementer-prompt.md`, `re-review-prompt.md`, and `task-reviewer-prompt.md` together; if `re-review-prompt.md` and `task-reviewer-prompt.md` are substantively the same role at two points in the loop, merge them into one `reference/reviewer-prompt.md` rather than keeping a near-duplicate; keep `implementer-prompt.md` as `reference/implementer-prompt.md`
- Drop entirely: `scripts/review-package`, `scripts/sdd-workspace`, `scripts/task-brief` (executable tooling)
- Cross-references: `Not for: 2+ genuinely independent tasks with no review loop needed between them (see dispatching-parallel-agents)`; mention `writing-plans` as the typical upstream input (a plan this skill then executes task-by-task)

---

### Task 13: `writing-skills`

**Files:** Create `skills/writing-skills/SKILL.md`, `skills/writing-skills/reference/anthropic-best-practices.md`, `skills/writing-skills/reference/persuasion-principles.md`, `skills/writing-skills/reference/testing-skills-with-subagents.md`
**Category:** Environment and tooling
**Interfaces:** Consumes nothing. Produces `skills/writing-skills/SKILL.md` + 3 reference files, consumed by Task 14 and by Task 16 (which needs this skill's own guidance to be accurate about the frontmatter/`CATALOG.md`/ship-pipeline conventions it documents).

- Source: `.../skills/writing-skills/SKILL.md` (679 lines — heaviest in the batch), `.../skills/writing-skills/anthropic-best-practices.md`, `.../skills/writing-skills/examples/CLAUDE_MD_TESTING.md`, `.../skills/writing-skills/persuasion-principles.md`, `.../skills/writing-skills/testing-skills-with-subagents.md`
- Category exemplar: none — this skill IS the exemplar-of-exemplars. Instead, read `CATALOG.md` in full and at least 4 existing `skills/*/SKILL.md` files spanning different sizes (a short one, e.g. `atomic-commits`; a medium one, e.g. `karen-and-the-manager`; the largest, `formidable/SKILL.md` plus one of its `reference/` files) to characterize the actual range this skill needs to teach
- Keep: everything about what makes a skill worth writing at all, the frontmatter/description register, testing a skill with subagents before shipping it, the persuasion-principles content on why a skill needs to be *compelling* enough that an agent actually invokes it rather than skipping past it
- This becomes the authoritative "how to write a TBaguette skill" reference — go further than a straight port: it should describe *this actual repo's* conventions specifically — `name:`/`description:` frontmatter register, `reference/*.md` used only when content doesn't fit inline (per Global Constraints above), placement in `CATALOG.md`'s category tables, and a pointer to `tending-tbaguette`'s ship pipeline (bump `EXPECTED_SKILL_COUNT`, translate, `karen-and-the-manager` gate, test, regenerate, commit) as the actual mechanism a new skill goes from draft to shipped in this specific repo — not just superpowers' generic advice
- Drop entirely: `render-graphs.js`, `graphviz-conventions.dot` (the rendering *tooling*). If the source content's advice to use an inline `dot` process-diagram in a skill's prose is worth keeping as a technique (TBaguette's own `brainstorming`, drafted in Task 1, uses exactly this technique inline), keep *that* advice in prose — a fenced code block as an illustration, not the rendering pipeline
- Reference files: `CLAUDE_MD_TESTING.md` under `examples/` in the source — decide whether its content is genuinely a worked example worth keeping (as `reference/testing-example.md`, adapted) or redundant with `testing-skills-with-subagents.md`'s own content; don't keep both if they overlap
- Cross-references: mention `naming-things` for the frontmatter-naming overlap; `Not for: using an already-written skill (see using-tbaguette)`

---

### Task 14: Integrate drafts and update `CATALOG.md`

**Files:**
- Modify: `CATALOG.md`
- Verify (no changes expected, but confirm present): all `skills/<name>/` directories from Tasks 1-13

**Interfaces:**
- Consumes: all outputs of Tasks 1-13
- Produces: a `CATALOG.md` with all 13 daggers for these skills removed, ready for Task 15's quality pass

- [ ] **Step 1: Confirm all 13 drafts exist**

```bash
for n in brainstorming verification-before-completion using-git-worktrees \
  finishing-a-development-branch test-driven-development systematic-debugging \
  requesting-code-review receiving-code-review writing-plans executing-plans \
  dispatching-parallel-agents subagent-driven-development writing-skills; do
  test -f "skills/$n/SKILL.md" && echo "ok: $n" || echo "MISSING: $n"
done
```
Expected: 13 `ok:` lines, zero `MISSING:`.

- [ ] **Step 2: Read every new `SKILL.md`'s frontmatter**

```bash
for n in brainstorming verification-before-completion using-git-worktrees \
  finishing-a-development-branch test-driven-development systematic-debugging \
  requesting-code-review receiving-code-review writing-plans executing-plans \
  dispatching-parallel-agents subagent-driven-development writing-skills; do
  echo "=== $n ==="; sed -n '/^description:/p' "skills/$n/SKILL.md"
done
```
Read each description. This is the exact text going into `CATALOG.md`'s "For" column (compressed to a phrase) and Task 17's translation input — accuracy here matters.

- [ ] **Step 3: Read every new `SKILL.md` in full for voice consistency**

Thirteen independently-dispatched drafts, even against a shared brief, will drift from each other more than a single author would. Read all 13 full files back to back. Fix directly, in place, anything that reads as off-register against the two universal exemplars (`orienting-in-unfamiliar-code`, `karen-and-the-manager`): a stray first-person aside, a heading structure that doesn't match `## Overview` / `## When to use` / body, a cross-reference named in one task's spec that didn't make it into the actual prose, a dropped-content item (a script, a prompt-template file) that still gets referenced as if it exists. This is an ordinary editorial pass — Task 15's `karen-and-the-manager` gate is a separate, adversarial pass on top of this, not a replacement for it.

- [ ] **Step 4: Edit `CATALOG.md`**

Open `CATALOG.md`. For each of the 6 standalone `†` rows (`brainstorming`, `verification-before-completion`, `using-git-worktrees`, `finishing-a-development-branch`, `test-driven-development`, `systematic-debugging`): remove the `†`, replace the "For" column with a short phrase derived from the skill's real `description:` (matching the terse register every other row already uses).

For the 4 combined rows: split `requesting-code-review † · receiving-code-review †` into two rows; split `writing-plans † · executing-plans †` into two rows; split `dispatching-parallel-agents † · subagent-driven-development †` into two rows; replace `writing-skills † · using-superpowers †` with a single `writing-skills` row (drop the `using-superpowers` mention — it's redundant with the existing `using-tbaguette` row, not a second hand-off, per the design spec).

Update the skill count in `CATALOG.md`'s own header line: "74 skills" → "87 skills".

- [ ] **Step 5: Verify no other `†` marks reference these 13 names**

```bash
grep -n '†' CATALOG.md
```
Expected: only `impeccable`, `design-system`, and `play-console` remain marked `†` (unrelated hand-offs, out of scope — see the design spec's "Explicitly out of scope").

---

### Task 15: Quality gate

**Files:** Potentially modifies any of the 13 `SKILL.md`/`reference/*.md` files from Tasks 1-13, and `CATALOG.md`

**Interfaces:**
- Consumes: Task 14's output
- Produces: the batch, post-fixes, ready for Task 16

- [ ] **Step 1: Invoke the quality gate**

Invoke `TBaguette:karen-and-the-manager` against the full batch (all 13 new skills + the `CATALOG.md` changes together, not 13 separate passes). Focus areas worth specifically provoking: voice consistency across 13 independently-drafted files (the exact failure mode 13 parallel dispatches risks), whether every stated cross-reference actually resolves to a real skill and reads naturally in context, whether any dropped source material (scripts, prompt-template files) left an orphaned reference behind ("see the accompanying script" with no script present).

- [ ] **Step 2: Apply what survives triage**

Per that skill's own output shape — fix what the manager confirms, leave what's declined with its stated reason.

---

### Task 16: Count and version bump

**Files:**
- Modify: `scripts/generate.py` (`EXPECTED_SKILL_COUNT`)
- Modify: `README.md`, `.claude-plugin/plugin.json` (description + version), `scripts/content_pipeline.py` (docstring mentions), `scripts/test_content_pipeline.py`, `scripts/test_generate.py`, and any other file the grep below finds

**Interfaces:**
- Consumes: Task 15's output (final skill count: 74 + 13 = 87)
- Produces: a source tree internally consistent at count 87, ready for Task 17's translation and Task 18's test/ship

- [ ] **Step 1: Bump the constant**

In `scripts/generate.py`, change `EXPECTED_SKILL_COUNT = 74` to `EXPECTED_SKILL_COUNT = 87`.

- [ ] **Step 2: Grep sweep**

```bash
grep -rn "\b74\b" --include="*.py" --include="*.md" --include="*.json" . \
  | grep -v '^\./docs/' | grep -v '^\./_preview/' | grep -v '/\.claude/worktrees/'
```

For each hit found (expect at minimum `README.md`, `CATALOG.md` — already done in Task 14 — `scripts/content_pipeline.py`'s docstrings, `scripts/test_content_pipeline.py`, `scripts/test_generate.py`), update "74" to "87" and, where the surrounding prose spells it out ("Seventy-four"), update the spelled form too ("Eighty-seven"). Also check `README.md`'s "8 more categories, 63 more skills" line — recompute: 87 total minus `formidable` (1) minus whatever the actual "Judgment and meta" category count is after Task 14 — read the category's row count directly from the updated `CATALOG.md` rather than deriving it by arithmetic here, to avoid compounding an off-by-one.

- [ ] **Step 3: Bump `.claude-plugin/plugin.json`**

Read the current `"version"` (may have moved past `0.6.0`/`0.6.1` if the harness-compatibility-layer plan or other concurrent work landed first — see Global Constraints). Bump the **minor** component (e.g. `0.6.1` → `0.7.0`), per this repo's convention of a minor bump for new skills. Update the `"description"` field's "Seventy-four" to "Eighty-seven."

- [ ] **Step 4: Confirm no stray "74" remains outside generated/output paths**

Re-run the Step 2 grep. Expected: zero remaining hits outside `docs/`, `_preview/`, and `.claude/worktrees/`.

---

### Task 17: Translate

**Files:** Modify `i18n/<locale>/descriptions.json` for each of the 12 locales

**Interfaces:**
- Consumes: Task 14's final English descriptions (Step 2's captured text) for all 13 skills
- Produces: 12 updated `descriptions.json` files, ready for Task 18's test/generate

- [ ] **Step 1: Dispatch the translation**

Dispatch one Agent call with `model: "sonnet"` (per `tending-tbaguette`'s explicit rule — pin the model for this substep regardless of what's driving the rest of this execution). Prompt must include: the 13 skills' exact final English `description:` text (re-read from each `skills/<name>/SKILL.md` after Task 15's fixes, not from this plan — Task 15 may have changed them), the list of 12 target locale codes, the instruction to translate the complete sentence (not a summary — `summarize_description()` truncates for card display later, the stored value must be the full text), and the register guidance from `docs/superpowers/plans/2026-08-14-website-i18n.md` Task 11 Step 3 (keep genuine technical terms as-is where a real bilingual technical writer would; don't force an awkward literal translation). The agent writes directly to `i18n/<locale>/descriptions.json` for all 12 locales, keyed by each skill's slug.

- [ ] **Step 2: Verify all 12 locale files gained exactly these 13 keys**

```bash
for locale in ar de es fr hi it ja ko pt ru tr zh; do
  echo "=== $locale ==="
  python3 -c "
import json
d = json.load(open('i18n/$locale/descriptions.json'))
names = ['brainstorming','verification-before-completion','using-git-worktrees',
  'finishing-a-development-branch','test-driven-development','systematic-debugging',
  'requesting-code-review','receiving-code-review','writing-plans','executing-plans',
  'dispatching-parallel-agents','subagent-driven-development','writing-skills']
missing = [n for n in names if n not in d]
print('missing:', missing if missing else 'none')
"
done
```
Expected: `missing: none` for all 12 locales.

---

### Task 18: Test, generate, ship, verify

**Files:** None new — verification and shipping only

**Interfaces:**
- Consumes: Tasks 14-17's complete output
- Produces: the shipped, live, locally-installed release

- [ ] **Step 1: Run the full test suite**

Run: `python3 scripts/run_tests.py`
Expected: fully green, including the `i18n` key-parity check (per its documented caveat, this catches an *unknown* slug, not a *missing* one — Task 17 Step 2 above is the actual completeness check for the 13 new keys).

If anything fails: fix the actual problem — do not proceed to shipping on red. If the failure traces back to a Task 1-13 draft, fix it directly rather than re-dispatching that task.

- [ ] **Step 2: Regenerate the site**

Run: `python3 scripts/generate.py --base-path /tbaguette-skills`

- [ ] **Step 3: Confirm the diff shape**

```bash
git status --short docs/ | grep -v '^ M docs/[a-z][a-z]/skills/' | grep -v '^ M docs/skills/'
git diff --stat docs/ | tail -1
```
Expected: the only unusual (non-per-locale-skill-page, non-per-skill-page) change is `docs/index.html` (new "fresh" entries and/or the updated count), plus 13 new `docs/skills/<name>/` and 13 new `docs/<locale>/skills/<name>/` directories per locale. Anything wider than that — stop and investigate before committing.

- [ ] **Step 4: Commit**

```bash
git add skills/ CATALOG.md README.md .claude-plugin/plugin.json scripts/ i18n/ docs/
git commit -m "$(cat <<'EOF'
Add TBaguette-native versions of 13 core skills, closing the
superpowers hand-off gap

brainstorming, verification-before-completion, using-git-worktrees,
finishing-a-development-branch, test-driven-development,
systematic-debugging, requesting-code-review, receiving-code-review,
writing-plans, executing-plans, dispatching-parallel-agents,
subagent-driven-development, and writing-skills are no longer †
hand-offs to the superpowers plugin -- they're real TBaguette skills,
cross-referenced into the existing 74. 74 -> 87 skills.
EOF
)"
```

- [ ] **Step 5: Fetch, integrate if needed, push**

```bash
git fetch origin master
```
If `origin/master` moved: `git merge origin/master`, resolve any `docs/`-confined conflicts by regenerating (`git checkout --ours -- docs/ && python3 scripts/generate.py --base-path /tbaguette-skills`), rerun `python3 scripts/run_tests.py`, commit the merge, then:
```bash
git push origin master
```

- [ ] **Step 6: Verify the live deploy**

```bash
gh api repos/LeSplooch/tbaguette-skills/pages/builds/latest
```
If the new commit's SHA hasn't triggered a build within ~30s: `gh api repos/LeSplooch/tbaguette-skills/pages/builds -X POST`. Once built, fetch `https://lesplooch.github.io/tbaguette-skills/` fresh (no cache) and confirm the skill count and at least one new skill (e.g. `/tbaguette-skills/skills/systematic-debugging/`) render live.

- [ ] **Step 7: Update the local install**

```bash
git -C ~/.claude/skills/TBaguette pull
```
If that directory doesn't exist on this machine, skip this step.

- [ ] **Step 8: Update memory**

This is a big enough change (13 new skills, closing a documented architectural gap) that future sessions benefit from knowing about it — update or add a project memory noting the superpowers-parity project shipped, what it covered, and the commit(s) it landed in.
