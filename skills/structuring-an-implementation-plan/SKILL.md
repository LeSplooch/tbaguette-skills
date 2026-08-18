---
name: structuring-an-implementation-plan
description: Use when a spec or set of requirements is settled and the next step is turning it into a multi-step implementation plan, before any code changes start. Covers deciding file structure ahead of the tasks, right-sizing tasks and steps, the plan document's required header and per-task structure, banning placeholder content, and self-reviewing a finished plan against its spec.
---

# Structuring an implementation plan

## Overview

A plan is written for the reader with the least context anyone touching this work will ever have — less than whoever wrote the spec, less than the reviewer, less than you an hour from now. Assume that reader is skilled but has questionable taste on anything left for them to decide: every fact or choice the plan omits becomes a decision they invent instead, and an invented decision is exactly what a fresh reviewer's gate exists to catch — one task too late. The fix isn't more prose; it's smaller tasks, exact paths and signatures, and no sentence that describes what to do without showing how.

## When to use

- A spec or requirements document is settled and the next step is turning it into tasks an implementer can pick up cold.
- The work spans more than one file, commit, or sitting — anything a single diff wouldn't cover.
- About to hand the work to a fresh session, a subagent, or another engineer who wasn't part of arriving at the spec.
- A previous plan's tasks kept triggering mid-implementation questions — the granularity or the detail was wrong, not the reader.
- Not for: whether a plan's effort estimate is realistic (see `estimating-effort`).
- Not for: recording why a decision was made a particular way (see `writing-adrs`).
- Not for: executing an already-written plan task by task (see `working-a-plan-task-by-task`).

## Scope, then file structure

A spec covering more than one independent subsystem should already have been split during `scoping-before-building` — that decomposition is upstream of this skill, not a decision made here. If a spec still describes several unrelated subsystems as one plan by the time it gets here, that step got skipped: stop and propose separate plans, one per subsystem, each producing working, testable software on its own — not tasks that only look finished together.

With scope settled, decide file structure — which files get created or modified, and what each one owns — before writing a single task. This is where the real decomposition happens, not a summary written after the fact:

- Give each file one clear responsibility, with a boundary you could describe in a sentence.
- Favor smaller, focused files. You reason best about code you can hold in context at once, and a fresh implementer does too — their reading comprehension is the resource being spent.
- Files that change together belong together. Split by responsibility, not by technical layer.
- Follow the codebase's existing patterns. If it already runs large files, don't restructure unilaterally — but if a file the plan touches has grown unwieldy, splitting it is a legitimate task, not scope creep.

A task that doesn't trace back to a file-structure decision is usually a sign the structure wasn't actually decided yet — it was discovered task by task instead.

## Right-sizing tasks and steps

A task is the smallest unit that carries its own test cycle and is worth a fresh reviewer's gate. Fold setup, configuration, scaffolding, and docs into the task whose deliverable actually needs them — don't create a task with nothing to test on its own. Split only at a boundary where a reviewer could plausibly approve one task and reject its neighbor; if approving one implies approving the other, they're one task wearing two numbers. Every task ends with something independently testable — a run someone else could execute and get a pass or fail from, without reading ahead.

Inside a task, a step is one action, done in two to five minutes. A test-first task's steps typically run:

- Write the failing test.
- Run it — confirm it fails, and for the reason you expect.
- Write the minimal implementation that makes it pass.
- Run it — confirm it passes.
- Commit.

Not every task is test-first — a pure config change or a docs update isn't — but the rule holds regardless: one action per step, small enough that a stalled task points at a single step, not a stretch of paragraph.

## The plan header

Every plan opens with the same header, so an implementer who has read one plan can navigate any of them:

```markdown
# [Feature Name] Implementation Plan

> **For agentic workers:** Work through this plan task by task with
> either `delegating-tasks-with-review-gates` (a fresh subagent per task, each
> gated by review) or `working-a-plan-task-by-task` (inline, in the current session's
> main thread). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** [one sentence — what this builds]

**Architecture:** [2-3 sentences — the approach]

**Tech Stack:** [key technologies and libraries touched]

**Spec:** [path to the spec or design doc this plan implements — the
plan argues from the spec, so the spec travels with it]

## Global Constraints

[The spec's project-wide requirements — version floors, dependency
limits, naming and copy rules, platform requirements — one line each,
copied verbatim from the spec. Every task's requirements implicitly
include this section.]

---
```

The blockquote is the whole handoff instruction — it's how whoever picks up the plan next knows how it's meant to run, not just what it contains. Global Constraints exists so a project-wide rule gets stated once, not re-derived or silently dropped in half the tasks that should honor it.

## Task structure

```markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.ext`
- Modify: `exact/path/to/existing.ext:123-145`
- Test: `exact/path/to/test.ext`

**Interfaces:**
- Consumes: [what this task uses from earlier tasks — exact names and signatures]
- Produces: [what later tasks rely on — exact names and types]

