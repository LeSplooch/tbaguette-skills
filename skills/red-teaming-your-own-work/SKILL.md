---
name: red-teaming-your-own-work
description: Use when a change, plan, design, recommendation, or answer is finished and has been examined only by whoever produced it, when nothing looks wrong and nothing has been tried to make it look wrong, before handing off work someone will act on, or after a previous delivery came back with a defect a single skeptical pass would have caught. Also use when a measurement has just confirmed a result you are about to act on, or when a search, sweep, benchmark, or experiment came back with nothing and that null is about to be reported as nothing being there. Covers adversarial self-review, attack checklists, re-testing a confirmed number for the shape rather than the point, bounding a null by the smallest effect the search could have seen, and bounded review passes.
---

# Red-teaming your own work

## Overview

Rereading work confirms it; attacking it finds defects. Spend one bounded pass before delivery assuming the work is wrong and hunting for where — the productive question is not whether a defect exists but where it is hiding.

## When to use

- A change, plan, recommendation, or answer is finished and only its author has seen it
- Nothing appears wrong, and nothing has been attempted to make it appear wrong
- Before handing off a change set, a design, or a decision someone will act on
- A previous delivery came back with a defect one skeptical pass would have caught
- Not for: confirming a claim has evidence behind it (`confirming-before-claiming-done`), or diagnosing a failure you already know about (`diagnosing-before-fixing`). This is for work that currently looks correct.

## The stance

"Is this right?" is answered by the same reasoning that produced the work, and it returns yes. "If this is wrong, where is it wrong?" forces the generation of candidate locations, which is a different task with a different output. Commit to finding at least one real defect; a pass that ends with none, and with no surprise about that, was a reread.

## The attack list

Run all six. Each costs a minute or two and lands somewhere predictable.

| Attack | The question | Where it usually lands |
|---|---|---|
| Wrong assumption | What did I take as given without checking? | The shape of an input, the behaviour of an external call, what one word in the request meant |
| Unhandled input | Empty, zero, one, maximum, absent, duplicate, out of order, concurrent, hostile, wrong type | Boundaries between components, not inside them |
| Misread requirement | Reread the original request verbatim, not your restatement of it | A qualifier in the request that never made it into the work |
| Silent failure | Where can this fail with nothing raised, logged, or returned? | Swallowed errors, defaults substituting for missing data, retries that give up quietly |
| Wrong altitude | Too specific — one case hardcoded; or too general — extension points with one caller | Abstraction and configuration added on speculation |
| Adjacent breakage | Who else calls this, parses this shape, relies on this ordering or this timing? | Callers you never opened, and anything depending on the old behaviour |

A seventh when the work is an addition: delete it mentally and ask what breaks. If nothing does, it is unjustified rather than wrong.

## Attack the strongest part

The weak part is already flagged in your own head — you know the rough edge and the TODO, and reviewing it returns what you already knew. Defects concentrate where attention stopped early: the obviously-correct core, the part you would skip in someone else's review, the piece written fastest because it was familiar.

Operational rule: name the two pieces you would not bother reviewing, and review exactly those. The confidence that makes them skippable is what left them unexamined.

## A confirmed measurement is one point, not a shape

The strongest part of a quantitative result is the confirmation itself, and it
deserves the same treatment as everything else above. A held-out check answers
whether *one point* agrees. It cannot tell a real effect from a noisy peak that a
lucky point happened to sit near, and this survives every conventional safeguard
because each of those addresses a different threat: pairing removes shared
variance, a stated noise floor removes differences too small to matter, held-out
samples remove fitting to the data you chose on. None of the three tests whether
the *shape* reproduces.

That is not a hypothetical failure. A comparison careful on all three counts can
still produce a result at four or five standard errors that falls to
indistinguishable once an unrelated instrument bug is fixed, an adopted change
whose advantage quietly decays from six standard errors to under one and a half,
and a fresh proposal at over two that does not reproduce at all on the next
untouched sample.

The attack is cheap: before acting on a confirmed result, re-run the whole sweep
— the curve, not the chosen point — on one further sample nothing has touched,
and ask whether the shape holds rather than whether the point confirms again. Two
readings decide it. A peak whose *neighbours* do not also beat the incumbent is a
spike, and a spike is noise. And if the incumbent turns out to be the best point
on the fresh curve, the original selection was noise from the start.

Do this because it discriminates, not because it is conservative. Run on two
consecutive results it will happily give opposite answers — killing a proposal
whose peak does not reappear, and vindicating an adopted change whose gap holds
on fresh data. A check that only ever says no is not a check, it is a veto, and
it earns nothing.

