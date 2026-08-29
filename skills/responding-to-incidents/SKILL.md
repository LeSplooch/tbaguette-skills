---
name: responding-to-incidents
description: Use when something is broken in production right now — an outage, a degradation, a bad deploy, data going wrong in flight, an alert that is real — and there is a clock and an audience. Also use when a debugging session has quietly become an incident because users are affected while you investigate, or when deciding whether to roll back or fix forward. Covers mitigating before diagnosing, holding the three roles when you are the only responder, preserving the evidence your own mitigation would destroy, choosing reversible mitigations over correct fixes, communicating on a cadence, and handing back to an ordinary diagnosis once the bleeding stops.
---

# Responding to incidents

## Overview

Ordinary debugging has one correct first move: reproduce the failure so it is
available on demand, then find where it actually originates, then fix it. An
incident inverts that, and the inversion is the whole skill. While you are
reproducing, users are still broken. **Stop the harm first; understand it
second.** A response that gets the cause exactly right in forty minutes is
worse than one that restores service in four and gets the cause right in an
hour.

This is uncomfortable precisely because the good instincts point the wrong way.
Everything that makes someone a careful engineer — do not act without
understanding, do not change what you cannot explain, do not guess — is correct
in a debugging session and costly in the first ten minutes of an outage.

That inversion is the famous half of this skill and it is not the half that
actually goes missing. Most responders reach for the rollback. What they do not
do, under a clock, alone, is **write anything down** — and every artifact an
incident owes is produced during the twenty minutes when producing it feels
like the least urgent thing available. The timeline nobody kept is the
postmortem nobody can write. The cadence nobody set is the escalation that
arrives while the fix is already working.

So this skill leads with what an incident produces, and argues the ordering
second.

## When to use

- Something is broken in production now, and users, data, or money are exposed.
- A deploy made things worse and the decision is roll back or fix forward.
- An alert fired and is real.
- A debugging session has become an incident: people are affected *while* you
  investigate, and nobody has said so out loud.
- Not for: a bug that is not currently harming anyone — `diagnosing-before-fixing`
  is faster and better for that, and this skill's inversions would be waste.
- Not for: writing it up afterwards — `writing-postmortems`.
- Not for: how to look at a live system without making things worse —
  `observing-production-safely`, which this skill leans on throughout.

## What an incident produces

Six artifacts. Each has required fields, because a field is what makes an
artifact checkable and a paragraph is not; and each is written **at the moment
it happens**, because every one of them is unreconstructable afterwards in a
specific way named in the last column.

| Artifact | Written at | Required fields | Why it does not happen by itself |
|---|---|---|---|
| **Declaration** | The moment you suspect, not the moment you are sure | What is failing, since when, in user terms, who owns it, and the time of the next update | Declaring feels like an admission. The bar drifts up to certainty, and the fifteen minutes before anyone says the word are the ones every postmortem in this category finds |
| **Impact statement** | Beat 2, before any mitigation | Who is affected, how badly, whether it is growing — in user terms, never system terms | "5xx at 30%" is the number you have; "one in three people cannot buy" is the number the decision needs, and nobody converts one to the other under pressure |
| **Timeline** | Continuously, one line per action, timestamped | Time, what you did, what you saw | Memory of an incident is compressed and wrong in predictable ways. This is the postmortem's entire raw material and it cannot be written later |
| **Evidence manifest** | Beat 3, before the mitigation lands | What was captured, from where, and where it now lives | The mitigation destroys the only environment where the bug exists. What you did not name in the next three minutes is gone |
| **Comms cadence** | Set at declaration, honoured until close | A stated interval, and each update carrying: what is known, what you did, when the next one comes | The single thing no unaided responder produces. Alone, the communicate role loses to the acting role every time, and it loses silently |
| **Handback note** | When users are safe | Impact window, the mitigation and whether it is temporary, what is now an ordinary defect, who owns it | "Users are safe" and "we know why" are different moments; without this the incident stays formally open while everyone treats it as over |

Two of these are worth their own sentence, because they are the two that are
almost never written and the two that cost the most.

**The cadence is an interval, not an intention.** Fifteen or thirty minutes,
stated at declaration, honoured *even when nothing has changed* — "no change
yet, still on the rollback, next update at :45" is information and silence is
not. An intention to keep people posted is what every responder already has;
it is not a cadence, and it is what gets dropped the moment something breaks.

**The evidence manifest is a list of names, not a feeling of having looked.**
Capture the failing *and* the succeeding requests in the same window — with a
partial failure rate, the contrast between them is the whole diagnosis and it
exists only while both are happening. Then the build or artifact id, the
resolved dependency set, the runtime config and flag state, and the metric
window. Record queue depths and half-completed work; **record, never clean up**.

## Declare it, out loud, early

The most expensive minutes of most incidents are the ones before anyone said
the word. A single engineer quietly investigating for twenty-five minutes is an
incident with no clock, no record, and no one else helping.

Declaring costs almost nothing and buys the rest of this skill. **When unsure
whether this is an incident, it is one** — over-declaring is a small,
recoverable embarrassment, and under-declaring is the thing every postmortem in
this category finds.

## The order

