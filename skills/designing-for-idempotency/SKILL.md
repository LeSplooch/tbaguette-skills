---
name: designing-for-idempotency
description: Use when an operation can arrive more than once — retries after a timeout with an unknown outcome, at-least-once queues, consumer restarts and redelivery, replayed webhooks, double-clicked buttons, resumed jobs, a crash partway through a write, or a confirmation, approval, or clarifying question being added partway through an operation that cannot pause. Also use when a setup, install, provisioning, or bootstrap step written to be safe to re-run may run again long afterwards, with someone having deliberately changed what it sets in between — a setting that keeps turning itself back on after every upgrade. Covers duplicate charges, duplicate emails and notifications, idempotency keys, deduplication windows, retry safety under concurrency, why a mid-operation gate re-runs every effect before it, telling a default from an assertion when both compile to the same write, and claims of exactly-once delivery.
---

# Designing for idempotency

## Overview

Assume every operation arrives twice. Retries, restarts, redeliveries, and impatient users are the same problem, and exactly-once *delivery* does not exist — exactly-once *effect* does, and you build it at the receiver.

## When to use

- Designing any mutating operation reachable over an unreliable channel: network, queue, IPC, or a button a human can press twice.
- A caller timed out and cannot tell whether the work happened.
- A consumer may reprocess after a restart, rebalance, or offset reset.
- Symptoms: duplicate charges, duplicate emails, doubled counters, a value that reverts to an old setting under load.
- Not for: deciding which failures are worth retrying (modeling-errors), or how much retrying the system will absorb (rate-limiting-and-backpressure).

## Three kinds of operation

| Kind | Test | Treatment |
|---|---|---|
| Naturally idempotent | `f(f(x)) == f(x)` — absolute set, delete-by-id, ensure-exists | Nothing needed. Verify it is genuinely absolute |
| Idempotent with a key | Anything that creates or emits something new: create order, capture payment, send message | Caller-supplied key plus a durable record of the outcome |
| Not makeable idempotent | A physical actuator pulse, a provider with no dedup support | Push the problem to the boundary: wrap in a keyed local operation and order the writes so a crash is diagnosable |

**The commonest accidental non-idempotency is a relative operation where an absolute one was available.** Prefer `set brightness = 40` over `increment by 5`; `set state = cancelled` over `toggle`; `set members = [...]` over `append`. Deltas are only worth it when the absolute value is genuinely unknown to the caller, and then they need a key.

## Convergence is the defect when somebody else writes between runs

The rule above prefers the absolute form because the two invocations it has in mind are seconds apart and come from the same caller: a retry, a redelivery, a double-click. Nothing happens in the gap. Under that assumption converging on the declared value is exactly right, and the relative form is the bug.

Setup steps break the assumption without looking like they do. An installer, a provisioning run, a first-launch routine, a bootstrap script — each is written to be safe to re-run, each is therefore written in the absolute form on this skill's own advice, and each then re-runs *weeks* later on an upgrade. In that gap a person changed the setting on purpose. `ensure autostart = on` is perfectly idempotent and it silently reverses their decision, every upgrade, forever. The user's report is that the setting "keeps turning itself back on", and the code review finds a correctly-written idempotent operation.

The distinction the absolute form cannot express is between **a default and an assertion**. A default says *if nobody has an opinion, use this*; an assertion says *this value is to hold regardless of who thinks otherwise*. Both compile to the same `set x = v`, and which one you meant is invisible at the call site — so it has to be carried in state rather than in the write:

- **Record that the default was applied, separately from the value.** The second run reads the marker, not the setting. Absent marker means first run, so apply; present marker means somebody owns this now, so leave it. This is an idempotency key in the sense of the section below, keyed on the provisioning act rather than on a request id.
- **Where the setting itself can hold it, make "unset" a real state.** Three values — on, off, never-chosen — let the setup step converge on the only one it has any business converging on.
- **Reserve the genuine assertion for things that are actually policy**, and say so where it is written, because the failure mode is the reverse: a security control that a user can turn off permanently because someone mistook an assertion for a default.

The general test, worth applying before reaching for the absolute form at all: **ask whether any writer other than this operation is expected to touch the value between invocations.** Where the answer is no — a retry of one request, a replayed message — the absolute form is right and this section does not apply. Where the answer is yes, idempotence stops being a safety property and becomes the mechanism by which one party's decision silently overwrites another's.

## Idempotency keys

- **Generated by the initiator, before the first attempt, reused unchanged across every retry of that attempt.** A key the server generates, or one regenerated per retry, is a request id and does nothing. This is the most common broken implementation and it passes every test that only retries once.
- Scope is `(tenant, operation type, key)`. Never global. One account's `abc123` must not collide with another's, and a key for "create refund" must not match one for "create invoice".
- Derive the key from the *intent*, not the payload: a UUID minted when the form renders or the button arms. Hashing the payload is a fallback that breaks deliberate repeats — sending the same person the same $5 twice on purpose is a legitimate action. If you hash, add an explicit repeat discriminator.
- Retention must cover the longest realistic replay horizon: `max(client retry budget, broker redelivery window, human retry behavior, incident replay)`. Floor of 24h; 7–30 days for money and messaging. **Publish the window in the contract** so callers know when reuse stops being safe.
- **Store the outcome, not a "seen" flag.** A seen-flag design returns success with no body on the duplicate, and the caller who lost the first response still has no identifier. Store: state (`in-progress` / `done` / `failed`), the response body or a pointer to it, the status, and a fingerprint of the request.
- On a repeat, return the *original* response — same status code, same body, ideally byte-identical, optionally marked as a replay. A duplicate create returns the original success, never a conflict.
- Concurrent duplicate (second arrives while the first is in flight): reserve the key atomically *before* doing the work. The loser waits briefly for the result or returns a retryable "in progress". Doing the work twice is wrong; returning success without the result is also wrong.
- Same key, different payload is a client bug. Reject with a distinct, non-retryable error. Silently returning the first result hides a real defect and is what naive implementations ship.