## A null result is bounded by what the search could have seen

The section above attacks a result that confirmed. The opposite outcome gets no
scrutiny at all, because a search that found nothing feels like it has nothing to
be wrong about — and that is the error. A null does not say no improvement
exists. It says the search could not tell an improvement from noise at the sample
size it ran, and until somebody measures the noise, "no effect" and "no effect
larger than X" are indistinguishable claims of which only one is true.

Both get written down as the same sentence. A ledger accumulating "no improvement
found" across eight sweeps reads as eight independent failures to find anything,
and is one unmeasured instrument reported eight times.

The measurement is cheap and it is one number: the **minimum detectable effect**
at the sample size actually used. Where the comparison is paired, take it from
the standard deviation of the per-sample *differences* and not from the spread of
the raw results; the two diverge by a lot exactly when pairing is doing its job,
and using the second is how an adequately powered search gets mistaken for a
hopeless one. Where the noise is larger than the effect being hunted, the
arithmetic is brutal — a search whose baseline scored 0.675 with a standard
deviation of 0.321, and whose paired differences had a standard deviation of
0.770, could only ever have seen improvements above about
26% of baseline at the twenty samples every one of its sweeps had used. Detecting
10% would have taken 131 samples; 5% would have taken 522.

What that buys is not the retraction of one result. It re-scopes every null the
search ever produced, downward and at once: each "as good as anything the search
can find" becomes "as good as anything the search could have seen," which is a
far weaker claim and the only one that was ever supported. Run it before the
first sweep rather than after the eighth, because it also decides whether the
sweep is worth running at all — a search underpowered for the smallest effect
worth acting on returns nulls whatever is true, and every hour it spends is spent
proving nothing.

The reading that survives is the useful one, so state it that way. A null at a
known detectable effect genuinely rules out the large effects, and ruling out a
large effect is a real finding. It is the unbounded null — the one quietly
claiming to have ruled out everything — that was never evidence.

## Distance, obtained cheaply

Reviewing immediately after writing re-executes the same reasoning path and reproduces the same blind spot; you read what you meant rather than what you wrote. A break works because the memory of intent decays faster than the artifact does. When no break is available, substitute:

- Read the diff, not the file. It strips your intent and leaves the change.
- Read as the caller or the operator, never as the author. State what it does from the artifact alone, then compare against what it was supposed to do.
- Read the pieces in reverse order, last to first.
- Discard the intermediate reasoning entirely and reload only the original request and the final artifact.

The comparison that finds defects is written-behaviour against requested-behaviour, with intended-behaviour excluded from the room.

## Bounded, then stop

One pass. Collect every finding before fixing any of them — fixing mid-pass truncates the pass, because attention moves to the repair. Then rank and act:

| Finding | Action |
|---|---|
| Wrong result on an input the system will actually receive | Fix before delivering |
| Correct, but fails on input nothing can produce | Note in the report, do not fix |
| Disagreement with the request's own design | Raise it, do not unilaterally change it |
| Naming, formatting, structure | Fix silently or leave; do not report |

Fixes written during the pass are unreviewed new work carrying the same defect rate as the original — re-attack the fix specifically, not the whole artifact again. A second full pass is warranted only when the first found a defect of a *kind* you had not considered, which means the pass was aimed wrong rather than that a third is owed. A second pass yielding only nits is the signal to stop.

A pass that comes back clean, on work about to ship, is not evidence there is nothing left — it is precisely the moment `karen-and-the-manager` exists for. This skill finds behavioral defects; that one runs after, adversarially, for everything a correctness-focused attack list does not aim at.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The pass finds only cosmetic issues | Asked whether it is right instead of where it is wrong |
| Confidence is higher after the review than before it | The pass confirmed intent rather than testing behaviour |
| Found a defect, fixed it, shipped without re-attacking the fix | Fixes treated as corrections rather than as new untested code |
| The review keeps expanding into a rewrite | No budget; adversarial mode has no natural stopping point |
| "Edge cases are handled" with no enumeration | Handled the cases you thought of, which is exactly the set that produced the defect |
| The part reviewed is the part you already doubted | Attacked the known weak point; the strong point stayed undefended |
| A large change reviews clean in one sitting | Reviewed the artifact you remember writing, not the one that exists |

## Red flags

- "It is simple enough to be obviously correct"
- "I wrote it carefully"
- "Testing will catch it" about something you could provoke with one command
- Ending the pass pleased rather than surprised
- Reviewing what you just wrote, in the order you wrote it, immediately after writing it
