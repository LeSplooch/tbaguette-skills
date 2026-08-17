---
name: delegating-tasks-with-review-gates
description: Use when executing a multi-task implementation plan task by task in the current session, when each task should go to a fresh subagent carrying none of the session's accumulated history, or when a task's implementation needs checking against both its requirements and its craftsmanship before the next task builds on it. Covers dispatching a zero-context implementer subagent per task, the two-stage review — spec compliance and code quality — that gates each one, working a bounded fix loop when review finds problems, and a final whole-branch review once every task is done.
---

# Delegating tasks with review gates

## Overview

Execute a plan by dispatching one fresh subagent per task, gate each one with a two-stage review before the next task starts, and close with one broader review across the whole branch. A subagent that inherits your session's transcript inherits its dead ends and abandoned approaches along with it — construct exactly what each task needs instead, and nothing it doesn't, so its judgment goes into the task rather than into making sense of a conversation it never joined. The same discipline protects your own context: tracking what's done, ruling on conflicts, holding the cross-task picture no single subagent has reason to hold — that coordination work is what the controller session is for, and it stays clean only if the tasks themselves happen somewhere else.

## When to use

- A plan exists — typically written by `structuring-an-implementation-plan` — and it's ready to execute task by task in the current session.
- Each task should go to a subagent that starts with none of the session's accumulated history: clean judgment instead of a growing transcript to make sense of.
- A task's work needs an independent check — against what it was asked to build and against how well it was built — before the next task is safe to build on top of it.
- Not for: two or more tasks that are genuinely independent of each other and can run concurrently, with nothing gating one on another's result (see `fanning-out-independent-work`).
- Not for: running a plan's tasks inline in the current session's main thread, with no fresh subagent per task (see `working-a-plan-task-by-task`).

## Before task one

Do this work in an isolated workspace — a worktree or a feature branch, never a shared trunk directly (see `isolating-work-with-worktrees` for setting one up).

Read the plan once. If it names a spec, read that too: the spec is the authority the plan argues from, and where the two disagree, the spec wins. A plan with no reachable spec still runs, but treat every ruling you make under it as provisional.

Scan the plan once for problems before dispatching task one: tasks that contradict each other or the plan's own global constraints, and anything the plan explicitly mandates that would itself be a defect under review — a test asserting nothing, a logic block duplicated verbatim. Rule on whatever the scan turns up and write down what you decided; a scan that finds nothing is still worth stating, not just skipping in silence.

Keep a running written record outside your own context, updated after every task and every fix round: which tasks are closed, which commits closed them, and every ruling made along the way. Tick the plan's own `- [ ]` checkboxes as tasks close, too — a plan written by `structuring-an-implementation-plan` carries them for exactly this, and letting them drift out of step with your record leaves two accounts of progress that disagree. Compaction erases what you remember; it does not erase that file or `git log`. Recovering from a lost context means trusting the record and the log over your own recollection — a controller that trusts its memory instead re-dispatches work that's already done, which is the single most expensive mistake this loop can make.

## Decide, don't stall

This loop runs to completion without pausing to check in between tasks — that's the point of keeping it in one session instead of routing every step through a person. Conflicts, ambiguities, a plan silent on some edge case: decide them against the spec (the plan is the spec's argument, not the reverse), write down what you decided and why in a line or two, and keep going. A wrong call costs rework someone can see in the record and undo; a session parked on a question it could have answered itself costs the whole session and buys nothing.

A short list of things actually justify stopping to ask: an irreversible or destructive operation, anything security-sensitive, a side effect that reaches outside this workspace — a push to a shared branch, a merge, a publish — or a plan broken enough that every path forward is a guess. Outside that list, rule and continue.

## Choosing a model for each role

Use the least capable model that can do the role well, and say which model explicitly every time you dispatch. An unspecified model quietly inherits the session's own, which is usually the most expensive one available and defeats the point.

| Role | Default tier |
|---|---|
| Implementer, task has a complete spec and touches 1-2 files | Cheapest tier — the work is transcription plus testing |
| Implementer, multi-file integration or real judgment calls | Standard tier |
| Implementer, architectural or design decisions | Most capable available |
| Reviewer, any task | Scaled to the diff's size and risk, not fixed to the implementer's tier |
| Scoped re-review of a small fix | Cheapest-to-mid tier |
| Fresh implementer after a stuck fix loop | At least one tier above whichever model got stuck |
| Final whole-branch review | Most capable available, always |

