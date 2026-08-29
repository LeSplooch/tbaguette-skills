---
name: checkpointing-long-runs
description: Use when work will outlive the context holding it — a plan spanning days or sessions, a sweep across hundreds of files, a run whose conversation is getting long enough to be compacted, or work about to be handed to another agent, another session, or a person. Also use the moment a compaction has already happened and the run has to decide what it still knows. Covers where the durable state lives, which seams are worth a checkpoint, writing the expensive things down before they are lost, re-reading rather than recalling at every boundary, and leaving a successor a brief instead of an archaeology problem.
---

# Checkpointing long runs

## Overview

A long run does not fail by forgetting. Forgetting would be survivable,
because a blank is visible and a blank makes you go and look. It fails by
remembering **fluently and partially**: after a compaction, the account you
hold of your own run is coherent, confident, and missing exactly the things
that were never restated — the dead end that cost an hour, the ruling that
settled an argument, the three files already committed.

So the controller re-dispatches work that is already done, re-explores a path
already proven empty, and re-opens a decision already made, all while feeling
completely oriented. That is the most expensive mistake in this class of work
and the one it is most confident about while making it.

Knowing this is not the hard part, and neither is intending to write things
down. What fails is the writing itself: a note taken at the moment it would
help is always the least urgent thing available, and the note taken later is
taken by the memory it was supposed to guard against. So this skill is mostly
about **what a written checkpoint has to contain** — because a checkpoint with
the wrong fields in it costs the same to write and buys nothing.

`recovering-agent-context` is the receiving end of this problem — what to do
when you arrive cold to someone else's half-finished work. This is the sending
end: what to leave so that arrival is cheap, whether the arriver is another
agent, another session, a person, or you in two hours with a compacted context.

## When to use

- The work will take more sessions than one, or more context than one.
- A sweep is running across enough files that its own progress is state.
- The conversation is long enough that a compaction is plausible.
- Work is about to be handed to a subagent, a colleague, or a later session.
- A compaction has already happened and the run must decide what it knows.
- Not for: arriving at work someone else started — `recovering-agent-context`.
- Not for: composing what a reviewer needs to judge a finished change —
  `handing-off-for-review`.
- Not for: a run that is merely unsupervised. That is `bounding-autonomous-work`,
  and the two compose: an autonomous campaign needs both.

## The rule that makes everything else work

**The record is written when the thing happens, not when the run ends.**

A record assembled at the end is a record written by whatever survived, which
is precisely the memory that cannot be trusted. Every rule below is a
specialization of this one.

## What one checkpoint has to contain

A checkpoint is not a status update, and the difference is three fields. A line
missing any of them is a sentence about the past rather than something the next
actor can act on.

| Field | What it is | Without it |
|---|---|---|
| **Anchor** | Something outside the conversation — a commit hash, a path, a command and the output it printed | "Task 3 done" is a claim. "Task 3 done, a1b2c3d, 48/48" is a checkpoint. Only one of them survives being doubted |
| **Establishes** | What is now known that was not known before, stated narrowly | A line that records activity rather than knowledge, so the next actor cannot tell what they may build on |
| **Leaves open** | What this specifically does not settle | The most expensive omission here, and it has its own section below |

## A negative result has two halves, and only one gets written

"I tried X and it didn't work" is half a finding. It is the half everyone
writes, and on its own it is worse than nothing, because it reads as closing a
door it did not close.

The missing half is the **boundary**: what the attempt actually ruled out, and
what it left standing. Those are almost never the same size as they look.

Pinning the clock and seeing no change reads as *time is not involved*. What it
establishes is far narrower — the clock **in this process** is not involved. A
cache with its own expiry, a database with its own `now()`, a broker with its
own visibility timeout each keep their own clock, and none of them was pinned.
The next actor reads "not a timing issue," skips the whole family, and pays for
it a day later.

There is a second boundary that gets lost the same way: **what the null
actually rests on**. A negative measured against an intermittent failure needs
enough trials to mean anything at all, and three clean runs against a
one-in-six failure rate is not evidence of anything. A negative result that
does not say how hard it was tested will be read as conclusive, because that is
how a flat statement reads.

So the required form of a dead end is four parts, and it is barely longer than
the one part usually written:

```
Tried:       what was done, concretely enough to not repeat it
Ruled out:   the narrow thing this actually eliminates
Still open:  what it looks like it eliminated and did not
Rests on:    how many trials, against what base rate — or "not measured"
```

**Deprioritized is not eliminated**, and the difference belongs in the file
rather than in the head of whoever ran it.

## What goes in, in order of what it costs to lose

Most runs write the cheap half and lose the expensive half, because the cheap
half is the part that feels like progress.

