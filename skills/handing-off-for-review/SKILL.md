---
name: handing-off-for-review
description: Use when completing a task or feature and about to hand it off before merging, when stuck and a fresh perspective would help, before a risky refactor to establish a baseline, or after fixing a complex or subtle bug. Covers what context a reviewer — human or subagent — needs up front, what to flag before they have to find it, when a request is premature, and what shape to ask the response back in.
---

# Handing off for review

## Overview

A review is only as useful as the context behind it. Dispatch a reviewer — human or subagent — with precisely crafted context: what changed, what it was supposed to do, and the exact range to look at. Never hand over your working session and expect the reviewer to reconstruct intent from your scrollback; that spends their attention on archaeology instead of the diff. Review early and review often, and make the request itself reviewable — one nobody can act on cold comes back as questions instead of findings.

## When to use

- Mandatory before merging to main, after completing a major feature, and after each task when the work is being driven via `delegating-tasks-with-review-gates`.
- Stuck, and a fresh perspective would help.
- Before a risky refactor, as a baseline to diff against later.
- After fixing a complex or subtle bug.
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
- **Scope, stated explicitly.** A subagent reviewer works read-only — inspect with `git show` / `git diff` / `git log`, never mutate the working tree, index, HEAD, or branch state; a different revision to inspect means a separate worktree (see `isolating-work-with-worktrees`), never a checkout on top of yours. And it reviews the range itself: no spawning a second reviewer for part of the diff. One review seat, spent directly, in passes if the diff demands it.
- **The shape you want back.** Ask for strengths named specifically, issues split by actual severity (critical / important / minor — not everything critical), and a plain verdict: ready to merge or not. An unstructured wall of prose is a request half-made; you'll spend as long triaging it as the reviewer spent writing it.

That's the whole handoff. Session history, dead ends, and the process story stay out of it (see `explaining-technical-work`).

## Flag it before they find it

A reviewer who discovers a known issue on their own stops trusting the parts of the diff they didn't have time to check twice. A reviewer told about it up front spends their attention where you don't already know the answer. Before dispatching, name:

- Deviations from the plan, and why — a reviewer left to guess whether a departure was deliberate either flags it as a bug or waves it through as intentional, and it's a coin flip which.
- Anything in the diff the stated intent doesn't explain — an incidental refactor, a bundled unrelated fix, a file touched for reasons the description doesn't cover.
- What you didn't test, and why — a gap you name is a note; a gap the reviewer finds unannounced reads as one you missed.
- Anywhere you're genuinely unsure — the line you'd bet against if pressed. Reviewers spend their first pass on what looks confident; tell them where the shaky part actually is instead of letting it read as confident by default.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The requester reviews their own diff instead of dispatching anyone | Self-review mistaken for a second pair of eyes; it spends the coordinator's context and catches only what the author already believed was fine |
| The reviewer gets a pasted session transcript | Context conflated with completeness — the transcript has everything except the two things actually needed: intent and range |
| A request goes out as "review my changes," no SHAs attached | The range was assumed obvious instead of stated; the reviewer has to guess where the diff even starts |
| The reviewer surfaces an edge case the requester already knew was untested | The gap was left to be discovered instead of disclosed up front |
| Review skipped because the change is "too small to matter" | Size substituted for readiness as the criterion — small changes fail for free too |
| Findings come back for a diff that's already changed underneath them | Dispatched before the edit reached a stable boundary |

## Red flags

- "It's too simple to need a review" — the excuse that precedes most defects nobody thought to look for.
- Pasting a session transcript in because writing three sentences of context feels slower.
- Dispatching a reviewer before running the checks that would have caught the obvious issues.
- A known rough edge left unmentioned, on the chance the reviewer doesn't find it either.
- A subagent reviewer told to loop in a second opinion of its own for part of the diff.