Turn count costs as much as per-token price: cheap models routinely take two or three times the turns on multi-step work, which can cost more overall than a mid-tier model would have. Treat mid-tier as the real floor for anything reasoning from prose rather than transcribing an already-fully-specified change.

## Dispatch the implementer

Record the commit the task starts from before dispatching. You'll need it as the base of the diff you eventually hand the reviewer — never assume a task's work is one commit and diff against its parent, which silently drops everything but the last commit of a multi-commit task.

Give the subagent exactly what this task needs and nothing else:

- its own requirements, as a self-contained brief — exact values, signatures, and test cases belong here verbatim, not paraphrased into the dispatch prompt
- the interfaces or decisions earlier tasks produced that the brief itself has no way to know about
- a pointer to any earlier parked or deferred finding that touches the same area, so the implementer doesn't reintroduce it
- the global constraints that bind every task
- where to write its report, and the short status contract it should return with

Never the rest of the plan, and never a summary of the session so far — a dispatch describes one task, not the controller's history. Hand over the brief and the report as file paths, not pasted text: anything pasted into a dispatch, or printed back by a subagent, stays resident in your own context for the rest of the session and gets re-read on every later turn.

Dispatch one implementer at a time. Two working the same tree at once is a conflict waiting to resolve itself badly. Invite questions before it starts work and answer them completely — an implementer that guesses instead of asking is a defect in the dispatch, not in the implementer. The implementer never spawns subagents of its own, and especially never a reviewer of its own: the review seat is already scheduled, dispatched by you once the report lands, and a self-spawned second opinion duplicates it at full cost while counting for nothing.

If several tasks in the plan are the same small edit repeated across files — the same constant, the same one-line fix — compose one dispatch listing every file and its change rather than one subagent per task, and review the batch as a single diff. Save one-dispatch-per-task for work that actually needs its own judgment, its own tests, or its own review.

Template: [reference/implementer-prompt.md](reference/implementer-prompt.md)

## Handle what comes back

| Status | What it means | What you do |
|---|---|---|
| DONE | Work complete, self-reviewed, committed | Move to review |
| DONE_WITH_CONCERNS | Complete, but the implementer has doubts | Read the concerns first — resolve anything about correctness or scope before review; note-only observations can wait for review |
| NEEDS_CONTEXT | Missing information blocked progress | Supply what's missing, re-dispatch |
| BLOCKED | Cannot complete as given | Diagnose why, then act |

A BLOCKED report is a signal to change something, not a cue to retry unchanged: more context if the gap is understanding, a stronger model if the task needs more reasoning than the current one has, a smaller task if it's simply too large, or a correction — ruled on and recorded — if the plan itself is wrong. Forcing the same model to retry a task it already called too hard wastes the round and tells you nothing new.

If the implementer asks questions, mid-task or before starting, answer in full before it continues. Don't rush a subagent into work it flagged as unclear.

## The two-stage review

Every task gets a fresh reviewer against its diff before the next task starts — never the implementer that built it, and never a reviewer the implementer spawned itself. This is a task-scoped gate, not the broad review; that happens once, at the end, across the whole branch.

The reviewer returns two verdicts, and both are required. Neither substitutes for the other, and the implementer's own self-review substitutes for neither:

- **Spec compliance** — does the diff match what the task asked for: nothing missing, nothing extra, nothing solved the wrong way.
- **Code quality** — is it well-built: clean separation, real error handling, tests that verify behavior rather than mocks, no file quietly outgrowing its one job.

Give the reviewer a real scope, not an open-ended one: the diff and the brief, plus the global constraints copied verbatim from the plan or spec — exact values, exact formats, the relationships between components the plan actually states. It should read the diff and the implementer's report, not crawl the wider codebase or re-run a suite the implementer already ran and reported on; a focused check is warranted only against a specific, named doubt. Never pre-judge a finding for it — "don't flag X" or "at most Minor" is you grading the work in advance, which is exactly what the second reviewer exists to avoid.

The reviewer may flag a requirement it cannot verify from the diff alone — something that lives in unchanged code or spans several tasks. That doesn't block anything by itself, but it's yours to resolve, not the reviewer's: you hold the cross-task context it doesn't. Confirm it one way or the other; a confirmed gap is a failed spec review and enters the fix loop like any other finding.

Template: [reference/reviewer-prompt.md](reference/reviewer-prompt.md), full-review mode.

## When review finds problems: the fix loop

Two kinds of finding leave the loop before it starts:

