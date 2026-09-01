---
name: fanning-out-independent-work
description: Use when two or more tasks look independent enough to hand to separate agents — unrelated failures, disjoint subsystems, a batch of scoped deliverables, parallel investigations. Covers telling genuine independence from work that only looks independent, avoiding collisions when agents share files or resources, writing prompts that stand alone, and reconciling results once parallel work returns.
---

# Fanning out independent work

## Overview

Independent work parallelizes; related work doesn't, no matter how much it looks like a list. The judgment that matters isn't whether the tasks can be described separately — it's whether either agent's approach would change if it saw the other's output. If not, dispatch both in the same turn: agents issued together run concurrently, agents issued one turn at a time run sequentially no matter how independent the underlying tasks are.

Each agent gets a context built only for its slice — not your session, not the other agents' assignments. That isolation is what makes the parallelism safe, and it's also what frees your own context for the work only you can do: judging independence, writing the prompts, and reconciling what comes back.

## When to use

- Two or more problems are on the table and each can be fully solved without knowing how the others turn out.
- A batch of scoped, disjoint deliverables — port N adapters, audit N modules, draft N documents — where no item's output feeds another item's input.
- Diagnosing one issue yourself would burn the context you need to dispatch the others.
- Not for: working through a multi-task plan one task at a time, each gated by a review before the next starts (see `delegating-tasks-with-review-gates`) — that loop is sequential by design, because every task builds on the one before it.

## Independent, or just described that way

For every pair of tasks headed to separate agents, ask one question: would either agent's approach change if it saw the other's output, or if it touched a file or resource the other also touches? If yes, it's one task wearing two descriptions.

| Disguise | What's actually shared | Resolution |
|---|---|---|
| Shared root cause | Fixing one changes or removes the other's symptom | One investigation, not two — or dispatch the second only after the first lands |
| Shared resource | Same port, branch, seed data, rate-limited API, or file both will write | Give each agent its own copy, or serialize the access |
| Hidden ordering | Step 2 needs step 1's output, flattened into a bullet list | Split into phases; parallelize within a phase, not across them |

A numbered or bulleted list is not evidence of independence. It's evidence someone wrote a list.

## A cold start is a feature, and it is now optional

The isolation above used to be automatic: a dispatched agent got a context built
only for its slice because there was no other kind. Harnesses now also offer the
opposite — an agent that *forks* the current session, inheriting the whole
conversation and the warm prompt cache with it, which makes it markedly cheaper
than a cold one for anything that needs what this session already knows.

That is a real saving and it is the wrong instrument for a fan-out, for the
reason the cold start exists. Every discipline in this skill is enforced by the
dispatched agent knowing nothing: the prompt has to be complete because nothing
else will be there, the write scope has to be stated because it cannot be
inferred, and the report has to stand on its own because the reader has no
shared memory to fill gaps from. A fork removes the cold start and every one of
those forcing functions with it — the prompt that would have failed loudly now
succeeds quietly, on context the agent should not have been reasoning from. And
several forks of the same session are several copies of the same assumptions,
which is the opposite of what dispatching separate agents was for.

Split it by what the task needs to know:

| The task needs | Use | Because |
|---|---|---|
| Only its own slice | A fresh agent | Isolation is what makes the parallelism safe, and the cold start is what keeps the prompt honest |
| Everything this session has established — a review of the diff just written, a second read of the current design | A fork | Reconstructing that context in a prompt is expensive, error-prone, and the part most likely to be summarized wrong |

Cost belongs in the judgment either way, because the multiplier is larger than
it feels: running work across several agents is commonly several times the token
spend of doing it in one session, and every fresh agent pays to rebuild
understanding this session already has. Fan out for isolation and for wall-clock
time — never because a list has more than one item on it.

## Avoiding collisions

Independence in the problem doesn't guarantee independence in the solution — two agents can have unrelated goals and still collide if what they write overlaps. Partition by write-set, not by topic:

- Read overlap is fine. Two agents reading the same file, schema, or doc is not a conflict. Write overlap is the only kind that matters.
- A shared registry, index, changelog, or catalog that every task would naturally want to update is a collision even when the underlying work is independent. Pull it out of the parallel batch and update it once, sequentially, after every agent has landed.
- State each agent's write scope as a hard constraint in its prompt — which paths are its to touch, plus an explicit "nothing outside this" — not as something the task description implies.
- When a clean partition isn't possible and two agents genuinely need the same file, isolate them into separate workspaces instead (see `isolating-work-with-worktrees`) rather than dispatching anyway and hoping the merge goes fine.

## Prompts that stand alone

Each agent starts cold: no access to your reasoning, your conversation, or what the other agents were told. A prompt that leaves anything to inference gets an inference back, and an inference that collides with another agent's inference is how two well-scoped tasks produce conflicting edits to the same line. Include, every time:

- **The specific problem** — the file, the failure, the deliverable — not the general category. "Fix agent-tool-abort.test.ts" beats "fix the failing tests."
- **The scope boundary**, stated as a constraint: which files and directories belong to this agent, spelled out even when it seems obvious from context the agent doesn't have.
- **The output contract**: what shape the report back should take, so it can be integrated without a second round of investigation to reconstruct what the agent actually did.

## Reconciling what comes back

Results land independently; integration happens once, together — not piecemeal as each agent finishes.

- Check for the collisions the dispatch was supposed to prevent: did two agents touch the same file, the same config key, the same exported name?
- Run the full verification pass across the combined result, not per agent — each slice can be internally correct and still break once merged.
- An agent's own summary of what it did is a claim, not a diff; verify the merged result the same way any other completion claim gets verified (see `confirming-before-claiming-done`).

An agent that quietly did more than its assignment — fixed something adjacent, renamed a neighbor while it was in there — is scope drift wearing a parallel-agent costume (see `managing-scope-drift`). Catch it the same way: does the report account for everything asked, and nothing extra?

## Common mistakes

| Symptom | Real cause |
|---|---|
| Parallel agents all made the same wrong assumption | They were forked from one session rather than started cold, so they inherited one set of assumptions instead of testing the prompt against none |
| Two agents' diffs conflict on the same file | Write-sets were assumed disjoint, never actually checked |
| A "fixed" task breaks a sibling task | The two shared a root cause and needed one investigation, not two |
| Integration trusted a summary that said "done" | The combined result was never re-verified |
| An agent changed far more than the task named | The prompt stated a goal but never a scope boundary |
| Parallel dispatch took as long as doing it serially | The tasks were sequential underneath a flat list; one agent sat waiting on another's output |
| A shared file breaks right after every agent reports success | Every agent tried to update it independently instead of once, after, sequentially |

## Red flags

- "These are basically the same kind of task" standing in for an actual independence check.
- One prompt template copy-pasted per agent with only the filename swapped.
- No stated output shape, on the assumption the agent will report something useful.
- Treating "all tests pass" in a summary as true without re-running it.
- A task list whose order turns out to matter, discovered only after two agents are already running.
- Reaching for a forked agent because it is cheaper, on work whose whole point was that it starts cold.
