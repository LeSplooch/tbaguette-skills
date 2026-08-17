---
name: landing-a-finished-branch
description: Use when implementation is done and its tests are green and it's time to decide how the branch lands, when choosing between a local merge, a pushed pull request, or leaving a branch alone, or when a worktree and its branch need to be torn down after the work has actually landed. Covers verifying the tree that's really about to ship, confirming the base branch, the merge/rebase/squash decision and what each does to history, and cleanup ownership for worktrees and branches.
---

# Landing a finished branch

## Overview

A branch is finished when its tests are green on the tree that's actually about to ship — not the tree from earlier in the session, and not a guess that the user is probably done with it. Verifying that is the only part of this you get to decide alone. What happens next — merge, rebase, squash, push a pull request, or leave it alone — is one choice with consequences for the branch's history, the branch itself, and the workspace it lived in, and that choice belongs to whoever owns the branch.

## When to use

- Implementation is done, the suite is green, and it's time to decide how the branch actually lands.
- Choosing between merging locally, pushing a pull request, or leaving a branch exactly as it is.
- Tearing down a worktree and its branch once work has landed — not as routine housekeeping before then.
- Not for: isolating work before it starts (see `isolating-work-with-worktrees`).
- By the time these decisions apply, the branch's commits should already read the way `atomic-commits` describes — one commit, one decision. That's the history a merge or rebase preserves and a squash throws away.

## Deciding it's actually finished

Run the full suite against the tree that's actually going to be integrated, not a memory of a green run from earlier in the session — a rebase, a fixup, or one more commit since then all invalidate it. If the suite is red, stop there: report the failures and don't move on to how the work should land. Offering an integration choice presupposes the branch is ready, and a red suite means that question hasn't arrived yet.

The other half of "finished" is knowing what it's finished *relative to*. Name the base branch — usually obvious from the branch's upstream or whatever plan started the work — and confirm it if there's any doubt. Getting the base wrong is one of the few mistakes here you can't cheaply take back once it's pushed.

## What's on the menu

Where the branch lives decides what you're even allowed to offer:

- **A normal checkout, branch checked out directly.** Every option applies, and there's no worktree to reason about afterward.
- **A worktree with a named branch.** Every option still applies, but cleanup afterward only runs from outside the worktree, and only for worktrees you created.
- **A detached HEAD in an externally managed workspace.** There's no local branch to merge into anything, so local merging isn't an option at all; the real choices shrink to pushing as a newly named branch or leaving it alone, and the workspace itself belongs to whatever put it there, not to you.

Tell these apart before offering anything: compare the repository's git directory against its common git directory, and check whether HEAD resolves to a branch name or is detached.

With that settled, lay out whichever menu applies and wait for an answer:

- Normal repo or named-branch worktree: merge back to the base branch locally, push and open a pull request, or keep the branch as-is.
- Detached HEAD: push as a new branch and open a pull request, or keep it as-is.

State it plainly and wait. Don't infer a decision from "they seem done with this" and don't act on anything short of a direct answer — which option to take is the one call in this whole process that isn't yours. A fourth path, discarding the work outright, is never part of this menu; it only exists in response to an explicit request to throw the branch away, covered on its own below.

## Merge, rebase, or squash

Each way of landing a branch's commits leaves a different shape in history:

| Approach | What trunk gets | Bisect afterward | Costs |
|---|---|---|---|
| Merge commit — local `merge`, or a forge's default "merge" button | Every branch commit, in order, plus one commit joining the two histories | Walks each original commit; atomic ones still stand alone as search steps | A busier trunk graph |
| Rebase — branch replayed onto the base's current tip, or "rebase and merge" | Every branch commit, in order, no merge commit — trunk stays linear | Same as a merge commit, without the extra node | Every replayed commit gets a new SHA; anything anchored to the old ones — an in-review comment, a branch stacked on top — has to move too |
| Squash — a forge's "squash and merge", or `merge --squash` | One commit, the whole branch flattened into it | Lands on the feature as a single suspect, not the commit that actually caused the problem | Whatever atomicity the branch had is gone the moment it lands |

Which one fits depends on what the branch's history already looks like. A branch built the way `atomic-commits` describes has a real sequence worth keeping, so a merge or a rebase preserves something real. A branch that's five commits of "wip" and "fix" has nothing in that shape worth protecting — squash it and let the pull request description carry the story instead.

## Carrying out the choice

**Merge locally.** Switch to the base branch, update it, merge the feature branch in, then run the suite again on the *merged* result — not the pre-merge branch, which proves nothing about the merge itself. If the merged tree fails, stop: nothing has been pushed, so the branch and worktree are still fully recoverable. Leave them exactly where they are and investigate. Only a green merged result earns cleanup and the branch's deletion.

**Push and open a pull request.** Push the branch, open the pull request through the forge's own tooling, following its template if it has one, and report the URL back. The worktree stays — review feedback gets addressed there, not in some fresh checkout.

**Keep as-is.** Report where the branch and worktree currently sit, and stop. Nothing about this option needs touching.

**Discard.** This path only opens when the user asks for it outright — never inferred from a branch that looks abandoned or a feature that looks finished. Before doing anything, show exactly what would be destroyed: the branch name, its full commit list, the worktree path. Then ask them to reply with the literal word "discard", say that nothing less will do, and wait for it — an agreeable-sounding reply is not that word, and a confirmation nobody was told how to give is a gate that only looks like one. Once it arrives, remove the worktree and force-delete the branch.

## Cleaning up

Cleanup only runs after a local merge or a confirmed discard. Pushing a pull request or keeping the branch as-is always leaves the worktree exactly where it was.

A plain checkout with no worktree has nothing to clean up beyond the branch itself.

Where a worktree is involved, ownership decides what you're allowed to touch: only worktrees created under a `.worktrees/` or `worktrees/` path are yours to remove. Anything else belongs to whoever put it there — a host environment, a person, another tool — and stays put; reach for a platform's own workspace-exit mechanism if one exists, rather than deleting the directory by hand.

If removal is refused because the worktree still holds modified or untracked files, that refusal is telling you something: those files exist nowhere else — an uncommitted note, a scratch file, a plan nobody saved yet. Never force past it on your own judgment. Show the user exactly what's at stake and let them choose — commit it, move it somewhere durable, or delete it — then remove the worktree.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Branch merged and reported done, then the merged tree turns out broken | The suite was verified on the pre-merge branch, never on the tree that actually got integrated |
| Work merged straight away with no menu offered | "They seem to want this landed" was treated as an actual decision |
| Branch and worktree deleted after a vague, agreeable-sounding reply | Discard needs the literal word, not a reply that merely sounds like consent |
| An unrelated worktree removed along with the finished one | Cleanup ownership was assumed instead of checked |
| A removal refusal pushed through with `--force` | The refusal meant those files existed nowhere else; forcing destroyed them instead of asking |
| Merge lands on the wrong parent branch | The base was assumed from habit instead of confirmed against the branch's actual fork point |
| A rejected push gets force-pushed to make it go through | The rejection meant the remote had moved; that needed investigating, not overwriting |

## Red flags

- "Tests were green earlier — no need to run them again."
- "They obviously want this merged, I'll just do it."
- "This looks abandoned, I'll offer to discard it for them."
- "That reply basically meant yes."
- "The PR's open now, so the worktree's just clutter."
- "This other worktree looks stale too, I'll clean it up while I'm in here."
- "The merged build failure is probably just flaky."
