---
name: reproducible-environments
description: Use when a build or test passes on one machine and fails on another, when an old tag or release can no longer be rebuilt, when onboarding needs undocumented setup steps, when a build breaks though no commit changed, when two builds of the same commit produce different artifacts, or when choosing between a version manager, a container, and a hermetic build. Covers pinning, lockfiles, toolchain declaration, isolation, and determinism. Also use when a verification step launches the real built artifact, or when a check that was supposed to touch nothing may have run against a real profile, store, or credential.
---

# Reproducible environments

## Overview

Every "works on my machine" is an input the build consumed without declaring. Reproducibility is not a property you add at the end; it is the count of undeclared inputs reaching zero, and the only way to know that count is to rebuild and compare.

## When to use

- A build, test, or artifact behaves differently on another machine, in CI, or after a clean clone
- The build broke and no commit changed
- You cannot rebuild the artifact a released version was cut from
- Onboarding a person or a new runner takes more than one command
- Choosing an isolation mechanism, or deciding whether the current one is worth its cost

Not for: runtime configuration and secrets that legitimately differ per environment (`configuration-management`), or moving pinned versions forward on purpose (`upgrading-dependencies`).

## The taxonomy of undeclared inputs

Every failure of this class is one row of this table. Diagnose by elimination, in roughly this order of frequency.

| Undeclared input | How it shows up | Declaration that fixes it |
|---|---|---|
| System package or shared library | missing binary or link error on a fresh host only | an in-repo manifest the setup reads, or a base image |
| Unpinned dependency version | broke overnight with no commit | lockfile committed, install run in frozen mode |
| Toolchain version | different output, or a syntax error on the older compiler | version file in-tree, asserted at build start |
| Ambient environment variable | works in your shell, fails in cron, CI, or a fresh login | explicit allowlist; clear the rest at the entrypoint |
| Network at build time | fails when a registry is slow, a tag moves, or a package is yanked | vendored or content-addressed inputs; no network in the build step |
| Wall clock | artifacts differ byte-for-byte between two runs | a fixed source-date epoch fed to every packaging step |
| Absolute path | debug info, panic messages, and cache keys differ per checkout | path remapping, or a fixed build directory |
| Locale and timezone | sort order, case folding, date and number parsing all shift | pin the locale and timezone in the build entrypoint |
| Host identity (user, uid, hostname) | leaks into archive metadata and embedded build stamps | normalize during packaging |
| CPU feature detection | binary built on a newer host crashes on an older one | set an explicit architecture baseline |
| Iteration or file order | generated code and archive members reorder between runs | sort inputs explicitly; never rely on directory order |

## Declare the toolchain as source, not as instructions

A version a human can get wrong without anything failing is a suggestion, not a pin. The ladder, weakest first: a wiki page; a README command; a version file the tooling reads automatically; a version file plus a bootstrap script that installs it; a fully declared image or hermetic build file.

The rung that matters is the one where a mismatch becomes an error. Assert the expected toolchain version at the start of the build and fail with the exact command to fix it — a silent 2% of contributors on the wrong compiler produces bug reports nobody can reproduce.

The environment definition and its documentation must be the same artifact. Prose setup steps that are not also executable have a half-life of weeks: the code moves, the page does not, and nothing fails when they disagree.

## Lockfiles: the promise and its limits

A lockfile guarantees the same resolution of the dependency graph from the same manifest, gives you a reviewable diff of transitive change, and pins content hashes if the format supports them.

It does not guarantee identical bytes. Post-install hooks run arbitrary code, native components compile against local headers, and platform-conditional dependencies resolve differently per OS and architecture. It also guarantees nothing if the registry is mutable and hash verification is off, or if CI installs in a mode permitted to rewrite the lock.

Rules: commit it; make CI install in frozen mode so drift fails the build rather than silently updating; review the lockfile diff rather than rubber-stamping it; lock applications, publish ranges for libraries.

## A rebuild under the same version is the mutable registry, seen from the other side

The lockfile and isolation advice above is written for whoever *consumes* a published artifact: pin by digest, freeze the lock, never trust a moving tag. All of it assumes the mutability is somebody else's doing. It is worth reading once from the other chair, because a registry becomes mutable at the moment somebody republishes — and that somebody is sometimes you.

The occasion is ordinary and does not present as a decision. A toolchain patch lands, the source has not changed, the artifact gets rebuilt, and the bytes come out different. Re-uploading them under the version already printed on them is the obvious move, precisely because nothing anyone would call a change has happened. But the diff being empty is a fact about the source, and the thing being published is the artifact. "No source changed" is an argument about one and a claim about the other.

What it costs is a population split that nothing records. Where an update channel decides staleness by comparing versions rather than digests — which is most of them, because the version is cheap and the digest needs fetching — every consumer already holding the old bytes sees a version equal to the one on offer and never fetches. Every consumer arriving after the upload gets the new bytes. Two groups now run different code under one name, indefinitely, and no log line on either side says so. A defect that reproduces for one and not the other is unattributable by construction: the version is the only handle either party has, and it matches.

So the rule is **a rebuild is a new version whenever the artifact is addressed by version rather than by digest.** The bump costs a number. Not bumping it costs the ability to ever say which bytes a given consumer is running.

