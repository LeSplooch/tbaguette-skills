---
name: reproducing-bugs
description: Use when a bug report must be turned into a failure that happens on demand, when a defect cannot be reproduced locally, when it only occurs in production or for a single user, when a failure is intermittent or shows up at a low rate, when a probe or measurement returns plausible values that might just be defaults, or when a fix has no test that proves it. Covers missing report variables, minimal reproductions, input reduction, flake rates, validating the harness before trusting it, and capturing evidence instead of guessing.
---

# Reproducing bugs

## Overview

Until the failure happens on demand, every fix is a guess and every claim of "fixed" is unfalsifiable. The reproduction is the deliverable that makes the rest of debugging possible, and it is the step most bug work skips.

## When to use

- A report describes a failure you have not yet seen with your own eyes
- You are about to change code based on a theory you cannot test
- The failure is described as "sometimes", "intermittent", or "flaky"
- It happens in production, on one customer's account, or on one device only
- Someone is about to close a ticket as "cannot reproduce"

Not for: the hypothesis loop once you can trigger it (`diagnosing-before-fixing`), or finding which change introduced it once you have a reliable test (`bisecting-failures`).

## The six variables reports omit

Every report describes what happened. Almost none describe the state that made it happen. Ask for all six before concluding anything.

| Variable | Ask for | Why it gets omitted |
|---|---|---|
| Version | exact build/commit/package version, not "latest" | reporter assumes one version exists |
| Input | the actual bytes or payload, not a description of it | "just a normal CSV" hides the BOM, the CRLF, the emoji |
| Sequence | everything done before, including the steps that worked | reporter reports the last action, not the state machine |
| Environment | OS, locale, timezone, clock skew, proxy, extensions, flags, permissions | invisible to the reporter by definition |
| Timing | wall-clock time, concurrency, first request after deploy or idle | nobody thinks the hour of day is data |
| Data state | actual row counts, tenancy, migration level, cache warmth, entitlements | reporter cannot see their own data shape |

Two shortcuts that beat interrogation: ask for a screen recording (it captures sequence and timing for free), and ask them to reproduce it on a fresh account (a failure that survives that is not data state).

## Reducing to a minimal reproduction

Reduce the input, not just the code. Most people delete code and leave a 40 MB fixture in place.

1. Freeze the environment first. Reduce one dimension at a time or you learn nothing from a passing run.
2. Halve the input; keep whichever half still fails. Repeat. This converges in about log₂(n) runs, so a 100,000-row file reaches a single row in ~17 runs.
3. When both halves pass, the failure is an interaction. Split along a different dimension instead: by field rather than by row, by option rather than by line.
4. When halving stalls, switch to removing one element at a time until nothing more can be removed. Stop at that point — it is 1-minimal and further effort buys nothing.
5. Replace each remaining component with the simplest thing that still fails: real service to stub, database to in-memory value, framework entry point to direct call.

Targets for a finished reproduction: runs in under 30 seconds, fits in one file, needs no network, no shared state, and no manual step. If it still needs the whole stack running, you packaged the bug rather than reduced it.

## Prove the harness before trusting its readings

A tool that reports values instead of raising is the most dangerous instrument in debugging, because a silent degradation and a real negative result are the same output. A probe reading a property off a component that never loaded returns the default, formatted exactly like a measurement. A query against a stale replica returns rows, not an error. A check run against a cached copy of the artifact you just rebuilt reports your fix missing, confidently, in the correct units.

Before concluding anything from a measurement, assert that the thing being measured is actually present, attached, and current — not merely that the read returned something. The cheapest form is a **positive control**: read one value whose answer you already know. A probe that cannot see the value you are certain is there cannot see the one you are asking about either, and every number it has produced so far is worthless.

Both directions need it. Before believing a fix landed, confirm the build under test is the build you just made; before believing it did not, confirm you are not reading a cached, stale, or default-substituted copy. The failure is identical either way — the run agreed with you, and nobody checked that it was looking at the right thing.

## "Cannot reproduce" is not a result — the rate is

Run it n times and write down the number. An unmeasured "intermittent" is the single most expensive phrase in a bug tracker.

| Observed rate | Reading | Next move |
|---|---|---|
| 100% | deterministic | proceed to root cause |
| 5–95% | ordering, resource pressure, uninitialized memory, hash or iteration order, clock | `debugging-concurrency` |
| under 5% over 200 runs | narrow window or rare input | amplify with stress, delay injection, or reduced buffer sizes |
| 0 over 200 runs | your harness differs from theirs | stop running, start diffing environments |

To claim a fix on a bug that reproduces at rate p, you need roughly n ≈ 3/p clean runs for 95% confidence that zero failures was not luck: 60 runs at 5%, 300 at 1%. Anything less and "it stopped happening" is indistinguishable from noise.

## When it only happens there

Capture, do not guess. The instinct to theorize about production is the instinct to be confidently wrong for two days.

- Work the ladder: existing telemetry, then correlation-id log retrieval, then added capture at the boundary, then replay locally. Adding capture beats adding a fix.
- Capture as one bundle: the input payload, the response, config as loaded at that moment, every component's version, the wall clock, the identity or tenant, and the surrounding log lines by correlation id. A partial capture usually means a second incident to get the rest.
- For a one-user bug, diff their account against a working account field by field. In order of likelihood: data state, an entitlement or feature flag, then locale, timezone, or character encoding.
- Record real traffic and replay it against a suspect build. Recorded traffic is a reproduction that already contains all six variables.
- Never mutate production state to test a theory — see `observing-production-safely`.

## The reproduction is the acceptance criterion

- Write it as an automated test before fixing anything.
- It must fail for the reported reason, not merely fail. Assert on the specific symptom — the wrong value, the exact error — so an unrelated future breakage cannot masquerade as the bug still being fixed.
- Run it against the unfixed build and watch it fail. A test never seen failing tests nothing, and this is the most common way a regression test ships broken.
- Keep the reduced version as the permanent regression test. Keep the original end-to-end scenario only if the reduction dropped a layer that turned out to matter.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Works on my machine" | six variables changed at once; nobody isolated which |
| Closed as cannot-reproduce | the missing report variable was never requested |
| Reproduction needs the full stack | the code was reduced, the input and environment never were |
| Bug returns after the fix | the reproduction captured a symptom, not the trigger condition |
| Reproduction needs a "wait a bit" step | the trigger is an expiry, timeout, or eviction; parametrize the clock and drive it |
| Symptom vanishes under logging or a debugger | timing-sensitive; observation changed the window |
| Ticket says "intermittent", no number | nobody measured the rate, which is the most useful single fact available |
| Fix verified by one passing run | at a 5% rate, one pass is 95% likely regardless of the fix |
| Probe returns plausible values that are all defaults | the thing being measured never loaded; the read succeeded and measured nothing |
| A fix that did land reports as missing | the check read a cached or stale copy instead of the current artifact |

## Red flags

- "It's obvious what's wrong, I'll just fix it" — then you are shipping an untested hypothesis
- "I can't reproduce it, so it's probably already fixed"
- "It only happens for one user, so it's an edge case" — one user is proof the code path exists
- Reaching for the debugger before the failure is deterministic
- Writing the fix and the test in the same edit
- Reducing the input while also changing the environment
- "The tool returned a number, so it ran" — a default is also a number
- Drawing a conclusion from a probe that was never given a positive control
