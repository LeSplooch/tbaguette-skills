---
name: writing-the-failing-test-first
description: Use when implementing any new feature, fixing a bug, or changing existing behavior — before any production code gets written. Also use when the change is meant to alter a cost rather than a behavior — an optimization, a cache, a batch, a cheaper backend — so every behavioral test passes both before and after it and the loop looks inapplicable. Covers writing one failing test first, confirming it fails for the right reason, writing the minimal code that makes it pass, and refactoring only once the suite is green again.
---

# Writing the failing test first

## Overview

A test written after the code has never been seen to fail. It cannot prove it catches the bug it claims to guard against, because the only code it has ever run against is code already built to satisfy it. Red-green-refactor — the actual mechanism behind the word "TDD" — exists to keep that proof honest: watch the test fail before the code exists, watch it pass with the smallest code that could work, then clean up with that proof already in hand. Skip the order and the tests still run; they just stop proving anything.

## When to use

- Writing new behavior — a feature, endpoint, or function that doesn't exist yet.
- Fixing a bug: the failing test is the reproduction, and that same test proves the fix (see `regression-test-from-bug` for scoping the assertion and naming the test after the defect).
- Changing behavior on purpose, where the new behavior can be stated as a test before the change lands.
- Refactoring code that already has tests protecting it — the loop still governs any new test added along the way.
- Not for: code with no tests and no spec, where you don't yet know what it's supposed to do — pin what it actually does first (see `characterization-testing`).
- Not for: which test level catches a given kind of bug (see `choosing-test-scope`).
- Not for: what makes a generated or fixture value trustworthy (see `designing-test-data`, `property-based-testing`).
- This skill owns the loop — red, green, refactor, repeat — not test design. What a good test contains, which layer it belongs at, and what data goes into it belong to the skills above; this one only governs the order you write things in.

## No production code before a failing test

Catching yourself already mid-implementation — "just this once," "I'll backfill the test after," "this one's too simple to bother" — is the signal to stop and delete what you wrote. Not comment it out, not keep it open in another tab to "adapt" while writing the test: delete it. A test written while looking at a solution you already have is not a test-first test no matter what the file timestamps say — you already know what the answer looks like, and you'll write the assertion to match it instead of to specify it.

That includes the version of this thought that sounds like judgment rather than shortcut: "I already know what this should look like, so rewriting it from a test won't teach me anything new." It's true and beside the point — the loop was never about being surprised by your own solution. It's about watching a test fail for the reason you predicted, which is the only check that the test could have caught you being wrong. A reason for skipping the order that sounds compelling is exactly the moment to run the order anyway; if the reason is right, the red step confirms it in thirty seconds, and if it's wrong, that's the bug the whole discipline exists to catch before it ships.

The deadline that feels closest isn't always the one with the least slack. Deleting and redoing test-first can feel like trading a calm evening for a scramble tomorrow, but tomorrow morning usually holds more real working time before a 9am review than the last thirty minutes before dinner tonight do — and redoing a design you just spent hours arriving at is rarely a second full pass, since the hard calls are already made. Do the actual arithmetic on the time available before treating "later" as the scarier option; it usually isn't the one that's actually short on room.

Exceptions exist: throwaway prototyping, generated code, one-off configuration. Those are exceptions you name out loud and get agreement on before you start, not defaults you slide into because the test felt inconvenient this time.

## Red — write a test that fails for the right reason

Write one test for one behavior, name it after that behavior, and before you write its body, answer: what production change would make this fail? If you can't name one, you don't yet know what you're testing — go find the actual behavior first, rather than writing an assertion and hoping it lands on something real.

That question also rules out change detectors: a test that only fails when someone deliberately redesigns a constant, a private field, or exact wording is testing that nothing changed, not testing behavior. Test the outcome that depends on the decision — retried three times, rejected with this specific error, sorted this way — never the decision's literal value.

Run the test and read the failure. A useful failure tells you why: feature missing is correct, a typo or a broken import is not — fix the harness and run it again until it fails for the right reason. A test that passes immediately is testing something that already works; that's not a red step, that's a test in the wrong place.

Deriving the expected value by hand — not by calling the same helper the code under test calls — is what makes the failure trustworthy. How to build that value well is `designing-test-data` territory; the loop only requires that it existed before green did.

