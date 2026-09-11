---
name: caching-strategy
description: Use when adding a cache, memo table, CDN layer, or precomputed read path, when users report stale or wrong data after saving a change, when choosing a TTL or cache key, when an expiring hot key or a cold restart floods the origin, when one tenant or locale sees another's data, when deciding whether a TTL should be chosen by the consumer or declared by each response, or when latency work tempts a cache in front of a slow query. Also use when a cached or memoized value has to come out the same on two machines, when a key is derived from an object's identity rather than its content, or when a hit rate stays low and the requests are routed or load-balanced before they reach the cache. Also use when reuse is decided by how long a leading run of the material matches rather than by a whole key, or when a cache whose inputs barely change is missing far more often than that should allow.
---

# Caching strategy

## Overview

A cache is a second copy of the truth that is permitted to be wrong. You are buying latency with correctness, and the only real design work is deciding how wrong, for how long, and who is harmed when it happens. "Cached" without a number attached is undefined behavior with a hit-rate graph.

## When to use

- Adding a cache, memoization layer, edge cache, or precomputed read path, or choosing its TTL, key shape, and invalidation mechanism
- Stale-data reports: saved a change, still see the old value
- The origin collapses when a hot key expires, a node restarts cold, or a deploy flushes everything
- One tenant, locale, or permission level is served another's data
- Not for: a derived store that must always agree with its source — that is replication, not caching, and the last section explains why the distinction decides the design
- Not for: an origin that is overloaded by demand rather than by repeated identical work — a cache is the wrong lever there, and `rate-limiting-and-backpressure` owns shedding, queueing, and slowing the producer down. The two meet at the stampede: caching decides how many callers miss at once, backpressure decides what happens to the ones who do

## Is this cacheable at all

All three must hold. Two out of three ships a bug.

| Property | The test | If it fails |
|---|---|---|
| Expensive | Miss path measured, not assumed — at least 10x the cost of a cache lookup | You added a hop, a dependency, and a failure mode for nothing |
| Reused | Projected hit rate ≥80% against the real key distribution, not the average case | Below ~80% the miss path pays lookup plus original work, and every write pays invalidation |
| Stale-tolerant | You can state the budget in seconds and name who is harmed at the boundary | You need a fresh read or a replica, not a cache |

Fix the source first. A cache in front of an unindexed query freezes the bad query in place and hands its full cost to the first request after every flush. Cache what is expensive to compute, not what is accidentally slow — and "expensive" is a measurement, so the miss path goes through `performance-profiling` before it goes behind a cache.

## Name the staleness budget in time

Write it next to the cache, in seconds, with the consequence of exceeding it.

| Data | Typical budget | Note |
|---|---|---|
| Immutable or content-addressed artifacts | unbounded | Version in the key; there is nothing to invalidate |
| Reference data, catalogs, currency and geo tables | hours–days | Long TTL plus an explicit purge on publish |
| Config and feature state | 5–60s | Past a minute, your mitigation is slower than your incident |
| Aggregates, counts, rankings | 10s–5min | Show "as of" and the staleness becomes a feature instead of a bug report |
| A user's own writes | 0 | Read-your-writes: bypass on the write path or update in place |
| Authorization, quota, balance, inventory decisions | 0 | Cache the inputs if you must; never cache the decision |

Two clocks matter and they differ: how stale the data may be for anyone, and how long an actor may fail to see their own change. The second is always shorter and is the one users report.

## Let the response say how fresh it is

A TTL chosen by the caller is a guess about data the caller does not own. The
side that produced the value knows things the consumer cannot: whether this
particular result is a stable catalog or a volatile counter, whether a write
just landed, whether it is serving a fallback it would rather you did not keep.
One TTL applied to every response from an endpoint has to be short enough for
its most volatile result, which means the stable ones are re-fetched constantly
for no reason.

So let each response carry its own freshness, and have the consumer honour it
with a local ceiling rather than replace it. Two rules keep that safe:

- **A per-response directive can only shorten, never extend, what the consumer
  was willing to hold.** Otherwise one bad value pins itself in every cache
  downstream, and the only cure is a deploy.
- **Absence is not zero.** A response that says nothing about its freshness
  gets the consumer's default, not "do not cache" and not "cache forever" —
  both of those turn a silent omission into a production incident.

This does not replace anything below. Invalidation, stampede control, and key
design all behave the same; what changes is that the number is now produced by
the side with the information rather than assumed by the side without it.

## Invalidation

