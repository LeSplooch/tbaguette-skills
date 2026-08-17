---
name: auditing-dependencies
description: Use when adding, upgrading, replacing, or removing a third-party package or library, when a lockfile diff adds transitive entries, when a vulnerability scanner or advisory alert fires and needs triage, when a dependency looks unmaintained, abandoned, or has changed owners, or when weighing supply chain risk, install and postinstall scripts, typosquatting and dependency confusion, vendoring, pinning, mirrors, and provenance.
---

# Auditing Dependencies

## Overview

Every dependency is code you ship, run at your own privilege, and never reviewed — and its dependencies are code you did not even choose. Adoption is the cheapest moment to say no and the only moment when saying no costs nothing.

## When to use

- Adding or replacing a package, or choosing between two libraries that do the same thing.
- A pull request's lockfile diff adds transitive entries, or a major version upgrade is proposed.
- A scanner, advisory feed, or audit command fails a build and the finding needs triage; or a dependency has gone quiet, been archived, or changed maintainers.
- Not for: what the code may do at runtime once installed (`least-privilege-design`); how it must treat the data it parses (`handling-untrusted-input`).
- Not for: actually carrying out a version bump once it's decided — `upgrading-dependencies` covers that half; this skill covers whether to adopt, and the quarterly sweep for a dependency that's gone quiet.

## Adoption review — ten minutes, before the first install

Ask in order and stop at the first disqualifier.

| Check | What to look at | Disqualifier |
|---|---|---|
| Need | Lines of your code it replaces | Under ~100 lines you could write and test — you just bought an upgrade treadmill for a helper |
| Transitive weight | The **resolved tree**, not the direct entry | A leaf utility pulling 40 packages is a statement about its author's judgment, not just its size |
| Maintenance | Release cadence, issue response, **how many humans hold commit rights** | One maintainer and no release in 18 months, on anything that parses input or opens sockets |
| Footprint | Does it open sockets, touch the filesystem, spawn processes, load native code, or phone home | Any of those in a library whose stated job needs none of them |
| Install-time execution | Does it run a script at install or build | Yes, and the script is not a compile step you can read and explain |
| Escape cost | How many files would import it; whether its types would enter your domain model | Its types spread past one adapter — removal becomes a rewrite |
| Provenance and license | Does the repository linked from the registry correspond to the published artifact; is there signing or build attestation; do the terms fit how you distribute | No source link, a repository that does not match the artifact, or unclear licensing |

When a dependency lands in the "may need replacing" tier — one maintainer, pre-1.0, vendor-specific, or unmaintained — put it behind a single adapter file. That turns removal from a quarter into a day.

## Lockfiles and reproducible resolution

A lockfile pins the full transitive graph with integrity hashes. Without one, two builds of the same commit ship different code, and both "works on my machine" and "we are not running the vulnerable version" become unverifiable claims. Applications commit the lockfile; libraries keep one for their own CI even when consumers resolve independently.

- **CI must install in frozen/locked mode** — the mode that fails when manifest and lockfile disagree rather than silently re-resolving. Without that flag the lockfile is documentation.
- Every entry needs an integrity hash and the installer must verify it. Hash verification is what turns a compromised or replaced artifact into a build failure instead of a silent substitution.
- Version ranges in the manifest are fine when the lockfile is committed and CI is frozen; they are a supply-chain hole the moment either is missing, because new code then arrives on an unrelated `install`.
- **Read the lockfile diff.** A one-line manifest change that produces 60 new lockfile entries is the actual change under review. This is the most-skipped step in dependency management.
- Prefer resolution that fails on conflict over silently installing two copies — with two copies, "we patched it" can be false for one of them.

## Transitive dependencies are the majority of the risk

Depth two and beyond is typically 80–95% of the shipped third-party code and 100% of the packages nobody evaluated. Your direct list is a review artifact; the lockfile is what ships. Each direct dependency's author performed your adoption review for you, at their standards, against their threat model.

Controls that work: treat the total resolved package count as a reviewed number and crossing it as a decision; use the ecosystem's override/resolution mechanism to force one patched version of a shared transitive package; prefer libraries that advertise few or zero runtime dependencies. If your tooling cannot print the complete tree with versions and hashes, fix that first — a tree you cannot enumerate is a tree you cannot audit.

## Install and build scripts are arbitrary code execution

Install hooks run as the developer or as the CI runner: home directory, signing and SSH keys, cloud credentials in the environment, the pipeline token with write access. They execute **before any test, scan, or review**, which makes them the highest-value target in the chain.

Controls in order of effectiveness: disable install scripts globally and allowlist the few that genuinely compile native code; install inside a container with no credentials mounted and no network beyond the registry; give dependency installation its own minimal identity in CI; keep credential-bearing steps in jobs that never install from an untrusted manifest. Never run an install in a job that holds publish tokens or administrative cloud credentials. Restoring from cache is not safer than installing — a poisoned cache entry is the same execution with less scrutiny, so key caches by lockfile hash.

