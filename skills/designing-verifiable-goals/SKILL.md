---
name: designing-verifiable-goals
description: Use when a run is about to be handed a goal to iterate toward on its own — a loop re-prompting itself until a condition holds, "keep going until it passes", "do not stop until it is twice as fast", an overnight port — and nothing yet says, mechanically, whether an iteration got closer, or when the harness or judge such a run is graded by is being built. Also use when the only available check is a model or a screenshot judging "close enough", when a long run's results land just past its bar, when its visible and held-out scores diverge, when a climbing run has slowed and nobody has asked why, or when a speed, size or cost target is about to be added to work that does not yet pass. Covers the check as the real goal, the rungs from a reference oracle through measured and judged proxies to a person, failure parity, a held-out slice, stacking objectives as constraints, what a judged proxy gets optimized into, where the bar sits, and keeping the climb fed.
---

# Designing verifiable goals

## Overview

A run left alone with a goal climbs whatever it is graded against, as far as that check can see. A run graded by a program climbs. A run graded by its own sense of done stops when it feels done, or wanders while looking busy, and from outside both look like the goal being worked.

So the goal a long run pursues is not the sentence in its prompt. It is the check consulted at each iteration. "Make it feel like the original" and "make it five times faster" are aspirations. What gets optimized is the command or the judge, faithfully, including in directions nobody meant. Building that check is the work that decides the result. Do it before the climb starts, in a setup pass, and freeze it.

## When to use

- A goal will be pursued by a run iterating on its own: a loop re-prompting itself, an overnight port, "do not stop until".
- Something already does the target behaviour (a tool being ported, a previous version, a conformance suite) and the new work should match it.
- The only available check is a judgment: a model looking at a screenshot, a rubric, "close enough".
- A second objective (speed, size, cost) is about to be added to work that passes, or does not yet.
- A long climb has slowed, or its check takes longer than the change it checks.
- Not for: the stop, decision and door bounds every autonomous or unattended run needs. `bounding-autonomous-work` owns those, and a climb needs them as well as this.
- Not for: measuring a shipped feature's model output. `evaluating-llm-output` owns that, and its judge rules apply here too, with the run's output in place of a shipped feature's.
- Not for: catching a run that already gamed its check. That is `confirming-before-claiming-done`.

## Before the climb starts

- [ ] The check is a command that prints pass or fail and a score, and it has been watched failing.
- [ ] The reference is pinned, and failures are among its recorded outputs.
- [ ] A held-out slice is split off, out of the run's sight, and large enough that the gap you care about beats its noise.
- [ ] The reference, corpus and harness are frozen and read-only to the run.
- [ ] Any second objective is gated behind the first, with a minimum gain per iteration.
- [ ] The bar is written as the result wanted, margin included.
- [ ] A judged proxy, if any, has rejected broken copies in both orders and at several sizes, is calibrated against hand labels, has one stated reference, and is re-measured at each checkpoint on a fresh hand-labelled sample of the loop's own output.
- [ ] The harness logs waiting and working time per iteration, and something outside the run asks what blocks it, on a stated interval.
- [ ] A spend ceiling covering tokens and compute is approved by whoever pays, and reaching it halts the run and reports.
- [ ] The other stop conditions are written (`bounding-autonomous-work`), with the budget counted in iterations that produced no accepted gain, not in failures of the bar, which fails on every iteration until it is met.

## Take the highest rung of check that exists

| Rung | Exists when | What the run climbs | Watch for |
|---|---|---|---|
| Reference oracle | Something already does the target behaviour | Equality with the reference's output, errors included, on a real corpus | It freezes the reference's bugs as the spec. Fine for parity; retire it once real specifications exist (`property-based-testing`) |
| Behaviour within a tolerance | Outputs differ legitimately: another binary, another renderer | What both do on the same inputs, inside a stated margin | The margin becomes the target |
| Measured proxy | The outputs cannot be compared directly, even inside a margin, but a property can be measured, from recordings or by driving the reference: trajectories, timings, curves | Distance between your measurements and the reference's under matched conditions: the same inputs where the reference can be driven, otherwise the same situations reconstructed from its recordings | It sees only what the instrument measures |
| Judged proxy | Only a judgment exists: look, feel, tone | A model or a panel comparing yours with a reference | The run optimizes the judge, below |
| A person | Nothing else | Nothing between checkpoints | If this is the highest rung, the work is not a climb. See "Choose the work by whether it can be checked" |

