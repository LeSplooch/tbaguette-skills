---
name: designing-ci-pipelines
description: Use when building or reworking a CI pipeline, when the build is too slow or nobody trusts its result, when a check fails only in CI, when a stale cache produces a wrong result, when deciding which checks block a merge, when retries are proposed to make a build green, when a pull request from a fork needs access it must not have, when a scheduled, nightly, or cron job is added, when a slow-cycle job keeps failing on one missing secret, credential, or tool after another, or when one that was supposed to be running turns out never to have run. Covers stage ordering, enumerating a job's prerequisites in one preflight step rather than one failure per cycle, feedback budgets, cache keys, required versus advisory checks, runner permissions, forcing a scheduled job's first run before trusting it, and reporting age of last success rather than status of last run.
---

# Designing CI pipelines

## Overview

A pipeline is a product whose user is a person waiting, and its budget is their attention. Every design choice is a trade against two failure states: feedback so slow that people stop waiting for it, and results so unreliable that people stop reading them.

## When to use

- Creating a pipeline, adding a stage, or reworking one that has grown organically
- Feedback takes long enough that people context-switch away from the change
- People rerun a red build before reading it, or merge with a failing check
- A failure reproduces only on the runner
- Deciding whether a new check blocks merges, and what a fork's pull request is allowed to touch

Not for: diagnosing one specific intermittent test (`flaky-test-triage`), deciding which tests should exist (`choosing-test-scope`), or making the build itself deterministic (`reproducible-environments`).

## Order by cost-to-signal

The fastest check that can fail must fail first. Ordering is free; parallelism is not.

| Stage | Budget | Catches |
|---|---|---|
| Format, lint, config and manifest validation | under 60s | typos, style, malformed pipeline and schema files |
| Type check or compile | 1–3 min | missing symbols, signature misuse |
| Unit tests | under 5 min, sharded | logic defects |
| Build and package the artifact | parallel with tests when independent | packaging and dependency-resolution breaks |
| Integration and contract tests | 5–15 min | wiring, boundaries, serialization |
| End-to-end, device, or browser | 10–30 min, smallest blocking set | full-stack regressions |
| Fuzzing, perf, full matrix, licence and vulnerability sweeps | nightly or post-merge | low per-commit yield, high cost |

Targets: first failure signal under 2 minutes, median pull-request feedback under 10 minutes, p95 under 20. Past 20 minutes the reviewer has moved on and the change waits a day per iteration.

When a stage busts its budget, work this order: delete checks with no failure in the last six months; shard by measured runtime rather than by file count; select by change impact with a mandatory full run before merge; demote to post-merge and accept the detection latency; then fix the slowness in the code under test. Adding runners is the last move, because it converts a design problem into a recurring bill.

## Caching that is correct

A cache must be a pure function of its key. If a stale entry can produce a green build, the cache is a correctness bug wearing a performance costume — and it is the most confusing failure class in CI, because the symptom surfaces in a step whose name never mentions the cache.

- Derive the key from every input: lockfile or manifest hash, toolchain version, OS, architecture, plus a manual salt you can bump.
- Fallback and prefix restore-keys are precisely where staleness enters. Use them only for content-addressed downloads, where a wrong entry cannot change the output.
- Safe: immutable fetched dependencies keyed by lockfile hash; compiler outputs keyed by a hash covering compiler version, flags, and every source input.
- Unsafe: build outputs keyed by branch name, test-result caches keyed by less than the full input set, anything written back into the source tree, and any cache shared between a fork's pull request and the base branch — that one is a poisoning vector as well as a staleness one.
- Ship a no-cache rerun path and use it as the first debugging step for anything that "only fails in CI". Run one scheduled cacheless build so a from-scratch break cannot hide behind a warm cache for weeks.

## What blocks a merge

Within a stage, run everything and report every failure — the developer fixes them in one pass. Between stages, stop at the first failure, because downstream stages cost more and their signal is usually implied. In a matrix, fail fast when legs are shards of one suite; run all legs when they are different environments, since "one platform or all platforms" is the entire diagnostic value. Always cancel superseded runs for the same branch.

