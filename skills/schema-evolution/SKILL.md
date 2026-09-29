---
name: schema-evolution
description: Use when changing a contract that is already in production — adding, removing, renaming, or retyping a field in a database schema, serialized format, stored document, API payload, or queue message. Also when a rolling deploy breaks deserialization, when old consumers cannot read new data, when a rollback fails on data the newer version wrote, when adding an enum value, when planning a version bump, or when a field goes missing after some component read a record and wrote the whole thing back. Also use when writing or reviewing the check that decides which stored versions a reader may accept, or when adding authentication to a channel that already-running clients depend on, especially one that tells them to update.
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
- Not for: moving or rewriting the data that already exists — that is `data-migrations`, a separate discipline with its own failure modes that follows the order this skill sets
- Not for: designing a new interface that has no deployed callers yet — that is `designing-apis`; this skill starts once a contract has callers you can no longer redeploy

## Backward and forward compatibility

| Term | Definition | Broken by |
|---|---|---|
| Backward compatible | New code reads old data | Requiring a field that old writers never produced |
| Forward compatible | Old code reads new data | Removing or repurposing a field old readers still use; strict readers that reject unknown fields |

A rolling deploy needs both at the same time, because both versions run concurrently for the length of the rollout, and every message or row written during that window is read by whichever version happens to pick it up. A rollback needs forward compatibility specifically: the older binary you are rolling back to must survive data the newer one already wrote. This is the case teams skip, and it is why "the deploy went fine" and "the rollback corrupted things" are the same incident.

Design readers to ignore unknown fields and tolerate absent optional fields from the first release. A reader that rejects unknown fields makes every future addition a breaking change and forces a version bump for work that should have been free.

**"Ignore" is the right instruction for a reader and the wrong one for anything that writes back.** A component that loads a record, changes part of it, and stores the whole thing again is a reader and a writer at once — a settings screen, an editor, a config rewriter, a normalizing proxy, an admin tool, a migration script that rewrites whole rows. Ignoring a field there does not mean tolerating it; it means deleting it on the next save. Forward compatibility asks that role to *preserve* what it does not understand — carry unknown fields through untouched and re-emit them — which is strictly stronger than tolerating them. Formats that retain unknown fields explicitly exist to supply exactly this, and a hand-rolled struct mapping never does it by accident.

The failure is silent at every step, which is why it survives to production: the read succeeds, the edit succeeds, the write succeeds, the schema still validates, and the field is simply gone — removed by the one component in the system that was never taught it existed. Nothing in the round trip is an error, so nothing logs one, and the loss is usually discovered by whichever consumer needed the deleted field, at a distance from the writer that caused it. So ask of every writer: does it build its payload from the full record it loaded, or from the subset it happens to render? The second one is a deleter, however carefully its own fields are handled. A partial-update verb where the format offers one (patch rather than replace, a field mask, a merge) makes the answer structural instead of a property of the code you have to keep re-checking.

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

## A new credential is a required field, and can cut off the update notice

Adding authentication to a channel that already has clients is the *Add a required field* row above, and it is an easy row to skip, because refusing clients without the credential is the whole point of the change. Every client already running sends nothing for it, so a server that refuses them breaks compatibility exactly as a required column would.

The break is worse than an outage when the same channel is how an old client finds out it is old: a version field, a relaunch instruction, an update notice. The client learns it must upgrade only by being told, and it is told over the channel just closed to it. It never upgrades, and the wait before the contract step never ends, because the clients it waits on can no longer be moved. The same holds for any tightening of that channel, such as a required handshake field, a new transport or a stricter parser, not only for a credential. So before tightening a channel, list what an old client learns over it that it cannot learn any other way. For clients built from now on, have them ask for the version before they authenticate, so the next tightening cannot strand them the same way.

The expand step is what the table already prescribes for a required field: optional, with a defined absent-case. A client that presents *no* credential gets a reduced, read-only answer: the least it needs to discover it is stale, such as the current version and the relaunch instruction, with every sensitive field blanked. Anything can now ask for that answer, not only old clients, so it has to be safe to hand to anyone. A client that presents a *wrong* credential is refused. An honest client with a wrong credential is usually holding a secret that has since been rotated away, and a refusal is what sends it back to re-read the secret, where a reduced answer would leave it believing it had connected. That is *keep the absence* from the section above, applied to a credential: absent and wrong are different states and get different answers. The reduced answer is itself an old field now, and it is contracted like one, on a runtime count of the clients still using it rather than on a date.

