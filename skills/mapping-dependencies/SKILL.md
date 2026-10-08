---
name: mapping-dependencies
description: Use when assessing what a change will break, estimating blast radius, chasing circular imports or build cycles, finding modules with too many dependents, checking whether layering or architecture rules still hold, planning to split, extract, or delete a module, or after a small local change caused a failure in something that never referenced it. Also use before adding a member to an enum, sealed type, or tagged union that other code switches on, lists, or parses — especially from another language — or when a new member compiled clean and still misbehaved. Covers the compile, runtime, and data graphs, blast radius by kind of change, and sorting a type's enumeration sites into the ones the compiler checks and the ones it cannot see.
---

# Mapping dependencies

## Overview

The dependency graph in the import statements is one of three graphs, and it is the one least likely to hurt you. Two components with no shared code and one shared table are more tightly coupled than two modules that import each other, because nothing will tell them when they disagree.

## When to use

- Before a signature, schema, or shared-behavior change, to enumerate who notices.
- Before adding a member to an enum, sealed type, or tagged union, to find every place that enumerates it — including the ones no compiler will flag.
- A cycle blocks a build, a test, an extraction, or a deletion.
- Deciding module boundaries, or whether a proposed split is even possible.
- An incident traced to a component that had no visible link to the change.
- Not for: choosing where to put a change once the graph is known (finding-the-seam), or one value's path (tracing-data-flow). Once a data edge shows a fact with more than one writer that has to stay that way, keeping the copies from drifting is `keeping-copies-in-sync`.

## Three graphs, not one

| Graph | An edge means | Lives in | Breaks when |
|---|---|---|---|
| Compile / link | A needs B's symbols to build | import, include, use statements; build files | a signature, type, constant, or visibility changes |
| Runtime | A's execution reaches B's | DI config, route tables, plugin manifests, dynamic loads, callbacks, service calls, queue topics, cron entries | behavior, ordering, or timing changes |
| Data | A and B agree on a shape | schemas, tables, wire formats, file layouts, cache keys, env vars, log and metric fields | a field changes, with zero shared code |

Only the first is mechanically extractable. The data graph is the one that causes incidents: both sides compile, deploy, and pass their own tests while disagreeing about a field. Every "how did that break, they don't even import each other" is a data edge.

## Building the real graph

1. **Extract compile edges mechanically.** Every ecosystem has a tool; failing that, search the import form and reduce to module-to-module pairs. Never hand-draw it — a hand-drawn graph is the graph you already believed, which is the thing under test.
2. **Collapse to the level of the question.** File level for a refactor, package level for architecture, deployable-artifact level for release planning. A 4,000-node file graph is not a map.
3. **Add runtime edges by hand, from the registries.** Read the DI container config, the route table, the plugin manifest, the topic and event names, the scheduler definitions. There are usually 5–30 of these and they are exactly the edges missing from every auto-generated diagram.
4. **Add data edges from shape ownership.** For each table, topic, cache namespace, file format, and env var: list every writer and every reader. Anything with more than one writer is a coupling point and a future incident.

## Numbers that mean something

| Signal | Threshold | Reading |
|---|---|---|
| Fan-in (dependents) | more than 20% of modules | Its interface is frozen in practice; change cost is superlinear |
| Fan-out (direct dependencies) | more than 15 | An orchestrator — acceptable at an entry point, a defect in a leaf |
| High fan-in and high fan-out together | any | A god module; it is the architecture, whatever the docs claim |
| Instability = out / (in + out) | leaves near 1.0, cores near 0.0 | A near-0.0 module that changes weekly is the highest-risk object in the repo |
| Depth from entry point | more than 6 layers | Each layer is a contract where a change either stops or does not; most do not |
| Cycle size | anything above 1 | Those files are one module wearing several filenames |

**The ranking that predicts pain:** sort modules by dependents × commits in the last six months. The top three produce most incidents. Two commands, and it outperforms any architecture diagram in the repo.

## Cycles and layering

A cycle means those files cannot be built, tested, understood, deleted, or extracted separately, regardless of the folder tree. Break it by extracting what both sides depend on — usually a type or interface, rarely behavior — or by inverting one edge with a callback or interface owned by the lower layer. Moving a function between the two files relabels the cycle; it does not remove it. Verify by re-extracting the graph, not by reading.

