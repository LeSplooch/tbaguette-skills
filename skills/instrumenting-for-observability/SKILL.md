---
name: instrumenting-for-observability
description: Use when deciding what to log, measure, or trace, when an incident could not be explained from the telemetry that existed, when a metrics or logging bill spikes from label cardinality, when defining an alert, SLI, or SLO, when a request or correlation id is lost across a queue or async boundary, when logs are unstructured formatted strings, when a failure counter has never once incremented, when a filter, gate, or safety rule rejects everything it sees and nothing distinguishes a strict rule from an input that never arrived, or when choosing a log level.
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

## Measure what a component consumed, not that it ran

Almost every counter answers *did this execute*. Very few answer *did it have anything to work with*, and those two diverge silently in one whole family of components: anything that learns, tunes, calibrates, adapts, scores, or ranks against accumulated outcomes.

A component like that reads a ground-truth source — realized results, labelled outcomes, completed actions. When the source is empty, because it was never populated or because the path that fills it has never once fired, the component does not fail. **Aggregates over an empty set are well defined.** Sums are zero, means return whatever the guard returns, a fitness function produces defined values over no candidates, and a generation counter advances on schedule. So the thing runs, emits its metrics, advances its bookkeeping, and looks productive from every dashboard and log line in the system. This is the same emptiness that makes a test assertion vacuous in `writing-the-failing-test-first` — an empty collection makes a computation succeed loudly rather than fail — arriving here as a metric instead of a green test.

The output is where the damage is. On an empty sample the component degrades to its prior, and priors are chosen to look *neutral* rather than to look *absent* — a middling score, a default weight, an even distribution. So the failure presents as a confident, plausible, unremarkable number rather than as an error, and nothing anywhere distinguishes "calibrated from zero observations" from "calibrated from ten thousand."

Two things follow, and both are instrumentation decisions rather than modelling ones:

- **Emit input volume and input age as signals in their own right**, next to the activity counter and never folded into it. "Ran 4,000 times" and "had 0 rows to read" are both true, and only the second one is news.
- **Carry the sample size to the point of use.** A consumer that receives a score without an *n* cannot tell a measurement from a default, and will render it identically either way — which is how a number nobody measured ends up in a UI presented as a learned value.

Then let the component refuse. A floor below which it declines to run and says why is better than fitting noise, and it converts a silent wrong answer into a visible abstention. Distinguishing the two reasons for an empty input matters as well: *no data yet* is a legitimate cold start that should idle and announce itself, while *the source is broken* should alarm. A volume counter that never leaves zero past a startup window is the discriminator between them, and it only exists if someone emitted it.

**The same emptiness turns a gate into a permanent silent refusal.** The section above is about a component that *computes*; the harder case is one that *decides*. A rule treating missing data as disqualifying is correct in isolation, and refusing on absence is often the right call — but if one of its inputs has no source in the environment it was deployed to, it does not reject some things. It rejects everything, always, for absence rather than for any measurement. Nothing errors, nothing is logged, and there is nothing to investigate, because the output is an empty stream and that is exactly what a strict rule in a quiet period looks like from outside. Unlike a cold start it cannot resolve by waiting, so it can run in that state indefinitely.

The instrumentation that separates them is one field. A gate has to record **which clause rejected**, and the clauses have to fall into two buckets that are never summed: rejected because something was measured and failed, versus rejected because nothing was measured. The first is the gate working. The second is a data-supply failure wearing the gate's uniform, and it becomes visible the moment it accounts for every rejection and keeps doing so.

Write the alarm on that narrowly or it will be trained away. A strict rule can legitimately refuse everything it sees for a long stretch, and an alarm on the rejection *rate* fires during exactly those stretches, gets dismissed, and gets dismissed again on the day it finally means something. Fire on the absence bucket becoming categorical — never on the rate, and never on emptiness alone.

## Record the outcome where the outcome is known

A metric gets placed where it is easy to write rather than where the thing it names becomes true, and the gap between those two points is where a whole class of failure hides. The section above is the same mistake at the other end — a counter that cannot see its input; this one cannot see its ending.

Success is recorded at the point a request is *accepted* — a status chosen, a header written, a job enqueued, a handler returning — because that is where the code has a natural fork and where the value is to hand. For anything that finishes later than that moment, the counter measures admission and reports completion. Streamed and proxied responses are the clearest case: once the status line is on the wire it cannot be retracted, so a failure in the body has no way to become an error the caller can read, and the success was tallied seconds before the failure happened. A queue publish counted as delivery and a fire-and-forget spawn counted as an execution are the same shape.

What makes this worth separating from an ordinary gap in coverage is the direction of the error. An event that is never counted leaves a hole, and a hole invites someone to look. An event counted as its own opposite produces a **clean** number over a broken period, and nothing about a zero prompts an investigation. Missing data is a question; wrong data is an answer.

Three consequences, all of them placement decisions rather than modelling ones:

- **Name the instant.** For each terminal counter, say out loud when it fires and when the work it names actually finishes. If those are two different sentences, the counter is in the wrong place.
- **Where the work outlives its status, instrument the end of the work** — wrap the stream, the body, the async completion, and count there.
- **Count a late failure as its own thing, not only as a failure.** The remedies differ: an error before the response is a status the caller can retry on, and one after it is invisible on both sides. Folding them together loses the distinction that would tell an operator which they have.

When the ending genuinely cannot be observed from where the counter lives, that is a finding rather than a dead end, and the fix is naming: a metric called *accepted* or *admitted* claims what it can prove, where one called *served* or *completed* implies an ending nobody watched. Renaming it is cheap and it stops the number being read as something it is not.

The tell that this is already happening is a success rate nobody can reconcile with what users report, sitting beside a failure count that stays at zero through incidents. A failure counter that has never once incremented is not evidence of reliability; it is evidence that nothing increments it.

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
| An adaptive component looks healthy for months and has learned nothing | Activity was instrumented; the volume of its ground-truth input never was |
| A filter rejected every item for months and nobody noticed | Rejections were counted but not attributed, so "failed the check" and "had nothing to check" landed in the same bucket |
| A learned score and a hardcoded default are indistinguishable downstream | Sample size did not travel with the value to its point of use |

## Red flags

- "We will add logging if it happens again"
- A new failure branch that increments nothing and emits nothing
- An alert whose runbook is "look at the graphs"
- Any metric label whose value comes from user input
- "Just log the whole object" as a debugging convenience
- A trace that stops at the queue boundary
- An SLI chosen because it was already being measured
- A report that filters out the records lacking its group-by field instead of bucketing them
- A component that learns, tunes, or calibrates, with no metric for how many observations it actually read
- A score, weight, or threshold presented to a user or another service without the *n* behind it
