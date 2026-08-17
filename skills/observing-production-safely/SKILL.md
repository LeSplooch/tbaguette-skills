---
name: observing-production-safely
description: Use when diagnosing a live system that users depend on, when the only evidence sits behind production data, when enabling a debug flag, verbose logging, a heap dump, a profiler, or a breakpoint against a running service, when tempted to change production state to test a theory, or when extracting dumps, traces, or samples that contain customer data.
---

# Observing production safely

## Overview

Every diagnostic is a change to production. Budget its cost, bound its scope, and write its rollback before enabling it — a meaningful fraction of outages are caused by the investigation into a smaller problem.

## When to use

- The failure cannot be reproduced anywhere else and the evidence only exists live
- About to raise a log level, attach a profiler, take a dump, or flip a debug flag on a running system
- Considering editing a record, replaying a message, or clearing a cache "just to see"
- Extracting logs, dumps, or samples that will contain real user data
- During an incident, when someone proposes a diagnostic under time pressure

Not for: reasoning about the defect once you have the evidence (`diagnosing-before-fixing`), or getting a failure to happen on demand elsewhere (`reproducing-bugs`).

Not for: deciding what should be emitted in the first place — that's `instrumenting-for-observability`, design-time work done before any incident exists. This skill is what's safe to switch on live, mid-incident, against whatever's already there.

## The ladder

Descend in order. Stop at the first rung that answers the question — most investigations are finished by rung 2 and reach for rung 5 out of habit.

1. **Existing telemetry.** Metrics, logs, traces, dashboards already collected. Zero marginal cost, and the only rung that is free.
2. **Read-only queries** against a replica, with a statement timeout and a row limit set before the query is typed.
3. **Sampled diagnostics.** One-in-N tracing, a debug header on a single request id, one canary instance out of the fleet.
4. **Passive process inspection.** Sampling profiler, thread dump, process counters, existing admin or health endpoints.
5. **Stop-the-world capture.** Heap dump, core dump, full snapshot. Drain the instance from the load balancer first, then capture, then decide whether to return it.
6. **Reproduce in a lower environment** with production-shaped data.
7. **Mutating production state.** Normally a hard stop. If genuinely required, it is a change: named owner, change record, announced, and reversible.

A breakpoint on a live service is not on this ladder at any rung. A stopped thread holds its locks, its connections, and its pool slot while health checks decide the instance is dead and the fleet reroutes onto the remaining capacity.

## Blast radius

| Diagnostic | Typical cost | Bound it by |
|---|---|---|
| Debug or verbose logging, fleet-wide | 10–100× log volume; disk fill; pipeline backpressure that drops the very lines you need; synchronous appenders add latency | one instance, TTL under 15 minutes, and check pipeline headroom first |
| Heap dump | pause proportional to heap — seconds to minutes at multi-GB — plus a heap-sized local write | drain first; require free disk ≥ 2× heap |
| Core dump | as above, and it contains every secret the process has in memory | treat the artifact as a credential |
| Sampling CPU profiler | 1–5% at ~99 Hz | safe on one instance; still not fleet-wide by default |
| Instrumenting profiler or dynamic instrumentation on a hot path | 2–50× on the instrumented path | one instance, one narrow method, short window |
| New high-cardinality label, tag, or trace attribute | unbounded series growth that takes down the metrics backend rather than the service | bound the value space before shipping it; user ids and URLs are not labels |
| Query against the primary | lock contention and replication lag | replica, timeout, explicit LIMIT |
| Packet capture | CPU and disk, and it records credentials in plaintext | narrow filter, short duration, encrypted destination |
| Extra shell session on the instance | memory, file descriptors, and CPU on an already-degraded host | prefer a drained instance |

The pattern to recognize: the diagnostics that are cheap on one instance are frequently catastrophic across the fleet, because the shared thing they consume — log pipeline, metrics cardinality, storage backend — is not per-instance.

## Rules that do not bend

- **Read-only until a hypothesis is confirmed.** Mutating state to test a theory destroys the evidence, may corrupt one customer's data, and teaches you nothing about cause even when the symptom stops — you cannot distinguish "I fixed it" from "I overwrote it".
- **One diagnostic at a time**, with the time it was enabled recorded. Two changes at once makes the resulting graph unreadable and the conclusion unearned.
- **Set the expiry when enabling, not afterwards.** Verbose logging: 15 minutes. Sampling changes: 1 hour. A drained instance: until the incident closes. Left-on debug flags and never-returned drained instances are the two artifacts that outlive every incident.
- **Write the rollback command in the same message as the enable command**, before running either.
- **One instance first, always.** A fleet-wide diagnostic is a deploy and deserves a deploy's caution.
- **Announce in the incident channel before, not after.** Someone else is reading the same graphs and will otherwise attribute your log-volume spike to the incident.
- **Prefer the drained instance to the live one.** A single instance taken out of rotation converts almost every rung-5 diagnostic from risky to free.

## Getting evidence out

Anything taken from production is production data until proven otherwise. Heap and core dumps contain credentials, session tokens, decrypted payloads, and complete request bodies — they are not "just memory".

- Before it crosses a boundary: know which regime covers it, put it in the approved store rather than a laptop or a chat attachment, redact or tokenize where the tooling supports it, and set a deletion date at the moment of creation rather than after the analysis.
- Never paste raw log lines, request bodies, or dump excerpts into a ticket, a chat, or a third-party analyzer without checking what regime applies. Attach a pointer to the controlled artifact instead.
- **Prefer aggregates.** A histogram of the field, a count grouped by error class, a hash of the value, a length distribution. Most hypotheses are testable without a single raw record leaving the boundary, and the aggregate is usually the better evidence anyway because it shows the shape.
- Log the access. If the extraction was appropriate, the audit record costs nothing; if it was not, the missing record is the finding.

## Reproducing lower down instead

Copy the shape, not the data. In rough order of how often it turns out to be the variable that matters:

1. Dataset size and the resulting query plan
2. Key skew — one tenant with 10,000× the median row count
3. Concurrency level and connection-pool pressure
4. Latency and failure behavior of the slowest dependency
5. Cache warmth, including a cold-start
6. Clock, timezone, locale, and character encoding

Generate or subset with masking. A synthetic dataset that reproduces the failure is more valuable than the real one: it has no handling obligations, it can live in the repository as a fixture, and it becomes the regression test.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Investigation became the incident | a diagnostic was enabled fleet-wide instead of on one instance |
| Metrics backend fell over during debugging | a high-cardinality label was added on a hot path |
| Logs stopped exactly when the interesting event happened | verbose logging filled the disk or backpressured the pipeline |
| Instance never returned to the pool | drained for a dump, no expiry recorded |
| Debug flag found on months later | no TTL was set when it was enabled |
| Cannot tell what fixed the symptom | three diagnostics and a config change went in together |
| Dump sitting in a chat thread | evidence was extracted with no plan for where it would live |
| "Fixed" by editing a record | mutation destroyed the evidence; the cause remains and will recur |

## Red flags

- "It's just a log level"
- "I'll only run it on prod for a second"
- "Let me change this one row and see what happens"
- "The dump is fine, it's only internal"
- Enabling anything without having typed the command that turns it off
- Attaching a debugger, adding a breakpoint, or running an interactive console against a serving instance
- Reaching for a heap dump before checking whether the dashboard already answers the question
