---
name: keeping-tbaguette-current
description: Use at the start of every conversation, unconditionally — not gated on how long it's been since the last check — and any time the user asks whether TBaguette is current, what changed recently, or wants to sync to the latest skills. Checks the installed plugin against the published repo, updates it if it can fast-forward cleanly, and reports what changed in a short, readable summary — never a raw commit log.
---

# Keeping TBaguette current

## What this is

The TBaguette plugin, once installed, is a git clone
(`~/.claude/skills/TBaguette`) that only ever moves forward when someone
explicitly re-runs the install command. Nothing tells a user it's fallen
behind. This skill is that check — and, since checking and then leaving the
user to go run a command themselves is half a job, it also applies the
update when it's safe to, and tells them what actually changed rather than
just "updated."

Three things, in order: **check**, **update** (only if safe), **record and
report**.

## Where things live

- The installed plugin: `~/.claude/skills/TBaguette` (a git clone tracking
  `origin/master` of `github.com/LeSplooch/tbaguette-skills`).
- The changelog this skill maintains: `~/.claude/tbaguette-update-log.md` —
  outside the plugin's own directory on purpose, since that directory gets
  overwritten by every pull and is no place to keep state that needs to
  survive one.

If `~/.claude/skills/TBaguette/.git` doesn't exist, there's nothing to do —
stop quietly. This shouldn't normally happen (this skill only runs from
inside that same installed plugin), but don't assume; check.

## 1. Check — already done automatically at session start

TBaguette's own `SessionStart` hook (`hooks/session-start`) now runs this
step for you, every session, deterministically — it doesn't depend on this
skill's description being matched at all. Its result shows up early in the
conversation as a `<TBAGUETTE_UPDATE_CHECK>` block: the fetch already run,
`HEAD` and `origin/master` already compared, the working tree's clean/dirty
state already read.

- **That block present, and this is close to where it appeared** (the first
  response of a fresh session, or right after `/clear`/`/compact`): use its
  data directly and skip straight to step 2 or 3 below. Don't re-fetch — the
  hook already did.
- **It's been a while** (many messages since session start), or **the user
  is explicitly asking right now** whether TBaguette is current: re-run the
  check yourself — the hook only guarantees this happens once per session,
  not that the result stays fresh for the rest of a long conversation. Same
  commands as before, now as the fallback path rather than the only path:

  ```
  git -C ~/.claude/skills/TBaguette fetch origin master --quiet
  ```

  (run with roughly a 15-second timeout). Compare
  `git -C ~/.claude/skills/TBaguette rev-parse HEAD` against
  `git -C ~/.claude/skills/TBaguette rev-parse origin/master`.

Either way — hook-provided or freshly run — update the `Last checked:`
timestamp in the log, including when the fetch itself failed. It's a record
of the most recent attempt for a human skimming the log, not an input to any
decision this skill makes.

- **Same SHA:** up to date. If this was a background/automatic check, don't
  say anything — silence is correct for "nothing to report." If the user
  explicitly asked, say so plainly: "You're on the latest version (v_X.Y.Z_,
  `abc1234`)."
- **Fetch failed (no network, DNS, etc.):** if this was a background check,
  drop it silently — a courtesy check failing is not the user's problem
  right now. If explicitly asked, report the failure honestly rather than
  claiming a false "up to date."
- **Different SHAs:** an update is available — continue to step 2.

## 2. Safety gate before touching anything

```
git -C ~/.claude/skills/TBaguette status --porcelain
```

If this prints anything, the installed clone has local changes — someone
hand-edited a file in there, which this directory was never meant to carry.
**Do not update automatically.** Tell the user plainly: their installed
TBaguette has local changes that would be affected by updating, so the
automatic update was skipped, and point them at
`git -C ~/.claude/skills/TBaguette status` to see what's there. Never
discard local changes on someone's behalf to force an update through.

A clean tree is the common case (this is meant to be a pure installed
dependency, not something anyone edits directly) — this check exists for
the exception, not the rule.

## 3. Update — fast-forward only

```
old_head=$(git -C ~/.claude/skills/TBaguette rev-parse HEAD)

git -C ~/.claude/skills/TBaguette merge --ff-only origin/master

new_head=$(git -C ~/.claude/skills/TBaguette rev-parse HEAD)
```

