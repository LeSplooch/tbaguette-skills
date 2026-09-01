---
name: performance-profiling
description: Use when something is slow and the cause is unknown, when an optimization needs proof that it helped, when latency, throughput, tail behavior, p95 or p99 must be characterized, when reading a flame graph, sampling profile, or benchmark result, or when a micro-benchmark reports an impossible speedup. Covers baselines, targets, percentiles, and benchmark artifacts.
---

# Performance profiling

## Overview

The bottleneck is almost never where it feels like it is; experienced engineers guessing at their own code are wrong more often than they are right. Measure the running system, optimize what the measurement names, then measure the whole workload again against a target chosen before any of it started.

## When to use

- Something is reported slow and no measurement exists yet
- An optimization is proposed, in progress, or already merged without a number attached
- Latency, tail behavior, or capacity must be characterized for an SLO or a budget
- Reading a flame graph, sampling profile, or benchmark output
- A benchmark result looks too good to be true

Not for: growth over time rather than cost per operation (`finding-resource-leaks`), or capacity questions that are really about queueing and load shedding (`rate-limiting-and-backpressure`).

## Before touching anything

Write down five things: workload, metric, current value, target value, environment. A missing target makes optimization unbounded, and unbounded optimization is how a two-day task becomes a two-week one.

- The target comes from a requirement, not from "faster". Useful anchors: 16.7 ms per frame at 60 Hz, ~100 ms for an interaction to feel instantaneous, ~1 s before attention drifts, ~10 s before abandonment, and whatever the SLO already promises.
- The baseline is a distribution, not a number. Run ≥ 30 iterations; record p50, p95, p99, and the run-to-run spread. **If the variance between two baseline runs exceeds the improvement you are hunting, fix the measurement before touching the code** — otherwise every result you produce afterward is noise you will interpret as signal.
- Confirm the workload is representative before profiling it. Profiling a fixture optimizes the fixture, and the resulting change frequently makes production slower.

## Latency and throughput pull in opposite directions

| Objective | Improved by | Cost to the other |
|---|---|---|
| Latency (time per operation) | less work per operation, less queueing, less batching, more headroom | lower utilization, worse cost per operation |
| Throughput (operations per second) | batching, pipelining, amortization, higher utilization | queueing delay, and the tail suffers first and worst |

State the objective with a load level attached: "p99 under 200 ms at 70% CPU". A latency target with no load level is unfalsifiable, and a throughput target with no latency bound is met by adding queue depth.

Queueing is the reason most "slow code" reports are capacity reports: waiting time scales roughly as 1/(1−ρ) at utilization ρ, so moving from 70% to 90% utilization roughly triples queue delay with no code change at all. Check utilization before opening a profiler.

## Percentiles

- The mean hides the complaint. One 10-second request among a thousand 20 ms requests moves the mean by 10 ms and is invisible — and it is the entire reason someone filed the ticket.
- Report p50 for what normal looks like, p95 and p99 for what users complain about, max for what pages someone. A mean reported alone is a reporting defect.
- **Percentiles do not average and do not add.** The p99 of a set of p99s is meaningless; aggregate from source histograms. Nor is end-to-end p99 the sum of per-hop p99s.
- Fan-out amplifies the tail: a request touching n backends in parallel waits on roughly the (1 − 1/n) quantile of backend latency. At n = 100, ordinary p99 backend behavior becomes the *typical* request. Tail latency work is most valuable exactly where fan-out is highest.
- Sample count sets the floor on what you can claim: ≥ 1,000 samples for a stable p99, ≥ 10,000 for p99.9. A p99 computed from 100 samples is one data point wearing a statistic's name.

## Reading a profile

1. **Separate CPU time from wall time first.** If wall ≫ CPU, the answer is off-CPU: locks, I/O, GC pauses, scheduler delay, dependency latency. Running a CPU profiler on an I/O-bound service is the single most common wasted profiling session, and it produces a flame graph that looks meaningful.
2. Use a sampling profiler to ask "where does the time go"; use an instrumenting profiler only on a region already under suspicion, because its overhead reshapes the answer.
3. **Flame graph: width is total time, and the x-axis is not time.** Read for the widest plateau near the top of a tower — that is where time is spent, as opposed to merely passing through. The tall spike everyone points at is usually depth, not cost.
4. Four shapes and their meanings: one dominant leaf (optimize that function); a broad flat spread with no plateau over ~5% (the architecture is the cost, not any function); a deep repeating tower (recursion or excessive layering); a wide framework base (per-call overhead × call count — reduce the count, not the per-call cost).
5. Differential before/after graphs answer "what changed" far faster than comparing two absolute graphs by eye.

Where flame graphs mislead: they average over a mix of operation types (profile one operation at a time); they miss work shorter than the sampling interval; inlined frames are attributed to their caller; sampling can be biased toward points where the runtime can safely interrupt; and off-CPU time is absent entirely unless it was explicitly requested.

## Amdahl's check

Before optimizing anything, measure the fraction *f* of total time it occupies. Maximum possible speedup is 1/(1−f). At f = 0.20, eliminating it completely yields 1.25× — so if the target is 2×, this work cannot reach it no matter how well it is done. One line of arithmetic, done first, kills roughly half of all proposed optimizations, including most of the ones that feel most satisfying to write.

