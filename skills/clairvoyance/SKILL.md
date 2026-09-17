---
name: clairvoyance
description: Use when a request has been understood and is about to be executed and nothing has yet asked whether it is the right request; before a plan, design, decision, metric, or document is accepted as given; when a third fix has failed, two requirements look incompatible, every option or idea so far has the same shape, or the approach came from the first file opened; when a subagent's report, a dashboard number, or a recalled fact is about to be trusted as evidence; when asked for the big picture, what a project is missing, or what else could be done; or when a run is closing and what it noticed is about to die with the context. Covers the frame and its sources, eight directions and the tells that pick one, moves — restating without the request's nouns, changing altitude, auditing constraints, deviation guidewords, the observation that tells two readings apart — lenses from other disciplines, a bounded orbit of the whole project, ideas produced as observations, and routing all of it to a gate, not into work.
user-invocable: true
argument-hint: "[sweep|orbit|imagine|lens <name>|seat <chair>|behind|under|above|beside|ahead|against|outside|around|repertoire|route]"
---

# Clairvoyance

## Overview

This library is a machine for doing the stated thing well. Every skill in it is sharp about its own stretch and all of them are pointed inward at the request: reproduce it, scope it, plan it, test it, prove it, land it. Run correctly, that machine produces an excellent implementation of whatever arrived in the first message. It has no organ for noticing that what arrived in the first message was the wrong thing to build — or that the plan, the metric, the decision, or the project it arrived inside was the wrong thing to stand on.

That is the gap this fills, and the name is a claim about the mechanism rather than a boast. Clairvoyance is *clear sight*, not prophecy. Nothing here predicts anything. It makes visible what was already there and unlooked-at — in the request, underneath it, around it, and in the whole it is a part of — because the reason it went unlooked-at is not that it was hidden. It is that the request came with a frame, and you accepted the frame in the first sentence you wrote back.

The frame is invisible for a specific reason worth stating plainly: **it is made of the requester's own vocabulary, and you inherit it by answering in their words.** "Add a retry to the uploader" hands you three commitments before you have thought anything — that there is one uploader, that retrying is the remedy, and that the failure is transient. Every skill in this library then executes flawlessly inside all three. A request is only the commonest carrier: a plan carries its author's judgments compiled out, a number carries its instrument, a project carries a purpose nobody has compared to its behaviour lately. Anything that arrives with a frame it did not announce is this skill's object.

People ask for this as "God's view", and the phrase is right about what it wants and wrong about what exists. There is no seat that sees everything. The view from above is assembled from several partial seats, chosen on purpose — and it is bounded, or it becomes the most sophisticated available way of not finishing. The counterweight this needs is `managing-scope-drift`, and it is not an opponent. It is the other half. The rule that makes both true at once is the whole discipline here: **seeing more is free; doing more is not.**

## When to use

- A request is understood, and the next action would be to start executing it.
- Before proposing a design — the one moment when a reframe is nearly free.
- Before a plan is executed, a decision relied on, a number acted on, a document sent, or a pattern followed because it is how things are done here.
- The third fix for the same bug did not hold, or the hypothesis list is empty.
- Two requirements look incompatible and a trade-off is about to be split.
- Every alternative — or every idea — generated so far has the same shape as the first one.
- The approach came from the first file you opened, or a plan is about to be executed exactly as written by someone who did not write it.
- Someone asks for the big picture, what a project is missing, what else could be done, or what they are not seeing.
- A run is closing, and what it noticed in passing is about to die with the context.
- A sweep is being asked for by name — someone wants the space around this, not the thing itself.

Bounded against its neighbours, because the overlap is real and filing this wrong makes it useless:

- **Not for** generating other answers to a settled question — `steelmanning-alternatives` owns option generation, and it operates on a candidate that already exists. This runs before there is one, and it questions the *question*. An `imagine` pass produces candidates for that skill to work on; it does not choose among them.
- **Not for** attacking a finished artifact — `red-teaming-your-own-work` owns adversarial review, and it asks whether the thing is wrong. This asks whether the thing is the point.
- **Not for** turning an idea into an approved design — `scoping-before-building` owns that, and this feeds it. A sweep that produces a reframe hands it to the design gate; it does not skip one.
- **Not for** enumerating adversaries — `threat-modeling` owns the actors who want this to fail, and the actor pass here hands off the moment it finds one.
- **Not for** justifying work nobody asked for. `managing-scope-drift` still governs everything a sweep, an orbit, or an idea makes you want to do, and it wins.

