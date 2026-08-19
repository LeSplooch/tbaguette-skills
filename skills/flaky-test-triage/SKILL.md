---
name: flaky-test-triage
description: Use when a test passes on rerun, fails only in CI, fails only when the whole suite runs, fails after midnight or across a DST change, fails under parallel execution, or fails on a loaded machine; or when someone proposes a retry, a skip, or a longer timeout to make the build green. Covers intermittent failures, order dependence, shared state, races, and quarantine.
---

# Flaky test triage

## Overview

A flaky test is a defect report whose owner has not been determined yet. The defect is in the test roughly half the time and in the product the other half — and the half that lives in the product is a race that users are already hitting at the same rate the test fails.

## When to use

- A test failed, was rerun, and passed, with no code change in between.
- A test passes alone and fails in the suite, or passes in the suite and fails alone.
- Failures cluster on CI, on one worker, at a time of day, or after a DST change.
- Someone is about to add a retry, a skip marker, or a larger timeout.
- The suite is rerun as a matter of routine, and nobody reads the first result.
- Not for: a test that fails every time — that is a deterministic bug, owned by `diagnosing-before-fixing`.

## The arithmetic that forces the issue

Independent per-test reliability compounds. At 99.9% per test, a 500-test suite is green 61% of the time; a 2,000-test suite, 13%. To keep a 2,000-test suite green on 95% of runs, each test must be 99.997% reliable. There is no per-test flake rate that is acceptable at scale, which is why "it only fails 1 in 50" is not a defense.

## Cause taxonomy

| Signature | Cause | First diagnostic |
|---|---|---|
| Passes alone, fails in suite | Order dependence: shared mutable state, cached singleton, unrolled DB write | Bisect the test order to find the polluter |
| Fails only under parallel execution | Contended resource: fixed port, shared temp path, same DB row, global config | Run the same subset single-threaded |
| Fails near midnight, month end, or DST | Wall clock read directly; date computed twice across a boundary | Rerun with the clock offset by +6 months and a `+05:45` timezone |
| Fails only on slow or loaded machines | Sleep or timeout tuned to the author's laptop, or a genuine race | Run under one core or artificial load |
| Fails at a stable rate with no pattern | Unseeded randomness in data, ids, or iteration order | Seed everything; check hash and map iteration order |
| Starts failing after N runs, then always | Resource leak: file descriptors, connections, disk, ports in TIME_WAIT | Watch handle and connection counts across the run |
| Fails only on the first run of the day or in a clean environment | Test depends on a warm cache, existing data, or a previously-created account | Run against a freshly provisioned environment |
| Order-dependent assertion on a callback-built list | The callback only preserves delivery order under a real dispatcher/event-loop context; without one — the normal case in a test harness — the guarantee disappears, whether the runtime falls back to unordered dispatch, throws, or no-ops | Check whether the code normally runs under a real dispatcher or event loop that the test doesn't supply |
| A different assertion fails each time | A real race in the product, not in the test | Stop triaging the test; treat it as a production incident |

The last row is the one that gets misfiled most often, and it is the one that matters most.

The order-dependent-callback row has the opposite prescription: the fix is asserting an order-independent invariant — max value reached, set membership, monotonic non-decrease — instead of position, or giving the test a synchronous stand-in for the missing dispatcher context, never a retry.

## Diagnosis

1. **Capture before rerunning.** Full output, seed, test order or shuffle key, worker index, host, timestamps, and any artifacts. A rerun destroys the only evidence; a flake report lacking seed and order is unactionable, so make CI print both on every run.
2. **Quantify in isolation.** Run the single test 100 times alone. Never failing alone means the cause is environmental or order-related, and further solo debugging is wasted.
3. **Quantify in context.** Run the full suite with the recorded order 10–20 times. Establish a reproduction rate before attempting any fix; without one, the fix cannot be evaluated.
4. **Bisect the order.** With a reproducing order fixed, binary-search the prefix of tests run before the victim. Roughly 10 runs isolates one polluter out of 1,000. This finds shared-state bugs no amount of reading finds.
5. **Vary one axis at a time.** Threads: parallel versus single. Clock: shifted date and non-hour-offset timezone. Load: constrained CPU. Randomness: fixed seed. Each axis that changes the failure rate names the category.
6. **Attribute.** Write down whether the defect is in the test or the product before writing any fix. A fix applied without attribution usually just moves the flake. When the defect is in the test — it's exercising something inherently non-deterministic rather than a real product race — `testing-the-untestable` covers making that source of non-determinism controllable instead of retried around.

## Quarantine, with an expiry

Quarantine is a scheduling decision, not a verdict. Every quarantined test carries four things or it is not quarantined, it is abandoned:

- A named **owner**.
- A linked **defect** describing the observed failure, with the captured evidence.
- An **expiry date**, 14 days or less. On expiry the test is fixed or deleted — no third option.
- Continued **execution**: it still runs and still reports, it just does not block the build. A quarantined test that stops running is deleted with extra steps, and the code it covered is now untested without anyone deciding that.

Cap the pool. Past ~1% of tests in quarantine, the suite no longer signals what people believe it signals; stop feature work until it drains.

## Why blanket retry is the wrong response

Retrying converts an intermittent product race into a green build. The rate the retry hides is the rate users experience — the test was measuring it, and the retry deletes the measurement. Two conditions make a retry defensible, and both must hold:

- The failure is attributed to infrastructure outside the code under test (a provisioning timeout, a registry fetch), not to product behavior.
- Every retry is **counted, reported, and alarmed** on a threshold. Uncounted retries always grow, because nothing pushes back.

Retrying at the whole-suite level is worse than at the test level: it hides which test is unreliable and multiplies CI cost against a failure nobody will ever look at.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "It passed on rerun, so it's fine" | The rerun destroyed the evidence; the failure rate is unchanged |
| Fix applied, flake returns weeks later | No reproduction rate was established, so the fix was never verified |
| Timeout raised repeatedly over a year | A race is being masked; each increase buys a shorter reprieve |
| Same test flakes after being "fixed" three times | The defect is in the product; the test keeps being adjusted instead |
| Nobody investigates because the report has no detail | CI does not print seed, order, or worker; every report is a dead end |
| Suite green, users hit intermittent errors | Retries hid a real race that the test was correctly detecting |
| A skipped test's feature breaks in production | Quarantine without expiry silently removed coverage |
| Flakes only appeared after enabling parallelism | Pre-existing shared state that serial execution was accidentally hiding |
| Failure moves to a different test after a fix | The polluter was never found; only the victim was patched |
| Test asserting the last item of a callback-built list is intermittently flaky | Looks like a product race, but it's the test harness missing the dispatcher context the callback needs to stay ordered |

## Red flags

- "Just rerun it."
- "It's flaky, ignore it" — said about a test nobody has attributed.
- "Add `@retry` and move on."
- "Bump the timeout to 30 seconds."
- "Skip it for now" with no date attached.
- "It only fails in CI, so it's a CI problem."
- Marking a test flaky without ever having reproduced it.
- A test file with more skip markers than assertions.
- Asserting a specific position — especially "the last item" — in a collection built by an async callback, with no real dispatcher or event loop under test.
