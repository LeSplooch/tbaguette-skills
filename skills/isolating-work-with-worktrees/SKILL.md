---
name: isolating-work-with-worktrees
description: Use when starting work that shouldn't touch the current checkout, before running a multi-step plan you might need to abandon cleanly, or when several tasks are about to run against the same repo at once. Covers judging whether isolation is worth its setup cost, detecting when you're already isolated, preferring a native isolated-workspace tool over a manual git worktree, and worktree placement, safety, and cleanup, including what a failed create step leaves running in the wrong tree.
---

# Isolating work with worktrees

## Overview

A worktree is a second checkout of the same repository on its own branch, so work that goes wrong can be thrown away without touching the checkout something else depends on staying clean — your own current branch a step from now, another agent's in-flight edits, a colleague's review. That protection costs a full setup cycle (install, build) and leaves a workspace on disk that has to be remembered and removed later. Isolation is worth choosing deliberately, not reaching for automatically; once it's chosen, prefer whatever isolated-workspace tool your harness already provides over building one from git plumbing yourself.

## When to use

- Starting work substantial or risky enough that you want a clean way to abandon it.
- Before running a multi-step plan, alongside `working-a-plan-task-by-task` — a bad step partway through shouldn't cost you the checkout you started from.
- Several independent tasks about to run against the same repo at once, pairing with `fanning-out-independent-work` — each one needs a workspace nothing else is writing to concurrently.
- The current checkout already has uncommitted work sitting in it — yours, a colleague's, another agent's — that a new change would step on.
- Not for: deciding whether to merge, rebase, or discard a finished branch (see `landing-a-finished-branch`).

## Detect before you build

Before creating anything, find out whether the question is already settled:

```bash
git rev-parse --git-dir
git rev-parse --git-common-dir
```

Different output means you're already inside a linked worktree — don't nest a second one inside it. Skip straight to setup, below. Matching output means an ordinary checkout, and the trade-off in the next section still needs an answer.

One thing produces the same mismatch without meaning the same thing: a submodule.

```bash
git rev-parse --show-superproject-working-tree
```

A path comes back — you're in a submodule, not existing isolation; treat it as an ordinary checkout. Nothing comes back and the earlier mismatch stands.

Worth noting once you know you're already isolated: whatever created the workspace may have left it on a detached HEAD instead of a branch. If so, a real branch still has to get created before this work can be merged or pushed anywhere. Flag that now, not at the point someone tries to push and it fails.

## Isolation is a trade, not a default

Weigh it like any other setup cost, not as something you always pay. It earns its keep when the checkout you'd otherwise use is one something else depends on staying stable — shared with another agent or person, mid-review, or already carrying unrelated uncommitted work — or when the change itself is exploratory enough that discarding it and starting over is a real, likely outcome. It's overhead when the checkout is already yours and clean and the change is small enough that a normal commit, or just undoing an edit, is the actual worst case.

If nothing has already settled the preference, ask before creating a workspace: name what's being protected, and whether it's worth a second install-and-build cycle. Honor a preference that's already been stated without re-asking. A decline means work in place — nothing past this point applies.

## Native tool first, git worktree second

Isolation can be built two ways. Check for the first before reaching for the second.

Your harness may already manage isolated workspaces under some name — `EnterWorktree`, a `/worktree` command, a `--worktree` flag. If something like that exists, use it and move on to setup below. It's the one thing that knows where its own workspaces live, how their branches get made, and how to take them apart again, and it does all three as a unit your harness can actually track. Reaching for raw git instead, when that tool was sitting right there, doesn't skip a step — it just produces a workspace your harness never learns about and therefore never manages.

Some native tools gate themselves tighter than this skill's own trade-off logic — invoked only on an explicit instruction to use a worktree, not on the judgment call above by itself. Check the tool's own description before assuming "use it and move on" is unconditional. A tighter gate isn't a contradiction to route around by falling back to raw git: it's satisfied the same way an unsettled preference already is — surface the trade-off and ask, per the section above, instead of either working around the gate or abandoning isolation because the tool declined it.

Fall back to plain git only when no such tool exists.

**Where.** An explicit directory named in your instructions wins outright. Otherwise, use whichever of `.worktrees/` or `worktrees/` is already a convention in this project (`.worktrees/` if somehow both are). Absent either, default to `.worktrees/` at the project root.

**Safety.** Confirm the chosen directory is actually gitignored before putting anything in it:

```bash
git check-ignore -q .worktrees
```

Not ignored — add it to `.gitignore` and commit that first. Skip this and the next broad `git add` sweeps the entire second checkout into a commit along with it.

**Create.**

```bash
git worktree add .worktrees/<branch-name> -b <branch-name>
```

A permission error here is a sandbox denial, not a bug worth fighting — say so and keep working in the current directory instead.

## When the create step fails, the next command still runs

