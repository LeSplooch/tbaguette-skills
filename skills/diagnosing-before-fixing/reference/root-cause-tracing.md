# Root cause tracing — trace the symptom back to its origin

A bug that manifests deep in a call chain rarely originates there. A file lands in the wrong directory, a request is rejected three services downstream, a value is null where something upstream should have set it — the instinct is to fix where the error surfaced. That patches a symptom and leaves the actual defect in place, waiting to resurface the next time the same bad value takes a different path to a different symptom.

## The technique

1. **Observe the symptom.** State exactly what went wrong and where it was noticed.
2. **Find the immediate cause.** What line of code directly produced this failure?
3. **Ask what called it, with what value.** Go one level up the call chain.
4. **Keep tracing up.** At each level, ask the same question again: what called this, and what did it pass?
5. **Find the original trigger.** Stop when you reach the place the bad value was actually produced — not merely passed along.
6. **Fix there**, not at any of the points along the way.

A short example: an error reports a file created in the wrong directory. The immediate cause is a write call with an empty path argument. That path came from a function three callers up that defaulted an unset argument to an empty string instead of raising. That function was called by test setup code reading a fixture value before the fixture had finished initializing. The defect — and the fix — is at the fixture, not at the write call where the symptom appeared. Fixing the write call would have stopped this one crash and left every other caller of that defaulting function just as broken.

## When linear tracing stalls

Add instrumentation instead of guessing further. Log immediately before the dangerous operation, not after it fails — a log placed after the failure never runs. Capture the value about to be used, the current context (directory, environment, whatever the operation depends on), and a captured stack trace, not just a message. In a test suite specifically, write these lines somewhere guaranteed to be visible; a logger with configurable verbosity is sometimes silent exactly when it's needed most, so an unfiltered write to the console is the safer choice while actively tracing.

When the graph of possible callers is too wide to trace one at a time — many call sites could plausibly be the one producing the bad value — narrow it the same way `bisecting-failures` narrows a commit range: isolate half the callers, rerun, keep the half that still reproduces, repeat. A handful of runs finds one culprit among hundreds.

## Trace, then defend

Finding the source is half the job. A trace that ends at "found it" and stops is a trace that gets repeated the next time a different code path produces the same bad value. Once the origin is known, add validation there — and, per [defense-in-depth.md](defense-in-depth.md), at the other layers the value passes through on its way to causing damage.