## When nothing behavioral can go red

Some changes are not meant to alter behavior at all. An optimization, a cache, a batched call, a switch to a cheaper backend — the whole point is that the answer stays the same and something else gets smaller. Write a test for one of those and it passes the moment it is written, against the old code and the new, and the natural conclusion is that the loop does not apply here.

That conclusion is wrong, and the diagnosis is one section up: a test that passes immediately is testing something that already works. The change did have a property that moved. It just was not the answer. **Assert the property the change existed to change** — elapsed time, queries issued, calls made, allocations, bytes read — and that assertion goes red before the change exactly like any other.

Skipping it is how a change ships having achieved nothing while its suite stays green. A slow lookup replaced by a fast path returned the right value and passed; an assertion on elapsed time failed immediately, because the fast path internally called a helper that still went through the rate limiter the change existed to escape. Nothing in the diff of the new path looked slow — the cost was reached one call further down, in code the change never touched. Only an assertion on the cost could tell a change that worked from one that did not.

Two things make such an assertion usable rather than flaky. Give it a **wide threshold**: it exists to catch a regression to the old order of magnitude, not to police a 5% drift. And make it exercise the **state the cost is actually paid in** — for anything cached, pooled, or lazily built, that is the cold one, and a test that reuses a warm fixture measures the half of the behavior that was never in question. `performance-profiling` owns choosing the number and defending it against noise; the loop only requires that something was red first.

## An assertion over the result cannot see what the result is missing

Deriving the expected value by hand covers the scalar case. The collection case slips past it, because the assertion looks like it is checking everything: *for each item in the result, assert it is well-formed.* Every item passes. The test is green.

It is green because it quantifies over a set the code under test chose. If the code dropped half its input, the survivors are all still well-formed. If it dropped all of it, the loop body never runs and the assertion is vacuously true — so **the test is at its most confident exactly when the code has failed hardest**, which is the reverse of what a test is for. Lossy-by-design code makes this routine rather than exotic: a parser that skips entries it cannot read, pagination, deduplication, permission scoping, any filter with a sensible default of "leave it out."

The fix is to get one number from outside the unit. Count the raw inputs, or list their keys, from the source the code read rather than from what it returned, and assert against that:

```
vacuous at zero, silent about everything dropped:
    for each item in load_all():  assert item is well-formed

fails the moment anything is dropped, including all of it:
    assert count(load_all()) == count(files in the source directory)
```

When dropping is deliberate — the loader is *supposed* to skip malformed entries — the count assertion still works, but the expected number has to be stated rather than derived: `assert count(loaded) == count(source) - 2`, with the two known-bad fixtures named. That is the point, not an inconvenience. An intended loss written down is a specification; an intended loss left implicit is indistinguishable from the unintended kind, which is how the whole failure started.

Any assertion whose expected value was produced by the code under test is self-referential, and universally-quantified ones are the dangerous shape, because emptiness makes them true rather than false. "Mutate it before you trust it" below is the check that catches it: delete the loader's body and see whether anything goes red. Nothing will.

## Green — the smallest code that passes

Write just enough production code to pass the one test in front of you, not the feature you can see coming. A config flag nobody's test asked for, a caching layer nobody's test exercises, a drive-by refactor of the neighboring function — all of that is scope the test hasn't earned yet. It arrives later, with its own test, when its own red step demands it.

Keep every collaborator the test touches real. Reach for a double only where something is genuinely slow or external, and once you do, don't assert on the double itself — assert on what your code does with what it returned. A test that checks a mock was called proves the mock exists, not that your code works; if a double is what you're tempted to assert on, unmock it or delete the assertion. `grounding-test-doubles` covers grounding a double against a system you don't control; `choosing-test-scope` covers the point where a double has grown enough conditional logic to be a second, untested implementation of the real thing.

Run the whole suite, not just the new test — a green new test next to a red old one is not green. Pristine output counts too: a warning everyone's learned to scroll past is exactly where the next real failure will hide.

If passing requires mocking everything in sight, that isn't a fact about the test. It's the design telling you it's too coupled to construct — simplify the interface or wire the dependency in properly, rather than reaching around it with more test scaffolding.

## Refactor — clean up with the net already there

