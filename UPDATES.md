# Update notes

What changed, newest first, written for someone who has TBaguette installed
rather than for someone reading the diff. The landing page renders the most
recent entries below the “Fresh from the oven” rail; this file is the whole
record.

**Scope: the plugin, and only what a user of it would notice.** An entry earns
its place if someone who has TBaguette installed would act differently for
having read it — a new skill, a skill that now says something different, a
change to how skills are named, grouped, or installed. Everything else stays
out, and the two categories that keep trying to get in are this repo's own
tooling (test suites, build gates, registries, version bumps) and the showcase
site's furniture (its layout, its search, its animations, its chrome). Both are
real work and neither is news about the plugin. They belong in the commit log.

The test is the reader, not the effort: a change can be the hardest thing
shipped that week and still not belong here, and a one-line fix to a skill's
wording can belong here absolutely.

Writing an entry is part of shipping a change here — see CLAUDE.md. The shape
is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

## 2026-08-30 — Diagnosing: a factor that never varied is not a factor you ruled out

- `diagnosing-before-fixing` now names the dismissal that survives a whole
  investigation: deciding a cause is eliminated because every run so far shows
  it was the same. A factor that never varies cannot explain variance, which is
  not the same as it not being the cause — and the check is one question, "name
  the run where it was different."
- It carries the reason the mistake is expensive rather than merely wrong: the
  dismissal is made once, early, in half a sentence, and every later hypothesis
  is built on top of it. When several individually-sound fixes have all failed,
  the thing to look at is the factor all of them held fixed.

## 2026-08-29 — The orchestrator now asks who is answering, not just what the work is

- `orchestrating-work-end-to-end` reads a second thing before the first action:
  not just what shape the work is, but what the run may assume about the world
  it runs in. Who answers a gate — someone replying within a turn, someone
  replying tomorrow, or nobody at all. How much run the work is worth. What a
  wrong turn costs. Who else is writing to the same tree. Its gates were always
  written for one setting of those four, and nothing ever said so.
- Work that nobody will see until it is finished now has an answer that is not
  "use your judgment". New skill: `bounding-autonomous-work`. A gate whose
  answerer is absent gets **substituted, not skipped** — an approval becomes a
  written design carrying the approach that lost and a condition that fires if
  it turns out wrong; a clarifying question becomes a recorded ruling naming the
  reading that lost, never a fact. It adds the four stop conditions a long
  unattended run needs written down before it starts, and the one gate that has
  no substitute in any circumstances: an irreversible action still gets a human,
  however obviously correct it looks at hour six.
- Small changes get a real lane rather than a warning. Express runs four beats
  instead of eight, behind entry conditions you check rather than hope — the
  target already known, one reviewable diff, proof a line away, nothing live,
  nothing new introduced. The floor under it is lower and is not zero, and it
  only ever promotes: the judgment that says "this got smaller" is made by the
  part of a run that most wants to be finished.
- New skill: `checkpointing-long-runs`, for work that outlives the context
  holding it. Its point is that a long run does not fail by forgetting — it
  fails by remembering fluently and partially, so the account you hold of your
  own run is confident and missing exactly what was never restated. It ranks
  what to write by what it costs to lose, and the top of that list is the dead
  ends, which almost nobody writes down.
- New skill: `responding-to-incidents`, and a Respond track to run it from.
  Ordinary debugging reproduces first and fixes second; an outage inverts that,
  because every minute spent reproducing is spent on users. It covers declaring
  it out loud early, preserving the evidence your own mitigation is about to
  destroy, ranking mitigations by how easily they come back out rather than by
  how right they are, holding all three incident roles when you are the only
  responder, and handing back to an ordinary diagnosis once people are safe.
- Two more tracks that were missing: **Review**, for judging work you did not
  write, where the gate is coverage of the diff rather than confidence in a
  conclusion; and **Author**, for a deliverable that is a document, whose verify
  phase is the one that always gets skipped. The skill had been describing
  reviews as something it routed since it shipped, with no row for one.
- The orchestrator can now be called directly with a verb — `route` to name the
  track and the dials and stop there, `resume` to enter at the first gate not
  yet met, `gate` to say what evidence is missing, `record`, `handoff`, `abort`.
