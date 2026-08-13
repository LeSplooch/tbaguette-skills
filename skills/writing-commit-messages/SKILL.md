---
name: writing-commit-messages
description: Use when writing or rewriting a commit message, when drafting a subject line, when a history reads as "fix", "wip", or "address review comments", when someone must judge from the log whether a change is safe to revert or change, when a message contains nothing but a ticket reference, or when squashing several commits into one message. Covers subject lines, body structure, rationale, trailers, and references.
---

# Writing commit messages

## Overview

The message is written for whoever finds this commit in a blame or a bisect years from now and must decide whether it is safe to touch. The diff already states what changed; only the message can state why, and only the author still knows.

## When to use

- Committing anything a future reader might need to revert, backport, or understand out of context
- Squashing a branch into one commit and composing the message that survives
- Reverting, or fixing an earlier commit
- The history under a file reads as "fix", "wip", "update", "address review comments"
- Not for deciding *what belongs in* the commit — see `atomic-commits`. Not for reconstructing intent from an existing history — see `code-archaeology`.

## The subject line is an index entry

It appears in one-line logs, bisect output, blame, tag ranges, and release notes. It is read at least a hundred times per reading of the body.

- **Under 50 characters**, imperative mood, capitalized, no trailing period. Imperative because the subject completes "applying this commit will ___", and because tool-generated subjects (revert, merge, cherry-pick) already use it — mixed moods make a log unscannable.
- **State the effect, not the mechanism.** "Fix crash on empty cache" beats "Add null check in cache lookup". The mechanism is in the diff; the effect is what a bisector is scanning for.
- **Pass the distinguishability test**: put it beside the other 40 subjects in this file's history. If it could be any of them — "Fix bug", "Update handler", "Refactor" — it is an index entry that indexes nothing.
- Wrap the body at 72 characters. Tools indent it, and terminals still wrap at 80.

## Body: four things, in this order

1. **The problem, as it existed before.** The condition, the observable evidence, how to reproduce. Written for a reader who does not have the ticket, the chat thread, or the incident channel.
2. **The approach taken**, and the invariant it depends on. One or two sentences.
3. **The alternative rejected, and why.** The highest-value line in most messages: it stops a future contributor from "fixing" your code back into the thing you already tried and reverted.
4. **The consequence.** What this makes possible or forecloses, what must now be kept in sync, whether deploy ordering matters.

A body is unnecessary when the subject alone answers *why* and the change is fully self-describing: typo fixes, comment edits, dependency bumps with no behavior change, mechanical renames. If deciding whether it needs a body takes more than a few seconds, it needs one.

Write the message before or during the change, not afterward. Composing it first is the cheapest detector of a non-atomic commit — you notice yourself writing "and". A message reconstructed from the diff afterward can only recover *what*, which the diff already had.

| Change | Body content |
|---|---|
| Typo, comment, formatting | None |
| Dependency bump | One line only if the bump was forced, naming what forced it |
| Bug fix | Reproduction condition, root cause, why this fix and not the obvious one |
| New behavior | Problem, approach, rejected alternative |
| Performance change | Measurement before and after, the workload, the environment measured on |
| Revert | What broke, the reverted hash and subject, whether re-landing is planned |
| Mechanical or generated | The exact transformation or command, so a reviewer can re-run it |
| Security fix landing before disclosure | Describe the change, not the exploitation path; the detail lands after disclosure |

## Trailers and references

Machine-readable key-value lines at the end, one per line, after a blank line: issue references, co-authors, reviewers, sign-off, and the hash a commit reverts or fixes.

- **Reference the tracker, never rely on it.** A ticket link is a bet that the tracker outlives the repository, and it usually loses — through migrations, acquisitions, and permission changes. Inline the two sentences that matter; link for the rest.
- The same applies to review discussion held in a hosting vendor's database. The commit message travels with the code through clones, mirrors, and vendor changes; the pull request description does not.
- **When fixing an earlier commit, cite its hash.** That citation is what lets a reader following blame stop looking.
- **When reverting, say why the revert is correct now**, not merely that something broke. A revert of a revert is the most confusing artifact any history contains, and it is always caused by two messages that each said only "revert X".

## The six-month test

The one check that matters: could a competent stranger, six months from now, decide from this message alone whether reverting the commit is safe?

Answering that requires three things the diff cannot supply — what breaks if this is removed, whether anything since has come to depend on it, and whether the original problem still exists. A message that lists what changed answers none of the three. Read your draft against those three questions before committing; if it fails, the missing sentence is almost always the rejected alternative or the consequence.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Message restates the diff in prose | Author described the patch instead of the decision |
| "Address review comments" | The commit records a conversation turn, not a change |
| Body is a ticket id and nothing else | Context outsourced to a system with a shorter lifetime than the repo |
| "Refactor for clarity" over a 900-line diff | No stated invariant, so a reviewer must verify every line by reading |
| Message written afterward from the diff | Only *what* is still recoverable at that point; *why* has already evaporated |
| Subject truncated in every tool | Over 50 characters, with the informative half at the end |
| Reader cannot tell whether a change is safe to revert | Message documented the implementation, not the constraints |
| Every message is technically accurate and none is useful | Written to satisfy a hook, not a reader |

## Red flags

- "The diff is self-explanatory" — it explains what, never why, and why is the entire point
- "I will explain it in the pull request" — that text lives in a vendor's database, the commit lives in the repository
- "It is obvious why" — obvious to you, today, with the incident still open
- Writing the message as the last step, from the staged diff
- A body that would be identical if the approach had been completely different
