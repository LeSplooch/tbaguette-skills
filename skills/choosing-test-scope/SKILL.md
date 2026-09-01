---
name: choosing-test-scope
description: Use when deciding whether a behavior belongs in a unit, integration, contract, or end-to-end test, when the end-to-end suite is slow or nobody trusts it, when a bug escaped every layer of tests, when mocks are re-implementing a real dependency, when fixtures were written from a specification rather than captured from a real response, when tests break on refactors that changed no behavior, or when arguing about the testing pyramid, coverage targets, and test ratios.
---

# Choosing test scope

## Overview

Test each behavior at the highest layer that can fail for exactly one reason. Higher than that and a red test names no cause; lower than that and a green test proves nothing about the shipped system.

## When to use

- Adding coverage and unsure whether it belongs in a unit, integration, contract, or end-to-end test.
- The end-to-end suite takes long enough that people rerun it instead of reading it.
- A defect reached production while every layer of tests was green.
- A double has grown enough conditional logic that it is a second implementation of a dependency.
- Tests break during a refactor that changed no observable behavior.
- Someone is proposing a coverage percentage or a pyramid ratio as a target.
- Not for: how to make a chosen layer deterministic — that is `testing-the-untestable`.

## What each layer alone can catch

| Layer | Only it catches | Typical cost | When red, it points at | Survives internal refactor |
|---|---|---|---|---|
| Unit (in process, no I/O) | Branch logic, edge cases, arithmetic, error paths hard to trigger from outside | <10ms | One function | Only if it tests the module's boundary, not its internals |
| Integration (your code plus one real dependency) | Query dialect, migrations, transaction and isolation behavior, serialization, driver semantics, index and collation effects | 10ms–2s | One seam | Yes |
| Contract (consumer expectations verified against the provider) | Schema drift and breaking changes between independently deployed parties | <1s per pair | One party | Yes |
| End-to-end (whole stack, real transport) | Wiring, configuration, authentication, deployment topology, cross-service ordering | 5s–minutes | Nothing specific — diagnosis is the dominant cost | Yes |
| Manual and exploratory | Qualities nobody can specify: comprehensibility, aesthetics, surprise | Human time | A conversation | n/a |

The "cost" column is the smallest part of the real cost. Maintenance and diagnosis time dominate: an end-to-end failure typically consumes a person's afternoon before anyone knows which component moved, and that cost recurs on every flake.

## Applying the one-reason rule

- **Too high** if a red test has more than one plausible cause. Split it or move it down until a failure names a component.
- **Too low** if the test could stay green while the shipped system is broken. Every behavior needs at least one test that exercises real wiring, even if only one per deployable.
- **Match the layer to where the behavior is decided.** If uniqueness is enforced by a database constraint, a unit test asserting uniqueness is a fiction about code that does not enforce it. If it is decided in the domain, an integration test for it is slow and redundant.
- Pick the layer per behavior, not per file. One class routinely deserves fast unit tests for its branching and one integration test for the query it emits.

## Contracts beat integration combinatorics

Cross-testing every consumer against every provider grows as the product of both. Six services that all call each other are 30 directed integration paths; the same coverage as contracts is 6 consumer expectation suites plus 6 provider verifications — 12 runs, each of which can execute without the other side deployed.

- Use contract tests when the two sides **deploy independently**. That independence is the whole reason schema drift can happen.
- Skip them when both sides ship as one unit — an integration test is cheaper and catches the same thing.
- The contract must be verified against the **real provider**, not against a shared mock. A mock both sides agree on is a shared belief, not a check, and it drifts with neither side noticing.

## Test the boundary you own

Do not test that the HTTP client sets headers, that the ORM emits valid SQL, or that the framework parses configuration. Those have their own suites, and duplicating them makes upgrades harder without making the system safer.

Do test:

- **Your adapter's translation** of the library's errors, timeouts, and edge results into your domain's vocabulary. Nobody else tests this, and it is wrong more often than any other code in the boundary.
- **One pinned assumption per behavior you would want to hear about on a version bump**: that this query returns rows in this order, that this call retries, that this serializer omits nulls. These are the tests that make a dependency upgrade a five-minute job instead of a week.
- **Configuration you wrote**, at the layer that loads it.

## Where a double's content comes from

The two-party version is above: a mock both sides agree on is a shared belief, not a check. The one-party version is worse, since no second party exists to disagree — a double and the parser that reads it, written from one document by one person, encode the same misreading and pass forever.

`grounding-test-doubles` covers fixture provenance and the live test that breaks the tie.

## Budgets, and the end-to-end death spiral

| Suite | Budget | What breaks past it |
|---|---|---|
| Unit | Whole suite under 60s | Stops running on save; feedback moves to CI |
| Integration | Under 5 minutes | Stops running before push |
| End-to-end | Under 10–15 minutes, 10–30 scenarios | Nobody reads failures; reruns replace diagnosis |

Every end-to-end scenario must be a user-visible journey no other layer can cover — sign in and complete a purchase, not "the validation message reads correctly". The spiral is mechanical: scenarios accumulate, runtime grows, flakes appear, reruns become routine, real failures get rerun away, and the suite is now a tax that catches nothing. It is cheaper to delete two thirds of it than to maintain the version nobody trusts.

