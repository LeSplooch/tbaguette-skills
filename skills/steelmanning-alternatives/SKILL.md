---
name: steelmanning-alternatives
description: Use when an approach was adopted because it was the first one that worked and no other was seriously examined, when a plan or design presents a single option or several that share one shape, when an alternative was dismissed in a single clause, when asked whether there is a better way, or when the chosen approach has started accumulating special cases. Covers first-idea lock-in, option generation, and recommending rather than surveying.
---

# Steelmanning alternatives

## Overview

The first workable idea gets adopted, and everything after it becomes that idea's defense. A second option earns its place only by differing in mechanism; a variant of the first is decoration on a decision already made.

## When to use

- An approach was chosen because it was the first that worked, and nothing else was examined
- A plan or design presents one option, or presents three that share a shape
- An alternative was dismissed in a single clause
- The chosen approach is fighting the problem — special cases are accumulating
- Not for: open-ended exploration of what to build alongside the user, which is `scoping-before-building`. This applies once a direction has been taken, explicitly or by default.

## The lock-in mechanic

Once an approach exists, reasoning shifts from generation to evaluation, and evaluation is biased toward the thing being evaluated. The reliable tell: the alternatives you listed are ones you could not argue *for*. If you cannot state the case for an option in terms its advocate would accept, you did not consider it — you staged a comparison to license the first idea.

## What would have to be true

The highest-value move here. Do not argue against the dismissed option; ask what would have to be true for it to be the right choice, then check whether those things are true. This converts an unfalsifiable taste judgment into a factual question that usually resolves in minutes.

Dismissed: handle it in the data layer rather than in application code. What would have to be true — that volume makes round-trips dominant, and that the logic is expressible there. Both are checkable now. Compare with "business logic does not belong in the data layer", which is checkable never and therefore is not a reason.

Apply the same test to the option you chose. What would have to be true for it to be right, and is it?

## Generating a second option that is actually different

If option B shares A's core mechanism and differs only in parameters, it is A-prime; keep going. Generators that produce difference rather than adjacency:

| Generator | Prompt |
|---|---|
| Invert the direction | Push instead of pull; the caller does it instead of the callee; compute on write instead of on read |
| Change the layer | Solve it in the data model, the build, the configuration, the operational setup, or the product definition instead of in the code |
| Do nothing | If this is simply not handled — what breaks, for whom, how often |
| Buy or build, both ways | An existing tool or platform feature, against thirty lines of your own with no new dependency |
| Relax a constraint | Which constraint, if lifted, makes the problem trivial — and who owns that constraint |
| Do it manually | At this frequency, is automation cheaper than a person doing it |
| The blunt version | No abstraction, no generality, all of it in one place. Wins on total cost more often than it gets proposed |

Two serious options plus "do nothing" is usually the right number. A third is worth generating when the first two share an assumption — that shared assumption is where the real alternative lives. Past three you are generating rather than deciding.

## Dismissals that sound like knowledge

The dangerous ones arrive with a reason attached: that is slow, that will not scale, that library is unmaintained, that is not how it is done here, we cannot because of X. Each is a factual claim with a truth value.

- A dismissal verifiable in under five minutes that has not been verified is a preference, not a reason.
- "We do not do that here" is evidence about the past. The original reason may have expired, may have concerned a system that no longer exists, or may never have been written down because it never existed.
- Inherited constraints deserve one question each: who owns it, and when was it last true.
- Performance dismissals with no number attached are the most common kind and are wrong in both directions at roughly equal rates.

## Recommend, do not survey

The deliverable is a recommendation with the runner-up named, not a matrix handed back to whoever asked. A survey without a recommendation moves the work to them and conceals that you have a view.

Form: recommend A, name B as the real alternative, give the one criterion on which they actually differ, and state the observable that would flip it. "Recommend A. B was the genuine alternative; A wins because it adds no dependency. Switch to B if the transform set grows past a handful of cases."

The one exception: the choice turns on something only the user holds — cost tolerance, team skill, a commitment you cannot see. Then present both and name precisely which unknown decides it, rather than presenting both because you declined to decide.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The alternatives all share the first idea's shape | Generated by perturbing option A instead of re-deriving from the problem |
| The comparison table's winner was obvious before the table existed | Table built to justify a decision already made |
| "That does not scale", with no number | An unverified performance claim standing in for a reason |
| Three options presented, no recommendation | Survey substituted for judgment |
| Rejected for a constraint nobody confirmed | An inherited assumption treated as a fact about the world |
| The alternative is reconsidered only once the chosen path hurts | Options generated at the point of pain rather than the point of choice |
| "Do nothing" never appears in the list | The problem's cost was assumed rather than estimated |

## Red flags

- "There is really only one way to do this"
- Writing the comparison after implementation has started
- "We already discussed this" about a discussion that contained one option
- An alternatives section every entry of which you would argue against
- Dismissing an option with an adjective — ugly, hacky, unclean — where a checkable claim was available
