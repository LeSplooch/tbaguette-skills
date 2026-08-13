---
name: modeling-state-machines
description: Use when something has a lifecycle — order, job, connection, upload, session, subscription, device, workflow step — or when several booleans and flags describe one thing. Also for status fields, impossible flag combinations, records stuck in a status forever, unexplained state changes, a crash partway through a transition, timeouts and cancellation, and adding or renaming a state that is already persisted.
---

# Modeling state machines

## Overview

Three booleans encode eight combinations; if only five are real, you shipped three bugs and no way to detect them. Name the states, enumerate the transitions, and make the forbidden ones observable failures rather than silent no-ops.

## When to use

- Anything with a lifecycle, or any entity carrying three or more flags that qualify each other.
- Conditions like `isLoading && !hasError && data != null` appearing in more than one place.
- Symptoms: "how did it get into this state?", rows stuck in `processing` forever, duplicate side effects after a reload, a cancelled item that still gets processed.
- Adding a state, or renaming one, in something already persisted.
- Not for: rewriting existing stored rows once the states change (data-migrations), or the wire compatibility rules for the state field (schema-evolution).

## Finding the real states

- **The tell:** N booleans on one entity with N ≥ 3, or a status field plus flags that qualify it. Write out all 2^N combinations and mark the impossible ones. Any impossible combination that is reachable in code is a bug you already have and have not noticed.
- A state is a set of *(allowed operations, allowed transitions, invariants)*. Two situations permitting exactly the same operations are the same state however differently they are described. Two differing in even one allowed operation are different states however similar they sound.
- Name the situation, not the last event: `awaiting-payment`, not `payment-requested`. Event-named states multiply, because every event eventually gets one.
- **Data valid in only some states belongs inside the state:** `Failed { reason, attempts }`, `Active { session, started_at }`. A record with six optional fields, each meaningful in one status, is boolean soup wearing a hat.
- Right-sizing: under 3 states you do not need a machine. Past roughly 12 for one entity you have merged two machines — usually a lifecycle and a payment or approval status. Split them; orthogonal concerns multiply into a combinatorial explosion inside one machine.

## Making illegal states unrepresentable

- Where the language has sum types, tagged unions, or sealed hierarchies (Rust, Swift, Haskell, ML, Kotlin and Java sealed types, TypeScript discriminated unions), model the state as one. Exhaustiveness checking then turns "add a state" into a list of compile errors at exactly the sites that must change. That is the whole payoff and it is worth restructuring for.
- Where it does not (C, Go, Python, Ruby, SQL schemas, wire formats), enforce at the boundary: one constructor per state, a single `transition(current, event) -> next` function that is the **only** writer, and a check constraint or trigger enforcing which columns may be non-null per status.
- **A state field with public write access is not a state machine; it is a variable.** No code outside the transition function assigns it, in any language.
- On the wire, encode a closed set of names, never booleans, and treat it as a growable enum: ship readers that tolerate an unknown state before writers that emit one.

## The transition table is the specification

- Rows are states, columns are events, cells are the next state or `—` for forbidden. Write it before the code. **The forbidden cells are the most valuable part** — they are what review and tests check against, and they are what nobody writes down.
- Every cell needs a decided answer for a *duplicate* event: the second `cancel` on a cancelled order is usually a successful no-op, occasionally a rejection, never undefined. At-least-once delivery guarantees you will get duplicates.
- **Timeouts and cancellation are events producing states, never ambient conditions.** `Expired` is a state. "Created over 30 minutes ago and still pending" computed at read time is not: two readers can disagree, and nothing fires the side effects of expiring. If a timeout matters, something must actively drive the transition — a scheduled sweep, a timer, or a lazy check that *writes*.
- Every non-terminal state needs an exit: a timeout, a retry limit, or a manual override. **A state with no automatic exit is where entities pile up forever**; audit for it explicitly, because the symptom appears months later as a support queue.
- Side effects belong to transitions, not states. "On entering `Shipped`, send the email" attached to the state re-sends on every reload and every replay.

## Where the state lives, and crashes

- Exactly one authoritative store per machine. State duplicated across a cache, a UI, and a database is three machines that will disagree; everything but one must be a projection that can be rebuilt and discarded.
- **Guard every write with the expected current state** (`UPDATE ... WHERE status = 'pending'`). This makes each transition a compare-and-set and kills lost-update races for free. A write that does not check the source state is how two workers both advance one entity.
- A transition spanning a state write and an external effect must have a named in-between state — `Charging` between `Pending` and `Paid`. "It crashed between the update and the API call" must land on a state you have a written recovery rule for, not an undefined one.
- Persisted states are data, not code. Never reuse a removed state's name. Renaming a state is a breaking data change even though it looks like a refactor: it needs a migration or a documented mapping, not a find-and-replace. If the per-state payload shape changes, carry a version alongside.
- Log every transition as `(entity, from, to, event, actor, at)`. This one table answers "how did it get like this" and removes the most expensive category of support investigation that exists.

## Testing

- Test every cell, including all forbidden ones. A forbidden transition must be **rejected and observable**, not silently ignored — a silent no-op is indistinguishable from success to the caller that just lost its work.
- Test a duplicate of every event in every state.
- Property test: from any reachable state, any random event sequence yields only named states and never violates an invariant.
- Assert reachability both ways: every state is reachable from the initial state, and every non-terminal state can reach a terminal one. An unreachable state is dead code or a missing transition; an unescapable one is a leak.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "How did it get into this state?" | No transition log, and the field is assigned in several places |
| Two flags contradict each other | Independent booleans instead of one state |
| Cancelled items still get processed | Forbidden transition silently ignored instead of rejected |
| Duplicate emails after a reload or replay | Side effect attached to a state rather than to a transition |
| Rows stuck in `processing` forever | A state with no timeout and no exit |
| A status rename broke old rows | Persisted state treated as code instead of data |
| Adding a state meant grepping for every switch | State modeled as a string or int rather than a closed type |
| Two workers both advanced the same entity | Write not conditioned on the expected current state |
| UI shows a state the backend does not have | State duplicated instead of projected |
| Expiry never triggers its side effects | Timeout modeled as a read-time computation |

## Red flags

- "Just add a boolean for it."
- "The status is whatever the last event set it to."
- "We compute expired at read time."
- "That transition can't happen" — with nothing enforcing it.
- A status column typed as free text.
- Any code outside the transition function assigning the state field.
- A transition table with no `—` cells.
