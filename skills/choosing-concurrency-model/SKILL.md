---
name: choosing-concurrency-model
description: Use when work must happen at the same time and the approach is undecided — OS threads, async or event loops, actors and message passing, a durable job queue, data parallelism, or sharding by key. Also for thread pool exhaustion, event loop stalls, deadlock, unbounded queue growth, memory blowups under load, tail latency spikes, cancellation that does not stop work, shared mutable state versus message passing, and whether to be concurrent at all.
---

# Choosing a concurrency model

## Overview

Pick from workload shape and failure tolerance, never from language fashion. Most concurrency pain is a model-selection error, and no amount of careful locking repairs a model that does not fit the work.

## When to use

- Starting any component that handles more than one thing at once, or adding a background worker, scheduler, or parallel path.
- Deciding shared mutable state versus message passing — a design decision, not an implementation detail.
- Symptoms: latency high while CPU sits idle, p99 far above p50, pool exhaustion, memory climbing until OOM, deadlock that only appears under production load.
- Not for: diagnosing a specific existing race or deadlock (debugging-concurrency), or setting queue bounds and shedding policy once the model is chosen (rate-limiting-and-backpressure).

## Three questions, in order

1. **What is the work waiting on?** Measure the split between on-CPU time and time blocked in syscalls. Above ~70% waiting, you want concurrency (overlap). Above ~70% on-CPU, you want parallelism or — first — a better algorithm. Concurrency never makes CPU-bound work faster; it makes it slower and harder to debug.
2. **How many units at peak?** Under ~100, almost any model works and you should pick for debuggability. Into the thousands, OS threads start losing to stack reservations (typically 512KB–8MB each) and scheduler overhead. Past ~10k concurrent waits, you need async or a lightweight-task runtime.
3. **Interactive or batch?** Interactive means a latency budget and a real cancellation story. Batch means throughput and the freedom to redo work, which makes queues far more attractive.

## The models

| Model | Genuinely good at | Failure mode | Debugging cost |
|---|---|---|---|
| OS threads + shared state | CPU parallelism, blocking libraries, real stack traces, one simple mental model per thread | Deadlock, torn invariants, races that surface only under contention | Low per-thread, high for races — nondeterministic and unreproducible |
| Async / event loop | 10k+ mostly-waiting connections, cheap tasks, near-deterministic ordering | One CPU-heavy or accidentally-blocking call stalls everything; the system's p99 becomes that call | High — stacks are shredded across suspension points. Attach a task id at spawn or you cannot reconstruct anything |
| Actors / message passing | Isolation, supervision, stateful entities, a path to distribution | Unbounded mailboxes (OOM), and deadlock via request-response cycles between actors | High — causality is spread across mailboxes; needs correlation ids as a first-class concept |
| Durable work queue | Long or retryable work, spiky load, crash survival, independent scaling | Duplicate execution, poison messages | **Lowest** — the unit of work is inspectable and replayable. Chronically underrated |
| Data parallelism (fork-join, map over partitions, SIMD) | Uniform CPU-bound work over a collection | False sharing; one straggler partition sets the wall time; overhead exceeds gain below roughly 10k elements or 1ms per chunk | Low — deterministic when the operation is pure |
| Single-threaded + partition by key | Per-key ordering with no locks, linear scaling to partition count | Hot keys and skew; anything spanning two partitions | Lowest of the concurrent options — each partition is a sequential program |

## Shared state versus message passing

- **Shared state** is right when many workers read data that rarely changes (a config snapshot, a warm cache) and copying is genuinely too expensive. Enforce it with either one lock, or a documented total order over locks, or a read-mostly structure (copy-on-write, immutable snapshot swap, RCU).
- **Message passing** is right when the data has a natural single owner, when it may need to cross a process or machine boundary later, or when you want supervision and restart.
- The hybrid that fails: "just one small lock per struct", acquired in different orders by different call paths. **If you cannot write the lock ordering as a total order on one page, the model is wrong** — you have chosen shared state without the discipline it requires.
- Never hold a lock across a suspension point (await, yield, blocking IO). This is simultaneously the most common async deadlock and the most common tail-latency cliff. If a critical section must span a suspension, use an async-aware primitive and forbid IO inside it.
- State the ownership rule for every piece of mutable state in one sentence. If you cannot, the design is not finished.

## Costs nobody budgets

- **Backpressure.** Every queue needs a bound and a policy when full: block the producer, drop oldest, drop newest, or reject the caller. An unbounded queue is not the absence of backpressure — it is backpressure delivered as an OOM at the worst moment.
- **Cancellation.** It must reach the actual blocking call, not merely set a flag. Ask, per model: when the caller times out at 2s, what happens to the in-flight work? "It keeps running" is a leak, and under load it becomes the outage. Structured concurrency — a scope that cannot outlive its children — deletes this entire bug class; use it wherever the language offers it.
- **Switching cost.** A thread context switch is ~1–10µs; a task switch on an event loop is ~100ns–1µs. That gap only matters above roughly 10k switches per second. Below that, choose for debuggability, not throughput.
- **The debuggability tax.** Budget it up front: a correlation id propagated through every hop, and a way to dump what is currently in flight. Retrofitting either during an incident is not possible.
- **Saturation signals.** Queue depth, wait time, and rejection count per pool. Without them, "it's slow" has no answer.

## When not to be concurrent

- Total work under ~10ms off the hot path. The coordination costs more than the work.
- One shared resource serializes everything anyway — a single disk, a single connection, a global lock, an external rate limit. Parallel callers to a serialized resource add queueing delay and zero throughput.
- Per-item work is below the coordination overhead; the parallel version measures slower and usually does.
- Batching is available. One request carrying 100 items beats 100 concurrent requests on nearly every axis: fewer round trips, one failure to handle, one retry, one place to observe.
- A sequential loop already meets the latency budget. Sequential-and-correct outranks concurrent-and-correct-most-of-the-time.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Latency high, CPU idle, more threads do not help | Serialized on one resource; the queue is somewhere you are not looking |
| p99 is 100× p50 on an event loop | A blocking or CPU-heavy call on the loop |
| Memory grows until OOM under load | Unbounded queue or mailbox; no bound, no policy |
| Deadlock only in production | Lock order is not a total order; contention exposes the path tests never take |
| Timeouts fire but the work continues | Cancellation never reached the blocking call |
| Thread pool exhausted and wedged | Pool threads blocked waiting on work submitted to the same pool |
| Race reproduces only in production | Tests exercise the shared state single-threaded |
| Parallel version slower than sequential | Chunk work below coordination overhead |
| Cannot tell what a request actually did | No correlation id propagated across hops |
| Adding workers made throughput worse | Contention or a downstream limit; the bottleneck moved, not disappeared |

## Red flags

- "Let's make it async, it'll be faster" — about CPU-bound work.
- "We'll add a queue" with no bound named.
- "Just add a lock here."
- "The timeout will handle it."
- Choosing the model because every example in the language's documentation uses it.
- Being unable to name the owner of each piece of mutable state in one sentence.
- Reaching for concurrency before profiling the sequential version.
