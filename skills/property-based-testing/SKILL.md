---
name: property-based-testing
description: Use when example tests only cover the cases someone thought of, when testing parsers, serializers, encoders, sorting, comparators, arithmetic, caches, or data structures, when a bug arrives from input nobody anticipated, when replacing one implementation with another that must agree, or when reaching for fuzzing, generators, shrinking, invariants, or round-trip checks. Not for behavior specified only as a table of cases.
---

# Property-based testing

## Overview

An example test asserts what one input produces. A property asserts a relationship that holds for every input, then lets a generator hunt for the counterexample. The generator finds the inputs you would never have written down — which is exactly where the bugs are.

## When to use

- The specification is a **rule**, not a list: "parsing undoes printing", "the result is always sorted", "cost never decreases when items are added".
- Input has structure with a large or unbounded space: text, numbers, collections, trees, wire formats, dates.
- Replacing, porting, or optimizing an implementation that must agree with the old one.
- Handling untrusted or externally-supplied input, where "never crashes" is itself a requirement.
- Not for: behavior whose whole specification is a table of cases (tax brackets, status-code mappings, business rules enumerated by a stakeholder). Writing a property there just restates the table twice.
- Not for: the red-green-refactor cycle, owned by `writing-the-failing-test-first`.

## The property catalogue

| Property | Shape | Catches |
|---|---|---|
| Round-trip | `decode(encode(x)) == x` | Asymmetric escaping, lost precision, dropped optional fields, encoding-dependent truncation |
| Oracle / differential | `new(x) == old(x)` or `fast(x) == naive(x)` | Divergence during a rewrite, optimization, or port |
| Metamorphic | Relate two runs when no oracle exists: `search(a AND b) ⊆ search(a)`, `f(sorted(x)) == f(x)` | Bugs in code whose correct output nobody can state |
| Invariant preservation | Every operation leaves the structure valid: balanced, sorted, sums to the ledger total | State-machine corruption after long operation sequences |
| Idempotence | `f(f(x)) == f(x)` | Normalizers, migrations, deduplication, retried writes, upserts |
| Commutativity / associativity | Order or grouping does not change the result | Merge, CRDT, aggregation, and parallel-reduction bugs |
| Never-crashes | No unexpected failure mode for any input | Parsers on untrusted input — lowest value per property, highest value per line |

Round-trip alone cannot detect an encoder and decoder that are wrong in matching ways. Pair every round-trip property with one hand-written example asserting the actual external representation.

Differential testing against a legacy implementation freezes the legacy bugs as the specification. Use it to prove *no behavior changed*, then retire it once real specifications exist.

## Generators are the real work

The generator defines the input domain, so a biased generator is a silently narrowed specification.

- **Constrain by construction, not by filtering.** To generate a valid interval, generate a start and a positive length; do not generate two numbers and discard the unordered pairs. A rejection rate above ~10% both starves the run and skews the distribution toward whatever passes the filter easily.
- **Weight the boundaries up.** Random integers almost never produce 0, 1, −1, or the type maximum; random collections almost never produce empty or single-element. Good libraries bias toward these; verify yours does, and add them if not.
- **Generate the shapes that break invariants**: duplicates, already-sorted and reverse-sorted input, all-equal elements, deeply nested structures, and values at every internal size boundary.
- **Inspect the distribution before trusting the result.** Classify and print a histogram of generated cases once. Generators that produce 4-character strings 95% of the time are common and make the suite look green for free.

## Shrinking is why the technique pays

A failure on a 400-element list of random records teaches nothing. Shrinking reduces it to the two-element case that still fails, which is usually a readable bug report.

- Shrinking must respect the generator's constraints. A shrinker that produces inputs the domain forbids reports false counterexamples and wastes a debugging session.
- The framework must re-run the shrunk case and confirm it still fails. A "minimal counterexample" that passes on replay means the failure was nondeterministic — you found a flaky test, not a bug in the function.
- If shrunk output is still large, the generator built an opaque type the shrinker cannot decompose. Build inputs from primitives the library knows how to shrink rather than from a hand-rolled constructor.

## Working the failure

1. Record the seed and the shrunk counterexample from the failure output. A property failure without a printed seed is unactionable.
2. Reproduce by replaying the seed before changing any code.
3. **Promote the shrunk case to a permanent example test**, named after the defect. The property may not generate that case again for a thousand runs; the example runs every time in milliseconds.
4. Fix, then confirm both the example and the property pass.
5. Keep both. The example is the regression guard; the property is the search for the next one.

Run counts: the default (usually 100 cases) is a smoke test that catches gross errors. Run 10,000+ with a fresh random seed on a nightly or pre-release job, where a multi-minute run is affordable and a new seed is an asset rather than a flake.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Property passes for any implementation | The property restates the implementation; a wrong implementation would satisfy it too |
| Property never fails, then a bug ships | Generator never reaches the failing region — empty, huge, duplicate, or boundary values are absent |
| Failure cannot be reproduced | Seed not printed, or the generator reads the clock or an external source |
| Counterexample is enormous | Shrinking is absent, blocked by a custom type, or reduces toward an invalid value and gives up |
| Suite is slow and gets its case count lowered | Property does I/O or database work per case; properties belong on pure logic, integration on examples |
| Round-trip green, real consumers break | Both directions share the same wrong assumption; no external-format example test exists |
| Rewrite matches old behavior including its bugs | Oracle property treated as a specification instead of a change-detector |
| Property is skipped after intermittent failures | A genuine nondeterminism in the code under test was misfiled as generator noise |

## Red flags

- "The property is basically the function" — then it proves nothing.
- Adding a filter to the generator to make a failing property pass.
- Narrowing the generator's range after a failure, instead of fixing the code.
- Reducing the case count to make CI faster while keeping the property as evidence.
- Deleting the shrunk example test because "the property covers it".
- Treating a nondeterministic property failure as flakiness rather than a discovered race.
