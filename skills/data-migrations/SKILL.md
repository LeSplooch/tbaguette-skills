---
name: data-migrations
description: Use when backfilling, transforming, re-encoding, or relocating data that already exists at a size too large for one transaction, when a one-off script must sweep millions of records, when a migration was killed partway and must resume without double-applying, when it has to run alongside live traffic, when a backfill causes lock contention or replication lag, or when a data change has no rollback.
---

# Data migrations

## Overview

A long-running data migration is a program that will be interrupted — by a timeout, a deploy, an OOM kill, or a human with a keyboard. The half-run state is not an edge case to handle at the end; it is the state you design for from the first line. Everything else in this skill follows from that.

## When to use

- Backfilling a new field, re-encoding values, splitting or merging records, changing units or formats
- Copying or moving data between stores, tables, collections, topics, buckets, or file layouts
- Any sweep large enough that a single transaction would time out, exhaust memory, or hold locks past a few seconds
- A migration that must run while production writes the same data
- A migration that was killed partway and someone is about to re-run it from the top
- Not for: changing the shape of the contract itself — that is `schema-evolution`, and it sets the order this skill's own live-traffic steps follow

## Before writing the loop

1. **Count and measure.** Get the exact row count and the per-record cost from a sample of at least 1,000 real records, including the worst decile — not the average, since large blobs and pathological rows dominate wall clock. Extrapolate to total runtime.
2. **Decide on the number.** Under a minute against a copy means it can be a single pass. Anything above roughly ten minutes, or past ~100,000 records, requires batching, a cursor, and throttling — this is not a judgment call.
3. **Dry run.** A mode that reads everything, computes every transformation, writes nothing, and reports counts by outcome: would-change, already-correct, would-fail, ambiguous. The ambiguous bucket is the one that matters; if it is non-empty, you do not yet understand the data.
4. **Rehearse on production-shaped data.** A restored copy at production scale, not a seeded dev database. Migrations fail on the data nobody knew existed: nulls in a column believed non-null, duplicate keys, invalid UTF-8, records from a decommissioned code path, values written before a validation rule existed.

## Batching and resumption

- **Ordering key** must be unique, immutable, and indexed. A timestamp is not unique; an `updated_at` cursor is disqualified outright when the migration itself touches that column, because rows move behind the cursor and are visited twice or never.
- **Keyset pagination**, never offset. "next N records ordered by key, where key > last-seen" stays constant-cost per batch; skipping N records to reach the next page degrades to quadratic and quietly turns a two-hour job into a two-day one.
- **Persist the cursor outside the process** — a control row, a small state table, a file — updated in the same transaction as the batch where the store allows it. A cursor in memory is a cursor that dies with the process, which is the only way the process ever ends.
- **Batch size** tuned so one batch commits in well under a second and touches few enough rows to avoid lock escalation. Start at 500–1,000 and adjust by measurement. Commit per batch; a migration inside one transaction is a migration that cannot be resumed and will block everything else while it fails.
- **Record progress and outcome per batch**, not just at the end: batch range, count changed, count skipped, duration, errors. This is how you answer "how far did it get" after the kill.

## Idempotency

Re-running a batch must be a no-op, because after any crash you cannot know whether the last batch committed.

- Prefer transformations that are naturally idempotent: assignment, not increment; set-to-computed-value, not append. If the operation reads `x = x + 1`, it is disqualified and must be rewritten as a function of an immutable source.
- Where the transform is not naturally idempotent, guard it: a selection predicate that matches only unprocessed records (the new representation still unset), a compare-and-set on a version, or a processed-marker written in the same transaction as the change.
- Never use "the migration already ran" as a global flag. Partial runs are the normal case, so the unit of "already done" is the record, not the job.
- The most expensive version of this bug is a partial run followed by a full re-run that double-applies a delta to records that were already correct — silent, uniform, and undetectable without the original values.

## Verifying