## Renaming and versioning

A rename is add, dual-write, backfill, switch reads, stop writing, remove — the same six steps, with the old and new name both live for the middle four. There is no atomic rename of a contract that has more than one deploy unit.

| Versioning strategy | Cost | Use when |
|---|---|---|
| No version, additive only | Requires permanent discipline; the schema accumulates deprecated fields | Default; correct for most internal contracts |
| Version field inside the payload | Every reader branches; branches never get deleted | Formats stored long-term where the reader must dispatch |
| Versioned endpoint, topic, or queue | Full duplicate code path and test matrix per version | External consumers you cannot coordinate with |
| Content negotiation | Cache keys, routing, and debugging all become version-aware | Public APIs with a contractual deprecation policy |

Every live version is a permanent code path, a test matrix multiplier, and a support obligation. Two versions is a strategy; four is an unfunded liability. Pick a sunset date before shipping v2 and put usage metrics per version behind an alert, because you will not be allowed to delete a version you cannot prove is unused.

### A version guard has to know which kind of change moved the number

This section is a repair, not a technique. The rule above says a meaning change gets a new field name rather than a redefinition in place — and version counters that move on meaning changes exist anyway, in every format old enough to have been evolved by somebody who did not read that rule. This is how to read one you have inherited.

A payload version is one integer standing in for two unrelated kinds of change, and a reader that compares it with an inequality has silently picked one of them. `version <= CURRENT` is the guard almost everybody writes, and it encodes *old data is readable, new data is not* — true for a number that has only ever moved additively, and exactly wrong for a number that moved because a field's meaning changed. Re-rendering an old row with current code applies the new meaning to the old quantity: seconds read as milliseconds, gross read as net, "empty means all" read as "empty means none". That is the silent-corruption row of the additive table above, arriving through the one check that was written to prevent it.

So the readable set follows from the reason the version moved, and only one of the two answers is an inequality. Where the version increments on additive changes, old rows are readable and `<=` is right. Where it increments because a meaning changed, the readable set is **exact equality**, and everything outside it is a migration rather than a read. A constant whose own documented rule is "bump this when a field's meaning changes" therefore cannot be compared with `<=` at all — and a single counter serving both purposes cannot be checked correctly by any comparison, which is the argument for either two counters or for giving the meaning change a new field name, per the rule above, instead of a version bump.

**Then check what fills the field when it is absent**, because the guard is worth no more than the value it reads. A version field with no explicit per-field default, in a format that fills missing fields from a whole-record default, reports *the version of the build doing the reading*. Every unversioned legacy row then claims to be current and sails through the check written to catch it. This is the reader-side default of the section above at its worst: invisible while the constant is still 1, and it converts the guard into a rubber stamp on the day the constant becomes 2 — the first day it was ever needed. The version is the field where the section above's rule is least optional: keep the absence, so a row that never declared a version stays distinguishable from one that declared the current one.

## Reserve what you remove

When a field, column, tag number, or enum ordinal is removed, mark the identifier reserved in the schema and never reuse it. Reuse is a silent data-corruption bug: archived rows, replayed messages, and old backups still carry the old identifier, and a new field wearing the same identifier decodes that data into the wrong meaning with no error. This applies to positional tag numbers, column names in stores that resolve by name, enum ordinals in formats that serialize the integer, and API field names any client may still send. Keep the reservation in the schema file, next to the live fields, where the next person will see it.

Where a column can be written by more than one kind of source, the provenance is part of the contract and needs its own field rather than a shared one — `tracking-data-provenance`.

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
| A version guard passes and the value it admitted is corrupt anyway | The guard was an inequality over a counter that moves when meanings change, so it waved old rows through |
| A version check has never rejected anything | The version field is reader-defaulted, so every unversioned row reports the current build's number |
| Clients from before a security change never update, and nothing tells them why | Authentication was added to the same channel that tells them they are out of date, so the notice is refused along with everything else |

## Red flags

- "Nobody uses that field" without a query, log, or metric proving it
- Add and remove in the same pull request, or steps 4 through 6 in one deploy
- "We will deploy both services at the same time"
- Reusing an identifier because it is free
- Repurposing an existing field because it happens to be unused
- A schema change with no plan for data already written in the old shape
- Treating the strictness of a validator as a substitute for reader tolerance
- A `<=` against a version constant whose documented rule is that it moves when a field's meaning changes
- A version field that inherits a whole-record default instead of being absent when it was never written
- A save path that rebuilds the whole record out of the fields the caller happens to know about