Step down only when the rung above is genuinely missing, and look hard first. A "feel" requirement usually has measurable parts, such as how high a jump goes, how long it takes to peak, and how fast a character accelerates and stops. A deterministic tool can pull those out of reference recordings, which turns much of a judged proxy into a measured one.

A reference oracle needs four things. A setup pass builds them, by a person or by an agent before the climb, and the climb starts only once they are frozen:

- **Pinned.** Exact version and environment. A reference that moves, moves the goal.
- **Failures are outputs.** Record errors, warnings and exit codes, and break part of the corpus on purpose so the reference has something to reject. A clean corpus cannot tell a port that rejects correctly from one that accepts everything.
- **Out of the run's reach.** The reference, the corpus and the harness are read-only to the run. Read-only stops edits to the check, not code that special-cases its inputs; the held-out slice below is what catches that (`confirming-before-claiming-done`).
- **Watched failing.** Plant a few known mismatches and confirm the harness reports them before trusting a green. A harness an agent wrote is a judge nobody has yet seen disagree.

## Hold some of the check back

Any case the run can see when it is graded is a case it can fit, and passing every visible case while failing unseen ones is common enough to plan for. This holds for proxies as much as for oracles: targets derived from every reference recording are targets the run can fit to those recordings. Split before starting: a working set the run climbs, and a held-out slice from the same distribution that it never sees, scored at checkpoints by something outside it. Size the slice so the gap you care about is bigger than its noise; forty cases cannot tell 80% from 85% (`evaluating-llm-output`).

The gap between the two scores is the reading that matters, and a widening gap is the run learning the tests. Report held-out failures back by category, never by case. A failing case handed back has just joined the working set.

## Stack a second objective as a constraint

Speed, size or cost goes on only once the first objective passes, and it goes on as "faster while every existing check still passes", enforced as a gate on each iteration. Score the held-out slice at every checkpoint of a speed climb too, because a speed climb is the strongest pressure there is to drop what the visible checks do not cover. Never fold the two objectives into one score: a climber offered that trade finds the exchange rate.

Measure the second objective so it can be trusted. Time the reference against itself to find the noise, and count an iteration as faster only past a minimum gain written in the target's units and larger than that spread. An iteration that takes the median past the bar counts whatever its step size. On a machine shared with other workers no clock is clean: give the timing runs cores of their own, interleave candidate and reference runs rather than timing them in blocks, and where the platform exposes one, record a count the scheduler cannot inflate (instructions, allocations, queries) beside wall-clock time, never instead of it (`performance-profiling` covers machine noise and interleaving).

## The bar is where it stops

A climber stops at the bar. Results clustered just past the threshold, every case at 1.21 against a target of 1.2, are the bar showing through, not evidence of headroom. Write the bar as the result you actually want, and put any margin in the bar itself. If 1.2 is the requirement and the reference timed against itself spreads by 5%, the bar is 1.26, on the median of a fixed number of interleaved runs. Then re-measure the run's "target met" yourself before you report it as met.

## A judged proxy is a reward, and it gets optimized

When the check is a model judging likeness to a reference, the run optimizes the judge, and the judge's leanings become features of the result.

- **The reference decides the result.** A run told to match a reference matches it, including everything nobody liked about it. Choose the reference as carefully as the goal. When it turns out wrong, replace it and restart the comparison rather than arguing with the run. Pick one reference for the goal rather than averaging several into a result none of them had; if one aspect really should follow a different reference, name which aspect follows which.
- **Effort goes where the score moves.** A proxy pulls iterations toward whatever it can see changing, such as a background element, a global brightness shift, or in prose a stock phrase the judge rewards, and away from parts that matter but barely move the score. Read the iteration log by what changed, not only by the number.
- **A judge counts only once it has been seen to reject.** Before its verdict steers anything, show it the build beside a deliberately broken copy (gravity doubled, speed halved, or for text the draft with its hedges stripped or its register flipped). Show each pair in both orders and at random positions, at least twenty times per break, and require the right pick in, say, 19 of 20. Break at several sizes, such as off by 50%, 20% and 5%. The smallest break it reliably catches is the finest difference it can steer; once the climb's changes are smaller than that, the judge has stopped steering.
- **Then calibrate it as `evaluating-llm-output` describes.** Label a sample by hand. On a held-back part, report how often the judge passes what the labeler passed and how often it fails what the labeler failed, as two numbers. Prefer a judge from a different model family than the one running the climb. Re-label on what the loop produced, because agreement measured before the loop does not survive it. A panel of judges takes the majority verdict and is measured the same way as a single judge.
- **Compare behaviour, not stills.** A screenshot judges appearance at one instant. Where the reference can be driven, capture several input sequences, hold some back as above, pin the clock and the random seeds in both, replay each through both, and compare what happens over time. Where it cannot be driven, measure motion from its recordings as above.

