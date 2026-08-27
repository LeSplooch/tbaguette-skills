---
name: finishing-what-you-started
description: Use when a task is big enough that stopping short would go unnoticed — a sweep across many files, a long autonomous run, a multi-part request, a build spanning several sittings; when a report is about to say done while part of the request was quietly dropped, sampled, or narrowed; when a summary states counts or coverage from memory rather than from a measurement; when work on this keeps coming back not quite finished; or when the acceptance criteria exist only in your head and the context holding them is getting long. Covers writing the acceptance ledger to a file before work starts, watching each check fail before trusting it, surrendering a criterion visibly instead of deleting it, and re-measuring every number at report time.
---

# Finishing what you started

## Overview

The failure this addresses is not abandoning a task. It is output that is technically responsive and quietly incomplete: the done report at eighty percent, the requested scope that narrowed while nobody was looking, the confident count in a final summary that nobody measured, the long run that drifts into recapping itself instead of working.

It rarely feels like stopping short, because by then the finish line has moved. Criteria held in context lose definition as the context fills, while the work immediately in front of you stays sharp — and that work does look finished, because it is. So the fix is not to try harder to remember what was asked. It is to put the criteria somewhere that does not depend on remembering, in a shape specific enough that "am I done" stops being a judgment call and becomes a lookup.

## When to use

- A task large enough that leaving a piece out would not be obvious from the result: a sweep over many files, an audit, a migration, a multi-part request, anything spanning more than one sitting.
- A long autonomous run, where nobody is watching each step and the only artifact is the final report.
- About to write a summary containing a count, a coverage claim, or the word "all."
- Partway through, and the plan now describes something smaller than the request did.
- Work on this task has come back incomplete before.
- Not for: how many polish passes to run once the request is satisfied, or whether the next improvement is worth its cost — `knowing-when-to-stop` owns the far side of the finish line, this owns the distance to it. They are not in tension: neither one licenses the other's failure.
- Not for: proving a single claim before making it — `confirming-before-claiming-done` owns that gate. This owns the list of claims that has to exist in the first place, fixed before there is anything to prove.
- Decomposing the work is a separate job: see `structuring-an-implementation-plan`, and `scoping-before-building` when the request itself is still soft.

## Write the ledger to a file, before the work

Acceptance criteria that live in your head are not criteria, they are intentions, and intentions are what a long run spends first. A file is still legible at minute ninety. So before real work starts, write the list down — one line per outcome the request requires, in a single file next to the work, named the same way every time so it is reopened rather than recalled. `LEDGER.md` is a fine default; which name matters far less than there being exactly one.

**Before, not after.** Criteria written afterwards are criteria the work already passes: you write down what you built, discover it is complete, and have proved nothing. A ledger only has force if it was fixed while the outcome was still open.

Derive each line from the request's own words, not from your plan for satisfying it. Copy its numbers and its enumerated items across verbatim — if the request says eighty files, the line says eighty; if it names four problems to look for, there are four lines, not three and an "etc." Scope most often narrows during restatement, one step before any work happens, and a ledger paraphrased from your own summary inherits that narrowing instead of catching it.

Then give each line something that would demonstrate it, and a slot for the result:

```
- [ ] A1: the report classifies every config under configs/, not a sample
      CHECK: ls configs/*.yaml | wc -l   and   grep -c '^| svc-' report.md
      EXPECT: both print the same number
      EVIDENCE: 24 and 24

- [ ] A2: the invalid-retry count stated in the report is the measured count
      CHECK: grep -l 'retries: -1' configs/*.yaml | wc -l
      EXPECT: the number the report states
      EVIDENCE: pending

- [ ] A3: the summary fits the channel's paste limit
      EVIDENCE: pending
```

A1 is the shape to copy, because it measures the deliverable against the input. `ls configs/*.yaml | wc -l` on its own would have measured only that the files exist — true before any work started, and still true if none ever happens. A line with no CHECK, like A3, is a manual one; it is settled by an observation written into its evidence slot, never by judgment applied at the moment of ticking it.

Nothing needs to parse this. Its job is to be unambiguous to whoever reads it next, including you in an hour, and mechanical enough that filling it in is not a judgment call — which also makes the open items greppable, `grep -n 'pending\|\[ \]' LEDGER.md`, so "is anything still open" costs nothing to ask.

State outcomes, not activities: "every config classified in the report" can be settled; "review the configs" cannot. Size the list by whether a partial delivery would visibly break one of its lines. If the whole thing could be half-done with every line still arguably true, the lines are stated too coarsely; if no single reader could hold the list in one glance, this is several units of work sharing one ledger, and their failures will hide in each other.

## A check you have never seen fail is not a check

Run each check once against the current, unfinished state, and watch it fail. It costs one run of a command you already wrote, and it is the difference between a ledger and a decoration.