## Benchmark traps

| Trap | Signature | Fix |
|---|---|---|
| No warm-up | first iterations 10–100× slower than later ones | discard warm-up iterations, report steady state, and report cold-start separately if it matters |
| Dead-code elimination | impossible numbers: 0 ns, billions of ops/sec, time independent of input size | consume the result through a sink the optimizer cannot see through |
| Constant folding | inputs known at compile time | generate inputs outside the timed region |
| Caching between iterations | run 2 is 100× faster than run 1 | clear caches, or measure the cached case deliberately and label it as such |
| Debug build, assertions, coverage, or a debugger attached | uniform 2–20× slowdown | measure optimized builds with production flags |
| Unrepresentative data | uniform keys where production is skewed; a dataset that fits in cache | match size, skew, and cardinality |
| Timer resolution | measuring below the clock tick | loop and divide |
| Machine noise | frequency scaling, thermal throttling, noisy neighbors, background indexing | pin frequency, isolate cores, and interleave A/B runs rather than all-A then all-B |
| Single-threaded micro-benchmark for a contended path | measures the uncontended case only | benchmark at production concurrency |

Interleaving A and B runs is the cheapest defense available and almost nobody does it: it converts a slow machine drift into noise instead of a fake win.

The next rung up is to stop comparing averages at all. Give both variants the
**same inputs**, in the same order, and difference them per input rather than in
aggregate: whatever made input 47 slow made it slow for both arms, so
subtracting cancels it and leaves the effect of the change. It is nearly free —
the same generator, the same seed, fed twice — and on a noisy workload it is
often the difference between a result and a shrug.

How much it buys is not a property of the technique, though, and reporting the
variance reduction alone reads as if it were. It is a property of the **pair
being compared**, and the quantity that says so is the correlation between the
two arms' per-input results. Report it beside the reduction. When it is low the
pairing is barely working, and there is usually one reason: the change altered
*which work happens* rather than only how fast the work went, so the two arms
met the same input very differently and most of that input's inherent difficulty
was never common to both and had nothing to cancel against. Pairing pays well
for a change that alters outcomes and much less for one that alters the path.

## Reliable is not large

Once A and B are being compared properly, the rule that decides which one wins
becomes its own defect. A significance test — paired, bootstrapped, whatever the
tooling offers — answers whether a difference is **consistent**, and consistency
is not size. A variant that is better by one percent on every single trial has a
tiny difference and a tinier spread, so the ratio between them is enormous and
clears any threshold trivially. The criterion is behaving exactly as designed
and adopting changes that will never be observable.

It compounds where the decision is automated — an autotuning loop, a regression
gate, an alert threshold, a rollout that promotes on a p-value. Every pass
accretes another change that is real and pointless, and complexity is permanent
while a one-percent win is not.

So pair every significance threshold with a **minimum effect size, written in
the units of the thing being decided**: milliseconds at p95, bytes, queries,
dollars. Both must clear, or the change does not land.

There is a diagnostic for this that needs no statistics at all, and it is worth
running on any loop that has been making decisions for a while: **run the whole
procedure twice and see which decisions move.** The parameters that come out
different between two runs of identical code are precisely the ones whose
measured effect is too small to survive noise — while anything with a decisive
effect lands identically both times. A list of the settings that wander is a
list of the settings the criterion was never really deciding.

## Stopping

- Stop when the target is met. Re-measure the **whole** workload — a component that got 10× faster while the end-to-end metric did not move is not a win, and this is the most common way a week of profiling work ends up worthless.
- Record the new baseline and add a regression guard with a threshold wide enough not to flake: typically 10–20% above the measured p95.
- Revert optimizations that met no target. Complexity is a permanent cost and the measurement was its only justification.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Optimized the wrong function | intuition was used instead of a profile, and intuition loses more often than it wins |
| Big local win, no end-to-end change | the optimized region was a small fraction of total time; Amdahl's check was never done |
| Improvement not reproducible later | the baseline was a single run and the machine drifted between measurements |
| Profile shows almost nothing interesting | the workload is off-CPU and a CPU profiler was used |
| Faster in the benchmark, slower in production | fixture data had different size, skew, or cache behavior than real traffic |
| Users still complain after the mean improved | the complaint was the tail; the mean is the wrong statistic for it |
| Latency degraded after a throughput optimization | batching and higher utilization were traded for queueing delay, on purpose, without saying so |
| Optimization work never ends | no target was written down before starting |
| A tuning loop keeps adopting changes and the end-to-end number never moves | the accept rule tested whether a difference was consistent, never whether it was big enough to matter |

## Red flags

- "This loop is obviously the bottleneck"
- Optimizing before a baseline exists, or "we'll measure after"
- A speedup reported from a single run of each variant
- A benchmark result that is a suspiciously round multiple, or that does not vary with input size
- A variance reduction quoted as a property of the method, with no correlation between the two arms beside it
- Reading a CPU flame graph for a service that spends its time waiting
- Continuing to optimize after the target was met
- An accept-or-reject rule stated only as a p-value or a confidence level, with no minimum effect size beside it
