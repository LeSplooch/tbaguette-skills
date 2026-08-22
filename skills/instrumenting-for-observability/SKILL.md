---
name: instrumenting-for-observability
description: Use when deciding what to log, measure, or trace, when an incident could not be explained from the telemetry that existed, when a metrics or logging bill spikes from label cardinality, when defining an alert, SLI, or SLO, when a request or correlation id is lost across a queue or async boundary, when logs are unstructured formatted strings, or when choosing a log level.
---

# Instrumenting for observability

## Overview

You choose what you will be able to see during an incident before the incident, and no amount of urgency later changes that. Instrumentation added while an outage is running explains the next occurrence, never this one. The bar is not "we have logging" — it is that a specific question you have not thought of yet can be answered in minutes from what is already being emitted.

## When to use

- Adding logging, metrics, spans, or events to new or existing code
- A postmortem where the honest answer to "why" is "we could not tell"
- Designing an alert, an SLI, or an SLO, or being asked what to alert on
- A telemetry bill jumping without a traffic increase
- A request that crosses a queue, job, or worker boundary and cannot be followed through it
- Not for: reading and interpreting an existing failure signal such as a stack trace — this skill is about what gets emitted in the first place
- Not for: safely enabling diagnostics against a live incident with what's already emitted — that's `observing-production-safely`; this skill is the design-time work done before any incident exists

## Three signals, three jobs

| Signal | Answers | Strength | Cannot do |
|---|---|---|---|
| Metrics | "Is it healthy, and is it worse than yesterday" | Cheap per event, always on, aggregatable, the only sane alerting substrate | Explain a single request; carry an id |
| Logs | "What happened to this one thing" | Arbitrary detail, high cardinality, exact values | Serve as an alerting substrate at volume; be cheap |
| Traces | "Where did the time go, and what caused what across boundaries" | Causality and latency attribution across services | Be complete — sampling means the interesting request may be missing |

Alert on metrics, explain with logs, attribute with traces. Alerting on a log search means the metric that should exist does not. If a trace is sampled, keep the trace id in the log line so an unsampled request can still be reconstructed, and always sample errors and slow requests at 100% regardless of the base rate.

## Emit events, not sentences

One structured event per unit of work, with fields, beats many formatted strings. A formatted string is data that has been destroyed and must be reconstructed by a regular expression written under pressure at 3 a.m.

Every event carries: timestamp with timezone, level, service and version, correlation ids, the operation, the outcome, the duration, and the error class on failure. Prefer one wide event at the end of a unit of work — carrying everything learned along the way — over five narrow lines that must be joined afterwards. Values go in fields, never interpolated into a message, so the message stays a stable, groupable constant.

## Cardinality is the bill

The cost of a metric is its number of distinct label combinations, and that number is the product of each label's distinct values, per instance. Two labels of 50 values each is 2,500 series; add a third with 1,000 and it is 2.5 million.

- **Never label a metric with** user id, request id, session id, email, trace id, full URL path containing identifiers, error message text, or any value derived from user input. These are unbounded by definition; the cardinality explosion arrives with the first crawler or the first fuzzer.
- **Do label with** low-cardinality enums the code itself controls: outcome, error class, endpoint template (`/users/{id}`, not the concrete path), method, region, tenant tier. The single highest-value label in existence is a bounded `reason` on a rejection or failure counter.
- Budget roughly 1,000 series per metric per instance as the point where a design needs justification. Route the high-cardinality dimensions to logs and traces, where per-event storage is the model and cardinality costs nothing extra.
- A metric that cannot be aggregated across instances (unbounded labels, or a gauge only meaningful on one host) is a log line wearing a metric's costume.

## Correlate across every boundary

Generate an id at the outermost entry point, propagate it everywhere, and log it on every event. Boundaries that must carry it: inbound and outbound requests, message headers or payload metadata, scheduled and background job arguments, retry attempts, and — where supported — a comment or tag on the database statement so a slow query maps back to a request.

Asynchronous boundaries are where propagation is silently lost. A queue consumer that starts a fresh trace has severed the causal chain exactly where the interesting causality was: carry the id in the message and link the consumer's span to the producer's. Keep both a *request* id and a longer-lived *causation* or *job* id when work fans out, so one user action that spawns forty jobs is still one searchable unit.

Surface the correlation id to the user in error output. A support ticket containing the id is the cheapest debugging tool that exists, and it costs one field in an error response.

## Instrument boundaries and decisions

**Boundaries:** inbound request, outbound call, queue publish and consume, storage access, cache lookup, lock acquisition, and process lifecycle. For each: duration, outcome, size where relevant, and retry count.

