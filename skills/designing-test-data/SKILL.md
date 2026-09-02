---
name: designing-test-data
description: Use when a test's setup is longer than its assertions, when fixtures are shared across files, when it is unclear which setup value causes a test to fail, when tests pass alone but fail in suite order, or when building factories, builders, object mothers, seed data, or bulk volume data for pagination and load tests. Also use when the fixtures for a rule that selects — a matcher, router, filter, alert condition, or suppression — were all written from the side that should match, or when such a rule fired on an input it should have ignored. Covers defaults, realistic versus minimal values, near-miss and negative corpora, and unicode and locale inputs.
---

# Designing test data

## Overview

The setup is the other half of a test's statement. If a reader cannot tell from the setup which value makes this test different from the one above it, the test communicates nothing and will not survive its author.

## When to use

- Setup is longer than the assertion, or repeats across files with small edits.
- A test fails and it is not obvious which of the twelve fields in the setup caused it.
- Tests pass in isolation and fail in suite order, or fail only under parallel execution.
- Seeding a store, queue, or file tree before exercising behavior.
- Needing hundreds or millions of records for pagination, sort stability, or load shape.
- Not for: the red-green-refactor loop itself, owned by `writing-the-failing-test-first`.
- Not for: a system taking on an entire new *category* of input for the first time — a new script, file format, or protocol version — where the risk lives in shared code no prior category ever exercised. Realistic values inside a category you already handle are this skill's job; auditing a category you don't yet handle is `auditing-new-input-categories`.

## The one-obvious-difference rule

Every value in a setup is either **irrelevant** — and must be a default, invisible — or **relevant** — and must be explicit, named, and adjacent to the assertion. Two tests that differ in intent differ in exactly one visible line.

- Setup that names more than 3 values the test never asserts on has the wrong defaults.
- Give the relevant value a name that states the reason: `expiredYesterday`, `maxLengthTitle`. A bare literal makes the reader reverse-engineer the intent.
- Assert only on fields the test set or the code derived. Asserting on an incidental default couples the test to the builder.

## Near-misses, not just examples

A rule that *selects* — a matcher, a router, a filter, an alert condition, a
suppression, a targeting expression — gets its fixtures written by whoever wants
it to work, and they are drawn, every one of them, from the side that should
match. Such a corpus can demonstrate that the rule fires. It cannot express the
rule firing when it should not, so the only failure it is capable of showing is
the one nobody was worried about.

The gap is not hypothetical and it is not exotic. Containment matching is the
common case: a short trigger sits inside a longer, unrelated word and the rule
fires on the accident. Every positive fixture passes. The defect arrives with
real input, which is also the first moment anyone looks at a non-matching case.

So the fixtures a selecting rule is missing are the **near-misses** — inputs that
contain the trigger, or sit one character from the boundary, and must not fire.
They are harder to write than positives for a structural reason: a positive is
derived from the intent you already hold, while a near-miss requires imagining
the thing you were not thinking of. Do not invent them. Take them from
recorded real input — a log, a transcript, a query history — where the accidents
are already present and cost nothing to find. The same record that measures how
often a proposed rule would fire (`automating-repetition`) is where its negative
corpus comes from.

Two are worth keeping permanently, and they work as a pair: one input that sits
just outside the boundary and must not match, and one that sits just inside it
and must. Either alone can be satisfied by moving the boundary. Together they pin
it, so loosening the pattern and tightening it both go red — which is the point,
because the fix for an over-firing rule is a tightening, and a tightening is how
the true positives get lost.

## Defaults are a design decision

- **Defaults must be valid and boring.** A default of empty string, zero, or null makes every unrelated test a secret edge-case test; the day that edge case is handled, dozens of unrelated tests go red.
- **Defaults must never satisfy the assertion by accident.** If the default status is `ACTIVE` and the test asserts `ACTIVE`, the test passes with the production logic deleted. Default to a value the code must change.
- **Recognizably fake, but legal.** Use reserved ranges: `example.com` / `example.org`, the `.invalid` and `.test` TLDs, `192.0.2.0/24` and `2001:db8::/32`, documentation-reserved phone and card numbers. This is what prevents a leaked default from reaching a real inbox or a live payment endpoint.
- **Uniqueness comes from a monotonic counter, never a random value.** Random uniqueness reproduces as a flake and produces unreadable diffs.

## Builders over shared fixtures

