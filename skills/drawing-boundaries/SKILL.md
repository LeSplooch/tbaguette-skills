---
name: drawing-boundaries
description: Use when deciding what belongs together — module, package, crate, library, process, or service splits; extracting or merging components; wrapping a vendor SDK behind an interface you own; restructuring a folder layout. Also for circular dependencies, a shared or common module that only grows, every feature touching every module, components that must deploy in a fixed order, and whether something warrants its own process, repository, or service.
---

# Drawing boundaries

## Overview

A boundary is a promise that two sides can change independently. If they cannot, you paid indirection, serialization, versioning, and cross-boundary debugging and got nothing back.

## When to use

- Deciding module, package, crate, library, or service splits; extracting a component; merging two that never move apart.
- Wrapping a vendor SDK, or reviewing a wrapper that mirrors the vendor exactly.
- Symptoms: every feature touches five modules, circular imports, a `common` module everyone depends on and nobody owns, deploys that must be ordered, a change needing three coordinated repos.
- Not for: the shape of the interface once the boundary exists (designing-apis), or measuring what currently depends on what (mapping-dependencies).

## The criteria

- **Cohesion:** things that change together live together. **Coupling:** what a change on one side forces on the other. Both are measurable from history, not intuition — ask which files appear in the same commit, how often. Files that co-change in more than ~60–70% of their commits belong together regardless of which technical layer they were filed under. This is the most useful and least used input to a module structure.
- Boundaries follow **change frequency and ownership**. Two things with different release cadences, different reviewers, or different risk appetites want a boundary between them. Two that always ship together do not, however different their technology.
- **The classic mistake is splitting by tier or by noun** — `models/`, `controllers/`, `services/`, or one service per entity in an ER diagram. Layer splits guarantee every feature crosses every module, which is precisely what a boundary exists to prevent. Split by capability (checkout, pairing, billing, ingest) and let each own its own storage, types, and layers internally.
- **The test:** name the next three plausible changes and count the boundaries each crosses. A good structure keeps most changes inside one. A median of four or more means the boundaries are in the wrong places, and no amount of interface polish will fix it.
- A boundary needs an owner who can say no. A boundary nobody owns drifts into a shared utility bucket within two quarters.

## Dependency direction

- **Depend on an interface you own, not one a vendor owns.** The adapter is yours and converts vendor types, vendor errors, and vendor lifecycles into yours in exactly one file. Worth doing for anything with a plausible replacement: storage, queue, payment, auth, notification, clock, filesystem, randomness. Not worth doing for anything whose replacement is fantasy — the language's collections, the core runtime.
- **Size the port to your need, not the vendor's feature list.** A port with 40 methods mirroring the SDK is a rename, not an abstraction, and it has already leaked every concept you were hiding. Real ports are typically 3–8 operations, derived from the calls the core actually makes.
- Policy and domain code must not import transport, storage, or framework types. The mechanical test: could the core compile or link with the vendor dependency absent? If not, the boundary is decorative.
- **A cycle between modules means the boundary is in the wrong place**, not that you need another interface. Break it by inverting (the lower module declares the interface the upper implements) or by extracting the shared concept into a third module. Adding an import in the other direction is not a fix.
- `common`, `utils`, and `shared` are where boundaries go to die: everything depends on them, nobody owns them, so they can never change. Acceptable only under one rule — zero first-party dependencies and a stated, narrow purpose. Anything ambiguous goes in the capability that uses it, duplicated if necessary.

## What a boundary costs

| Kind | What it buys | What it costs |
|---|---|---|
| Function or type, same module | Naming, testability | Near zero |
| Module or package, same build | Compile-time enforcement, ownership, independent reasoning | One more name; the only cheap boundary that actually enforces anything — **start here always** |
| Versioned library or artifact | Independent release, reuse across builds | Version skew, a deprecation policy you must actually run, diamond dependencies |
| Separate process or IPC, one host | Fault isolation, independent restart, language choice | Serialization, lifecycle management, two logs to correlate |
| Network service | Independent deploy and scale, team autonomy | Partial failure as a permanent condition, latency, retries and idempotency, tracing, schema versioning, and an integration-test story that can cost more than the feature |

Move outward only when a concrete requirement demands it — a divergent scaling profile, a fault-isolation requirement, a team or compliance boundary, a different release cadence. "It felt cleaner" does not pay for a network hop. **A boundary drawn as a module can be promoted outward cheaply later; one drawn as a service is very hard to pull back in.** Draw in-process, promote under demand.

## When not to split

- **The two sides share a transactional invariant.** If correctness requires both to change atomically, they are one unit; splitting buys a saga and a new class of bugs in exchange for a diagram.
- You have fewer than about three real use cases. The abstraction is a guess, and a wrong boundary costs far more than duplication. Duplicate twice, extract on the third.
- **The distributed monolith** is the outcome of splitting without decoupling. Diagnose with four questions: can each side be deployed alone, in either order? Does a schema change require a coordinated release? Does one going down make the other useless rather than degraded? Do they share a database? Two yeses and you have one system with network calls inside it — every cost of distribution, none of the independence.
- A shared database between two components is not a boundary; it is a boundary with a hole in it. The schema is the real interface and nobody versioned it or wrote it down.
- **Chatty crossings mean the boundary cuts a cohesive operation.** A loop containing a cross-boundary call, or N calls per user action, is a signal to move the loop across the boundary (one coarse call) or move the boundary.
- **Merging is a legitimate refactor and badly under-used.** Two modules that always change together and are used by nobody else should be merged, then re-split later along the seam the changes actually revealed.

Before merging two things that look alike, establish that the resemblance is a shared reason rather than a coincidence — `judging-duplication`.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Every feature touches every module | Split by technical layer instead of by capability |
| Circular imports between modules | Boundary in the wrong place, or a missing third concept |
| `shared` / `common` grows without limit | A boundary with no owner, used as a dumping ground for anything ambiguous |
| Components must be deployed in a fixed order | Distributed monolith; the interface between them is not versioned |
| Vendor types appear throughout the core | Port defined by the vendor's API rather than by your need |
| A "port" with 40 methods | Mirroring the SDK, so nothing is actually abstracted |
| A simple change needs three repos and three reviews | Ownership boundaries and change boundaries do not line up |
| Two services share a table | The schema is the real interface, and it is unversioned |
| N+1 calls across a service boundary | Boundary cuts through a cohesive operation |
| An interface with exactly one implementation, forever | Extracted before a second use case existed |

## Red flags

- "Let's make it a microservice so it's decoupled."
- "Put it in `common` for now."
- "It's cleaner as its own package" — with no second consumer.
- "We just need to coordinate the deploys."
- "We'll abstract the database in case we switch" — said while copying one vendor's exact method names.
- Any architecture diagram whose top-level boxes are all technical layers.
- Extracting an interface to make something testable when the real problem is that it does too much.