`merge --ff-only`, not `pull` — step 1 already fetched; there's no reason
to make the network do that work twice. `--ff-only` itself is deliberate:
this clone should always be a straightforward fast-forward of a clean
tracking branch, so if it *isn't* — the published repo's history changed in
a way a plain merge can't reconcile, which is rare and worth a human
looking at — fail loudly rather than reaching for `reset --hard` or a merge
commit to force it through. Leave the install exactly where it was, tell
the user what happened and that manual attention is needed, and stop.

For `old_version`/`new_version` (used in the report below): read
`.claude-plugin/plugin.json`'s `"version"` field directly — with your Read
tool, before and after the merge — rather than shelling out to a language
runtime the install never required (`git clone` is the entire install; it
promises nothing about Python being available). If the field is missing or
the file won't parse, don't block on it: fall back to reporting the short
commit SHAs alone and leave the version number out of the summary.

## 4. Understand what changed — read it, don't just relay it

```
git -C ~/.claude/skills/TBaguette log --oneline "$old_head..$new_head"
git -C ~/.claude/skills/TBaguette diff --name-status "$old_head..$new_head" -- skills/
```

`--name-status`, not `--stat` — it prefixes every changed path with `A`
(added), `M` (modified), or `D` (deleted), which is what actually tells a
new skill apart from a modified one; `--stat` just shows line-count bars
with no way to tell "new file" from "big edit" apart. A skill directory
whose `SKILL.md` shows `A` is new; one showing `M` was changed. Read the
actual commit messages too, not just subject lines — this repo's own
commits explain *why*, and a good summary reflects that, not a re-typed
`git log`.

## 5. Report — a summary, not a log dump

Three to six lines, structured, calibrated to what a *user of the skills*
actually cares about: new and meaningfully-changed skills first, since
that's what they're here for; a site or tooling change only if it's
substantial enough to be worth a sentence (skip a typo fix or a CSS nudge
unless that's literally the entire update). Illustrative shape — invented
skill names and changes, to show the structure, not a real changelog entry
to copy:

> ## TBaguette updated: v0.2.0 → v0.3.0
>
> **New**
> - `some-new-skill` — one clause on what it's for
>
> **Updated**
> - `some-existing-skill` — one clause on what changed and why it matters
>
> Also in this update: <a substantial site/tooling change, only if one shipped>.

If this was a background check that found and applied an update, still
report it — an update that happened silently in the background is exactly
the kind of thing worth one paragraph at a natural point in the
conversation, just not an interruption of whatever the user actually asked
for.

## 6. Record it

Prepend a dated entry (newest first) to `~/.claude/tbaguette-update-log.md`
under `## Updates`, and refresh the `Last checked:` line. Create the file
with its header if it doesn't exist yet:

```markdown
# TBaguette update log

Maintained by the `keeping-tbaguette-current` skill. Records every time the
installed plugin (`~/.claude/skills/TBaguette`) was checked or updated
against the published repo.

Last checked: 2026-08-14T10:03:12Z

## Updates

### 2026-08-14 — v0.2.0 → v0.3.0 (`d168319` → `44bee4f`, 3 commits)

- **New:** `keeping-tbaguette-current` — checks for and applies TBaguette updates, with a readable changelog
- Also: the site was renamed to "TBaguette's Atelier."
```

This is what turns "did I update recently, and to what" from a question
only `git log` inside a specific clone can answer into something readable
on its own, going back as far as this skill has been running.

## What this skill deliberately does not do

- **Never rewrites history, never force-pushes** — it only ever fetches and
  fast-forward merges.
- **Never touches anything outside `~/.claude/skills/TBaguette`** and its
  own log file.
- **The plugin's `SessionStart` hook only ever reads.** It runs step 1
  (fetch, compare, `status --porcelain`) and nothing past it — no merge, no
  changelog write, no settings change. Steps 2–6 stay exactly as documented
  above, run by Claude, not by the hook script. This was previously a "we
  deliberately don't install a hook for this" line; it changed on purpose —
  see `superpowers/specs/2026-08-15-using-tbaguette-hook-design.md` for why
  a plugin-shipped, read-only check was judged worth reversing that stance,
  and `hooks/session-start` for exactly what it runs.
- **Never discards local changes** to force an update through — see the
  safety gate above.