Verification is designed before the migration runs and executed both per batch and in aggregate, because verification saved for the end is verification that gets skipped when the job finally finishes at 3 a.m.

| Check | What it catches | When |
|---|---|---|
| Source vs destination counts | Rows dropped, filters wider or narrower than intended | Per batch and total |
| Aggregate checksums per bucket (sum, hash of ordered values) | Systematic transformation errors, silent truncation | Per batch |
| Sampled record comparison, old value preserved beside new | Wrong logic on real content | Continuous, ≥1,000 records |
| Boundary set: nulls, min/max, oldest, newest, non-ASCII, largest payload, duplicates | The cases the sample will not contain | Before and after |
| Invariants that must hold regardless (totals, referential integrity, uniqueness) | Everything else | After |

Decide in advance what a mismatch means. The acceptable count is normally zero; if it is not zero, the tolerated categories must be enumerated and counted before the run, not rationalized after. A mismatch stops the migration — it does not get logged and stepped over, because the second mismatch will have the same cause and the fiftieth will be why you restore a backup.

## Running against live traffic

The dual-write-before-backfill-before-cutover order is `schema-evolution`'s expand/migrate/contract sequence — this migration follows it rather than restating it; reversing that order is the most common way an online migration corrupts data. Two things belong here instead, specific to running the backfill step itself while production keeps writing:

**Never clobber a live write.** Update only where the new representation is still unset, or compare timestamps and yield to the newer value. A backfill that unconditionally overwrites will reverse concurrent user edits, and the users who notice are the ones who edited during your window.

**Throttle against a live signal**, not a fixed sleep: replication lag, request latency percentile, lock waits, error rate, or queue depth. Pause above a threshold, resume below it — a fixed delay tuned at 2 a.m. is wrong at 9 a.m. Provide a kill switch the process polls between batches, stored where an on-call engineer can flip it without access to the terminal that launched the job. A migration nobody but its author can stop is an incident with a delay fuse.

## There is usually no rollback

Once the old value is overwritten it is gone, and "revert the migration" is a script that must be written, tested, and run at the moment everyone is most tired. Three substitutes, in order of preference:

- **Never destroy in place.** Write the new representation to a new location and leave the old one untouched. Cutover becomes a config change, and rollback becomes the same config change in reverse.
- **Preserve the original** — a shadow column, a copy table, an export — for a stated reversibility window (30 days is a reasonable default), with a scheduled deletion that someone owns. An undated "we can clean it up later" copy becomes a permanent, unowned cost and a compliance exposure.
- **Forward-only fixes.** Accept that the correction is another migration, and make sure the current one records enough per-record history to write it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Job restarts from the beginning after a crash | Cursor lived in process memory |
| Progressively slower batches, then a stall | Offset pagination, or an index invalidated by the writes themselves |
| Some records processed twice, some never | Ordering key not unique or not immutable; often a timestamp the migration mutates |
| Production latency spikes during the backfill | No throttle, batches too large, or lock escalation on the working set |
| Values doubled or shifted by a constant | A non-idempotent transform re-run after a partial failure |
| Users' recent edits silently reverted | Backfill overwrote rows a live write had already updated |
| Counts match but content is wrong | Verified only cardinality, never a checksum or sampled comparison |
| Migration succeeds in staging, fails in production | Rehearsed on seeded data lacking real nulls, duplicates, and encodings |
| Nobody can stop it | Kill switch was ctrl-C in a terminal that has since closed |

## Red flags

- "It should only take a few minutes" with no count and no per-record measurement
- A single transaction over the whole table
- Running the real thing before a dry run has reported an empty ambiguous bucket
- Re-running from the top after a failure "to be safe"
- Backfilling before dual-write is live
- The rollback plan is "restore from backup" for a store that is still taking writes
- Verification described as "we will spot-check a few afterwards"
- A `sleep` between batches chosen by intuition instead of a measured load signal
