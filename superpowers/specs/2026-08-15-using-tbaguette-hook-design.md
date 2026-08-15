# using-tbaguette + SessionStart hook — design

## Purpose

Every TBaguette skill today depends on Claude's own judgment matching a
`description:` string. That's the same mechanism this repo has always relied
on, and it means "use TBaguette's skills" is a preference Claude can forget,
deprioritize, or rationalize past — never a guarantee. This adds TBaguette's
first real guarantee: a `SessionStart` hook that force-injects a standing
instruction to check `TBaguette:*` skills before every response, for the rest
of the conversation, in every project — not just when the plugin's own
skill-matching happens to fire.

The mechanism is the one already proven in this exact ecosystem: the
`superpowers` plugin's `using-superpowers` skill, injected via its own
`SessionStart` hook. That hook is what puts `<EXTREMELY_IMPORTANT>You have
superpowers.</EXTREMELY_IMPORTANT>` in front of every session where
Superpowers is installed. This design gives TBaguette the equivalent.

Folded into the same hook, per explicit decision: `keeping-tbaguette-current`'s
step 1 (fetch + compare against the published repo) now runs deterministically
every session instead of depending on the same probabilistic skill-matching
its own description asks for but can't force. This reverses a documented
decision in that skill's own file (see "Rewriting keeping-tbaguette-current"
below) — a deliberate, explicit trade-off, not an oversight.

## Mechanism: SessionStart hook

Three new files under `hooks/`, following the pattern Superpowers already
proves works in this ecosystem, trimmed to what TBaguette actually needs
(single target platform — Claude Code — so no Cursor/Copilot-CLI output
branching):

- **`hooks/hooks.json`** — declares the hook. Not referenced from
  `plugin.json`; Claude Code auto-discovers `hooks/hooks.json` by file
  convention alone (confirmed: Superpowers' own `plugin.json` has no
  `"hooks"` field at all).

  ```json
  {
    "hooks": {
      "SessionStart": [
        {
          "matcher": "startup|clear|compact",
          "hooks": [
            {
              "type": "command",
              "command": "\"${CLAUDE_PLUGIN_ROOT}/hooks/run-hook.cmd\" session-start",
              "shell": "bash",
              "async": false
            }
          ]
        }
      ]
    }
  }
  ```

  Matcher covers fresh start, `/clear`, and `/compact` — matches
  Superpowers', and matters here specifically because `using-tbaguette`'s
  instruction is meant to survive a compaction, not just the first message.

