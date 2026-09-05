# Where a sweep sits in each track

`orchestrating-work-end-to-end` routes every request to one of seven tracks.
Clairvoyance has a seat in all seven, and the seat is genuinely different in
each — a sweep run at the wrong moment in a track is not merely lower-value,
it is one of the two ways this skill does damage. The other is skipping the
routing table.

The shared rule: **a sweep goes at the last moment before the frame becomes
expensive to change, and never at a moment when something more urgent is
running.** Everything below is that rule applied.

---

## Build

**Seat:** between phase 1 (frame) and phase 2 (design). After the ledger
exists — you need the request's own words written down before you can ask
whether they are the right words — and before the first approach is proposed.

**Why there:** it is the only moment in the whole track when a reframe is
free. At phase 2 it costs a paragraph. At phase 4 it costs the plan. At phase
6 it costs the implementation, which is why a brilliant reframe at review is
usually declined and should be.

**Directions that pay:** **behind** first — the ledger you just wrote is a list
of mechanisms, and it is worth one question whether they are the outcomes.
Then whichever tells fired.

**A hit routes to:** the gate, row one. The design has not been written yet, so
reopening costs nothing but the paragraph you were about to write anyway. Say
the reframe out loud, name what it displaces, and let the phase-2 approval
decide it — a reframe adopted without the gate is the same defect as a design
adopted without the gate.

**Second seat:** phase 8, restricted to **ahead** and **beside**, feeding
`offering-the-next-move`. Run it *before* the run record is torn down: the
record's rulings and surrendered lines are the sweep's best input, and once
they are deleted the closing offer gets reconstructed from memory, which
reliably yields the obvious next step and nothing that was actually learned.

---

## Diagnose

**Seat:** when the hypothesis list empties, or when the third fix does not
hold. Not at the start — a bug with an untried obvious cause does not need a
sweep, it needs the reproduction `diagnosing-before-fixing` asks for.

**Why there:** three failed fixes is not three bad patches. It is one wrong
model, applied three times, correctly. The patches were fine. The thing they
were patching was chosen by a frame nobody examined.

**Direction that pays:** **under**, near-exclusively. The assumption is in the
reproduction, not in the fix — most often that the failure is where it
surfaced, that it is deterministic, that it is one bug rather than two, or
that the component named in the report is the component at fault.

**A hit routes to:** back to reproduce, with the assumption named. A sweep that
produces a new *patch* has done the wrong thing; it produces a new *hypothesis*,
which then earns a reproduction like any other.

---

## Respond

**Seat:** at **stabilize**, and again in the postmortem. Nowhere earlier.

**This one is a prohibition, and it is the most important line in this file.**
An incident has a clock and an audience, `responding-to-incidents` inverts the
usual order deliberately — mitigate before you diagnose — and a frame question
asked while users are broken is precisely the "let me understand this properly
first" red flag that skill names. Nothing a sweep can produce is worth a minute
of ongoing harm. The mitigation may well be crude, wrong-headed, and the thing
a sweep would have improved. Ship it anyway.

**Directions that pay, once the harm has stopped:** **under** on the cause,
**ahead** on what the mitigation itself has now committed you to — a crude
mitigation is a decision, and it is usually still in production a quarter
later. And in the postmortem, **above**: an incident of this *shape* has often
happened before under a different name, and that is the finding worth more than
the timeline.

**A hit routes to:** the postmortem's action items, not into the live
stabilization.

---

## Investigate

**Seat:** after gathering, before the conclusion is written.

**Why there:** an investigation's characteristic failure is answering the
question asked with great rigour when the question that mattered was one
sideways from it. The evidence is all in hand at this point, which is exactly
what makes the reframe checkable rather than speculative.

**Directions that pay:** **behind** — why is this being asked, and what
decision hangs on the answer? An investigation commissioned to support a
decision should be answering the decision's question. And **outside**, which is
where an unfamiliar-feeling question turns out to have a standard name.

**A hit routes to:** the report, as a named second question with its own
confidence marking, per `calibrating-confidence`. Both questions get answered:
the one asked, fully, and the one you believe matters, marked as your inference.
Substituting the second for the first is not a reframe, it is not answering.

