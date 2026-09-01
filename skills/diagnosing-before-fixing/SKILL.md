---
name: diagnosing-before-fixing
description: Use when a bug, test failure, crash, or any behavior that doesn't match what the code is supposed to do needs a fix and none has been proposed yet, especially when the obvious quick fix is tempting under time pressure or an earlier attempted fix didn't hold. Covers the reproduce-hypothesize-test loop, tracing a symptom back to where it actually originates rather than where it surfaced, escalating from repeated failed fixes to questioning the architecture, and validating a fix at every layer the bad data passes through.
---

# Diagnosing before fixing

## Overview

A fix aimed at where a failure was noticed and a fix aimed at where it was caused are usually different changes — only the second one stays fixed. The loop is mechanical: reproduce the failure on demand, form one hypothesis specific enough to be wrong, design the smallest test that could prove it wrong, run it, and act on what comes back. Guessing under pressure feels faster; a thrash of unverified fixes is not — the systematic pass wins on wall-clock time, not just on correctness.

## When to use

- A test fails, a process crashes, or output stops matching what the code is supposed to do, and no cause has been named yet.
- A fix is about to be proposed for where the error surfaced rather than for why it happened.
- Under time pressure, when the fastest-looking change is also the one nobody has verified explains anything.
- A first fix didn't hold, or a second one didn't either.
- The failure crosses more than one component — client and service, build and sign step, app and database — and it isn't yet clear which one is actually at fault.
- Not for: a failure that is harming users, data, or money *right now*. The loop below reproduces before it fixes, and during an outage every minute spent reproducing is spent on the people affected — `responding-to-incidents` inverts the order, and hands back here once they are safe.
- Not for: the specific technique once you're already inside the loop — isolating which change introduced a regression (`bisecting-failures`), interpreting a crash or trace (`reading-stack-traces`), a failure tied to timing or interleaving (`debugging-concurrency`), memory or handles that grow without bound (`finding-resource-leaks`), or a failure that only sometimes happens (`flaky-test-triage`). This skill owns the loop; those five own the specialized ground inside it.

## The loop

1. **Reproduce.** Trigger it on demand, or measure a rate if it won't reproduce every time. Nothing to trigger yet at all — that's `reproducing-bugs`. Intermittent, and the failing thing is specifically a test — that's `flaky-test-triage`.
2. **Hypothesize.** State one specific cause: "X is happening because Y." If you can't say what result would prove it wrong, it isn't a hypothesis yet — it's a hunch with better grammar.
3. **Test.** Design the smallest change or probe that could falsify the hypothesis. One variable. Run it.
4. **Act.** Confirmed — fix at the source. Refuted — form a new hypothesis with what was just learned. Don't bolt a second untested change onto the first.

Repeat from step 2 on a refuted hypothesis. Repeat from step 1 if the investigation reveals the reproduction was never actually triggering the reported failure.

## Specialized terrain inside the loop

This skill is the loop, not a technique for reading a stack trace or narrowing a commit range. Once a hypothesis narrows the failure to a specific shape, the move usually already has a name:

| The hypothesis narrows to... | Reach for |
|---|---|
| Which commit, config change, or dependency bump introduced it | `bisecting-failures` |
| What a crash, exception, panic, or trace is actually saying | `reading-stack-traces` |
| Timing, thread or task interleaving, or load-dependent behavior | `debugging-concurrency` |
| Memory, descriptors, or connections that grow without bound | `finding-resource-leaks` |
| A failure that passes on rerun or only shows up in CI or under parallel load | `flaky-test-triage` |

Each of those skills still ends in a hypothesis confirmed or refuted — come back here and feed the result into step 4.

## Investigate first

A hypothesis formed before this step is a guess wearing a hypothesis's clothes.

| Do | Why |
|---|---|
| Read the full error and trace, not just the first line | The message usually names the exact invariant that broke |
| Confirm the reproduction is real — same steps, same result, every time or at the measured rate | A hypothesis tested against a reproduction that wasn't actually reproducing the reported bug explains nothing |
| Check what changed — diff, recent commits, new dependency, config, environment | Most bugs are introduced, not discovered. A diff too large to read by eye is what `bisecting-failures` is for |
| Find a working comparison and list every difference, however small | "That can't matter" is usually said about exactly the difference that does |

When the failure crosses more than one component — a pipeline stage, a client and a service, a build feeding a signing step — don't guess which layer is broken. Add a log at each boundary: what entered the component, what left it, what the environment actually was there. Run it once. The evidence, not a hunch about which layer you understand best, tells you where to look next.