- **Minor findings** get noted in the running record and deferred to the final review, which decides what actually needs fixing before merge. They never trigger a round.
- **A finding that conflicts with what the plan's text actually mandates** is yours to rule on, the same way any plan conflict is: weigh it against the spec, decide, write down why. Don't dismiss the finding just because the plan called for it, and don't dispatch a fix that overrides the plan without a recorded ruling either.

Everything else enters the loop. One round is one fix dispatch plus one scoped re-review:

- **Early rounds resume the implementer that did the work.** It still has the task, the code, and its own reasoning intact — hand it the open findings verbatim.
- **A loop that survives a couple of rounds with the same implementer** is usually telling you it can't see its own problem, not that it needs one more try. Move to a fresh implementer with no sunk cost in the current approach, on a model at least one tier stronger, and say plainly that a prior attempt is written up in the report file for it to read.
- **Every round ends the same way.** The implementer fixes, re-runs the tests that cover the amended code, and appends a fix report: what changed, which tests, the command, the output. Then a re-review, scoped to exactly two questions — was each finding actually addressed, and did the fix itself break something new. Anything the re-reviewer notices outside that scope goes to the deferred list, not back into the loop. Template: [reference/reviewer-prompt.md](reference/reviewer-prompt.md), scoped-re-review mode.

Give the loop a small, fixed cap rather than letting it run open-ended. A loop that hasn't converged in a handful of rounds is a structural problem, and more rounds spent hoping is not a strategy. At the cap, stop dispatching and adjudicate every open finding yourself:

- Wrong, or genuinely arguable: park it, with the reasoning recorded, so the final review sees both sides.
- Real, but nothing later depends on it: park it too, with a ruling that says it's real and deliberately deferred.
- Real and load-bearing — a later task builds on it, or it reveals a defect in the plan itself: rule on the smallest change that unblocks what depends on it, record what you decided, and carry that ruling into the next task's dispatch.

Every adjudication is a recorded ruling. A finding dropped with no line explaining why is a defect that will resurface later with no memory of why it was let go.

Close the task once its review is clean, or once every remaining finding is parked or ruled at the cap — never earlier, and never move to the next task while a Critical or Important finding sits neither fixed nor resolved.

## The final review

Once every task is done, dispatch one review across the whole branch — not per-task scope this time, and on the most capable model available regardless of what individual tasks used. Compose that dispatch the way `handing-off-for-review` describes: what was built, what it should do, the exact range, and the shape you want the findings back in. Hand it the running list of deferred minors and parked findings too, so it triages what actually blocks merge instead of rediscovering all of it from nothing.

If it finds something, fix the whole list in one dispatch — not one fixer per finding, which rebuilds context and reruns the suite for each one at a cost that can exceed every task that came before it — then run exactly one scoped re-review of that fix. Adjudicate whatever's left the same way the task loop's cap works: park it or rule on it, and record which. There's no second fix wave; anything load-bearing left after that surfaces to whoever owns the branch next.

Once the final review is clean, this skill's job is done. How the branch actually lands — merge, pull request, or something else — belongs to `landing-a-finished-branch`, not here.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A task marked complete with an open spec gap | "Close enough" stood in for a clean verdict — a failed spec check is not done, only fixed or parked-with-a-ruling |
| The controller edits code directly mid-review | Fixing in the controller session skips the review gate it's supposed to protect, and pollutes the context that's supposed to stay clean |
| A fix round runs six, seven, eight times on the same task | No fixed cap — rounds kept being spent hoping instead of adjudicating a structural failure |
| A fix ships with no re-review because it "was small" | Small fixes regress too; skipping the re-review is how they land unseen |
| The next task dispatches before this one's review is clean or parked | The task loop's gate was treated as optional under time pressure |
| A dispatch prompt balloons with several tasks' worth of pasted history | Session history got pasted in instead of handed over as an interface the brief states plainly |
| The implementer's self-review stands in for the task review | Self-review and independent review catch different things; one was skipped, not just deferred |
| An implementer spawns its own reviewer and the controller treats that as covered | A duplicate review seat, not a second opinion — the real review still has to run |

## Red flags

- "The self-review already looked thorough, I'll skip the task review this once."
- "One more fix round will probably converge" — said again, past the point where it already didn't.
- "I'll just fix this myself instead of dispatching, it's faster."
- "This finding is obviously wrong, I'll drop it" — with nothing written down about why.
- "Should I continue?" surfacing after a task that closed clean, with nothing that called for a stop.
- A dispatch prompt that includes "here's everything that happened in the tasks before this one."
- Reviewing this task's diff with the previous task's context still doing the judging.