Ratios are an output, not a target. A codebase that is mostly glue is legitimately integration-heavy; a codebase with a rich domain model is legitimately unit-heavy. Coverage percentage is a floor detector — it can prove a module has none — and stops being informative as a target above roughly 80%, where teams start writing tests without assertions to reach the number.

## Code whose job is to prevent something is tested at its call path

The table puts wiring in the end-to-end row, and for ordinary code that is unremarkable: an unwired feature does nothing, somebody notices within the day, because the thing they asked for is visibly missing.

Guard code inverts it. A redaction step, a permission check, an input validator, a rate limiter — when one of those is not wired in, nothing is missing. The output still appears, the request still completes, and the check simply never ran. A thorough unit suite over the function goes on passing, and it never could have said otherwise: "is anything calling this" is a question about the caller, and no quantity of coverage of the callee answers a question about its callers.

The failure also tends to err *safe*, which removes the last mechanism that would have surfaced it. A stripping step that runs on an empty input strips nothing and lets nothing through either; a filter that is never consulted, in a pipeline that defaults to rejecting, rejects. Nobody is harmed, so nobody reports it, and the documentation describing the check keeps being true about the code and false about the system. **A safety property that is accidentally too strong produces no symptom at all** — only the too-weak direction ever complains, which is why an audit that checks one direction finds nothing and concludes the right thing is happening.

Two consequences worth making explicit:

- The test that carries the weight feeds a real input the guard should act on through the real call path, and asserts on what the pipeline actually emitted. The unit tests stay; they stop being the evidence.
- When auditing a safety property, deliberately check whether it is too strong as well as whether it is too weak. Both are defects. Only one of them will ever come to find you.

## When a bug escapes every layer

For each defect that reached production, ask one question: **what is the lowest layer that could have caught this?** Add the test there, and only there.

If the honest answer is "only end-to-end", the behavior exists solely in the wiring, and that is a design finding rather than a testing one — extract a seam so the decision lives somewhere addressable. Answering the question with "add another end-to-end scenario" every time is how the end-to-end suite reaches the size at which it stops working.

## An accumulator is checked against a second derivation

Sometimes the honest answer is "no layer", because the escape was never about layers. A systematic offset — a per-unit figure that omits a fee the total includes, a rate applied at the wrong precision throughout — makes every individual number wrong in the same way and every individual number plausible, and anything that accumulates carries that error into every contribution it adds up. A test checking one of those numbers passes whenever the expectation holds the same mistake, and the expectation usually does, because it came from the same reading of the same specification by the same person. This is the one-party problem from "Where a double's content comes from" arriving as arithmetic rather than as a fixture: one belief, written down twice, agreeing with itself.

Adding more point-checks does not escape that. A hundred of them agree a hundred times, at whichever layer they run. What breaks the tie is a quantity computed by a **different route** and asserted against the first: a running total accumulated from individual events against the same figure derived from opening and closing balances; inventory on hand against receipts minus shipments; a queue's processed count against enqueued minus remaining. Neither route is the authority. The assertion is that they agree, and a disagreement names a defect without yet saying which side holds it.

This is cheap and it is normally one test per accumulator — money, inventory, quotas, counters, capacity, anything that adds up over time. The layer follows the one-reason rule as usual, with a condition on top of it: the two routes have to be genuinely independent, so a layer at which both of them run through the function that holds the error buys nothing. That test agrees with itself for the same reason the point-checks did.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Green suite, broken deployment | Everything tested with doubles; no layer exercised real wiring or configuration |
| Every refactor breaks dozens of unit tests | Tests written against internals rather than the module boundary |
| A double grew conditionals and its own bugs | A real dependency was needed; the double is now an unversioned second implementation |
| End-to-end failure takes an afternoon to attribute | Behavior tested above the layer that decides it |
| Contract tests pass, integration breaks | Contracts verified against a shared mock instead of the real provider |
| Every mocked test passes; the first real call returns nothing | Fixtures were written from a reading of the provider, so the suite tests the reading |
| A shape change surfaces as an empty result rather than an error | The boundary parses leniently; an unrecognized payload must raise, not yield nothing |
| Coverage is 90% and bugs still escape | Coverage measures execution, not assertion; branches are run, not checked |
| Every figure in a report is individually plausible and the total is wrong | Each expectation came from the same understanding of the rules the code did; nothing computed the answer by a second route |
| A well-tested guard function that nothing calls | Coverage of the callee cannot answer a question about its callers, and a guard that never runs produces no symptom to notice |
| Integration suite has thousands of cases | Business-rule permutations tested through the database instead of in the domain |
| Dependency upgrade needs a week of manual testing | No pinned-assumption tests; the boundary's behavior was never written down |
| The team reruns CI as a first response | A suite past its wall-clock budget; rerunning is now cheaper than reading |

## Red flags

- "We'll catch it in end-to-end."
- "Add a browser test for that validation rule."
- "Mock the database, it's faster" — for a behavior the database decides.
- "We need 100% coverage."
- "Every line item has a test" — said about a total that nothing recomputes independently.
- "The redaction/permission/validation function is thoroughly tested" — said without having checked that the pipeline calls it.
- "The pyramid says we need more unit tests" — stated with no failing behavior in mind.
- Writing a test at a given layer because the harness there was already set up.
- "I built the mocks from their reference implementation, so they're accurate."
- A fixture never once compared against a real response.
- An end-to-end scenario added for every escaped bug, regardless of cause.