In a repository more than one session uses, creation fails for ordinary reasons: the path is already registered to someone else's worktree, the directory exists, the branch is checked out elsewhere. `git worktree add … && cd … && setup` stops at the failure, but only that one chain. The next command, issued on its own line or in its own call, starts from wherever the shell already was — your primary checkout, or another session's tree — and does exactly what it would have done in the fresh one. A patch applied there lands on a checkout that carries someone's uncommitted work; a copy lands in the other session's worktree. The failed create did print an error, and what the next command printed scrolled it away.

Make the wrong tree unreachable instead of hoping to notice it:

- **Choose a path nobody else would.** Put the task or branch name in it, plus something unique to this run. A "natural" name is the one another session also picked. Before creating, check the path does not exist and is not already listed by `git worktree list --porcelain`.
- **Do not let the working directory decide where a mutation lands.** Address the new tree by absolute path for every write (`git -C <abs>`, `cp … <abs>/`), and confirm `git -C <abs> rev-parse --show-toplevel` prints that path before the first one. If a `cd` is unavoidable, it goes in the same `&&` chain as the create, and so does every command that mutates — a guard covers only the commands inside its chain. `portable-shell-scripting`'s section on `cd` has the mechanics of why a failed `cd` or chain stays silent.
- **A printed error is not proof that nothing changed.** Before running any mutating command against a tree you do not own, record its state somewhere outside that tree: the full `git status --porcelain` output, `git diff HEAD --binary > <elsewhere>/snapshot.patch`, and a copy of any untracked file a mistake could clobber, since the diff does not hold them. Compare afterwards. Output that moved is the alarm (a bare count can stay level while the files change); the snapshot is what lets you put the tree back byte for byte instead of from memory.

## Cleanup is not optional

A native tool dismantles its own workspace as part of finishing, which is a large part of why it's worth preferring. A worktree built by hand has no equivalent — it stays until someone deliberately removes it.

```bash
git worktree remove .worktrees/<branch-name>
git worktree prune
```

`remove` while the directory is still there and the work is merged or abandoned. `prune` once the directory is already gone some other way and git is still holding a reference to it. Do this at the same moment the branch's fate gets decided, not as a chore to circle back to — a worktree that's merely "done for now" still has its branch checked out, so git refuses to check that branch out anywhere else until the worktree is gone.

## A new workspace starts at zero

Neither path leaves you with dependencies installed or anything built. Set the workspace up the way this project already declares it should be:

| Manifest present | Setup |
|---|---|
| `package.json` | `npm install` |
| `Cargo.toml` | `cargo build` |
| `requirements.txt` / `pyproject.toml` | `pip install -r requirements.txt` / `poetry install` |
| `go.mod` | `go mod download` |

Then run the test suite once, before changing anything, and note what it says. That's the baseline every later result gets read against — without it, the first failure you hit is unexplained, and there's no way to tell whether it's the workspace or the work. If the baseline itself fails, report that and let the user decide whether to proceed or dig in — don't quietly assume either answer.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A worktree gets created inside another worktree | Detection step skipped; an isolated workspace looks identical to a fresh checkout until the paths are actually compared |
| A submodule checkout gets treated as already isolated | git-dir/git-common-dir mismatch trusted without ruling out a submodule first |
| A native worktree tool exists but never gets used | `git worktree add` reached for out of habit, not because it was the only option |
| Worktree contents show up staged in an unrelated commit | Directory was never confirmed gitignored before work started in it |
| A branch can't be checked out anywhere, no obvious reason | A finished worktree was never removed, so git still considers the branch in use there |
| `.worktrees/` fills with entries nobody remembers creating | Cleanup treated as optional instead of part of finishing the work |
| The first failure in a new workspace is baffling | No baseline test run before starting; no way to tell the workspace from the change |
| A one-line typo fix gets its own worktree | Setup cost never weighed against what was actually at risk |
| A patch or copy meant for the new workspace appears in the primary checkout or another session's tree | The create step failed (path already taken), the chain stopped, and the next separate command ran in the old directory |

## Red flags

- "I'm obviously not already isolated" — a harness-managed worktree and a submodule both look like an ordinary checkout until the paths actually get compared.
- "`git worktree add` is right there, why look for a native tool" — the two minutes spent finding it is what buys automatic cleanup later; skip it and cleanup becomes a manual chore with no reminder attached.
- "This has to already be ignored" — check instead of assuming; an unignored worktree directory is one broad `git add` away from landing in a commit whole.
- "I'll remove this worktree later" — later is where worktrees go to sit forever, quietly keeping their branch unavailable everywhere else.
- "It's a brand-new workspace, the baseline has to be clean" — new isn't the same as passing; run the tests before trusting whatever failure shows up afterward.
- "The create printed an error, but the next step is just a copy" — an error above later output is how the wrong-tree write gets read as harmless; confirm the toplevel of the tree you are about to write to.
- "Better safe than sorry, I'll isolate this one too" — isolation has its own setup cost, and paying it reflexively for a trivial change is the same mistake pointed the other way.
