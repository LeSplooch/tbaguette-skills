---
name: atomic-commits
description: Use when a working tree has grown several unrelated changes, when deciding what belongs in one commit, when a diff mixes a rename or a reformat with a logic change, when a bisect lands on a commit too large to reason about, when a revert or a backport drags in changes nobody asked for, when the checkout is shared with another agent, a colleague, or a tool that writes to it, or when a reviewer cannot find the real change in a diff. Covers splitting by file and by hunk, mechanical versus semantic separation, staging in a tree you do not solely own, and commit granularity.
---

# Atomic commits

## Overview

One commit, one decision. A commit is atomic when it builds and passes its tests standing alone, and reverting it removes exactly one choice from the tree — not when it is merely small.

## When to use

- The working tree holds two or three unrelated changes and you are about to stage all of it
- A diff mixes a rename, a reformat, or a regenerated artifact with a logic change
- A bisect landed on a commit too large to reason about
- A revert or a backport drags in changes that were not part of the fix
- The checkout is shared with another agent, a colleague, or a tool that writes to it on its own schedule
- Before publishing a branch whose history is still a draft
- Not for: what the message should say — see `writing-commit-messages`. Not for choosing merge, squash, or rebase at integration time — see `finishing-a-development-branch`.

## What "logical" means

Three tests, ordered by how often they catch something:

1. **The "and" test.** If the subject line needs "and", or wants to be a bulleted list, it is two commits.
2. **The revert test.** Say out loud what breaks if this commit alone is reverted. If the answer names two independent things, split it.
3. **The build test.** Each commit compiles, and its tests pass, with no later commit applied.

The build test overrides the other two. Atomic means *the smallest change that still works*, not the smallest change. A dependency upgrade and the call-site edits it forces are one commit, because neither half builds without the other. Splitting them yields two broken commits, which is strictly worse than one large one: a commit that does not build poisons bisect across the whole range that contains it.

## Splitting a tree that grew three changes

| Situation | Split by | Notes |
|---|---|---|
| Changes sit in disjoint files | file | Cheapest. Stage whole paths, no hunk surgery. |
| One file, feature plus a drive-by fix | hunk | Interactive staging, then verify each half separately. |
| Rename or move plus a behavior change | mechanical first | The mechanical commit may be enormous; see below. |
| Reformat plus an edit | mechanical first | Record the format commit's hash in the blame-ignore list. |
| Generated artifacts plus their source | source first, generated second | Message names the generator and its version. |
| Change plus the refactor that enabled it | refactor first | Reorder history so the enabler precedes the change. |
| One edit whose two callers each needed something | do not split | One change with two consequences is still one change. |

After any hunk-level split, verify each commit in isolation: set the remainder aside, build, test, restore. Splitting by hunk without this check is the standard way a branch acquires a commit that never compiled, discovered months later by someone bisecting an unrelated failure.

A mechanical commit is allowed to be arbitrarily large **when its message names a transformation the reviewer can re-run and diff against it**. "Rename X to Y across the tree, no other edits" is reviewable at 4,000 lines because the reviewer verifies the rule, not the lines. "Cleanup" is not reviewable at 40.

## Staging a tree you do not solely own

The rules above assume every dirty file in the tree is yours. Often it is not: a second agent working the same checkout, a colleague, an editor plugin, a formatter on save, a watcher regenerating artifacts. There, a broad stage is not a shortcut but an unreviewed commit of someone else's in-progress work under your message — their change ships attributed to something it has nothing to do with, and the reasoning behind it exists nowhere in the log. The author of the sweep is rarely the one who notices.

Stage by naming the files you wrote. A path-scoped stage feels like the careful version of staging everything and is not: foreign edits sit in the same directories yours do, so narrowing the sweep still sweeps. Targeting is enumerating — anything whose argument is a directory is a sweep with a smaller radius, and the commit-all shortcut is the same sweep with the review step removed as well.

Re-read the status output immediately before *each* stage, not once when the task begins. Noticing an unfamiliar dirty file at branch time does not protect a commit made an hour later; "pre-existing, riding along untouched" is an observation about the past, not a property of the tree. The cheap audit at the other end is a `--stat` diff of the finished branch against the commit it started from: every path listed should be one you can account for, and the one you cannot is the one that got swept in.

If the sweep already happened and is published, do not rewrite. Land a follow-up commit that names what was pulled in and where it actually belongs. History that is wrong and annotated stays navigable; history that is silently corrected under everyone else's checkouts does not.

## Separating pure refactors from behavior changes

The mixed commit is the one nobody can review and nobody can revert. A reviewer facing 700 moved lines with 6 changed ones cannot find the 6, so they approve the shape and miss the semantics. Split so that one commit changes structure with behavior fixed, and the next changes behavior with structure fixed. Each is then checkable by a different, cheaper method: the refactor by "does anything observable differ", the change by "is this the right behavior".

Land the enabling refactor first even when it is justified only by what follows. The honest workflow is inverted: write the feature to discover what the code should have looked like, then reorder history so the refactor comes first and the feature arrives as a small diff on top. History is a designed artifact, not a transcript.

## Where atomicity pays

- **Bisect** — its resolution is your commit size. A 2,000-line culprit tells you the day, not the cause, and you are back to reading a diff. Every non-building commit forces a skip, and enough skips leave a region permanently unsearchable.
- **Revert** — a mixed commit forces a hand-edited partial revert, composed under incident pressure, untested, by whoever happens to be awake.
- **Cherry-pick** — a fix entangled with a refactor does not backport. You either drag the refactor into a stabilization branch or retype the fix by hand, creating new unreviewed code in the most conservative branch you own.
- **Review** — defect detection drops sharply past roughly 400 changed lines in one sitting; readers switch from reading to pattern-matching. Splitting 900 lines into a 750-line mechanical commit and a 150-line semantic one is what gets the semantic part actually read.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "I will clean up the history before merging" and it never happens | Splitting was deferred past the point where you still remember which hunk belonged to which idea |
| Subject line contains "and" or a bullet list | Two commits wearing one hat |
| Reverting a bad commit breaks something unrelated | Fix and refactor shipped together |
| Bisect lands on a 1,500-line commit | Commit boundaries were drawn by time of day, not by decision |
| Tests only pass on the branch's final commit | Commits are chapters of a story, not units of change |
| A split commit fails CI in isolation | Split by hunk without building each half |
| Reviewer asks why a file is in the diff | A drive-by edit stowed away in an unrelated change |
| A commit contains a file you never opened | The stage was drawn by location, not by authorship |
| Every commit is "wip", "fix", "review feedback" | The branch is a working log; it was never rewritten into a history |

## Red flags

- "It is all one feature really"
- "Splitting it would take longer than writing it"
- "Nobody reverts individual commits anyway"
- "It gets squashed at merge, so the boundaries do not matter" — squashing collapses your careful split back into precisely the mixed commit you avoided
- Staging everything at once without reading the diff first
- "I scoped the stage to the directory I was working in" — so did the change you did not write
- "Those files were already dirty when I started" — true when you checked, and not a claim about now
- Discovering at commit time that the tree holds three changes — that is a planning failure, and the fix is committing before starting the next thing, not better staging
