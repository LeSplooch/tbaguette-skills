---
name: explaining-technical-work
description: Use when writing an explanation of technical work for another person — a status update, a summary of a change or an investigation, a handoff, an escalation, a design walkthrough, or a recommendation. Also when the reader is not in the code, unsure how much detail to include, or when a draft has become a narrative of what was tried. Covers leading with the conclusion, altitude, naming uncertainty, and length.
---

# Explaining Technical Work

## Overview

An explanation is written for one named reader with one pending question. Everything that does not move that reader toward their next action is a cost they pay and you do not. Name the reader and the question before the first sentence, or the document will be written at your altitude by default.

## When to use

- Reporting what an investigation found, or what a change does
- A status update, escalation, or handoff to someone not in the code
- Recommending a course of action to someone who will decide
- A draft has grown a chronology of what was tried
- Readers keep replying with questions the document already answered
- **Not for:** documentation that must outlive this exchange → `writing-durable-docs`. Version-to-version changes → `writing-release-notes`. Incident analysis → `writing-postmortems`.
- **Not for:** the set of choices that goes underneath the report → `offering-the-next-move`. They are two deliverables in one message, and the length budget above applies to this one only — the explanation does not get shorter to make room for the options.

## Answer first

- Line one is the conclusion, stated as a claim. The reasoning follows it. A reader who stops after one line still has the answer; one who needs the reasoning keeps going.
- Chronology is the default shape and it is the wrong one. "I looked at X, then Y, then found Z" makes every reader reach the end for a fact that fit in a sentence.
- The conclusion answers *their question*; it is not a summary of your activity. "The leak is in the connection pool, not the cache" is a conclusion. "I investigated the leak" is a status with the answer removed.
- With no conclusion yet, say what is known and what is still open, in that order, labelled as in progress. "No answer yet, here is what is ruled out" in line one beats a real answer in paragraph four.
- Answer-first is not the same as short. A three-page brief still opens with the recommendation.

## Choose altitude by what the reader will do next

| Their next action | Give them | Cut |
|---|---|---|
| Decide | The options, the cost of each, your recommendation, what changes if they pick differently | Implementation detail, your process |
| Review or approve | The risk, the blast radius, what you verified and what you did not | The mechanism, unless it carries the risk |
| Implement | Interfaces, invariants, failure cases, where to start | Rationale beyond one line, rejected alternatives |
| Stay informed | Status, what changed, when the next update lands | Everything else |
| Learn the system | The model, the why, one worked example | Exhaustive edge cases |

Wrong altitude is the most common failure here and it is invisible to the writer, because the correct level always feels like the level you are already at. The check is external: name the reader, name their next action, then read your draft against that row.

## Say what it means, not what you did

- The specific failure: reporting activity where the reader needed implication. "Refactored the queue consumer and added retries" tells a manager nothing. "Duplicate charges can no longer happen during a broker failover" answers the question they actually had.
- Apply *so what, for this reader?* to every sentence about your work. No answer means cut it or translate it.
- Delete the process narrative. What you tried, in what order, and which dead ends you hit is rarely what was asked. Two exceptions, both narrow: when the reader would otherwise repeat the dead end, or when the process **is** the evidence (a bisect, a measurement) — and then it is one line, not a story.
- Use the reader's vocabulary, not the system's. Internal service and class names are a private language; translate into what the reader owns — "checkout", "the nightly export". Introduce an internal name only when they will need it to search.

## Name uncertainty rather than smoothing it

- Confident prose about something half-known is the most expensive habit in technical writing, because the reader cannot separate it from the parts you are sure of and calibrates on the whole document.
- Mark each load-bearing claim as verified (and how), inferred (from what), or assumed (and what would falsify it). `calibrating-confidence` owns those three tiers and how each must read in a sentence; this is where they meet a specific reader who is about to act.
- "I believe X because Y, but I have not checked Z" is more useful than "X" — the reader now knows exactly what to check and can do it without you.
- Distinguish "I do not know", "nobody knows", and "I did not check". They lead to three different next actions and collapsing them wastes the reader's.
- State a confidence level only where it would change the reader's decision. Elsewhere it is hedging that costs a line.

## Make claims checkable

- Point at the evidence: the file and symbol, the query, the log window, the commit, the measurement and its method. A claim with a pointer can be verified without you; a claim without one must be trusted or re-derived, and both are more expensive than the pointer.
- Numbers get units and a baseline: "p99 fell 340ms → 90ms under the same synthetic load", not "much faster".
- Where a claim rests on a single observation, say so. One run is evidence; it is not a measurement.

## Length is a cost the reader pays

- Match the question. A one-line question gets a one-line answer. Answering a yes-or-no question with five paragraphs is a failure, not diligence, and it teaches the reader to stop asking.
- Rough calibration: a status update ≤5 lines; "what did you find" one short paragraph plus evidence pointers; a decision brief ≤1 page with the recommendation on line one; a walkthrough as long as it needs, with headings so it can be skimmed and re-entered.
- Padding signals: restating the question, "as mentioned above", background the reader already holds, a closing summary of a document short enough to reread.
- Cut in this order: process narrative, hedges, background they have, alternatives they will not choose, detail below their altitude.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "So what does that mean for us?" | Activity reported where implication was needed |
| Reader asks something the document answers on line 30 | Conclusion not placed first |
| The update is skimmed and misunderstood | Written at the writer's altitude, not the reader's |
| Reader acts on a guess as though it were established | Uncertainty smoothed into declarative prose |
| Long explanation, and the follow-up questions arrive anyway | Answered the interesting question rather than the asked one |
| Every update is the same length regardless of the news | Length driven by effort spent, not by information carried |
| Reader keeps asking for "the actual numbers" | Claims made without evidence pointers |
| The text is full of service names the reader has never used | The system's vocabulary substituted for the reader's |
| Reader escalates something you already handled | The resolution was buried under how it was found |

## Red flags

- Starting with "So I started by…"
- Writing before you can name the reader
- A paragraph you would skip if the reader were standing next to you
- Hedging every sentence so that nothing in it can be wrong
- Explaining the mechanism when the question was about the risk
- A closing paragraph that restates the opening one
- A first line that restates the question instead of answering it
- Including the dead ends because they were the expensive part