- [ ] **Step 1: [one action]**
- [ ] **Step 2: [one action]**
- [ ] **Step N: Commit**
```

Two details carry more weight than they look like they should:

- **Line ranges on Modify, not just the path.** `existing.ext:123-145` tells the implementer where to look before they open the file; a bare path means reading the whole thing to find the change.
- **Interfaces is the only place cross-task agreement lives.** An implementer working Task 7 will not read Task 3's steps — they read Task 7's Interfaces block and trust it. If Task 3 actually produces a function called `clearLayers()` and Task 7's Interfaces block says `clearFullLayers()`, that mismatch ships, because nothing forces anyone to notice before runtime.

## No placeholders

Every step needs the actual content an implementer would otherwise have to invent. These are plan failures, not style preferences — an implementer who hits one either stops to ask, costing a round trip, or guesses, costing a review cycle:

| Placeholder | What it hides |
|---|---|
| "TBD", "TODO", "implement later" | The task wasn't planned — it was named |
| "Handle appropriately", "add validation", "handle edge cases" | The specific edge cases, unstated because they were never enumerated |
| "Write tests for the above" | The actual test code and its assertions |
| "Similar to Task N" | The actual content — a plan isn't guaranteed to be read in order, and Task N may not be nearby when this one is worked |
| A step that describes an action without showing it | The code, command, or exact text a code step requires |
| A type, function, or method used but never defined in any task | Which task actually owns it |

A plan with a placeholder in it isn't a draft of a finished plan — it's a to-do list wearing a plan's formatting.

## Self-review

Before calling the plan finished, check it against the spec with fresh eyes — genuinely fresh, not the eyes that just finished writing it. Run this yourself first. For a plan whose blast radius justifies the extra confidence, the same checklist is also what to hand a second reviewer — human or a freshly dispatched subagent with no sunk cost in the plan's current shape — since writing it and reviewing it are different acts even when done back to back by the same author.

| Check | Question |
|---|---|
| Spec coverage | For every section or requirement in the spec, which task implements it? List the gaps. |
| Placeholder scan | Does anything match the "No placeholders" table above? |
| Type consistency | Do the names, signatures, and types a later task's Interfaces block relies on match what the producing task actually defines? |
| Task decomposition | Does every task have a clear boundary, with steps concrete enough to act on without guessing? |
| Buildability | Could an implementer with zero context on this codebase follow the plan start to finish without getting stuck? |

Fix what you find inline, immediately — no separate re-review pass required. A spec requirement with no task gets a new task, on the spot.

Calibrate what's worth flagging: an issue that would cause an implementer to build the wrong thing, stall, or guess is real. A wording preference or a "nicer" phrasing isn't, and it doesn't block the plan. When a second reviewer runs this same list, the output is a status — approved, or issues found — with each issue naming its exact task and step and why it would derail implementation, and any recommendations kept separate and explicitly non-blocking: a plan doesn't fail review over advice.

## When execution reveals the plan was wrong

A plan is a prediction, and predictions miss once real code pushes back — a signature comes out different than guessed, a file needs splitting the plan never called for. That's not a failure of this skill; it's `working-a-plan-task-by-task` and `delegating-tasks-with-review-gates` correctly treating a plan/codebase contradiction as a blocker to raise rather than a wrinkle to absorb. What this skill owns is what happens to the *document* once that's resolved: note the actual outcome at the task that produced it, briefly, so a reader hitting Task 9's Interfaces block finds the signature Task 4 really shipped, not the one originally guessed at. An Interfaces block that quietly drifts from what got built costs the next task the same way an undefined reference would — the difference is only that this one used to be right and nobody updated it. Whether to also revise the original steps or just annotate the divergence is a call for whoever's executing; either way, the plan stops earning its keep as a reference the moment its predictions and the shipped code disagree and nothing on the page admits it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Implementer stops mid-task to ask what a step means | A placeholder slipped past self-review, or a step described the goal without showing the content |
| Two tasks feel like they should really be one | Split at a boundary a reviewer wouldn't actually reject independently — right-sizing skipped |
| A task can't be tested until three others are also done | File structure and task boundaries were decided separately instead of together |
| Task 9 calls something Task 4 never defined under that name | Interfaces blocks written from memory instead of checked against the producing task |
| The plan reads fine but nobody can say which spec line it satisfies | Tasks written from the shape of the code, not from the spec, and never checked back against it |
| Self-review finds nothing | Run by the same eyes that just wrote the plan, in the same sitting, with no actual distance |
| A plan skips Global Constraints in the header | The spec's project-wide rules got copied once per task instead of once, and drifted |
| An Interfaces block cites a signature the codebase no longer has | The plan recorded a prediction and never got updated once execution produced something different |

## Red flags

- "I'll figure out the exact command when I implement this" — that's a placeholder with different words.
- "Similar to Task N" anywhere in the document.
- A task with no run, test, or check anyone could point to as its pass/fail signal.
- File structure decisions still being made while writing Task 4 — structure should already be locked by then.
- Self-review finished in the same breath as the writing, with no fresh read of the spec in between.