## In thirty seconds

The whole method, in five lines. The rest of this file is those five in full.

1. **Name the frame.** What is the object — a request, a plan, a number, a project, the run you are in — and where did its frame come from? An approach whose rejected siblings you cannot name was not chosen; it was inherited.
2. **Restate it without its own nouns.** If you cannot, you have not understood it. If you can, the alternatives are usually visible in the restatement.
3. **Look where the tell points.** One direction for a small change, three for an ordinary one, all eight for a campaign. No tell fires, no sweep.
4. **Name the observation that would tell two readings apart**, and go and look. A reading nothing could distinguish from its rival is not a finding.
5. **Route everything** — to a gate, the ledger, the record, or a discard with a reason — before anything acts on it. Then say what you found in one line, or say you found nothing in one line.

| The tell | Look | With |
|---|---|---|
| The request names a mechanism, not an outcome | behind | ideal final result, why now |
| A third fix failed; the bug moved; a parameter does nothing | under | key assumptions check |
| The second or third time this shape has come up | above | the reference class by shape |
| The change touches something else's boundary, or another writer is in the tree | beside | flows across the boundary |
| Easy now, permanent afterwards; the radius is live | ahead | pre-mortem, pace layers |
| A fourth special case just arrived | against | via negativa, do nothing |
| The problem feels novel | outside | the stranger's first question |
| More than one owner; a date, budget, or audience the request never named | around | show me the incentives, the empty chairs |
| A number is about to be acted on | under | Goodhart, the ruler measures the hand |
| Asked for the big picture, or arriving on inherited work | `orbit` | one lens per pane |
| Every idea so far has one shape | `imagine` | the divergence lenses |

## Sight is free. Action is not.

This is the pin. Without it, a skill about seeing more is an engine for scope inflation with a nice name, and every observation it produces arrives already feeling like a mandate.

A sweep — one bounded pass, sized by the table further down — produces **observations**. It does not produce work. Neither does an orbit, and neither does an idea: an idea is an observation about the possibility space, and it routes exactly like a finding about the request. Every observation gets routed into exactly one of four places before anything is acted on, and being routed is not optional — an observation that skips the table is scope drift wearing a costume.

| Route | When | What it becomes |
|---|---|---|
| **Back to the gate** | The observation changes what should be built | Reopen the phase that owns it — usually design. The spec changes visibly, and the approach that lost stays written down |
| **Into the ledger** | The observation changes what would count as done | A new acceptance line, watched failing once like any other, per `finishing-what-you-started` |
| **Into the record** | It is real, and it is not for today | A ruling in the run record, and an input to `offering-the-next-move` at close. This is where most of them go, and where every idea goes |
| **Discarded, with a reason** | It is interesting and it is not true, not yours, or not worth it | One line saying why. A discard with no reason gets re-derived by the next session |

Two things follow that are easy to skip. **Nothing is acted on straight out of a sweep** — not even the obviously-correct finding, because "obviously correct" is exactly how the frame felt an hour ago. And **the third row is the common case**, not the consolation prize. A run that routes everything to the first row is not perceptive; it is one that has decided the request was wrong seven times in an afternoon.

Then the rule that outranks all four rows: **a sweep is never a reason something did not ship.** "I looked into it and the request was the wrong shape" is the most sophisticated available form of not doing the work, and it is indistinguishable from insight right up until someone asks where the deliverable is. `finishing-what-you-started` still governs; a reframe raised and declined leaves the original ledger fully owed.

The table says *where*; it does not say *when*, and one class of observation cannot wait for the close. A live vulnerability, a data-loss path, or a false premise the current work is already standing on is not a row-three finding that happens to be serious — it is an interruption, and `managing-scope-drift` owns the judgment about which observations earn one. Severity sets the timing; the routing table still sets the destination.

One row has a second home. A discard whose lesson would hold in a repository you have never seen is not only a discard — that is exactly what `tending-tbaguette` queues, and the two sentences it costs are cheaper now, while the reasoning is still in front of you, than the reconstruction later.

## Where the frame comes from

Five sources, roughly in descending order of how much they steer. Visibility runs the other way and does not run cleanly: the last one below is both the least visible and among the strongest, which is what makes it worth its own entry.

