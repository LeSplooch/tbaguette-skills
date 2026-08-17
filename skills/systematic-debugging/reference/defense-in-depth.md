# Defense in depth — validate at every layer the bad data passes through

A single validation check feels sufficient the day it's added. It stops being sufficient the day a different call path, a test mock, or a routine refactor bypasses the one place that used to catch it. Fixing a bug at one layer means "this instance is fixed." Validating at every layer the data passes through means the bug is structurally impossible — not fixed, unrepeatable.

This is a different concern from validating input at a trust boundary — `handling-untrusted-input` owns parsing external data into a type that cannot hold an invalid value in the first place. These layers guard an internal invariant against your own code's *other* call paths, after a specific bug has already shown exactly where an invalid value can sneak through.

## Map the checkpoints before adding anything

Trace the data flow first ([root-cause-tracing.md](root-cause-tracing.md) if the origin isn't already known): where does the bad value enter, and everywhere it's used before it does damage. Each point on that path is a checkpoint. Adding four checks inside one function is not defense in depth — it's one layer with extra steps.

## The layers

- **Entry point.** Reject obviously invalid input where it first arrives at this component — empty, wrong type, out of range, a path that doesn't exist. This catches most cases and is the cheapest to write.
- **Business logic.** Reject input that's well-formed but doesn't make sense for this specific operation. A non-empty string can still be the wrong string; entry validation doesn't know that, the operation does.
- **Environment guard.** Block a dangerous operation outright in a specific context — refuse to touch anything outside a scratch directory while running under a test runner, refuse to call a production endpoint from a local profile. This layer catches the case where the first two were bypassed by a mock or a different code path entirely.
- **Debug instrumentation.** Log the value and a stack trace right before the operation that would misuse it. This layer doesn't prevent anything — it exists so that if all three above still miss a case, the next occurrence is a five-minute diagnosis instead of another multi-hour trace.

## Why one layer keeps not being enough

Each layer fails independently, which is exactly why stacking them works:

- A second, legitimate call path skips the first function entirely — entry validation never runs.
- A test mock replaces the object business logic validation lived on — that layer never runs either.
- A platform or environment difference introduces a danger the first two never anticipated, because it didn't exist when they were written.
- When the first three all miss, instrumentation is what turns the next occurrence into a quick fix instead of a repeat investigation.

None of the four is redundant with the others. "The entry check already covers this" is exactly the assumption the next refactor breaks.
