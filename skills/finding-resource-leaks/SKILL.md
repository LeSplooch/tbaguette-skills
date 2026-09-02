---
name: finding-resource-leaks
description: Use when memory, file descriptors, sockets, threads, connections, timers, or event subscriptions grow without bound, when a process is killed for running out of memory or hits a too-many-open-files error, when a periodic restart is what keeps a service alive, when a heap snapshot must be read, or when usage climbs under steady load, or when a lock, a global flag, or a document-wide style is released by an event or callback that may never fire. Covers growth detection, retention versus allocation, and error-path leaks.
---

# Finding resource leaks

## Overview

A leak is retention, not allocation. The allocation site is merely where the object was born; the defect is the reference that outlives its purpose — which is why reading the allocating code almost never finds it. Detect leaks by measuring growth under steady load, never by inspecting source.

## When to use

- A process is killed for memory, hits a descriptor limit, or exhausts a connection pool
- A scheduled restart, or a "just restart it weekly" note, is holding the system up
- Resource usage climbs across hours or days without a corresponding rise in traffic
- Latency degrades steadily after deploy and recovers on restart
- Duplicate side effects appear, multiplying with uptime — a subscription leak wearing a logic bug's clothes

Not for: cost per operation at fixed load (`performance-profiling`), or a bounded queue backing up under real load (`rate-limiting-and-backpressure`).

## Detect by growth

- Hold offered load constant for at least 4 hours and plot the resource against time. Legitimate use rises then plateaus. A leak rises with a roughly constant slope and never flattens.
- **Measure the floor, not the peak.** The value after a forced collection or an idle period is the live set; peaks are noise. A floor that rises across three or more cycles is a leak, and this single measurement removes most false alarms.
- Quantify per operation: Δresource ÷ operations in the window. "4 KB per request" and "one descriptor per failed connect" are actionable. "Memory grows" is not, and it is what most tickets say.
- Project from the slope: at this rate, when does it hit the limit? A leak that takes longer than the deploy interval is real but low priority. One that hits in six hours is an outage on the first quiet weekend.
- Load must be steady. Growth under rising load is capacity, not a leak, and mistaking the two sends people hunting references that do not exist.

## The resource table

| Resource | Symptom | Where it hides | Measure |
|---|---|---|---|
| Managed heap | out-of-memory kill; GC frequency and duration climbing before the crash | caches, listener registries, thread-locals on pooled threads, static/global maps, closures capturing large scopes | live-set floor after collection |
| Native/off-heap | resident size far exceeds heap; no GC pressure | buffers, memory-mapped files, native library handles, allocator arenas | RSS minus runtime-reported heap |
| File descriptors | "too many open files"; suddenly cannot open anything, including logs | files, sockets, pipes, watches, poll instances not closed on error paths | descriptor count vs. limit |
| Sockets | port exhaustion, connect timeouts, CLOSE_WAIT pileup | CLOSE_WAIT specifically means *your* side never closed | connection count grouped by TCP state |
| Threads/tasks | rising context switches and stack memory; "cannot create thread" | a thread per request, unbounded pools, workers blocked forever | thread count and state histogram |
| Pooled connections | pool exhaustion under load that used to work fine | borrowed and never returned on the exception path | in-use gauge over time |
| Timers/schedules | CPU baseline drifting up; work multiplying | rescheduled on every event, cancelled on none | count of scheduled tasks |
| Subscriptions/listeners | duplicated side effects, N× handling, latency growth | subscribe in a lifecycle hook that runs more than once, with no matching unsubscribe | listener count per emitter |
| Disk/temp files | disk-full outage that also breaks logging | temp files on the error path, unrotated logs, orphaned upload chunks | directory size trend |

## Leak versus not-a-leak

This is where most investigations go wrong, usually by two days.

- **Unbounded cache.** Grows forever and looks identical to a leak. The fix is a size bound and an eviction policy, not a reference hunt. A cache with no maximum and no TTL is a leak with better branding.
- **Allocator and runtime retention.** Many allocators and runtimes never return freed pages to the OS. Resident size plateaus high and stays there while the live set is flat. **Decision rule: if the runtime's own live-set floor is flat and only the OS number grows, stop looking for a reference** — no reference exists to find.
- **Fragmentation.** Live set flat, resident growing, allocation sizes highly variable. Fixed by allocation strategy or arena sizing, never by finding an owner.
- **Deliberate growth.** An in-memory index, a session store, a growing queue. Compare against intent, then check whether it has a bound.
- **Lazy cleanup.** Finalizers, deferred collection, TTL reapers. Wait at least twice the reaper interval before calling anything a leak.

## Reading a heap snapshot

