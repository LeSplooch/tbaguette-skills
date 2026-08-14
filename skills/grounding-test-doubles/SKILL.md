---
name: grounding-test-doubles
description: Use when writing a mock, stub, fake, or fixture for something you do not control — an HTTP API, a vendor SDK, a queue, a device, another team's service — when a fully green suite is followed by a failure on the first real call, when deciding between recorded and hand-written fixtures, or when a parser and its test data were written by the same person from the same document. Covers fixture provenance, capturing over composing, contract and live tests, and making an unrecognized shape fail loudly.
---

# Grounding test doubles

## Overview

A double is exactly as correct as what its author believed while writing it. Composed from a specification, a vendor's reference implementation, or a careful reading of someone else's source, it inherits every misreading in that reading — invisibly. The suite built on it then agrees with itself forever.

**A green run against your own fixtures is positive evidence about the fixtures, not about the integration.**

## When to use

- Building any double for a system you do not control.
- The parser and the fixture it is tested against were written by the same person, from the same document, in the same sitting.
- A change is green locally and fails on the first real call.
- Choosing between a recorded payload and a hand-written one.
- Not for: which layer to test at (`choosing-test-scope`), constructing your own domain objects (`designing-test-data`), or faking time, randomness, and the filesystem (`testing-the-untestable`) — those you *do* control, and this problem does not arise.

## The one-party trap

A mock that both sides of an interface agree on is a shared belief rather than a check — a known problem with a known answer, which is a contract test.

The one-party version is worse, and it is the common one: **there is no second party available to disagree.** You read a vendor's documentation, you write a parser, you write a fixture, and both artifacts encode the same reading. Every test passes. The suite is now a very thorough proof that you are self-consistent.

Nothing inside the loop can break the tie. The correction has to come from outside it, which is what everything below is for.

## Capture over compose

Prefer fixtures **captured from a real response** over fixtures composed from a document.

One recorded payload per route, saved verbatim and committed, costs a single live call and is worth more than any amount of careful reading. Rules that make it hold up:

- **Save it verbatim.** Do not reformat, reorder keys, prune fields you consider irrelevant, or "clean up" the whitespace. The parts you would tidy are exactly the parts your reading got wrong.
- **Scrub secrets and personal data on the way in**, never later — a recorded fixture is a real response, with real tokens and real customer records in it. See `redacting-sensitive-output`.
- **Record the date and the provider version** next to the file. A fixture with no date cannot be judged stale.
- **Re-record on a schedule**, not only when something breaks. A fixture is a photograph, and providers move.

Composing by hand is legitimate for the cases you cannot capture: error responses the provider will not produce on demand, rate-limit bodies, malformed payloads. Mark those as composed so a future reader knows which fixtures are evidence and which are guesses.

## One live test per integration

A recorded fixture proves one shape at one moment. That leaves two gaps it structurally cannot close: shapes you never saw, and drift after the recording.

At least one live test per external integration earns its place — excluded from the default run, executed on a schedule or before a release, hitting the real provider with real credentials. It is the only thing that turns both a misreading and a provider's later change into a failing test instead of a support ticket.

Keep it honest about what it is: it needs credentials, it can fail for reasons unrelated to your code, and it must never gate an unrelated pull request. A live test that blocks CI gets deleted within a month, which is worse than not having one.

## Assume the shape varies where you have not looked

None of these are exotic, and all of them survive a suite whose fixtures were written by the same person who wrote the parser:

| Variation | What it looks like |
|---|---|
| Inconsistent envelopes | Wrapped once on most routes, twice on one |
| Type drift | A number as bare JSON here, a quoted string there |
| Absence spelled several ways | `null`, missing key, empty string, sentinel `0`, `"N/A"` |
| Booleans by three spellings | `true`, `"true"`, `1` — often across endpoints of one provider |
| Collections that collapse | An array of one arriving as a bare object |
| Pagination that changes at the edge | The last page omitting the cursor field rather than nulling it |
| Numeric precision | An integer that exceeds what the reader's number type holds |
| Errors as success | HTTP 200 with an error code in the body |

Every one of these is a real provider's behavior, and every one is invisible to a fixture set that has only ever seen the happy route.

## Make the mismatch loud

A boundary that yields an empty collection for a payload it failed to recognize is indistinguishable from a genuinely empty upstream, and **"zero rows" is the easiest wrong answer in the world to accept.** It looks like a quiet day. It propagates as a legitimate value. Nothing alerts.

An unrecognized shape must raise. The rule generalizes past parsing: when a component cannot tell "nothing to report" from "I did not understand the input", it should be built so those two produce different outcomes, and the second one should be the noisy one.

This is the single highest-value line item in this skill, because it converts every failure above — misreading, drift, a shape you never captured — from silent data loss into an exception with a stack trace.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Every mocked test passes, the first real call returns nothing | Fixtures composed from a reading; the parser shares its misreading |
| A provider changed and nothing failed for weeks | Only recorded fixtures; no live test to notice drift |
| "Zero results" accepted as normal for a month | Unrecognized shape returned empty instead of raising |
| A fixture was tidied and the test still passed | Tidying removed the exact irregularity the parser had to handle |
| Live tests deleted after they blocked a release | They were wired into the default run instead of a scheduled one |
| A recorded fixture leaked a real token into the repo | Scrubbed after committing, or not at all |
| Nobody can say whether a fixture is current | No date or provider version recorded next to it |

## Red flags

- "I wrote the fixture from the docs, so it matches."
- "All the tests pass" — as a claim about an integration.
- "The fixture had some weird formatting, so I cleaned it up."
- "It returns an empty list if it can't parse it, which is safer."
- "We don't need a live test, we have full coverage."
- A fixture file with no recorded date, version, or origin.
- The same person writing the parser and its only test data, from one document, with no capture step.
