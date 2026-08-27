---
name: offering-the-next-move
description: Use at the end of every task that produced something — a change, an artifact, an answer someone will act on — and before the turn that reports it ends; when a run is about to close with a summary and a general invitation to say what's next; when the ledger holds a surrendered line, the record holds a ruling that could have gone the other way, or something turned up in passing and was never pursued; when the options in hand would fit any task of this shape; or when landing is about to tear down the plan and record those options would have come from. Covers harvesting options from what the run already wrote down instead of inventing them, ranking by what it costs the reader not to be asked, the anatomy of a real option and the four never to offer, recommending without hedging, one question rather than four, and routing the answer back through the track selector.
---

# Offering the next move

## Overview

The end of a run is the moment you know the most you will ever know about what should happen next, and the moment that knowledge is most likely to be thrown away. The ledger holds a line that was surrendered. The record holds a ruling that could have gone the other way. Somewhere in the transcript is the thing you found on the way past and did not chase. None of it is in front of your human partner, who gets a summary and an invitation to think of something.

So end with the choice already assembled. Not because asking is polite — because the option set is itself a deliverable, built from information only this run has, and a summary discards it. The failure this exists against is not forgetting to ask. It is asking generically: a menu derived from the *kind* of work rather than from this instance of it, which looks like a considered handoff and carries none of the run's actual information.

## When to use

- Any task that produced something is ending — a change, an artifact, an answer someone will act on.
- An orchestrated run has reached landing, or an investigation is about to file its report.
- The draft ends in prose plus a general invitation to say what comes next.
- The options in hand would fit any task of this shape, which means they came from the category rather than from the run.
- Not for: the questions that make the work possible in the first place. `scoping-before-building` asks those one at a time, before anything is built; this runs after, and asks what comes next rather than what this is.
- Not for: naming what is unfinished. `knowing-when-to-stop` owns the explicit handoff and what it owes the reader — this takes that named remainder and makes it selectable.
- Not for: how to write the summary the options sit under, which is `explaining-technical-work`.

## Always, and what always means

Every task that produced something ends with an offer. The size of the task is not an exemption: a two-line fix that turned up a second instance of the same bug has more to offer than a week of implementation that turned up nothing.

The boundary is what the turn produced, not how long it took. An exchange that changed nothing is not a run and closes normally. Everything else gets the offer.

A run that failed or stalled gets one too, and it is where the offer is worth the most. What blocked it, what would unblock it, and what is worth doing instead are three real options that nobody outside the run can assemble. `knowing-when-to-stop` covers stopping while blocked; this is what that stop hands over.

There are four ways the rule gets talked out of, and none of them holds:

| The thought | Why it does not hold |
|---|---|
| "There is obviously nothing left to do." | Then the offer is short and one option says exactly that — and your partner gets to disagree with "obviously," which they cannot do with a sentence you decided not to write. |
| "They will tell me what they want next anyway." | They will, from a blank page. You are the one holding the list of what this run turned up. |
| "It is a small task; a menu would be ceremony." | The offer's size tracks what was found, not what was asked. Padding a thin harvest out to four options is the real ceremony — offer the two that are real. |
| "I already listed the next steps in the summary." | A paragraph is not a set of choices. The reader still has to re-read it, extract the forks, weigh them, and compose a reply. That is the work the offer exists to have already done. |

One objection is not a rationalization and deserves a real answer. A harness that ships a question tool usually tells you to reserve it for decisions you cannot resolve yourself — do not ask what you could infer, do not ask what has an obvious default. That rule governs questions that *block* work, where interrupting to ask something you could have looked up spends your partner's attention to save your own. This is the opposite moment: the work is finished, nothing is waiting on the answer, and the question is not what you should do but which of what this run turned up they want. The first rule does not reach the second, and applying it there does not avoid an interruption — it withholds information.

## Harvest, do not invent

Options are not generated at close-out. They are collected, from things the run already wrote down, in this order:

| Rank | Source | Where it already lives | Why it ranks here |
|---|---|---|---|
| 1 | A surrendered acceptance line | the ledger | Their own request, unmet. Nothing you noticed on your own outranks something they asked for and did not get. |
| 2 | A ruling that could have gone the other way | the run record's rulings | You chose; they never got the choice. The branch not taken is visible only from inside the run. |
| 3 | Scope pushed out on purpose | the phase log, `managing-scope-drift`'s record | Out of scope *for this run* and out of scope are different questions, and only one of them has been answered. |
| 4 | Something found and not pursued | the transcript, and nowhere else | Adjacent breakage, a second instance of the bug just fixed, a comment that turned out to be live. Highest decay of anything here: not offered now is lost. |
| 5 | The obvious continuation | the plan's remaining tasks | Weakest slot on the menu. They would have asked for this themselves, so it costs them nothing not to be asked. |

The ranking rule falls out of the table: **would they have thought of this without you?** If yes, it is a weak option. A menu built entirely of row 5 is the generic menu — technically responsive, informationally empty. Row 4 is where the value concentrates, for the same reason it decays fastest: the option nobody else could have offered is the one with no home in any file, and it lives exactly as long as this context does.

**Harvest before the teardown.** Landing deliberately retires the plan and the run record. Assemble the offer while they still exist; tearing down first and then trying to recall what was in them is how rows 1 through 3 quietly become row 5.

## What a real option looks like

An option is an action with a size, not a topic. "Add tests" is a topic — the reader cannot tell what they would be agreeing to. "Cover the retry path — one test, the case the review flagged and nobody closed" is an action they can price.

Two fields doing two different jobs:

- **Label** — the action, in a few words, in verb form. It is read first and is sometimes all that is read.
- **Description** — what it costs, and what changes if they decline. Never a longer restatement of the label; that is the most common way a menu becomes decorative.

| Weak | Real |
|---|---|
| **Refactor the parser** — the parser would benefit from some restructuring for maintainability | **Split the parser's two jobs** — about an hour, and it leaves the file the next schema change has to touch in one piece instead of two |
| **Fix remaining issues** — address what is left | **Handle empty input** — the one acceptance line surrendered at proof; ten minutes, and it is the input their importer actually sends |

## The four never to offer

1. **Work inside the scope already agreed.** "Should I also fix the test I broke?" is not a fork, it is an unfinished task in a question's clothing, and offering it launders your own incompleteness into their decision. Finish it, then offer.
2. **A decision you are better placed to make.** You have the code in front of you and they do not. Deciding, then saying what you decided and how firmly, beats handing back a judgment that was yours — `calibrating-confidence` covers marking it honestly.
3. **Anything you would not actually do if it were chosen.** Every option is a commitment. A decoy spends a slot and buys a second round of asking.
4. **The same option twice.** Two phrasings of "keep going" beside one "stop" is a two-option menu padded to look like three. Padding is the tell that the harvest was thin, and a thin harvest is a short menu, which is a fine thing to be.

## Always leave a way to stop

One option always ends the work, and it is a position rather than a leftover. "Ship as is — the surrendered line only affects an input path nothing reaches yet" tells them something they did not have: your read on whether stopping here is defensible. "Nothing further" tells them nothing.

A menu whose every option is more work is a menu manufacturing the work it offers, and it reads as pressure however carefully the options were harvested. The stopping option is what makes the rest of them an offer rather than a queue.

## Recommend, do not hedge

Lead with the option you would take, marked as the recommendation. A menu with no recommendation is unfinished thinking handed over as though it were a choice: you read the code, sat through the failures, and saw what the run turned up, so declining to say what that adds up to withholds the most useful thing you have.

A recommendation is a claim and takes a claim's treatment. One you would abandon at the first sentence of pushback says so in its own description. Offering it at the same weight as one you would defend does not remove the hedge — it moves it into the silence around the recommendation, where the reader cannot see it.

## One question, not four

Close-out is one question by default. Two is an interrogation at the exact moment your partner is deciding whether this is finished.

A second question is legitimate only when both of these hold:

- **The answers are independent.** If the second only makes sense given one particular answer to the first, it is a follow-up rather than a second question, and asking it now forces them to answer a hypothetical. Wait, then ask.
- **Both answers change what you do.** If either answer leads to the same next action, you are collecting a preference, not a decision.

The most common legitimate second axis is *what next* alongside *how this lands* — genuinely independent, and both change the next action.

## Mechanics

Your harness probably has a structured question tool under some name — `AskUserQuestion`, an elicitation call, a prompt with selectable options. Use it. Options written into prose are options the reader has to retype.

What holds across implementations, and what each one costs you:

| Constraint | Consequence |
|---|---|
| A handful of options, not a list | The ranking above is triage rather than tidiness — most of what you harvest will not fit |
| Labels are short, and the header shorter still | Put the action in the label and everything that costs or persuades in the description |
| Free text stays reachable regardless | Never spend a slot on "something else"; it is already there and the slot is not |
| One selection unless the tool is told otherwise | Options that are not mutually exclusive need the multi-select flag, or they need to be one option |

Where no such tool exists, none of the content rules change: a numbered list, one line each, and "reply with a number." Where nobody is listening at all — a headless run, a scheduled job, a subagent reporting to a controller rather than to a person — the offer goes into the report as that same ranked list and the run ends. It still gets assembled; it is simply delivered to whoever reads the report instead of to a prompt no one will see. The widget is the delivery. The harvest, the ranking, and the recommendation are the skill.

## Where the answer goes

A chosen option is a new request, not a resumption. Run it back through the track selector in `orchestrating-work-end-to-end` — a finished run does not confer its track on whatever follows, and a chosen option that reopens a surrendered acceptance line routes *backward* to the phase that owns it rather than forward from wherever you stopped.

The offer goes into the run record before it is retired, because it is part of what the run concluded. The choice usually arrives after that — in a later turn, or never — so it belongs to whatever comes next rather than to the record being closed. Either way the unchosen options are the ones worth keeping: they are the record of what was deliberately not done, for the same reason a skipped phase gets a line saying why. Where landing deletes the record, they move into whatever outlives it — the handoff note, the pull request description, the message to whoever picks this up next.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The menu would fit any task of this shape | Options generated from the category of work; the harvest never ran |
| Four options, two of which mean the same thing | A thin harvest padded to fill the widget instead of offered short |
| The reader picks the free-text option nearly every time | The real fork was never on the menu — rows 1 through 4 were skipped in favor of row 5 |
| An option turns out to be something you were already obliged to do | Scope already agreed, handed back as though it were a choice |
| Every option on the menu is more work | No stopping position, so the menu manufactures the work it offers |
| Descriptions restate their labels at greater length | The description's real job — cost, and what declining changes — went unwritten |
| The offer was assembled after the record came down | Harvested from memory; rows 1 through 3 had already been deleted |
| The chosen option runs as a continuation of the finished run | A new request inherited a track instead of being routed |

## Red flags

- "I'll summarize and let them tell me what they want next."
- "There's obviously nothing left to offer."
- "This was too small to bother asking about."
- Filling the fourth slot because there is a fourth slot.
- A recommendation withheld because both options seem fine — they seem fine to the one person who watched the run.
- Offering something you would argue against if they picked it.
- "I listed the next steps in the summary" — a paragraph they have to re-read is not a choice.
