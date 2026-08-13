---
name: reproducible-environments
description: Use when a build or test passes on one machine and fails on another, when an old tag or release can no longer be rebuilt, when onboarding needs undocumented setup steps, when a build breaks though no commit changed, when two builds of the same commit produce different artifacts, or when choosing between a version manager, a container, and a hermetic build. Covers pinning, lockfiles, toolchain declaration, isolation, and determinism.
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

## Isolation layers, weakest to strongest

| Layer | Pins | Leaves open | Buy it when |
|---|---|---|---|
| Documented steps | nothing | everything | never; it is the baseline being replaced |
| Version manager | language runtime and its direct tools | system libraries, OS, other toolchains, env | laptops drift on runtime version |
| Container pinned by digest | OS, system packages, every installed tool | build-time network, clock, host env and mounts passed in | "missing library on the new machine" |
| Hermetic sandboxed build | every declared input; undeclared ones become errors | non-determinism inside the tools themselves | two builds of one commit differ, and that matters |

Choose by the failure you actually have, not by prestige. A container referenced by a mutable tag, or one that runs a package-manager update during the build, has bought you a slower build and no pinning at all — the pin is the digest and the frozen package set, never the tag.

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

## Red flags

- "Just install the latest version" as a setup instruction
- "We'll pin it later" — the pin costs minutes now and an archaeology session later
- Any build input named `latest`, `stable`, `main`, or an unversioned download URL
- A build step that refreshes a package index or upgrades system packages
- Claiming reproducibility that has never been measured by rebuilding and comparing
- The fix for a broken build was to delete a cache or regenerate the lockfile without reading the diff