| Form | What it buys | Where it fails |
|---|---|---|
| Inline literal | Maximum locality | Duplicated and drifting; each copy learns a different truth about the schema |
| Shared fixture object or file | One place to change | Shared mutable state — the leading cause of order-dependent failures |
| Object mother (`anExpiredTrial()`) | Intent-revealing names | Grows to dozens of near-identical scenario names |
| Builder (defaults plus overrides) | Delta is visible; isolation by construction | Must be maintained alongside the schema |

Default to builders; layer 3–6 object-mother functions over the builder for canonical scenarios. The mechanical difference that matters: a builder returns a **new instance per call** and reads nothing global; a fixture returns a **reference**. That alone decides whether order dependence is possible.

Two further rules that most builders get wrong:

- Separate `build()` (in memory) from `create()`/`persist()` (written to a store). Conflating them drags I/O into tests that need none.
- Builders compose: building an order builds its customer unless one is supplied. A builder that demands a fully-formed graph is a fixture wearing a builder's name.

## Realistic over minimal

| Hazard | Minimal value that lies | Use instead |
|---|---|---|
| Encoding | `"abc"` | A 4-byte emoji, a combining-accent pair, an RTL name — separates bytes, code points, and graphemes |
| Length | `"test"` | Schema maximum, maximum+1, a 1-character value, and empty-but-legal |
| Case and locale | `"Alice"`, `"Bob"` | `Ä`, `ø`, `ı` — collation order and lowercasing differ by locale |
| Money and decimals | `10` | The currency's real minor-unit precision, a negative, and a pair like `0.1 + 0.2` |
| Time | A round timestamp | A leap day, a DST transition, an end-of-month, 23:30 in a `+05:45` zone |
| Identifiers | `"id1"` | The real format and length; plus a value that looks numeric but is not |
| Scale | 3 rows | One past every internal boundary: page size+1, batch size+1, buffer size+1 |

Use realistic values wherever the code touches encoding, length limits, locale, ordering, arithmetic precision, or a size boundary. Elsewhere, minimal is correct and cheaper.

The table above assumes the script, format, or locale itself is one the system already handles — an RTL name is a realistic *value* only once RTL is a category the code has ever run at all. The first RTL name, the first new file format, the first new plural rule: that is a different problem, not covered by picking a better value, and it is `auditing-new-input-categories`'s job.

## Volume without hand-writing it

- Generate from a **seeded** pseudorandom source and print the seed in the failure output. Unseeded generated volume is an unreproducible test. (This is bulk data with a shape you already know — a generator searching an input domain for a case that breaks a property is `property-based-testing`, a different problem that happens to share the seeding discipline.)
- Generate at **boundary+1**, not at a round number. If paging is 100, build 101 rows; 1,000 rows tests nothing more and costs ten times as much.
- For performance-shaped tests, **distribution beats count**: a long tail (a handful of records with 10,000 children, most with 0–2) exposes N+1 queries and missing indexes that a uniform 100-each dataset never will.
- Insert bulk data through the store's native path, not the domain layer, when the test is about read behavior — otherwise setup dominates runtime.
- Past ~1s of setup per test, move to one suite-scoped read-only dataset built once; tests that mutate create their own records instead.

All of the above assumes you own the shape you are building. For a double standing in for something you do not control, the governing question is where its content came from — see `grounding-test-doubles`.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Test fails only when run after another test | Shared fixture handed out by reference; one test mutates the next test's input |
| Changing one fixture field breaks 30 tests | Those tests assert on values they never set — the fixture is an unwritten specification |
| Nobody can tell what the test is about | Every field is explicit, so the relevant one is indistinguishable from the noise |
| Fixture has 40 fields and none can be removed | Kitchen-sink fixture; no test declares what it needs, so nothing can be proven unused |
| Test still passes with the production logic deleted | The default already equals the expected value |
| Suite gets slower every sprint | Every test seeds the whole world; setup cost tracks fixture size, not test needs |
| Test data reaches a real inbox or a live endpoint | Plausible-looking defaults instead of reserved-range ones |
| One test fails around the end of each month | Data derived from the current date crossing a month, quarter, or DST boundary |
| Generated-data failure cannot be reproduced | No seed recorded, or the seed was time-derived |

## Red flags

- "I'll just add one more field to the shared fixture."
- "The test needs the whole object graph anyway."
- "Random data finds more bugs" — said without recording a seed.
- Copying a fixture file and editing two fields.
- Loading a production dump as test data.
- An expected value computed by the same expression as the code under test.
- A setup block you scroll past to reach the assertion.
