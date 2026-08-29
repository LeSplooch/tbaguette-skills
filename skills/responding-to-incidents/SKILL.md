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

The second thing that goes wrong is quieter. An incident has three jobs going
at once, and when there is one responder, the one that gets dropped is always
the same one: telling people what is happening.

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

## Declare it, out loud, early

The most expensive minutes of most incidents are the ones before anyone said
the word. A single engineer quietly investigating for twenty-five minutes is an
incident with no clock, no record, and no one else helping.

Declaring costs almost nothing and buys the rest of this skill. **When unsure
whether this is an incident, it is one** — over-declaring is a small,
recoverable embarrassment, and under-declaring is the thing every postmortem in
this category finds.

## The order

| # | Beat | Goal | Gate to the next |
|---|---|---|---|
| 1 | **Declare** | It is named, timestamped, and someone owns it | An owner exists and a record has been started |
| 2 | **Assess** | Blast radius: who is affected, how badly, and is it growing | Impact is stated in user terms, not system terms |
| 3 | **Preserve** | The evidence your mitigation is about to destroy is captured | Logs, the bad artifact, and the current state are saved somewhere the mitigation cannot reach |
| 4 | **Mitigate** | The harm stops, by any reversible means | Impact measurably stops growing — observed, not assumed |
| 5 | **Stabilize** | The mitigation holds without a hand on it | It survives unattended, and someone has said so |
| 6 | **Hand back** | The incident becomes an ordinary diagnosis | Users are safe; `diagnosing-before-fixing` takes over from here |
| 7 | **Write it up** | The organization learns something | `writing-postmortems` |

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
a cadence**: act, then update, then act. Set a real interval — every fifteen or
thirty minutes — and honor it even when nothing has changed, because "no change
yet, still on X, next update at :45" is information and silence is not.
Communicating stops being optional the moment anyone outside the response is
waiting.

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
| The same incident, one quarter later | Closed at "users are safe" and never handed back to a diagnosis |
| The alert was disabled and never re-enabled | Noise reduction treated as a mitigation |

## Red flags

- "Let me just understand this first" — with users still broken.
- "I don't want to roll back without knowing why it broke."
- "I'll update everyone once I know something."
- "It's probably fine now" — with nothing observed that says so.
- "Let me clear out the bad records so it stops erroring."
- "We know what it was, we don't need a write-up."
- "I'll just try a few things and see."
