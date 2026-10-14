---
name: property-based-testing
description: Use when example tests only cover the cases someone thought of, when testing parsers, serializers, encoders, sorting, comparators, arithmetic, caches, or data structures, when a bug arrives from input nobody anticipated, when replacing one implementation with another that must agree, when a rule must hold for every member of a closed catalogue the code already lists — themes, locales, roles, enum values — or when reaching for fuzzing, generators, shrinking, invariants, or round-trip checks. Not for behavior specified only as a table of cases.
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
- Not for: authoring setup data whose shape you already fully know — `designing-test-data` covers builders, defaults, and volume for that. Both reach for a seeded random source, but this skill's generator defines a search domain hunting for a counterexample; that skill's generated data is bulk volume with a known shape.

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

## When the domain is a closed catalogue, enumerate it

A generator earns its keep on a space too large to walk. Some properties range over a space that is small and already written down: the themes an app ships, the locales it translates, the roles, permission sets, enum members, supported file formats, registered routes. Sampling that list at random is strictly worse than reading all of it. A hundred random draws can miss the one bad member, and a run that passes by luck looks the same as one that passes by construction.

- **Iterate the list the code itself exposes**, not a copy of it in the test. Then a member added next month is checked the day it lands, by a test nobody edited. A hand-typed list in the test is the second copy that goes stale.
- **Cross it with the other dimension** the property depends on (every theme under each base theme, every locale at each text length), and state the property once. This is still a property, a rule over a domain, not the table of cases excluded above: the catalogue supplies the inputs, never the expected outputs.
- **Assert the catalogue is non-empty, and at least as large as it is known to be.** A loop over an empty or filtered-away list passes for any implementation.
- **Make a failure name its member**, so the report is "variant 31 under the dark base fails the threshold", not an index into a cross product.
- **Prefer a measured bound to equality** when the property is a quality, such as contrast, size or latency: assert the number clears a threshold, and print the number.

Then pin the case that was reported as a number. Reconstruct the original failing input, measure it the way the property measures, and assert the measurement was bad under the old rule. That guard is the proof the property can fail, and without it a green sweep says nothing about whether the sweep could ever have caught the report. `regression-test-from-bug`'s step 5 (revert the fix, watch the test go red) proves the same thing once, by hand; the pinned number is the form of that proof that stays in the suite, which matters for a sweep, because one that cannot fail on the known bad case is silently measuring the wrong thing.

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
| A sweep over a closed list stays green after the list grows | The test carries its own copy of the catalogue, or samples it instead of enumerating the one the code exposes |
| Property is skipped after intermittent failures | A genuine nondeterminism in the code under test was misfiled as generator noise |

## Red flags

- "The property is basically the function" — then it proves nothing.
- Adding a filter to the generator to make a failing property pass.
- Narrowing the generator's range after a failure, instead of fixing the code.
- Reducing the case count to make CI faster while keeping the property as evidence.
- Deleting the shrunk example test because "the property covers it".
- Sampling a closed catalogue at random, or typing its members into the test, when the code already lists them
- Treating a nondeterministic property failure as flakiness rather than a discovered race.
