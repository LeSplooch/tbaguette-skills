# Phase routing: where every skill plugs in

The spine in `SKILL.md` names one owner per phase. This file names everything
else — every skill in the library, indexed against the phase where it earns its
place, so checking "what else does this phase call for" is a lookup rather than
a memory exercise.

Read the section for the phase you are entering, and the track sections when
picking a track. Reading it end to end is only worth doing once, to learn its
shape.

A skill appearing under two phases is not a mistake. `judging-duplication`
decides a design question and catches a review finding, and the reason to reach
for it differs each time. What matters is the "reach for it when" column, not
the placement.

## Contents

- [Before the run — the library itself](#before-the-run--the-library-itself)
- [Before phase 1 — arriving cold](#before-phase-1--arriving-cold)
- [Phase 1 — Frame](#phase-1--frame)
- [Phase 2 — Design](#phase-2--design)
- [Phase 3 — Isolate](#phase-3--isolate)
- [Phase 4 — Plan](#phase-4--plan)
- [Phase 5 — Implement](#phase-5--implement)
- [Inside phase 5 — when implementation stalls](#inside-phase-5--when-implementation-stalls)
- [Phase 6 — Review](#phase-6--review)
- [Phase 7 — Prove](#phase-7--prove)
- [Phase 8 — Land](#phase-8--land)
- [Diagnose track](#diagnose-track)
- [Investigate track](#investigate-track)
- [Change-in-place track](#change-in-place-track)
- [Outside the spine](#outside-the-spine)

## Before the run — the library itself

Ahead of orienting, ahead of framing, ahead of naming the track.

| Skill | Reach for it when |
|---|---|
| `keeping-tbaguette-current` | Always — first, before anything else. Usually already answered by the start-of-session check, in which case read that answer instead of re-fetching. Reports and moves on; never blocks the run |

## Before phase 1 — arriving cold

| Skill | Reach for it when |
|---|---|
| `recovering-agent-context` | Anything here was touched by a prior session or another agent — their dead ends are already paid for |
| `orienting-in-unfamiliar-code` | Nobody present wrote this code and the request assumes you know where things are |
| `code-archaeology` | The current shape only makes sense as the result of decisions nobody wrote down |
| `reading-specifications` | A ticket, RFC, or standard is the input, and it has to become testable requirements before it can be framed |
| `routing-around-capability-gaps` | The work needs something this model or harness cannot do; find that out before promising the run, not during it |

## Phase 1 — Frame

| Skill | Reach for it when |
|---|---|
| `scoping-before-building` | Always — it classifies the request and says the size out loud |
| `finishing-what-you-started` | Always — the acceptance ledger is this phase's deliverable, written before the work, each line watched failing |
| `reading-specifications` | The request's own words contain MUST/SHOULD, or edge cases it never addresses |
| `managing-scope-drift` | The request is already growing between restatements |
| `estimating-effort` | Someone needs a range before committing, and a point estimate would be false precision |
| `deciding-reversibility` | The work has a one-way door in it — the framing changes if a mistake cannot be walked back |
| `steelmanning-alternatives` | The first idea arrived with the request and nothing else has been considered |

## Phase 2 — Design

| Skill | Reach for it when |
|---|---|
| `scoping-before-building` | Always — this phase is its sectioned-design gate, ending in an explicit yes |
| `steelmanning-alternatives` | A single approach is about to be presented as the approach |
| `deciding-reversibility` | Choosing how much design rigor this decision actually deserves |
| `revalidating-decisions` | An existing decision constrains this one, and its premises may have expired |
| `writing-durable-docs` | The design is being written down rather than said — this phase's gate depends on that document being readable months later, not just today |
| `writing-adrs` | A choice made here will look wrong to a competent newcomer, or has already been argued twice |
| `reading-specifications` | A spec, ticket or standard is the input to this design rather than its output |

**Shape and structure**

| Skill | Reach for it when |
|---|---|
| `designing-apis` | The change adds or alters an interface other code will depend on |
| `drawing-boundaries` | Deciding what belongs in which module, or whether to split at all |
| `modeling-errors` | Failure modes need classifying before a mechanism gets picked for them |
| `modeling-state-machines` | Booleans are multiplying and illegal combinations are becoming representable |
| `choosing-concurrency-model` | Anything concurrent — pick by workload shape, before code |
| `designing-for-idempotency` | Anything that can be delivered, retried, or replayed more than once |
| `caching-strategy` | Latency is being bought with correctness, and invalidation needs a plan |
| `rate-limiting-and-backpressure` | The system can be given more work than it can do |
| `schema-evolution` | A contract already in production has to change |
| `configuration-management` | Deciding what is config, what is code, and what is a secret |
| `instrumenting-for-observability` | Choosing what to emit — done at design time or not at all |
| `tracking-data-provenance` | Values arrive observed, imported, inferred, or defaulted, and downstream will not be able to tell |

**Security, decided here or not at all**

| Skill | Reach for it when |
|---|---|
| `threat-modeling` | The design has an attacker in it, and the model has to fit in a design review |
| `least-privilege-design` | Deciding what this component is allowed to reach |
| `handling-untrusted-input` | Anything parses input someone else controls — a design-phase concern, not a review-phase one |

**Where it goes and what it is called**

| Skill | Reach for it when |
|---|---|
| `mapping-dependencies` | The blast radius of the change is not yet known |
| `finding-the-seam` | Several places could hold the change and one of them is much cheaper |
| `judging-duplication` | The design either repeats something or couples two things to avoid repeating it |
| `naming-things` | The design introduces vocabulary the rest of the work will inherit |
| `formidable` | Anything with a user interface, on any stack |
| `writing-adrs` | The decision is one a future reader will need to re-judge against its premises |

## Phase 3 — Isolate

| Skill | Reach for it when |
|---|---|
| `isolating-work-with-worktrees` | Always — including to decide the isolation is not worth its cost |
| `reproducible-environments` | The baseline suite fails and it is not clear whether that is the code or the machine |
| `designing-ci-pipelines` | The work changes what runs on every commit, or the baseline takes long enough to matter |
| `feature-flagging` | The change will land before it is meant to be reachable |

## Phase 4 — Plan

| Skill | Reach for it when |
|---|---|
| `structuring-an-implementation-plan` | Always — bite-sized tasks, exact paths, no placeholders |
| `choosing-test-scope` | Deciding which level each task's verification belongs at |
| `designing-test-data` | Tasks need fixtures or builders, and the plan should say which |
| `fanning-out-independent-work` | Two or more tasks are genuinely independent and could run at once |
| `incremental-migration` | The plan's shape is expand–migrate–contract rather than a sequence of features |
| `estimating-effort` | The task breakdown is about to imply a schedule |
| `automating-repetition` | The same manual sequence appears in three or more tasks |

## Phase 5 — Implement

| Skill | Reach for it when |
|---|---|
| `delegating-tasks-with-review-gates` | A fresh subagent per task, gated by a two-stage review — the default for a multi-task plan |
| `working-a-plan-task-by-task` | The plan runs inline in this session, no subagent per task |
| `fanning-out-independent-work` | Tasks that truly do not touch each other, dispatched at once |
| `writing-the-failing-test-first` | Always, inside every task that changes behavior |
| `designing-test-data` | Building the inputs those tests run on |
| `grounding-test-doubles` | A test double stands in for something real and could drift from it |
| `testing-the-untestable` | Time, randomness, network, filesystem, identifiers, or concurrency are in the way |
| `property-based-testing` | The behavior has invariants worth generating cases against |
| `characterization-testing` | Existing behavior has to be pinned before it can safely change |
| `auditing-new-input-categories` | The change teaches the system a new *category* of input, not another instance of an old one |
| `validating-numeric-input` | Numbers cross a boundary — NaN, infinity, overflow, and locale all defeat naive comparison |
| `handling-untrusted-input` | Implementing the parse boundary the design placed |
| `secrets-hygiene` | A key, token, or credential is anywhere near the diff |
| `redacting-sensitive-output` | Logs, errors, or reports could carry data that must not appear in them |
| `naming-things` | Every identifier the task introduces |
| `refactoring-safely` | Behavior must be preserved exactly while structure changes |
| `judging-duplication` | The third occurrence of something has just appeared |
| `finding-the-seam` | The obvious insertion point turns out to be the expensive one |
| `configuration-management` | New knobs, defaults, or environment-dependent values appear |
| `feature-flagging` | The task lands behind a flag |
| `formidable` | The task touches any user-facing surface |
| `portable-shell-scripting` | The task writes shell that has to run somewhere other than this machine |
| `atomic-commits` | The task's tree has grown more than one logical change |
| `resolving-merge-conflicts` | The branch has diverged from its base far enough to need integrating mid-run |
| `automating-repetition` | The same edit is being repeated by hand across many files |

## Inside phase 5 — when implementation stalls

| Skill | Reach for it when |
|---|---|
| `diagnosing-before-fixing` | Anything is failing and the cause is not known — before any fix is attempted |
| `reading-stack-traces` | There is a trace, and which frame actually identifies the defect is not obvious |
| `reproducing-bugs` | The failure is not yet available on demand |
| `bisecting-failures` | It worked at some point and the change that broke it is unknown |
| `debugging-concurrency` | The failure moves when you look at it |
| `finding-resource-leaks` | Something grows over time rather than failing outright |
| `performance-profiling` | Something is slow and the hot path is being guessed at |
| `flaky-test-triage` | A test fails intermittently and retrying is being mistaken for a fix |
| `tracing-data-flow` | A wrong value arrives somewhere and its origin is unclear |
| `mapping-dependencies` | The fix's blast radius is larger than the file it lives in |

## Phase 6 — Review

| Skill | Reach for it when |
|---|---|
| `handing-off-for-review` | Always — composing what the reviewer needs before it is asked for |
| `reviewing-code-deeply` | Reviewing it yourself, in priority order, including what is absent |
| `verifying-review-feedback` | Feedback has arrived and each item needs verifying before it is acted on |
| `red-teaming-your-own-work` | Nothing looks wrong, and nothing has been tried to make it look wrong |
| `karen-and-the-manager` | The review came back clean — which is exactly when this is most warranted |
| `auditing-dependencies` | The diff added, bumped, or replaced a third-party package |
| `secrets-hygiene` | Before anything is published, as a scan of what the diff actually contains |
| `judging-duplication` | The review found repetition and it needs deciding, not just noting |
| `calibrating-confidence` | Findings are being reported with more certainty than the evidence supports |

## Phase 7 — Prove

| Skill | Reach for it when |
|---|---|
| `confirming-before-claiming-done` | Always — name the command that would prove it, run it fresh, read what it printed |
| `finishing-what-you-started` | Always — every ledger line re-measured now, surrenders marked, nothing quietly narrowed |
| `regression-test-from-bug` | The work fixed a defect; the test that failed before it must pass after it |
| `choosing-test-scope` | The proof runs at a level that cannot actually observe the claim |
| `flaky-test-triage` | The suite is green only sometimes, and that is being read as green |
| `red-teaming-your-own-work` | A last adversarial pass before anyone acts on this |
| `karen-and-the-manager` | The finishing pass, right before `knowing-when-to-stop` bounds it |
| `knowing-when-to-stop` | The findings are shrinking and the next pass would be polish |
| `calibrating-confidence` | Marking what is verified against what is still inferred in the final report |

## Phase 8 — Land

| Skill | Reach for it when |
|---|---|
| `landing-a-finished-branch` | Always — merge, PR, or leave it, plus worktree and branch cleanup |
| `atomic-commits` | History should be bisectable by whoever comes next |
| `writing-commit-messages` | Every commit that lands |
| `resolving-merge-conflicts` | The base moved while the work was in flight |
| `writing-release-notes` | Someone downstream has to decide whether to take this |
| `writing-adrs` | A decision made during the run should outlive the branch |
| `writing-durable-docs` | Behavior changed in a way the existing docs now describe wrongly |
| `explaining-technical-work` | Reporting the run to whoever asked for it |
| `crouton` | The report has to fit somewhere small without losing what decides things |
| `deleting-code` | The old path a migration replaced is now genuinely dead |
| `feature-flagging` | A flag introduced during the run now needs a removal date |
| `knowing-when-to-stop` | Naming what is deliberately left undone, rather than drifting into it |
| `offering-the-next-move` | Always — the last beat: the run closes with the next move offered as a choice, not described |

## Diagnose track

| Skill | Reach for it when |
|---|---|
| `diagnosing-before-fixing` | Always — this track's owner, from symptom to actual origin |
| `reproducing-bugs` | The gate before any fix: a failure available on demand |
| `reading-stack-traces` | A trace exists and may be pointing at the wrong frame |
| `bisecting-failures` | Binary search over commits, inputs, configs, or versions |
| `debugging-concurrency` | Races, deadlocks, and bugs that vanish under observation |
| `finding-resource-leaks` | Degradation over time rather than a clean failure |
| `performance-profiling` | The complaint is slowness, with no measured baseline yet |
| `flaky-test-triage` | The symptom is intermittent and the cause taxonomy has to be worked |
| `tracing-data-flow` | Following one wrong value back to where it was born |
| `observing-production-safely` | The only reproduction is live, and diagnosis must not become the incident |
| `regression-test-from-bug` | The gate after the fix: named for the defect, failing before, passing after |
| `writing-postmortems` | The failure reached users, or would have |

## Investigate track

| Skill | Reach for it when |
|---|---|
| `orienting-in-unfamiliar-code` | The question is about code nobody present wrote |
| `recovering-agent-context` | Prior sessions already explored this and their findings are recoverable |
| `mapping-dependencies` | The question is about coupling, layering, or blast radius |
| `tracing-data-flow` | The question follows a value from source to sink |
| `code-archaeology` | The question is why, and the answer is in the history |
| `reading-specifications` | The question is what a spec actually requires |
| `auditing-dependencies` | The question is about code that ships without having been reviewed |
| `estimating-effort` | The deliverable is a range, and precision would be false |
| `revalidating-decisions` | The question is whether an old decision still holds |
| `calibrating-confidence` | Always — the close-out gate: verified, inferred, or assumed, marked per claim |
| `explaining-technical-work` | Always — conclusion first, altitude set by what the reader will do next |
| `crouton` | The answer has a length budget |
| `offering-the-next-move` | Always — the report closes with the choice it implies, assembled rather than left to the reader |

## Change-in-place track

| Skill | Reach for it when |
|---|---|
| `deciding-reversibility` | Always — the opening gate; a one-way door is a different job |
| `upgrading-dependencies` | Versions have fallen behind, or an advisory names something in use |
| `incremental-migration` | Anything big enough that a big-bang cutover is being considered |
| `data-migrations` | Data has to be backfilled, batched, resumable, and verified |
| `schema-evolution` | A live contract changes shape |
| `deleting-code` | Something appears unused and has to be proven dead first |
| `refactoring-safely` | Structure changes while behavior must not |
| `characterization-testing` | Legacy behavior needs pinning before it moves |
| `feature-flagging` | The cutover needs a switch and a way back |
| `observing-production-safely` | The change is being watched live |
| `configuration-management` | The change is a config or environment change wearing a code change's clothes |
| `secrets-hygiene` | A credential is being rotated, revoked, or moved |
| `reproducible-environments` | The change only reproduces on one machine |
| `auditing-dependencies` | The upgrade pulls transitive changes nobody asked for |
| `resolving-merge-conflicts` | A long-lived branch has to be integrated as part of the change |

## Outside the spine

These do not belong to a phase. They govern the run itself, or the library.

| Skill | Reach for it when |
|---|---|
| `using-tbaguette` | Every turn, in every phase — the per-response skill check this file does not replace |
| `routing-around-capability-gaps` | Any phase where the harness or model cannot do what the phase needs |
| `crouton` | Any phase whose output has a length budget |
| `calibrating-confidence` | Any phase reporting something as known that is inferred |
| `knowing-when-to-stop` | Any phase whose returns have started shrinking |
| `managing-scope-drift` | Any phase where the work is quietly widening or narrowing |
| `writing-postmortems` | The run itself failed in a way worth learning from |
