---
name: handing-off-for-review
description: Use when completing a task or feature and about to hand it off before merging, when stuck and a fresh perspective would help, before a risky refactor to establish a baseline, after fixing a complex or subtle bug, or before sending a change to a project whose maintainers you do not work with. Covers what context a reviewer — human or subagent — needs up front, what to flag before they have to find it, when a request is premature, and what shape to ask the response back in. Also use when a change merged from parallel workers is too large for one reviewer and will be read by several, and their findings have to be ranked.
---

# Handing off for review

## Overview

A review is only as useful as the context behind it. Dispatch a reviewer — human or subagent — with precisely crafted context: what changed, what it was supposed to do, and the exact range to look at. Never hand over your working session and expect the reviewer to reconstruct intent from your scrollback; that spends their attention on archaeology instead of the diff. Review early and review often, and make the request itself reviewable — one nobody can act on cold comes back as questions instead of findings.

## When to use

- Mandatory before merging to main, after completing a major feature, and after each task when the work is being driven via `delegating-tasks-with-review-gates`.
- Stuck, and a fresh perspective would help.
- Before a risky refactor, as a baseline to diff against later.
- After fixing a complex or subtle bug.
- A change too large for one reader that cannot be split, such as the hand-merged result of parallel workers: several reviewers, one area each (below).
- Not yet: your own checks haven't passed — tests, lint, a manual run (see `confirming-before-claiming-done`). A reviewer finding what the test suite would have found is attention spent on something that was still yours to catch.
- Not yet: mid-edit, on a diff that isn't yet one coherent unit of work — wait for the boundary; don't hand over a moving target.
- Not for: how to actually perform a review — that's the reviewer's job (see `reviewing-code-deeply`). Once the review comes back, acting on it — fixing, pushing back, deciding what's worth deferring — is `verifying-review-feedback`, the other half of this pair; this skill ends at the request.

## What a reviewer needs up front

The reviewer — subagent or human — starts cold. Everything it knows about the change comes from what you hand it:

- **What was built.** A brief, concrete description of the outcome — not the plan restated.
- **What it should do.** The plan, requirements, or task text to check the diff against. State intent poorly and the review can only check internal consistency, which passes cleanly on a change that solves the wrong problem.
- **The exact range**, base and head SHA, not "my recent changes":
  ```bash
  BASE_SHA=$(git rev-parse HEAD~1)  # or origin/main
  HEAD_SHA=$(git rev-parse HEAD)
  ```
  Hand over the stat alongside the full diff so the reviewer can gauge size before committing to depth.
- **Scope, stated explicitly.** A subagent reviewer works read-only — inspect with `git show` / `git diff` / `git log`, never mutate the working tree, index, HEAD, or branch state; a different revision to inspect means a separate worktree (see `isolating-work-with-worktrees`), never a checkout on top of yours. And it reviews the range itself: no spawning a second reviewer for part of the diff. One review seat, spent directly, in passes if the diff demands it. More seats, when the change needs them, are the coordinator's to allocate (below), never the reviewer's.
- **The shape you want back.** Ask for strengths named specifically, issues split by actual severity (critical / important / minor — not everything critical), and a plain verdict: ready to merge or not. An unstructured wall of prose is a request half-made; you'll spend as long triaging it as the reviewer spent writing it.

That's the whole handoff. Session history, dead ends, and the process story stay out of it (see `explaining-technical-work`).

## Flag it before they find it

A reviewer who discovers a known issue on their own stops trusting the parts of the diff they didn't have time to check twice. A reviewer told about it up front spends their attention where you don't already know the answer. Before dispatching, name:

- Deviations from the plan, and why — a reviewer left to guess whether a departure was deliberate either flags it as a bug or waves it through as intentional, and it's a coin flip which.
- Anything in the diff the stated intent doesn't explain — an incidental refactor, a bundled unrelated fix, a file touched for reasons the description doesn't cover.
- What you didn't test, and why — a gap you name is a note; a gap the reviewer finds unannounced reads as one you missed.
- Anywhere you're genuinely unsure — the line you'd bet against if pressed. Reviewers spend their first pass on what looks confident; tell them where the shaky part actually is instead of letting it read as confident by default.

## A change too large for one reader is read by several, each on one area

`reviewing-code-deeply` answers a diff past the size where detection collapses by asking for a split. Sometimes there is nothing to split: several workers wrote slices in parallel and the result was merged by hand, so the combination is the thing nobody has read, and it is one change. Several readers cost several reviews, so this is for a change whose missed defect is expensive. Then the review is itself a fan-out, and four choices decide whether it finds more than one reader would.