1. **The request's vocabulary.** Every noun is a commitment and every verb is a chosen remedy. This is the strongest frame and it operates before you have made a single decision.
2. **The first file you opened.** Orientation is not neutral. The module you read first supplies the model you will fit everything else into, and if you opened the wrong one, every subsequent read confirms it — `orienting-in-unfamiliar-code` is about doing this well, not about doing it without consequence.
3. **The codebase's existing pattern.** "How it is done here" answers a question nobody asked out loud, and the answer is usually right, which is what makes the times it is wrong so expensive.
4. **The previous turn.** Yours. An approach adopted three responses ago is now a premise, and nothing in the transcript marks the moment it stopped being a choice.
5. **A report you did not produce.** A subagent's conclusion, a tool's output, a search that came back empty, a dashboard number, a plan handed to you already written, a memory from a session you do not remember. Each arrives as a finding with its framing already compiled out — which is the same shape a request has, and it is trusted more, because it looks like evidence rather than like a choice. A plan is the sharpest case: executing one exactly as written is correct discipline and also means adopting every judgment its author made without ever seeing one of them.

The tell for all five is the same and it is worth learning: **you can state what you are doing but not what it is instead of.** An approach whose rejected siblings you cannot name was not chosen. It was inherited.

### Two frames disagreeing is the cheapest sight there is

They are five sources, not one frame, and they do not always agree. The request's vocabulary says *retry*; the codebase's pattern says everything here is idempotent and retried at the edge. The subagent's report says the parser is at fault; the file you opened first says the parser has one caller.

A disagreement like that is worth more than any sweep you could run on purpose, because the noticing has already been done for you — you did not have to step outside the frame, two frames stepped outside each other. It is also the easiest thing in this file to walk past, because the reflex on meeting one is to pick the more authoritative source and move on. Do not pick. **A conflict between two frames is a finding before it is a question to settle**, and the move it hands to is the fifth one below — name the observation that would tell the two apart, then go and look.

## What can be swept

The method was written for a request, and a request is one instance of the general case: anything that arrives carrying a frame it did not announce. What changes per object is where its frame comes from, when a look is cheap, which direction pays first, and where a hit goes. [reference/objects.md](reference/objects.md) has each in full.

| Object | Frame from | Cheap moment | Look first | A hit goes to |
|---|---|---|---|---|
| A request | its vocabulary | before a design exists | behind | the design gate |
| A bug report | the report's wording | the third failed fix | under | back to reproduce, as a hypothesis |
| A plan | its author's unwritten judgments | before task one | under, then against | the plan, never the task |
| A design or spec | the first approach proposed | before the yes | against, beside | the gate |
| A decision or ADR | its dated premises | when relied on | ahead, read backward: what changed since | `revalidating-decisions` |
| A number, report, or dashboard | the instrument | before acting on it | under | a tier mark, per `calibrating-confidence` |
| A document | the reader it assumed | at *frame the reader* | beside, ahead | the draft |
| A codebase or pattern | "how it is done here" | the fourth special case | above, against | the record, or an ADR |
| A project | its charter vs. its behaviour | the orbit moments | around, then all eight | the orbit record |
| A conversation | your own previous turns | a conviction grown without evidence | under | said out loud, once |
| The run itself | the track and envelope chosen at the start | a stop condition, or a third failure of one shape | against, under | the phase log, as an envelope change |

Two rows carry a warning. A hit on **a codebase** is an observation about the pattern, routed to the record — never a refactor started from inside a sweep. A hit on **the run itself** can only ever be an envelope change written down — never a silent downgrade, because the part of a run that most wants to be finished is the part reading this.

## The eight directions

Where to look. Each is a direction *from* the object, not a technique — and each has a tell, because running all eight every time is furniture, and a skill that fires constantly gets ignored precisely when it matters. [reference/directions.md](reference/directions.md) has each one in full: its question set, its characteristic yield, and the specific way each one fails.

| Direction | Looks at | The tell that picks it |
|---|---|---|
| **Behind** | The goal the request is already a solution to | The request names a mechanism rather than an outcome |
| **Under** | The assumptions its own vocabulary smuggles in | A third fix did not hold, or the bug moved instead of dying |
| **Above** | The class this is one instance of | This is the second or third time something of this shape has come up |
| **Beside** | What becomes possible, trivial, or broken once this exists | The change touches a boundary something else depends on |
| **Ahead** | The second-order consequence, and who inherits it | The work is easy now and permanent afterwards |
| **Against** | The inversion — delete it, do nothing, do it by hand, make it impossible | The design is accumulating special cases |
| **Outside** | What a different discipline calls this, and whether it is solved there | The problem feels novel, which it almost never is |
| **Around** | The field — who has a stake, what forces act, why the world sent this now | More than one owner, or a date, budget, or audience the request never named |