---

## Review

**Seat:** during coverage, before judgment. `orchestrating-work-end-to-end`
already makes coverage of the diff this track's gate; this is what makes
coverage mean more than "every changed file was opened".

**Directions that pay:** the absence pass from the main file, aimed at the
diff. What is *not* in this change — the caller not updated, the state not
handled, the test for the case the author did not think of, the migration for
the data already in production. `reviewing-code-deeply` owns weighing absence
alongside presence; this is the method for finding it.

**A hit routes to:** a finding, phrased as a question about a case rather than
as an accusation. And the sweep applies to the review itself: a review that
generated forty comments and no observation about the change's *shape* has
inspected without seeing.

---

## Author

**Seat:** twice. At *frame the reader*, and again before *verify*.

**Why twice:** the first is about who this is for, which determines everything
downstream and is nearly free to change at that moment. The second catches the
document that is now internally consistent and answers a question nobody has.

**Directions that pay:** **beside** — who *else* reads this, and what do they
need that the primary reader does not? The on-call engineer, the person
evaluating this in a year, the newcomer with none of the context. And
**ahead** — what does this document make impossible to say later? A document
that overstates a certainty commits its author's successors to defending it.

**A hit routes to:** the draft, directly, since the artifact is the
deliverable and the frame *is* the work here rather than a precondition to it.

---

## Change in place

**Seat:** before `deciding-reversibility`, not after.

**Why there:** reversibility is a question about a chosen operation. Asked
first, it silently accepts that the operation is the right one. The sweep goes
one step earlier, at the operation itself.

**Direction that pays:** **against**, first and hardest. The cheapest migration
is the one that does not happen because the thing being migrated can be
deleted; the cheapest upgrade is the one avoided by removing the dependency.
Then **behind** on the thing itself, which is Chesterton's fence in this
library's vocabulary: `code-archaeology` recovers why it exists, and a thing
whose reason has expired is a deletion rather than a migration.

**A hit routes to:** a different operation entirely, which then re-enters this
track at reversibility. That is a track staying put, not a downgrade.

---

## The envelope, dial by dial

The track says where the seat is. The envelope says what the sweep may do
from it.

| Dial | Effect on the sweep |
|---|---|
| `presence=paired` | **Ask rather than infer.** *Behind* is cheap here — one message beats a page of inference about intent, and it cannot be condescending if it is a question |
| `presence=async` | Batch the sweep's one question with the other open ones; continue on the original frame meanwhile |
| `presence=autonomous` / `unattended` | The sweep runs; **a reframe may not be self-approved.** Write the observation, the reading that lost, and a stop condition. Then build what was asked. See below |
| `amplitude=express` | One direction, selected by tell. No tell fires, no sweep, and that is correct rather than skipped |
| `amplitude=standard` | Three: the tells that fired, plus **against** |
| `amplitude=campaign` | All seven at the design gate; **ahead** and **beside** again at close. The discard pile goes in the record |
| `radius=sandbox` / `repo` | Ordinary sizing |
| `radius=live` | **Ahead is mandatory.** The second-order consequence is the entire reason the radius is live, and the sweep is the only phase that looks at it |
| `crew=solo` | Ordinary sizing |
| `crew=fanned` / `shared tree` | **Beside is mandatory**, aimed at the other writer: what you are about to do that they cannot see, and what they are doing that you cannot |
| any `register` | The sweep's *output* compresses; its *routing* does not. An empty sweep is one line. A finding routed to the record is written in full, because the record is read by someone who was not here |

### The autonomous stop, stated once more

An unattended run may sweep, and may not act on what it finds by changing what
is being built. Building something other than what was asked, with nobody
present to agree, is not a bounded risk that confidence can retire — it is
the definition of the failure `bounding-autonomous-work` exists to prevent,
arrived at through a door marked *insight*.

The substitution is the standard one: the gate keeps its job and changes its
mechanism. Write the observation into the record, name the reading that lost,
add a stop condition that fires if the original frame turns out wrong in a way
the run can detect, and continue on the original frame. The reframe is then
waiting for whoever reads the report — which is the run finishing correctly,
not the run being blocked.