- **Pin what is read.** Check the commit under review out into its own detached worktree that reviewers only read, give every reviewer that path and the SHA, and remove it once the reports are in. Authors keep fixing in the live tree while reviews run, and a reviewer reading it reports line numbers and behavior that are gone by the time the report is read. Waiting for a stable boundary is better when the authors can pause; when they cannot, pin.
- **One area per reviewer, named in its brief.** Choose areas as risk lenses over the whole change, by where it can fail and where the tests cannot reach, such as the state machine, the persistence path or the input handling, not by directory. A reviewer told to review everything spreads one reader's attention evenly and spends it where everyone expects the code to be fine. Areas overlap at the edges; leave the overlap, because it is where agreement comes from. Each reviewer is still a single seat that reads its area directly.
- **Say that the authors' reports and green tests are not evidence.** A summary is a claim, and a suite shares its authors' blind spots. In one run the only high-severity defect was a handler that consumed an event before testing whether it was meant for it, and thousands of passing tests missed it because nothing in the suite could perform the gesture. The disclosures from the section above still go in, as leads to check rather than verdicts to accept. Name what the suite cannot exercise, such as gestures, ordering and failure paths. In place of the strengths list and the merge verdict above, which mean little for one area, ask for defects only, each with a failure scenario (the input or sequence, what happens, what should have happened), plus a line on what the reviewer read and what it did not. Every claim is then checkable and none arrives as taste.
- **Keep the readers independent until every report is in.** Cold-start reviewers, and no reviewer sees another's findings. A fork of the authoring session suits one reader who needs what the session knows; several forks are copies of one set of assumptions (`fanning-out-independent-work`), and agreement between copies is not agreement.

Then reconcile. Verify each reported defect in the code before touching anything, as `verifying-review-feedback` says, because a reviewer's confidence is not a measurement. Take the fixes in order of agreement: two readers who never saw each other's report landing on the same defect, in different words, is stronger evidence than either one's severity label, so it is the surest fix to take first. Agreement ranks; it does not gate. The areas were split on purpose, so the most serious defect may have exactly one witness, and a lone report is neither true nor false for being alone. Verification decides.

## Someone else's project decides what an agent may send it

Everything above assumes a reviewer who agreed to review. A pull request or patch to a project you do not belong to lands in front of maintainers who agreed to nothing, and many of them have now written down what they will accept from an agent: a study of a few hundred open-source projects' published policies in 2026 found about two thirds requiring substantial human involvement in a contribution and about half requiring AI assistance to be disclosed, with some refusing it outright. Before opening anything upstream, read the contributing guide, the pull request template, and any policy on generated contributions, and follow what they say — disclose the assistance where they ask, name the human who has read and stands behind the change, and keep to the size they will review. A change sent against a stated policy costs a maintainer the same read as any other and gets closed on sight; the next contribution from the same account inherits that. And opening it is a message to a person outside the run: an unattended run prepares the change and the disclosure, then stops for a human (`bounding-autonomous-work`).

## Common mistakes

| Symptom | Real cause |
|---|---|
| The requester reviews their own diff instead of dispatching anyone | Self-review mistaken for a second pair of eyes; it spends the coordinator's context and catches only what the author already believed was fine |
| The reviewer gets a pasted session transcript | Context conflated with completeness — the transcript has everything except the two things actually needed: intent and range |
| A request goes out as "review my changes," no SHAs attached | The range was assumed obvious instead of stated; the reviewer has to guess where the diff even starts |
| The reviewer surfaces an edge case the requester already knew was untested | The gap was left to be discovered instead of disclosed up front |
| Review skipped because the change is "too small to matter" | Size substituted for readiness as the criterion — small changes fail for free too |
| Findings come back for a diff that's already changed underneath them | Dispatched before the edit reached a stable boundary |
| Four reviewers returned four overlapping surveys, and the serious defect was in none of them | Each was asked to review everything: no area, no failure scenario required |
| Findings cite lines that no longer match the code | Reviewers read the live tree while the authors kept fixing; the commit under review was never pinned |
| An upstream pull request closed without a review | The project's contributing guide or its policy on generated contributions was never read, and the change broke it |

## Red flags

- "It's too simple to need a review" — the excuse that precedes most defects nobody thought to look for.
- Pasting a session transcript in because writing three sentences of context feels slower.
- Dispatching a reviewer before running the checks that would have caught the obvious issues.
- A known rough edge left unmentioned, on the chance the reviewer doesn't find it either.
- A subagent reviewer told to loop in a second opinion of its own for part of the diff.
- The same one-line brief ("review the change") sent to several reviewers.
- Ranking fixes by the loudest single severity label when two independent readers named the same defect.
