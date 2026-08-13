---
name: keeping-tbaguette-current
description: Use at the start of a conversation if it has been 24 hours or more since the last check (read the timestamp in ~/.claude/tbaguette-update-log.md first — skip silently and instantly if it isn't due yet, never shell out to git just to find out), or any time the user asks whether TBaguette is current, what changed recently, or wants to sync to the latest skills. Checks the installed plugin against the published repo, updates it if it can fast-forward cleanly, and reports what changed in a short, readable summary — never a raw commit log.
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

## 1. Check — rate-limited, so this never becomes noise

Read `~/.claude/tbaguette-update-log.md`'s `Last checked:` line first (see
format below — it's always ISO-8601 UTC, e.g. `2026-08-14T10:03:12Z`). If
the file doesn't exist yet, there's no prior check — treat that as due now,
same as if 24 hours had already passed. Otherwise: if less than 24 *elapsed*
hours have passed (not "the calendar date changed" — a check at 11:58pm and
another four minutes later are the same rate-limit window even though the
date rolled over) and the user didn't explicitly ask for a check, stop here
— don't run `git fetch` at all. This is what keeps "automatic" from meaning
"a network call on every single message."

If it's due (or explicitly requested), fetch with a short timeout — this is
meant to be a quiet background courtesy check, and a stalled connection
should never hang the whole turn waiting on it:

```
git -C ~/.claude/skills/TBaguette fetch origin master --quiet
```

(run with roughly a 15-second timeout).

Compare `git -C ~/.claude/skills/TBaguette rev-parse HEAD` against
`git -C ~/.claude/skills/TBaguette rev-parse origin/master`. Update the
`Last checked:` timestamp in the log either way, including when the fetch
itself failed — an attempt happened, and this is what keeps a multi-day
network outage from turning into a check racing to retry on every message
once it's back; it'll pick back up cleanly at the next 24-hour mark.

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
- **Never modifies Claude Code settings or installs a hook.** Reliable
  "every session" triggering the way this skill's own description asks for
  depends on normal skill matching. A user who wants a harder guarantee can
  wire up their own `SessionStart` hook that nudges toward this skill —
  that's a deliberate, visible choice for them to make about their own
  global settings, not something installing a plugin should do on their
  behalf.
- **Never discards local changes** to force an update through — see the
  safety gate above.
