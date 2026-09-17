# Lenses — instruments borrowed from other disciplines

A direction says where to look from. A lens is a specific instrument to look
with — a question some other field turned into a standard technique because
its practitioners kept being blind in the same place. Each one here is filed
under the blindness it cures, because that is how you pick one: name what you
are probably not seeing, then take the lens built for it.

The catalogue is not a checklist. Running every lens is furniture, and the
tell for each is what selects it. Most sweeps use one; an orbit uses one per
pane; `lens <name>` runs a single one by hand when you already know which.

Each entry: what it asks, the tell that selects it, what it characteristically
yields, how it fails, and — where this library already has the technique in
full — the skill that owns it. When an owner is named, the lens is the reason
to open that skill, not a substitute for it.

---

## Vocabulary lock-in — you inherited the words

**Ideal final result** (TRIZ). *Asks:* what if the outcome simply occurred,
with no mechanism added at all — nothing built, nothing run, nothing
maintained? Then: what is the least that has to exist for that? *Tell:* the
request names a mechanism. *Yields:* the smallest design available, and often a
deletion. *Fails as:* a wish written down as a plan; the lens ends at the least
mechanism, never at zero.

**The job it is hired for** (product research). *Asks:* who "hires" this, in
what situation, to make what progress — and what did they fire to hire it?
*Tell:* the request describes a feature and never a moment in someone's day.
*Yields:* the outcome-level statement of the request, which is what an
acceptance line should be made of. *Fails as:* inventing the person; ask them
when `presence=paired`. *Owner:* `scoping-before-building` aims its questions
at purpose for this reason.

**Commander's intent** (military planning). *Asks:* if the situation changed
after the order was given, what would the person who gave it still want? State
the request one level of purpose up, so it survives a changed situation. *Tell:*
a run that will outlast the assumptions in the request. *Yields:* the stop
condition an unattended run should carry. *Owner:* `bounding-autonomous-work`
for the stop conditions; `finishing-what-you-started` for the ledger line.

## Anchoring — the first thing seen became the model

**Key assumptions check** (intelligence tradecraft). *Asks:* list every
assumption the current reading rests on; for each, would the work change if it
were false, and how would you know? Examine only the load-bearing ones.
*Tell:* a third fix did not hold, or a conclusion has hardened without new
evidence. *Yields:* the one assumption that explains a run of failures. *Fails
as:* infinite regress — three load-bearing assumptions is usually all there
are. *Owner:* the `under` direction is this lens applied to a request;
`diagnosing-before-fixing` applies it to a reproduction.

**Quality of information check** (intelligence tradecraft). *Asks:* for each
piece of evidence — where did it come from, how was it obtained, could the
source be wrong or partial, and is anything else corroborating it? *Tell:* a
conclusion resting on one report, one search, one tool result. *Yields:* the
finding that the decisive evidence was never corroborated. *Owner:*
`calibrating-confidence` marks the tiers; this is the pass that asks.

**The stranger's first question.** *Asks:* what would someone with none of this
context — not the codebase, not the thread, not the field — ask first? *Tell:*
expertise everywhere in the room and no dissent. *Yields:* the question the
experts stopped asking years ago. *Owner:* `red-teaming-your-own-work` calls it
distance obtained cheaply and has the mechanics.

## Scale — the size was assumed

**Ten times, one tenth.** *Asks:* what changes if the load, the count, the
budget, the team, or the deadline were ten times larger — or ten times smaller?
*Tell:* a design with numbers in it nobody chose. *Yields:* the mechanism that
only works at the assumed size, in either direction — the elaborate thing
built for a scale of one is the commoner find. *Fails as:* futureproofing;
this produces an observation and a threshold, never a feature.

**Population one.** *Asks:* what does this mechanism do when the population it
needs — reviewers, corroborators, tenants, users — is one, or zero? *Tell:* any
design built on independent agreement. *Yields:* the safeguard that is inert
until the day it matters. *Owner:* `scoping-before-building`, sizing a trust
circle.

**Altitude ladder.** *Asks:* say the problem at five altitudes — the line, the
module, the system, the product, the world it operates in — one sentence each.
At which altitude is it cheapest to solve? *Tell:* a fix that keeps growing, or
a request stated at a level nothing else in the design lives at. *Yields:* the
level the problem should be solved at, which is rarely the level it was stated
at. *Owner:* `deciding-reversibility` decides at the right altitude;
`explaining-technical-work` chooses the altitude of a report. This is also the
sixth move in the main file, because it works on anything.

## Class — the instance was taken as unique

**The reference class by shape.** *Asks:* what is this an instance of — named by
its shape, never by its domain — and what did the last several members of that
class cost, take, or turn into? *Tell:* the second or third time this shape has
come up, or an estimate about to be built from decomposition alone. *Yields:*
the number the inside view cannot produce, and the general case that four
special cases were hiding. *Fails as:* premature abstraction — one instance is
not a class; `judging-duplication` wins by default. *Owner:*
`estimating-effort` for the number; `automating-repetition` for when the
repetition becomes machinery. This is the `above` direction's instrument.