They are not orthogonal and are not meant to be — *behind* and *under* both sit upstream of the request, *above* and *beside* both look outward from it, and a single finding often arrives through two at once. The tell is what does the selecting, not a taxonomy; a finding that came in through the wrong door is still a finding.

**Behind** is the highest-yield and the most dangerous, and both for the same reason: it produces the strongest results and it is the one that turns you insufferable. A recovered goal is a **hypothesis about someone else's intent**, and it gets stated as one — "is this about X, or about Y?" — never as "what you actually want is". You do not have privileged access to what they meant. You have noticed that their words admit two readings, which is a fact about the words.

**Outside** is the one that gets skipped as decorative and is not. On this skill's own creation it was the direction that paid best; the worked record is in [reference/self-application.md](reference/self-application.md).

**Around** is the one a view from above is mostly made of. The seven before it are about the problem; this one is about the room it was asked in — the people who will decide what happens to any answer, and the incentives, deadlines, and habits acting on them. Its failure is the one to watch: motive is not observable, so an *around* finding stays with what is — the metric, the date, the org chart, the message — and phrases the rest as a question.

## The seven moves

The directions say where to look. These say how. Each one breaks a different kind of frame; the fifth turns what the others find into something you can check. They are mechanical on purpose — "think differently" is not an instruction anyone can follow, and a creativity prompt that cannot fail cannot help.

**Restate it without its own nouns.** Take the request and say it again using none of the words it arrived in. "Add a retry to the uploader" becomes "make a transfer that sometimes stops partway end up complete." Two things fall out. If you cannot do it, you have not understood the problem yet and the next hour was going to be expensive. If you can, the alternatives are usually visible in the restatement — *resumable* was not in the original sentence, and now it is the obvious answer. The vocabulary was doing the constraining, and this is the cheapest way to find out.

**Check your repertoire.** This is the direct answer to "take a path you have not taken", and it works because your unexplored paths are not random — they are the same ones every time. Name the move you are about to make and the family it belongs to. Then name a family you have not sampled. The habitual families are recognisable: read more, add a layer, add a flag, retry harder, extract a function, ask the user. The ones reliably left unsampled are: **delete it, invert it, move the decision earlier, change who does it, make the bad state unrepresentable instead of checking for it, do it once by hand and find out whether it recurs.** Picking from the second list is not a guarantee of a better answer. It is a guarantee that the answer was chosen rather than defaulted to.

**Audit the constraints.** List what the request treats as fixed, then ask of each one: *who would have to agree to lift this?* Three answers, and only one is a constraint. "Physics, or the standard, or the regulator" — real, design inside it. "A named person or team" — negotiable, and the cost of asking is usually one message. "Nobody, it is just how we have been doing it" — that was never a constraint, and it has been shaping the design anyway. `revalidating-decisions` owns the case where the constraint is a recorded decision with a premise that has since expired.

**Find the contradiction.** When a request seems to demand two incompatible things and the instinct is to split the difference, the trade-off is usually the frame's shadow rather than a law. State both requirements in their strongest form and ask what would have to be true for both to hold at once. Sometimes nothing, and you have learned the trade-off is real and can defend it. Often enough, the answer is a third arrangement in which the conflict does not arise — and it was invisible because the frame presented the two as endpoints of one dial. This is the move that most often produces something genuinely new, and it is the one nobody reaches for, because splitting the difference always looks reasonable.

**Name the observation that would tell them apart.** The moves above produce competing readings, and the reflex is then to argue for the better one — which is how a sweep turns into an opinion nobody can check. Do not argue. Ask instead: *what could I look at that would come out differently depending on which reading is true?* One grep, one log line, one question, one file. Then go and look.

This is the move that makes the whole skill a technique rather than a sensibility, and it is borrowed rather than invented — intelligence analysis builds its entire method on it, on the grounds that evidence consistent with every hypothesis is evidence for none of them. The same applies here. A reading that nothing could distinguish from its rival is not insight, however well it is put, and noticing that early is worth more than winning the argument. When the discriminating observation is cheap, the sweep stops producing hypotheses and starts producing findings; when there is none, say so and mark the reading as an inference, per `calibrating-confidence`.

**Change altitude.** Say the problem one level up and one level down from where it was stated — the line, the module, the system, the product, the world it operates in — one sentence each. A problem is rarely cheapest to solve at the altitude it arrived at: the uploader's retry is a line-level fix to a system-level property, and the "make everything idempotent" that a system-level reading suggests is sometimes a one-line change at the edge. The finding is the altitude where the sentence is shortest. `deciding-reversibility` decides at the right altitude; this is how you find out which one that is.