## Trace backward to the source

A bug rarely originates where it's noticed. The error surfaces three calls downstream from the bad value — a file created in the wrong directory, a request rejected two services later, a null where something upstream should have set it — and fixing where it surfaced only patches the symptom while the actual defect waits to resurface down a different path.

Trace backward instead: what directly caused this? What called that, and with what value? Keep asking one level up until you reach the place the bad value was actually produced, then fix there — not at any of the points along the way. When reading the call chain by eye stalls, a stack trace captured immediately before the operation, not after it fails, usually unsticks it.

[reference/root-cause-tracing.md](reference/root-cause-tracing.md) has the full technique, including how to instrument a trace when the chain can't be followed by reading alone.

## Prove it before you build it

Testing a hypothesis means changing the smallest thing that isolates the variable, running it, and reading the result before touching anything else. A second untested change stacked on top of the first means neither result — pass or fail — tells you anything about either change alone.

When the test has to wait on something asynchronous, wait for the actual condition, not a guessed duration:

```
before — guesses at timing:
  sleep 50ms
  read result
  expect result is set

after — waits for the condition:
  wait until result is set, timeout 5s
  read result
  expect result is set
```

A fixed sleep before checking a result is a race wearing a test's clothes — it passes on a fast, idle machine and fails the moment either stops being true. Poll for the condition instead, with a timeout and an error that names what never became true. The one legitimate fixed delay is testing timing behavior itself, a debounce interval or a throttle window — and even then, wait for the trigger condition first and comment the number so it reads as derived rather than guessed.

[reference/condition-based-waiting.md](reference/condition-based-waiting.md) has the polling pattern and the ways it commonly gets implemented wrong.

## Fix at the source, then close every layer

Write the failing test first — the smallest reproduction, automated if a framework is available, a one-off script if it isn't. A fix nobody watched fail against a red test is running on faith; `writing-the-failing-test-first` covers writing it properly and `confirming-before-claiming-done` covers confirming the result before calling it done.

Make one change, addressing the cause the trace pointed at — not a bundle of adjacent cleanups, and not a second fix stacked on an unconfirmed first one. If it doesn't hold, that's a refuted hypothesis: return to investigating with what was just learned, rather than reaching for fix attempt number two before understanding why number one failed.

One validation point feels sufficient the day it's added and stops being sufficient the day a different call path, a test mock, or a routine refactor bypasses it. Once the trace shows where bad data enters and everywhere it does damage on the way through, validate at every layer in between. [reference/defense-in-depth.md](reference/defense-in-depth.md) has the four layers and why each one catches what the others miss.

## A retry is an experiment only if something differed

Re-running a failing operation feels like gathering evidence. It only is evidence if something changed between the runs — the input, the environment, the code, the credentials, the time. When nothing differed, the second run is a re-observation of the first and the third is a re-observation of the second, and the cost is paid three times for one fact.

Byte-identical output across attempts is itself a finding, and a useful one: it says the failure is deterministic, so the cause is structural rather than transient. That is the moment to stop attempting and start reading — the error text word by word, the documented behaviour of whatever emitted it, the platform rule it may be quietly stating. A message that repeats to the character is very often a system telling you a fact about itself that no local change can alter.

This sits in deliberate tension with the reproduction step above, and the tension is the point. Identical results are exactly what you want when confirming a reproduction is real, and exactly what should stop you when you are attempting a fix or an operation. Same observation, opposite meaning, and the question that separates them is always **what differed between those two runs?** If the honest answer is "nothing", no information was produced.

Note the cost, too. The three-failed-fixes signal below is real but expensive — it charges three fixes before it fires. This one is available immediately, before any fix has been written, and it costs a re-read.

## A factor that never varied has not been ruled out

Something gets dismissed early — the machine, the time of day, which of two paths the input took, which backend served the request — because every run so far shows it was the same. That is not elimination. A factor that never varies cannot *explain* variance, and that is a different statement from it not being the cause: the runs that would have shown its effect were never taken.

The tell is cheap and always available: **name the run where that factor was different.** If you cannot, it was observed, not tested.

The mistake is expensive because it is quiet. The dismissal is made once, early, in half a sentence, and everything after it is built on top — so the same wrong assumption survives every later hypothesis, including the sound ones.

It also survives the evidence that should kill it. A harness can be *recording* the factor on every single run, and a value that never changes reads as reassurance rather than as an untested variable — the data that would overturn the assumption is right there, being read as confirmation of it. When several individually-reasonable fixes have all failed, the thing to look for is not another hypothesis but the factor every one of them held fixed.

