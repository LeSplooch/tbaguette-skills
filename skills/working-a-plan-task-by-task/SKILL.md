---
name: working-a-plan-task-by-task
description: Use when a written plan is ready to execute inline, in the current session — whether it was authored earlier in this session or handed over already-written — or when partway through a plan and needing to keep task order and verification from drifting. Covers critically reviewing a plan before starting, per-task checkpoints, keeping progress in the plan document rather than in your own context, and stopping at a genuine blocker instead of guessing past it.
---

# Working a plan task by task

## Overview

A plan is a set of predictions about what each step will do once it lands. Executing it is not transcription of those steps — it's checking every prediction against reality before the next task builds on top of it. The checkpoint is what turns "I did the steps" into "I verified the steps did what the plan said they would."

This mode is inline: every task runs in the current session's main thread, with no fresh subagent spun up per task. That keeps the whole plan in one head, which is the point — and also means your context carries every task's debris, so the plan document, not your memory, stays the record of what's done.

## When to use

- A plan already exists — written earlier in this session or handed over as a finished document — and the next move is building it.
- Partway through a plan and at risk of losing track of which task is verified, which is guessed at, and which hasn't started.
- Worth pairing with `isolating-work-with-worktrees` before task one: a plan is exactly the multi-step work where a bad step partway through shouldn't cost the checkout you started from.
- Not for: authoring the plan (see `structuring-an-implementation-plan`).
- Not for: dispatching a fresh subagent per task with a review loop (see `delegating-tasks-with-review-gates`).

## Review before touching code

Read the whole plan before starting task one, then review it critically — not for typos, for soundness: is the task order right, does each step name a concrete verification, is there a gap a stranger executing this would trip on. Raise concerns before starting, not at task four when the gap has already become a blocker.

If the plan is unsound at the level of approach — not one unclear step but the wrong shape entirely — stop here and get it revised before running anything.

## One task at a time

For each task: mark it in progress, follow its steps as written, run the verification the plan specifies, mark it complete. Then move on.

Tick the plan's own checkboxes as you go — a plan written by `structuring-an-implementation-plan` uses `- [ ]` syntax on every step precisely so progress lives in the document rather than in whoever is holding it. Update them in the file, at the moment the step actually passes, not in a batch at the end reconstructed from memory. A plan whose boxes are all still empty at task six is one where a compaction, a crash, or a handoff loses everything about where the work stood. When this execution is one phase of a larger run, the checkboxes are the plan's own record and the run record is the arc's — `orchestrating-work-end-to-end` owns the second one; keep both, and keep them agreeing.

Verification happens at the checkpoint, not at the end. A ten-task plan with one verification pass after task ten has let nine tasks drift unchecked. Run the plan's stated check and read what it actually printed before ticking anything — `confirming-before-claiming-done` is the standard the checkpoint has to meet, and "the step looked right" doesn't meet it. Where a task's steps are red-green-refactor, `writing-the-failing-test-first` owns the order they run in; this skill only requires that the plan's own steps all actually ran.

Follow the steps as written rather than improvising a shortcut that skips one — a well-written plan's steps are already bite-sized; the discipline is running all of them, not judging which look safe to skip.

## Stop at a genuine blocker

A missing dependency, a failing test, an instruction that doesn't parse, or a verification that still fails on retry — these are blockers. Stop and ask rather than push through on a guess; a wrong assumption doesn't cost one task, it costs every task built on top of it before someone notices.

A blocker is not the same as a wrinkle. A step whose wording is ambiguous but whose intent is clear isn't a blocker — use judgment, note the interpretation, keep going. A step that contradicts what's actually in the codebase is a blocker. And when the problem turns out to be the plan's approach, not just one step, that's a reason to return to review rather than improvise past it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A late task fails for a reason task three should have caught | Verification deferred to the end instead of run at each checkpoint |
| Half the plan silently reinterpreted by the time it's done | Concerns never raised at review, so ambiguity got resolved ad hoc, task by task |
| A wrong assumption baked into three more tasks before anyone notices | Guessed past a blocker instead of stopping |
| The plan's approach turns out wrong at task four | The pre-flight read was for typos, not soundness; the shape was never actually judged |
| Nobody can say which tasks are done after a context loss | Checkboxes left untouched, so progress lived only in the session that lost it |
| The plan says one thing, the code does another, and no one recorded which won | A contradiction between plan and codebase treated as a wrinkle to interpret rather than a blocker to raise |
| A failing run names no single step to blame | Several steps run as a batch before anything was checked |

## Red flags

- "This step is basically the last one, I'll skip ahead."
- "The verification's probably fine, I already know this part works."
- "Not sure what this step means, but I can guess and keep moving."
- "I'll tick the boxes at the end — I know what I've done."
- "The plan says X, but the codebase obviously wants Y, so I'll just do Y."
- "I'll run the whole verification once at the end, it's faster that way."
- "I'm not skipping it, I'm just reordering it — it'll still run, later." The discipline is running it at the checkpoint so a failure gets attributed to the task that caused it; deferred-but-eventually-run doesn't preserve that, it just delays finding out which task actually broke.