**Run the deviation guidewords.** For each parameter of the thing — every input, output, timing, quantity, and actor it has — apply each of a fixed list and describe the case it produces: *no, more, less, as well as, part of, reverse, other than, early, late, before, after.* Process safety built this as a way of finding hazards nobody would have imagined; it is also the most reliable generator of ideas in this file, because the guideword does the generating and the parameter does the grounding, and neither of them is your imagination. "The upload arrives *early*" and "the upload arrives *as well as* its own duplicate" are cases the request never contained and two designs it silently excluded. `auditing-new-input-categories` and `property-based-testing` turn the cases into checks.

## Look for what is absent

Everything above finds things that are wrong. The rarer, harder sight is finding what is **missing**, and it is harder for a structural reason: absence is invisible to inspection. You cannot review your way to it, because review examines what is in front of you and this is not. Economics has a name for it — *the seen and the unseen* — and the discipline it names is the same: for every effect you can point at, name the one you cannot.

The instruments read green here. The tests pass, the diff is clean, the ledger is ticked, and none of them can report a case nobody wrote, a state nobody modelled, a caller nobody updated, a reader nobody considered. Green is a statement about what was checked. It is silent about the shape of what was not.

Four questions that actually surface absence, because they do not start from the artifact:

- **Enumerate from the domain, not the code.** List every state the *thing itself* can be in — every input category, every actor, every lifecycle stage — and then find each one in the work. The ones with no counterpart are the finding. `auditing-new-input-categories` and `modeling-state-machines` own the enumeration; this is why you run it.
- **Ask who is not in the room.** Every request is written from one seat. The operator, the person who inherits this at 3 a.m., the reader with no context, the caller you did not write, the second tenant — one of them is usually absent from the design entirely, and their absence is not visible from inside it. `seat <chair>` runs one of these on demand.
- **Ask what would have to exist if this were true.** If the stated cause is real, what else would show it? A cause with no corroborating trace anywhere else is a story, not a diagnosis — `diagnosing-before-fixing` runs this on bugs specifically, and it generalizes. This is the fifth move aimed at a cause rather than at a reading, and the absence it finds is the corroboration that never turned up.
- **Ask what was not asked.** People in the requester's position usually ask a certain set of things, and a request unusually short for its consequences has compiled some of them out because they were obvious to the person writing. The unasked question is the requirement nobody will ever state.

`reviewing-code-deeply` applies absence-hunting to a diff, which is the same instinct one level down. This applies it to the request.

## Lenses: instruments borrowed from other fields

A direction says where to look from. A lens is a specific instrument to look with — a question some other discipline turned into a standard technique because its practitioners kept being blind in the same place. [reference/lenses.md](reference/lenses.md) holds about thirty of them, filed under the blindness each cures, because that is how you pick one: name what you are probably not seeing, then take the lens built for it.

| You are probably blind to | Because | A lens built for it |
|---|---|---|
| The smallest possible design | the request's words named a mechanism | ideal final result; the job it is hired for |
| One of your own assumptions | the first thing seen became the model | key assumptions check; quality of information check |
| The wrong size | a number nobody chose is in the design | ten times, one tenth; population one; altitude ladder |
| The class this belongs to | the instance was taken as unique | the reference class by shape |
| What is changing under you | the present was assumed permanent | why now; pace layers; pre-mortem; indicators |
| A seat nobody sat in | every request is written from one chair | the empty chairs; show me the incentives; the mirror of the organization |
| The whole | a part was mistaken for it | the purpose of a system is what it does; leverage points; flows across the boundary |
| What is not there | absence is invisible to inspection | the seen and the unseen; deviation guidewords; the unasked question |
| The instrument | the number arrived with authority | Goodhart; the ruler measures the hand; the base rate |
| Buy versus build | the two were never distinguished | the evolution axis |

The catalogue is not a checklist. Running every lens is furniture; the tell for each is what selects it, and most sweeps use one. Where this library already holds the technique in full — `threat-modeling` for attackers, `revalidating-decisions` for a premise that expired, `mapping-dependencies` for blast radius — the lens is the reason to open that skill, not a substitute for it. `lens <name>` runs one by hand.

## Orbit: the whole from above