Vary it on purpose. That is the whole fix, and it is usually cheaper than the fix you were about to attempt — a routing flag, a different host, forcing the other branch. When the un-varied factor is *which commit*, that is what `bisecting-failures` is for. If varying it is genuinely impossible, say so and record it as an *assumption* rather than a *finding*.

## Several anomalies at once describe the apparatus, not the hypothesis

The loop's **Act** step takes the result and reads it as confirmation or refutation. That reading assumes something nobody checked: that the experiment ran. A test built to settle a hypothesis is new code, and new code has its own bugs — so a bad result has two possible authors, the world or the instrument, and they call for opposite responses. Believing the instrument retires a hypothesis that may be correct, and retires it silently, because a refuted hypothesis leaves nothing behind pointing at itself.

The discriminator is available before any analysis, and the counting is the whole of it: **count the ways the result is anomalous, and ask whether one wrong idea explains all of them.** A wrong hypothesis is a single mistaken belief about the cause, so it usually surfaces as a single deviation — the number moved the wrong way, or failed to move. A run that simultaneously never finishes a third of its attempts, triples its latency, and contains none of the category the change was built to select for is three unrelated failures, and no one wrong idea about the cause produces three unrelated failures. That is the signature of an apparatus that is broken, and the honest verdict on it is *no result*, not *refuted*.

One anomaly is evidence and several are a symptom, which inverts the usual instinct that a worse result is a stronger signal. When the count is high, stop reading the verdict and go read the new code — the hypothesis has not been tested yet, whatever the numbers say.

## When fixes keep failing

Three failed fixes in a row is a different problem than the one being solved. If each attempt reveals the same coupling in a new place, or needs a bigger change than the last one to hold, the architecture is what's wrong — not the last three hypotheses. Say so explicitly and question the pattern before attempting a fourth; a fourth patch on a bad architecture just becomes the fifth.

Occasionally a complete investigation turns up nothing fixable: the cause is environmental, an inherent timing dependency, or in code nobody here controls. That's a legitimate outcome — document what was ruled out, add the retry, timeout, or error handling the situation actually calls for, and instrument it for next time. Treat the conclusion with suspicion before accepting it, though: the large majority of "no root cause" verdicts turn out to be an incomplete investigation wearing a conclusion's clothes.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "The issue is simple, I don't need the loop" | Simple bugs have root causes too; skipping the loop on an easy-looking one is how it comes back twice |
| Fix applied, same class of bug reappears elsewhere | The trace stopped at the first plausible cause instead of the actual origin |
| "Let me just try changing this and see" | A change made before a hypothesis exists is a guess wearing an experiment's clothes |
| Several changes made, then the test run once | Whichever result comes back, nothing was isolated — pass or fail explains neither change |
| Fix confirmed by one passing run | One run at a nonzero failure rate isn't confirmation — see `reproducing-bugs` for how many clean runs actually are enough |
| "One more fix attempt" after two failures | Three failures is an architecture signal, not a reason to try a fourth patch |
| Test written after the fix, to "prove it works" | A test that never watched the bug fail proves nothing about whether it would have caught it |
| "I don't fully understand it, but this might work" | Uncertainty deployed instead of named — say what isn't understood and go investigate exactly that |
| The same command run four times with the same error | A retry treated as an experiment when nothing differed between the runs |
| "We already ruled that out" | The factor was the same in every run, so it was observed rather than tested |
| A hypothesis dropped on a result that was anomalous in several unrelated ways | One wrong idea does not produce several unrelated failures; the experiment broke, so it returned no verdict to act on |

## Red flags

- "Quick fix now, investigate properly later"
- "Just try changing X and see if it works"
- "It's probably X" — said before anything traced the failure to X
- Skipping the failing test because the fix "obviously" works
- Proposing a fix before tracing where the bad value actually originates
- "One more fix attempt" — said after two have already failed
- Each fix reveals a new instance of the same problem somewhere else
- Adding a sleep to make a flaky repro script pass, instead of waiting for the condition
- Investigating looks like nothing's happening, visibly, in front of people who are waiting; a bad retry's cost is invisible and lands later — that asymmetry is what's pulling, not an actual difference in risk
- "I'll skip it with a tracked TODO" — said with real intent, about a TODO that dies the moment the release ships and priorities move on
- An operation attempted again with nothing changed between the attempts, and the identical error read as bad luck rather than as determinism
- "That's the same for every run, so it can't be that"
