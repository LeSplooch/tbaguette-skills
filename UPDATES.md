# Update notes

What changed, newest first, written for someone who has TBaguette installed
rather than for someone reading the diff. The landing page renders the most
recent entries below the “Fresh from the oven” rail; this file is the whole
record.

Writing an entry is part of shipping a change here — see CLAUDE.md. The shape
is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

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

## 2026-08-24 — Skills link to each other, and the site says what changed

- Every mention one skill makes of another is now a link to that skill’s page
  — 362 of them across the site. The library is densely cross-referenced and
  none of it was navigable before; `diagnosing-before-fixing` naming
  `regression-test-from-bug` was a dead string.
- Both forms link: the `code`-span mentions that run through the bodies and
  tables, and the bare ones in the “Not for: … (other-skill)” line at the top
  of most skills. A page never links to itself, and a code span that is not a
  skill stays plain.
- The landing page carries an **Update notes** section under the “Fresh from
  the oven” rail: the newest entry’s bullets in full, everything earlier
  folded behind one disclosure that needs no JavaScript to open.
- Notes come from `UPDATES.md` at the repo root, not from the commit log, so
  an entry can say what is different for you rather than which files moved.
- The rail and the notes answer different questions side by side. The rail
  names *which* skills changed in the last 48 hours; the notes say *what*
  changed, including work that touches no skill at all.

## 2026-08-23 — Three skills sharpened, and English-only again

- `least-privilege-design` now covers being asked to justify a permission
  something already holds: check what actually calls it before defending it
  in the abstract.
- `confirming-before-claiming-done` closes the gap between “pushed” and
  “reached the user” — a deploy nothing has fetched yet is not evidence the
  change landed.
- `feature-flagging` says to pin the value of a flag that suspends a rule, so
  a test proves the suspension instead of inheriting whatever the environment
  happened to have set.
- The site dropped its eleven translated locales and serves English only.
  Translated URLs (`/fr/`, `/de/`, and the rest) no longer exist. Nothing
  about installing TBaguette or about skill content changed.

## 2026-08-22 — The spine the rest of the library hangs off

- New skill: `orchestrating-work-end-to-end`. Invoke it when a request will
  take more than one edit — it routes the work to a track, names the phase
  you are in, and says what evidence opens the next one. It is the skill that
  sequences the other ninety-one.
- The install prompt on the site works out which harness it was pasted into
  and takes that route, instead of assuming Claude Code and falling through
  to a filesystem clone.
- `rate-limiting-and-backpressure`, `instrumenting-for-observability`,
  `deleting-code`, and `authoring-a-new-skill` each gained a case they
  previously got wrong.

## 2026-08-21 — When the model itself is the blocker

- New skill: `routing-around-capability-gaps` — what to do when the thing
  being asked for is outside what this model or harness can actually do,
  rather than quietly delivering something adjacent and calling it done.
- The install instructions warn that a conversation already open when you
  install will not see the new skills. Start a fresh one.
- The “Fresh from the oven” carousel works again: a click on a tile reaches
  its skill, dragging no longer swallows that click, the fan sits level in
  its rail, and it stopped pushing the whole page sideways at narrow widths.

## 2026-08-20 — Finishing, and a version you can read

- New skill: `finishing-what-you-started`, for long runs and multi-part
  requests where stopping short would go unnoticed. Write the acceptance
  ledger to a file before starting, and re-measure every number at report
  time rather than quoting it from memory.
- The installed plugin’s version shows next to the wordmark in the site
  header, so you can tell at a glance whether what you have matches what is
  published.
- The skill check is re-asserted every turn rather than only at the start of
  a session, which is where it used to quietly lapse in long conversations.