An `orbit` is the view of a whole project from outside it: one bounded page that says what it is at five altitudes, what it actually does against what it says it is for, who sits in its chairs and which are empty, what crosses its boundary, what is changing under it and at what pace, what it has never had, and which forces will decide what happens to any proposal made about it. It is what the request for a God's view is actually asking for, assembled from seven partial seats rather than one omniscient one. [reference/orbit.md](reference/orbit.md) has the panes, the lens each is made with, the output shape, and the moments.

Its bounds come before its method, because this is where the skill is most dangerous:

- **One page**, seven panes of one to three sentences each. **At most three findings**, each carrying the observation that would confirm or refute it, each routed like any other.
- **At named moments only** — on arrival at inherited work after orientation, before a milestone decision, when stuck three times at the same place, in a postmortem, or on request. Never a mode, never during an incident.
- **Never a redesign.** Anything found above the level of a rule — a goal, a paradigm, "this should be a different project" — goes to the record and the closing offer. `managing-scope-drift` wins hardest here.
- **An orbit that finds nothing is a project that is what it says it is**, and that is a result. An orbit is most valuable the second time it is run: the diff between two orbits is the project's actual direction, which no single one can show.

## Imagine: ideas as observations

The skill was built as a diagnostic rather than a brainstorm, and it stays one. But a request for ideas is a real request, and the same machinery answers it once one thing is settled: **an idea is an observation about the possibility space**, and it routes exactly like a finding about the request — to the record, mostly, and to the closing offer, where the person decides which of them becomes work.

An `imagine` pass is bounded divergence. Pick the lenses built to produce candidates rather than to check them — ideal final result, ten times and one tenth, the inversion forms, an analogy from a *named* discipline, the deviation guidewords, a pre-mortem inverted into what would have prevented it — and run three or four of them once. Stop at five candidates. For each, write two things: the idea in one sentence, and the observation that would tell whether it is worth anything. A candidate with no such observation is a wish, and it is discarded with that reason. Then rank by what it would cost to find out, and deliver the list under one question, per `offering-the-next-move`. Nothing on it is built from inside the pass.

What this is not: `steelmanning-alternatives` chooses between options and recommends; an imagine pass produces the options it then works on. It is not a way to make a dull task interesting — its seats are few: the design gate, an emptied hypothesis list, the frame-the-reader moment, and the search for the operation `against` wants instead of a migration; [reference/track-seats.md](reference/track-seats.md) closes it everywhere else. And the unexplored path is worth *considering*; it is never worth *choosing* on that ground.

## Sizing the sweep

A sweep is one bounded pass at a named moment. Not a mode you enter, and never a background hum — the moment it runs continuously it becomes furniture, and furniture gets read past.

Size it by the run's amplitude, from `orchestrating-work-end-to-end`, and pick by the tells rather than by working down the table:

| Amplitude | Sweep | Which | Orbit and imagine |
|---|---|---|---|
| `express` | One direction, one question | The one its tell selects. No tell fires, no sweep | Neither, unless asked for by name |
| `standard` | Three directions | Whichever tells fired, capped at the two strongest, plus **against**, which is cheap and needs no tell | An imagine pass stops at three candidates; no orbit |
| `campaign` | All eight, once, at the design gate | And once more at close, restricted to **ahead** and **beside**, feeding the offer | One orbit at arrival if the work is inherited; an imagine pass stops at five |

One pass. If a second sweep is genuinely warranted it is because something changed — a gate failed, a premise died, the track moved — and that change is what justifies it, not the feeling that there is more in there. There is always more in there. That is not evidence. `knowing-when-to-stop` owns the general form of that judgment, and this is its sharpest case: a sweep has no natural stopping point at all, because the space around a request is unbounded and every pass through it can be made to produce something. An orbit is that case squared.

## Where it sits in a run

Clairvoyance has a seat in every track `orchestrating-work-end-to-end` routes to, and the seat is different in each. One row below is a prohibition rather than an invitation, and the envelope list underneath adds the other. [reference/track-seats.md](reference/track-seats.md) has the detail: the entry moment, the direction that pays there, what a hit routes to — and the far fewer seats an orbit or an imagine pass gets.

