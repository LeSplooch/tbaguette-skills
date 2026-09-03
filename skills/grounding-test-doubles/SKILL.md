---
name: grounding-test-doubles
description: Use when writing a mock, stub, fake, or fixture for something you do not control — an HTTP API, a vendor SDK, a queue, a device, another team's service — when a fully green suite is followed by a failure on the first real call, when deciding between recorded and hand-written fixtures, when a parser and its test data were written by the same person from the same document, or when a benchmark's fixture describes a richer configuration than the shipping system actually produces. Covers fixture provenance, capturing over composing, contract and live tests, and making an unrecognized shape fail loudly.
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

## Your own system is also something you do not control

The discipline above gets applied to things that belong to somebody else, and lapses for fixtures modelling your own configuration — capability flags, feature toggles, the environment description a run is handed — because those feel knowable in a way a vendor's payload does not. They are not. Your production code path is edited by other people on other days for other reasons, and a fixture hears about none of it.

Where this does real damage is a fixture belonging to a **benchmark**, a scoring run, or an evaluation suite — anything whose numbers get quoted. An ordinary stale fixture eventually produces a red test, which is self-correcting. A stale fixture in a benchmark produces a *number*, and the number gets repeated in decisions long after anyone remembers what it was measured against. Nothing goes red, because the suite's job is to check the code against the fixture, and the fixture against reality is the one claim nothing in the suite has ever verified.

The asymmetry that hides it: the divergence is usually the fixture being *richer* than production, not poorer. A fixture gets written with everything populated, because populated is what its author was demonstrating, while production leaves half of it empty because the code that would fill it was never built. Both look correct in isolation, and the fixture reads as the more thorough of the two.

- **Diff the fixture against what the production path actually constructs**, field by field. This is *capture over compose* from the section above, turned inward, and the capture is cheap: call the real constructor and compare. Do it when the number is about to be quoted somewhere that matters — a decision, a report, a comparison against a previous run — rather than when something already looks wrong.
- **Have the benchmark build its inputs from the same function production uses**, and make the richer variant a flag. A richer configuration is legitimate to measure, and often it is what the system is being built toward — but it should be asked for by name rather than arrived at by inheritance.
- **Say which configuration a number was measured under, wherever the number is written down.** A score with no configuration beside it cannot later be told apart from one measured against the shipping system, and it will not be.

When the diff comes back, do not assume the fixture is the wrong half. A fixture that describes a capability production no longer has is also how a silent regression announces itself, months late and from an unexpected direction — the fixture was right when it was written and nothing failed when production stopped matching it.

A number measured against a configuration the system never produces is not conservative and not optimistic. It is unrelated, and it does not announce itself as unrelated.

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