- **`hooks/run-hook.cmd`** — polyglot batch/bash dispatcher. On native
  Windows (this repo's README explicitly documents a native-PowerShell
  install path, so this needs to actually work there, not just under WSL),
  `cmd.exe` runs the batch half, which locates a `bash` (Git for Windows in
  its standard install locations, or `bash` already on `PATH`) and hands off
  to it. On macOS/Linux/WSL, the batch half is inert (`:` is bash's no-op)
  and execution falls through to the Unix section. If no bash can be found
  on Windows, exit `0` silently — the plugin still works, just without
  session-start context injection on that machine, same fallback behavior
  Superpowers uses.

- **`hooks/session-start`** — the actual bash script (extensionless
  filename, deliberately — Claude Code's Windows auto-detection prepends
  `bash` to any command containing `.sh`, which would double-invoke it).
  Builds one JSON payload, printed to stdout as
  `hookSpecificOutput.additionalContext`. No multi-platform branching, since
  TBaguette only targets Claude Code.

## `using-tbaguette` skill

New `skills/using-tbaguette/SKILL.md`. Content mirrors `using-superpowers`'s
posture (a hard rule, a red-flags table, an explicit non-conflict note for
when another plugin's own "check skills" notice is active in the same
session — true today, since Superpowers is installed alongside TBaguette in
this very environment) without copying it — TBaguette's skills are mostly
narrow and non-competing (unlike Superpowers' broad brainstorming/TDD
process skills), so no "skill priority" ordering section is needed.

Categorized under `judgment-and-meta` in `content_pipeline.py`'s
`CATEGORIES`, first position — it's the same kind of foundational,
process-level skill as `karen-and-the-manager`, and governs how every other
skill in the library gets reached.

The file's content is read and injected verbatim by `hooks/session-start`,
the same way `using-superpowers/SKILL.md` is read and injected verbatim by
Superpowers' own hook — so this file is simultaneously a normal, independently
invocable skill (`TBaguette:using-tbaguette`) and the hook's payload.

## Folding in the update check

`hooks/session-start` also performs `keeping-tbaguette-current`'s **step 1
only** (check — fetch, compare, working-tree status) deterministically in
bash, and injects the raw result as a second context block. Steps 2–6
(safety gate, fast-forward merge, understanding the diff, writing the
human-readable report, logging) stay entirely Claude's job, invoked from the
existing skill, using the injected data instead of re-fetching. Mechanical,
side-effect-free work (network I/O, string comparison) belongs in the hook;
judgment (deciding how to phrase a report, handling a failed fast-forward)
stays with Claude.

Two refinements over the literal current wording of `keeping-tbaguette-current`:

- **Path**: the hook checks `${CLAUDE_PLUGIN_ROOT}/.git`, not a hardcoded
  `~/.claude/skills/TBaguette`. This is self-referential — it checks
  wherever *this running instance* actually lives — and closes a real
  footgun already on record: git's upward directory search can silently
  attribute a non-repo `TBaguette` folder to a *different*, unrelated
  ancestor repo (e.g. `~/.claude` itself, if that happens to be a git
  checkout) rather than failing. Checking `.git` as a literal path first,
  the same guard `keeping-tbaguette-current` already specifies in prose,
  avoids ever invoking `git -C` against the wrong repository.
- **Timeout**: `git -c http.lowSpeedLimit=1000 -c http.lowSpeedTime=15
  fetch` instead of wrapping the call in the external `timeout` command —
  portable (pure git config, no GNU-coreutils dependency this repo's
  explicitly-supported macOS/Windows targets can't guarantee), and matches
  the "roughly a 15-second timeout" intent already in the skill's prose.

Injected block shape (exact field values vary; this shows the structure):

```
The TBaguette:keeping-tbaguette-current skill's step 1 (check) already ran
automatically for this session -- do not re-fetch.

- Installed plugin path: <CLAUDE_PLUGIN_ROOT>
- HEAD: <sha>
- origin/master: <sha>
- Comparison: same (up to date) | different (update available) | unknown (rev-parse failed)
- Working tree: clean | dirty (local changes present)

If the comparison is "different" and the tree is clean, follow the
TBaguette:keeping-tbaguette-current skill's remaining steps using this data.
If the tree is dirty, follow that skill's local-changes handling instead of
updating. If the comparison is "same," this was a background check -- stay
silent unless asked.
```

Two degraded cases get their own short block instead: no `.git` present at
`${CLAUDE_PLUGIN_ROOT}` ("not a git clone — nothing to do", matching the
skill's own existing rule), and fetch failure (offline/DNS/timeout — "stay
silent unless asked", matching the skill's own existing rule for background
checks).

### Rewriting keeping-tbaguette-current

The skill currently documents, under "What this skill deliberately does not
do":

> **Never modifies Claude Code settings or installs a hook.** Reliable
> "every session" triggering... depends on normal skill matching... not
> something installing a plugin should do on their behalf.

This is now false and needs rewriting, not just quietly contradicted. New
wording states plainly that the plugin's own `SessionStart` hook
(`hooks/session-start`) now runs step 1 automatically every session,
describes exactly what it does and does not do (read-only: fetch, compare,
status — never merges, never touches the changelog), and that steps 2–6
remain this skill's job as before, now using pre-fetched data when it's
present in context rather than re-running step 1. The "record it" and
"report" sections are otherwise unchanged.

## Bookkeeping (74th skill)

- `README.md` — "73 skills" → "74 skills" in the opening line. (The "8 more
  categories, 63 more skills" sentence is unaffected — `using-tbaguette`
  lands inside the already-named `judgment-and-meta` category, which that
  sentence doesn't count by number.)
- `CATALOG.md` — "73 skills" → "74 skills"; new row in the "Judgment and
  meta" table, first position.
- `.claude-plugin/plugin.json` — description's "Seventy-three" →
  "Seventy-four"; version `0.5.0` → `0.6.0` (matches this repo's existing
  convention of a minor bump per skill added, confirmed against the commit
  that added `keeping-tbaguette-current`: `0.2.0` → `0.3.0`).
- `scripts/content_pipeline.py` — add `"using-tbaguette"` to the
  `judgment-and-meta` category's `skill_slugs` (required — `build_content`
  raises `ValueError` on an uncategorized skill directory, which is exactly
  the guardrail that makes this a required edit, not an optional one); bump
  the three prose "73" references in docstrings/comments.
- `scripts/generate.py` — `EXPECTED_SKILL_COUNT = 73` → `74` (a deliberate
  guardrail against schema drift; the module's own comment says exactly what
  to do here).
- `scripts/test_content_pipeline.py` — the two hardcoded `73` assertions →
  `74`; the test method is already misnamed from a prior count change
  (`test_finds_exactly_66_skills` asserting `73`) — corrected to
  `test_finds_exactly_74_skills` while touching this line anyway; docstring
  "73-skill corpus" mention updated.
- `scripts/test_generate.py` — docstring "73-skill corpus" mention updated.

## Testing

New `scripts/test_hooks.py`, wired into `run_tests.py`'s `SUITES` list
(required — per that script's own docstring, anything not listed there
silently never runs):

- `hooks/hooks.json` parses as JSON and has the expected `SessionStart` /
  matcher / command shape.
- `hooks/session-start` is executable and, run directly with
  `CLAUDE_PLUGIN_ROOT` pointed at this repo's own root, emits valid JSON
  containing `hookSpecificOutput.hookEventName == "SessionStart"` and an
  `additionalContext` string that contains the verbatim content of
  `skills/using-tbaguette/SKILL.md`.
- Update-check block, exercised against throwaway local git repos (a fake
  "origin" and a fake "clone" of it in a temp directory — file-path remote,
  no network, no GitHub dependency, faster and less flaky than
  `test_install_command.py`'s real-clone approach):
  - clone HEAD == origin/master → "same (up to date)" reported.
  - origin/master has a commit the clone doesn't → "different (update
    available)" reported, with correct SHAs.
  - clone has an uncommitted local change → "dirty" reported regardless of
    SHA comparison.
  - no `.git` under the checked path → "not a git clone" block, no git
    commands attempted.

## Out of scope

- Making the hook actually reach this user's own local install: the memory
  record on file is explicit that `~/.claude/skills/TBaguette` isn't
  currently a git clone there, and TBaguette isn't in `settings.json`'s
  `enabledPlugins` (it loads as a `skills-dir` plugin instead) — so this
  hook won't fire in that specific environment until both are addressed.
  That's a follow-up for the user's own machine, not a repo change.
- Any change to `keeping-tbaguette-current`'s steps 2–6 (safety gate,
  fast-forward merge, report format, changelog format) — those are
  unchanged; only step 1's triggering mechanism changes.
- Cursor/Copilot-CLI/other-agent output branching in `hooks/session-start` —
  TBaguette targets Claude Code only, per this repo's own README.