| Write | Cost of losing it | Usually written? |
|---|---|---|
| **Negative results** — what was tried and did not work, and why | Highest. The next actor pays the same hour to learn the same thing, and has no way to know they are repeating it | Almost never |
| **Rulings** — decisions that could have gone the other way, with the losing option | High. Resurfaces as a re-argued question with nobody remembering the reasons | Rarely |
| **Assumptions** — what was settled without an answer | High. Silently becomes a fact, and the fact outlives the run | Rarely |
| **What is genuinely finished**, with the evidence | Medium. Causes duplicate work, which is at least visible | Sometimes |
| **What is left to do** | Low. Usually recoverable from the plan and the code | Nearly always |

The ordering is the point. If a checkpoint has room for one line, it is a
negative result, not a status update.

## Where the state lives

One file, appended to, in whatever the run already uses — the ledger from
`finishing-what-you-started`, the plan document, or the run record from
`orchestrating-work-end-to-end`. A second file competing with the first is two
accounts of progress that will disagree, and the disagreement will be
discovered at the worst moment.

Three properties make it durable rather than decorative:

- **Append, never rewrite.** A rewritten log agrees with whatever you now
  believe, which makes it useless for the one job it has.
- **Anchored to something outside the conversation.** A commit hash, a file
  path, a test command and its output. "Task 3 done" is a claim; "task 3 done,
  a1b2c3d, 48/48" is a checkpoint.
- **Readable by someone with no context at all.** Names spelled out, paths
  absolute enough to find, no pronouns pointing at things only this
  conversation saw.

## The seams worth a checkpoint

Not every N minutes. Checkpoint at the places where state becomes expensive to
reconstruct:

- **After anything irreversible or externally visible** — a commit, a push, a
  deploy, a sent message. Before it, the world and the record agree by default;
  after it, only the record says so.
- **At every phase or task boundary**, including one that failed.
- **The moment a dead end is confirmed dead.** This is the one people skip,
  because a dead end feels like nothing happened. It is the most valuable line
  in the file.
- **Before dispatching anything in parallel**, so that a partial fan-out is
  legible rather than a mystery.
- **When the context is visibly filling.** Not after — the checkpoint written
  under compaction pressure is written from a memory already degrading.

## After a boundary, read; do not recall

At the far side of a compaction, a session restart, or a handoff, the order of
trust is fixed and it is not close:

1. The repository — `git log`, `git status`, the diff, the tests as they run **now**.
2. The record — the ledger, the plan's checkboxes, the phase log.
3. Your own recollection of the run.

Read the first two before acting. A fluent memory of having done something is
not evidence of having done it, and this is exactly the situation that produces
fluent memories of things that never happened. Re-measure counts rather than
restating them; `confirming-before-claiming-done` owns why a recalled number is
not a measured one.

## Handing to a successor

A brief, not an archive, and a brief is a set of named sections rather than a
short essay — a successor scanning for one of these should find it as a heading,
not as a clause inside a paragraph:

- **The goal, in the requester's own words**, quoted rather than paraphrased.
- **Where it is now** — anchored to commits and paths.
- **The single next action**, specific enough to start.
- **What is known not to work**, each one in the four-part form above. This is
  the section that is worth the whole brief, and the first one cut for length.
- **Every open assumption and ruling.**
- **What would make this run stop** if the successor is also unattended.

Length is a feature here, not a cost, and `crouton` decides the budget when
there is one. What is never acceptable is a brief that is short because
compressing it was easier than writing it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A second session redoes committed work | Recollection trusted over `git log`, which is the default after a compaction |
| The same dead end explored twice | Negative results were never written; only forward progress was |
| A whole family of causes skipped, wrongly | A dead end recorded without its boundary, so "pinning the clock changed nothing" was read as "not a timing issue" |
| A negative result quoted later as settled fact | It never said what it rested on, and a flat statement reads as conclusive |
| The plan's checkboxes are all empty at task six | Progress lived in context. A compaction now costs the entire run |
| A decision gets re-argued with nobody remembering why | Rulings never recorded, so the reasoning left with the context |
| The handoff brief is accurate and useless | Written for someone who was there — pronouns, shorthand, unstated referents |
| The record was written up at the end and reads beautifully | It was written by the surviving memory, which is the one thing it was meant to guard against |
| A counted claim in the final report is wrong | The count was recalled from an earlier checkpoint rather than re-measured |

## Red flags

- "I'm holding it all fine" — said by a context about to be compacted.
- "I'll write it all up at the end."
- "That didn't work, moving on" — with nothing written down about what didn't.
- "Ruled that out" — said about an attempt whose boundary was never established.
- "It's not a timing issue" — from one clock, in one process.
- "I think task 4 was done?"
- "The summary can just say what I remember doing."
- "Checkpointing is overhead on a run this size" — the runs that need it are
  exactly the ones where it feels like overhead early.
