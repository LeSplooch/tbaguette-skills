# Orbit — the whole from above

`orbit` is the view of a whole project from outside it: one bounded pass that
looks at the thing everyone is standing inside and says what it is, who it is
for, what moves through it, what is changing under it, and what it has never
had. It is the closest this skill comes to the "God's view" people ask for, so
the first thing to say is what that phrase cannot mean. **There is no seat that
sees everything.** The view from above is assembled from several partial seats,
chosen on purpose, each of which sees one thing the others cannot — and a
person reading an orbit should be able to tell which seat every sentence came
from. An orbit that claims omniscience has stopped being sight and become a
manifesto.

## Bounds first, because this is where the skill is most dangerous

An orbit is the one pass in this skill with no natural stopping point at all:
the space around a whole project is unbounded, and every pass through it can be
made to produce something. So the bounds come before the method.

- **One page.** Seven panes, each one to three sentences. If a pane needs more,
  the finding is in the excess and it goes to the routing table, not into the
  pane.
- **At most three findings**, each with the observation that would tell whether
  it is real, each routed. Everything else is description, and description is
  allowed to be *all* an orbit produces — an orbit that finds nothing is a
  project that is what it says it is, and that is a result.
- **At named moments only.** Not a mode, not a background hum, not something
  run because the work inside the frame got boring.
- **Never during Respond.** An incident has a clock. `responding-to-incidents`
  owns the order, and a whole-project view while users are broken is the
  "let me understand this properly first" red flag at its largest.
- **Never a redesign.** Anything found above the level of a rule — a goal, a
  paradigm, "this project should be a different project" — routes to the
  record and the closing offer. `managing-scope-drift` still wins, and it wins
  hardest here.
- **Presence governs.** With someone present, an orbit's findings are three
  questions. With nobody present, they are three lines in the record and the
  run continues on the original frame, per `bounding-autonomous-work`.

## The moments

| Moment | Why then |
|---|---|
| On arrival — after `orienting-in-unfamiliar-code` and `recovering-agent-context` have run, before phase 1 frames the first request | The only moment when nothing has been assumed yet. Orientation says what the code is; orbit says what the project is |
| Before a milestone decision — a launch, a rewrite, a dependency taken on, a boundary moved | A decision at this altitude is made from a frame nobody has looked at since arrival |
| When stuck three times at the same place | The third failure is one wrong model applied three times, and sometimes the model is of the project, not the bug |
| In a postmortem | An incident of this shape has often been here before under another name, and the orbit is where the name gets found |
| On request — "give me the big picture", "what is this project missing" | The one moment the page is the deliverable rather than an input |

Between moments, an orbit stays valid until one of its panes changes, and the
tell that one has is usually a finding from an ordinary sweep that does not
fit the last orbit's description of the project. Re-run the pane, not the orbit.

## The seven panes

Each pane is one seat. Write which seat, then what it sees, in one to three
sentences. A lens from [lenses.md](lenses.md) is named beside each pane because
it is the instrument that pane is made with; use it or a better one, not all
of them.

**1 — Altitudes.** The project in one sentence at each of five heights: the
line (what a function here does), the module, the system, the product (what a
person gets), the world (what it changes outside itself). The finding, when
there is one, is the height at which the sentence could not be written — the
project that has no product sentence, or no world sentence. *Lens:* altitude
ladder.

**2 — Purpose, as revealed by behaviour.** What the project says it is for, in
its own words, and what it actually does — to its users, its operators, its
data, its dependents — with the charter ignored. Where the two diverge, say
which one the code is defending. *Lens:* the purpose of a system is what it
does.

**3 — The chairs.** Who touches this: who pays, who operates, who maintains,
who integrates, who is affected without knowing it, who could veto it. Then
the empty ones — the seat nobody was in when the last three decisions were
made. *Lens:* the empty chairs; `threat-modeling` the moment an adversarial
chair is occupied.

**4 — Flows across the boundary.** Draw the boundary — what counts as this
project — then what crosses it in each direction: money, data, trust,
attention, dependencies, obligations. The finding is a flow with no owner on
this side. *Lens:* flows across the boundary; `mapping-dependencies` for the
graphs.

**5 — Time.** What is changing under this project — a dependency, a platform, a
team, a market — and at what pace. Which of its layers churn and which hold,
and whether anything is being built at the pace of the wrong layer. What was
true when its oldest decisions were made that is not true now. *Lens:* pace
layers; `revalidating-decisions` for the premise that expired.

**6 — Negative space.** What this project has never had, what it has refused,
what it lost and did not replace, and what nobody asks for because everyone
knows the answer. The closed pull requests and the wontfix list are half of
this pane; the other half is what the domain has that the tree does not.
*Lens:* the seen and the unseen; the history in the tree.

**7 — Forces.** The incentives, deadlines, budgets, rules, and habits that will
decide what happens to any proposal here regardless of its merits — and which
of the constraints everyone designs around would lift if someone asked.
*Lens:* show me the incentives; the constraint audit from the main file.

## The output

An orbit record is a markdown block with the seven panes as headings, one to
three sentences each, and a final section:

```
## Findings (at most three)
1. <observation> — would be confirmed or refuted by: <the check> — routed: gate | ledger | record | discard (<reason>)
```

Where it goes depends on the envelope. In a run with a record, it is a section
of the record, per `finishing-what-you-started`'s ledger file. In a project
that keeps design notes, it goes where those live, dated, so the next orbit
can be diffed against it — **an orbit is most valuable the second time it is
run**, because the diff between two orbits is the project's actual direction,
which no single one can show. Handed to a person, it is the page plus at most
one question, through the harness's own question tool, per
`offering-the-next-move`.

## How it fails

| Symptom | Real cause |
|---|---|
| Seven panes of description and no finding, three orbits running | Possibly nothing — that is a legitimate result. Or the panes are being written from one seat, usually the author's; check which seat each sentence came from |
| The orbit proposes a different project | A finding above the level of a rule was routed to the gate instead of the record. Leverage points above "rule" are observations, not work |
| The orbit took an afternoon | No bound was set before starting. One page, seven panes, three findings, at a named moment |
| The person got a page they did not ask for | The orbit was delivered instead of routed; only the "on request" moment makes the page the deliverable |
| Every orbit finds the same three things | They were never routed to a decision, so each pass rediscovers them. A finding declined is a ruling; record it and stop finding it |
| The orbit ran during an incident | The prohibition in the bounds. Stop, mitigate, orbit in the postmortem |
| Sentences about what people *want* | Motive is not observable. Rewrite as what they are measured on, have said, or are constrained by, or rewrite as a question |