Every advisory check eventually becomes ignored — treat that as a law and design around it. A check overridden or ignored more than 1 time in 10 is either wrong or should be required; fix it or delete it, and never leave the third option running. New checks land advisory with a written expiry of about two weeks and a named owner who decides; at expiry it is promoted or removed. For a large pre-existing violation count, ratchet against a committed baseline so new violations block while old ones do not — that is a required check, not a permanent advisory one.

## Flakes and the lying pipeline

Blanket retries make the pipeline lie: it reports green for a real, intermittent, user-visible defect. Retry at most once, and only at the layer where the flake is genuinely infrastructural — image pull, artifact download, runner eviction — never around an assertion.

Record every retry, because a flake rate is a number and an unmeasured one is an opinion. A test above roughly 1% flake rate goes into quarantine within one business day: still runs, no longer blocks, has an owner and a 30-day deadline, and is deleted at the deadline rather than renewed. A quarantine list nobody empties is just a slower version of ignoring the failure.

## Reproducing a CI failure locally is a design requirement

Every stage must be runnable by a developer in one command, in the same image, without pushing. Enforce it structurally: pipeline definitions set up the runner and call a script; no logic lives in the pipeline file, because logic there is logic nobody can execute locally. When the only way to test a stage is to push, the debug loop becomes 10 minutes per iteration and the history fills with "fix ci" commits.

Give every failure enough to act on: the exact command, the resolved tool versions, and the artifact or log bundle. A failure that cannot be reproduced is debugged twice.

## Secrets, permissions, and untrusted pull requests

- Default the job token to read-only and grant additional scopes per job, never per pipeline.
- A pull request from a fork is attacker-controlled code. It gets no secrets, no write-scoped token, no self-hosted runner, and no elevated run of a pipeline definition it authored.
- Split the flow: an untrusted job builds and tests with no credentials and uploads an artifact; a trusted job triggered on that run holds the secrets, consumes the artifact, and never executes code from it.
- Pin third-party actions, plugins, and images by digest or commit hash. A floating tag is remote code execution with a changelog.
- Pass secrets through the environment, not command lines — process listings and traces leak them — and treat every log line as public. Prefer short-lived federated credentials to long-lived static ones, and gate production deploys behind an approval.
- Review pipeline definitions like production code, and require the change that edits a pipeline to be exercised by that same change, or it lands untested on the default branch by construction.

## A scheduled job is untested code until it has run once

Everything else in a pipeline has an audience. A pull-request check goes red and blocks a merge; somebody is standing there waiting for it. A scheduled job has nobody waiting, and that changes what its silence means: for a PR check, silence is absence of trouble, while for a scheduled job, success and total non-existence produce exactly the same nothing.

So a job that has never once run successfully is indistinguishable from a healthy one on every dashboard that reports last-run status, because a job with no runs has no status to report and renders as blank, neutral, or simply missing from the list. The ways it ends up never-run are all mundane — merged before its credentials were armed, a schedule expression that parses but never matches, disabled at the platform level, pinned to a branch that was later renamed, or suspended automatically because the repository went quiet long enough for the provider to stop scheduling it.

The consequence is what makes this worth a section rather than a bullet. The breakage never announces itself. It surfaces as the absence of whatever the job existed to maintain — the backup that is not there, the certificate that expired, the index that went stale, the dependency audit nobody has seen in four months — and it surfaces after the deadline the job was protecting has already passed.

Two things follow:

- **Force one run when the job lands, rather than waiting for its schedule.** A job's first successful run is the only evidence it can run at all; until then it is untested code that happens to have a cron expression. Treat the change as unfinished until that run is green, the same way `confirming-before-claiming-done` treats any requirement about a condition that has not occurred yet.
- **Report age of last success, not status of last run.** These differ exactly when it matters. A job that succeeded once in March and has been suspended since is green on the second measure and four months stale on the first, and only one of those two numbers would have told anyone. Whatever surfaces job health needs to separate *passing*, *failing*, and *never executed* rather than folding the third into either of the first two.

