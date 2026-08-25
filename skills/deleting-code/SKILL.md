---
name: deleting-code
description: Use when removing code, a feature, an endpoint, a config key, a column, or a dependency that appears unused, when deciding whether something is genuinely dead, when a deprecation needs a removal path and a deadline, when code has been commented out or kept "just in case", or when the callers of something cannot all be seen from inside the repository. Covers proving deadness, deprecation sequences, and removing tests and config alongside.
---

# Deleting code

## Overview

Deletion is the highest-leverage change available and the one people make on the thinnest evidence. The question is never "does anything call this" — it is "what would constitute proof that nothing does", and a clean search is not proof.

## When to use

- Removing a feature, endpoint, flag, module, table column, or dependency
- Something looks unused and you are about to trust a search that found nothing
- A deprecation has been in place for a while with no removal date attached
- Code was commented out rather than removed, or kept because it "might be useful"
- Not for: removing an old implementation as the final phase of a replacement — see `incremental-migration`. Not for judging whether historical code is load-bearing — see `code-archaeology`.

## Proving deadness

Each layer below hides from the layer above it. A search for the symbol name finds the first two rows and nothing else.

| Reference kind | How it hides from search | What actually finds it |
|---|---|---|
| Static call | Does not | Compiler, linker, or an index-aware search across the whole tree |
| Interface member, override | Name appears only on the base type | Delete the member and see what fails to compile |
| Reflection, annotations, DI containers | Name exists only as a string | Grep the string form in every casing convention the stack uses |
| Build-time codegen, macros | Caller is not in the source tree | Search generated output, not sources |
| Configuration and flags | Reachable only where some flag is on | Read flag state in every environment, not the default |
| Data-driven dispatch | Route table, plugin manifest, database row, message type | Query the data store, not the code |
| Persisted state | A type name is written into stored records | Cannot delete until those records are drained or migrated |
| External consumers | Not in your repository at all | Telemetry — nothing else works |
| Rare scheduled paths | Runs quarterly | Telemetry over a window longer than the period |

Then instrument. This is the one case where `instrumenting-for-observability`'s rule inverts: normally you emit what changed, here you need to emit that nothing did, and a counter that only increments is the only shape that can prove absence. Add a counter or a sampled log at the entry of the suspected-dead path, ship it, and wait longer than the longest business cycle that could reach it — 30 days as an absolute floor, and one full billing or fiscal cycle for anything customer-facing. Delete when the counter is zero **and you can explain why it is zero**. Zero from a build that never reached two regions is not evidence; confirm the instrumented version is actually running everywhere before you start the clock.

Search the non-code artifacts too, because they fail silently rather than loudly: dashboards, alert rules, saved queries, runbooks, support macros, CI configuration, infrastructure definitions, and translation catalogs. Deleting a log line or a metric name blanks a dashboard nobody notices until an incident.

## Deprecation is a schedule, not a label

Fix the whole sequence and its dates at step 1. A deprecation notice without a removal date is a permanent annotation, and the codebase already has several.

1. **Announce** with a removal date and a named replacement. No date, no deprecation.
2. **Warn at the call site** — compile-time where the language allows, otherwise once per process and sampled, never on a hot path.
3. **Fail outside production** first, then error by default with a documented escape hatch.
4. **Remove the escape hatch.**
5. **Delete.**

For consumers outside your control, the notice period is whatever your versioning contract promises, and never less than one full major release. Escalation dates are set once and held; renegotiating them at each step is how a deprecation reaches its fourth year.

## Delete, do not comment out

Commented-out code is invisible to search and replace, never compiled, never tested, silently rotted against the interfaces around it, and read by the next person as "possibly still needed". Version control is the archive — but only if retrieval works, which means the deleting commit's message must name the feature in the words a future searcher would use. That, not the code block, is what makes the deletion recoverable.

Delete the whole vertical, not the entry point: tests, fixtures, config keys, feature flags, documentation, dashboards, alerts, translations, and any dependency now unreferenced. Orphaned tests are the most common leftover — they keep passing, consume CI time forever, and convince readers a feature still exists.

Tests go with the feature, with one distinction: a test that exercises *only* the deleted feature is deleted, while a test that exercises a **general invariant through** the deleted feature is rewritten against a surviving path. Deleting that second kind quietly drops coverage of a rule that still holds, and it is the coverage nobody notices losing.

Once the deletion is made, the compiler is the completeness check — but it only testifies about files it compiled. In most toolchains the default build and the test build compile different sets, and benchmarks, examples, and feature- or platform-gated code are further sets again. A symbol still referenced from one of those compiles clean under the default target and surfaces later as what reads like an unrelated breakage. Before calling a deletion finished, run the compiler over every target that can reference the symbol, not just the one the default build command covers; in an uncompiled stack, the pass that resolves names — typically collecting and importing the full test suite — plays the same role, with the same scoping caveat. Dead-code warnings are computed over the same partial file set, so they are not a census of callers either: a build that skips tests can flag a still-live test helper as dead, and says nothing at all about an exported symbol no matter who calls it.

Keeping code "just in case" is not free. It is read by every future reader, matched by every search, compiled into every build, updated by every sweeping refactor, and scanned by every audit — and unreachable code is where unpatched vulnerabilities sit longest, because nobody prioritizes fixing something nobody runs.

## When consumers are outside your view

If you cannot enumerate your consumers, you cannot prove death, and the choice is between four honest options: version the interface and remove only in a new major; keep a logging shim that redirects and reports who arrives; require registration and delete only what is unregistered; or accept the break with the rollback prepared.

Whichever you pick, delete at a time you can watch it — never bundled with other changes, so the revert is clean, and never immediately before a period when nobody is looking. A deletion is a deploy whose blast radius you could not estimate, which is exactly the kind that needs a fast, isolated undo.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "No references found", then a production break | Searched code only — not config, data, telemetry, or external callers |
| Feature deleted, its tests still green and still running | Tests were never in the deletion's scope |
| Deletion pronounced complete on a clean default build, then the test suite breaks somewhere that looks unrelated | The default target never compiled the tests, benchmarks, examples, or gated code that still referenced the symbol |
| Code commented out "temporarily" three years ago | Nobody was willing to own the decision, so it was deferred into the file |
| Deprecation warning present for years | No removal date was ever attached to it |
| A dashboard went blank after a cleanup | A log line or metric name was treated as internal |
| Delete reverted, and the feature comes back broken | Deletion was bundled with unrelated changes, so the revert was not clean |
| Dead code keeps being updated by refactors | It looks alive because it is maintained, and it is maintained because it looks alive |
| Old implementation kept "for reference" beside the new one | Reference material stored in the build instead of in history |

## Red flags

- "Let's keep it, it might be useful later"
- "I will comment it out just in case"
- "Nobody uses this" with no counter behind it
- "We can always get it back from history" used as the reason not to write a findable commit message
- Deleting bundled with a release, or right before nobody is watching
- Deleting something because a search was clean, when the stack uses reflection, plugins, or configuration-driven dispatch
- "It compiles" — from a build that never compiled the tests