| # | Beat | Goal | Produces | Gate to the next |
|---|---|---|---|---|
| 1 | **Declare** | It is named, timestamped, and someone owns it | Declaration, timeline opened, cadence set | An owner exists and the timeline has its first line |
| 2 | **Assess** | Blast radius: who is affected, how badly, and is it growing | Impact statement | Impact is stated in user terms, not system terms |
| 3 | **Preserve** | The evidence your mitigation is about to destroy is captured | Evidence manifest | The named items are somewhere the mitigation cannot reach |
| 4 | **Mitigate** | The harm stops, by any reversible means | Timeline entries, one per action | Impact measurably stops growing — observed, not assumed |
| 5 | **Stabilize** | The mitigation holds without a hand on it | — | It survives unattended, and someone has said so |
| 6 | **Hand back** | The incident becomes an ordinary diagnosis | Handback note | Users are safe; `diagnosing-before-fixing` takes over from here |
| 7 | **Write it up** | The organization learns something | Postmortem, from the timeline | `writing-postmortems` |

Beats 3 and 4 are the two most often collapsed, in that order. A rollback that
also deletes the only copy of the failing artifact has stopped the bleeding and
guaranteed the same incident next quarter.

## Mitigations are ranked by reversibility, not by correctness

In beat 4 you want the fastest action you can undo, not the best action you can
justify. `deciding-reversibility` is the whole calculus, applied under a clock.

Roughly in order of preference: flip a flag off; roll back the deploy; shift
traffic away; scale, throttle, or shed load; disable the specific feature;
break the specific dependency. Only after those: a forward fix.

**Fixing forward is a mitigation of last resort** — reserve it for when
rollback is impossible or would itself cause harm, which is rarer than it feels
at the time. It is a change written under pressure, unreviewed, going straight
to production, by someone who does not yet understand the failure. Everything
that makes a change safe is absent.

Two more rules that pay for themselves every time:

- **One change at a time, and observe between them.** Three simultaneous
  mitigations mean nobody can say which one worked or what the other two did.
- **Write down every action as you take it**, with a timestamp. The timeline is
  the postmortem's entire raw material, and it cannot be reconstructed later —
  memory of an incident is compressed and wrong in specific, predictable ways.

## Three roles, even when you are alone

An incident runs three jobs concurrently. With one responder they are all
yours, and the third one silently loses every time.

- **Decide** — hold the state of the world, choose the next action, own the call.
- **Do** — run commands, read output, make the changes.
- **Communicate** — tell people what is known, what is being done, and when
  they will next hear from you.

Alone, the trick is not doing them well simultaneously. It is **interleaving on
the cadence you already set at declaration**: act, then update, then act. That
artifact exists precisely so this role has something to fail against — an
interval you can be late for, rather than an intention you can quietly hold.

The instant a second responder is available, split Decide from Do. The person
typing cannot also be holding the state of the world.

## What not to do while the clock is running

- Do not investigate root cause before the harm has stopped. It is the single
  most natural mistake here.
- Do not run exploratory queries or debuggers against production without
  `observing-production-safely` — an investigation that becomes a second
  incident is a category of its own.
- Do not clean up. Deleting the bad rows, rotating the noisy log, restarting
  the interesting process — all destroy evidence beat 3 was meant to save.
- Do not silence the alert to reduce noise unless you have first recorded what
  it was firing on.
- Do not skip the write-up because the fix was obvious in hindsight. Obvious in
  hindsight is a property of hindsight.

## The handback

An incident ends when users are safe, not when the cause is known. Those are
two different moments and conflating them keeps an incident formally open for
days while everyone treats it as over — which is how the follow-up fix never
lands.

Close it explicitly: impact stopped at a stated time, the mitigation in place
and whether it is temporary, what is now an ordinary defect, and who owns it.
The remaining work is a `diagnosing-before-fixing` run with a
`regression-test-from-bug` gate at its end — and the regression test is what
turns "we rolled back" into "this cannot happen again."

## Common mistakes

| Symptom | Real cause |
|---|---|
| Twenty-five minutes of investigation before anyone knew there was an incident | Nobody declared; the bar for declaring was set at certainty |
| Service restored, cause unknowable | Mitigation ran before evidence was preserved |
| Nobody can say which of three actions fixed it | Changes batched under pressure, with no observation between them |
| The forward fix caused the second outage | Fix-forward chosen because rollback felt like giving up |
| Stakeholders escalated while the fix was already working | The communicate role was dropped, which is what happens to it by default |
| The postmortem timeline is vague and partly wrong | Reconstructed afterwards from memory instead of written down as it happened |
| Everyone agrees what happened and no two accounts match | No timeline artifact; six people reconstructing from compressed memory |
| The team kept people posted and stakeholders still escalated | An intention to communicate rather than a stated interval with a next-update time on every message |
| The evidence was "captured" and the diagnosis still stalled | No manifest — the failing requests were saved and the succeeding ones in the same window were not |
| The same incident, one quarter later | Closed at "users are safe" and never handed back to a diagnosis |
| The alert was disabled and never re-enabled | Noise reduction treated as a mitigation |

## Red flags

- "Let me just understand this first" — with users still broken.
- "I don't want to roll back without knowing why it broke."
- "I'll update everyone once I know something."
- "It's probably fine now" — with nothing observed that says so.
- "Let me clear out the bad records so it stops erroring."
- "We know what it was, we don't need a write-up."
- "I'll keep everyone posted" — an intention, offered where an interval belongs.
- "I've got the timeline in my head."
- "I'll just try a few things and see."