- It also gained a Refuse list, in the same register as `formidable`'s: a plan
  document for a two-task change, a clarifying question you could answer by
  reading one file, a track named after the first three edits. None are banned.
  Reaching for one when the choice was free means you were performing process
  rather than routing.
- Later the same day, two of the three new skills were reshaped to lead with
  what they make you **write down** rather than with the judgment they argue
  for. `responding-to-incidents` now opens on the six artifacts an incident
  produces — declaration, impact statement, timeline, evidence manifest, comms
  cadence, handback note — each with required fields and each with a note on
  why it does not get written by itself. The cadence in particular is now an
  artifact with a form: a stated interval, honoured even when nothing has
  changed, rather than an intention to keep people posted.
- `checkpointing-long-runs` gained the thing it was missing: what a single
  checkpoint has to contain, and a required four-part form for a dead end —
  what was tried, the narrow thing it actually rules out, what it *looks* like
  it ruled out and did not, and how many trials the null rests on. "Pinned the
  clock, no change" reads as "not a timing issue" and means nothing of the
  sort; every cache, database, and broker keeps its own clock and none of them
  was pinned. Deprioritized is not eliminated, and the difference now belongs
  in the file rather than in the head of whoever ran it.

## 2026-08-28 — Running TBaguette next to the other libraries you already have

- Getting started now answers the question nobody asks until something looks
  broken: what happens when another skill library is installed beside this one.
  Nothing has to be uninstalled to make room. Two skills that share a name stay
  separately reachable, because TBaguette's are always called with its prefix —
  `TBaguette:naming-things` is this library's, the same name without the prefix
  is whoever else's.
- If you also run something that injects its own check-the-skills-first notice,
  Superpowers being the likely one, both fire and both apply. They are not
  competing: `using-tbaguette` speaks only for TBaguette's own skills. When the
  other library has the better skill for what you are doing, that is the one to
  use — the rule was always to check what is available, never to prefer this
  library's answer for being this library's.
- Removing it is deleting one directory, and the page now says so outright:
  `~/.claude/skills/TBaguette` and nothing else. No uninstaller, no settings
  entry to unpick, nothing left behind to unwind later.

## 2026-08-28 — TBaguette is free software, and you can improve it from your side

- TBaguette is licensed under the GNU General Public License, version 2. Pass a
  changed copy of these skills on to anyone and it goes on under the same terms,
  with source. Nothing about how you *use* them changes.
- New skill: `tending-tbaguette`. It catches the lessons worth keeping while you
  work — a correction that generalizes past the codebase you are in, a gap you
  hit in a skill while that skill was running — and turns them into a pull
  request here. It asks before anything is pushed, every time, and a yes for one
  contribution is never a yes for the next.
- Editing a skill inside your own install is a dead end:
  `keeping-tbaguette-current` will not update a plugin directory that has local
  changes, so a hand-edit silently pins your install at that commit for as long
  as it sits there. `tending-tbaguette` is the way out, and a merged change
  comes back to you through the ordinary update.
- `orchestrating-work-end-to-end` now runs it immediately after the currency
  check, and `keeping-tbaguette-current` hands the local-changes case over
  instead of leaving you at a status command with no next step.
- `tending-tbaguette` names where a project-specific rule actually belongs: the
  project's own `CLAUDE.md`, which sits *above* a library skill instead of
  modifying it. Most edits someone wants to make to an installed skill are not
  corrections to it — they are one project's fact colliding with a deliberately
  general rule, and that was never an upstream change to begin with.
- It also corrects the belief that makes hand-editing feel harmless: that the
  next update quietly overwrites your edit. It does not. The update reads the
  tree, backs off rather than discarding your work, and stops there — so the
  edit persists and blocks every update behind it.
- The library now goes by its full name, **TBaguette's Atelier**, on the site, in
  the READMEs, and wherever your agent lists its installed plugins. Nothing you
  type changes: the plugin is still `TBaguette`, skills are still
  `TBaguette:<skill-name>`, and the install still lands in
  `~/.claude/skills/TBaguette`.

## 2026-08-27 — Saying something is not installed now has a rule of its own

- `confirming-before-claiming-done` names another way a check goes wrong, and it is
  the first one in that skill about claiming something is *not* there. Open the one
  place a plugin, a patch, a hook, or a migration would live, find it untouched,
  report "not installed" — fresh evidence, gathered first-hand, and wrong. That
  location was the right one for an earlier version and stays untouched forever now
  that nothing writes to it, so the check returns the same clean answer whether the
  thing is installed or not.
