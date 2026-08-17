---
name: confirming-before-claiming-done
description: Use when about to say a fix, a feature, or a test suite is done, fixed, or passing; when a change is about to be committed, pushed, or handed off on the strength of that claim; when a subagent's or tool's own success report is about to be repeated as fact; or when the only thing behind the claim is that the code looks right and nothing has actually been run. Covers naming the command that would prove the claim and running it fresh, treating a stale or partial run as current evidence, and the gap between believing something works and having confirmed it.
---

# Verification before completion

## Overview

"It should work now" and "it works" are different claims. The first is free — it falls out of having made the edit and having a plausible story for why the edit fixes things. The second costs one command: run the actual check, read what it actually printed, and only then use the word "done." Being careful, understanding the bug deeply, or having been right the last ten times does not substitute for that command. It only means the claim hasn't been checked yet.

The rule survives paraphrase. "Should be passing," "looks good," "that should do it," and "I'm fairly confident this is fixed" are the same unverified claim in different clothes. If the check hasn't run since the last change, none of them are earned.

## When to use

- About to say a fix, a feature, or a test suite is done, fixed, working, or passing.
- About to commit, push, open a PR, or hand work off on the strength of that claim.
- Reporting what a subagent, a CI run, or a tool said about itself, instead of what you independently checked.
- Moving on to the next task because this one feels finished.
- The only thing behind the claim is that the diff looks right and nothing has actually been executed.
- Not for: judging whether a past decision was actually correct in hindsight (see `revalidating-decisions`).
- `calibrating-confidence` is the adjacent concern: marking your own uncertainty honestly as verified, inferred, or assumed. This skill is the concrete act that earns the verified label in the first place — running the check before the claim.

## Name the check, then run it

Before typing the claim, name the exact command whose output would prove it. No candidate command means the claim isn't checkable yet — say that, instead of skipping straight to the confident version.

Run it in full: not the fast subset, not just the file that changed, the whole thing the claim is actually about. Read the whole output, not just the line that flatters the claim — a summary can say "34 passed" while the same output shows a separate failure the summary doesn't count. Then check what the output actually says against what the sentence is about to claim: a build that compiles is not a test suite that passes, and a linter with zero complaints hasn't touched runtime behavior at all.

Only once that comparison holds does the claim get made — and it gets made with the evidence next to it, not implied. "34/34 tests pass" carries different information than "tests pass."

When the check itself is unreliable — an intermittent bug that only reproduces some fraction of the time — a single clean run doesn't carry the same weight it would for a deterministic one. That's a reason to run it enough times to get real signal, or to report status honestly as still-in-progress, not a reason to fall back to a hedged claim instead. "Should be fixed, let me know if you still see it" spends the same unearned confidence a flat "it's fixed" would; softer wording doesn't make one weak attempt add up to evidence.

## The claim and what actually proves it

| Claim | Real evidence | Doesn't count |
|---|---|---|
| Tests pass | This session's run, fresh, zero failures | A run from before the last edit; "should still pass" |
| Lint is clean | This session's run, zero warnings | Spot-checking only the file you just touched |
| Build succeeds | Fresh build, exit code checked | Lint passing; no red squiggles in the editor |
| A bug is fixed | Reproduced the original symptom on the new code, and it's gone | The diff looks like the right fix |
| A regression test guards it | Red on the old code, green on the fix, both watched | Passes once, never run against the broken version |
| A subagent finished the task | The diff it actually produced, read | Its own summary of what it did |
| Requirements are met | Checked line by line against the spec | The tests pass, so it must be done |

## Evidence goes stale the instant code moves

A test run is a claim about the exact code that existed the moment it ran. Change one more line afterward — even a line that "shouldn't touch" the part under test — and the run now describes a version of the code that no longer exists. "It passed ten minutes ago" and "it passes" stop being the same sentence the instant anything lands in between.

This is what makes "I fixed it" and "I confirmed the fix" different acts, not just different phrasings. Declaring something fixed the moment the edit is typed, without re-running the check against the post-edit code, fails for the identical reason a stale test run fails: the evidence on offer was gathered before the thing it's supposed to prove even existed.

## A report is not a check

A subagent reporting success, a CI badge sitting green, a teammate saying it should be fine — none of these are verification, they're claims, and repeating one as your own confirmed status launders someone else's unchecked belief into something that sounds checked. Read the diff the subagent actually produced instead of its summary of the diff. Open the CI log instead of trusting the badge. Run the command yourself instead of describing having run it. The report may well be accurate — that's a separate fact from whether it's been checked.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Should pass now" in a commit message or handoff note | The verification step was replaced with confidence in the fix |
| Tests declared passing based on a run from before the last edit | Evidence treated as durable when it's only valid for the code it ran against |
| "The agent said it completed the task" reported as the task being complete | A tool's self-report repeated as independently checked fact |
| A regression test added and trusted without ever seeing it fail | Never run against the broken code, so it's unknown whether it tests anything |
| Build green, shipped, runtime error in the first minute | Compilation was checked; behavior never was |
| "Looks right" standing in for "ran and confirmed" | Review of your own diff mistaken for verification of its behavior |
| One failing test out of many waved off as unrelated | A partial pass rate treated as a pass |

## Red flags

- "Should," "probably," "looks like it," or "I'm fairly confident" appearing anywhere near a completion claim.
- Satisfaction expressed — "great," "done," "that's it" — before the check has run in this message.
- About to commit, push, or open a PR, and the last thing that ran was the edit, not a check.
- A claim that would change if reworded, because the wording was doing the work the evidence should be doing.
- "It's probably fine, I'm confident in the fix" as a reason to skip the command that would confirm it.
- Tired, near the end of a long task, and tempted to call it done to stop working rather than because it's verified.
