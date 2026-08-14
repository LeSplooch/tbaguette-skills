---
name: writing-adrs
description: Use when recording a technical or architectural decision — picking between technologies, changing a system boundary, adopting or refusing a pattern. Also when a past choice looks wrong and nobody remembers why, when the same argument recurs, when someone asks why the system is built this way, or when deciding whether a choice is worth recording. Covers decision records, ADRs, design docs, rejected alternatives, status, and supersession.
---

# Writing ADRs

## Overview

A decision record exists for one purpose: so a future reader can determine whether the decision still holds, without finding you. That test — not the template — decides what belongs in it. The forces make it checkable; the rejected alternatives make it re-openable.

## When to use

- A choice is expensive to reverse, or commits the project to an interface, a format, or a vendor
- The result will look wrong to a competent newcomer without context
- The same question has now been argued in two separate threads or reviews
- Someone is about to "just decide it in the standup"
- **Not for:** how the system works today → `writing-durable-docs`. What shipped in a version → `writing-release-notes`. Why an incident happened → `writing-postmortems`. The implementation plan that follows the decision.

## Which decisions earn a record

Any one predicate is sufficient.

| Predicate | Concrete test |
|---|---|
| Expensive to reverse | Undoing costs more than a week, or touches persisted data, a public interface, or a paid commitment |
| Will look wrong later | A competent reader's first reaction is "why not the obvious thing?" |
| Argued twice | The same question resurfaced in a second discussion, review, or channel |

Not worth a record: anything reversible in an afternoon, anything with only one viable option (no alternatives means no record — a code comment carries it), and anything a linter or formatter could enforce.

Volume is diagnostic. More than ~2 per week and the team is recording tasks; fewer than ~1 per quarter on an actively changing system and the reasoning is living in chat logs that will be unsearchable within a year.

## The five parts

1. **Title** — a numbered, immutable noun phrase naming the decision: `0014: Single writer for the ledger table`. Not "Database decision", not a question.
2. **Status** — Proposed / Accepted / Superseded by NNNN / Deprecated, each with a date. A record with no date cannot be evaluated against a timeline.
3. **Context and forces** — the situation and the constraints that were live *at the time*: load figures, team size, deadline, existing commitments, what was unknown. Numbers, not adjectives — "12k writes/sec peak, two engineers, must ship before the contract renewal" is checkable in two years; "high load and limited resources" is not. This section is what a future reader compares against the present to decide whether the decision still stands. Mark which of these forces you do not control: one you own changes only when someone here changes it, and that change arrives with a commit, while one belonging to a third party — a vendor's capabilities, a platform's rules, a dependency's limits — can expire with nothing in the repo moving. Flagging which is which tells a future reader exactly what to re-verify, instead of leaving them to re-litigate the whole decision in order to test one part of it.
4. **Decision** — active voice, present tense, one sentence first: "We route all ledger writes through a single service." Specifics follow. "It was decided that" is the passive that erases the owner.
5. **Consequences** — what becomes true, split into what gets easier, what gets harder, and what the team must now not do. Include the ones you dislike. A consequences section with no downside reads as advocacy and readers discount the entire record.

## The alternatives are the point

- Every alternative gets what it was and **the specific reason it lost**. "Rejected: X — requires a coordinator we don't have staffed until Q3" survives; "Rejected: X — not a good fit" is a placeholder that tells a future reader nothing.
- Include the option you would have taken with more time, money, or people. Future readers arrive precisely when those exist, and that entry is the trigger to revisit.
- Include "do nothing" when it was genuinely live, with what it would have cost.
- Where an alternative lost on a measurement, cite the number *and how it was taken*. A benchmark without a method cannot be re-run and so cannot be challenged.
- The failure mode is recording only what was accepted. Then every future reader re-derives and re-rejects the same three options at full cost, and eventually someone adopts a rejected one because nothing said it had been tried and why it failed.

## Write it at decision time

- Alternatives are legible only while they are still live. After implementation the chosen path looks inevitable, and the rejected ones compress into "we considered some other things".
- Draft the record at status Proposed and hold the discussion on the draft. Comments on it are the decision process; the merged file is the outcome. This also removes the separate "write it up" task that never happens.
- Order of writing: forces → alternatives → decision → consequences. Writing the decision first turns the alternatives section into justification, which is the same failure in a different order.
- If a record must be reconstructed after the fact, label it retroactive and give the original decision date. Readers weigh a reconstruction differently, and should.

## Status and supersession, never edit history

- Records are append-only in substance. Fix a typo; never rewrite context, alternatives, or the decision.
- A changed decision produces a **new** record. The old one is marked `Superseded by NNNN` and stays readable in full. Editing the old record destroys the only evidence of what the team believed and why the belief was reasonable.
- The superseding record's context names the force that changed: "0014 assumed 12k writes/sec peak; sustained peak is now 400k."
- Deprecated ≠ Superseded. Deprecated means the subject is gone. Superseded means a later decision replaced this one and the subject still exists.

Marking which forces you do not control is what makes the record cheap to re-check years later; `revalidating-decisions` is the reader's half of that exchange.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Why is it built this way?" and nobody knows | The decision was recorded, the alternatives were not |
| Records written after shipping, all reading as obviously correct | Written once the outcome had reshaped the memory of the options |
| Nobody reads the records | Filed away from the code, or diluted by trivial ones |
| The same debate restarts every six months | No forces recorded, so nobody can tell whether it still applies |
| A twelve-page design document | The decision is buried; a reader cannot extract it in 30 seconds |
| Every record's status is Accepted, forever | No supersession discipline; the set describes a system that no longer exists |
| Consequences are all upside | Written to persuade a reviewer, not to inform a successor |
| A reader has to ask the author what the record means | The record failed its only test |
| A decision still binding years after its blocker disappeared | The forces mixed what the team controls with what a third party does, so nobody knew which one to re-check |

## Red flags

- "Everyone knows why we did this"
- "I'll write it up once it's shipped"
- "We didn't really consider alternatives" — either it was trivial and needs no record, or they exist and are unwritten
- Editing an accepted record so it matches the current system
- A record with no date, or a decision in the passive voice
- Listing alternatives that were never seriously in contention, to make the choice look considered
- Filing the record somewhere the implementers will not pass through
