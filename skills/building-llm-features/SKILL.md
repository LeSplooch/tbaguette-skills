---
name: building-llm-features
description: Use when code calls a language model at runtime — adding the first call, choosing or changing the model identifier, moving a prompt out of a string literal, parsing structured output or tool calls, handling a refusal, a truncated response, or a rate limit, or a provider announcing a model's retirement; when token spend or latency is higher than expected; when deciding what of a prompt and its completion to log; when model output flows into a query, a page, a shell, a file path, or a tool with side effects; when a model edits records it was shown only a redacted, truncated, or summarized view of; or when a test hits a hosted model or asserts on its exact wording. Covers the model as a pinned behavioral dependency, prompts as versioned code, the outcomes a call can have beyond success and error, output as untrusted input, token and time budgets, fallbacks that must pass the same evals, telemetry that keeps content out of spans, and where the test seam goes.
---

# Building LLM features

## Overview

A call to a language model is a remote dependency with three properties no other dependency has all of. Its behavior changes without any change on your side — a provider updates a model in place, retires it on a date, or ships a successor that reads the same prompt differently. Its output is text shaped by everything in its context, including whatever a user or a fetched document put there. And it is metered by the token, in both directions, at prices that make an unbounded loop a billing event.

Most of what goes wrong in these features is ordinary engineering applied too late: an unpinned dependency, an unvalidated boundary, an unbounded resource, an unmodeled failure. This skill is where each of those lands for a model call, and it hands off to the skill that owns the general form.

## When to use

- Writing the first call to a model in a codebase, or a new feature built on one.
- Choosing, changing, or being forced off a model identifier.
- A prompt lives in a string literal next to the code that sends it.
- Output is parsed as structured data or as tool calls.
- A model is asked to edit or rewrite data its context held only part of — redacted, truncated, or summarized.
- A response came back refused, cut off, or rate-limited, and the code treated it as an answer.
- Spend or latency is higher than anyone predicted.
- Deciding what of a prompt and completion to log, trace, or keep.
- Model output reaches HTML, SQL, a shell, a URL, a file path, or a tool that changes something.
- A test calls a hosted model, or asserts on its exact wording.
- Not for: measuring whether the output is good — `evaluating-llm-output` owns evals and judges. Not for: designing the tools a model will call (`designing-apis` has a section on callers that are models). Not for: an agent working in your repository rather than a model inside your product — see `bounding-autonomous-work` and `least-privilege-design`.

## The model is a pinned, behavioral dependency

- **Pin the exact dated identifier**, not an alias that follows the latest release. An alias upgrades you on the provider's schedule, silently, with no diff to review and no change to roll back.
- **Keep it in one place**, with the parameters that go with it, not repeated at each call site. The day it has to change, it changes in one reviewed line.
- **Know when it expires.** Providers retire models on published schedules and requests fail afterwards; record the date beside the identifier and start the move while both models still answer. `upgrading-dependencies` treats that move as the behavioral major it is.
- **Measure a model change and a prompt change separately.** A new model often needs its prompt adjusted, so they may well ship together — but run the model swap on the old prompt first, then each prompt edit, so every movement in the evals has one cause.

## Prompts are code

The prompt text, the model identifier, the sampling and output parameters, the tool definitions, and the output schema are one unit of behavior. Keep them together, in the repository, under review like any other code, with a version the telemetry can record. Any change to any part of the unit runs the evals (`evaluating-llm-output`), including the "harmless wording tweak".

Keep instructions and data in separate places inside the request. Instructions go where the provider puts instructions; user text, retrieved documents, and tool results go in their own clearly delimited slots, never concatenated into the instruction string. That does not make injected instructions harmless — nothing does — but it stops your own template from promoting user text into instructions by construction. `configuration-management` covers what, if anything, may vary per environment.

## A call has more outcomes than success and failure

