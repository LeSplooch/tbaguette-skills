---
name: debugging-concurrency
description: Use when a failure depends on timing, thread or task interleaving, or load, when a test passes alone but fails in parallel, when a hang, deadlock, livelock, lost update, or stale read appears under concurrency, when adding logging or a debugger makes the symptom disappear, or when a thread dump or hung process must be read. Covers races, happens-before reasoning, stress reproduction, and deterministic replay.
---

# Debugging concurrency

## Overview

A concurrency bug is a missing happens-before edge. The symptom is timing; the defect is an ordering constraint you assumed and never established, and no amount of reasoning about "what usually happens first" will find it.

## When to use

- Behavior changes with load, core count, machine speed, or run order
- A hang with no crash, or a value that is occasionally wrong with no error anywhere
- The symptom disappears under a debugger, a profiler, or added logging
- A test passes alone and fails in parallel, or fails only in CI
- A dynamic race detector or thread sanitizer reported something on a passing run

Not for: a flaky test whose cause is shared fixtures or ordering in the harness rather than the system under test (`flaky-test-triage`), or an intermittent failure whose rate has not been measured yet (`reproducing-bugs`).

## Classify before investigating

| Symptom | Class | The thing that is missing |
|---|---|---|
| Wrong value, no hang, no error | data race or lost update | atomicity across the whole read-modify-write, not just the parts |
| Full hang, CPU near idle | deadlock | a global lock acquisition order |
| Hang with CPU pinned | livelock or spin | backoff, or a progress condition that can actually become true |
| Passes alone, fails in parallel | shared global, static, temp path, port, or fixture | isolation |
| Occasional stale read, no corruption | visibility under the memory model | publication: atomic release/acquire, a fence, a channel, or a lock |
| Only on more cores or under load | a legal interleaving that was previously improbable | nothing changed except probability |
| Vanishes when observed | any of the above | the observation widened or closed the window |
| Correct until scaled up | queue growth, pool exhaustion | backpressure and bounds |
| Duplicate side effects | at-least-once delivery meeting non-idempotent code | idempotency key (`designing-for-idempotency`) |

## Reason from happens-before

For any two operations that touch the same state, name the edge that orders them. The edges are few: program order within one thread; lock release to a later acquire of the *same* lock; queue or channel send to the matching receive; spawn to task body; join or await to continuation; atomic store-release to load-acquire on the *same* location.

- No edge means both orders are legal, and the compiler and CPU may also reorder within each thread. "It only ever happens in this order in practice" is not an edge; it is an observation about one machine, one build, and one load level.
- **Check-then-act across an edge boundary is the dominant shape.** Exists-then-create, empty-then-take, valid-then-use, size-then-index, absent-then-insert, unlocked-then-lock. Every one needs the check and the act inside the same critical section, or a single compare-and-swap.
- **Atomicity does not compose.** Two atomic operations are not one atomic operation, and a concurrent collection makes each call safe while leaving every sequence of two calls unsafe. This is the most common way a "thread-safe" data structure still loses updates.
- Volatile or atomic fields give visibility, not mutual exclusion. A counter increment on an atomic field is still three operations unless the increment itself is the atomic one.
- The class-level fix beats the instance-level one: immutability, confinement to one thread or actor, and message passing remove the whole category rather than the occurrence you found. That is a model change, not a patch — `choosing-concurrency-model` covers picking one that cannot express the bug, which is the only fix that does not have siblings.

## Make it more likely, not less

The instinct is to stabilize. Invert it — an unreproducible race cannot be verified as fixed.

- **Stress:** loop the case 1,000+ times, with concurrent workers at 2× core count. Keep the whole run under 60 seconds so it can serve as a bisect step later.
- **Pin to one core** to force preemption-driven interleavings, then run on the highest core count available for true-parallel ones. These find different bugs; do both.
- **Inject a delay at the suspect point.** A sleep or yield between the check and the act turns a one-in-a-million race into near-certainty. This is also the cheapest possible hypothesis test: if the injected delay makes it fail every time, the window is confirmed, and if it does not, your theory is wrong and cost you two minutes.
- **Shrink everything bounded:** pool size to 1, queue capacity to 1, buffer to 0, timeouts to their minimum. Contention appears immediately.
- **Randomize scheduling:** yield with some probability at every synchronization point, if the runtime offers a hook or a systematic-interleaving test mode.
- **Run the race detector or thread sanitizer.** These report a missing edge on runs that produce correct output; a passing run with a race report is a confirmed bug, not a false positive.
- **Soak** for 4–24 hours at steady load for deadlocks that need a rare combination, and watch monotonic counters (see `finding-resource-leaks`).

## Reading a hung process

Before dumping anything, check whether it's actually stuck — dumps cost minutes, this check costs seconds. `ps -o etime,time,pcpu -p <pid>` (or the platform equivalent) compares elapsed wall-clock time against CPU time accumulated: a process at 0:03 CPU time after an hour elapsed is blocked, worth the dump comparison below; one at 40:00 CPU time after that same hour is running, just slower than expected — not deadlocked.

