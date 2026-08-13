---
name: writing-postmortems
description: Use when writing an incident review or postmortem after an outage, degradation, data loss, failed deploy, or security event; when building a timeline, arguing about root cause, or drafting follow-up actions; when a draft is drifting toward blaming a person or converging on a single cause; or when deciding what to publish externally. Covers blameless analysis, contributing factors, detection and mitigation timing, and action item ownership.
---

# Writing Postmortems

## Overview

A postmortem is an engineering document about conditions, not an account of who did what. Any sentence in the analysis that ends at a person is unfinished — the unwritten remainder is what made that action reasonable at the time, and that remainder is the only part you can fix.

## When to use

Write one for: any customer-visible degradation, any data loss or corruption regardless of size, any security event, any incident where the mitigation was applied without understanding why it worked, and any near-miss caught by luck rather than by a control.

- **Not for:** coordinating a live incident. Documenting a recurring procedure. Recording a decision that has not failed yet → `writing-adrs`.
- Skip the formal document for a fully-understood single-service blip under the alerting threshold with no customer impact — but log it, because three of those are a pattern.

## Blameless in mechanism, not just tone

Polite language is the easy half and it fools people into thinking the work is done. The mechanism:

- Every human action in the timeline is annotated with **what was visible at that moment** — the dashboard as it read, the alert text as it fired, the runbook as it was written, and the prior N times the same action was correct.
- The question is never "why did they do that". It is "what made that the reasonable action, and what would have made a better one obvious?"
- Names appear in the timeline as roles ("the on-call engineer"), never in the analysis as causes. If a "why" chain terminates in a name or a personality trait, the chain stopped early.
- Delete "human error", "failed to", "should have", "neglected to", "forgot to". Each is a stopped investigation wearing the costume of a finding.
- **Counterfactuals are the trap.** "If only they had checked the dashboard" describes a world that did not exist, feels like analysis, and produces no action item — you cannot fix the past. Convert each one into a present-tense system question: "what would have put that number in front of them?" That question has an answer you can build.
- Blameless is not ownerless. The analysis assigns no blame; the action items assign named owners.

## A timeline built from evidence

- Each entry: timestamp with timezone (prefer UTC), what happened, and the evidence — log line, graph, alert, message, deploy record. An entry backed only by recollection is marked as recollection.
- Include the unremarkable precursors: the deploy three days earlier, the config change, the traffic shift, the certificate issued 89 days ago, the feature flag flipped last week.
- Mark the points where **understanding changed**, not only where the system changed. "14:22 — responders believed the cache was the cause" carries as much information as "14:22 — cache evicted". The gap between the system's state and the responders' model of it is where mitigation time is actually spent.
- Required marks: change introduced, impact began, first signal of any kind, first human awareness, mitigation started, impact ended, resolution confirmed.

## Measure detection and mitigation separately

| Interval | From → to | A bad number indicts |
|---|---|---|
| Detect | Impact began → first human aware | Alerting and observability |
| Diagnose | Aware → cause understood well enough to act | Tooling, runbooks, system legibility |
| Mitigate | Action started → impact ended | Rollback path, flags, blast-radius controls |
| Total impact | Impact began → impact ended | The customer's number, and the only one they see |

Reporting total impact alone hides which of the three is broken. A four-hour incident detected in 40 minutes and mitigated in 5 is a diagnosis problem, and "add more alerts" would be the wrong action item. Also record **how** it was detected — customer report versus internal alert. Customer-reported is a finding in its own right regardless of how fast the rest went.

## Contributing factors, not a root cause

- Complex systems fail from a conjunction of conditions. Forcing a singular "root cause" makes you pick one necessary condition among several, and the one picked is reliably the last thing changed or the cheapest thing to fix.
- Five-whys converges on a single narrative because each "why" is answered once. Where a why has two or more true answers, it **branches** — and the branches are where the cheap fixes hide. A five-whys that never branched was a choice, not a discovery.
- Separate three roles explicitly; a factor is usually only one, and fixing the trigger fixes nothing:
  - **Trigger** — what set it off this time (a deploy, a traffic spike, an expiry). Often mundane, and it will recur.
  - **Vulnerability** — the condition that made the trigger harmful (unbounded queue, missing timeout, single writer, no back-pressure). This is the durable finding and the one worth spending on.
  - **Amplifier** — what made it larger or longer (retry storm, alert fatigue, absent rollback path, nobody awake with access).
- Test the set: remove one factor and ask whether the incident still happens at this severity. If yes, it was context, not a contributing factor.
- Record what worked. Controls that limited the blast radius are findings, and they are exactly what gets deleted next quarter if nothing documents that they paid for themselves.

## Action items that get finished

Each item needs four things — a specific change, one named person, a due date, and a tracked ticket. Missing any one and it will not happen.

- Size limit: finishable by one person in under about two weeks. Anything larger is a project and needs its own record and its own sequencing.
- Classify each as **prevent** (remove the vulnerability), **detect** (find it sooner), or **mitigate** (shrink the impact). A list of only prevent items is optimistic — the next failure arrives through a different trigger, while detect and mitigate items pay out on failures nobody predicted.
- "Improve monitoring", "add more tests", "be more careful", "document the process" — unowned, undated, unfinishable. Their presence is the clearest sign the analysis stopped at a symptom.
- Cap the count. Three to five items that ship beat fifteen that do not; a list of fifteen teaches the team that the list is theatre.
- Track completion and report it. An unfinished action item from a previous postmortem reappearing as a contributing factor in a new one is the highest-signal metric a team has about itself.

## Publishing

- **Internal:** the full document — timeline, evidence, factors, actions, owners — circulated as widely as the organization allows. A postmortem read only by the team that wrote it teaches only them, which is the smallest possible return on the incident.
- **External:** impact (what, when, who was affected), what was done, what changes. No internal names, no unconfirmed causes, no dates you do not control.
- Draft within ~48 hours while recollection is retrievable, but publish only once the timeline is evidence-backed. A fast, wrong timeline gets cited forever.
- The review meeting reads the document; it is not a presentation. If it needs slides, the document failed.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The review turns into someone defending a decision | Actions were stated without stating what was visible at the time |
| Last quarter's action items are still open | Unowned, undated, or too large to finish |
| The same class of incident recurs with a new trigger | The trigger was fixed and the vulnerability was left |
| "Root cause: engineer ran the wrong command" | Investigation stopped at the person; the missing guardrail was the finding |
| Twenty pages nobody read | Narrative padding around a timeline and a factor list that were the whole document |
| Every incident produces "add an alert" | Detection is the only lever the team knows; prevent and mitigate never considered |
| Timeline assembled from memory a week later | Evidence was never captured during the incident |
| Leadership asks "was this fast or slow?" and nobody can say | Only total impact was measured, hiding which interval is broken |

## Red flags

- "We just need people to be more careful"
- "The root cause was…" (singular, in a system with more than one moving part)
- "If they had just…"
- An action item with no name beside it
- Writing action items before the timeline is finished
- Concluding that the system worked as designed and nothing needs to change
- Deciding not to write one because "we already know what happened"
- A timeline that starts when the alert fired
