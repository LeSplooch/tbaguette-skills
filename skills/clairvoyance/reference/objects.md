# What can be swept — the objects, one by one

The main file's method was written for a request, and a request is one instance
of the general case: **anything that arrives carrying a frame it did not
announce.** A plan carries its author's judgments compiled out. A metric
carries its instrument. A codebase carries "how it is done here". A project
carries a charter nobody has compared to its behaviour lately. The run you are
in carries the track and envelope you chose an hour ago, which have become
premises.

The method is the same for every object — name the frame, restate without its
nouns, look in the direction the tells select, name the observation that would
tell two readings apart, route what you find. What changes per object is four
things: where its frame comes from, the moment a look is cheap, the direction
that pays first, and where a hit goes. That is this table, then the notes.

| Object | Its frame comes from | The cheap moment | Look first | A hit routes to |
|---|---|---|---|---|
| A request | its own vocabulary | before a design exists | behind | the design gate |
| A bug report | the report's wording — where it surfaced, one bug, that component | the third failed fix, or an empty hypothesis list | under | back to reproduce, as a hypothesis |
| A plan | the judgments its author made and did not write down | before the first task runs, and at every task that cannot be done as written | under, then against on the heaviest task | phase 4 — the plan, never the task |
| A design or spec | the first approach anyone proposed | before the yes | against, beside | the gate, with what it displaces named |
| A decision or ADR | its premises, dated when it was written | when it is about to be relied on | ahead (what has changed since) | `revalidating-decisions` |
| A measurement, report, or dashboard | the instrument, and whoever chose it | before anything acts on the number | under, plus a measurement lens | `calibrating-confidence`, as a tier mark |
| A document | the reader it was written for | at *frame the reader*, and before verify | beside, ahead | the draft |
| A codebase or architecture | "how it is done here" | on orientation, and at the fourth special case | above, against | the record, or an ADR |
| A project | its charter, against what it actually does | the orbit moments in [orbit.md](orbit.md) | around, then all of them | the orbit record |
| A conversation | your own previous turns | when a conviction has grown without new evidence | under | said out loud, once |
| The run itself | the track and envelope chosen at the start | a stop condition firing, or the third failure of one shape | against, under | the run record's phase log |
| A tool or instrument | what it can and cannot see | when its result is about to stand for the whole | under | `calibrating-confidence` |

## Notes on the objects that are not requests

**A plan.** Executing a plan exactly as written is correct discipline and also
means adopting every judgment its author made without seeing one of them. The
plan's frame is those judgments. The cheap moment is before task one, and the
tell inside execution is a task that cannot be implemented as written — that
is the plan's frame failing, not the implementer's, and
`working-a-plan-task-by-task` routes it back to the plan for that reason. Sweep
the plan, never the task.

**A decision.** A recorded decision has a principle and a premise, and only the
premise decays. The sweep's question is `ahead` run backward: what has changed
since this was written that its author could not have known? The finding is a
premise that expired; `revalidating-decisions` owns what happens next, and the
sweep must not overturn the decision itself — that is that skill's job, done
with the record open.

**A measurement.** A number arrives with the authority of an instrument and the
frame of whoever pointed it. Three questions, from the measurement lenses:
what does this reading say about the instrument, what would it read with the
change absent, and what does it stop measuring once it is a target. The
finding is a tier mark — verified, inferred, assumed — on the number before it
is acted on. `diagnosing-before-fixing` owns the case where the instrument
never ran.

**A codebase.** "How it is done here" is the strongest frame in the list because
it is usually right, which is what makes the cases where it is wrong
expensive. The tell is not the pattern itself but its fourth special case. A
hit is an observation about the pattern, routed to the record or to an ADR —
never a refactor started from inside a sweep. `steelmanning-alternatives` owns
what to do once the pattern is admitted to be a choice.

**A project.** The one object whose frame nobody wrote: it is the sum of what
everyone assumes the project is for. The method for it is [orbit.md](orbit.md),
and its bound is stricter than any other object's, because the space around a
whole project is unbounded and every pass can be made to produce something.

**A conversation.** Your previous turn is the fourth frame source in the main
file, and a conversation is that source compounding. The tell is a conviction
that has grown across several turns without new evidence entering — an approach
adopted three responses ago that is now a premise. The sweep is one sentence:
say what you are now assuming that you were not assuming at the start, and let
the person correct it.

**The run itself.** A track and an envelope were chosen at the start of the run
and have since become the water it swims in. `orchestrating-work-end-to-end`
re-reads them at phase boundaries; this is the sweep that asks whether the
*process* is the frame — a Diagnose run that should have been a Build, an
express amplitude that has quietly become a campaign, a fix loop whose third
iteration is the tell. A hit routes to the run record as an envelope change,
which is a decision, and never to a silent downgrade.

**A tool or instrument.** Everything observed through a tool arrives shaped by
what that tool can see. A search that returns nothing is a fact about the
search. A truncated result is a fact about the pipe. The sweep's question is
what this instrument structurally cannot show, and the hit is a tier mark or a
second instrument — `calibrating-confidence` owns reading a hit list as the
input to a check rather than as the check.

**The library itself.** A skill that fired and turned out to be silent about
the case in front of you is a frame too — the library's. That observation
does not route to any gate in this run; it routes to `tending-tbaguette`'s
queue, in two sentences, and the run continues.