| Track | The moment | If it hits |
|---|---|---|
| **Build** | Between frame and design, before the first approach is proposed | Back to the gate — the design has not been written yet, so a reframe costs a paragraph |
| **Diagnose** | When the hypothesis list empties, or the third fix does not hold | **Under**. The assumption is in the reproduction, not in the patch |
| **Respond** | **Not during mitigation.** At stabilize, and again in the postmortem | Nothing may delay stopping the harm. A frame question with users broken is the "let me understand this properly first" red flag, exactly |
| **Investigate** | Before the conclusion is written | The question asked may not be the question that matters. Report both, and mark them per `calibrating-confidence` |
| **Review** | On coverage, before judgment | The absence pass above. What the diff does not touch is the finding a review most often misses |
| **Author** | At *frame the reader*, and again before verify | **Beside** and **ahead** — who else reads this, and what this document makes impossible to say later |
| **Change in place** | Before `deciding-reversibility`, not after | **Against** first: the cheapest migration is the one you do not do because the thing can be deleted |

The envelope constrains it too, and one of those is a hard stop:

- **`autonomous` or `unattended`** — a sweep may run; a reframe may not be self-approved. Changing what is being built, with nobody there to agree, is a decision to build something nobody asked for, and confidence does not convert it. Write the observation, the reading that lost, and a stop condition; then continue on the original frame. An orbit's findings and an imagine pass's candidates go to the record only. `bounding-autonomous-work` owns the substitution.
- **`live` blast radius** — **ahead** is not optional. The second-order consequence is the whole reason the radius is live.
- **`shared tree`** — **beside** first, and aimed at the other writer. What you are about to do that they cannot see is the finding.
- **`campaign`** — the discard pile goes in the record. Its whole value is that the next session does not spend an hour re-deriving it and discarding it again.

## Delivering sight

A sweep's output reaches a person, and this is where an otherwise good pass turns into something nobody wants to receive again.

**One question, not a monologue.** The deliverable is not the sweep. It is the single thing that would change what happens next, phrased as a choice, through the harness's own question tool — `offering-the-next-move` owns that discipline and this feeds it. Four observations delivered as a paragraph is a lecture; the best one delivered as a question is a decision they get to make. An imagine pass delivers its ranked list the same way: the candidates under one question, never five questions. Where nobody is there to receive it, the question does not evaporate: it becomes a line in the run record and a line in the report, phrased as the choice it would have been, so that whoever reads the run gets to make it late rather than never.

**Say it once.** A reframe declined is a reframe answered. Raising it again in different words at the next phase boundary is not diligence, it is not taking the answer. Record the ruling and build the thing they asked for, properly.

**They own the scope.** Sight is yours to produce and theirs to spend. "I see why you'd frame it that way, do it as asked" is a complete and correct outcome, and it goes in the record as a ruling — not as a defeat, and not as something to relitigate when the approach gets hard later.

**Attribute the uncertainty.** A sweep produces hypotheses, mostly about intent and consequence, which are the two things you have least evidence about. Mark them as such. A reframe presented with the confidence of a measurement is the fastest way to get every future sweep discounted — and an orbit pane that names what someone *wants* rather than what they are measured on has crossed from sight into story.

## The null is the normal result

Most sweeps find nothing, and that has to be a reportable, unremarkable outcome — or the sweep will manufacture something to justify having been run. This is not a hypothetical failure. A pass that is rewarded for finding things finds things.

Most requests mean what they say. Most frames are correct, which is why they became the frame. A sweep that comes back empty is reported empty, in one line, and it is the pass working rather than the pass failing — the same way a coverage check that finds no gap is a real result. The number to be suspicious of is not zero. It is seven. An orbit with seven panes of description and no finding is a project that is what it says it is; an imagine pass with five candidates and no discriminating observation for any of them has produced wishes.

Two consequences. **An empty sweep is one line, not a paragraph explaining what you looked for.** And a run whose every sweep finds a reframe is not perceptive; it has learned that finding things is how the sweep gets approved of.

## Commands

Invoked directly, this skill takes a verb. With none it runs `sweep`, which is the whole method: size by amplitude, select by tell, route everything it finds.

| Command | Does |
|---|---|
| `sweep [target]` | The default. Name the frame and its sources, run the directions the tells select, route every observation. Ends by naming what it found, or by saying it found nothing |
| `orbit [project]` | The whole from above: seven panes, one page, at most three findings, at a named moment. [reference/orbit.md](reference/orbit.md) |
| `imagine [target]` | Bounded divergence: three or four candidate-producing lenses, at most five ideas, each with the observation that would tell whether it is worth anything, ranked by the cost of finding out and delivered under one question |
| `lens <name>` | One instrument from [reference/lenses.md](reference/lenses.md), by name, applied to the thing in front of you |
| `seat <chair>` | One look from a named seat — the operator at 3 a.m., the auditor, the integrator, the customer's customer — at what that person notices first. The roster is at the end of the lenses file |
| `behind` · `under` · `above` · `beside` · `ahead` · `against` · `outside` · `around` | Run that one direction regardless of what the tells say. Useful when you already know which way to look, and as a second opinion on a sweep that came back empty from a direction you doubt it really tried |
| `repertoire` | The move on its own: name what you are about to do, name its family, name a family you have not sampled. Cheapest thing here and the one worth running most often |
| `route` | You already have an observation and need it placed. Run it through the four-row table and write the line it earns — including the discard, with its reason |

