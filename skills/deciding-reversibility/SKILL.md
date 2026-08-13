---
name: deciding-reversibility
description: Use when a choice is blocking progress and the deliberation is costing more than the choice would, when picking a name, a library, a file layout, a schema, an interface, or a default, when a discussion has gone several rounds without new information entering it, or when an action would write data, publish an interface, delete something, or otherwise be expensive to undo. Covers one-way and two-way doors, cost of delay, and decision altitude.
---

# Deciding reversibility

## Overview

Match the cost of deciding to the cost of being wrong. The dominant failure is not bad decisions — it is expensive deliberation spent on choices that could be undone in ten minutes, while the genuinely irreversible ones get made in passing.

## When to use

- A choice is blocking progress and deliberating now costs more than the choice does
- Picking a name, a library, a layout, a schema, an interface, or a default
- A discussion has run several rounds with no new information entering it
- About to write data, publish an interface, send something, or delete
- Not for: generating the options in the first place — `steelmanning-alternatives` owns that. This owns how much process the choice deserves once options exist.

## What makes a door one-way

Not importance, and not how permanent it feels. These predictors:

- **Data gets written in a shape you would later have to migrate**, or data gets destroyed.
- **Consumers outside your control have copied it** — published interfaces, wire and file formats, URLs, command-line flags, anything already integrated against.
- **It was said out loud** — user-visible names, external commitments, anything announced.
- **Trust or secrecy is spent** — leaked credentials, rewritten history, messages sent.
- **Other work will compound on it** before anyone revisits it.

Two-way despite feeling permanent: internal module boundaries, most naming, directory layout, any dependency reachable only through your own wrapper, algorithm choice behind a stable interface, defaults with no data written against them yet.

The rule that catches most misclassification: **reversibility is the cost to undo after the work built on top of it, not the cost to undo today.** A choice that is trivial to change now and gets built on for two weeks was a one-way door the day it was made. The clock closes doors, not the choice.

## Matching process to weight

| Cost to undo | Budget | Form |
|---|---|---|
| Minutes — rename, reorder, swap a local implementation | Decide immediately, alone | Pick; note the runner-up in one clause |
| Hours to a day — move a boundary, change an internal shape before data exists | One bounded comparison of two options | Pick; state the deciding criterion in a line |
| Days to weeks — a data shape with rows in it, an interface another team consumes | Write the options and the criteria; involve the owner | A recorded decision |
| Effectively permanent — public API, destructive migration, deletion, external send | Do not decide unilaterally | Surface it and wait for an explicit answer |

Calibration: if undoing costs under roughly an hour and no data or external consumer is involved, deciding should cost under five minutes. Three rounds of discussion about a ten-minute change has already cost more than the mistake would have.

## Delay against wrongness

Both terms are computable to within an order of magnitude, which is all that is required.

- **Cost of delay** = what is blocked, times how long. Dependent work queued behind a decision makes deliberation superlinearly expensive, and this is the term that gets forgotten.
- **Cost of being wrong** = undo cost, times probability of being wrong. A likely-but-cheap error is usually a better trade than an unlikely-but-expensive one.

The stopping predicate is not whether you feel sure. It is: **if another round of analysis would not change which option you pick, the analysis is finished.** Name the evidence that would change the answer; if you cannot name any, or it is not obtainable now, decide.

## Convert one-way doors into two-way ones

Usually higher leverage than deciding well — change the decision's class so it can be made in five minutes instead of five days.

- Put the choice behind an interface so the implementation can be swapped later.
- Version the format on day one so a second version is legal without a migration.
- Confirm the migration path exists before committing to a shape, rather than after.
- Gate it so rollback is a configuration change rather than a deployment.
- Keep the old path alive until the new one has carried real traffic.
- Deprecate rather than delete; append rather than overwrite.

The wrapper generally costs less than the analysis it removes. When it does not, that is the signal the decision genuinely deserves the analysis.

## Decide at the right altitude

The two errors are symmetric. Imposing a project-wide convention to settle one file turns a local choice into a global one and manufactures a default nobody validated. Re-deciding the same thing at twenty call sites produces twenty answers and no convention.

Predicate: if everyone facing this choice would make it identically, it belongs in one place. If the right answer varies with local context, lifting it upward creates a bad default and a stream of overrides.

## Pick and move, or refuse to

"Just pick one" is correct when the choice is reversible, writes no data, is invisible outside the code, and the options sit within noise of each other on every criterion that was named as mattering.

It is negligence when the choice writes data, is user-visible, sets a security or privacy default, is the thing you were specifically asked to think about, or the options differ by an order of magnitude on a criterion someone named. There, the fast move is not to pick — it is to surface the choice with a recommendation.

Record every fast decision in one line: the choice, the runner-up, and the observable that should trigger revisiting it. A decision with a named revisit trigger is one you are entitled to make quickly.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Extended analysis of something changeable in ten minutes | Weighted by how important it feels rather than by undo cost |
| Decided fast, and it wrote data | Confused "easy to change the code" with "easy to change the consequences" |
| The same question re-decided in every file | Never lifted to the altitude where it gets made once |
| Asked the user to choose between options they have no stake in | Escalated a two-way door; the interruption costs more than the error would |
| Waiting on information that is not coming | No named evidence that would change the answer |
| A once-reversible choice is now load-bearing | Weeks of work compounded on it while it was still provisional |
| Both options built to avoid choosing | Two implementations to maintain, and the decision still pending |

## Red flags

- "Let us make sure we get this right the first time" about something internal and wrapped
- A decision thread whose last two rounds introduced no new information
- Reaching for a migration, a deletion, or an external send at the same speed as an internal rename
- Choosing the harder-to-reverse option because it is marginally more elegant
- Treating a choice as settled because implementation already started on it
