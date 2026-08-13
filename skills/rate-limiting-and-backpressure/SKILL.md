---
name: rate-limiting-and-backpressure
description: Use when a system is receiving more load than it can serve, when designing throttling, quotas, or 429 responses, when a queue keeps growing or drains hours late, when retries amplify a partial failure into a full outage, when a connection or thread pool is exhausted, when latency climbs instead of requests failing, or when choosing between shedding load, queueing it, and slowing the producer down.
---

# Rate limiting and backpressure

## Overview

When demand exceeds capacity, the only question is which work fails and how quickly it finds out. A system with no explicit answer still answers: it queues everything, latency grows without bound, and every caller times out on work that will be completed and then discarded. Choosing the victim deliberately is the entire discipline.

## When to use

- One caller, tenant, or job is degrading the service for everyone
- Designing quotas, throttles, or the response a caller gets when over the limit
- A queue whose depth grows through the day, or that drains long after the traffic ended
- Timeouts, pool exhaustion, or a cascading failure spreading between services
- Retry behavior amplifying a partial failure — error rate rising after a dependency starts recovering
- Not for: capacity planning and scaling decisions; this covers what happens in the window where capacity is already insufficient

## Shed, queue, or slow down

| Response | Trade | Correct when |
|---|---|---|
| Shed — reject immediately | Preserves latency for accepted work, discards the rest | A human is waiting, the work is not durable, or the result expires quickly |
| Queue — accept and defer | Preserves work, spends latency, and hides the overload until the queue is full | Work is durable and asynchronous, and the bound is set from an acceptable wait |
| Slow down — backpressure to the producer | Preserves both, requires a producer that can be slowed | The producer is under your control: an internal caller, a stream, a batch job, a replication feed |

Rule: never accept work you cannot drain faster than it arrives under the worst arrival rate you have actually measured. Queueing is not a capacity increase; it is a loan against latency, repaid by whoever is still waiting when the queue finally drains.

## Algorithms

| Mechanism | Burst behavior | State | Failure mode |
|---|---|---|---|
| Token bucket | Allows a burst up to bucket size, then steady refill rate | Two numbers per key | Burst size chosen without thought lets one caller consume a full window instantly |
| Leaky bucket / GCRA | No burst; output perfectly smooth | Two numbers per key | Legitimate bursty clients are punished; queueing variant hides overload |
| Fixed window counter | Up to 2x the limit across a boundary | One counter per key per window | The boundary spike is real traffic and it will find you |
| Sliding window log | Exact | One timestamp per request | Memory grows with traffic — the limiter becomes the bottleneck |
| Sliding window counter | Approximate, smooth | Two counters per key | Slight inaccuracy at boundaries; the right default for most services |
| Concurrency limit / semaphore | Bounds simultaneous work, not arrival rate | One counter per resource | Slow callers hold slots; requires a timeout on hold time or it deadlocks under a slow dependency |

Rate limits and concurrency limits solve different problems and most systems need both. A rate limit protects fairness between callers; only a concurrency limit protects a finite resource — a connection pool, a thread pool, memory — because the resource is consumed by requests in flight, not by requests per second. If exactly one control is affordable, bound concurrency: it degrades gracefully under a dependency slowdown, while a rate limit calibrated for a healthy dependency admits exactly as much work when everything is already stuck.

## Choosing the dimension

Limit on the dimension that owns the cost: per API key, per tenant, per account, per endpoint class, or per resource. Getting this wrong punishes the wrong caller, and the caller who gets punished is usually the innocent one.

- **Per IP** breaks on NAT, corporate egress, and mobile carriers — one limit shared by thousands of unrelated users — while an attacker rotates addresses freely. Use it only as a pre-authentication backstop.
- **Global only** means the heaviest tenant sets everyone's experience, and the limit is invisible to the caller who triggered it.
- **Per endpoint with equal weights** is wrong whenever endpoints differ in cost by more than about 10x. Charge a weight — a report costing 50 tokens and a lookup costing 1 — so the limit tracks work rather than request count.
- Keep a **global backstop below the sum of per-tenant limits**, because that sum is not your capacity and never was. Every tenant simultaneously behaving within its limit is a normal Tuesday, not a hypothetical.
- Scope limits to what the caller can control. A limit whose denominator includes work triggered by other users produces a caller who cannot comply no matter what they do.

## Backpressure

Backpressure means the constraint reaches the producer, not that the consumer buffers harder. Concretely: bounded queues that reject or block on insert, not accept; stopping reads from the socket rather than draining into memory; withholding acknowledgement of the next message until the current one is done; a semaphore held for the full life of the request, not just its CPU phase; flow-control windows on streams.

