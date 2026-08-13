---
name: finding-the-seam
description: Use when deciding where in a codebase to make a change, when a fix could plausibly go in several places, when a diff is spreading across many files, when behavior must be altered without editing the code that owns it, when something cannot be tested without booting the whole system, or when an extension point, adapter, wrapper, or injection point is needed to land a change safely.
---

# Finding the seam

## Overview

A seam is a place where behavior can be changed without editing the thing whose behavior changes. The question is never "how do I make this work" but "what is the smallest set of observable things I can put at risk to make this work" — and the smallest diff and the smallest blast radius are usually different places.

## When to use

- Two or more plausible sites for the same change, and no obvious reason to prefer one.
- A change is metastasizing across files and something is clearly wrong with the approach.
- Behavior must vary by environment, tenant, platform, or caller.
- A test would require standing up the whole system.
- Not for: enumerating who is affected once the site is chosen (mapping-dependencies), replacing a system over many releases (incremental-migration), or isolating a dependency purely to test it (testing-the-untestable).

## Seam types

| Seam | Mechanism | Available in |
|---|---|---|
| Parameter | Pass the varying thing in | Everywhere. Try this first, every time |
| Interface / trait / protocol | Swap the implementation at a declared boundary | Any language with a nominal or structural contract |
| Higher-order function | Pass behavior as a value | Anything with first-class functions |
| Data / table | Move the variation into a lookup the code already reads | Anywhere the variation is enumerable |
| Configuration | Change a value; no code diff | Anywhere config is read at startup |
| Container / registry | Change what the wiring hands out | DI frameworks, plugin registries, service locators |
| Build or link | Swap a module, object file, feature flag, or conditional compilation unit | Compiled and feature-gated ecosystems |
| Subclass / override | Replace a method on a base | Class-based OO — the weakest seam, since it couples you to the parent's internals and to its call order |
| Process or wire boundary | Change the other side | Services, subprocesses, sidecars, plugins |
| Preprocessor, macro, monkeypatch, bytecode rewrite | Rewrite before or at load | Last resort; invisible to readers and to every tool |

## Ranking candidate sites

Take the first that works. The ordering is by blast radius, not by diff size.

1. **Configuration or data.** Zero code diff, no rebuild, and in many setups revertible without a deploy.
2. **A new implementation behind an existing interface.** Purely additive; nothing that already runs changes. Risk is confined to the one wiring line that selects it.
3. **A change at a boundary the value already crosses** — an adapter, mapper, handler, serializer, or edge. The contract there is already explicit and already tested, so the change is visible to review and to the type system.
4. **A change in a leaf** — a module with fan-in of 0–2. Blast radius equals its dependents, which you can name.
5. **A change to a shared utility or a widely-used type.** Last, always. Its dependents are everything, its tests belong to nobody, and reviewers approve it precisely because it looks small.

The inversion this ordering corrects: the tempting change is nearly always #5, because it is the fewest characters. A three-line edit to a shared string helper is a larger change than a 200-line new adapter. Character count is not risk.

## The narrowest spanning interface

List every place the new behavior must be visible. Find the lowest node in the dependency graph through which every path to those places passes — the dominator. That node is the seam.

- If the dominator is one function or one boundary, the change is local and you are done deliberating.
- If the dominator is the entry point, the behavior is cross-cutting. It belongs in the composition root — the one place objects are wired together — not sprinkled at each site. Cross-cutting behavior implemented at N sites is N places to forget it.
- If there is no single dominator, there are genuinely two changes. Split them and stop trying to find one clever location.

**The seam test.** A candidate is a real seam only if all four hold:

- Behavior changes there without editing the thing whose behavior changes.
- The new behavior is testable without booting the whole system.
- Existing tests for the surrounding code pass unchanged. If they must change, you are altering a contract, not using a seam.
- Undoing it is one revert.

## When no seam exists

Create one first, in its own commit, with no behavior change:

1. **Extract the seam.** Parameterize the varying thing, introduce the interface, hoist a constructed dependency up to the caller, or move a hard-coded value into data. Tests pass, behavior byte-identical.
2. **Commit.** On its own.
3. **Make the behavior change against the new seam.** Small, and reviewable as a behavior change rather than as a diff full of moved code.

Verification for step 1: the diff contains no new conditionals and no changed literals. A "pure refactor" that adds an `if` is not one. If the extraction exceeds a few hundred lines or cannot be argued as a no-op, the seam chosen is too wide — pick a narrower one.

**The cost check before creating any seam.** A seam is an abstraction, and an abstraction with exactly one implementation is a cost with no payment: an extra indirection for every future reader, and a contract that constrains the one implementation for no benefit. Create one when a second implementation exists now, or when the alternative is untestable code. "We might need it later" is how a codebase acquires nine interfaces with one implementer each, and each of those makes the next trace longer.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Diff touches 14 files for one behavior change | Changed a leaf 14 times instead of the boundary once |
| One-line change to a shared helper broke unrelated features | Optimized for smallest diff instead of smallest blast radius |
| Test requires the whole system running | No seam; the dependency is constructed inside rather than passed in |
| Refactor and fix in one commit | Neither can be reviewed, and the fix cannot be reverted without the refactor |
| Added a boolean parameter so a function does two things | Not a seam — two functions sharing a name and a body |
| Nine interfaces, one implementation each | Seams created speculatively; every one is a permanent reading cost |
| Behavior implemented at every call site "for now" | The dominator was the composition root and it was skipped |

## Red flags

- "I'll just add a flag parameter."
- "I'll patch it at each call site."
- "It's only two lines in the shared utility."
- "I'll do the refactor and the fix in one commit, it's cleaner."
- "I'll subclass it and override the method."
- "The tests need updating for this refactor" — then it is not a refactor.
