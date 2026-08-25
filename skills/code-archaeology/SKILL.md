---
name: code-archaeology
description: Use when code exists without an explanation and the reason matters — an odd workaround, a magic constant, a conditional nobody can justify, a comment that contradicts the code, an unexplained TODO, or a line that looks safe to delete. Also when locating the commit that introduced a behavior or regression, following a file through renames and moves, judging whether old code is still load-bearing, or when a recorded decision appears to rule out the work in front of you.
---

# Code archaeology

## Overview

Code records what was decided; history records why, and only history knows what was tried and rejected. The reason a line exists is almost never in the file that contains it, and the cost of deleting a line whose reason you did not find is paid in production.

## When to use

- Before deleting, simplifying, or "cleaning up" anything you cannot explain.
- A constant, sleep, retry count, or special case has no justification in the code.
- A regression appeared between two known-good points.
- A comment and the code it sits above disagree.
- Not for: how a value moves at runtime (tracing-data-flow), or first contact with a whole repo (orienting-in-unfamiliar-code).

## Question to command

| Question | Command | Note |
|---|---|---|
| Who last changed these lines | `git blame -w -C -C -M -L <a>,<b> <file>` | Never run bare `blame`; the flags are the difference between an answer and noise |
| Who changed them before that | `git blame <commit>^ -L <a>,<b> -- <file>` | Walk parents one at a time to step back through reformats |
| When did this string or symbol first appear or vanish | `git log -S'<string>' --oneline -- <path>` | Pickaxe: matches commits where the occurrence count changed |
| Every commit whose diff mentions a pattern | `git log -G'<regex>' -p -- <path>` | Broader than `-S`; use when the token moved rather than appeared |
| Every change to one function | `git log -L :<function>:<file>` | Survives the function moving within the file |
| Which commit changed the behavior | `git bisect start <bad> <good>` then `git bisect run <script>` | The only technique here that scales past a few dozen commits |
| Full history across renames | `git log --follow -- <file>` | `--follow` handles one path; `-C -C` in blame handles content copied between files |
| The change's real scope | `git show --stat <commit>` | The sibling files are the scope |
| Was this tried and reverted | `git log --grep='<term>' --grep='[Rr]evert' -i --oneline` | The highest-value search in this table |

## Blame's cardinal failure

Blame answers "who last wrote these bytes", never "who decided this". A reformat, a lint autofix, a license header, an import reorder, or a mass rename resets blame for an entire file. When blame returns a commit titled `reformat`, `style`, `lint`, `bump`, or a tool name, you have learned nothing — blame its parent and repeat. Run with `-w -C -C -M` from the start so most of that noise never appears.

## Bisect the test, not the impression

`bisecting-failures` owns the search itself — the exit-code contract, the axes other than commits, and what to do when the history will not build. What matters here is that bisecting is usually cheaper than the reading this skill otherwise describes, and the threshold is low: past about a page of log, stop reading and start halving.

Write the smallest command that exits nonzero on the bad behavior *before* starting the bisect. Then `git bisect run` turns a thousand commits into roughly ten builds, unattended. Doing it by hand is why people convince themselves bisecting is not worth it.

Rules that make it work: the script must be robust to a tree that does not build (exit 125, or `git bisect skip`); it must not depend on state left by the previous iteration; and it must test the behavior, not a test-suite result that changed for unrelated reasons in the range.

## Read the siblings

A commit's other files tell you its true scope. A one-line parser fix that also touched a config default and a fixture was not a one-line fix — it was a contract change with a small diff. A commit that touched only the file you are reading tells you the author believed the change was local, and whether they were right is now checkable.

Then read outward in this order, because each layer holds intent the previous one dropped:

1. **The commit message** — what changed.
2. **The merge or pull request** — why now, and what else shipped with it.
3. **The linked issue** — the symptom that started it, in the reporter's words.
4. **The review discussion** — what was considered and rejected. This is the most valuable layer and the only place that information exists anywhere.

**Reverts and re-lands are the highest-signal artifacts in any history.** A revert says the change was wrong in production; the re-land shows exactly what was missing the first time. Before making a change, search the history for a prior attempt at the same change. Removing a workaround that was already removed and restored once is a distinct and very common failure.

## Date the code

Run `git log -1 --format=%ad -L <a>,<b>:<file>` on the region. Age is the cheapest input to "can this go":

| Age of the line | Reading |
|---|---|
| Under 3 months | Probably still load-bearing; ask the author, they remember |
| 1–3 years | Check whether the condition it guards still exists |
| Older than a major dependency, platform, or protocol version in use | Likely vestigial — but confirm the dependency actually moved, since workarounds outlive the bugs they name |

## The reason has its own expiry date

Finding the rationale is not the end of the dig. A record fuses a stable **principle** with a **premise** about the world, and premises held by a vendor, platform, dependency, or protocol expire without any commit touching this repo — so nothing in this skill will show you one moved. Presence of a recorded reason is not evidence it still holds.

`revalidating-decisions` covers the split and the re-check.

## When history is worthless

| History shape | Signal | Fall back to |
|---|---|---|
| One "initial commit" with thousands of files | Imported from another repo | The old repo; failing that, released versions in a package registry, release notes, the issue tracker |
| Squash-merge only | Intent lives in the forge, not the repo | The pull request number in the commit trailer, then its discussion |
| Vendored or third-party tree | History belongs upstream | Diff the vendored copy against the matching upstream tag; the local patches *are* the intent |
| A rewrite (`v2`, `next`, a parallel package) | Pre-rewrite history describes a system that no longer exists | Tests carried across from the old version — those are the surviving requirements |
| Generated files | The churn belongs to the generator | The schema or template's history |

The universal fallback when history is silent: the test that shipped with the code. A commit adding a strange condition alongside a test named for an empty batch from a legacy producer has already told you the reason. When even that is missing, the next fallback is the log, metric, or alert the code emits — someone added it because something happened.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Blame points at a formatting commit | Ran bare `blame`; needed `-w -C -C -M`, or the parent |
| "This file has no history" | Renamed or moved; needed `--follow` or `-C -C` |
| Read thirty commits hunting a behavior change | Should have written a one-line test and bisected |
| Every message says "fix" or "wip" | Intent is in the pull request and issue, not the repo |
| Deleted a workaround; it broke again | Never searched for a prior revert of the same deletion |
| Found the commit, still no reason | Stopped at the message; the rationale was in the review thread |
| Concluded "no reason, it's safe" | Absence of a recorded reason is not evidence of absence |
| Found a documented reason and closed the question | Presence of a recorded reason is not evidence it still holds |

## Red flags

- "The comment explains it, that's enough" — the comment is contemporaneous with the code, not with the decision.
- "This is obviously dead code."
- "Nobody remembers, so there was no reason."
- "I'll read the log until I spot it" — past a page, bisect.
- "It's just a formatting change" — about a commit you are attributing a decision to.
- "It's in the design doc, that was already decided" — decided on a premise, which is a different claim from still true.
- Reading a document's present tense as evidence about the present.
