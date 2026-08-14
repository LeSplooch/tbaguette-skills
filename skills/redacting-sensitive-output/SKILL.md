---
name: redacting-sensitive-output
description: Use when secrets, credentials, tokens, or personal data could reach a log line, error message, stack trace, crash dump, metric label, trace span, analytics event, support bundle, or a recorded test fixture. Covers what a redactor must not do with the match it found, allowlisting fields rather than denylisting patterns, partial reveals and re-identification, encoded and nested payloads, and testing by asserting the input is absent rather than the marker present.
---

# Redacting sensitive output

## Overview

Redaction is the last line of defence, applied at the moment data leaves the program for somewhere a human or a vendor will read it. It fails quietly by construction: a redactor that redacts nothing produces output that looks exactly like output it redacted correctly.

The two rules that carry most of the value: **the replacement must not quote the match**, and **the test asserts the input is gone, never that the marker is present.**

## When to use

- Any log, error, trace, metric, crash report, support bundle, or analytics event that can contain values from a request, a config, or a user.
- Recording a fixture from a real response (`grounding-test-doubles`).
- Adding debug output during an incident, when the temptation to log the whole object is highest.
- Sharing a reproduction, a bug report, or a paste with a vendor or a colleague.
- Not for: where secrets should live and how to rotate them (`secrets-hygiene`), restricting who can reach the data at all (`least-privilege-design`), or deciding what is worth emitting in the first place (`instrumenting-for-observability`) — redaction assumes the data already reached code that should not emit it.

## The replacement must not quote the match

A redactor that substitutes a marker containing the first N characters of whatever it matched — added for debuggability, or to make a log line useful — **reproduces the thing it exists to remove** whenever the match is around N characters long, and then hands it to exactly the consumer the filter was protecting.

The trap covers secret masking, PII scrubbing, and profanity filtering equally, and it is attractive precisely because the quoted context is what makes the log useful.

What is safe to emit about a redacted value:

| Safe | Unsafe | Why |
|---|---|---|
| That a value was present | Any substring of it | Short values are fully reproduced by a "prefix" |
| A fixed-width marker | A marker whose length tracks the input | Length leaks entropy and identifies format |
| Which field or rule matched | The matched text as "context" | The context is the secret |
| A keyed hash, for correlation | A plain hash | Plain hashes of low-entropy values are trivially reversed |
| A stable opaque identifier | "Last 4 characters" of a low-cardinality value | Enough to re-identify across records |

Partial reveals are a product decision, not a debugging convenience. Last-4 on a card number exists because a standard permits it; the same pattern applied to an API key or an email address leaks a real fraction of a real secret to whoever reads the log.

## Allowlist fields, do not denylist patterns

Pattern-based redaction over free text is a losing game: it must anticipate every format, and it fails open — an unmatched secret passes through silently, which is the same failure shape as `handling-untrusted-input`'s argument against denylists.

Prefer structure. Emit a chosen set of fields, and let anything unlisted be dropped by default rather than inspected. This inverts the failure: the mistake becomes a missing field in a log line, which someone notices and fixes, instead of a leaked credential nobody sees.

Where structured logging is available, this is nearly free — mark fields sensitive at the type or schema level so the redaction travels with the value instead of being reapplied at every call site. A value that knows it is a secret cannot be accidentally formatted by a call site that forgot.

Pattern matching still earns a place as a **backstop over the allowlisted output**, catching a token that ended up inside a field that was supposed to be safe. Backstop, not primary.

## Where it gets missed

Redaction is usually applied to the obvious log call and nowhere else. The gaps, in rough order of how often they leak:

- **Exception messages and stack traces.** Frameworks render arguments and local variables; a constructor that received a token puts it in the trace.
- **The URL.** Query strings are logged by every proxy, load balancer, and access log in the path, and they are outside your process. Secrets never belong in a query string.
- **Request and response bodies** logged wholesale for debugging, especially behind a flag someone left on.
- **Nested and encoded payloads** — base64 blobs, JSON inside a JSON string, gzipped bodies, JWT payloads. A field-level redactor never looks inside them; decode before matching, or refuse to log the field.
- **Metric and trace labels**, where a user identifier or email becomes a high-cardinality dimension retained for months.
- **Crash dumps and heap snapshots**, which contain everything by definition.
- **Support bundles and diagnostic exports**, assembled by a different code path with its own idea of what is safe.
- **Third-party SDKs** — error reporters and APM agents that capture request context automatically, with their own scrubbing config you have not reviewed.

## Testing it

Assert the **absence of the input** in the output. Never the presence of the marker.

The marker is trivially satisfiable while the removal fails, so a test that only looks for `[REDACTED]` passes against a redactor that emits the marker *and* the secret, and against one that emits the marker while leaving the original elsewhere in the line.

- Feed a known sentinel value through the whole emission path, then assert the sentinel appears nowhere in the captured output.
- Test the boundary lengths specifically: a value shorter than the marker's context window, and one exactly at it.
- Test the paths that are not the happy log call — throw an exception carrying the secret, and assert the sentinel is absent from the rendered trace.
- Where practical, run the assertion over the real sink rather than the formatter, so anything the sink adds is covered too.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Redacted" output still contains the matched string | The replacement embedded the match for context |
| A test passes against a redactor that redacts nothing | Asserted the marker's presence, never the input's absence |
| Secrets in logs despite a redactor | Leaked via a stack trace, a URL, or an SDK's automatic capture |
| A token inside a base64 field passed through | Field-level matching never decoded the payload |
| Short secrets leak, long ones do not | A fixed-size prefix reveals a short value entirely |
| A new field started leaking silently | Denylist by pattern, which fails open on anything unanticipated |
| Redacted logs still identify individuals | Partial reveals correlate across records |
| Marker length varies with the input | The redaction leaks the value's length |

## Red flags

- "I'll include the first few characters so it's debuggable."
- "It's hashed, so it's fine."
- "We redact that in the logger" — about a value reaching an error reporter.
- "The regex covers all the token formats."
- "It's just in the URL."
- "Let me log the whole request object to debug this."
- A redaction test whose only assertion is that the marker appears.
