---
name: designing-apis
description: Use when defining an interface other code will call — a public function or library entry point, an HTTP or RPC endpoint, an IPC or wire message, a plugin contract, or an exported module boundary. Covers naming and granularity, required versus optional parameters, defaults, pagination and ordering, growing an enum, opaque tokens, separating write paths by data provenance, deprecation and sunset, versioning, and judging whether a proposed change is breaking.
---

# Designing APIs

## Overview

The interface is the only part you cannot refactor later: everything behind it is yours, everything in front belongs to callers you cannot deploy. Design the failure modes, the growth path, and the compatibility promise before the happy path.

## When to use

- Adding anything another team, process, or version will call: exported function, endpoint, message type, plugin hook, config key.
- Deciding one call versus three; required versus optional; what to return for a collection.
- Adding a field, parameter, enum member, or error code to something already shipped.
- Someone asks "is this change breaking?"
- Not for: choosing a single identifier's wording (naming-things), changing a contract that already has deployed callers (`schema-evolution`), deciding where the boundary belongs at all (drawing-boundaries).

## Shape and granularity

- Write three call sites — the common one, the rare one, the awkward one — before the implementation. The common case must be reachable with the fewest arguments; the rare case must be reachable at all. If the awkward one requires reaching around the API, the shape is wrong.
- **One call per caller intent, not per stored entity.** If every caller makes the same N calls in the same order, that sequence is the API and the N calls are leaked implementation.
- Granularity follows call cost: in-process ≈ nanoseconds, IPC ≈ microseconds, network ≈ milliseconds. A boundary costing 1ms cannot expose per-field accessors; a boundary costing 10ns should not force a batch.
- Name the effect, not the mechanism: `cancel`, not `setStatusCancelled`. Mechanism names age badly because the mechanism changes and the name cannot.
- **A boolean parameter is a design failure at the first one.** `render(true)` is unreadable at the call site and the second flag creates a combinatorial mess. Use a named option type or enum from flag one.
- Never let a field's meaning depend on another field's value. "If `kind` is A, `payload` is X" is a tagged union; ship it as a discriminated union with a closed shape.
- **One write path per provenance, not per row shape.** Where a field encodes how strongly something is believed, writing an outside claim through the operation that means "we observed this" erases the distinction permanently — matching row shapes are what make the shortcut tempting, not what make it safe. `tracking-data-provenance` covers the claim taxonomy and the promotion rules that turn a laundered value into authority.

## Required, optional, and absence

| Concept | Encode as |
|---|---|
| Always meaningful | Required field |
| Usually omitted, safe fallback exists | Optional with a **documented default** |
| Meaningfully absent | Explicit `unset` variant or sentinel, never a null shared with "not provided" |
| Patch semantics (unset vs cleared vs set) | Three encodings, not two — a nullable field cannot express this |

Optional-with-default beats nullable for two reasons: null forces every caller to branch, and on any wire format null is indistinguishable from "an older client did not send this field". Put the default in the contract so it is versionable; a default replicated into each caller silently drifts per client.

## Designing for growth

- **Additive-only is the cheapest promise you can keep.** Additive means: a new optional field, a new operation, a new enum member *only if tolerance shipped first*, a new error code *only if callers were told to handle unknown codes by class*.
- **Never return a bare closed enum you may need to grow.** Every closed enum is a promise you will never add a member. If growth is plausible, ship an `unknown`/default arm and the client's obligation to tolerate it from day one — `schema-evolution` covers the deployment order for actually adding the member later without crashing the fleet that hasn't caught up.
- Opaque tokens — cursors, continuation handles, resume tokens, ETags — must be opaque in fact, not just in the docs: encode and sign or version the payload, and reject a token you did not mint. If callers can parse it, they will depend on it, and their dependency becomes your contract.
- Reserve removed field names and tag numbers so a future field cannot reuse them and misparse old data — `schema-evolution` has the full mechanics of why reuse corrupts archived data with no error.
- An open `extra: map<string,string>` bag becomes a schema you never versioned and can never validate. Prefer a reserved namespace with an explicit no-compatibility statement, or nothing.

## Collections, errors, idempotency

- Any operation returning a collection needs, in v1: a `limit` with a documented default and maximum, a **total** ordering, and an opaque cursor. Retrofitting is breaking even behind a new version, because "returns all of them" is what callers built on.
- Ordering must be total — tiebreak on a unique key. Ordering by a non-unique column under pagination silently duplicates and skips rows; this is the most common pagination bug and it never appears in small test data.
- Enumerate the failures at design time; they are part of the type whether or not the language says so. Minimum: a stable machine-readable code, a retryable indicator, and an identifier the caller can quote back. See modeling-errors.
- For every mutating operation, answer "the caller timed out and does not know the outcome" before shipping. If the answer is "they cannot tell", you designed a double-charge. See designing-for-idempotency.

## Versioning and the compatibility contract

The menu of versioning strategies and their real costs is the same whether you're picking one now or dealing with a contract that's already shipped — see `schema-evolution`'s versioning table rather than a second copy here. What belongs at design time instead: state three things explicitly before shipping v1 — which surface is covered, how long an obsolete thing keeps working (in absolute time or releases), and how a break is signaled. **Deprecation without a runtime signal is not deprecation** — nobody reads changelogs; things that emit warnings get fixed. "Experimental" is only real when it is mechanically unpleasant to use (opt-in flag, `unstable_` prefix); a documentation note does not stop anyone from depending on it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Every new feature adds another boolean parameter | Options never modeled as a type; the first flag set the pattern |
| Every caller wraps the API in the same helper | Wrong granularity — their helper is the API you should have shipped |
| Adding an optional field broke old clients | Contract never stated unknown-field tolerance, so parsers were strict |
| "Page 2 and page 3 both contained item X" | Non-total ordering under pagination |
| A new enum member crashed the fleet | Member added before tolerant clients were deployed — see `schema-evolution` for the deployment order |
| v2 quietly returns fewer results than v1 | Unbounded collection in v1; adding a limit is breaking regardless of the version label |
| Callers keep getting the call order wrong | Sequencing is a real constraint and belongs inside one call or an explicit state machine |
| Support asks callers to "check the error text" | No stable error code in the contract |
| A caller broke when you changed a cursor's format | Token documented as opaque but never made opaque |
| An outside recommendation trips a threshold meant for first-party evidence | One write path served two provenances, so the row records the value and forgets where it came from |

## Red flags

- "We'll add pagination when it gets big."
- "It's internal, we can change it later" — about something whose callers you do not deploy.
- "Just return null for now."
- "Callers shouldn't depend on that."
- "We'll mark it experimental in the docs."
- Adding a flag parameter instead of asking whether this is really two operations.
- Reusing a write path because the row shape matches, without checking that the claim's provenance does.
