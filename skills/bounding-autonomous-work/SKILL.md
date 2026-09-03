---
name: bounding-autonomous-work
description: Use when a stretch of work will finish before any human reads a word of it — a delegated task, a goal handed over instead of a plan, a subagent dispatched without a way to ask, a hook or cron or loop with no reader, or a question just asked into a silence that the run is about to answer for itself. Also use when a run is about to defer something to a human because it believes it cannot verify it. Covers substituting each approval gate rather than skipping it, telling a real door from an untried one, the four pre-committed stop conditions that halt a run instead of letting it drift, the actions no confidence level licenses without a human, and reporting to someone who was not there.
---

# Bounding autonomous work

## Overview

An agent working with someone present is bounded by that person. Every few
turns they see what happened and can say stop, and that single mechanism is
quietly enforcing a dozen obligations nobody has written down. Remove the
person and those obligations do not get looser — they get **unwritten**. The
run keeps all of them and loses the only thing that was checking any of them.

What replaces that is not more care. Care is what the run already has, and
care is exactly what fails here: a careful agent alone in a room will answer
its own question, believe the answer, and build six hours of competent work on
top of it. The replacement is bounds, written down before the work starts, in a
form that fires without anyone reading them.

The failure this exists to prevent is not a runaway agent doing something
alarming. It is the ordinary one — a long unsupervised run that did good work
in the wrong direction, because the question that would have caught it in
minute three was asked into an empty room and then quietly settled by the only
party present.

## When to use

- Work will finish before anyone reads a word of it.
- A *goal* was handed over rather than a plan: "get X working", "clean this
  up", "keep going until the suite passes".
- A subagent is about to be dispatched with a task it cannot ask questions
  about.
- A hook, a cron, a loop, or a CI step will run this with no reader.
- A question was just asked, nothing answered it, and the run is about to
  continue anyway. That moment is the trigger, not a reason to skip this.
- Not for: how much deliberation one decision deserves — `deciding-reversibility`.
- Not for: when a polish loop has stopped paying — `knowing-when-to-stop` covers
  diminishing returns. Stop conditions here fire whether or not returns are good.
- Not for: work that merely takes a long time with someone watching. That is
  `checkpointing-long-runs`.

## Three bounds, all written before the first action

1. **The decision bound** — what you may settle alone.
2. **The stop bound** — what makes you halt and report rather than continue.
3. **The door bound** — what no confidence level licenses without a human.

Written *before* is not a stylistic preference. Each of these has to be
decided by a version of you that does not yet want the run to succeed. Written
at the moment they would fire, all three are negotiable, and all three get
negotiated.

## 1. The decision bound: substitute the gate, never skip it

An absent answerer does not delete a gate. It changes what closes it.

| Gate | With someone there | Alone |
|---|---|---|
| Approval of a design | An explicit yes | The design written down, the approach that lost named with its reason, a reversibility bound, and a stop condition that fires if it turns out wrong |
| A clarifying question | Ask, one at a time | Answer it from the code if the code answers it. Otherwise pick, and record it as a ruling naming the other reading — never as a fact |
| "Is this what you meant?" | Ask | Re-derive each acceptance line from the request's literal words, quoted, and check the built thing against the quote rather than against your memory of it |
| A second pair of eyes | A human, or a reviewer subagent | `red-teaming-your-own-work` then `karen-and-the-manager`, run as hard gates rather than as a courtesy, ideally by a reader with no memory of building it |
| Permission for something costly | Ask, wait | Not substitutable. See the door bound |

A decision made alone is legitimate when four things are true of it: it is
**written where the absent reader will find it**, it is **named as a decision**
rather than presented as a fact, it **carries the reading that lost**, and it is
**cheap to reverse** — or it was escalated precisely because it was not.

The tell for an illegitimate one is its grammar in the final report. A
legitimate decision reads *"set the retry cap to 5 — the spec was silent, 3 and
10 were both defensible, one line to change."* An illegitimate one reads *"the
retry cap is 5."* The second sentence has laundered a choice into a fact, and
the reader now has no idea a decision was ever made.

## 2. The stop bound: four conditions, pre-committed

Write these into the run record before the work. Each names something
**observable** — a count, a path, a named system, a diff against the ledger —
because a stop condition that names a feeling never fires.

| Condition | Fires when | What it catches |
|---|---|---|
| **Budget** | The same gate has failed a stated number of times — three attempts at one green test, two rewrites of one function, one hour on one task | The loop that is converging on nothing while every individual attempt looks reasonable |
| **Surprise** | The work reaches a file, subsystem, service, or concept that the frame never named | The scope that grew sideways rather than forward, which is invisible from inside because each step was small |
| **Door** | The next action would be irreversible, external, or expensive to undo | The careful six-hour run that is reckless in its final command |
| **Drift** | The acceptance ledger would have to be edited to accept what was built | Success redefined into reach, which is the single most comfortable failure available to an unsupervised run |

Two rules make them real.

**A stop condition halts and reports; it does not ask.** There is nobody to
ask. The run stops with the work in a recoverable state, the record current,
and a report that names what fired, what was done, and what the next actor
would need. Stopping cleanly is a successful outcome of an autonomous run —
the mode it exists to make available.

**A fired condition is never reasoned past.** "Budget says stop, but I can
see the fix" is the sentence this whole skill is written against, and it is
almost always true and almost never worth acting on. If a condition fires and
turns out to have been badly drawn, that is a finding for the report, not a
licence to continue under a condition you have just rewritten to permit you.