1. **Sort by retained size or dominator, never by instance count or shallow size.** Count and shallow size point at strings and boxed primitives every single time and are essentially never the answer.
2. **Diff two snapshots** separated by a known workload. The delta by retained size names the leak in one step; a single snapshot mostly shows what the program legitimately holds and invites hours of pattern-matching on innocent objects.
3. Follow the **shortest path from a root** to a representative growing instance. The offending reference is on that path, and it is usually the second-to-last edge — the last edge is the container, not the bug.
4. Check root types in this order: static and global fields, thread-locals on pooled threads, registered listeners and callbacks, class loaders and module registries, native handles.
5. For non-GC runtimes, a leak sanitizer or heap profiler gives the allocation stack of blocks still live at exit. Useful, but it names the birth site, so pair it with the ownership question below or you will "fix" the wrong function.

## Ownership across paradigms

Every resource has exactly one owner and a lifetime that is written down at the point of acquisition, not inferred later.

- **Scope-bound release** — destructors, `defer`, `with`, `using`, try-with-resources, bracket. The only mechanism that survives an early return and an exception, which is why it is the default answer wherever the language offers it.
- **Reference counting** — cycles never reach zero. Break the back edge with a weak reference: parent to child strong, child to parent weak, and every observer or callback pointing backwards weak.
- **Tracing GC** — reachability is the entire rule. A listener registered on a long-lived emitter keeps its enclosing object alive forever no matter what scope it was created in. This is the most common managed-heap leak in existence.
- **Manual acquire/release pairs** — every acquire needs a release on every path. An acquire with no matching release in the same function is a review finding, not a style preference.
- **Across an async boundary** — the callback outlives the caller's scope, so scope-based release either runs too early or not at all. Name the owner explicitly and attach the release to a cancellation or completion signal.

## Error paths are where the leaks actually are

The happy path runs millions of times and is almost never the leak. The leak lives on the path that runs 0.1% of the time: a timeout, a validation rejection, a retry, a cancellation, a partial read, a failure between two acquisitions.

- The partial-acquisition shape: acquire A, acquire B fails, return — A leaks. Release in reverse order via a scope guard registered immediately after each acquisition, never in a block at the end of the function.
- Cancellation is the most-missed path of all: the caller went away, and everything acquired on its behalf still needs releasing.
- Test it directly. Fault-inject at each acquisition point and assert the resource count returns to baseline. A loop that opens, fails, and closes 10,000 times while watching the descriptor count is the cheapest leak test that exists and catches the majority of these.

## A release that waits to be told

The error-path shape above is an acquire whose release is *skipped*. This is
the acquire whose release is *never called*, and it hides better, because the
code that releases is right there and looks correct.

The tell is where the release is attached. Acquiring mutates something — a
lock, a global flag, a document-level style, a registry entry, a temp
directory — and the release hangs off a notification that the thing is over: a
`close` event, a completion callback, a finaliser, a lifecycle hook. That works
exactly as long as the notification arrives, and the notification is the part
you do not control. A platform that never fires it for your case, a library
that fires it only on some paths, an early return between the two, a listener
attached after the event already went out — each leaves the mutation standing
with nothing left that would undo it.

It is worse than an ordinary leak in one specific way: the resource is usually
global, so what leaks is not a handle nobody sees but the behaviour of
everything else. A scroll lock never lifted is a whole page that cannot
scroll, long after whatever locked it is gone, and nothing anywhere is
obviously broken.

**Prefer observing the state that actually changed over being told it
changed.** Where the platform exposes the state itself — an attribute, a
property, an observable, a poll — drive the release from that, and the release
becomes correct regardless of which notifications fire. Where it does not,
release at every exit you own *and* keep the handler, because the two together
fail independently. The question that finds these before they ship: **if that
event never arrives, what stays changed?**

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Memory keeps growing" with no number | growth was never quantified per operation, so nobody can say when it matters |
| Fixed the top allocation site, nothing changed | the allocation site was innocent; the retaining reference was elsewhere |
| Chasing a leak that does not exist | measured resident memory instead of the live-set floor; the allocator was holding pages |
| Snapshot analysis stalled on strings and arrays | sorted by count or shallow size instead of retained size |
| Leak reproduces only in production | error paths are rare in tests; fault-inject instead of waiting |
| Added a periodic restart | converted an outage into a recurring one and stopped the investigation |
| Growth returns after the fix | one call site was fixed; the same shape exists at the others |
| Weak references sprayed everywhere | cycle diagnosed by guessing rather than from a root path |

## Red flags

- "We just restart it every night, it's fine"
- Reading allocation code before measuring the growth rate
- Declaring a leak from a single snapshot, or from one peak reading
- Blaming the garbage collector
- Concluding "fixed" from a 15-minute run when the slope needs hours to be visible
- Adding a cache bound and calling the reference leak fixed without re-measuring the floor