Two corollaries, both cheap:

- **Publish a versioned name even where the name consumers actually use is a moving one.** A moving name cannot be referred back to, so an install taken from it three weeks ago has no identity and nothing can reconstruct what it received. Publishing the immutable name alongside it costs one upload, changes nothing for consumers who want the convenience, and is the entire difference between a population you can count afterwards and one you cannot.
- **Where the channel can compare digests, let it.** Version comparison is a proxy for *are these the same bytes*, adopted because it is cheap. This section is what happens on the days the proxy and the question disagree — and the proxy wins silently, which is the property that makes it worth replacing rather than watching. `designing-ci-pipelines` and `auditing-dependencies` carry the consumer half of the same rule.

If it has already happened, the remedy is not to correct the record but to end the ambiguity: publish the rebuilt artifact under a new version, so that every consumer converges on bytes something can name. The existing split does not heal — it is simply capped, and the population that took the reused version stays unidentifiable for as long as it runs.

## Isolation layers, weakest to strongest

| Layer | Pins | Leaves open | Buy it when |
|---|---|---|---|
| Documented steps | nothing | everything | never; it is the baseline being replaced |
| Version manager | language runtime and its direct tools | system libraries, OS, other toolchains, env | laptops drift on runtime version |
| Container pinned by digest | OS, system packages, every installed tool | build-time network, clock, host env and mounts passed in | "missing library on the new machine" |
| Hermetic sandboxed build | every declared input; undeclared ones become errors | non-determinism inside the tools themselves | two builds of one commit differ, and that matters |

Choose by the failure you actually have, not by prestige. A container referenced by a mutable tag, or one that runs a package-manager update during the build, has bought you a slower build and no pinning at all — the pin is the digest and the frozen package set, never the tag.

## A check that launches the real artifact inherits the real environment

Isolation gets applied to the build and to the test suite, then quietly skipped for the one command that runs the actual artifact — because that command *is* the verification, and wrapping it feels like weakening the thing being proved. It is the opposite: an application resolves its data directory, profile, credentials and state through environment variables with a home-directory fallback, so a check that sets no environment opens the operator's real store on every run it has ever had. `bounding-autonomous-work` covers why an invocation is an execution and what to isolate it in; this is what to check **afterwards**, because the obvious check is wrong twice over.

Point every variable the artifact reads at a scratch directory, unset the fallbacks, and then assert two things rather than one — each catches exactly what the other cannot:

- **A negative check, aimed at the right files.** "The real store was not modified" gets written as an mtime comparison on the main database file, and a short run leaves that file byte-identical while putting half a megabyte into its write-ahead journal. Compare mtime *and* size, across the primary file *and* its sidecars. The obvious check passes while the application writes freely.
- **A positive check that the run got far enough to matter.** A launch that died in its first second, never reached its storage layer and exited satisfies every "nothing was touched" assertion by touching nothing. Assert that a store *was* created under the scratch directory. Without it, your strongest evidence is indistinguishable from the binary failing to start.

## Verify rather than assume

- Build twice and compare artifact hashes. Vary deliberately between the two: different directory, user, hostname, timezone, locale, and a clock at least a day apart. An unvaried rebuild proves almost nothing.
- When hashes differ, unpack both and diff recursively, including metadata — mtimes, modes, uid/gid, and entry order account for most differences before any content does.
- Normalize at packaging time: fixed timestamps, uid/gid zeroed, entries sorted, compression metadata stripped.
- Run the comparison as a scheduled job, not once. Reproducibility regresses the week after anyone stops checking.
- The harder test, worth doing annually: check out a tag at least a year old on a clean machine and build it. The two-build check catches ordering and timestamps; the old-tag check catches everything else.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Works on my machine" | an input was consumed but not declared — usually a system package or an env var |
| Build broke and nothing changed | something unpinned moved upstream; the commit was never the whole input |
| CI green, laptop red | CI starts clean; the laptop carries years of accumulated state |
| Containerized and still not reproducible | mutable base tag, or a package update running inside the build |
| Two builds of one commit differ | embedded timestamps, absolute paths, or unsorted iteration order |
| A year-old tag no longer builds | the build fetched from the network and the network moved on |
| Lockfile committed, versions still drift | the install command is allowed to update it and CI never runs frozen |
| Fails only overnight or on one continent | unpinned timezone or locale |
| Onboarding takes days | setup lives in prose that has never been executed |
| The verification step's own runs show up in the production audit trail | The check launched the real artifact with no environment set, so it resolved the operator's real profile through the home-directory fallback |

## Red flags

- "It only runs for a few seconds, it cannot have written anything." — a write-ahead journal fills in the first second, and the main file's mtime never moves.
- "Nothing was modified, so the isolation worked." — also true of a binary that failed to start.
- "Just install the latest version" as a setup instruction
- "We'll pin it later" — the pin costs minutes now and an archaeology session later
- Any build input named `latest`, `stable`, `main`, or an unversioned download URL
- A build step that refreshes a package index or upgrades system packages
- Claiming reproducibility that has never been measured by rebuilding and comparing
- The fix for a broken build was to delete a cache or regenerate the lockfile without reading the diff