## Idempotent, commutative, associative

Idempotent: twice equals once. Commutative: order does not matter. Associative: grouping does not matter.

**Retry safety needs all three the moment two senders can interleave.** A retried `set x = 5` is perfectly idempotent and still wrong: it can land after `set x = 7` and resurrect the old value. This zombie-write bug is invisible in single-writer tests and appears the week you scale out.

Fixes, in order of preference:

1. Condition the write on a version or generation — compare-and-set, ETag, `WHERE version = n`. Turns a late retry into a harmless rejection.
2. Attach a monotonic sequence or timestamp from the initiator and resolve last-write-wins on it.
3. Choose a genuinely commutative value type: sets, max, grow-only counters.

"The retry will be fast enough to land first" is not a fix.

## Non-idempotent side effects at the boundary

- Order the work: **record the decision durably → perform the effect → record completion.** A sweeper or outbox retries anything stuck between step 1 and step 3. This converts "exactly once" into "at least once plus dedup at the receiver", which is the only version that exists.
- Pass a dedup key to the provider wherever supported — payment idempotency key, message deduplication id, email `Message-ID`. Where unsupported, keep a local `(effect, key) → sent_at` table and treat it as authoritative.
- Split trigger from effect: the handler writes an intent row, a separate worker performs and marks it. Crash before the effect means retry; crash after means at most one duplicate, which the provider-side key absorbs.
- Never ack or commit the queue offset before the effect is durable. Ack-then-work loses messages; work-then-ack duplicates them, and duplicates are the failure mode you have a design for.
- Where no provider key exists, budget an accepted duplicate rate and state it ("at most one duplicate per failed attempt") rather than claiming exactly-once.
- **Decide what happens after the dedup window expires.** A replay past the window is a new request. For money and messaging it must fail closed — reject as expired. Every naive implementation fails open, which means the one duplicate charge you get is the one from the six-day-old retry.

## A gate added mid-operation turns one call into several whole ones

Adding a confirmation step to a dangerous operation feels like it can only make
it safer. Where the system carrying the operation cannot hold an open
conversation — and most cannot — it usually does the opposite, for a reason that
has nothing to do with the gate itself.

The standard way to ask a question partway through a stateless call is to
abandon the call, ask, and have the caller re-send the *entire* original request
with the answer attached. The second attempt is a fresh, complete execution. So
every effect the operation produced before it reached the question point happens
again: the row inserted during validation, the file written while staging, the
upstream reservation, the notification fired on entry, the counter incremented.
One logical operation becomes N complete attempts, and N grows with how many
times the caller is asked. A gate placed to prevent one bad outcome has
multiplied every effect that precedes it.

Nothing in a request/response protocol prevents this, and few supply an
operation identity by default — duplicate detection is left to whoever
implemented the operation, which is to say usually nobody. The same shape
appears well outside protocols: a re-posted web form, a payment step-up, a
re-authorization round trip, a workflow resumed from a checkpoint, any queue
with at-least-once delivery.

Three answers work, and picking one is part of adding the gate rather than a
follow-up to it. **Put the gate strictly before the first effect**, so an
abandoned attempt did nothing worth repeating — which usually means gathering
every decision up front instead of discovering mid-flight that you need one.
**Give the operation an identity the executor enforces at-most-once on**, minted
by the caller, carried across every attempt, and checked before any effect. Or
**make the effects themselves idempotent**, in the sense the sections above
describe, so that repeating them is genuinely free and the retry stops
mattering. What does not work is adding the gate and deciding none of this: a
confirmation bolted onto a non-idempotent operation is a multiplier wearing a
safety control's name.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Duplicate charges or emails during an incident | Key generated per attempt, or retention shorter than the replay window |
| Retry returns a conflict error | Duplicate treated as an error instead of returning the original result |
| Duplicate returns success with an empty body | Stored a seen-flag instead of the stored outcome |
| Two duplicates both perform the work | Key not reserved atomically before starting |
| A setting flips back to an old value under load | Idempotent but not commutative; no version condition on the write |
| Consumer reprocesses everything after a rebalance | Offset committed before the effect was durable, or no receiver-side dedup |
| "Our broker is exactly-once" | It isn't; the guarantee ends at the ack, and your handler is past it |
| Works in tests, duplicates in production | Tests retry sequentially; production retries concurrently |
| Dedup passes but the row is written twice | Dedup keyed on a payload hash that includes a timestamp |

## Red flags

- "The client only sends it once."
- "We'll dedupe by looking for an existing record with the same fields."
- "Exactly-once delivery."
- "Just make it idempotent" with no key named and no store named.
- A create operation with no caller-supplied key.
- Committing the offset, ack, or transaction before the external effect is durable.
- A retry policy written before anyone checked whether the operation is safe to repeat.
