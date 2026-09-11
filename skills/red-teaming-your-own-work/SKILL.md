---
name: red-teaming-your-own-work
description: Use when a change, plan, design, recommendation, or answer is finished and has been examined only by whoever produced it, when nothing looks wrong and nothing has been tried to make it look wrong, before handing off work someone will act on, or after a previous delivery came back with a defect a single skeptical pass would have caught. Also use when a measurement has just confirmed a result you are about to act on, when a search or benchmark came back with nothing and that null is about to be reported as nothing being there, when a draft cites a document, spec, or API from memory rather than a reopened copy, or when an artifact built for the author and a few testers is about to be published to end users. Covers adversarial self-review, attack checklists, re-testing a confirmed number for its shape, reopening every cross-reference, auditing what a shipped artifact does on a stranger’s machine, bounding a null by the smallest effect the search could have seen, and bounded review passes.
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

Run all seven. Each costs a minute or two and lands somewhere predictable.

| Attack | The question | Where it usually lands |
|---|---|---|
| Wrong assumption | What did I take as given without checking? | The shape of an input, the behaviour of an external call, what one word in the request meant |
| Unhandled input | Empty, zero, one, maximum, absent, duplicate, out of order, concurrent, hostile, wrong type | Boundaries between components, not inside them |
| Misread requirement | Reread the original request verbatim, not your restatement of it | A qualifier in the request that never made it into the work |
| Silent failure | Where can this fail with nothing raised, logged, or returned? | Swallowed errors, defaults substituting for missing data, retries that give up quietly |
| Wrong altitude | Too specific — one case hardcoded; or too general — extension points with one caller | Abstraction and configuration added on speculation |
| Adjacent breakage | Who else calls this, parses this shape, relies on this ordering or this timing? | Callers you never opened, and anything depending on the old behaviour |
| Unverified reference | Which statements here are about a document I did not reopen? | Cross-references, cited requirements, "as X already says", recalled API or spec behaviour |

An eighth when the work is an addition: delete it mentally and ask what breaks. If nothing does, it is unjustified rather than wrong.

## Attack the strongest part

The weak part is already flagged in your own head — you know the rough edge and the TODO, and reviewing it returns what you already knew. Defects concentrate where attention stopped early: the obviously-correct core, the part you would skip in someone else's review, the piece written fastest because it was familiar.

Operational rule: name the two pieces you would not bother reviewing, and review exactly those. The confidence that makes them skippable is what left them unexamined.

## The artifact you ship has a different audience than the one you built

A build is written for the audience it has while it is being built: the author, a
handful of testers, machines the author controls, a few days of use. Plenty of
behaviour is correct for that audience and hostile for the next one. A timer that
disables the binary after three days keeps a test build from lingering; the same
timer bricks the product a paying stranger installed. A counter that hides its
state in system-looking filenames so a reinstall cannot reset it is a reasonable
trial-enforcement hack and an antivirus flag the moment it reaches a machine the
author does not own. Verbose logging, a hardcoded dev endpoint, a test backdoor, a
phone-home that was fine on the author's network — each is a decision that was
right for the development audience and never re-examined when the audience changed.

The reason a normal review misses all of these is that publication does not feel
like a change to the thing under review. The code is the code; shipping is
"packaging". So the audit that catches them is not a code review — it is a
different question asked of the finished artifact: **what does this do on a
stranger's machine?** Not "does it work" but what it writes and where, when it
stops working, what it sends and to whom, what it leaves behind that a reinstall
will not clear. Every answer that assumes a machine the author controls is a
defect the moment the artifact leaves.

The tell that you are standing on one is a mechanism whose user-facing message is
addressed to the developer — *recompile to continue*, *dev build*, *trial
expired* — surfacing to someone who cannot act on it. The remedy is the same in
every case and it is an ordering, not a deletion: gate the development-audience
behaviour behind an off-by-default build flag, so the shippable build is the
default and the dev-only behaviour is the thing you opt into, never the reverse.
A kill switch that ships enabled because disabling it was one more step is the
failure this catches.

## A reference to another document is a claim, and it reads as a verified one

Three attacks in the table above already look outside the artifact — an external call's behaviour, the original request, the callers you never opened. A cross-reference is the fourth, and the one that disguises itself as already checked: it asserts something about a document that is not in front of the reader, and its form does the opposite of flagging that. A backticked name, a section title, a file path, a spec clause — each looks like a *link*, an invitation to go and check — while it is functioning as an *assertion* the reader will act on without going anywhere. Readers do not open them, and that is not laziness. Not having to open it is what a citation is for.

The failure survives review because of its shape. A reference recalled rather than reopened is usually *directionally* right — the cited thing really is about this topic — and wrong in exactly the part that was load-bearing: it inverts the cited example, attributes a test the source does not apply, or names the source's mechanism a little off. And the correction is the more dangerous half. Rewriting a vague citation into a precise one — naming the rule the other document supposedly applies — *feels* like the rigorous version and reads that way to a reviewer, while having introduced a second false claim about a file that still nobody has opened. Naming a mechanism is routinely mistaken, by its author and by its reader, for having checked it.

The check is mechanical and costs about a minute. Before shipping, pull every reference to something outside the draft — grep your own text for the backticked names, paths, and section titles — and open each target. Not to confirm the topic; to confirm the sentence you wrote about it. Whatever you cannot reopen gets softened to what you actually know or deleted outright: "as noted elsewhere" is honest, and a wrong attribution is not.

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
| A citation is wrong in a way that survived two reviews | Nobody opened the cited file, because a reference reads as something already checked |
| A vague reference was sharpened into a precise one that is false | Naming the mechanism felt like verifying it, and the reviewer read it the same way |
| A shipped build bricks itself, phones home, or trips antivirus on a user machine | A development-audience behaviour was never re-examined when publication changed the audience |

## Red flags

- "It is simple enough to be obviously correct"
- "I wrote it carefully"
- "Testing will catch it" about something you could provoke with one command
- Ending the pass pleased rather than surprised
- Reviewing what you just wrote, in the order you wrote it, immediately after writing it
- A cited document nobody in the review opened, including you
- "It definitely says that" about a file you have not had open during this piece of work
- "Shipping is just packaging" — treating publication as a step that changes nothing about the artifact under review
