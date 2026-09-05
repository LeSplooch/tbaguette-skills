---
name: resolving-merge-conflicts
description: Use when a merge or rebase reports conflicts, when a long-lived branch has diverged and integration looks painful, when a merge is textually clean but the build or the behavior breaks afterward, when the same conflict reappears at every commit of a rebase, when one file conflicts on every single integration, when a repository commits generated build output so every branch conflicts in it, or when deciding whether to abort and re-approach rather than push through a resolution.
---

# Resolving merge conflicts

## Overview

A conflict is two answers to one question. Resolving it means answering the question, not choosing a hunk — and the result compiling proves nothing at all, because the merged tree is a version of the code that has never existed and has never been run.

## When to use

- A merge or rebase stopped with conflicts
- A clean merge was followed by a broken build, or by behavior nobody intended
- The same region conflicts repeatedly through a rebase
- One file conflicts on every integration, regardless of who is working
- Not for: choosing merge, rebase, or squash as an integration strategy — see `landing-a-finished-branch`. Not for isolating branches so they stop colliding — see `isolating-work-with-worktrees`.

## Understand both intents before touching a marker

- **Read both sides' commits, not just their diffs.** What problem was each solving? This is where commit-message quality pays out — and where its absence costs an hour (see `writing-commit-messages`).
- **Name the question in one sentence**: "both sides needed the retry count to vary per caller". If you cannot state it, you do not yet understand the conflict, and any resolution is a guess with a build behind it.
- **Read the whole file and the surrounding commits**, not the marked region. The correct resolution frequently exists on neither side.

## Resolve by re-applying intent

Picking one side is correct only when that side's intent genuinely subsumes the other's. Otherwise the resolution is a third version: one side's structure with the other side's behavior re-implemented inside it.

- **When one side moved or renamed something the other side edited**, take the move first, then re-apply the edit by hand at the new location. Textual merges get this combination wrong silently, and the lost edit is invisible in the merge diff.
- **When both sides added to the same list, registry, or switch**, the answer is almost always both, in a deterministic order — but check for a collision in whatever the entries key on.
- **Where the reconciliation is large enough to be a change of its own, it is one.** Land the merge with the minimal mechanical resolution, then the reconciliation as a separate commit on top.

That last rule exists because **code introduced inside a merge commit is code nobody reviews**. Diff tools hide it by default, review interfaces routinely skip it, and blame attributes it to a merge. Novel logic written during a conflict resolution — under time pressure, in someone else's code — is exactly the code that most needs review, and it is the code least likely to get any.

## Semantic conflicts: clean merge, wrong program

| Pattern | What the merge does | What catches it |
|---|---|---|
| One side renames a function; the other adds a caller under the old name | Merges clean; fails at build, or at runtime in dynamic languages | Full build and test suite on the **merged tree** |
| One side tightens an invariant or precondition; the other adds a call that violates it | Merges clean, builds clean, wrong at runtime | A test asserting the invariant, not the call site |
| One side moves validation from caller to callee; the other adds a caller that validates | Double validation, or none | Reviewing the merge as a change, holding both intents |
| One side adds a field; the other adds a serializer or an exhaustive match | Merges clean, drops the field silently | Compiler exhaustiveness where it exists; a round-trip test where it does not |
| One side changes a shared default; the other relies on the old value | Merges clean, behaves differently | A test that pins the default explicitly |
| Both sides add a migration, a sequence number, or an ordered identifier | Merges clean, collides at runtime | An ordering or uniqueness check in CI |
| Both implement the same helper differently in different files | Merges clean, two implementations diverge from here on | Review, and nothing else |

The systemic fix is to test the merge result, not the branch. CI that validates each branch and the target, but never their combination, misses every row above — the most common gap in an otherwise disciplined pipeline, and the reason merge queues exist.

**A generated artifact committed to the repository conflicts as content and resolves as a revert.** Build output, a rendered site, a generated client, a compiled schema — anything a hook or a pipeline writes back into the tree — conflicts on every single integration, and both sides look equally authoritative because each is internally consistent with its own branch's source. Resolve it to your side and the merge is textually clean, the build is green, the tests pass, and you have just un-published everything that landed while your branch waited: the artifact you kept was generated before those changes existed.