| Outcome | How it shows | What the code does |
|---|---|---|
| An answer | Normal stop reason | Validate it, then use it |
| A refusal | A refusal stop reason or field, or a polite non-answer in the text | Handle as its own branch; never retry the same request verbatim hoping for a different mood |
| A truncated answer | The stop reason says the output limit was hit | Treat as incomplete: a prefix of valid text, or half an object, is not an answer |
| A transient failure | Rate limit, overload, timeout | Retry with backoff and a budget (`rate-limiting-and-backpressure`) |
| An answer that parses and is wrong | Nothing at the transport layer | Only evals and validation catch it |

Branch on the stop reason the provider reports, not on whether the text looks finished. Retry only the transient class, and keep the retry budget small (`modeling-errors`). When a call can trigger a tool with side effects, the retry is a second invocation of that tool unless the tool is idempotent (`designing-for-idempotency`).

**Structured output guarantees shape, not truth.** Constrained decoding produces output that parses against a schema; it does not enforce every bound a schema can express, it does not hold on a refusal or a truncation, and it has no opinion on whether the values are right. Parse the output into a type at the boundary and reject what the type cannot hold (`handling-untrusted-input`). The schema is a contract with the model and with everything downstream, so changing it is `schema-evolution`, not an edit.

**A model can only give back what it was shown.** When the context holds a lossy view of the data — redacted for privacy, truncated to fit, summarized to save tokens — never ask for the whole record back. A model asked to restate a field it never saw does not reliably leave it alone: it writes a plausible value, or copies the redaction placeholder, and either one parses, validates against the schema, and replaces the real value. Ask for a whole collection back and every record in it is rewritten that way, including the ones nobody asked to change, and any record cut off by the truncation comes back missing.

Ask for a list of changes instead. Name each record by a short, stable id the caller issued (`designing-apis`' *Identifiers the caller can carry*), never by its position or by a name the model might paraphrase. A new record comes back marked as new and gets its id from the caller; removing one is its own explicit operation, never inferred from an id missing from the reply. Treat a field the model sends as "set this" and a field it leaves out as "keep this", and reject a change to a field it was never shown. If clearing a field must be possible, give clearing its own explicit value, because one null cannot mean both (`designing-apis`' *Required, optional, and absence*). Do the merge on the side that holds the full record, and reject any id the caller did not issue. This is `schema-evolution`'s writer that builds its payload from the subset it rendered, with invention in place of deletion, and invention is the harder of the two to catch, because an invented value does not look missing. Test it that way: run an edit aimed at one field of one record, and assert that every other record, and every other field of that one, is byte-identical after the merge.

## Its output is untrusted input to every sink

Anything that reached the model's context can steer its output: a user's message, a retrieved page, a tool result, a document a user uploaded. So the output is exactly as trusted as the least trusted thing it read, and it reaches a query, a page, a shell, a URL fetch, or a file path only through the same encoding and validation as a request field. Rendering it as HTML is an injection sink; letting it pick a URL to fetch is a request forgery; letting it name a file is path traversal.

- **Scope what it can do, not what it is told.** A feature whose model can call tools gets the narrowest set that serves the feature, each with the narrowest credential (`least-privilege-design`). Instructions in the prompt are a request to the model, never a control.
- **Never let one request hold all three legs.** Untrusted content in the context, access to data the requester should not see, and a channel out — a link it renders, a message it sends, a tool that writes — together are how injected text exfiltrates. Remove one leg for the whole request; `handling-untrusted-input` has the general rule. That stops exfiltration only: a tool that deletes, pays, or writes does its harm with no channel out, so its reach is set by the bullet above.
- **Assume the system prompt will be read by a user.** No credentials, no internal hostnames, no authorization logic that works only while it stays secret (`threat-modeling`).

## Budgets, before the bill

