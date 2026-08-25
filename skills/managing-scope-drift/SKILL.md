---
name: managing-scope-drift
description: Use when work has grown past what was asked or the diff is larger than the request implies, when a fix seems to require touching something adjacent, when a real problem is found in passing, when tempted to also clean up, rename, refactor, or improve nearby code, or when one requested item is drifting toward being quietly omitted. Covers necessary versus adjacent versus discovered work, silent widening and narrowing, and reporting the remainder.
---

# Managing scope drift

## Overview

Scope changing is not a failure; changing it silently is. Both directions count equally — quietly doing more and quietly doing less are the same defect, because both take from the user the ability to decide what they are paying for.

## When to use

- The work has grown past what was asked, or the diff exceeds what the request implies
- A fix appears to require touching something adjacent
- A genuine problem is found in passing, unrelated to the task
- Tempted to also clean up, rename, refactor, or improve nearby code
- One requested item is harder than the rest and is drifting toward omission
- Not for: judging whether the work is finished to standard — that is `knowing-when-to-stop`. This is about what belongs in the work at all.

## Three classes, one test

For every candidate piece of work, ask: can the request be delivered, working, without this?

| Class | Test | Action |
|---|---|---|
| Necessary | The request is blocked or broken without it | Do it. One line in the report. No permission needed |
| Adjacent | Improves code you happen to be touching; the request works regardless | Do not do it. Record it, report it |
| Discovered | A real defect or false premise found in passing, unrelated to the request | Report it. Fix only if it blocks the request, and never silently |

The necessary/adjacent line is where most drift enters, always wearing the same disguise: "I had to restructure it to add this cleanly." Sharper test — if the extra work were reverted and the requested change kept, would the deliverable still pass its own acceptance? If yes, it was adjacent.

## When to interrupt

Discovered work interrupts immediately only for: data loss or corruption, credential or secret exposure, a false premise underneath the request itself (`reading-specifications` covers spotting one before the work rather than during it), or a discovery that makes the current approach wrong. Everything else waits for the final report.

The gate matters in both directions. Interrupting for a medium-severity finding costs a context switch and teaches the user that your interruptions are skimmable, which is what makes the next high-severity one get skimmed.

## Widening and narrowing

**Silent widening is not generosity.** It inflates the diff until the requested change can no longer be reviewed on its own; it couples unrelated risk into a single revert; it spends a budget nobody agreed to; it invalidates the estimate; and it removes the option to say "not now". A twelve-line change buried inside four hundred lines of cleanup gets approved without being read — the cleanup did not add value, it removed review.

**Silent narrowing is worse**, because the user believes they have something they do not. The pattern is specific and predictable: four items were requested, three were straightforward, the fourth was hard, and the report describes the three. The hardest item is the one that goes missing, and it is also the one they most needed done.

The rule covering both: **every item in the request appears in the report with a state** — done, partial with what is missing, or not done with why. No requested item silently absent, and nothing present that was not asked for.

That rule needs somewhere to live that a long task cannot erode, which is what `finishing-what-you-started`'s acceptance ledger is: the request's items written to a file before the work, so narrowing has to happen against a list rather than against a memory of the request. Its surrender rule is this section's other half — a line marked surrendered is narrowing reported, a line quietly edited or deleted is narrowing concealed, and the second one leaves a fully checked list behind.

## Noticing it in flight

Tripwires. Any one means stop and re-read the request text, not your memory of it.

- Files touched exceed what the request implied. Count them.
- You are editing a file you did not expect to open when you started.
- The justification for the current edit chains through more than one "and to do that I need to".
- Elapsed effort has passed roughly twice the initial estimate with no re-estimate.
- You are writing something whose value you could not have justified at the start of the task.

When necessary work turns out to be substantial, say so before doing it rather than after: name what has to change first, its rough size — a range, on `estimating-effort`'s terms, since a point estimate offered mid-drift is the least reliable number anyone will produce that day — and offer the smaller alternative. One sentence, and it preserves a choice instead of presenting a completed fact.

## Handing off the remainder

Adjacent and discovered items belong in the report as actionable one-liners: where it is, what is wrong, roughly how big. "Also noticed some technical debt" is not a handoff — it transfers a feeling rather than a task. An item that cannot be stated with a location and a size was not a finding, it was an impression.

When the user adds scope mid-task, do not absorb that silently either. Say what it does to the current deliverable — finish first, switch now, or both with the second later — and let them choose.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The diff is several times the size the request implied | Adjacent work reclassified as necessary in the moment |
| "While I was in there, I also…" | Widening reported past the point where it could be declined |
| A requested item is missing from the report | Silent narrowing; the hardest item is the one that disappears |
| A bug found in passing was fixed and never mentioned | Discovered work absorbed; the user never learns the behaviour changed |
| A refactor justified as required by the feature | Reverting the refactor would not break the feature |
| The handoff list is vague debt | Findings were never converted into items at the moment they were found |
| The task never converges | Every discovery re-entered the scope; the work has no defined edge |

## Red flags

- "While I am here"
- "It would be wrong to leave it like this"
- "This only takes a minute", said about the third such minute
- Finding a second problem and fixing it before reporting the first
- Noticing the request had four parts only while writing the summary
- A handoff you would have to explain in two unrelated halves