`sweep`, `orbit`, and `imagine` are bounded by the amplitude table. The single-direction verbs, `lens`, and `seat` are deliberately not, because asking for one by name means a tell already fired for you.

## Refuse

Category defaults, not bans. Each can be right in a specific run — reaching for one when the alternative was available means the sweep was performing rather than seeing.

- A sweep on a request whose frame you have not stated. You cannot look outside a box you have not drawn.
- Eight directions on a two-line change.
- A reframe delivered as a correction to what the person meant.
- An observation acted on without passing the routing table.
- A sweep run because the work inside the frame turned out to be hard or boring. That is avoidance with a method attached, and it feels identical from the inside.
- A finding kept because it is interesting. Interesting is not a route.
- A second sweep on the same frame with nothing changed in between.
- A sweep, or an orbit, during an active incident, before the harm has stopped.
- Novelty preferred because it is unexplored. The unexplored path is worth *considering*; it is not worth *choosing* on that ground.
- An orbit as a mode, a habit, or a page longer than one.
- An idea delivered without the observation that would test it. That is a wish.
- A lens run because it is in the catalogue rather than because its tell fired.
- An *around* finding about what someone wants. Motive is a question, never a finding.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A brilliant reframe, delivered after the design was approved and built | The sweep ran at review instead of before design, which is where it was free |
| Every sweep produces three findings | The pass is being scored on yield, so it finds what it needs to find |
| The user got a paragraph of alternatives they did not ask for | The sweep's output was published instead of routed; only one of four routes ends at them |
| A sweep found the real problem and the run built it anyway | Routed to the record when it belonged at the gate — or the gate was skipped because reopening it felt expensive |
| The same reframe raised three times in one run | It was declined, and declining was read as not-yet-understood |
| Seven directions came back empty and the eighth was stretched to say something | The null rule was never accepted; empty was treated as a failed pass |
| An unattended run quietly built something other than what was asked | A reframe was self-approved. The one thing this skill's output cannot do without a human |
| The sweep became the work | No bound. One pass at a named moment is the whole shape; a mode has no exit |
| Nobody can say why an interesting finding was dropped | Discarded with no reason, so the next session re-derives it and drops it again |
| The alternatives all had the same shape | The frame was never named, so every option was generated inside it |
| Two readings of the request were argued for a page and neither won | No discriminating observation was named, so there was nothing either reading could have failed |
| The sweep was excellent and the thing that was asked for never shipped | A reframe is not a delivery. The original ledger stayed owed the whole time |
| The orbit proposed a different project | A finding above the level of a rule was routed to the gate instead of the record |
| Five ideas, all plausible, none checkable | The imagine pass skipped the discriminating observation; it produced wishes |
| A plan was swept task by task and nothing was found | The object was wrong: the frame is in the plan's judgments, and a task cannot see them |
| A number was acted on, and the instrument had never run | The measurement was swept as a request would be, without a measurement lens |

## Red flags

- "What you actually want is..."
- "While we're in here, this whole approach is wrong."
- "Let me think about this more broadly first" — with users currently broken.
- "This is a really interesting adjacent problem."
- "They said no, but I don't think they understood the implication."
- "The sweep didn't find anything, let me look harder."
- "It's technically out of scope, but it's obviously better."
- "I'll just do the reframed version, it's the same amount of work."
- "Nobody's around to approve the reframe, so I'll use my judgment."
- "I know why they asked for this" — said about a person who is available to be asked.
- "The interesting finding here is really the deeper question" — said with the deliverable unfinished.
- "Both readings are defensible, so let me lay out the case for mine" — with no observation named that either one could have failed.
- "From a God's-eye view, this project should really be..." — one seat, claiming to be all of them.
- "Here are twelve ideas" — none of them with a way to find out.
- "Let me run the full orbit before I touch this" — on a two-line change.
- "They clearly want..." — about someone's motive, in an *around* finding.
