---
name: red-teaming-your-own-work
description: Use when a change, plan, design, recommendation, or answer is finished and has been examined only by whoever produced it, when nothing looks wrong and nothing has been tried to make it look wrong, before handing off work someone will act on, or after a previous delivery came back with a defect a single skeptical pass would have caught. Covers adversarial self-review, attack checklists, and bounded review passes.
---

# Red-teaming your own work

## Overview

Rereading work confirms it; attacking it finds defects. Spend one bounded pass before delivery assuming the work is wrong and hunting for where — the productive question is not whether a defect exists but where it is hiding.

## When to use

- A change, plan, recommendation, or answer is finished and only its author has seen it
- Nothing appears wrong, and nothing has been attempted to make it appear wrong
- Before handing off a change set, a design, or a decision someone will act on
- A previous delivery came back with a defect one skeptical pass would have caught
- Not for: confirming a claim has evidence behind it (`confirming-before-claiming-done`), or diagnosing a failure you already know about (`diagnosing-before-fixing`). This is for work that currently looks correct.

## The stance

"Is this right?" is answered by the same reasoning that produced the work, and it returns yes. "If this is wrong, where is it wrong?" forces the generation of candidate locations, which is a different task with a different output. Commit to finding at least one real defect; a pass that ends with none, and with no surprise about that, was a reread.

## The attack list

Run all six. Each costs a minute or two and lands somewhere predictable.

| Attack | The question | Where it usually lands |
|---|---|---|
| Wrong assumption | What did I take as given without checking? | The shape of an input, the behaviour of an external call, what one word in the request meant |
| Unhandled input | Empty, zero, one, maximum, absent, duplicate, out of order, concurrent, hostile, wrong type | Boundaries between components, not inside them |
| Misread requirement | Reread the original request verbatim, not your restatement of it | A qualifier in the request that never made it into the work |
| Silent failure | Where can this fail with nothing raised, logged, or returned? | Swallowed errors, defaults substituting for missing data, retries that give up quietly |
| Wrong altitude | Too specific — one case hardcoded; or too general — extension points with one caller | Abstraction and configuration added on speculation |
| Adjacent breakage | Who else calls this, parses this shape, relies on this ordering or this timing? | Callers you never opened, and anything depending on the old behaviour |

A seventh when the work is an addition: delete it mentally and ask what breaks. If nothing does, it is unjustified rather than wrong.

## Attack the strongest part

The weak part is already flagged in your own head — you know the rough edge and the TODO, and reviewing it returns what you already knew. Defects concentrate where attention stopped early: the obviously-correct core, the part you would skip in someone else's review, the piece written fastest because it was familiar.

Operational rule: name the two pieces you would not bother reviewing, and review exactly those. The confidence that makes them skippable is what left them unexamined.

## Distance, obtained cheaply

Reviewing immediately after writing re-executes the same reasoning path and reproduces the same blind spot; you read what you meant rather than what you wrote. A break works because the memory of intent decays faster than the artifact does. When no break is available, substitute:

- Read the diff, not the file. It strips your intent and leaves the change.
- Read as the caller or the operator, never as the author. State what it does from the artifact alone, then compare against what it was supposed to do.
- Read the pieces in reverse order, last to first.
- Discard the intermediate reasoning entirely and reload only the original request and the final artifact.

The comparison that finds defects is written-behaviour against requested-behaviour, with intended-behaviour excluded from the room.

## Bounded, then stop

One pass. Collect every finding before fixing any of them — fixing mid-pass truncates the pass, because attention moves to the repair. Then rank and act:

| Finding | Action |
|---|---|
| Wrong result on an input the system will actually receive | Fix before delivering |
| Correct, but fails on input nothing can produce | Note in the report, do not fix |
| Disagreement with the request's own design | Raise it, do not unilaterally change it |
| Naming, formatting, structure | Fix silently or leave; do not report |

Fixes written during the pass are unreviewed new work carrying the same defect rate as the original — re-attack the fix specifically, not the whole artifact again. A second full pass is warranted only when the first found a defect of a *kind* you had not considered, which means the pass was aimed wrong rather than that a third is owed. A second pass yielding only nits is the signal to stop.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The pass finds only cosmetic issues | Asked whether it is right instead of where it is wrong |
| Confidence is higher after the review than before it | The pass confirmed intent rather than testing behaviour |
| Found a defect, fixed it, shipped without re-attacking the fix | Fixes treated as corrections rather than as new untested code |
| The review keeps expanding into a rewrite | No budget; adversarial mode has no natural stopping point |
| "Edge cases are handled" with no enumeration | Handled the cases you thought of, which is exactly the set that produced the defect |
| The part reviewed is the part you already doubted | Attacked the known weak point; the strong point stayed undefended |
| A large change reviews clean in one sitting | Reviewed the artifact you remember writing, not the one that exists |

## Red flags

- "It is simple enough to be obviously correct"
- "I wrote it carefully"
- "Testing will catch it" about something you could provoke with one command
- Ending the pass pleased rather than surprised
- Reviewing what you just wrote, in the order you wrote it, immediately after writing it