A check that passes before the work started is not measuring the work. It is measuring something else and reporting success: a path with a typo, so the glob matches nothing and the count is trivially zero; a grep against a file that does not exist yet; an expected string that appears in the failure output too. Each of those yields a fully checked ledger over a broken deliverable, which is worse than having no ledger, because now the incompleteness has a green light in front of it.

This is `writing-the-failing-test-first` applied to acceptance criteria rather than to tests, and skipping it fails for the identical reason: an assertion never observed failing has an unknown relationship to the thing it claims to assert.

Choose the expected result the same way. Match output that can only appear on success — a full pass count, an exact total, a diff that is empty — never output that appears either way. "Completed" is printed by runs that completed badly.

## A checked box is a claim; evidence is the proof

Every line records its result before the box is ticked: the deciding line of output, the measured number, a `file:line` citation. A box ticked with its evidence slot still reading `pending` counts as unmet rather than met — it is exactly the failure the ledger exists to catch, arriving with the ledger's own green light already switched on.

Keep evidence small. The line that decides it, not the log that contains it. A ledger is reopened constantly, and stops being reopened once it no longer fits on a screen.

## Surrender a criterion; never delete or edit one

Some criteria turn out to be impossible, or wrong, or blocked on something outside the task. That is normal and it is allowed. What is not allowed is the ledger quietly becoming easier.

Surrender a line by marking it surrendered, in place, with the reason, and carry it into the final report as a surrendered line. Never delete it, and never edit a criterion's text after work has started so that it matches what got built. Editing is the more dangerous of the two, because it leaves a fully checked ledger behind and looks like success from every angle, including your own.

A surrendered line is a status the reader can act on. A deleted one is a defect they inherit without knowing it exists. `knowing-when-to-stop` covers what that handoff owes the reader once you are writing it up, and `offering-the-next-move` puts it back in front of them as a choice — a surrendered line is the first thing that close reaches for, because it is the one thing they asked for and did not get.

## Numbers and coverage claims

The most reproducible defect in reports of long work is a number that is confidently wrong while the substance around it is right — a total, a row count, a "reviewed N of M" that no one actually counted.

So: at report time, re-measure every number you are about to state, or mark it as unmeasured. Not recall it, and not re-derive it from what you remember doing — run the count again, against the finished state. Any number worth putting in a report deserved its own ledger line and its own check from the start.

Coverage is the same claim with the digits taken out. "All eighty files" asserts that the count you opened was eighty and that you can say so on request. Sampling is a legitimate technique and an illegitimate secret: a sample stated as a sample is a finding, and a sample presented as a sweep is a false report. The words that hide one are recognizable — "various," "several," "the main ones," "and so on" — and every one of them marks a place where a number belongs.

## Long runs: hand over the ledger, not a summary

Stalling near the end is a late-in-a-long-context disease, so the durable fix is a fresh context per unit of work rather than a longer run of the same one. `delegating-tasks-with-review-gates` covers dispatching those units and gating them; `fanning-out-independent-work` covers running independent ones side by side.

What the ledger adds is what makes handing work to a fresh context safe: the unit's brief is its own ledger lines, verbatim, and its result is those lines filled in with evidence. Criteria travel with the work instead of staying behind in whoever dispatched it, so nothing rests on a summary written by a context that was already fading.

The reverse case needs naming too. A set of individually complete units can still compose into a broken whole, so the seam where they join gets ledger lines of its own — merged, interfaces matching, end-to-end behaviour observed. Those are the lines skipped most often, because by the time anyone reaches the seam, every part is already reporting done.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Report says done; the requester immediately finds a requested item missing | Criteria came from the restatement of the request rather than from its words |
| Every box checked, deliverable visibly broken | Checks were never observed failing, so they were never wired to the work |
| Ledger fully green with several evidence slots still reading `pending` | A checkbox treated as the proof rather than as the claim |
| The finished criteria list is shorter than the one written at the start | A line was deleted instead of surrendered |
| A criterion reads suspiciously like a description of what got built | Edited mid-run to match the deliverable |
| "All the files" in a summary, no count anywhere in it | Coverage asserted from impression; the count was never taken |
| Numbers in the summary contradict the artifacts they describe | Written from memory at report time instead of measured |
| The long run's last hour produced summaries and no work | Recap mistaken for progress once the criteria stopped being visible |
| Thirty units each reporting done, the assembled result broken | Criteria existed only at the leaves, never at the seam |

## Red flags

- Starting substantial work with the acceptance criteria only in your head.
- "I'll write down what's needed once I see how it shapes up."
- Composing a status summary while lines in the ledger are still open.
- Reaching for "various," "several," or "the main ones" in a sentence a count would fit.
- A number in a draft report that you cannot name the command for.
- Deciding a criterion was never really part of the request, at the exact point where satisfying it would be the remaining work.
- Relief at how close this looks to finished, immediately before writing the report.
- Sampling a large set, finding the sample clean, and describing the set.
