---
name: incremental-migration
description: Use when replacing a system, library, API, storage engine, or data format that is already in use, when a change is too large to land in one release, when planning a cutover from an old path to a new one, when old and new implementations are both live and neither is finished, when a change touches hundreds of call sites, or when a rollback would leave data written by the new path unreadable by the old.
---

# Incremental migration

## Overview

Any large replacement ships as a sequence of individually deployable, individually revertible steps. The big-bang cutover is a plan that assumes you will not be interrupted, that nothing will be discovered mid-flight, and that rollback is a button — and all three assumptions fail together, at the worst moment.

## When to use

- Replacing a subsystem, dependency, protocol, or storage layer that has live traffic
- A change whose full form would touch more code than can be reviewed or reverted at once
- Two implementations are already live and the second one has stopped advancing
- Deciding when it is safe to cut over from an old path to a new one
- Not for: the internals of a data backfill or a schema version sequence — see `data-migrations` and `schema-evolution`. Not for the toggle mechanism itself — see `feature-flagging`.

## Two shapes, and which applies

**Expand–migrate–contract** — for interfaces, schemas, formats, and anything with readers and writers. Add the new thing alongside the old (purely additive, backward compatible), move writers then readers, then remove the old. Each phase deploys separately.

**Strangler fig** — for whole subsystems. Put a seam in front of the existing implementation *first* — a façade, router, adapter, or proxy that changes nothing — and ship it. Then move one capability at a time behind that seam until nothing routes to the old implementation, and delete it. The seam ships before any migration work, as its own change, with its own verification that behavior is unchanged.

A field rename under expand–migrate–contract, as concrete as it gets:

```
1. add new field, optional/nullable                       deploy
2. write both, read old                                   deploy
3. backfill existing rows; verify equality, not completion
4. read new, still write both                             deploy
5. stop writing old                                       deploy
6. drop old field                                         deploy
```

Six deploys where the naive plan had one, and every one of them is revertible by redeploying the previous artifact. **The deploy order and the rollback order are opposites**, which is why a schema change and the code requiring it never ship together: rolling back the code leaves the schema ahead, which is survivable, while rolling back the schema under live code is not. Hold each intermediate step for at least one full release cycle so a tested rollback target actually exists.

## Running both and deciding to cut over

Serve the old path, execute the new one in shadow, compare, log divergence. Cut over on evidence, not on a feeling that it looks fine:

| Gate | Threshold |
|---|---|
| Divergence rate | Below a number stated in advance, and every remaining class of divergence individually explained — "small" is not a gate |
| Coverage | At least one full business cycle, including the peak and the period-end that the median hour never shows |
| Cost and latency | Measured on the shadow path under real traffic, not on synthetic load |
| Failure behavior | The new path has been observed failing, and the fallback observed working |

Normalize before comparing or the signal drowns: timestamps, generated identifiers, collection ordering, and floating-point representation diverge legitimately on every single call. Count *categories* of divergence, not raw mismatches — one unexplained category matters more than ten thousand instances of a known one.

Cut over by configuration, not by deploy, so the reversal costs seconds. Ramp by percentage or cohort. Keep the old path warm and reversible for a stated window, and schedule its deletion on a date chosen at cutover time, while everyone still cares.

## Keeping the migration observable

The progress number must be derived from the system. A hand-maintained tally is a number nobody can verify, therefore nobody trusts, therefore nobody updates.

- **A counted marker** — a deprecation annotation or lint rule on the old API, counted by CI on every build, so the remaining-call-sites figure is a build output.
- **A runtime counter on the old path, labeled by caller.** This is the only instrument that finds the callers you did not know existed, and there are always callers you did not know existed.
- **Burn-down over time**, where the derivative matters more than the level. Flat for two iterations means the migration is dead, and dead is a decision to make explicitly, not a pause.
- **An alarm when the old-path counter rises.** Someone just added a new call site to the thing you are removing.

## Not becoming the permanent half-migration

Two live implementations, each half-understood, with every new feature written twice, is the most common terminal state of a migration and the most expensive place in any codebase to work.

- Name an owner and a completion date **before step 1**, not after step 4.
- Schedule the contract phase as work with a ticket. "Cleanup when we have time" is a decision to keep both forever, stated politely.
- Enforce "no new call sites of the old path" mechanically — a lint error, a build failure, a CI count that may not increase. Social enforcement decays in about three weeks.
- Migrate the weirdest low-traffic caller first, to buy unknowns cheaply. Never start with the highest-traffic one; you learn the same lessons at the maximum possible cost.
- Budget the last 5% as its own project. The stragglers are strange for a reason, and that reason is the whole remaining risk.
- If remaining call sites have not dropped in two iterations, either re-staff it or abandon it and delete the new path. Carrying both is worse than either finishing or reverting.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "We will switch everything in one release" | Plan has no state between "old" and "new", so it has no rollback either |
| Rolling back the cutover corrupts or loses data | Dual-write phase skipped; the old path cannot read what the new one wrote |
| Backfill reported done, new path returns different answers | Backfill verified by completion count instead of by equality |
| Shadow comparison drowns in noise, gets muted | Raw outputs compared without normalizing legitimate divergence |
| New path shipped; old one still there a year later | Contract phase was never scheduled as work |
| Unknown callers keep surfacing after cutover | No runtime counter on the old path before the decision |
| Every feature now implemented twice | The half-migrated state was accepted as the normal working condition |
| Migration "90% done" for the third month | The last 5% was budgeted as a rounding error |

## Red flags

- "It is cleaner to just do it all at once"
- "We will remove the old path after the launch"
- "Nobody uses the old endpoint" with no counter proving it
- "The migration is basically done" repeated across months
- Deploying a schema change together with the code that requires it
- A migration with no named owner, or with an owner who has left
