---
name: testing-the-untestable
description: Use when a test calls the wall clock, sleeps, hits the network, writes to the real filesystem, reads environment variables, generates UUIDs or random values, or depends on thread scheduling; when a test only fails at month boundaries, in another timezone, or on a slow machine; or when choosing between a fake, stub, mock, spy, and a real dependency in a container. Covers dependency injection at boundaries and seeding.
---

# Testing the untestable

## Overview

Nothing is untestable; some things are un-injected. Every source of nondeterminism reaches the code through a specific call, and testing it means owning that call rather than working around it downstream.

## When to use

- Code reads the current time, sleeps, retries with backoff, or expires something.
- Code generates ids, tokens, salts, shuffles, or samples.
- Code opens sockets, resolves DNS, reads files, or reads environment and config.
- Code spawns threads, tasks, or processes and the test's outcome depends on their interleaving (`debugging-concurrency` once an interleaving bug is already loose, rather than being designed against).
- A test fails only in another timezone, on the 1st of the month, or on a loaded CI worker.
- Not for: diagnosing a suite that is already intermittently failing — that is `flaky-test-triage`.
- Not for: deciding what the substitute on the far side of the seam should *return*. This skill places the seam; `grounding-test-doubles` decides whether the double is faithful — captured from the real thing rather than composed from memory, and raising rather than shrugging on a shape it does not recognise. A seam in the right place feeding an invented payload still ships a green suite over a broken integration.

## Own the boundary, one seam per category

| Source | Injected as | What it costs to skip |
|---|---|---|
| Wall-clock time | A clock the caller supplies; `now()` never called directly in domain code | Tests that fail at midnight, month end, or across DST |
| Elapsed time | A separate monotonic source; never derived by subtracting wall clocks | Duration and timeout tests that flake when the clock is adjusted |
| Timezone and locale | Explicit parameter, never the ambient process default | Green locally, red on a differently-configured machine |
| Randomness | A seeded generator passed in | Failures that cannot be reproduced |
| Identifiers | An id source with a counter in tests | Assertions weakened to a regex, hiding real id bugs |
| Network | A client interface at your adapter, or a real server on a loopback port | Suites that fail when the office wifi does |
| Filesystem | A root directory passed in; a unique temp dir per test | Tests that pass only in the directory the author ran them from |
| Environment / config | Read once at startup into an injected value object | Process-global mutation that breaks under parallelism |
| Concurrency | An injected executor the test can run inline or step deterministically | Races that only appear on a machine you do not own |

One seam per category, at the boundary of the code you own. Injecting at every call site produces a test that reconstructs the whole world, which is the same failure as mocking everything.

## Time, and why sleeping is always wrong

Sleeping encodes a guess about a machine speed you do not control. Too short and it flakes; too long and it taxes every future run. Both outcomes get worse as the suite grows.

Replace a sleep with one of three things, in order of preference:

1. **Advance a fake clock explicitly.** The test states "45 minutes pass" and the code observes it instantly. Timeouts, expiries, backoff schedules, and rate limits all become instant and exact.
2. **Wait on a condition with a timeout**, polling at 1–10ms up to a generous ceiling (5–10s). The test finishes as soon as the condition holds, so a generous ceiling costs nothing on the happy path and produces a clear failure otherwise.
3. **Expose a synchronization point** in the code — a completion signal, a drained queue, an awaited handle. If the code cannot tell you when it is done, callers in production cannot know either.

Global time-freezing that patches the runtime is a shortcut with a specific failure: it also freezes the timeouts and retries inside libraries you do not control, and the test deadlocks in a way that looks like a hang, not an assertion.

Fake clocks come in two flavours: **manual** (the test advances it) and **auto-advancing** (it jumps to the next scheduled timer). Manual is the default; auto-advancing is for code that schedules work you cannot enumerate.

## Fake, stub, mock, or real

| Double | Behavior | Assert on | Cost and failure mode |
|---|---|---|---|
| Dummy | Nothing; fills a parameter | — | Free |
| Stub | Returns canned answers | The code's output | Cheap; goes stale silently when the real contract changes |
| Fake | A working, simplified implementation | The code's output, and state in the fake | Moderate to write; earns it back across many tests; drifts from the real semantics |
| Spy | Records calls, real or stubbed behavior | Interactions, after the fact | Cheap; tempts assertions on internals |
| Mock | Pre-programmed expectations, verified | Interactions, as a specification | Couples the test to call sequence; refactor-hostile |
| Real, in a container or in-process | Actual semantics | The code's output and the dependency's state | Seconds of startup; the only thing that catches semantic mismatches |

Rules that decide it:

- **Use the real dependency when your code depends on its semantics, not just its signature.** SQL dialect, transaction isolation, index and collation behavior, uniqueness enforcement, blob size limits, message ordering, and serialization formats are semantics. An in-memory substitute for a real database is the classic source of "passed in CI, failed in production".
- **Use a fake for anything you own the interface to** and use in more than ~5 tests: a clock, a queue, a key-value store, a payment gateway you wrap.
- **Use a stub for a single-use collaborator** whose answer is an input.
- **Use a mock only when the interaction itself is the requirement**: the audit log must be written, the payment must be charged exactly once, the cache must not be consulted twice.
- Escalate when a double crosses ~50 lines or grows conditional branches. At that point it is an unmaintained reimplementation, and the real thing in a container is cheaper and more honest.
- Amortize container cost across the suite: start once per suite, isolate per test by namespace, schema, or transaction rollback — not by restarting.

## Auditing determinism

Run these four checks; each one catches a class nothing else does.

- **Same seed, twice** — diff full output. Any difference is unowned nondeterminism.
- **Shuffled order** — exposes shared state and hidden ordering assumptions.
- **Shifted environment** — a clock offset of +6 months, a non-UTC timezone with a non-hour offset (`Asia/Kathmandu`, +05:45), and a non-English locale. This finds date-arithmetic, formatting, and collation assumptions in one run.
- **Constrained CPU** — one core, or the suite under load. Races surface here and nowhere else.

The first check has a failure mode of its own, and it is the reason the third
one is on the list rather than being folded into it. **Two runs that share an
ambient condition are blind to dependence on it.** Run "same seed, twice" back
to back in one process and both invocations read the same clock, the same
environment, the same working directory, the same machine identity, the same
free disk — so a system genuinely reading any of those produces identical
output, the check passes, and it passes for as long as the two runs stay
adjacent. A determinism test that holds an input fixed across both of its own
runs is certifying independence from exactly the thing it never varied.

So vary the ambient condition *between* the two runs rather than around them:
different clock values, separate processes, different working directories,
different environments — the shifted-environment check applied to the pair
rather than to the suite. Where varying it genuinely is not possible, the test
still has value and is weaker than its name claims, and the name is where to
say so: `same_seed_same_output_within_one_process` tells a later reader what
was not covered, and `is_deterministic` does not.

## Ordering, and the tiebreak that is randomness

Seeding the generator is the first thing to reach for when a run stops reproducing, and it leaves a gap that is easy to miss. The seam table above gives identifiers their own row and their own injection point, separate from randomness, and skipping that row is what this failure is made of: v4 UUIDs, session tokens and generated slugs are usually minted by a library drawing on the operating system's entropy, not on the generator you seeded. Seed every source you own and the ids keep changing anyway — so the same-seed check comes back with a difference, and the search sets off after a second bug that does not exist.

The place that difference does the most damage is a sort. A comparator's final term decides the order of every pair the earlier terms tied on, so a final term that is a randomly-generated id breaks each of those ties at random. Nothing about the code looks careless: `(created_at, id)` reads as a deliberate total ordering, and it is one — it is simply a different one each time.

Tiebreak on something intrinsic to the record instead: a natural key the domain already has, its insertion index, a hash of its content — anything derived from the record rather than minted alongside it. This matters more often than it sounds, because **ties are the normal case rather than the edge case.** Any batch that stamps its rows from a clock read once at the top gives every one of them the same value, so across that batch the tiebreak is not breaking ties, it is the sort.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Test passes locally, fails in CI | Ambient timezone, locale, filesystem case-sensitivity, or core count differs — `reproducible-environments` covers pinning the ones the test never declared |
| Test fails once a month or once a year | Wall clock read directly; month end, leap day, or DST crossed the assertion |
| Timeout test takes 30 real seconds | No fake clock; the test is waiting rather than advancing |
| Sleep durations keep being increased | A race is being masked; the sleep length is now load-bearing |
| Reproducing a failure requires the original machine | Randomness or id generation unseeded and unrecorded |
| Every generator is seeded and the output still differs run to run | Ids minted from the OS entropy rather than the seeded generator, and something downstream sorts on one |
| A determinism test passed for months over a system that was reading the clock | Both of its runs read the same clock; the ambient input it existed to exclude was held fixed across the pair |
| Everything is mocked and nothing catches bugs | Seams placed at every call rather than at the boundary; tests assert the mock's script |
| Test suite hangs with no output | Global time-patching froze a library's internal timer |
| Passes alone, fails in parallel | Process-global environment variables, a fixed port, or a shared temp path |
| Green tests, broken production query | An in-memory substitute stood in for a real store with different semantics; `grounding-test-doubles` keeps one live test per integration for exactly this |

## Red flags

- "Just add a sleep, it's only 200ms."
- "It's flaky on CI, bump the timeout."
- "We can't test that, it hits the network" — the adapter is the thing to test.
- "The mock verifies we call the service correctly" — for behavior no user can observe.
- Setting an environment variable inside a test and restoring it in teardown.
- A test asserting on an id with a pattern match because the id could be anything.
- A randomly generated id as the last term of a sort key.
- A determinism check whose two runs happen back to back, in one process, on one clock.
- Retrying a whole test to deal with a race.