## Time — the present was assumed permanent

**Why now.** *Asks:* what changed to make this arrive this week? A request that
could have been made a year ago and was not has a recent cause outside the
sentence. *Tell:* any request. *Yields:* the trigger, which is often the real
requirement. This is `behind`'s third question and earns its own entry because
it is the cheapest lens here.

**Pace layers** (Stewart Brand). *Asks:* which layer does this change live in —
the ones that churn (interface, feature) or the ones that hold (schema, data
model, contract, organization)? Is it being built at the pace of a different
layer? *Tell:* a fast change to something slow, or a permanent change made for
a passing reason. *Yields:* the interface frozen for a UI whim, the schema
migrated for a quarter's experiment. *Owner:* `deciding-reversibility` for the
door it turns out to be.

**Pre-mortem** (Gary Klein). *Asks:* it is a year later and this has failed
badly. Write the first sentence of the postmortem. Then: what would have to be
true today for that sentence to get written? *Tell:* a plan with no named
risks, or a room where nobody is objecting. *Yields:* the failure nobody would
raise as a criticism but everyone will state as a prediction. *Fails as:* a
list of generic risks; the sentence has to be specific enough to be checked.
*Owner:* `red-teaming-your-own-work` attacks the finished thing;
`writing-postmortems` is where the real one goes if the lens is ignored.

**Indicators** (intelligence tradecraft). *Asks:* if the current reading is
wrong, what would you see first — and can it be watched? *Tell:* a decision
about to be relied on for a long time. *Yields:* a tripwire instead of a
belief. *Owner:* `bounding-autonomous-work` for stop conditions;
`revalidating-decisions` for the reopen condition on a recorded decision.

**The history in the tree.** *Asks:* what did this look like a year ago, what
was tried and reverted, and what has this project already refused? *Tell:* a
proposal that feels obvious and has no trace of ever being made. *Yields:* the
closed pull request that already answered the question. *Owner:*
`orienting-in-unfamiliar-code` (the decisions that are not in the code) and
`code-archaeology` (the reason a line exists).

## Actors — one seat was assumed

**The empty chairs.** *Asks:* which seat was nobody sitting in when this was
written — the operator at 3 a.m., the new hire, the auditor, the person who
inherits it, the integrator, the second tenant, the customer's customer, the
translator, the user on the slowest machine? What does each notice first?
*Tell:* a request written from one seat, which is every request. *Yields:* the
reader or user the design silently excludes. `seat <chair>` runs one of these
on demand; the roster is at the end of this file. *Owner:*
`explaining-technical-work` for the reader's seat; `formidable` for the user's.

**Attacker tiers.** *Asks:* who could want this to fail, with what capability?
*Tell:* a new trust boundary, a new class of data, a new integration. *Owner:*
`threat-modeling` — entirely. This entry exists so the actor pass does not
forget the adversarial chairs, and it hands off the moment it finds one.

**Show me the incentives.** *Asks:* who benefits, who pays, who is measured on
what, and where does each measurement point that the request does not? *Tell:*
a requirement nobody can justify on the merits, or a sound design that keeps
being argued with. *Yields:* the force that will shape the outcome regardless
of the design. *Fails as:* cynicism, and speculation about motive — stay with
what is observable: the metric, the deadline, the message. Motive is a question
you ask, never a finding you write.

**The mirror of the organization** (Conway). *Asks:* which team builds and runs
each part of this, and does every boundary in the design match a boundary in
who maintains it? *Tell:* a component two teams both half-own, or one nobody
does. *Yields:* the boundary that will drift because nobody owns the property
across it. *Owner:* `drawing-boundaries`.

## System — a part was mistaken for the whole

**The purpose of a system is what it does** (Stafford Beer). *Asks:* describe
what this system actually does — to its users, its operators, its data —
ignoring every statement of what it is for. Where do the two diverge?
*Tell:* a charter, a README, or a mission that nobody has compared to
behaviour lately. *Yields:* the gap between what a project says and what it
is, which is the first pane of an orbit.

**Leverage points** (Donella Meadows). *Asks:* at what level is this request
intervening — a parameter, a buffer, a structure, a delay, a feedback loop,
an information flow, a rule, a goal, the paradigm the goal comes from? Is
there a level up where the same outcome costs less? *Tell:* the same class of
fix has been applied repeatedly at the parameter level. *Yields:* the
information flow or rule whose change makes the parameter-tuning unnecessary.
*Fails as:* the grand redesign; anything found above "rule" routes to the
record, never to the gate.

**Flows across the boundary.** *Asks:* draw the boundary of the thing under
discussion; what crosses it — money, data, trust, attention, dependencies,
obligations — in each direction, and which of those has no owner on this side?
*Tell:* a property that is sensible inside every component and wrong in
composition. *Owner:* `drawing-boundaries` (the property no single component
can see) and `mapping-dependencies` (the three graphs).

