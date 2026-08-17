---
name: knowing-when-to-stop
description: Use when work looks done and the next action would be another polish, refactor, or check; when a self-review loop has run several times and the findings keep shrinking; when the first green result tempts an immediate finish; when the same class of fix has failed several times running; or when remaining work is about to be left in place without being named. Covers diminishing returns, gold-plating, bounded passes, and explicit handoff.
---

# Knowing when to stop

## Overview

Stopping early ships something that merely looks unbroken; stopping late spends the user's budget on polish nobody asked for. Both come from the same omission — no finish line defined before the last mile started.

## When to use

- The work looks done and the next action would be another pass, polish, or check
- A verification loop has run several times and the findings are getting smaller
- The first green result arrived and finishing right now is tempting
- Work will be left unfinished and is about to go unnamed
- The same class of fix has failed several times in a row
- Not for: whether a specific claim has evidence behind it — `confirming-before-claiming-done` owns that gate. This owns how many passes to run and what finished means.
- Paired with `karen-and-the-manager`: that skill is built to never run out of complaints on purpose; this one decides how many of them are worth another pass.

## Define done before the last mile

Done is: every item in the request demonstrably works, on the inputs the user actually has, and everything remaining is named. Write that sentence at the start; the task ends when it is true.

Without it, done degrades into one of two defaults — the first thing tried passed, or you ran out of obvious things to do — and neither has anything to do with the request.

## Done against not obviously broken

| Dimension | Not obviously broken | Done |
|---|---|---|
| Tests | The test you wrote passes | The requested behaviour is exercised, including the input the user described |
| Errors | Nothing raised | Failure paths were provoked deliberately and behaved |
| Scope | The main path works | Every requested item has a stated state |
| Inputs | It worked on your example | It worked on the real or boundary input |
| Integration | The unit works | The caller works, in place |

The most common early stop is a green result on the thing you built, never checked against the thing that was asked. Re-read the request verbatim before declaring done — the original wording, not your restatement of it, since the restatement is where the requirement quietly changed.

## Bounded passes instead of open-ended review

1. Complete the work fully.
2. One batched inspection round — assemble every check worth running, run them together, read all findings before fixing any.
3. Fix everything that round produced, in one batch.
4. At most one confirming round.
5. Stop.

Open-ended self-review has no terminating condition and reliably converges on cosmetics, because cosmetics are inexhaustible. Ceiling: if a round yields only findings you would not have bothered mentioning to a colleague, the passes are finished.

Stop when any two of these hold:

- The last pass produced nothing that would change behaviour.
- You are modifying things you already modified in an earlier pass — churn, not progress.
- Findings have become naming, formatting, or preference.
- The pass costs more than the expected cost of the defects it would find.
- You cannot state what the next pass is looking for. This is the sharpest of the five: a pass with no hypothesis is a reread, not a check.

## Gold-plating, specifically

Each of these is unrequested scope wearing the costume of thoroughness:

- Generalizing for a second caller that does not exist
- Adding configuration nobody asked to configure
- Handling inputs the system cannot produce
- Documenting internal code no one requested documentation for
- Optimizing with no measurement showing the cost mattered
- Extending test coverage to code this change did not touch

The test is not whether it improves the code. It is whether the person who asked would spend their own remaining budget on it, having seen the current state.

## Stopping while blocked

Blocked is also a stop, and it takes the same discipline. Three failed attempts of the same shape mean the model of the problem is wrong, not that the fourth variation will land — stop fixing, restate the problem, and either change the diagnosis or hand it back. Iterating on the fix instead of on the diagnosis is the most expensive available way to be stuck.

## The explicit handoff

Whatever is not done gets stated with four things: what it is, where it is, why it stopped, and what it would take. Unfinished work that is named is a status; unfinished work that is quiet is a defect the reader inherits without knowing.

This includes checks you chose not to run. "Not exercised against real data", "not tested under concurrency", "not run on the second platform" is what the reader needs in order to decide what happens next, and omitting it is the same failure as omitting a broken feature.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The fifth pass finds only naming issues | Passes ran until boredom rather than to a criterion |
| Declared done; the user immediately finds a requested item broken | Verified the built thing, never re-read the request |
| Improvement continued past the point the request was satisfied | Done defined by feeling rather than by the request text |
| Re-editing lines you edited two passes ago | Churn misread as refinement |
| Flexibility added with no second caller | A requirement anticipated instead of waited for |
| The unfinished part is absent from the report | Stopping conflated with finishing |
| Five attempts at one defect, all the same shape | Iterating on the fix instead of on the diagnosis |

## Red flags

- "One more quick improvement"
- "While I am verifying, let me also…"
- "It passed, so we are done" after a single check
- Relief interpreted as completion
- A summary with no remaining-work section on a task that plainly has one
- Reaching for the next polish because stopping feels like giving up