- What it tells you to do instead: sweep the whole target tree for the artifact's own
  name, ask the running process what it actually has open and loaded, and read which
  candidate the runtime's resolution order really selects. A location you know about
  can confirm presence the moment the thing turns up in it, and can never establish
  absence — so an absence is only ever earned by a search, and the claim should name
  what was searched.
- The skill now fires on negative claims too. Asking it to check whether something is
  installed, applied, or registered used to reach nothing: its triggers were all
  claims that something was done, fixed, or passing.

## 2026-08-27 — Work now ends with a choice instead of a paragraph

- New skill, `offering-the-next-move`: a run closes by offering what to do next
  as a selectable set of options through your harness's question tool, rather
  than describing the possibilities in a summary paragraph and leaving you to
  turn them back into a decision.
- The options are harvested, not invented. It reads the acceptance line that was
  surrendered, the ruling that could have gone the other way, the scope pushed
  out on purpose, and the thing found in passing and never chased — then ranks
  them by one test: would you have thought of this without the run? The obvious
  next step ranks last, because you would have asked for it anyway. Expect the
  menu to name things you did not know were on the table.
- It refuses a few things on your behalf. No option for work that was already
  inside the agreed scope — that gets finished first rather than handed back as
  a question. No option it would argue against if you picked it. No two options
  that mean the same thing padded out to fill the widget. And one option always
  ends the work, with a real read on whether stopping there is defensible, so a
  menu can never be four flavors of more.
- A run that failed or stalled gets an offer too, and that is where it pays
  most: what blocked it, what would unblock it, and what is worth doing instead.
- `orchestrating-work-end-to-end` now runs it as the last beat of landing and of
  any investigation's report, and says to assemble the offer before the run
  record is torn down — the record is where most of the options come from.
- Nothing is listening on a headless or scheduled run, so there the same ranked
  list goes into the report instead of into a prompt nobody would see.
- Sharpened after watching fresh agents close the same finished task with and
  without it. The failure worth naming turned out not to be the obviously generic
  menu — it is the polite one: a wrap-up that reports every finding accurately and
  then tells you what it intends to do about them. *Tonight I'll rebuild the
  runner. That's my first job tomorrow.* All the information arrives and none of
  the decision does. The skill now says that outright — a closing sentence in the
  first person and the future tense is a decision that was never offered.
- The wrap-up you were already getting does not shrink to make room for the
  options. The report comes first and in full — what was done, the evidence, what
  is not done and why — and the offer sits underneath it. The test the skill sets
  itself: if you ignore the options entirely you should be no worse off than if
  none of this existed, so anything you would lose by skipping them was in the
  wrong place.
- The offer arrives through your harness's question tool, so a direction is one
  click rather than a sentence you have to type back. Options written into the
  message body are a fallback for harnesses that have no such tool, not a
  shortcut when one is sitting there.
- `orchestrating-work-end-to-end` now opens by checking that TBaguette itself is
  current, ahead of naming the track and ahead of any work. Nothing to do on your
  side: where the start-of-session check has already answered, it reads that
  answer instead of hitting the network again, it stays silent when you are up to
  date, and it never blocks a run — a library that cannot update is reported and
  the work carries on.
- Four more skills now hand off to that close-out, so it reaches you from where
  runs actually end rather than only from the orchestrator: landing a branch,
  writing up technical work, marking what you are confident about, and
  surrendering an acceptance line. The last of those is the one worth knowing —
  a criterion you asked for and did not get is the first thing the close reaches
  for, so it comes back to you as a decision rather than as a footnote in the
  report. Landing also now says to assemble the offer before cleanup deletes the
  plan and run record it draws on.

## 2026-08-25 — The orchestrator now owns the documents a run leaves behind

- `orchestrating-work-end-to-end` treated writing things down as a landing chore:
  `writing-durable-docs` and `writing-adrs` were reachable only from the final
  phase, while the design phase's own gate already required "a written spec" and
  said nothing about how to write one, where to put it, or who keeps it true. Both
  now sit in the design phase too, alongside `reading-specifications` for when a
  spec is the input rather than the output.
