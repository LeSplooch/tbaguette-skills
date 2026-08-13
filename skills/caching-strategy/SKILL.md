---
name: caching-strategy
description: Use when adding a cache, memo table, CDN layer, or precomputed read path, when users report stale or wrong data after saving a change, when choosing a TTL or cache key, when an expiring hot key or a cold restart floods the origin, when one tenant or locale sees another's data, or when latency work tempts a cache in front of a slow query.
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

## Is this cacheable at all

All three must hold. Two out of three ships a bug.

| Property | The test | If it fails |
|---|---|---|
| Expensive | Miss path measured, not assumed — at least 10x the cost of a cache lookup | You added a hop, a dependency, and a failure mode for nothing |
| Reused | Projected hit rate ≥80% against the real key distribution, not the average case | Below ~80% the miss path pays lookup plus original work, and every write pays invalidation |
| Stale-tolerant | You can state the budget in seconds and name who is harmed at the boundary | You need a fresh read or a replica, not a cache |

Fix the source first. A cache in front of an unindexed query freezes the bad query in place and hands its full cost to the first request after every flush. Cache what is expensive to compute, not what is accidentally slow.

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

Prerequisite for all three: jitter every TTL by ±10–20%. Fixed TTLs written at deploy time expire together on the same second forever. The same reasoning applies to cold start — a restarted or flushed cache is a simultaneous stampede on every key, so surviving total cache loss at peak traffic is a requirement, not a nice-to-have, and it is worth testing deliberately.

## Negative caching

Cache authoritative "does not exist" answers, or a flood of misses on nonexistent keys becomes an origin denial of service that anyone can trigger by enumerating identifiers.

- Negative TTL far shorter than positive; one tenth is a reasonable starting ratio
- Cache only authoritative negatives. Never cache a timeout, an internal error, or a partially failed dependency — that pins an outage in place after the outage has ended
- Invalidate the negative entry on create, or a just-registered entity is missing for a full TTL and the user retries into the same answer

## Key design

Everything the answer depends on belongs in the key: entity id, tenant, locale, role or permission scope, serialization version, and every flag that changes the computed value. Whatever is omitted becomes a value served to the wrong asker.

- Build keys in one function, never by concatenation at call sites. The multi-tenant leak is never caused by not knowing about tenant scoping; it is caused by one of eleven call sites that forgot it. Make the tenant a required parameter of the key constructor so forgetting it fails to compile or fails a test.
- Prefix keys with a schema epoch. Bumping the epoch invalidates everything atomically, which is the only reliable mass purge and the correct move on any deploy that changes the shape of a cached value.
- Watch cardinality from both directions: keys that are near-unique per request never earn a hit and only consume memory; keys too coarse serve one asker's answer to another.
- Cache the smallest reusable unit. A whole rendered response inherits the shortest staleness budget of any field inside it, and usually that field is a permission or a balance.

## When it is not a cache

If a stale read is a correctness bug rather than a freshness trade, do not build a cache. The tell is someone proposing a TTL of zero, or saying "we will just invalidate it everywhere reliably." Reliable cross-process invalidation is distributed consensus wearing a disguise. Build a replica with a stated consistency guarantee, publish the replication lag as a number, and route the reads that cannot tolerate that lag to the authoritative copy.

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

## Red flags

- "We will just invalidate it everywhere when it changes"
- Nobody in the room can state the staleness budget in seconds
- The TTL was chosen because it seemed reasonable and has never been revisited against an incident
- A correctness argument that depends on the cache being available
- A cache added before the miss path was profiled, or a key assembled inline in a second place
- "It is only cached for five minutes," said about permissions, quota, or a balance
