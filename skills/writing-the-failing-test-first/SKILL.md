---
name: writing-the-failing-test-first
description: Use when implementing any new feature, fixing a bug, or changing existing behavior — before any production code gets written. Covers writing one failing test first, confirming it fails for the right reason, writing the minimal code that makes it pass, and refactoring only once the suite is green again.
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

Exceptions exist: throwaway prototyping, generated code, one-off configuration. Those are exceptions you name out loud and get agreement on before you start, not defaults you slide into because the test felt inconvenient this time.

## Red — write a test that fails for the right reason

Write one test for one behavior, name it after that behavior, and before you write its body, answer: what production change would make this fail? If you can't name one, you don't yet know what you're testing — go find the actual behavior first, rather than writing an assertion and hoping it lands on something real.

That question also rules out change detectors: a test that only fails when someone deliberately redesigns a constant, a private field, or exact wording is testing that nothing changed, not testing behavior. Test the outcome that depends on the decision — retried three times, rejected with this specific error, sorted this way — never the decision's literal value.

Run the test and read the failure. A useful failure tells you why: feature missing is correct, a typo or a broken import is not — fix the harness and run it again until it fails for the right reason. A test that passes immediately is testing something that already works; that's not a red step, that's a test in the wrong place.

Deriving the expected value by hand — not by calling the same helper the code under test calls — is what makes the failure trustworthy. How to build that value well is `designing-test-data` territory; the loop only requires that it existed before green did.

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