The unbounded queue is how load becomes latency instead of visible failure. It absorbs exactly the signal you needed, and it keeps absorbing until every item in it is already past the deadline of whoever submitted it — at which point the system spends 100% of its capacity producing results nobody is waiting for. Two defenses: propagate a deadline with every request, and **discard work whose deadline has passed before starting it**. Dropping dead work at the head of the queue is often the single change that recovers an overloaded system.

Sizing is arithmetic, not intuition. By Little's Law, wait time equals queue length divided by service rate, so pick the maximum acceptable wait and set the bound to service rate × that wait. A queue of 10,000 in front of a consumer serving 100 per second is a 100-second wait, which is a decision someone should have made on purpose.

## Watch age, not just depth

Queue depth alone is ambiguous — 10,000 items is healthy at 5,000 per second and an outage at 5 per second. Alert on **age of the oldest unstarted item**, which is directly comparable to the latency objective. Supporting indicators, all leading rather than lagging: in-use concurrency versus the limit, time spent waiting for a slot, rejections broken down by reason, and the ratio of arrival rate to service rate — sustained above 1.0, everything else is a countdown.

## Retries

- **Exponential backoff with full jitter:** `delay = random(0, min(cap, base × 2^attempt))`. Backoff without jitter re-synchronizes every client that failed together, so they all return at the same instant and rebuild the herd that caused the failure. Jitter is not a refinement of backoff; without it, backoff schedules the next outage.
- **Cap attempts and total elapsed time.** Three attempts within a bounded budget is a reasonable default. The deadline matters more than the count: a retry issued after the caller has given up is pure load.
- **Retry budgets:** permit retries only up to a fraction of successful traffic — 10% is a common ceiling — and fail fast beyond it. This is the control that stops a partial outage from becoming total, because when the dependency is failing, retries triple the offered load precisely when capacity has fallen.
- **Retry only what is safe:** idempotent operations, on retryable classes only — timeouts, connection failures, explicit 429 or 503 with a retry hint. Never retry a rejection caused by the request itself.
- **Retry at one layer.** Three layers retrying three times each is 27x amplification, and each layer looks reasonable in isolation. Choose the layer that owns the deadline and make the others propagate.

## Breakers and shedding

A circuit breaker trips on error *ratio* over a rolling window with a minimum sample count — tripping on one failure out of one is a self-inflicted outage — stays open for a cooldown, then half-opens to a limited probe before restoring. Its purpose is to stop spending capacity and latency on calls that are already known to fail, and to give the dependency room to recover.

Shed by value, not uniformly. Rank traffic and reject in ascending order of worth: bulk and batch work first, then retries, then anonymous and free-tier traffic, then interactive user work; never health checks, authentication refresh, or the control plane that lets you fix the incident. Shed at the edge, before a connection, thread, or transaction has been claimed — rejection after the expensive resource is held provides no relief at all. Uniform shedding is how a background job crowds out the paying user, because that job retries harder and cares less.

## Tell the caller

A limit a client cannot see before hitting it forces every client to discover it by failing. Return a distinct, machine-readable status for "over your quota" versus "we are overloaded" — the first is the caller's to fix, the second is not and should be retried. Include a `Retry-After`-equivalent hint with jitter already applied, and headers or fields naming the limit, the remaining allowance, and the reset time. Document the limit, the window, the dimension, and the burst allowance. Keep the error shape identical to other errors so client code handles it on an existing path rather than a new one.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Latency grows without bound, nothing errors | Unbounded queue absorbing the overload signal |
| Load spikes every 30 seconds after an outage begins | Backoff without jitter synchronizing all clients |
| A dependency recovers and immediately falls over again | No retry budget or breaker; full retry backlog lands at once |
| Pool exhausted though the rate limit was never breached | Rate limited but not concurrency limited; a slow dependency held every slot |
| One tenant degrades everyone despite per-tenant limits | Sum of per-tenant limits exceeds real capacity; no global backstop |
| Legitimate users throttled while abuse continues | Limiting per IP behind NAT while the attacker rotates addresses |
| Queue drains hours after traffic stops | Work with expired deadlines never discarded |
| Rejecting requests does not reduce load | Shedding after the connection, transaction, or thread is already claimed |
| Clients retry into the limit in a tight loop | No retry hint, no remaining-quota signal, limit undiscoverable in advance |

## Red flags

- "We will just make the queue bigger"
- A queue with no maximum size anywhere in the design
- Retries added at a new layer without checking what the layers below already do
- A limit chosen as a round number with no measured capacity behind it
- A breaker that trips on a single failure, or that never half-opens
- Alerting on queue depth but not on queue age
- "The client should not be sending that much" instead of a limit that makes it observable
