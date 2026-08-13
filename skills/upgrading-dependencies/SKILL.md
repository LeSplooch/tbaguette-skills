---
name: upgrading-dependencies
description: Use when dependency versions have fallen behind, when a security advisory names a package in use, when planning a major version bump or a runtime upgrade, when a lockfile diff brings transitive changes nobody asked for, when an upgrade breaks something the test suite did not catch, when deciding whether to batch bumps or take them one at a time, or when a library has become unmaintained and replacing it is on the table.
---

# Upgrading dependencies

## Overview

Staying current is either a cheap routine or one enormous forced upgrade during an incident; there is no third option, and the choice is made by default every week nobody upgrades. Batch what is boring, isolate what is risky, and know the rollback before the merge.

## When to use

- A routine bump, a scheduled upgrade sweep, or an automated update PR needs review
- An advisory, audit, or end-of-support notice names something in the tree
- A major version, framework, or language runtime bump is being planned
- An upgrade landed and something broke that no test covers
- A library is unmaintained, forked locally, or blocking other upgrades

Not for: vetting a package's supply-chain risk before adopting it (`auditing-dependencies`), replacing a system over many releases with both paths live (`incremental-migration`), or backfilling stored data after a format change (`data-migrations`).

## Cadence and batching

| Class | Grouping | Cadence | Merge rule |
|---|---|---|---|
| Patch | batch all of them | weekly | auto-merge on green |
| Minor | batch by ecosystem or owner | every 1–2 weeks | one reviewer skims each changelog |
| Major | never batched, one per change | one at a time | its own review, its own deploy |
| Security advisory | smallest possible diff | out of band, immediately | patch-level bump only, not a version jump |
| Language runtime or toolchain | alone | its own release | canary before full rollout |

A batched change that goes red gets split, never debugged as a batch — bisecting a 40-package bump costs more than opening 40 single-package changes. Keep direct dependencies within about 30 days of current. Two majors behind stops being an upgrade and becomes a migration, with the cost profile of one.

## Read before you diff

Work in this order: release notes, then the deprecation and removal sections specifically, then the migration guide, then the issue tracker filtered for regressions filed since that release. Only then look at the code.

- Skipping versions means reading every intermediate release's removals, not just the target's. The warning you never saw was printed by the version you skipped.
- Before running anything, search your own code for each removed symbol, renamed export, changed config key, and changed default.
- Semantic versioning promises API compatibility and says nothing about behavior. The genuinely dangerous release is a minor that changes a default: timeout, retry count, pool size, certificate verification, encoding, precision, sort stability.
- A changelog that is just a list of commit subjects is itself a finding. The maintainer is not tracking what they broke, so budget more verification, not less.

## Mechanical versus behavioral

Classify the upgrade before starting. Failure to classify means the changelog has not been read.

| | Mechanical | Behavioral |
|---|---|---|
| Looks like | renamed symbol, moved import path, changed signature, removed option, stricter types | changed default, changed ordering, new serialization, different error type, altered concurrency or precision |
| Found by | compiler, type checker, linter, immediately failing tests | nothing you own; production finds it |
| Effort scales with | number of call sites; often codemod-able | how deeply the behavior is relied on |
| Failure mode | loud and immediate | silent and delayed |
| Rollout | merge on green | canary with a named metric to watch |

## Verify past the test suite

The suite encodes what someone thought of before the upgrade, and the upgrade changed what is worth thinking about. Green is necessary and never sufficient.

Check the seams tests rarely reach: process startup and configuration parsing; the error, timeout, and cancellation paths; the shape of logs, metrics, and traces (a "just a logging library" bump silently reshapes every dashboard and alert built on it); serialized output compared byte-for-byte against a pre-upgrade sample; artifact size; steady-state memory and startup time; licence changes.

Read the whole lockfile diff, not the manifest line you edited. The upgrade you requested is rarely the whole diff — resolution, deduplication, and hoisting move versions of packages you never named, and that is where the surprise usually lives.

## Rollout and the limits of rollback

Write the revert into the change before merging: the exact command, and the conditions that trigger it. Then check whether the revert actually works, because rollback is impossible or lossy when the new version has written data or a schema the old version cannot read, changed an on-disk or wire format, altered a hash or derivation used as a key, rotated key material, or emitted a message external consumers already processed.

For those, use two phases: deploy a version that reads both formats and writes the old one, flip writes in a later release, and keep the read-both capability for at least one release beyond that.

Canary anything that affects runtime behavior: 1% → 10% → 50% → 100%, with a bake at each step covering at least one full traffic cycle — 24 hours for a daily peak, a week if weekly batch jobs touch the dependency. Watch error rate, p99, and steady-state memory, since leaks appear over hours rather than minutes. Ship the upgrade in its own deploy; bundling it with a feature costs you the ability to attribute the regression to either.

## When the dependency is the liability

Any two of these mean budgeting a replacement rather than another upgrade:

- No release in 18–24 months, with open unfixed advisories
- Archived, or the maintainer has said it is unmaintained
- You carry more than two local patches, forks, or monkeypatches against it
- You use under 10% of its surface area
- Its constraint on a shared transitive dependency blocks upgrades elsewhere — the strongest signal, because the cost shows up as upgrades you never connected to it
- Each of the last two upgrades took more than a week

The cheap first move is not replacement, it is wrapping the library behind an interface you own. That converts the eventual swap into a one-file change and is worth doing even if the swap never happens.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Upgrade change is red and hours vanish into it | 30 packages moved at once; nothing isolates the suspect |
| Tests green, production degraded | a behavioral change; the suite only encodes pre-upgrade assumptions |
| "Semver said the bump was safe" | semver constrains API shape, never defaults or behavior |
| A package nobody touched changed version | transitive resolution; the lockfile diff went unread |
| Rollback failed when it was needed | the new version wrote data or schema the old one cannot read |
| Every upgrade of this library hurts | local patches make it a fork with extra steps |
| A removal broke the build without warning | deprecation warnings ran for a year and were never filed as work |
| An unrelated library blocks this upgrade | its constraint pins a shared transitive dependency |
| The emergency upgrade took a week | four years of skipped upgrades arrived as one jump, during an incident |

## Red flags

- "We'll upgrade when we need to" — the need arrives as an advisory with someone else's deadline
- A major version batched with anything at all
- Merging because the pipeline is green, with no account of what changed
- A version override or pin added with no comment naming the condition for removing it
- A deprecation warning acknowledged and not filed as work
- The upgrade and a feature riding in the same deploy
- Pinning a version to make a failure go away without recording what the failure was
