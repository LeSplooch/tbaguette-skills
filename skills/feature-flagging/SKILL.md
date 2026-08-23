---
name: feature-flagging
description: Use when hiding unfinished work behind a toggle, when planning a gradual rollout, a kill switch, or a dark launch, when gating an experiment or a paid entitlement, when old flags have accumulated and nobody removes them, when flag combinations have made the test matrix unmanageable, when a rollback by flag flip fails, or when choosing between a flag and a separate branch or build.
---

# Feature flagging

## Overview

A flag decouples deploying code from releasing behavior. It is also a permanent branch in the code, in the test matrix, and in the head of whoever is on call — so every flag gets a type, an owner, and an expiry on the day it is created, or it gets rejected.

## When to use

- Shipping incomplete work to production without exposing it
- Ramping a change by percentage or cohort, or needing to disable one without a deploy
- Gating an experiment, a plan tier, a region, or a role
- Cleaning up flags that outlived the feature they guarded
- Not for: moving traffic between two implementations over weeks — the flag is one instrument inside `incremental-migration`. Not for how configuration is stored, layered, and validated — see `configuration-management`.

## Four types, four lifespans

Treating these as one mechanism is the root cause of most flag pain.

| Type | Purpose | Lifespan | Who flips it | Default | Ends when |
|---|---|---|---|---|---|
| Release | Ship dark, ramp, decouple deploy from release | Days to weeks | The team that wrote it | Off | Removed at 100% after a stable period |
| Operational (kill switch) | Shed load, disable an expensive path, degrade deliberately | As long as the subsystem | On-call, at 3am, under stress | The state that is safe when nothing is known | Never removed; tested on a schedule |
| Experiment | Measure a hypothesis against a control | One experiment cycle | Assignment service, not a human | Control | Removed with the result, winner inlined |
| Permission / entitlement | Who is allowed this — plan, tenant, region, role | Permanent product rule | Product or sales operations | Deny | Never; it is authorization, not debt |

Two failure modes fall straight out of that table. Modeling an entitlement as a release flag puts your pricing model into a system whose whole purpose is deleting things — the cleanup process will eventually try to remove your paid tier. Modeling a release flag as entitlement config puts it somewhere with no expiry, so it lives forever.

Kill switches carry an extra requirement: evaluation must not depend on anything that fails during the incident you built the switch for. Resolve locally from cache, fall back to a defined value when the flag service is unreachable, keep the network out of the hot path, and take effect without a deploy — the last one is the entire point. Exercise it on a schedule; an untested kill switch is a comment claiming a capability.

## Flag debt

- **The removal ticket is created in the commit that introduces the flag**, referencing the flag key. No ticket, no flag. Nothing later in the process reliably creates it.
- **The expiry date lives in code or metadata.** Past expiry the build warns; past a grace period the build fails. This is the only mechanism that works; reminders, dashboards, and good intentions all decay.
- **A flag that suspends a rule gets its current value pinned in a test.** Not one that selects between behaviours — both of its states are valid. But where one state is a deliberate, accepted violation, pin it, and make the failure message the handover note: why the deviation exists, what ends it, and that ending it means deleting the assertion. The expiry above fires on a date; this fires the moment someone flips the flag, at the person flipping it. A comment explaining the trade fires never.
- **Removal deletes the flag and the dead branch in one commit**, then verifies no configuration anywhere still names the key. Removing the flag while keeping both branches is how a "removed" flag becomes unreachable code, and how an orphan config key becomes a mystery in the next audit.
- **A release flag past two release cycles, or roughly 60 days, is a live incident waiting.** Its off branch has not executed in weeks, and untested code does not work — meaning the rollback path you are counting on is the least-exercised code in the system.

## The combinatorial problem

N independent boolean flags is 2^N reachable states, and testing them all stops being possible at N=3. The honest rule that replaces the impossible matrix:

1. Test both states of the flag under change, **against current production values of every other flag**.
2. Test the all-off configuration — that is the rollback state, and it must be known-good.

Two configurations, not 2^N. To keep that rule valid, keep flags independent by construction. **When two flags interact, they are one flag with three states, not two booleans** — encode it as a mode or enum so the impossible combination cannot be expressed. Cap simultaneously live release flags in one code path at about three; past that, nobody on call can state what the system is currently doing.

## Defaults that fail safe

The default is what happens when the flag service is down, the key is misspelled, the config has not loaded yet, or the process is starting. Every one of those paths must land on a defined, safe value — and safe means the code path already running in production today.

- New behavior defaults off. Kill switches default to the healthy path, and the disabled path must itself be a valid product state, not a half-configured one.
- Never make the safe state the one that requires a successful network fetch.
- **A misspelled or unknown key must be loud.** Register keys and fail at startup on an unknown one. A silent false looks exactly like "the feature shipped but does not work", and teams lose days to that specific ambiguity.
- **Log resolved flag values alongside errors.** Flag state is an input to every bug report; without it you will debug the branch the user was not running.

## When a flag is the wrong answer

Use a branch, a separate build, or a different technique when:

- The two versions cannot coexist in one binary — incompatible dependency versions, conflicting global state, a different data model
- The conditional would appear in more than about 10 places or cross more than two modules; that is a seam problem, solved by polymorphism or an adapter, not by scattered branches
- The work is a spike likely to be discarded — a flag makes throwaway code permanent by making it cheap to leave in
- The gate is a security boundary; flags are configuration, and configuration is mutable by more people, with less review, than code
- The change alters a persisted format on the write side without the read side handling both — that is a migration, and a flag on it only controls how fast you corrupt data

## Common mistakes

| Symptom | Real cause |
|---|---|
| Flag still present a year after launch | No expiry, no owner, and removal was never scheduled work |
| Flipping the flag off to roll back makes things worse | The off branch has not executed since the flag was added |
| Kill switch did nothing during the incident | Its evaluation depended on the failing component, or it had never been tested |
| Test matrix has become unmanageable | Interacting flags modeled as independent booleans |
| Feature "does not work" for exactly one environment | Misspelled key silently resolving to the default |
| Removing a flag broke production | The wrong branch was kept, or config still referenced the key |
| A deliberate deviation was reverted by accident | Its rationale lived in a comment, and no assertion failed when the flag flipped |
| Cleanup process keeps proposing deletion of a paid feature's gate | Entitlement modeled as a release flag |
| A reported bug cannot be reproduced | Flag values at the time of the error were never recorded |

## Red flags

- "We will remove the flag right after launch"
- "It is just a temporary flag"
- "Default it on so we do not have to change config" — the default *is* the failure mode
- "The flag service will be up"
- A flag name with a version number and no owner
- Adding a flag because the change is risky, without having decided what evidence would justify turning it on
