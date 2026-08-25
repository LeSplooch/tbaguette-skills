---
name: writing-durable-docs
description: Use when writing or restructuring documentation — READMEs, guides, API references, onboarding material, architecture notes — when existing docs have gone stale or contradict the code, when deciding where a doc belongs or whether to delete one, or when a single page is trying to be a tutorial and a reference at once. Covers doc types, rationale over mechanics, colocation, executable examples, and stale-doc removal.
---

# Writing Durable Docs

## Overview

Documentation rots because it duplicates what the code already states, and the copy always loses. Durable docs record what code cannot express — the why, the constraint, the rejected alternative — and where they must state mechanics, they are generated or executed so drift breaks a build instead of misleading a reader.

## When to use

- Writing a README, guide, reference page, or onboarding path
- A doc contradicts the code, or a documented step fails when run
- Deciding where a doc should live, or whether to delete it
- A page has grown four unrelated sections and nobody reads past the first
- Someone asks a question the docs "answer" but nobody could find
- **Not for:** why a decision was made and what lost → `writing-adrs`. What changed between versions → `writing-release-notes`. Explaining one piece of work to one person → `explaining-technical-work`.

## Four types, never two on one page

| Type | Reader's state | Serves | Fails when it |
|---|---|---|---|
| Tutorial | Has no problem yet; wants competence | A guaranteed-success path, one route only | Offers options or explains internals |
| How-to | Has a specific problem; is in a hurry | A recipe for one goal, assumes competence | Teaches concepts or starts from zero |
| Reference | Knows what they want; needs exact detail | Complete, structured, dry, no narrative | Tells a story or picks favorites |
| Explanation | Wants understanding; not at a keyboard | Context, why, trade-offs, history | Contains steps to follow |

The audience question that decides everything: **does this reader already have the problem?** No → tutorial or explanation. Yes → how-to or reference. Get this wrong and no amount of editing helps.

Merging is the default failure: a README that opens with a tutorial, drifts into a partial reference, and buries three how-tos. Every reader then reads the wrong 80% of it, and each type's edits corrupt the others — a tutorial gains options until it stops working, a reference gains narrative until it stops being complete.

## Write the why; the what will drift

- The code is the source of truth for what happens. Prose restating it is a second source of truth that loses on the next commit.
- Three things worth writing down: **the constraint** (why the timeout is 30s and not 5s), **the rejected alternative** (why this is not a queue), **the invariant not enforced by types** (callers must hold the lock).
- Test: delete any sentence a reader could confirm by reading the code in under 60 seconds. If the whole doc dies this way, it should have.
- Same test at comment level. `// retry three times` is noise. `// upstream rate-limiter returns 429 in bursts of ≤3; more retries deepen the burst` survives every refactor of the retry loop.

## Make drift fail a build

Ordered least to most durable. Move each doc up the ladder until the cost stops being worth it.

| Technique | Drift shows up as | When to stop here |
|---|---|---|
| Prose describing behavior | Silent rot | Only for explanation and why |
| Snippet pasted into prose | Silent rot | Never, past ~5 lines |
| Snippet included by reference from a compiled/tested file | Broken build or failing test | Good default for examples |
| Examples executed as tests (doctest-style, example functions, or a script that extracts fenced blocks and runs them) | Failing test | Best for how-to and tutorial |
| Reference generated from the source of truth — schema, types, IDL, CLI parser, migration files | Cannot drift | Always, for reference |

Rule of thumb: any code block over ~5 lines that nothing executes will be wrong within two releases. Either execute it or shorten it below the length at which it can be subtly wrong. Never hand-edit generated reference — the edit is lost and the reviewer learns to ignore that file.

## The claims no build can check

Every rung of that ladder needs something to execute or generate. A claim that something **is** the case has a mechanism behind it; a claim that something is **not** the case has nothing to run, so it is pinned at the top rung — prose, silent rot — permanently.

*Contains no telemetry. Sends nothing to a third party. Has no runtime dependencies. Never writes outside its own directory. Stores no personal data.* These are the sentences readers rely on hardest, because they are usually promises about privacy, licensing, or safety rather than descriptions of a parameter list. They are also the ones nothing in the ladder above can protect.