| Strategy | Cost | Failure mode |
|---|---|---|
| TTL expiry | Trivial, no coupling to writers | Every value is silently wrong for up to one TTL; synchronized expiry causes stampedes |
| Write-through | Write latency, and writers must know about the cache | Any writer that bypasses the path — a migration, an admin tool, a second service — leaves the entry wrong forever |
| Write-behind | Fastest writes, absorbs bursts | Loss window on crash, and ordering inversions when two writes race |
| Explicit purge | Precise, budget can be near zero | You must enumerate every derived key an entity touches; the one nobody remembered is the classic stale bug, and purge is a distributed operation that fails silently |
| Versioned / immutable keys | One indirection to read the current version | Nothing to invalidate: write a new key, flip a pointer. Old entries linger until evicted, so size for it |

Default to TTL plus versioned keys, and add explicit purge only where the budget is shorter than the TTL you can afford. Treat every purge as best-effort: the TTL must remain a correct-if-slow backstop, because purges get dropped.

On write, replace rather than delete where you can compute the new value. Delete-then-recompute leaves a window in which a concurrent reader repopulates the entry with the pre-write value it already fetched — a stale entry with a fresh timestamp, which is the hardest kind to diagnose.

## Stampede and thundering herd

One popular key expires, every concurrent request misses, and the origin takes the full fan-in at once. Three standard mitigations — use at least two:

1. **Single-flight** — one loader per key; concurrent callers wait on its result. Bounds origin load at one request per key regardless of demand.
2. **Serve stale while revalidating** — return the expired value immediately and refresh in the background. Requires a second, longer hard expiry, or stale becomes permanent whenever the refresh path is broken.
3. **Early probabilistic refresh** — as expiry approaches, a random and rising fraction of readers refresh early, so the crowd never converges on one instant.

All three bound how much work reaches the origin; none of them bounds what the origin does once it is already behind, which is `rate-limiting-and-backpressure`'s half of the same incident. Prerequisite for all three: jitter every TTL by ±10–20%. Fixed TTLs written at deploy time expire together on the same second forever. The same reasoning applies to cold start — a restarted or flushed cache is a simultaneous stampede on every key, so surviving total cache loss at peak traffic is a requirement, not a nice-to-have, and it is worth testing deliberately.

## Negative caching

Cache authoritative "does not exist" answers, or a flood of misses on nonexistent keys becomes an origin denial of service that anyone can trigger by enumerating identifiers.

- Negative TTL far shorter than positive; one tenth is a reasonable starting ratio
- Cache only authoritative negatives. Never cache a timeout, an internal error, or a partially failed dependency — that pins an outage in place after the outage has ended. This is why the distinction `modeling-errors` draws between "the answer is no" and "I could not get an answer" has to survive into the cache layer: collapse the two and the second gets stored as the first
- Invalidate the negative entry on create, or a just-registered entity is missing for a full TTL and the user retries into the same answer

## Key design

Everything the answer depends on belongs in the key: entity id, tenant, locale, role or permission scope, serialization version, and every flag that changes the computed value. Whatever is omitted becomes a value served to the wrong asker.

- Build keys in one function, never by concatenation at call sites. The multi-tenant leak is never caused by not knowing about tenant scoping; it is caused by one of eleven call sites that forgot it. Make the tenant a required parameter of the key constructor so forgetting it fails to compile or fails a test.
- Prefix keys with a schema epoch. Bumping the epoch invalidates everything atomically, which is the only reliable mass purge and the correct move on any deploy that changes the shape of a cached value. A cache is a place a retired serialization shape outlives the code that wrote it, so it belongs in the same inventory `schema-evolution` keeps of readers you cannot deploy in lockstep.
- Watch cardinality from both directions: keys that are near-unique per request never earn a hit and only consume memory; keys too coarse serve one asker's answer to another.
- Cache the smallest reusable unit. A whole rendered response inherits the shortest staleness budget of any field inside it, and usually that field is a permission or a balance.

## Where reuse is decided by a prefix, order the material by volatility

Everything above treats a cached value as atomic: it matches or it does not. A second family of caches does not work that way. Where reuse is decided by **how long a leading run of the material is identical** — a prefix match rather than an equality check on a whole key — the arrangement of the material inside a single entry becomes a cost decision, and it is one nobody makes on purpose.

The rule that falls out is counter-intuitive enough to be worth stating flatly: **a volatile element is not charged for its own size. It is charged for everything stable that sits behind it.** A timestamp, a working directory, a branch name, a request id — a few dozen bytes, each of them different on every call — placed at the front of a large stable body means the match ends immediately and the entire body is recomputed or re-sent every time. Move those same bytes to the end and the body is reused. The payload did not change size; what it costs did, and by far more than its own bytes could account for.

The diagnostic follows from it and is not the obvious one. When reuse underperforms, the instinct is to ask how *much* of the material changes between calls, and the answer is usually "almost none", which makes the poor hit rate look inexplicable. The question that resolves it is **how early does the first difference occur** — because everything after that point is lost regardless of how stable it is.