A generated file has no side to take. **Resolve it by regenerating from the merged source**, once the source-side conflicts are settled — `--ours`, `--theirs`, and a careful hand-merge are all wrong, and the hand-merge is the most dangerous of the three because it looks like diligence and produces a file no generator would ever emit. A `.gitattributes` entry cannot regenerate anything for you, so the regeneration stays a manual step or a job for the merge tooling — but marking such a file so git stops producing a plausible-looking auto-merge is worth doing on its own, because the conflict you are shown is safer than the merge you are not.

This also changes what branch lifetime costs, and the branch-lifetime bullet below understates it for such a repository. Conflict *probability* scales with how long you are away; here the *damage* does too, because a stale artifact does not merely fail to include recent work — it actively removes it.

## Make conflicts rarer instead of better

- **Branch lifetime dominates everything else.** Conflict probability scales with how much lands on the target while you are away. Integrate the target into the branch daily so conflicts arrive one at a time, while both intents are still fresh in someone's memory. A branch past 3–5 days should be integrated or split.
- **Agreed automatic formatting**, applied by a tool, deletes the entire class of whitespace conflicts. Do the initial sweep as one mechanical commit and record its hash in the blame-ignore list.
- **A file that conflicts constantly is a design signal, not a merge problem.** Central registries, monolithic constants files, single translation catalogs, and generated lockfiles all serialize the whole team through one file. Fixes are structural: directory-per-entry instead of one list, append-only ordering, ownership split by module, or marking generated files to be regenerated rather than merged.
- **Two people editing the same 50 lines is a coordination failure** that no tool resolves. The cheap fix happens before both finish, not after.

## Reuse and retreat

**Recorded resolution reuse** (`git rerere`) records how you resolved a given conflict and replays it when the identical conflict reappears — which it does at every commit of a long rebase, and at every integration of a long-lived release branch. Enable it *before* starting the rebase; it can only replay what it recorded. Two things only learned by being burned: it will replay a **wrong** resolution silently and indefinitely, so run the tests even on auto-resolved steps; and it makes a bad integration strategy survivable, which is not the same as correct.

Aborting is cheap. A resolution fought through over an hour is not, and it is where the worst code enters a repository. Abort and re-approach when:

- You are resolving the same region for the third commit of a rebase — squash the branch and resolve once, or merge instead of rebasing
- More than about five files conflict in a single step, or you cannot state the conflict's question in one sentence
- Thirty minutes have gone into one step with no clear model of the right answer
- **The other side landed a large mechanical change** — a reformat, a mass rename, a directory move. Abort, apply the identical transformation to your branch as its own commit, then rebase; both sides now agree textually and the conflict evaporates
- The branch is long and the base moved far: rebase onto the commit *before* the disruptive change and merge forward, or re-apply your work as a fresh patch on the new base. Re-applying 200 lines by hand often beats resolving twelve commits, and produces a history someone can read

## Common mistakes

| Symptom | Real cause |
|---|---|
| Resolution took one side wholesale, everywhere | The other side's intent was never read; this is the fastest way to silently delete someone's work |
| Merge clean, build broken | Semantic conflict — the text merged, the meaning did not |
| Merge clean, build green, behavior wrong | An invariant changed on one side and no test asserted it |
| A merge silently reverted work that had already shipped | A committed build artifact was resolved to one side instead of regenerated from the merged source |
| The same conflict resolved six times in one rebase | Long branch rebased commit-by-commit without recorded reuse or a prior squash |
| Merge commit contains code present in neither parent | Reconciliation written inside the merge, therefore unreviewed |
| One file conflicts on every single integration | A structural problem in that file, being paid for as a merge problem |
| Someone's change vanished and nobody noticed for weeks | Resolved hunk by hunk, verified by "it compiles" |
| Post-merge history is unreadable | A rebase fought to completion where an abort and a different base was the answer |

## Red flags

- "I will take mine and let them re-apply theirs"
- "It compiles, so the merge is fine"
- "I will sort out the merge in a follow-up commit"
- Resolving a conflict in code you do not understand without asking whoever wrote it
- Being unsure which commit of a rebase you are currently on
- Reaching for a merge tool before having read either side's commit messages