## Keep the climb fed

During a long climb the operator's job is not reading each change. It is keeping the run supplied, and what runs short is often not the model. Write this half into the goal too:

- **The harness, not the run, appends one line per iteration:** what the run says it changed beside the harness's own summary of the files and regions touched, the score the harness computed, and time spent waiting (for a build slot, a machine, a lock) and working, both timed by the harness. Without the waiting column, a climb starved for compute looks like a climb out of ideas.
- **The check's latency is the climb's speed limit.** An iteration is a change plus a check. When each check waits hours for a build slot, the run gets a handful of attempts a day however capable it is, and adding workers adds queue. The cuts that work: build once and snapshot the warm environment, so each worker starts from it instead of rebuilding everything; cache the reference's functional outputs once two runs of it have been seen to agree, normalising fields that legitimately vary, but never cache its timings; size compute to the peak the workers reach, not to their average (`fanning-out-independent-work` has how to measure it).
- **Have something outside the run ask what is blocking it, on a schedule.** A second session that reads the log and the run's history and answers "what is slowing this down right now?" finds what the worker is too busy to report: the machine that keeps running out of memory, the queue, the tool it needs and lacks. It diagnoses and writes down what it found. A person supplies it, because a machine, a tool or more quota is a door under `bounding-autonomous-work`, unless the goal pre-authorised that supply with a limit. Then ask again, at the interval the goal names.
- **A slowdown can be a climb of its own:** the slow compiler, the slow suite. The climb works without fixing it, so under `managing-scope-drift` it is adjacent work, recorded and reported, and it becomes a new goal with its own check only when someone decides so. Something that stops every check outright is necessary work: the run fixes it and logs one line.

## Choose the work by whether it can be checked

Long climbs suit work that is tedious for a person and mechanically checkable: a large port against a reference, a conformance chase, a speed-up under a fixed suite. Tedium does not wear a run down the way it wears a person down, but a missing check costs it everything, so interesting work with no check is a poor climb at any budget. A long climb still spends real money, in tokens and often more in compute, and spending money someone else pays for is a door under `bounding-autonomous-work`. Get the ceiling approved by whoever pays before the start, and write reaching it as a stop condition that halts the run and reports.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The run reported done early and the result is unusable | The goal had no check, so looking finished was the only signal |
| Parity on the corpus, broken on the first malformed input | Only successes were compared; failures were never outputs |
| Perfect on its own tests, poor on anyone else's | No held-out slice, or held-out failures handed back case by case |
| Faster and subtly wrong | Speed was scored together with correctness, or gated only on visible checks the run then learned to fit |
| Every benchmark lands at 1.21 on a 1.2 target | The bar was the stop, and the margin was never in it |
| Hours spent on a detail nobody asked about | A judged proxy rewarded what it could see change |
| The result reproduces what nobody liked about the reference | The reference was chosen because it was available, not because it was wanted |
| The judge approved every iteration of the last two days | Its changes had shrunk below the smallest break the judge was ever shown to catch |
| Four workers ran at the speed of one | The check's queue was the limit, so workers added queue |
| The climb slowed for days and nobody knew why | Nothing outside the run asked what was blocking it |

## Red flags

- "It'll know when it's done."
- "The model can tell whether it looks right" — with nothing measuring the model.
- "Give it every case, more data is better."
- "Make it correct and fast" as one instruction.
- "It hit the target" — with every number at the target.
- "It's slow, add more agents."
- "The run can build its own harness as it goes."