Which is also why the two intuitive remedies buy nothing while looking like progress. Extending how long entries are retained treats a placement problem as a lifetime problem: the entry was not evicted, it was never matched past its third line. And splitting the material so that a stable head and a stable tail sandwich the volatile part still leaves the tail behind the first difference, so it buys nothing while costing a restructure.

So when a preamble, prompt, header block, or any shared leading material is assembled from several sources, **sort those sources by how often each one changes, most stable first**, and put anything per-invocation at the very end. Where a boundary can be declared explicitly, declare it after the stable material rather than wherever the code happens to be assembling things.

## Who else computes this key, and how did the request get here

Key design above settles what the key must contain. Two questions it does not touch are decided entirely outside the cache, and both fail without ever looking like a cache problem.

**The first is who else computes the key.** A key derived from a property of one object in one process — an identity hash, a memory address, a pointer, an iteration order, a per-run salt — is perfectly stable everywhere you are likely to test it, and different on every other machine. That is harmless while the cache is a private speedup. It becomes a correctness bug the moment two processes are under a contract to *agree*: replicas rendering the same view, a deterministic simulation, a shared or content-addressed store, two clients that must produce identical bytes. Derive the key from the content that determines the answer, never from the object that happens to be holding it.

Accept what that trades, and note that the argument is not the usual one about collision rates. Both key schemes collide: a content hash on two inputs that hash alike, an identity key on a reused address or a recycled handle after the original was freed. What differs is *who* collides. A content hash's collisions are shared — every process computing it collides on the same pair and draws the same wrong thing — while an identity key's collisions are private to one process, on top of already disagreeing on every value it gets right. **Under an agreement contract a consistently wrong answer is detectable and recoverable, and an inconsistently right one is neither.** Where the contract is agreement rather than accuracy, choose the failure mode that stays symmetric.

**The second is how the request arrived.** A cache sitting behind a chooser — a load balancer, a scheduler, a router picking a backend by cost, latency, or throughput — only ever hits when consecutive equivalent requests land on the same choice. A chooser optimizing its own dimension scatters them by design, and the locality the cache depends on is destroyed by a component that does not know the cache exists. This one is expensive to find because of where it presents: hit rate is the number that moves, so it reads as a cache-tuning problem, and every lever available inside the cache — bigger, longer, warmer — is the wrong one. Before tuning a disappointing hit rate, ask whether anything between the caller and the cache is free to send an identical request somewhere else. Affinity is the fix, and it belongs to the router rather than to the cache.

## When it is not a cache

If a stale read is a correctness bug rather than a freshness trade, do not build a cache. The tell is someone proposing a TTL of zero, or saying "we will just invalidate it everywhere reliably." Reliable cross-process invalidation is distributed consensus wearing a disguise. Build a replica with a stated consistency guarantee, publish the replication lag as a number — an emitted signal, on `instrumenting-for-observability`'s terms, not a figure someone can look up during an incident — and route the reads that cannot tolerate that lag to the authoritative copy.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Value stale long after the write | A writer bypasses the write-through path — migration, admin tool, or another service |
| Origin CPU spikes on a regular cadence | Un-jittered TTLs, all set at deploy time, expiring on the same second |
| Cache restart takes production down | Full traffic on the miss path was never a tested state |
| Hit rate high, latency unchanged | The cheap part got cached; the expensive call still runs before or after the hit |
| One user sees another's data | Key assembled inline at a call site, missing tenant or role scope |
| Garbage or crashes right after deploy | Serialized shape changed with no key epoch bump; old bytes decoded by new code |
| "Not found" persists after creation | Negative entry never invalidated on write |
| Outage continues after the dependency recovers | An error response was cached |
| Memory grows without bound | Per-request-unique keys, or no eviction policy and no size ceiling |
| Two machines render the same input differently | The key was derived from a process-local identity, so no two processes agree on it |
| Hit rate stays low no matter how the cache is tuned | A router or scheduler upstream scatters equivalent requests; the locality was destroyed before the cache saw them |
| Inputs barely change between calls and the hit rate is still poor | Reuse is decided by a leading match, and a volatile element sits in front of the stable material |

## Red flags

- Tuning how long entries are retained, on a cache that is missing rather than evicting.
- "We will just invalidate it everywhere when it changes"
- Nobody in the room can state the staleness budget in seconds
- The TTL was chosen because it seemed reasonable and has never been revisited against an incident
- A correctness argument that depends on the cache being available
- A cache added before the miss path was profiled, or a key assembled inline in a second place
- "It is only cached for five minutes," said about permissions, quota, or a balance
- A key built from something process-local, in a cache two processes are expected to agree on
- A hit-rate investigation that has never looked upstream of the cache at what chose the backend