**Decisions:** which branch was taken and why. Every rejection, fallback, downgrade, deduplication, skipped record, retry, circuit-breaker trip, and flag evaluation deserves a counter with a `reason` label and — on the unusual paths — an event. "The request was rejected" answers nothing; "rejected, reason=quota_exceeded, tenant_tier=free" ends the investigation.

**Not loop bodies.** Logging per iteration is both a performance defect and a signal-to-noise catastrophe; emit one summary event with counts and a histogram instead. Keep production INFO volume bounded per unit of work — roughly 1–5 events per request, never per row.

## Levels that mean something

| Level | Rule | Test |
|---|---|---|
| ERROR | A human must act, and the action is knowable from the event | If nobody is expected to act, it is not ERROR |
| WARN | Degraded, retried, or self-healed; worth a trend and a threshold, not a page | If it fires steadily forever, it is INFO with a counter |
| INFO | State transitions and unit-of-work outcomes, at a volume you would pay for | If it is per-row or per-iteration, it is DEBUG or an aggregate |
| DEBUG | Off in production by default, enableable per component or per request without a redeploy | If you cannot turn it on for one tenant, it will never be used in an incident |

Handled-and-recovered is not ERROR. An error logged and then rethrown is logged twice or more; log at the boundary that decides the outcome, and attach context at every layer beneath it by wrapping rather than by logging.

## Never log

Credentials, tokens, cookies, authorization headers, keys, payment data, government identifiers, full request or response bodies containing user content, and any personal data beyond an opaque id. Redact at the emitter with an allowlist of fields safe to print — a denylist of sensitive-looking names always misses one, and by then it is in an index with 90-day retention and broad read access. A log store is a data store: retention, access control, and deletion obligations apply to it exactly as they do to the database.

## SLIs from the user's experience

Measure where the user feels it — at the edge or in the client — not where it is convenient to instrument. An SLI is a ratio of good events to valid events, and both halves need a written definition before the alert exists.

- Latency as percentiles with the count beside them (p50, p95, p99). A mean hides the tail, and the tail is the experience being complained about.
- Availability as successful requests over valid requests. Settle the denominator argument up front: a client's own malformed input is usually excluded; your rate limits, timeouts, and capacity rejections are included, because from the user's side those are your failure.
- The same denominator discipline covers grouped reports: records that legitimately lack the group-by field get their own explicit bucket instead of being filtered out. The null-guard that removes noise is the same clause that removes a whole legitimate category — a stage that never sets the field, a client version that predates the field — and the filtered result still renders as complete: an empty report reads as "nothing has happened yet," and nothing visible distinguishes it from "the query excluded the only populated category." Validate a new aggregate against data where the field is absent, not only where it is populated.
- Alert on symptoms the user can feel, not on causes. CPU, memory, and queue depth are diagnostic panels; the page fires on error ratio and latency breaching the objective, with a burn rate that gives a human time to respond.
- Every incident review ends with one question: what field, counter, or span would have made this obvious in the first minute? Add it before the review is closed, or it will be missing again.

Deciding what to emit is upstream of making it safe to emit; `redacting-sensitive-output` covers the second half.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The incident could not be explained afterwards | Only inputs and crashes were instrumented; the decisions between them were not |
| Telemetry bill spikes with flat traffic | An unbounded label — a path with identifiers, an error string, a user id |
| A request cannot be followed past a queue | Correlation id not carried in the message; consumer starts a new trace |
| Alerts fire constantly and are muted | Alerting on causes and on WARN instead of on user-visible symptoms |
| Logs exist but the answer needs a new regular expression | Values interpolated into message strings instead of emitted as fields |
| One failure produces twelve log lines | Logged at every layer on the way up instead of once at the deciding boundary |
| Dashboards look healthy during a customer outage | Measured at the server, not at the edge; means instead of percentiles |
| A report is empty — or one category short — while matching records exist | A null-guard on an optional field excluded the records that legitimately lack it |
| Secret found in the log index | Redaction by denylist, or a whole object logged for convenience |
| DEBUG is useless because enabling it needs a deploy | Log level fixed at build time rather than adjustable at runtime |

## Red flags

- "We will add logging if it happens again"
- A new failure branch that increments nothing and emits nothing
- An alert whose runbook is "look at the graphs"
- Any metric label whose value comes from user input
- "Just log the whole object" as a debugging convenience
- A trace that stops at the queue boundary
- An SLI chosen because it was already being measured
- A report that filters out the records lacking its group-by field instead of bucketing them
