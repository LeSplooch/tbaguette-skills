---
name: reading-specifications
description: Use when turning a ticket, issue, bug report, RFC, standard, design doc, or a user's request into work; when requirements are vague, contradictory, or silent on edge cases; when unsure whether to ask a clarifying question or proceed on a stated assumption; when interpreting MUST, SHOULD, and MAY; or when acceptance criteria have to be written before any code is built.
---

# Reading specifications

## Overview

Every specification has three layers: what it states, what it implies, and what it assumed without noticing. The stated layer is the smallest and causes the least rework. Almost all disagreement surfaces in the layer nobody wrote down, and the only way to reach it is to enumerate the silence in writing before building.

## When to use

- A ticket, issue, or request arrives and implementation is about to start.
- Two readings of the same sentence produce different behavior.
- Implementing against a standard, protocol, or third-party contract.
- An estimate is being given for prose nobody has decomposed.
- Not for: an interface you are defining rather than consuming (designing-apis), or deciding where the change lands once the requirement is settled (finding-the-seam).
- Not for: settling *what to build* in the first place, before anything is written down — that's `scoping-before-building`, a live conversation this skill's own "ask or assume" table assumes has already happened and produced the document being read here.

## Three layers

| Layer | How it appears | Rework risk |
|---|---|---|
| Stated | Written in the text | Low — but check for contradiction between sections; two authors, two paragraphs |
| Implied | Follows necessarily ("export to CSV" implies escaping, encoding, a filename, a row limit) | Medium — you will build something, possibly not theirs |
| Assumed | The author never registered it as a question: which timezone, which sort, what the second click does | Highest — the entire source of "that isn't what I meant" |

Spend effort in inverse proportion to how much of it is written. Surface the assumed layer as a written list, because the author cannot recognize their own unstated assumption until they see it in someone else's words.

## Keyword discipline

| Word | Binds you to | Test implication |
|---|---|---|
| MUST, SHALL, REQUIRED | Conformance; violating it makes the implementation wrong, not merely different | A failing test blocks release |
| MUST NOT, SHALL NOT | Absolute prohibition | A test asserts the absence |
| SHOULD, RECOMMENDED | Deviating requires understanding the full implications and weighing them | Deviation allowed, documented at the deviation site |
| MAY, OPTIONAL | Implementations legitimately differ | A test that the *other* choice does not break you |

The keywords bind the implementer, not the input. Two symmetric failures: treating SHOULD as MUST produces gold plating and rejects valid peers; treating SHOULD as MAY ships a defect and calls it a choice.

**Unmarked prose flips meaning by document type, and this is where most misreadings start.** In a formal standard, unmarked prose is descriptive — background, rationale, restatement — and imposes nothing; the normative content lives only in keyworded sentences, grammars, and tables. In a ticket, an issue, or a message from a person, the opposite holds: every sentence is a requirement, MUST is implied throughout, and the words "just", "simply", "also", "obviously", and "quick" mark the sentences hiding the most work. Decide which kind of document you are holding before reading a line of it.

## Finding the silence

Run this list against any requirement. Seven of the ten are typically unanswered, and each is where a disagreement will surface later at full cost.

- **Empty, one, many.** Behavior at zero items and at ten million.
- **The second time.** Idempotency, double submit, retry, replay, resume.
- **Concurrent.** Two actors doing it simultaneously; last-write-wins or conflict.
- **Failure.** Partial success, timeout, downstream unavailable — fail-open or fail-closed. Nearly always unstated, and the author nearly always holds a firm implicit answer.
- **Ordering.** Specified, incidental, or guaranteed.
- **Time.** Timezone, clock skew, expiry, what "day" and "month" mean at boundaries.
- **Identity.** What makes two of these the same thing; what happens to duplicates.
- **Authorization.** Who may; what the answer looks like when they may not.
- **Existing data.** What happens to records already stored in the old shape.
- **Reversal.** Whether it can be undone, by whom, and how long after.

Ten minutes answering these in writing is the highest-leverage act in the whole task.

## Rewrite as testable statements

Convert each requirement to `Given <state>, when <event>, then <observable>`. Anything that will not fit that form is not a requirement, and needs relabeling:

| Not a requirement | What it is | What to do |
|---|---|---|
| "Make it fast" | A goal | Attach a number. "p95 under 200 ms at 500 requests/second" is testable; "fast" is a wish |
| "Must run on existing hardware" | A constraint | A design boundary, checked at design time, not in the suite |
| "Follow the existing pattern" | A preference | Resolve now; otherwise it is re-litigated in review |
| "Improve the experience" | A direction | Ask what observable would change, or drop it from scope |

Then count the statements. A "small" ticket that produces 14 testable statements has a wrong estimate, and that is the first thing to report — before any code, and separately from any question.

## Ask or assume

| Condition | Action |
|---|---|
| The answer changes the data model, a public interface, or is expensive to reverse | Ask. Block on it |
| Two readings are both plausible and produce different observable behavior | Ask, with both readings written out and a recommendation |
| Cheap to change later and one reading is clearly likelier | State the assumption in the change description and proceed |
| Guessing a number: limit, timeout, page size, retry count | Proceed with a named constant and a comment; the reviewer answers it in place |
| Answering requires them to check with someone else or run something | Ask now — latency is the cost and it only grows |

Batch the questions — this is asynchronous clarification against a spec or ticket author, where round-trip latency is the cost being minimized, not a live conversation. One message with five numbered questions gets answered; five messages get one answer. Include your best guess for each so the reply can be "yes to all but 3". (`scoping-before-building` asks one at a time instead, because that skill is a real-time back-and-forth where each answer can change what's worth asking next — batching there would just stall a conversation that's already live.) Never ask a question answerable by reading the code, the tests, the history, or by running the thing — "how does the current one behave" is not a question for a human.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Built the wrong thing from a clear ticket | Implemented the stated layer; never enumerated the assumed one |
| Endless clarification round-trips | One question at a time, with no proposed answers attached |
| Blocked for a day on a detail that did not matter | Asked where a documented assumption would have done |
| Reviewer says "obviously it should X" | X lived in the silence; the checklist was never run |
| Implemented a SHOULD as a hard rejection | Treated non-normative guidance as normative; broke interoperability |
| Estimate off by 4x | Never counted the testable statements |
| Two sections of the spec implemented inconsistently | Contradiction in the stated layer, never reconciled because the doc was read once, linearly |

## Red flags

- "It's obvious what they mean."
- "I'll work out the edge cases while coding."
- "The ticket says 'simply', so it's small."
- "I'll ask once I've started" — the question gets cheaper to ask and more expensive to answer with every hour.
- Writing code before a single `given/when/then` line exists.