When CPU time is climbing, check system load and swap (`uptime`, `vmstat`, `free`, or their equivalents) before concluding anything is wrong with the process itself — it may just be losing a scheduling fight with everything else on the box. Killing and restarting doesn't fix that fight; it discards whatever warm-up, cache, or partial progress the process had built and drops it back into the same contention, sometimes ensuring it never finishes.

Take at least three dumps 5–10 seconds apart. One dump cannot distinguish stuck from busy — the comparison is the whole technique.

1. Count threads by state. A wall of threads in the same wait is contention or pool exhaustion; two threads in a cycle is a deadlock.
2. Look for the cycle: thread A waiting on a lock held by B, B waiting on one held by A. Most runtimes will detect and print simple cycles for you; do that before reading manually.
3. Ignore idle pool threads. A 400-thread dump usually has two or three that matter.
4. Diff the dumps. Threads on the same frame in all three are stuck. Threads that moved are merely slow.
5. If the deadlock is not a lock cycle, look for the resource ones: a bounded pool where a task holds one connection while waiting for a second, a single-threaded executor whose task blocks on work submitted to itself, or a lock held across an I/O call with a long timeout.

Where no dump facility exists, attach a debugger and dump all thread backtraces, send the runtime's dump signal, or take a core. Deterministic record-and-replay tooling, where the platform has it, is worth reaching for before any of this: the second-worst property of a race is that the evidence is gone by the time you look.

## Fixes that are second bugs

| The "fix" | What it actually did |
|---|---|
| Add a lock until the symptom stops | introduced an unwritten lock-ordering constraint, usually widening a critical section over I/O |
| Make the field atomic or volatile | fixed visibility; the compound read-modify-write is still not atomic |
| Increase the timeout | converted a deadlock into a slow drain of blocked workers, discovered later and further away |
| Retry on failure | hid the race and doubled the writes; catastrophic if the operation is not idempotent |
| One global lock over everything | correct, unshippable at load, and it converts races into deadlocks |
| Sleep to let the other thing finish | an unenforced happens-before with an expiry date set by the next fast machine |
| Reorder two statements until it passes | depends on the compiler not reordering them back, which it is permitted to do |
| Kill and restart a process that was only slow, not hung | discarded its warm-up, cache, or partial progress and re-entered the same contention it was losing before |
| Relocate a reentrant deadlock's block, reasoning about scheduler timing | still a blocking wait somewhere in the call chain; correct until a different caller, version, or load pattern schedules it differently |

The test for a real fix: state in one sentence which happens-before edge now exists that did not before. If the sentence is "there is a lock now" without naming what it orders against what, a symptom was removed.

A reentrant deadlock — a blocking wait nested inside another blocking call on the same thread or executor — has a second test, because the first fix attempted is almost always to relocate the block: run it earlier, run it on a different thread, reason about which thread the runtime will actually schedule it onto. That reasoning is fragile by construction — it holds until the next caller, version, or load pattern schedules something differently, and the hang comes back. The fix that survives removes every blocking wait from the call chain instead: await or its equivalent end-to-end, no blocking invoke anywhere in the path. If the call chain still contains a blocking wait anywhere, the fix isn't done, no matter how confidently the relocation was reasoned through.

## The question that finds the class

For each piece of shared state, write its invariant explicitly. Then, for every pair of operations that touch it, ask: what interleaving breaks this invariant? Then check whether the same shape exists at the other call sites — a check-then-act bug found in one place typically has three to eight siblings in the same codebase, and fixing only the reported one guarantees the ticket comes back.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Bug disappears under the debugger | breakpoints and single-stepping serialize the threads, closing the window |
| Cannot reproduce after adding logging | the log call's own synchronization created the missing edge by accident |
| "Fixed" but recurs in a month | one call site fixed; the same check-then-act shape remains at the others |
| Only reproduces in production | production has more cores, more load, and a different scheduler than the developer machine |
| Concurrent collection used, updates still lost | each operation is atomic; the sequence of two is not |
| Deadlock only under load | the rare path that takes the locks in the other order needs contention to be visible |
| Thread dump shows nothing wrong | one dump cannot distinguish stuck from busy; three dumps can |
| Race detector output dismissed | it reports missing edges on runs that happen to produce correct output — that is the point |

## Red flags

- "It's probably just a flaky test" as a conclusion rather than a hypothesis
- "That can't happen, the other thread always finishes first"
- Adding a sleep, a retry, or a longer timeout to make a test green
- Concluding it is fixed because it passed 10 times, when the pre-fix rate was 1 in 500
- Reasoning about interleavings without naming the shared state and its invariant
- Treating a race-detector warning on a passing run as noise
- Killing and restarting a slow process without first checking CPU-time-vs-elapsed
- A deadlock fix justified by which thread something will run on, rather than by removing the blocking wait
