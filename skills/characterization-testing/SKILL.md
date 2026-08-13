---
name: characterization-testing
description: Use when changing, refactoring, extracting, porting, or deleting code that has no tests, when the current behavior is unknown or undocumented, when a rewrite must match a legacy component output for output, or when reaching for golden files, approval tests, snapshots, or output diffs. Covers pinning existing behavior, recording known bugs, and retiring the pins.
---

# Characterization testing

## Overview

A characterization test asserts what the code *does*, not what it should do. It is a scaffold that makes a refactor observable, and it is the only kind of test whose correct first run is a pass rather than a failure.

## When to use

- About to change, extract, port, or delete code with no test coverage and no written specification.
- A rewrite must match a legacy component output for output before it may replace it.
- Behavior is known only through the running system, and the people who wrote it are unavailable.
- A dependency or runtime upgrade needs proof that observable output did not move.
- Not for: behavior somebody can actually specify — write the specification as a test instead, via `test-driven-development`.
- Not for: a known defect — that is `regression-test-from-bug`. Characterization pins bugs deliberately; it does not fix them.

## Choosing the observation point

The pin must survive the refactor, and that is entirely decided before any output is recorded.

**Rule:** the assertion must be expressible without naming a single symbol you intend to rename, move, split, inline, or delete. If the test calls the function you are about to dissolve, the refactor breaks the test and you learn nothing about behavior.

- Pin at the **stable outer boundary** of the change: the module's public entry point, the HTTP handler, the command's stdout and exit code, the rows written, the messages emitted.
- Pin **effects as well as returns**. A function whose real work is a write, a publish, or a log leaves nothing to assert on if you only capture the return value. Capture the effects through the same seam you would use to fake them.
- If the only stable boundary is far too coarse — a whole application run per case — introduce one seam first, verify the seam changed nothing, and pin there. Adding a seam is a smaller risk than refactoring blind.

## Building the pin

1. **Draw inputs from production, not imagination.** Sample real requests, records, or files. Take a **stratified** sample across time, size, type, and account age — the first 100 rows are the oldest and least representative data you own. 30–100 well-spread cases beat 10,000 near-identical ones.
2. **Record the output by running the code and accepting what comes out.** This is the one place where a test passing on its first run is correct.
3. **Verify the pin has teeth before trusting it.** Mutate the source — flip a comparison, change a constant, drop a field — and confirm the recorded output changes. Pins that assert on empty output, swallow exceptions, or capture a value nobody computes pass happily forever, and this check is the only thing that catches them.
4. **Refactor.** Any diff in the pinned output is either a bug you introduced or a bug you fixed. Both require a decision. There is no third category, and "probably fine" is not a decision.

## Keeping goldens from churning

| Churn source | Fix |
|---|---|
| Timestamps and durations | Inject a fixed clock, or redact to a placeholder token |
| Generated ids, UUIDs, tokens | Deterministic id source, or normalize to `<id:1>`, `<id:2>` in order of appearance |
| Map, set, or query iteration order | Sort before serializing |
| Floating-point output | Round to a fixed precision at serialization |
| Absolute paths, hostnames, ports | Normalize to placeholders |
| Version and build strings | Strip them |
| Whole-object dumps | Project to the fields the change can plausibly affect |

Two thresholds decide whether a golden is doing its job:

- A golden that changes on more than **1 in 5 unrelated commits** is over-specified; narrow the projection until it stops.
- A golden over **~200 lines** is not reviewed by anyone; split it by scenario. Nobody reads a 4,000-line diff, and everyone approves one.

Store goldens as text with one logical item per line and a stable field order, so a diff is readable. A single-line JSON blob is technically a golden and practically unreviewable.

## Recording bugs on purpose

Current behavior includes current bugs, and the pin must capture them or the refactor will "fix" them by accident, in a change nobody flagged.

- Assert the wrong behavior explicitly and **name the assertion so it reads as wrong**: `current_behavior_rounds_half_down_incorrectly`, with a comment pointing at the defect record.
- Never let a bug enter a golden silently. An unlabeled golden becomes the specification within one team rotation, and someone will later defend it.
- Fixing the bug is a separate commit, with its own regression test, and a golden update whose diff is the entire visible content of that commit.

## Approval discipline and retirement

- Review the diff **hunk by hunk**. A one-key "accept all" is the standard way a regression enters a golden file, and it costs the technique its entire value.
- Approve per file, never per run. A run-level accept blesses changes in files you did not look at.
- Characterization tests are **debt with interest**. Delete each one when a real specification test covers the same behavior, and delete the whole set once the refactor lands and specifications exist.
- Track the count. A codebase where the golden count only ever grows is one where the specifications were never written, and where every future change is negotiated against a recording nobody understands.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The refactor breaks every pinned test | The pin named internals; the observation point was inside the change |
| Golden passes with the implementation gutted | Never mutation-checked; it asserts on empty, default, or swallowed output |
| Golden diff on every unrelated commit | Timestamps, ids, or iteration order leaking into the recording |
| Nobody reviews golden diffs anymore | Files too large, or a blanket accept command exists in the workflow |
| A bug was "fixed" by an unrelated refactor | The buggy behavior was never pinned, so nothing objected |
| A wrong output is now defended as intended | A bug was recorded without a label saying it was one |
| Rewrite matches the legacy system including its defects | Correct outcome for the cutover, wrong outcome to keep — pins were never retired |
| The pins are still there two years later | No retirement step; scaffolding was mistaken for a test suite |
| Cases all behave identically | Unstratified sample — every input came from the same slice of production |

## Red flags

- "Just accept the new snapshot, it's probably the refactor."
- "I'll pin it after I start refactoring."
- "The golden is huge but the diff is usually small."
- "This output looks wrong but let's not change it now" — without labeling it.
- Recording output from a build that has an unmerged local change in it.
- Sampling inputs by hand because production data was inconvenient to obtain.
- Keeping characterization tests as the permanent test suite for the rewritten component.
