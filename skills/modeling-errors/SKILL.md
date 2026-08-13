---
name: modeling-errors
description: Use when deciding how a failure should be represented or handled — choosing between exceptions, result or either types, error codes, panics, and supervisors; writing a catch, rescue, or recover block; designing an error type or an error contract; deciding whether to wrap, log, rethrow, retry, or swallow. Also for silent failures, swallowed exceptions, undiagnosable production incidents, duplicated log noise, and callers parsing error strings.
---

# Modeling errors

## Overview

Classify the failure before choosing a mechanism. Almost all bad error handling is one mechanism applied uniformly to four kinds of failure that need four different treatments.

## When to use

- Adding error handling to a new operation, or reviewing a `catch` you did not write.
- Designing the error side of an interface: what callers can be told, and what they can do about it.
- Symptoms: a subsystem "silently does nothing", one failure produces four log lines, a retry loop that can never succeed, an incident where nothing recorded what actually failed.
- Not for: the wire-level shape of the error contract at a public boundary (designing-apis), or the safety of the retry itself (designing-for-idempotency).

## Four classes, four treatments

| Class | Examples | Mechanism | Caller's obligation |
|---|---|---|---|
| Expected, recoverable | not-found, validation rejection, version conflict, insufficient funds | Ordinary return value or a typed/checked failure — it is an outcome, not an exception to the rules | Handle it; this is normal control flow |
| Expected, fatal to this unit of work | missing config at startup, disk full, unparseable input stream | Propagate unchanged to the error boundary | Abandon the unit of work |
| Programmer error | broken invariant, precondition violated, unreachable branch reached, index out of range | Crash, panic, or abort as loudly as the environment permits | None — the caller *is* the bug |
| Infrastructure / transient | connection reset, lock timeout, throttle, peer restarting | Same as class 1, but carrying a retryable marker and a hint | Retry within a budget, then degrade |

**Treating class 3 as class 1 is the single most common way a system becomes undiagnosable.** A broad catch low in the stack turns a bug into a plausible-looking default value, and the wrong answer propagates for months. Class 3 must never be inside a retry loop and must never be caught by a generic handler. Where crashing is unacceptable (firmware, kernel, single-process device), fail the smallest schedulable unit and record the specific invariant that broke — do not substitute a default.

Class 4 is defined by one property only: **retrying the identical input might succeed.** If it cannot, it is class 1 or 2 no matter how infrastructural it looks. An authorization failure from a network call is not transient.

## Choosing the mechanism

- Choose what the language makes ergonomic, not what you prefer. In an exception-based codebase, a lovingly hand-rolled result type does not remove exceptions — it just means third-party ones are now unhandled.
- The landscape: unwinding exceptions (Java, Python, C#, C++, Ruby); result/either values (Rust, Haskell, Swift typed throws, Go's error-return convention); error codes with out-params (C, syscalls, most embedded); conditions and restarts (Common Lisp — the handler decides at the raise point, the only mainstream mechanism that can resume); crash-and-supervise (Erlang, Elixir — class 3 discipline made structural).
- Three properties actually differentiate them: whether the type system can tell you an error is unhandled, whether the mechanism carries a cause chain and a stack, and whether the happy path pays a cost. Rank those for your context and the choice falls out; arguing result-versus-exception in the abstract does not.
- Whatever the mechanism: **the set of failures an operation can produce is part of its signature**, documented even when the language refuses to type it.

## Errors as values a caller can act on

Five things, every time:

1. **What failed** — a stable machine-readable code. Not prose. Prose gets reworded and somebody downstream is matching on it.
2. **What was attempted** — the operation plus identifying inputs, secrets and personal data redacted.
3. **Whether retry can help** — a boolean, plus a retry-after duration when known. The callee knows this; the caller is guessing.
4. **A correlation identifier** — request, trace, or job id, so a user's report maps to your log line.
5. **The cause chain** — preserved as structure, never flattened into a string.

A good message names what was being done, what happened, and what changes the outcome: `connect to config store at <addr>: connection refused (retryable, retry after 2s)`. Never "an error occurred"; never a message that only makes sense to the person who wrote the raise site.

## Wrap, swallow, rethrow, boundary

| Action | Correct when | How it goes wrong |
|---|---|---|
| Wrap with context | Crossing a layer where the inner error lacks nouns the caller knows | Five layers of `failed to X: failed to Y: failed to Z: EOF`. Wrapping must add **nouns** (which file, which record, which attempt), never verbs |
| Swallow | The failure is genuinely not an outcome — an optional cache read, a best-effort metric | No counter, so a 30%-failing subsystem goes unnoticed for a year. Swallowing requires a comment naming why *and* a metric |
| Rethrow unchanged | You had nothing to add | Underused; people wrap out of habit |
| Translate to your own type | At a boundary you own, so vendor error types stay out of your callers | Translating everything to one generic type, destroying the classes above |
| Retry | Class 4 only, at a layer that owns a deadline and a budget | Retries nested at three layers multiply: 3×3×3 attempts and a 27× load amplification during an outage |

**The error boundary is one per unit of work** — request, message, job, frame, transaction. It catches whatever remains, logs once with the full chain and the correlation id, converts to the caller-facing contract, and decides the unit's disposition (fail, retry, dead-letter). A service typically has 2–5, one per kind of unit. Log at the boundary, not at the raise site, and never at both: log-and-rethrow at each layer is why "how many errors happened" is unanswerable in most systems.

## Common mistakes

| Symptom | Real cause |
|---|---|
| One failure appears four times in the logs at four layers | Log-and-rethrow instead of logging once at the boundary |
| An incident with no record of which call failed | Broad catch low in the stack substituting a default |
| Retry loop burns quota and never succeeds | Non-retryable class retried; no retryable marker in the error |
| "It just returns empty" | Not-found modeled as an empty collection; caller cannot distinguish |
| Callers parse error message text | No stable code in the contract |
| Bug reproduces only in production and leaves no trace | Programmer error caught by a generic handler |
| Wrap chain reads "failed to process: failed to handle: failed to run" | Wrapping with verbs instead of nouns |
| One bad message kills the whole consumer | Error boundary at process granularity instead of per-message |
| Load spike during a partial outage | Retries nested at multiple layers, multiplying attempts |

## Red flags

- "Catch everything here so it doesn't crash."
- "Log it and continue."
- "Return null on failure and let the caller figure it out."
- "That can never happen" — written next to the code that handles it happening.
- "Add a retry" as the first response to a flake, before classifying it.
- A catch block whose body is empty, or contains only a log statement.
- An error type with one variant and a string.
