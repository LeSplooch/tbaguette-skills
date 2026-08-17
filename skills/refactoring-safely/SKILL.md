---
name: refactoring-safely
description: Use when restructuring existing code without changing what it does, when a change touches many call sites at once, when tests break in the middle of a restructure, when code needs cleaning up before a feature can be added, when working in untested or legacy code, when two near-identical functions look like an obvious deduplication, or when tempted to rewrite rather than transform. Covers behavior preservation, extracting and inlining, large-scale renames, load-bearing duplication, and refactor-versus-rewrite decisions.
---

# Refactoring safely

## Overview

A refactor changes structure and preserves observable behavior. The instant one diff does both, it is neither reviewable nor revertible, and nobody — including you — can say which half broke.

## When to use

- Restructuring working code so a pending change becomes easy
- A rename, move, or signature change that ripples through many call sites
- Cleaning up code you did not write and cannot fully test yet
- Weighing an incremental restructure against starting over
- Not for: building the safety net in untested code — see `characterization-testing`. Not for creating a test seam where none exists — see `finding-the-seam`. Not for restructuring stretched across releases — see `incremental-migration`.

## What "behavior" actually includes

The definition is broader than the return value, and every item below has broken production during a "pure refactor":

- Return values, raised errors, **and error types** — a caller matching on a specific error class breaks when extraction wraps it
- Side effects, and the **number of times** they occur; inlining a memoized call multiplies it
- Iteration and result ordering wherever anything downstream depends on it, including ordering nobody documented
- **Log messages and metric names that alerts match on** — these are a production API with an on-call consumer, no matter what the language says about visibility
- Serialized shapes, wire formats, and persisted class names
- Public symbol names reachable by reflection, dynamic dispatch, plugin registries, or another language's bindings
- Latency and allocation where a contract or a budget depends on it

A restructure that changes any of these is a behavior change wearing a refactor's name. Ship it as one, with a message that says so.

## One transformation at a time

Apply a single named move, run the tests, commit. The interval between two green states is the size of the search space when it goes red — keep it under a few minutes of work. "Tests green at the end" is not the same discipline and does not produce the same outcome. Each commit this produces is the atomic kind `atomic-commits` covers — a refactor commit that also carries a behavior change isn't atomic no matter how small it is.

| Move | Preserves when done right | How it breaks when done in bulk |
|---|---|---|
| Rename | Everything, given every reference is updated | Reflective, string-keyed, cross-language, and config references are invisible to the tool |
| Extract function | Behavior, given captured state is passed faithfully | Turning captured state into a parameter changes evaluation order and time of read |
| Inline | Behavior, given the callee is pure | Inlining a side effect changes how often it happens |
| Move to another module or type | Behavior, given initialization order holds | Import graph, visibility, and static-initialization order all shift |
| Introduce parameter | Behavior, given the default matches current use | A default chosen for the new caller silently changes every old one |
| Replace conditional with dispatch | Behavior, given the fallback branch is preserved | The unhandled case that used to fall through now throws |
| Merge two near-duplicates | Behavior, given nothing outside the process can tell them apart | The difference was the point, and its observer is a signature verifier, a stored format, or a wire peer |

Chained small moves beat one large one because each is mechanically checkable, individually revertible, and individually explainable in a subject line. The composite is none of the three. Where an automated refactoring tool exists for a move, use it — it updates references you would not have found. Where none exists, restrict yourself to transformations precise enough that you could describe the rule and have someone else verify it.

## Duplication that is load-bearing

Before unifying a near-duplicate pair, find out who observes the difference. When the observer sits outside this process — a peer, a wire format, a stored file — the duplication is a contract, and the merge fails there rather than here, long after the diff.

`judging-duplication` covers making that call.

## Before touching untested code

A refactor without a net is an unverified rewrite of a small region. Establish the net first, and keep the net's construction in its own commit.

- **Characterization tests** capture what the code does today, bugs included, without judging correctness. They are change detectors, not correctness tests; delete or rewrite them once real tests exist.
- Where nothing is testable, make the **smallest possible seam** first — parameterize one dependency, extract one interface. That seam change is itself unnetted, so make it tiny, make it with the tool, and review it separately.
- Cover every branch you intend to move. Where you cannot, restrict the work to moves the tool performs mechanically and accept that hand edits in that region are behavior changes by default.
- When tests are genuinely impossible: compare old and new against recorded inputs, checksum a serialized output over a corpus, or lean on compiler-enforced exhaustiveness. All three beat nothing; none beats a test.

**Refactor toward the change you are about to make**, not toward abstract cleanliness. Make the change easy, then make the easy change. Restructuring with no pending change is speculation that pays interest and never returns principal, and it burns the review budget you will want next week.

## Refactor or rewrite

Rewrite only when **all four** hold. Any one failing means incremental restructuring, however unappealing.

1. **Behavior is knowable independently of the code** — a specification, a conformance suite, a protocol document, or a running system you can compare against. Where the code is the only specification, a rewrite guarantees silent behavior loss.
2. **The old system can keep running** while the new one is built and compared. Otherwise the rewrite is also a big-bang cutover, and it inherits every failure mode of one.
3. **The problem is architectural, not local** — the constraint is baked into a data model, a concurrency model, or a dependency that no sequence of local moves can reach.
4. **The whole cost is affordable**, including the long middle where both exist and every feature is built twice.

Rewrites lose because ugly code is dense with undocumented bug fixes: each strange branch is somebody's production incident, each odd constant is a vendor's undocumented behavior. The new version is elegant precisely because it does not yet know about them, and it will rediscover them one customer report at a time. Before rewriting, mine the old code's history for fix-shaped commits touching that region — each one is a behavior requirement absent from your new specification, and that list is the honest scope of the work.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Small refactor" that touches 60 files and adds a feature | Structure and behavior in one diff; neither is now reviewable |
| Tests edited in the same commit as the refactor | The net was adjusted to fit the change, which means there was no net |
| Refactor breaks something no test covered | Behavior was observable through a channel outside the suite — a log, a metric, an ordering |
| Post-refactor latency regression | Extraction added a call and an allocation on a path that was hot for a reason |
| Rename missed some references | Reflection, string keys, configuration, or another language's bindings |
| Rewrite 90% done and permanently stuck | The last 10% is the undocumented behavior, and it was never in scope |
| Refactor abandoned midway; two shapes now coexist | Transformation was too large to finish in one sitting and too large to revert |
| Reviewer cannot say whether behavior changed | The diff does not distinguish moved lines from edited ones |
| Deduplicated two similar functions; a counterpart started rejecting the output | The difference was a contract with an outside observer, not copy-paste |

## Red flags

- "I will just fix this bug while I am in here"
- "The tests are slow, I will run them at the end"
- "It would be faster to start over" said without naming where the behavior specification lives
- "I will add the tests after the refactor"
- Editing a test's expected value so a refactor passes
- Being unable to name, in one phrase, which transformation you are currently applying
- "These two are identical apart from one character" — said without naming what outside the process compares them