**Feedback and delay.** *Asks:* what loop does this sit in, and how long
between an action here and its visible effect? What will someone do in the
meantime? *Tell:* a control that is adjusted before its last adjustment has
taken effect. *Yields:* the oscillation nobody modelled. *Owner:*
`rate-limiting-and-backpressure` for the mechanics.

## Negative space — what is absent was invisible

**The seen and the unseen** (Bastiat). *Asks:* for every effect you can point
at, name the one you cannot — the thing not built because this was, the user
who left without a ticket, the request never made because it was known to be
refused. *Tell:* a benefit stated with no cost beside it. *Yields:* the cost
that was paid silently. This is the absence section in the main file, under
the name the discipline gave it.

**Deviation guidewords** (HAZOP, process safety). *Asks:* for each parameter of
the thing — every input, output, timing, quantity, actor — apply each
guideword and describe the case it produces: **no**, **more**, **less**, **as
well as**, **part of**, **reverse**, **other than**, **early**, **late**,
**before**, **after**. *Tell:* a design reviewed only against the cases its
author imagined. *Yields:* cases, and ideas, that no unguided brainstorm
produces, because the guideword does the generating and the parameter does the
grounding. This is the seventh move in the main file. *Owner:*
`auditing-new-input-categories` and `property-based-testing` turn the cases
into checks.

**The unasked question.** *Asks:* what do people in the requester's position
usually ask that this request does not? *Tell:* a request unusually short for
its consequences. *Yields:* the requirement that was compiled out because it
was obvious to them.

**Ask the fluent expert.** *Asks:* what would a native of this category — this
script, this currency, this protocol, this regulation — flag first as the thing
outsiders always get wrong? *Tell:* extending a system to a category it has
never handled. *Owner:* `auditing-new-input-categories`.

## Measurement — the instrument was trusted

**Goodhart's law.** *Asks:* if this number becomes the target, what does it
stop measuring, and what will people do to move it? *Tell:* a metric about to
become an acceptance criterion or an incentive. *Yields:* the second metric
needed to keep the first honest.

**The ruler measures the hand** (Wittgenstein's ruler). *Asks:* a measurement
is as much a statement about the instrument as about the object — what does
this reading say about the thing that produced it? *Tell:* a number that is
implausibly good, byte-identical across a change, or zero. *Yields:* the
instrument that never ran, the proxy wearing the name of the thing it stands
in for. *Owner:* `diagnosing-before-fixing` and `calibrating-confidence` each
carry a case of this.

**The base rate and the control.** *Asks:* how often does this shape occur
anyway? What happens if nothing is done? What would the measurement read with
the change absent? *Tell:* an effect claimed with no comparison. *Yields:* the
improvement that was regression to the mean. *Owner:*
`red-teaming-your-own-work` bounds a null by what the search could have seen.

## Evolution — build and buy were not distinguished

**Where it sits on the evolution axis** (Wardley). *Asks:* is each component
of this genesis (nobody has done it), custom (some have), product (you can buy
it), or commodity (it is a utility)? Build what is genesis; buy or borrow what
is commodity. *Tell:* something being hand-built that has a price list. *Yields:*
the component that should be a dependency, and the one that should not.
*Owner:* `steelmanning-alternatives` has buy-or-build as one of its generators.

## Inversion — the negation was never tried

Owned in full by the `against` direction. The named forms, so they can be run
by hand: **via negativa** (what would removing something fix?), **do nothing**
(what actually happens?), **the worst possible idea** (state the worst answer,
then invert each of its properties), **relax the constraint**
(`revalidating-decisions` owns the case where the constraint is a recorded
decision), and **by hand, once** (do it manually to learn whether it recurs).

---

## Seats — the chairs `seat <chair>` can sit in

Each seat is one look, from one position, at what that person notices first.
Pick the seat the request was not written from.

| Seat | Notices first |
|---|---|
| The operator at 3 a.m. | Is there a runbook, an alert, a rollback, and a way to tell this failure from the last one |
| The new hire | Whether the names mean what they say and the docs match the tree |
| The auditor | Who approved this, when, and what evidence exists that it did what it claims |
| The maintainer in a year | Why any of this is the way it is, with everyone who knew gone |
| The integrator | The contract, its versioning, and what happens when they time out |
| The second tenant | Every place the first one's assumptions leaked into shared state |
| The customer's customer | What the end user experiences that the buyer will never see |
| The accountant | The cost per unit of the thing, and which unit |
| The regulator | What is recorded, retained, and provable |
| The one on the slowest machine, smallest screen, or no network | Whether it works at all |
| The translator | Every string that cannot grow and every layout that assumed English |
| The person who deletes this | What has to be true for that to be safe, and whether it ever will be |
| The attacker | Hands off to `threat-modeling` the moment the chair is occupied |