- It also distinguishes the four things a run produces, which get confused in one
  direction: the spec and the ADR outlive the branch and are maintained as though
  they were scratch, while the plan and the run ledger are scratch and get left in
  the repo for a year. Each now has a stated lifespan, and phase 8 has to decide
  what happens to the scaffolding rather than leaving it because nobody looked.
- The rule that costs most to miss: routing backward to the design phase leaves
  the spec describing an approach that lost. Leaving it is worse than never having
  written one, because the next reader gets a confident description of the wrong
  design with nothing marking it as dead. The skill now says the document changes
  in the same move that reopens the phase — and that the rejected approach and the
  reason for rejecting it are what a later reader most needs, not what to delete.

## 2026-08-25 — Two rules in one skill that cancelled each other out

- `configuration-management` carried a contradiction that produced a broken
  build if you followed it. One section tells you a switch needing to flip
  without a restart is a flag, not config. An earlier rule says **never read
  config at the call site** — and read together, those build a flag loaded once
  at boot, which cannot flip during the incident it exists for. Each rule is
  correct alone, which is why the pair survived review. The call-site rule now
  carries an explicit carve-out, keyed to something checkable: if the value must
  change without a restart, evaluating it at the call site is the mechanism, not
  a violation, and what gets validated at startup is the flag client's wiring
  rather than its value.
- The same skill's flag-expiry rule — *delete a flag older than one release
  cycle in one direction or the other* — is written for release flags and
  misfires badly on an operational kill switch, where deleting toward on
  reinstates the incident and deleting toward off removes the feature. It now
  says which flag type it is about, and that a kill switch takes a removal
  *condition* and a scheduled exercise instead of a date, because the arm you
  need at 3am is the arm that never runs.
- `authoring-a-new-skill` now warns about the gate that actually ambushes people
  when adding a skill: the library refuses a skill that no other skill points
  at, so a new one turns the suite red until some neighbour's "Not for:" line
  redirects to it — a skill worth adding is one some existing skill should be
  handing off to.

## 2026-08-25 — A ship step that told you to do work that no longer exists

- `authoring-a-new-skill` was still instructing anyone shipping a new skill to
  translate its description into every locale the site builds. The site has been
  English-only since 23 August, when `i18n/` was deleted and the locale list was
  cut to English alone — so that step described work that cannot be done and does
  not need doing. Removed. The same checklist named only `CATALOG.md` as the place
  a new skill gets filed, omitting the registry in `content_pipeline.py` that
  actually refuses to build without it; it now names both, and points at how to
  create a category when none of the existing ones fit.
- Ten more skills gained cross-references to the skills that own the other half
  of their problem, found by reading all 92 end to end rather than by sampling.
  The ones worth naming: `configuration-management`'s "this is really a flag"
  section now hands off to `feature-flagging` instead of restating its rules;
  `code-archaeology` and `atomic-commits` both discussed bisection at length
  without ever naming `bisecting-failures`; `explaining-technical-work` asked you
  to mark claims verified, inferred, or assumed without mentioning that
  `calibrating-confidence` defines those three tiers; and `deleting-code`'s
  proof-of-deadness step now says why it inverts the usual observability rule —
  you are trying to emit that nothing happened, and only a counter can show that.
- Across both of today's updates the library went from 308 cross-references to
  355, and from three skills that referenced nothing at all to none.

## 2026-08-25 — Two new categories, and a pointer that led nowhere

- The library is organised into **twelve categories instead of ten**, and eleven
  skills moved. **Planning and delegation** collects the five skills about turning
  an approved design into tasks and getting those tasks done —
  `structuring-an-implementation-plan`, `working-a-plan-task-by-task`,
  `delegating-tasks-with-review-gates`, `fanning-out-independent-work` and
  `routing-around-capability-gaps` — which were previously split between
  "Communicating" and "Environment and tooling" despite each one's own text
  defining its edges against the others. **Finishing and proving** collects the
  five that guard the finish line: `finishing-what-you-started`,
  `confirming-before-claiming-done`, `red-teaming-your-own-work`,
  `karen-and-the-manager` and `knowing-when-to-stop`. "Judgment and meta" had
  grown to fourteen skills spanning four unrelated activities; it is now ten.
  `reading-specifications` also moved out of "Reading code", where it was the one
  entry that is not about reading code, to sit beside `scoping-before-building`.
  Nothing about invoking a skill changes — the names, and every skill's own page
  URL, are exactly as they were.