Only once the suite is green. Remove duplication, rename for clarity, extract a helper — structure changes, behavior doesn't. Wanting to change what a test asserts mid-refactor is a stop sign: either the test was pinned to an internal detail instead of a behavior, or this "cleanup" just changed behavior, which makes it a new red step wearing a refactor's name.

The same discipline covers the test-only code the loop tends to leave behind. A `reset()` or `destroy()` that exists only so a test can tear something down doesn't belong on the production class — it belongs in a test utility. A production class carrying methods only tests ever call has grown a second interface that nothing but the test suite uses.

Run the suite after each structural change, not once at the end — the gap between two green states is how much code you'd have to search when one goes red. Then pick the next behavior and start red again, one test at a time; batch several behaviors into one red step and the next failure won't name a single cause.

## Mutate it before you trust it

Before calling a piece of behavior finished, mutate the code you just wrote — flip a comparison, swap a branch, delete a validation — and confirm some test goes red for each mutation you'd realistically make. A suite that stays green through a real mutation didn't prove what you thought it proved: the behavior was never actually covered, or the test was tautological from the start. `characterization-testing` runs this identical check against a pinned legacy behavior instead of a new one — same technique, different target.

A mutation only tells you something if the result *changed*. Red after the
mutation is half an observation; the claim is that the test is red **because
of** the mutation, and that needs the green reading beside it. A check that was
already failing — a broken selector, a fixture that no longer loads, an
assertion against a value it can never see — goes red under every mutation you
make and looks exactly like a check that works. The pair is the evidence, never
the single reading, and the question that separates them is the same one a
retry has to answer: **what differed between those two runs?** If the honest
answer is "nothing, it was red both times", nothing was learned and the check
itself is now the thing to go and read.

That check is the actual definition of done, not "coverage went up." Ship the tests the behavior needs, in the order the loop produced them, and stop. A test added afterward to satisfy a process rather than a red step is maintenance cost with no proof behind it.

## When you get stuck

| Stuck on | What it's telling you |
|---|---|
| Don't know how to write the test | Write the call you wish existed — the assertion first, then whatever setup makes it compile. The test is the API's first real consumer; if you can't call it the way you want yet, that's the design decision to make now, not during green. |
| Test setup is huge before you even reach an assertion | Not a loop problem — see `designing-test-data`. |
| Test depends on time, randomness, the filesystem, or the network | Not a loop problem either — see `testing-the-untestable` for making the dependency fake-able. |

## Common mistakes

| Symptom | Real cause |
|---|---|
| Test passes the moment it's written | Written after the code, or asserts a value computed by the same logic it's testing — proves nothing either way |
| Can't say what production change would break a passing test | The test was never watched failing; it may be asserting on a constant or an implementation detail instead of behavior |
| "I'll write the test right after, same result" | Tests-after answer "what does this do"; tests-first answer "what should it do" — the case you forgot never gets asked |
| Implementation grows a feature its test never asked for | Green step scope-crept into design instead of stopping at "this one test passes" |
| A teardown or reset method exists only for tests to call | Test-only cleanup leaked onto the production class instead of into a test utility |
| Refactor step quietly changes an assertion | Refactor is being used to smuggle in a second, unproven change |
| Whole suite re-run "to be safe" instead of reading the one failure | The failure message was never actually read closely enough to say why it failed |
| Bug fixed with no test written first | The only evidence the fix works is manual checking, which leaves nothing to catch the regression |
| A change made for speed shipped with only correctness tests | Every behavioral assertion passed before the change too; the property that moved was a cost, and nothing asserted it |
| A suite over a collection stays green while the collection is empty | Every assertion quantified over the result the code produced; nothing counted the input |
| Code kept and tests backfilled because "I already know this cold" | Confidence in the solution stood in for proof a test could catch it being wrong |

## Red flags

- "Too simple to bother testing."
- "I'll add the test right after — same result."
- "Already tested it by hand, it works."
- "I'll keep this implementation open in another tab while I write the test."
- "Already spent three hours on this; deleting it now would be wasteful."
- "It's about the spirit of TDD, not the ritual."
- "This case is different, the rule doesn't really fit here."
- "I already know what this should look like — rewriting it teaches me nothing."
- "The suite's green" — said about a test nobody watched fail first.
- Every assertion in a test iterates something the code under test returned, and no number in the test came from outside it.