## Discovering prerequisites one failure at a time costs a full cycle each

A job that has run once is past the section above and straight into the failure that replaces it: it runs, and it fails on the first thing it is missing. You supply that one; the next run fails on the second. Where you can trigger the job by hand, that loop costs minutes and the section above's advice — force a run — is also the answer here, six times over. This section is about the jobs where you cannot. A real release, a job whose inputs only exist at the hour it fires, one that consumes the previous night's output, a provider with no manual dispatch, an environment whose credentials are only injected on the schedule: for those, every missing prerequisite costs a full period, and a list of six that would fit on one screen takes six periods to read, one line at a time.

What hides the shape is that the message names the reporter rather than the requirement. A job missing six credentials fails in seconds inside whichever early step happens to touch the first one, and the error belongs to that step: a checkout complaining about a token, a client refusing to construct, a registry returning 401. Nothing in it is false and nothing in it is a list. Read as a diagnosis it says *you are missing this*, when the true statement is *you are missing this and an unknown number of others* — and the gap between those two readings is every remaining cycle.

So give any job with an expensive cycle a first step that enumerates every prerequisite it will need — each secret, credential, tool, network destination, and mounted path — checks all of them, and reports every failure at once instead of exiting on the first. Two things make that worth more than it looks:

- **An expired credential fails identically to one that was never set.** Both surface as an authorization error from whatever tried to use it, at whatever hour the job fires, with nobody reading. Only a preflight that separates *absent*, *present and rejected*, and *present and accepted* distinguishes them, and that is the difference between a fix and an investigation.
- **Test the preflight against every state it claims to distinguish**, including the ones you expect never to see. A check that has only ever printed *ok* is indistinguishable from a check incapable of printing anything else — `writing-the-failing-test-first`'s rule, and it applies to the instrument you just built exactly as much as to the code it guards.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "It only fails in CI" | an undeclared environment difference, or a stale cache entry |
| A green build shipped a broken artifact | the cache key omitted an input, so a stale entry was reused |
| Nobody reads pipeline output any more | advisory checks with no expiry taught everyone that red is survivable |
| Flake count flat for months | retries made flakes invisible, so nothing was ever owned or fixed |
| "Fix CI" commits accumulate on the default branch | no stage can be run locally; pushing is the only way to test |
| 40-minute pull-request pipeline | stages ordered by history rather than cost-to-signal |
| More runners did not help | the critical path is serial, or the cache misses every run |
| A secret leaked through a pull request | a fork's PR ran with the same permissions as a branch PR |
| The same failure gets debugged twice | logs omitted the command and the resolved versions |
| A scheduled job's dashboard is green and the thing it maintains is months stale | Last-run status reported where age of last success was the question |
| A nightly job has failed for weeks, each time on a different missing secret | No preflight, so each run discovers exactly one prerequisite and the cycle is a night |
| A credential that worked last month now fails and nobody can tell if it was revoked | Absent and rejected produce the same error from the step that used it |

## Red flags

- "Just hit rerun" as the standard response to red
- Adding a retry, a sleep, or a longer timeout to make a specific test pass
- A cache key that does not include the toolchain version
- Logic accumulating in pipeline configuration that exists nowhere a developer can run
- A check that has been advisory for more than a month
- Any job holding secrets that executes code from a fork
- "The build is red, but it's unrelated" said more than once in a week — trust is already gone, and a real failure will be rerun rather than read
- A scheduled job merged and never once run by hand — its first real execution will be unattended, at whatever hour it fires
- Job health shown as pass/fail, with no way to see a job that has never executed
- A job on a slow cycle whose required secrets and tools are written down nowhere the job itself checks
- A preflight or health check that has printed the same word on every run it has ever had