- `tracing-data-flow` had been sending readers to a skill that does not exist.
  Its "Not for" line pointed at `systematic-debugging`, which is Superpowers'
  name for the thing this library calls `diagnosing-before-fixing`; it survived
  because it was written without backticks, so no search for skill names ever
  saw it. Fixed.
- Twelve skills gained cross-references to the skills that own the other half of
  their problem. `caching-strategy` and `rate-limiting-and-backpressure`
  previously mentioned **no other skill at all** — the two halves of the same
  overload incident, with nothing pointing between them. Also newly wired:
  `managing-scope-drift` to the ledger that makes silent narrowing visible,
  `steelmanning-alternatives` to the premises that expire underneath an old
  decision, `testing-the-untestable` to the question of whether a double is
  faithful once the seam is in the right place, and `data-migrations` to the
  fact that a migration is almost always a one-way door.

## 2026-08-25 — Eight lessons from real work, folded back in

- `instrumenting-for-observability` separates *did it run* from *did it have
  anything to work with*. Anything that learns, tunes, calibrates or ranks reads
  a ground-truth source, and when that source is empty the component does not
  fail — aggregates over an empty set are well defined, so sums are zero, fitness
  functions return values, counters advance, and it looks productive on every
  dashboard. Worse, it degrades to its prior, and priors are chosen to look
  neutral rather than absent, so the failure arrives as a confident middling
  number. Emit input volume and age as their own signals, carry the sample size
  to the point of use so a consumer can tell a measurement from a default, and
  let the component refuse below a floor instead of fitting noise.
- `drawing-boundaries` picks up the properties that boundaries quietly destroy.
  A rule like *run the cheapest checks first* gets written for one stage, and
  that stage obeys it — while the layer above does the reverse, because a module
  cannot observe an ordering only visible from outside it, and cannot enforce
  what it cannot observe. Every layer passes its own review and the composition
  stays broken. For gates the audit is mechanical: list each one, annotate cost
  and what it needs to know, and check the ordering is monotonic *across*
  boundaries. A gate that runs after the spend is not a gate, it is a receipt.
  The same inversion hides in retries that multiply, inner timeouts longer than
  outer ones, per-client rate limits that blow the aggregate, and TTLs in series.
- `diagnosing-before-fixing` adds a signal that arrives before the expensive one.
  Re-running a failing operation is only an experiment if something differed
  between the runs; when nothing did, the second attempt is a re-observation of
  the first. Byte-identical output across attempts is itself a finding — the
  failure is deterministic, so the cause is structural, and that is the moment to
  read the error rather than run it again. It sits in deliberate tension with the
  reproduction step, which wants identical results: same observation, opposite
  meaning, separated by asking what differed. The three-failed-fixes rule already
  in the skill charges three fixes before it fires; this costs a re-read.
- `writing-durable-docs` names the claims its own drift ladder cannot reach.
  Every rung of that ladder needs something to execute or generate, so a claim
  that something *is not* the case — no telemetry, no third-party calls, no
  runtime dependencies, stores no personal data — is stuck at silent rot
  permanently. Those are also the sentences readers lean on hardest, because they
  are promises rather than descriptions. And they break backwards: a negative
  claim goes false when something is added somewhere else entirely, in a diff
  with no reason to touch the doc. Invert the check — a test that fails when the
  forbidden thing appears — and treat a doc asserting behaviour as a call site of
  it. Plus: two copies of one document with no generation step is a fork.
- `designing-ci-pipelines` names why scheduled jobs rot unnoticed. Every other
  check in a pipeline has somebody waiting on it, so silence means no trouble. A
  scheduled job has no audience, and its success and its total non-existence
  produce the same silence — which is why one that has never run successfully
  looks identical to a healthy one on any dashboard showing last-run status. It
  surfaces as the absence of the thing the job maintained, after the deadline it
  protected. Two habits fix it: force one run by hand when the job lands, and
  report *age of last success* rather than status of the last run, since a job
  suspended in March is green on the second and four months stale on the first.