## 3. The door bound

These are not substitutable, not by care, not by confidence, and not by how
obviously correct the action is. An autonomous run may prepare each of them in
full — write the command, stage the change, draft the message — and may not
take the step.

- Anything irreversible against real data or real systems: a destructive
  migration, a delete, a truncate, a force-push, a rewrite of published history.
- Anything that leaves the machine and reaches a person: a sent message, a
  published artifact, a posted comment, a release.
- Anything spending money or a quota that someone else pays for.
- Anything touching credentials: rotating, revoking, granting, or moving one.
- Anything the frame did not name and cannot be undone — the combination is
  what matters; either alone is survivable.
- Anything the requester specifically said to ask about, however trivial it
  turns out to be.

An autonomous run that hits a door is not blocked. It has reached its correct
end: the preparation is done, the action is written down, and a human takes
one step instead of the whole run.

### A door you have not tried is not a door

Everything on that list is an **action**. The common failure is not taking one of
them by accident; it is stopping in front of something that only resembles one. A
run concludes that a change cannot be verified without the person who owns the
system — the credential is not here, the service is unreachable, the real values
cannot be obtained from this machine — and defers. That feels like exactly the
humility this skill asks for, and it is a different thing.

A door is an action whose consequences a human has to own. Being unable to
*check* something is a claim about your own capabilities, and it stays a claim
rather than an observation until an attempt has actually failed. The two are easy
to confuse: the sentence has the same shape, the paragraph above has just
finished saying that stopping is a correct ending, and deferring costs nothing at
the moment it is written. It is not free afterwards. An unattended run exists to
spend its own time instead of the requester's, and an unnecessary deferral
inverts that — it hands back the one thing the run was supposed to absorb, and it
arrives looking like diligence.

So before writing one down: **name the cheapest experiment that would settle it,
and run that experiment.** A throwaway probe program, one request against the
real thing, a single value put through the real encoder. The tell for a deferral
that has not earned itself is that its justification names a capability nothing
in this session has ever tried — and a capability you have not exercised is a
guess about yourself, held to a lower standard than any other claim in the
report.

This applies while you are still deciding whether something is a door, never
after a stop condition has fired. A tripped condition is not a deferral awaiting
a probe — section 2 governs it unchanged, and it is not reasoned past.

An experiment that answers immediately earns one more look than one that
struggles. A probe reading a default, a cache, or a stub returns a confident
value in the right shape, and `reproducing-bugs` covers proving the instrument
before trusting what it says. Withdrawing a deferral on a bad probe is worse than
the deferral was.

A probe that succeeds does not move the door it was testing. Establishing that
you *could* rotate the credential or reach the live service is not permission to
do either, and the confidence a working probe produces is exactly what makes
taking the step next feel like a formality. The experiment settles what you can
*know*. What you may *do* is unchanged by how well it went.

The experiment is bound by the same list as everything else. One that would
itself take an irreversible action, spend someone's money, or reach a person is
not a cheap experiment — it is the door, and the deferral was right the first
time. What this asks for is the read-only half: the probe that observes, the
request that only fetches, the encode that throws its output away.

If the experiment is genuinely out of reach, the deferral stands, and it is now
worth reading: it can say what was attempted and how it failed, which is the
difference between a handover and a shrug.

## Reporting to someone who was not there

The report is the entire deliverable, because the conversation that would
normally carry the rest of it did not happen. It carries four things that a
supervised run's summary can leave implicit:

- **Every ruling**, in decision grammar, not fact grammar.
- **Every assumption** that the code could not settle, with the reading that lost.
- **What stopped and why** — including conditions that fired and were correct.
- **What was surrendered**, named as surrendered rather than dropped.

Set the altitude by what the reader will do next, per `explaining-technical-work`,
and mark each claim verified, inferred, or assumed, per `calibrating-confidence`.
An unsupervised run's report is read by someone reconstructing hours they did
not watch; a confident summary that turns out to be inferred costs them the
whole reconstruction.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Six hours of good work in the wrong direction | A question was asked into a silence and answered by the asker, unrecorded |
| The report reads as a description of a finished system | Rulings written in fact grammar; every choice laundered into a property |
| A stop condition that never fired all run | It named a feeling ("if things go wrong") rather than an observable |
| The run halted and left a broken tree | Stopping was treated as failure rather than as a designed outcome, so nobody planned the state it stops in |
| The acceptance criteria were met and the requester was unhappy | Drift fired silently: the ledger was edited to match the build rather than the build to match the ledger |
| The final command did the damage | The door bound was checked once at the start, when nothing was irreversible yet |
| An open question in the report that one command would have answered | A verification was filed under the door bound because checking it resembled doing it |
| The subagent came back confident and wrong | It was dispatched with a goal and no bounds, which is this skill's trigger, not an exception to it |

## Red flags

- "I'll ask about that at the end." There is no end with a reader in it.
- "This cannot be verified from here" — written without naming an attempt that was made and failed.
- "Nobody's around, so I'll use my best judgment" — said instead of writing the
  judgment down.
- "Budget's blown but I'm nearly there."
- "This is technically irreversible but it's obviously fine."
- "This can't be verified without them" — said without naming the experiment that
  would have verified it.
- "I've already proved it works, so actually doing it is a formality." The probe
  moved; the door did not.
- "I'll note the assumptions in the summary" — assumptions noted later are
  assumptions remembered, and half of them will not be.
- "The task said keep going until it passes, so I kept going."
