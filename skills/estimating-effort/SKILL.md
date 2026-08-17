---
name: estimating-effort
description: Use when asked how long something will take, how big a change is, or whether work fits a given window; when producing a plan, schedule, or sequencing commitment with durations attached; when a previous estimate has already been overrun and a new number is needed; or when the spread is wide enough that any single number would be invented. Covers reference classes, ranges, the planning fallacy, spikes, and re-estimation.
---

# Estimating effort

## Overview

An estimate assembled by decomposing the plan you can see underruns systematically, because the visible plan is the part that goes right. Anchor on how comparable work actually went, adjust only for differences you can name, and use decomposition as a cross-check rather than a source.

## When to use

- Asked how long something takes, how large a change is, or whether it fits a window
- Producing a plan with durations, a schedule, or a sequencing commitment
- An estimate has already been overrun and a replacement number is needed
- The plausible range is wide enough that a point estimate would be fiction
- Not for: breaking work into steps — `structuring-an-implementation-plan` owns the decomposition. This owns the numbers attached to it, and whether they carry information.

## Reference class first, decomposition second

1. **Name the class by shape, not domain.** "A change inside one module that already has tests." "An integration with an external service whose behaviour is undocumented." "A rename across an unknown number of call sites." The domain — billing, rendering, sync — predicts far less than the shape.
2. **Find real outcomes.** Version-control history for comparable changes, how long the last two similar tasks actually took, elapsed wall-clock rather than remembered effort. Remembered effort systematically excludes the parts that went badly.
3. **Start from that number.** Adjust only for differences you can point at. "We understand this codebase better now" is the standard unearned adjustment and is nearly always wrong, because whatever familiarity existed was already priced into the previous outcome.
4. **Decompose as a check.** When the bottom-up sum lands well below the reference class, the decomposition is missing tasks. That is the near-universal direction of the discrepancy — not the reference class being pessimistic.

## Why decomposition alone makes it worse

Decomposition enumerates the tasks you can name. The cost lives in the ones you cannot: integration between the pieces, the second-order change to callers, review latency and the revision it produces, environment and build breakage, and the repair for whatever verification turns up. Missing tasks are asymmetric — they only add. Every named subtask can be estimated correctly and the total still underruns.

"Double it" works because it approximates the missing mass, and it is still a poor estimate: it conceals which unknowns dominate, so no one can act on it. Naming the top two unknowns and how far each swings the number is strictly more useful than any multiplier.

## Ranges, and what the width means

Give an interval you would defend eight times out of ten. The ratio between its ends is the real output.

| High ÷ low | Reading | Action |
|---|---|---|
| Under 2x | Understood work | State the range; plan against the high end |
| 2x to 4x | One or two genuine unknowns | Name them; the estimate is explicitly conditional on them |
| Over 4x | Not yet an estimation problem | Time-box a spike; estimate the spike, not the work |

A spike is a bounded investigation whose deliverable is a better estimate. Fix its budget before starting and state the single question it answers. A spike with no exit condition is just the work begun without an estimate, which is the thing the spike existed to prevent.

## What makes an estimate honest

The test: if this estimate is overrun, could the reader have predicted from what you told them *which* unknown did it? If not, the estimate was under-specified regardless of whether the number happens to land.

- "About two days" carries no information about whether it is a measurement or a wish.
- "One to two days if the export format matches the sample, four to five if it does not, and an hour of investigation settles which" is actionable — the reader can choose to buy the hour.
- Include review, revision, and verification. An estimate covering only "until it works once" runs at roughly half the real cost.
- Estimates of your own future diligence — that this time it gets done properly — are not evidence and belong in no number.

Never form the number after hearing the deadline. The estimate exists before the constraint is known; then answer the fit question separately, as a yes or no plus what would have to be cut.

## Re-estimating

New information changing the number is the process working. Re-estimate the moment the information lands, not when the original number expires: the same revision delivered at discovery is a decision the user gets to make, and delivered at the deadline is a report of failure.

State the delta and its cause in one line. Silent absorption is the failure mode — quietly cutting tests, verification, or scope to defend a number nobody would have insisted on if asked.

Percent-complete is not an estimate. "90% done" measured against enumerated tasks stalls indefinitely, because the remainder is the unenumerated ones. Report what is left as items, not as a fraction.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Every task lands on roughly the same unit | The number is a comfortable rhythm, not a forecast |
| A point estimate with no range | A range would expose that the number is a guess |
| Bottom-up sum well under the last comparable job | The decomposition captured the plan rather than the work |
| Estimate produced after the deadline was mentioned | Anchored on the asker's number |
| Overrun explained as "unexpected issues" | The unknowns were never named, so every one is unexpected by construction |
| The revised estimate is wrong in the same direction | Re-derived from the plan again instead of from elapsed evidence |
| "It depends" offered in place of a number | A range is how "it depends" gets said usefully |

## Red flags

- "Should be quick", "just a config change", "one-line fix" said before opening the file
- A number produced before any comparable past outcome was recalled
- An estimate covering the happy path with nothing for integration, review, or fixing what verification finds
- A schedule that assumes nothing else lands in the same window
- Reporting progress against the original estimate while privately knowing it is gone