## Triaging an advisory: reachability first

Each step can end the triage. Order the work so effort follows risk.

1. **Reachability.** Is the vulnerable code on a path you can reach? Check three things: is the package a runtime dependency or only build/test; is the specific vulnerable API called by you or by the direct dependency that pulled it; is the affected feature enabled in your configuration. Most alerts die here.
2. **Exposure.** Is that reachable path fed by untrusted input, and from where — pre-auth request, authenticated user, admin only, or a file an operator supplies? A parser flaw reachable only from config you write is a different item from one reachable from an upload.
3. **Fix availability.** A patch release exists → upgrade; the upgrade is cheaper than continuing the analysis. No patch → in order: override the transitive version, disable the affected feature, add a compensating control at the boundary, fork or vendor with the patch, replace the dependency.
4. **Record the decision** with the advisory identifier and the reasoning. Untracked triage is repeated on every scan, and the third repeat is when someone silences the whole rule.

Suppressions are per-advisory with an expiry date. A blanket ignore is how the next real finding is missed. Severity scores are computed for the worst plausible deployment of that library, not yours: a critical in an unreachable path outranks nothing, and a medium on a pre-auth path outranks most criticals.

## Name confusion is an adoption-time check

Typosquatting and its harder variants: a hyphen or underscore swap, a scoped versus unscoped pair, the same name in a different registry, a plausible "successor" name, and **dependency confusion** — a public package matching the name of an internal one that was never published.

- Copy package names from the project's own documentation — never from a search result, a chat message, a blog post, or a generated suggestion — and confirm the namespace, repository, and download signals match the package you meant.
- Dependency confusion is fixed by configuration, not vigilance: bind your internal namespace to your internal registry and never configure a public registry as a fallback for internal names. It is the one attack in this list that succeeds with nobody making a mistake at the keyboard. Reserve your internal names publicly where the ecosystem allows it.
- This check does not exist at runtime. Once installed, a squatted package is indistinguishable from the real one.

## Vendoring and pinning

| Approach | Buys | Costs | Use when |
|---|---|---|---|
| Range + committed lockfile | Routine patch flow | Requires frozen CI installs and actual lockfile review | Default |
| Exact pins everywhere | Deterministic, no surprise resolution | Manual upgrade work; drifts behind security patches | Regulated builds, firmware, anything slow to redeploy |
| Vendored source in tree | No install-time fetch, reviewable diffs, offline builds | You own every merge; upgrades become patch management | Few dependencies, air-gapped or high-assurance |
| Internal mirror or proxy | Availability, one chokepoint for scanning and allowlisting, immunity to upstream deletion | Infrastructure to run | Once the org is past a handful of engineers |

Pinning without a scheduled upgrade cadence converts supply-chain risk into unpatched-vulnerability risk. Batch upgrades roughly monthly on a quiet day beats emergency upgrades under an advisory.

## The unmaintained dependency is a slow-motion incident

Nothing fires when a project is abandoned, which is the whole problem — detect it deliberately. Quarterly, list dependencies with no release in 12 months and no commit in 18. The escalation runs: releases stop → security issues get no response → repository archived → **ownership transfers to a stranger** → package deleted or renamed. The transfer is the dangerous one: an abandoned package with an existing install base and a new, unknown owner is the classic supply-chain acquisition, and the first release after a long silence is the moment to look. Treat a maintainer handoff as a fresh adoption review, not an upgrade.

Options, in order: replace with a maintained equivalent; absorb it — read the actual source size first, it is usually smaller than the fear of replacing it; fork and maintain; vendor and freeze at the feature set you use.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Scanner alerts ignored in bulk | Triaged by severity rather than reachability; the queue filled with unreachable findings and lost credibility |
| "We upgraded" but the vulnerable version is still installed | Two copies in the tree; the transitive one was never overridden |
| Lockfile committed, CI still resolves new versions | Install not run in frozen/locked mode |
| Build breaks when a package is deleted upstream or a registry is down | The supply chain includes uptime you do not control and no mirror exists |
| A one-line utility pulled in a large tree | Reviewed the direct package, never the resolved graph |
| CI compromised through a dependency | Install ran in a job holding publish or cloud credentials |
| The wrong package installed under a plausible name | Name typed from memory or copied from a search result |
| A dependency cannot be removed | Its types leaked into the domain model; no adapter boundary was ever drawn |

## Red flags

- "It's only a dev dependency" — install hooks run at your privilege regardless of which section lists it.
- Download counts or stars used as the entire review.
- Adding a dependency for something under ~100 lines.
- Spending an hour on reachability analysis when a patch release exists.
- "We'll deal with the unmaintained one when it breaks."
- A dependency being added during an incident, at speed, from a search result.
- A public registry configured as a fallback for a private namespace.
