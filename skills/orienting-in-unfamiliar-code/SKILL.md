---
name: orienting-in-unfamiliar-code
description: Use when opening a codebase for the first time, inheriting an unfamiliar or legacy repo, onboarding onto a new project, or being asked to change code that nobody present wrote. Covers where to start reading, locating entry points and real module boundaries, telling live code from dead, finding where the work actually happens, and reconciling documented architecture with the one the imports reveal. Also use when a text search for a common identifier returns too much to read, when the question is who calls this or whether anything still uses it, or when deciding how far to trust a resolver's zero-references answer.
---

# Orienting in unfamiliar code

## Overview

Orientation is a bounded reconnaissance with a deliverable: the name of the file you are going to change and a list of the things that will notice. It is not comprehension of the system, which no one has and which you will never reach by reading.

## When to use

- First contact with a repo you did not write, or returning after a year.
- A task lands and you cannot name the file it touches.
- Documentation and code disagree and you need to know which is running.
- You are estimating a change in a system you have not measured.
- Not for: a specific value's path through the system (tracing-data-flow), the reason a line exists (code-archaeology), or the coupling graph (mapping-dependencies).
- Run alongside `recovering-agent-context`: this skill reads what the code itself can show you; that one reads what prior AI sessions and instruction files learned here that the code can't — different sources, same first-contact moment.

## The read order

Work down this table. Each row answers a question the next row assumes.

| # | Read | Answers | Done when |
|---|---|---|---|
| 1 | Build and manifest files, lockfile, CI config, container/deploy definitions | What this repo produces, what it depends on, how it is built, tested, and run | You can name every artifact the repo emits |
| 2 | Entry points: `main`, server bootstrap, CLI dispatch, exported API surface, job and migration entries | Where control begins, and how many beginnings there are | You have counted them |
| 3 | Test names only — not bodies | The intended contract, stated as a requirements list someone maintained | You can list the features from test names alone |
| 4 | Directory shape, one level deep | Which names are real boundaries and which are dumping grounds | You can guess where a new feature would go |
| 5 | Churn: `git log --format= --name-only --since=6.months \| sort \| uniq -c \| sort -rn \| head -40` | Where work actually happens, which is rarely where the docs point | You have the top ten files |

Check the root for a file the project wrote *for people in your position* —
whatever this ecosystem currently calls the agent instruction file. It is the
one document whose entire purpose is answering "what should I know before
touching this", so it is worth a read before anything you have to infer. Treat
it as intent rather than fact, the same as any prose in the repo: it says what
someone wanted to be true, and the imports say what is.

CI config is the most honest document in any repo: it is the only description of the project that fails when it goes stale. Read it before the README. The lockfile gives the real dependency set; the manifest gives the intended one.

Count entry points before reading any. One entry point is a program; twelve is a platform, and "understanding it" means something different — you orient per entry point, not per repo.

## Breadth before depth

Never follow a call more than one level deep on the first pass. When a call is interesting, write the question down and keep moving. Depth-first reading is how a 30-minute orientation becomes three hours with no file identified, and it is the single most common way this fails.

Three passes, each cheap:
1. **Surface** — the table above. Produces a map with holes.
2. **Spine** — one request, job, or command followed end to end, naming each layer it crosses without reading inside them. Produces the layer list, which is what you actually need.
3. **Target** — full depth on the two or three files the task touches, plus their tests.

Stop at pass 2 unless the task demands more. Most tasks do not.

## A text search and an index answer different questions

Searching text answers *where does this string appear*. A resolver — a language
server, an IDE index, a tags file, a compiler-backed query — answers *where is
this defined* and *who uses this*. Those look like the same question right up to
the point where orientation is hardest: a short or common identifier, a method
name shared across unrelated types, a symbol re-exported under a different name.

Text search degrades badly there, and it degrades in a way that feels like
progress. A common name returns hundreds of lines, the reflex is to narrow the
pattern, and narrowing discards true positives along with the noise — leaving a
shorter list that reads as a result and is missing the call that mattered. Where
the ecosystem has a resolver, stand it up as soon as pass 1 has told you what
language this is — before the passes that do the reading, rather than after all
of them — because it changes which questions are cheap: *who calls this* stops being
an afternoon, and *is anything still using this* becomes answerable instead of
guessed.

It does not license depth-first reading. Breadth before depth is about attention,
not about tooling, and an index that makes any single call one keystroke away
makes it easier, not harder, to disappear down one. What it changes is the other
half of that rule — the questions you wrote down and kept moving past are now
answerable in one step when you come back to them.

And keep it honest in the other direction, because this is where an index quietly
misleads. **It only knows what it can resolve.** Reflection, names built from
strings, config-driven wiring, dependency injection by convention, dynamic
dispatch across a boundary, and anything crossing into another language are
invisible to it — and those are precisely the seams that surprise you later. So a
**zero-references result is a hypothesis, not a finding**: it means the resolver
saw no reference, which is the claim you wanted only if the resolver could see
everything. Confirm it with a text search over the whole repository, including
configuration, templates, and data files, before deleting anything or concluding a
path is dead. The index is the fast first pass; the search is the check on what
the index cannot see.

## The build graph is the structure

The folder tree is a filing decision; the build graph is a constraint. When they disagree, the build graph is the architecture:

- Compilation or packaging units that cannot depend on each other are a real boundary; two folders in the same unit are not, no matter how they are named.
- A folder named `common`, `core`, `shared`, `utils`, `lib`, or `misc` is named for the moment nobody could decide. Its contents are unrelated. Treat it as a namespace, never as a module.
- Modules that ship separately, version separately, or deploy separately are separate systems and should be oriented separately.

## Claimed versus actual

Docs, ADRs, and diagrams describe the architecture at the moment someone last cared. Imports describe today. Check three things, in this order:

- **A layer everything bypasses.** The docs describe a repository/service layer; half the handlers query storage directly. The layer is aspirational.
- **A "deprecated" module with recent commits.** It is not deprecated; something depends on it that nobody wants to name.
- **A module the diagram omits entirely.** Usually the one with the highest churn.

Each divergence you find is worth more than the rest of the orientation. Write them down; they are the constraints your change will hit.

## Time-boxing

Spend 20–40 minutes, or 10% of the estimated task, whichever is smaller. Orientation ends when you can state: the file to change, the interface it sits behind, the tests that cover it, and one risk. Not when you feel comfortable — comfort arrives long after competence and costs hours.

Write the map down in the working notes or the PR description. An unrecorded orientation is repaid in full every session.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Hours in, still reading, no file named | Depth-first from the entry point instead of the three-pass sweep |
| The README's design matches nothing you find | Docs pinned at the last time someone cared; trust CI and imports |
| Cannot tell which of five similar modules is live | Never checked churn; dead code and live code read identically |
| Read the framework's source to understand the app | Depth escape; the app's use of the framework is the fact you needed |
| Same questions re-derived every session | Orientation was never written down |
| Estimate off by 4x after a "quick look" | Stopped at the folder tree, never counted entry points or layers |

## Red flags

Thoughts that mean you have stopped orienting and started avoiding the task:

- "I should understand the whole module before touching it."
- "Let me just follow this one call chain."
- "I'll read the tests properly once I understand the code" — the tests are how you understand the code.
- "The architecture doc says…" without having checked an import.
- "One more file and it will click."