- `portable-shell-scripting` covers the mirror of a trap it already had. A `cd`
  inside `$(…)` cannot escape; a `cd` at the front of one command in a session
  that persists between commands re-roots every relative path used afterwards, by
  every later command, including ones written by someone who never saw it. The
  expensive part is not the breakage but that "no such file" and a grep matching
  nothing are exactly what a real absence looks like — so the result is a
  confident false negative that reads as a finding about the codebase. Prefer
  `git -C`-style path options, then absolute paths, then `(cd x && …)`; and print
  `pwd` before believing a surprising negative.
- `writing-the-failing-test-first` gains the collection case, which the scalar
  advice about hand-deriving expected values does not reach: an assertion of the
  form *for each item in the result, assert it is well-formed* quantifies over a
  set the code under test chose. Drop half the input and the survivors still
  pass; drop all of it and the loop never runs, so the test is at its most
  confident exactly when the code has failed hardest. Get one number from outside
  the unit — count the source, not the result — and when a loss is deliberate,
  state the expected number rather than deriving it.
- `threat-modeling` now asks a question that catches a whole class of bypass: a
  limit, quota or entitlement is a property of *state*, but it almost always gets
  implemented as a check on one *transition* — the interactive one someone had in
  mind. Seeding, import, migration, restore and sync reach the same state without
  passing it, usually with more authority and less scrutiny. Enumerate the
  producers, not the check. And ask whether anything re-examines the state after
  it exists, because a gate that runs only at creation cannot repair what arrived
  around it.

## 2026-08-25 — A check that would pass even if nothing worked

- Some things you are asked to confirm are not about the present at all: that a
  service comes back after a restart, that a cache rebuilds cold, that a project
  builds from a fresh clone, that a backup restores. The check to hand almost
  always measures the present instead — it is running, it answers, it is set to
  start — and those pass just as convincingly in the world where the requirement
  is completely unmet.
- One question tells the two apart: would this check still pass if the condition
  the requirement names had never once occurred? If yes, it is a proxy, and
  re-running it proves nothing further. Worth asking before calling anything done
  on a service, a cache, a scheduled job, or a backup.
- What settles it is inducing the condition once — actually restarting the host,
  clearing the cache, cloning into an empty directory. That is disruptive and
  often not yours to authorise, so it now says to treat it as a reversibility
  decision and get the owner's go-ahead first. When you cannot, the honest report
  is "configured but unverified", plus the test that would settle it — not the
  proxy dressed up as proof.
- Two things read a result better than any status string: a start timestamp
  beside a boot timestamp tells you *who* started something, and a restart
  counter separates coming up cleanly from crashing and being retried, which look
  identical from outside.
- Asking for something to be made to start on boot, survive a restart, or work
  from scratch now reaches `confirming-before-claiming-done` on its own.

## 2026-08-23 — Three skills sharpened

- `least-privilege-design` now covers being asked to justify a permission
  something already holds: check what actually calls it before defending it
  in the abstract.
- `confirming-before-claiming-done` closes the gap between “pushed” and
  “reached the user” — a deploy nothing has fetched yet is not evidence the
  change landed.
- `feature-flagging` says to pin the value of a flag that suspends a rule, so
  a test proves the suspension instead of inheriting whatever the environment
  happened to have set.

## 2026-08-22 — The spine the rest of the library hangs off

- New skill: `orchestrating-work-end-to-end`. Invoke it when a request will
  take more than one edit — it routes the work to a track, names the phase
  you are in, and says what evidence opens the next one. It is the skill that
  sequences the other ninety-one.
- `rate-limiting-and-backpressure`, `instrumenting-for-observability`,
  `deleting-code`, and `authoring-a-new-skill` each gained a case they
  previously got wrong.

## 2026-08-21 — When the model itself is the blocker

- New skill: `routing-around-capability-gaps` — what to do when the thing
  being asked for is outside what this model or harness can actually do,
  rather than quietly delivering something adjacent and calling it done.
- A conversation that was already open when you install will not pick up the
  new skills. Start a fresh one.

## 2026-08-20 — Finishing what you started

- New skill: `finishing-what-you-started`, for long runs and multi-part
  requests where stopping short would go unnoticed. Write the acceptance
  ledger to a file before starting, and re-measure every number at report
  time rather than quoting it from memory.
- The skill check is re-asserted every turn rather than only at the start of
  a session, which is where it used to quietly lapse in long conversations.
