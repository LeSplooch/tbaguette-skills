---
name: karen-and-the-manager
description: Use when work is about to be called done, shipped, or handed off and has only ever been reviewed by whoever built it; when a red-team pass or a code review just came back clean and that itself feels suspicious; when polish keeps getting waved through as good enough; or when the strongest remaining tool for catching defects is a reviewer who refuses to be satisfied. Pairs with knowing-when-to-stop, invoked right after, to bound the result.
---

# Karen and the manager

## Overview

A persona-forced adversarial pass for the moment a normal review already came back
clean. Two escalating roles: Karen, who is never satisfied and complains about
everything down to the smallest detail, and the manager she demands to see, who
separately decides which of her complaints are worth acting on. The persona is not
decoration — exaggerating the critic's standards past what a polite, calibrated pass
permits is what surfaces the defects that pass waves through.

## When to use

- Work is about to be marked done, shipped, or handed off.
- `red-teaming-your-own-work` or a code review just came back clean — that is exactly
  when complacency is highest, not evidence there is nothing left to find.
- Not for mid-implementation debugging (`systematic-debugging` owns that) or first-pass
  design critique (`reviewing-code-deeply`, or formidable's `critique`/`audit`, own
  that). This runs last, against a result that already looks finished.
- Pairs with `knowing-when-to-stop`, invoked immediately after. Karen has no stopping
  instinct by design — that is supplied by the manager's triage and by the paired skill,
  not by Karen holding back.

## Karen's pass

Go through the finished result as the most dissatisfied stakeholder who will ever see
it. Nothing is beneath comment: spacing off by a hair, a label that is technically
correct but not the clearest word available, an edge case that "works" but looks
improvised, one place that is inconsistent with everywhere else, a default that was
merely acceptable rather than actually decided. Every complaint names a real, specific
location — not "this could be better," but *"the confirm button here says 'OK.' Three
other confirm actions on this same result say the actual consequence. Now I don't trust
that anything here was decided on purpose instead of typed first and left."* **Fix:**
rename to match the other three, then check whether any of those three are the odd one
out instead.

No complaint is too small to write down, and the pass is failed if it produces fewer
than ten. Immediately under each complaint goes its concrete fix — Karen is not useful
if she stays a feeling.

## The manager's pass

"Let me speak to the manager" is the second half, not a punchline. Two jobs:

1. **Completeness check** — read Karen's list and ask what she missed. A short list is
   the tell that the first pass wasn't harsh enough; go back through with fresh eyes
   before trusting it.
2. **Triage** — not every complaint survives contact with a manager. For each: fix it,
   or state plainly why it is actually fine (a real reason, not "close enough") and
   leave it. A manager who overturns nothing is rubber-stamping the complaint list; one
   who fixes everything is letting the customer run the business. Both failure modes are
   worth naming out loud in the write-up, not just avoided silently.

## Output shape

- Karen's numbered grievances, each with its concrete fix.
- The manager's verdict on each: fixed, or declined with a stated reason.
- A final count — raised / fixed / declined.
- Handoff to `knowing-when-to-stop` with that count as its input. Deciding how many of
  the declined or fixed items earn another pass happens there, not inside Karen's pass —
  she does not get a vote on when to stop.

## Playing it out loud

In an interactive session with a person actually reading along, stage the arrival
instead of announcing it. Karen does not knock. The interruption lands mid-sentence, in
the middle of whatever calm work was being narrated, as a genuine surprise rather than a
labeled transition — one entrance per invocation, not a running bit repeated per
complaint. She is loud, she is never on your side, and she is carrying a purse she is
fully willing to use. For example: *"Wait… I can hear Karen comi— OOOW!! NOT THE PURSE
AGAIN!!!" (ducks, keeps working) "...okay. Okay. She has a point about the third one."*

The theater is a delivery choice layered on top of the pass, never a substitute for it —
every grievance underneath still needs the real, specific complaint and the real,
concrete fix from Karen's pass above. Skip this entirely for a non-interactive run, a
headless context, or any output no one is actually reading live — including a subagent's
own report, which a human only reads later, secondhand, after the run has already ended:
the substance travels without the performance, and an automated pipeline gains nothing
from a purse joke it cannot see.

## Common mistakes

| Symptom | Real cause |
|---|---|
| List comes back with three vague items | Karen was played politely instead of unsatisfied; rerun assuming the reviewer dislikes the result on principle |
| Every complaint gets fixed, including ones that contradict each other | The manager's triage step was skipped; some complaints are supposed to be declined |
| Complaints just restate what the last review already found | Karen didn't go past that pass's altitude — she owes the details a normal review is too coarse to catch |
| The pass runs mid-implementation, derailing unfinished work | Wrong tool for that moment — this is a finishing pass, not a design or debugging one |
| Fixing spirals into a second and third Karen pass on the same work | `knowing-when-to-stop` wasn't invoked right after; the pairing exists specifically to close this loop |

## Red flags

- "It's basically fine" appearing anywhere in Karen's own voice — she does not say that,
  ever; that verdict belongs to the manager, if it's earned.
- A grievance with no concrete fix attached.
- Stopping after the first satisfying-sounding complaint instead of continuing to the
  finest detail.
- Skipping the manager's triage and fixing the whole list top to bottom without judgment.