They break differently, too, and that is what makes them slip. A positive claim goes stale when the thing it describes changes — the same diff is at least in the neighbourhood. A negative claim goes false when something is **added somewhere else entirely**, in a change that never touches the document and has no reason to. Nobody wiring up a new dependency thinks to grep the README for a sentence denying that dependencies exist. So a doc asserting what the software does is part of the software's surface: treat every such claim as a call site of the behaviour, and a behaviour change is not finished while a call site still says the opposite.

The defense is to invert the check rather than execute the doc. Assert the absence the sentence claims, as a test that fails when the forbidden thing appears: grep the built artifact for network calls, fail the build on a non-empty dependency list, assert the written-paths set. That drops an uncheckable prose promise to the bottom rung, where drift breaks a build. Where no such test is possible, the claim needs an owner and a review date like any unowned doc — and it needs to be written down somewhere the person adding the forbidden thing will actually pass, which is next to the code, not in the docs directory.

## A mirror with no generation step is a fork

The same document in two locations is two documents. They drift from the system independently, and from each other, and nothing enforces the mirror because nothing knows it is one. Generate one copy from the other, or diff them in CI; those are the only two states that stay true. When you suspect drift and want it located cheaply, count occurrences of the load-bearing term in each copy — a large asymmetry points straight at the paragraph that moved in one and not the other.

## Colocation and ownership

- A doc lives beside the thing it describes, in the same repo, ideally the same directory. Then a behavior change and its doc land in one diff and a reviewer sees the omission.
- Wikis, shared drives, and separate docs repos have no diff and no reviewer, so they rot invisibly and confidently. Move them in, or accept them with a named owner and a review date.
- Every doc has exactly one of: colocated with the code, generated from the code, or stamped with an owner and a review date. A doc with none of the three is unowned and will mislead someone.
- Link to stable identifiers — a permalink at a tag or commit, a named anchor, a symbol name — never a line number.
- Organize by reader task, not by system structure. A tree that mirrors the module graph is navigable only by people who already know the system.

## Deleting is maintenance

- Wrong docs are worse than no docs. No docs sends the reader to the code; wrong docs are trusted, acted on, and cost more than the code would have.
- Delete on sight when: the feature is gone, the steps fail when run, the last edit predates the module's rewrite, or nobody will claim it.
- The content is in version control. "Might still be useful" is not a reason to keep a page that will mislead someone this quarter.
- 90%-correct pages are the most dangerous: readers who verify one claim extend that trust to the rest.
- Where policy forbids deletion, stamp a dated deprecation banner at the top with a pointer to the live source, above the content — not below it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| README is 800 lines, nobody reads past install | Four doc types merged into one page |
| Docs go stale after every release | They describe mechanics the code already states |
| New hire's setup guide fails at step 4 | The guide was only ever read, never executed |
| Doc says what a function does, reader still can't tell when to call it | Written from the author's mental model, not the reader's entry point |
| Everything is documented, nothing is findable | Organized by system structure; search returns the wrong altitude |
| Reference page disagrees with the schema | Hand-maintained copy of a source of truth |
| "We'll document it after the refactor" | Docs treated as a separate artifact instead of part of the change |
| Team writes docs, then answers the same questions in chat anyway | Doc answers the question the author had, not the one readers arrive with |
| A published doc denies a feature the software shipped weeks ago | A negative claim, which no example or generated reference can ever check |
| Two copies of one document disagree | Mirrored by hand, so there is no generation step and no diff to fail |

## Red flags

- "I'll write the docs at the end"
- Adding a fourth unrelated section to a README
- Copy-pasting a code snippet into prose
- Writing "simply", "just", or "obviously" — each marks a step the author no longer sees
- A page that explains the architecture to a reader who wanted one command
- Keeping a stale page "for reference"
- Documenting a workaround instead of deleting the reason for it
- A document promising the absence of something, with no test that fails when it appears
- The same guide living at two paths, neither generated from the other
