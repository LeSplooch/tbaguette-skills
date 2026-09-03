# The run envelope: five dials

The track says what shape the work is. The envelope says what the run may
assume about the world it executes in. Both get read at the same moment and
neither substitutes for the other: an autonomous express run in a sandbox and
an autonomous campaign against live data share a track and nothing else.

Read all four dials before the first action, and write them on the run
record's first line. Each has a default that is right often enough to be
dangerous — a default that is usually right is a default nobody notices
being wrong.

## Contents

- [Presence — who answers a gate](#presence--who-answers-a-gate)
- [Amplitude — how much run this is worth](#amplitude--how-much-run-this-is-worth)
- [Blast radius — what a wrong turn costs](#blast-radius--what-a-wrong-turn-costs)
- [Crew — who else is writing](#crew--who-else-is-writing)
- [Register — what the run spends](#register--what-the-run-spends)
- [Reading the four together](#reading-the-four-together)
- [Failure modes](#failure-modes)

## Presence — who answers a gate

| Value | You know it by | What it does to the run |
|---|---|---|
| **paired** | The human answered the last thing you said, within a turn or two | Gates are asked, one question at a time. Questions are cheap here and nowhere else |
| **async** | A human exists and answers in hours — a review comment, a ticket, a chat that goes quiet overnight | Gates batch into one message; the run continues on everything that does not depend on the answer, and parks what does |
| **autonomous** | The run will finish before anyone reads a word of it — a delegated task, a plan handed over whole, a goal with a leash | Every gate that was a question becomes a written self-answer carrying a stop condition. See `bounding-autonomous-work` |
| **unattended** | Nobody is reading the output live at all — a hook, a cron, a loop, a CI step | Autonomous, plus: the run must be idempotent, must leave a trail a stranger can act on, and must not depend on state it did not itself create |

The line between `async` and `autonomous` is not how long the wait is. It is
whether the answer arrives **before this run ends**. A question that will be
answered tomorrow is `async` if the run can park and wait, and `autonomous` if
the run must finish tonight — the same question, two different pieces of work.

Presence is the dial that changes underneath you. A `paired` run becomes
`async` the moment the replies stop, and `async` becomes `autonomous` the
moment you decide to finish rather than wait. Both transitions are decisions
and both belong on the record; see the envelope-change table in `SKILL.md`.

## Amplitude — how much run this is worth

| Value | The work | The run |
|---|---|---|
| **express** | The target is already known, the change fits one reviewable diff, and a test either exists or can be written in the same breath | Four beats, not eight. See [express.md](express.md) |
| **standard** | A feature, a bug, a migration — days at most, one context window at a time | The spine as written |
| **campaign** | The work outlives the context that started it: many sessions, many hands, or a plan long enough that nobody holds it | The spine, plus checkpoint discipline and a record that is the only memory. See `checkpointing-long-runs` |

Amplitude only ratchets **up**. Express promotes to standard the instant an
entry condition turns out false; standard promotes to campaign the instant the
context that holds the run gets close to full. Nothing demotes, however small
the remainder looks in hindsight — the reason is the same one the track rule
gives: the judgment that says "this got smaller" is made by the part of the run
that most wants to be finished.

The exception, and it is the only one: **your human partner can demote.** "Just
make the edit, skip the ceremony" is an instruction. It goes on the record
named and attributed, so the run still says what process it actually ran.

## Blast radius — what a wrong turn costs

| Value | Where the work lands | Gate it adds |
|---|---|---|
| **sandbox** | A throwaway worktree, a scratch branch, a local database, a dry run | None. This is what isolation is *for* — spend the freedom |
| **repo** | Commits, branches, a pull request. Everything is recoverable by someone with the history | Nothing lands without its own verification actually run |
| **live** | A deploy, a migration against real data, an outbound message, a published artifact, a rotated credential, a package release | `deciding-reversibility` before the action, not after. One-way doors get a human — in every envelope, including autonomous ones |

Blast radius is a property of **the action**, not of the run. A run that has
been `sandbox` for six phases becomes `live` at the single step that pushes,
and it is exactly then — one command from the end, with everything green — that
the check gets skipped. Radius is re-read before every irreversible action, not
once at the start.

Two undercountings are worth naming because both feel like `repo` and are not:
a push to a branch that something deploys from, and any write to a checkout
another agent or person is also holding.

## Crew — who else is writing

| Value | Situation | What it changes |
|---|---|---|
| **solo** | You are the only writer in this tree | Nothing |
| **fanned** | You dispatched subagents that write in parallel | `fanning-out-independent-work` owns the split; every task's boundary is a file boundary, and the merge is yours, not theirs |
| **shared tree** | Another agent, another session, or a person is editing the same checkout | Never stash, never reset, never assume the tree you read is the tree you are about to commit. Stage by hunk, re-read before writing, and prefer a worktree of your own |

The tell for `shared tree` that gets missed most: a diff far larger than the
work you did. A checkout that suddenly reports changes across files you never
opened is not a broken tool, it is a second writer, and the next destructive
git command is about to cost someone else their afternoon.

## Register — what the run spends

Set once, at the route, and held for the whole run. `crouton` owns it; this
names the setting so it lands on the record with the other four instead of
being rediscovered at every phase boundary.

| Value | Read it as | What it changes |
|---|---|---|
| **trimmed** | Full grammar, preamble and closing recaps gone | The default. Anything read once and carefully, or quoted later |
| **clipped** | Fragments and dropped articles, facts still joined into sentences | Someone is watching the run live and can ask a follow-up cheaply |
| **telegraphic** | One line per fact, nothing joining them | Status during a long run; findings a reader will scan and pick from |

The value names the *prose*, but the dial binds the larger half: ranges rather
than whole files, `grep` and `sed -n` over `cat`, no re-read to confirm an edit
that already reported success, background said once. That half is where a
multi-phase run's tokens actually go, and it does not vary with the value.

Three things never move with this dial, per `crouton`'s own gate: the run
record, every artifact the run lands, and any warning before something
irreversible. Compress the run, never the record, and never the warning.

## Reading the four together

Some combinations carry a rule that neither dial carries alone.

| Combination | The rule it creates |
|---|---|
| autonomous × live | The strictest cell on the board. The run may prepare the irreversible action in full and may not take it. It stops, leaves the command written down, and reports |
| autonomous × express | Legal and common — this is most delegated small work. The floor still applies: the change is still proven by something run, not by reading it |
| unattended × anything | The report is the entire deliverable. Nobody will ask a follow-up question, so nothing may be left implicit |
| campaign × shared tree | The record is contended too. One writer owns it, appends only, and every other writer adds rather than edits |
| express × campaign | Not a combination — a contradiction. Something has been misread; re-derive the amplitude before continuing |

## Failure modes

| Symptom | Real cause |
|---|---|
| A question asked into a silence, then answered by you anyway, unrecorded | Presence read as `paired` when it was `autonomous`; the substitution `bounding-autonomous-work` owns never happened |
| An eight-phase run on a one-line change | Amplitude never read; `standard` taken as the default rather than as a judgment |
| The run was careful for hours and reckless in its last command | Blast radius read once at the start instead of before the irreversible step |
| Two agents' work landed and one of them silently lost | Crew read as `solo` because the tree looked quiet at the moment it was read |
| An express run that ran for a day | Promotion refused because the work was "nearly done" — which it was, four times |
| A perfect report nobody could act on | `unattended` treated as `autonomous`: written for a reader who could ask a follow-up question, to a channel where nobody can |
