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