- Cap output tokens on every call, and total time per request — time to the first token and time to the last are different numbers, and streaming only improves the first.
- Cap spend per user or tenant. An endpoint that turns input into model calls is an endpoint that turns input into money, and loops, retries, and agents multiply it.
- Send work nobody waits for through the provider's batch interface, which is typically priced well below the interactive one.
- Order the request stable-first, volatile-last, so a prompt cache can reuse the shared prefix, and confirm hits in the usage the response reports rather than assuming them (`caching-strategy`).
- A token count is a property of a tokenizer. The same text costs a different number of tokens on a different model, so every budget and every client-side estimate is re-measured when the model changes.

## A fallback model is a different model

Falling back to another model on failure or overload is a reasonable design, and it has one condition: the fallback passes the same evals as the primary, on its own run, before it is ever routed to. Otherwise the feature quietly becomes a different product at exactly the moment the provider is having a bad day, and nobody sees the quality drop because the error rate went down. Exercise the fallback path deliberately; one that only runs during outages has never been tested (`confirming-before-claiming-done`).

## Telemetry, and what stays out of it

For every call, record the model requested and the model that answered, the prompt version, input and output token counts, the stop reason, latency, and cost. Where an open telemetry standard publishes conventions for model calls, use its attribute names so tools can read them, and pin the convention version, since those conventions are young and names have moved (`instrumenting-for-observability`).

The prompt and completion text are different in kind. They routinely contain personal data and whatever users pasted in, the conventions themselves make capturing them opt-in, and a trace backend is rarely where that data is allowed to live. Store content, if at all, in a separate place with its own retention and access, referenced from the span rather than inside it (`redacting-sensitive-output`). A value a model produced and you stored is inferred, not observed; record it as such, or it will later be read as fact (`tracking-data-provenance`).

## Where the test seam goes

- **One seam at the model client**, so unit tests never call a hosted model (`testing-the-untestable`). A seed or zero temperature does not make a hosted model repeatable, and some models reject sampling parameters outright.
- **Recorded responses, including the awkward ones.** Capture real responses — refusals, truncations, tool calls, malformed output — and replay them (`grounding-test-doubles`); a hand-written happy-path response tests only the path that was never going to break. Keep one live test per integration so the recordings cannot drift away from the real API unnoticed.
- **Assert structure and invariants, never wording.** The output parses, the required fields are there, the total matches the items, no tool was called with an out-of-range argument (`property-based-testing`). An assertion on exact text breaks on the next model update and was never testing correctness.
- **Quality is not a unit test.** Whether the answers are good is measured as a rate, by evals, on their own gate.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Behavior changed overnight with no deploy | The code named an alias, and the provider moved it |
| Requests started failing on a date nobody had in a calendar | A pinned model was retired and its date was never recorded |
| A half-sentence or half an object was shown to a user as an answer | The output-limit stop reason was never checked |
| After a model edit, fields nobody touched changed, on records nobody mentioned | The model saw a redacted or summarized view, was asked to return whole records, and the caller stored them as they came back |
| A refused request was retried until it "worked" | Refusal handled as a transient failure |
| Model output rendered as markup ran script in someone's browser | Output treated as the application's own text rather than as untrusted input |
| An injected instruction in a document made the feature send data out | One request held untrusted content, private data, and a channel out |
| The bill tripled after a feature launch | No per-call token cap, no per-user budget, and retries multiplying both |
| Quality dropped during an outage and error rates looked fine | A fallback model that had never been evaluated took the traffic |
| Personal data turned up in the tracing backend | Prompt and completion content captured into spans by default |
| Unit tests fail after a provider update, with nothing broken | Tests asserted on the model's exact wording |

## Red flags

- A model identifier that ends in "latest", or no identifier at all because the SDK picks a default.
- A prompt assembled by concatenating user text into the instruction string.
- A bare JSON parse of model output with nothing between it and the rest of the program.
- A model asked to return a whole record or a whole list when its context held only a redacted, truncated, or summarized view of it.
- A retry loop around a model call with no cap on attempts or on spend.
- "The prompt tells it not to do that" offered as the control.
- A new model or a new prompt shipped with no eval run between the change and the release.
- Full prompts and completions in the application log "for debugging".