Layering rules: write the intended layers once as a list of permitted edges, then check it mechanically in CI. An architecture rule with no enforcing check is a preference, and it is already violated — check before assuming otherwise; it holds nearly every time. A rule that must be violated in three known places is a rule with three declared exceptions, which is fine and is still enforceable; an unwritten rule is not.

## Blast radius before a change

Three different sets, computed three different ways:

- **Signature or type change** — the transitive compile closure of the changed symbol. Tool-computable, bounded, and reliably the smallest of the three. The compiler finds what you missed.
- **Behavior change** — direct runtime callers plus everything depending on ordering, timing, or side effects. Not tool-computable; enumerate by hand from the runtime graph. Nothing will warn you.
- **Shape change** — a field, column, message, or key: every reader and every writer of that shape, including versions still deployed, messages already in flight, cached copies, and rows already stored. Almost always the largest set, and the one routinely forgotten.

Blast radius is not the number of files a diff touches. It is the number of things that can be observed behaving differently — those numbers are frequently inverted, which is why a one-line change causes an outage and a 600-line one does not.

**Adding a member to an enum, sealed type, or tagged union is where "the compiler finds what you missed" stops being true.** Every place that enumerates the type is one of two kinds. An *exhaustive* site — a match or switch the checker requires to cover every member, with no default arm to absorb a new one — becomes a compile error the moment the member exists (or a warning, which counts only if the build fails on it). A *silent* site compiles unchanged and handles the new member by accident: an `else`, `default`, or `_` arm; a hand-written list of members, such as a picker's options, an allowlist, or a table of which kinds need a permission check; a parser in another language or process that maps a tag it does not recognize to some default kind. "It compiles" proves the first kind and says nothing about the second, and the second is where the damage lands — the default arm that writes the wrong shape back onto the new member, the permission table that never lists it and so never checks it.

So sort the sites before writing the member. Search for the names of the existing members — the closest sibling of the new one first, since a site that handles it is the likeliest to need the new one too — rather than for the type's name, which misses string tags, other languages, and lists that never mention the type. Mark each hit exhaustive or silent. The silent sites across a language or process boundary carry the most risk, because no compiler or find-usages connects them to the change at all. Where you can, convert a silent site to exhaustive — remove the default arm, replace a hand-written list with a property every member must declare — rather than adding one more case to it, so the next member is caught by the compiler instead of by this search. A reader of stored or transmitted data still needs its unknown branch for versions you do not control (`schema-evolution`); there, make that branch keep the raw value or refuse it, never turn it into one of the real members.

## Why "everything imports it" is a constraint

A module with 200 dependents no longer has an interface you can change; you can only add to it, and each addition makes the next change harder. Treat any widely-imported utility as a frozen public API that no single owner reviews. Two consequences: additions to it deserve the scrutiny of a public API change, and the correct fix for "this shared helper needs a new behavior" is usually a new module the callers that want it depend on, not a wider helper.

## Common mistakes

| Symptom | Real cause |
|---|---|
| One-line change, unrelated production failure | A data-graph edge; no compile edge existed to warn anyone |
| The architecture diagram matches nothing | Drawn once, never enforced; no mechanical layering check |
| Cycle "fixed" by moving code between the files | Edge relabeled, not removed |
| Everything depends on a `common` or `shared` package | A package named for indecision accumulated the whole graph |
| Deleted an unreferenced module; it broke later | Runtime-only edge via reflection, config, or a plugin registry |
| Refactor stalled on a module that will not extract | A cycle, discovered at extraction time rather than before planning |
| "Only a new field, fully backward compatible" | Old readers, queued messages, and stored rows are also readers |
| A new enum member compiled clean and misbehaved | It fell into a default arm, was missing from a hand-written list, or was mapped to another kind by a parser in another language; the compiler only saw the exhaustive matches |

## Red flags

- "Just put it in the shared utilities for now."
- "Nothing else uses this" — asserted without extracting the graph, including runtime and data edges.
- "It's only an additive change."
- "The diagram in the wiki shows…"
- "The compiler will catch it" — true only for the compile graph, which is the graph least likely to hurt you, and within it only for exhaustive matches.
