---
name: schema-evolution
description: Use when changing a contract that is already in production — adding, removing, renaming, or retyping a field in a database schema, serialized format, stored document, API payload, or queue message. Also when a rolling deploy breaks deserialization, when old consumers cannot read new data, when a rollback fails on data the newer version wrote, when adding an enum value, or when planning a version bump.
---

# Schema evolution

## Overview

Once a contract is in production you no longer own both sides of it. Every change must be safe for readers you cannot redeploy and for data written by writers you cannot recall. The question is never "is this change correct" but "is this change correct while both versions are running, and again while rolling back."

## When to use

- Adding, removing, renaming, retyping, or re-scoping a field in anything persisted or transmitted
- A rolling deploy produces deserialization errors, unknown-field rejections, or missing-column failures
- Adding a value to an enum, status, or any other closed set
- Old messages sit in a queue or a log and will be read by new code, possibly weeks later
- Planning an API or payload version bump, or being asked whether one is needed
- Not for: moving or rewriting the data that already exists — that is a migration, and it is a separate discipline with its own failure modes

## Backward and forward compatibility

| Term | Definition | Broken by |
|---|---|---|
| Backward compatible | New code reads old data | Requiring a field that old writers never produced |
| Forward compatible | Old code reads new data | Removing or repurposing a field old readers still use; strict readers that reject unknown fields |

A rolling deploy needs both at the same time, because both versions run concurrently for the length of the rollout, and every message or row written during that window is read by whichever version happens to pick it up. A rollback needs forward compatibility specifically: the older binary you are rolling back to must survive data the newer one already wrote. This is the case teams skip, and it is why "the deploy went fine" and "the rollback corrupted things" are the same incident.

Design readers to ignore unknown fields and tolerate absent optional fields from the first release. A reader that rejects unknown fields makes every future addition a breaking change and forces a version bump for work that should have been free.

## Expand, migrate, contract

The only safe shape for a breaking change is three non-breaking changes. Each numbered step is a separate deploy that must be independently revertible.

1. **Expand.** Add the new field, column, table, or message variant. Nullable or defaulted, written by nobody, read by nobody. Deploy.
2. **Dual-write.** Every writer populates old and new. Old remains authoritative. Deploy, then let it soak long enough to cover the longest-lived consumer, retry queue, or cached payload.
3. **Backfill.** Fill the new representation for pre-existing data, then verify it against the old before trusting it.
4. **Switch reads.** Readers move to the new field, with a fallback to the old when the new one is absent. Deploy. This is the reversible checkpoint: if the new path is wrong, revert this deploy alone.
5. **Stop writing old.** Remove the old write, keep the old data. Deploy. Wait out the retention of anything that might still be replayed.
6. **Contract.** Drop the old field and reserve its identifier permanently.

Steps 5 and 6 belong to a later release than step 4 — always. Compressing them into one deploy is what turns a routine change into an outage, because it eliminates the state in which the old reader still works.

## Additive-only rules

| Change | Safe? | Condition |
|---|---|---|
| Add an optional field | Yes | Readers ignore unknowns; absent must be meaningful |
| Add a required field | No | Old data has no value for it; make it optional with a defined absent-case |
| Add an enum value | Only if | Every reader had an explicit unknown branch before you shipped it |
| Widen a type (int32→int64, add a union arm) | Usually | Old readers may still truncate or reject; verify the reader, not the schema |
| Narrow a type, tighten a constraint | No | Existing data violates the new rule by definition |
| Remove a field | No | Deprecate, stop writing, reserve; deletion is step 6, not step 1 |
| Change units, timezone, precision, or nullability semantics | No | Silent corruption with no error anywhere — the worst class |
| Rename | No | It is add plus dual-write plus backfill plus remove |

A field's meaning is part of its contract. Changing seconds to milliseconds, local time to UTC, gross to net, or "empty means all" to "empty means none" is a breaking change that no type checker, schema validator, or test of the schema itself will catch. When meaning changes, add a new field with a new name and evolve to it — never redefine a name in place.

## Optional, defaulted, nullable

Three different things, routinely conflated, and the confusion is the source of most "why is this zero" bugs.

| Kind | Wire/storage state | Reader sees | Use for |
|---|---|---|---|
| Optional | Absent | "Not provided" — distinguishable from any value | New fields; anything where "unset" is a real state |
| Defaulted | Absent, filled by the reader | A concrete value indistinguishable from one that was written | Only when the default is correct for all historical data |
| Nullable | Present, explicitly null | "Known to be nothing" | Domain values that are genuinely and deliberately empty |

A reader-side default hides the difference between old data and a real value, which means you can never later ask "which rows predate this field." When that question matters, use optional and keep the absence. Writer-side defaults freeze at write time and are safe against later default changes; reader-side defaults change retroactively for all historical data the moment someone edits the constant.

## Renaming and versioning

A rename is add, dual-write, backfill, switch reads, stop writing, remove — the same six steps, with the old and new name both live for the middle four. There is no atomic rename of a contract that has more than one deploy unit.

| Versioning strategy | Cost | Use when |
|---|---|---|
| No version, additive only | Requires permanent discipline; the schema accumulates deprecated fields | Default; correct for most internal contracts |
| Version field inside the payload | Every reader branches; branches never get deleted | Formats stored long-term where the reader must dispatch |
| Versioned endpoint, topic, or queue | Full duplicate code path and test matrix per version | External consumers you cannot coordinate with |
| Content negotiation | Cache keys, routing, and debugging all become version-aware | Public APIs with a contractual deprecation policy |

Every live version is a permanent code path, a test matrix multiplier, and a support obligation. Two versions is a strategy; four is an unfunded liability. Pick a sunset date before shipping v2 and put usage metrics per version behind an alert, because you will not be allowed to delete a version you cannot prove is unused.

## Reserve what you remove

When a field, column, tag number, or enum ordinal is removed, mark the identifier reserved in the schema and never reuse it. Reuse is a silent data-corruption bug: archived rows, replayed messages, and old backups still carry the old identifier, and a new field wearing the same identifier decodes that data into the wrong meaning with no error. This applies to positional tag numbers, column names in stores that resolve by name, enum ordinals in formats that serialize the integer, and API field names any client may still send. Keep the reservation in the schema file, next to the live fields, where the next person will see it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Rollback breaks, forward deploy was clean | Forward compatibility never tested; old code cannot read new data |
| Errors only during the rollout window, then clean | Two versions ran simultaneously and only one direction was safe |
| A field is zero or empty for old records and nobody knows if that is real | Reader-side default erased the difference between absent and set |
| Consumer crashes weeks after a change | A replayed or long-retained message carried the old shape |
| Data decodes into the wrong field with no error | A removed identifier was reused |
| An enum value causes a crash in a downstream service | Readers had no unknown branch when the value was added |
| Off-by-1000 or off-by-hours arithmetic | Units or timezone semantics changed under an unchanged field name |
| The change cannot be applied to the existing store at all | The field was added as required with no default, over records that predate it |

## Red flags

- "Nobody uses that field" without a query, log, or metric proving it
- Add and remove in the same pull request, or steps 4 through 6 in one deploy
- "We will deploy both services at the same time"
- Reusing an identifier because it is free
- Repurposing an existing field because it happens to be unused
- A schema change with no plan for data already written in the old shape
- Treating the strictness of a validator as a substitute for reader tolerance
