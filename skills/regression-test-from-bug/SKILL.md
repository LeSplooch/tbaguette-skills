---
name: regression-test-from-bug
description: Use when a defect has been reported, reproduced, or patched, when writing a test for a production incident or hotfix, when a bug that was fixed before has come back, or when deciding what to assert so the same defect cannot return. Covers choosing the layer, naming the test after the defect, scoping the assertion, and whether to keep the test once the fix lands.
---

# Regression test from a bug

## Overview

A test that has never failed for the reason it claims to guard is decoration. The defect is the specification: it names an assumption the code violated, and the test's job is to make that assumption cost nothing to hold forever.

## When to use

- A defect has been reproduced and is about to be fixed.
- A hotfix shipped without a test and needs one added behind it.
- A bug that was fixed before has reappeared, meaning the previous guard was absent, deleted, or scoped wrong.
- A production incident is being written up and needs a permanent guard, not an action item.
- Not for: turning a vague report into an on-demand failure — that is `reproducing-bugs`.
- Not for: finding the root cause — that is `diagnosing-before-fixing`. Write this test after the cause is known, or the test pins the symptom.

## The procedure

1. **Reproduce outside a test first**, at whatever layer is easiest. Until it reproduces, you have a hypothesis, not a bug.
2. **Write the test at the layer where you observed it**, even if that is slow and end-to-end. Confirm it fails — and read the failure message. A failure for a different reason (missing fixture, typo, wrong route) means the test is not yet about the bug.
3. **Then push it down.** Find the narrowest layer that still shows the failure, port the test there, confirm red again, and delete the wide version. Exception with an observable predicate: if the defect lives in the wiring between components — configuration, serialization across a boundary, dependency registration, ordering across services — no narrow test can see it, and the wide one stays.
4. **Fix.** Confirm green.
5. **Re-confirm the teeth.** Revert the fix (stash it), run the test, watch it go red, restore the fix. This is the highest-value step and the most-skipped one; it is the only proof that the test and the fix are actually connected.

## Scope the assertion to the invariant, not the trace

The stack trace tells you where the code noticed the problem. The invariant tells you what was actually untrue. Test the second.

| Defect | Too narrow | Right scope |
|---|---|---|
| Crash on an empty collection | "Does not raise a null-reference error" | An empty cart totals to zero and renders |
| Off-by-one on the second page | "Page 2 returns 10 items" | Every item appears exactly once across all pages, for a collection of size boundary+1 |
| Wrong total after a specific discount | "Order 4471 totals 89.10" | Total equals the sum of lines minus discounts, for the discount combinations that exist |
| Timestamp wrong for one user | "User in Denver sees 3pm" | Rendered local time round-trips to the stored instant across a DST transition |
| Duplicate charge on retry | "Second call returns an error" | Two identical requests with the same idempotency key produce exactly one charge |

Asserting the absence of a specific error type is the most common miss: the fix that swallows the error passes it, and so does the fix that returns the wrong answer quietly.

**Cover the class, not the instance.** Before closing, check whether the same mistake exists at sibling call sites — the same unguarded parse, the same unit conversion, the same missing lock. If there are N of them, parameterize the test over all N. Fixing one instance of a class and testing only that instance is why the same bug returns wearing a different ticket number.

## Naming and locating

- Name after the defect condition and the expected behavior, in the domain's words: `refund_of_partially_shipped_order_credits_shipped_lines_only`. Not `test_calculate_refund_3`, not `test_bug_4471`.
- Put the tracker id in an annotation, comment, or commit message — not the test name. Names must stay readable after the tracker is migrated or retired.
- File it with the behavior it protects, not in a `regressions/` folder. A quarantine directory for regression tests turns into a place nobody reads, and it separates the test from the code that would need updating with it.
- One test per defect. Bundling three fixes into one test means a single failure cannot tell you which regression returned.

## Keeping it afterwards

A good regression test looks redundant to everyone who did not debug the incident — that is exactly its value: it holds an assumption nobody currently believes is at risk.

- Delete it only when the **code path it covers is deleted**. Not when it looks similar to another test, not when it slows a suite, not when a refactor makes it awkward to compile.
- A refactor that would break the test is a refactor that may reintroduce the bug. Rewrite the test against the new structure and confirm it still fails against the pre-fix behavior; do not delete it to make the build compile.
- If the test breaks and the new behavior is deliberately different, change the assertion in a commit that says so. Silent edits to a regression test's expected value are how a fix is un-shipped.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Test written after the fix, passed on the first run | It was never proven to detect the defect; it may assert something unrelated |
| Same bug returns six months later | The test asserted the instance, not the invariant, and a different call path violated it |
| Test asserts "no exception thrown" | Scoped to the crash rather than the wrong result the crash was hiding |
| Test is slow and end-to-end for a pure logic bug | Step 3 skipped; it was left at the layer where it was first observed |
| Test deleted during a refactor | It sat in a separate folder with a name nobody could connect to a behavior |
| Test passes against both the buggy and fixed code | The setup does not reach the defective path — usually a default that avoids the edge case |
| A parallel bug ships in the sibling module | Only the reported instance was fixed; the class was never enumerated |
| Regression suite grows and nobody trusts it | Tests named after tickets; a red one carries no meaning without the tracker |

## Red flags

- "The fix is obvious, the test can come later."
- "I verified it manually, the test would just repeat that."
- "I'll write the test after the fix so I know what to assert."
- "This test is basically covered by the other one."
- "The refactor broke this old regression test, deleting it."
- Marking the incident resolved with a follow-up ticket for the test.
- A test whose expected value was copied out of the fixed code's actual output.
