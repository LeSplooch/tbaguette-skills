# TBaguette's Atelier

96 skills, shipped as the `TBaguette@skills-dir` plugin. Invoke any of them as
`TBaguette:<skill-name>` — they also load automatically when their `description:`
matches the situation. This file is for humans browsing what exists.

- Plugin root: `~/.claude/skills/TBaguette/`, skills in `skills/<name>/SKILL.md`
- Inventory and token cost: `claude plugin details TBaguette`
- Turn the whole group off: `claude plugin disable TBaguette@skills-dir`

Skills marked † are **not** part of the Atelier — they live loose in `~/.claude/skills/`
or come from the `superpowers` plugin, and are listed here only to show where an
Atelier skill hands off to a neighbour.

## UI and design

| Skill | For |
|---|---|
| `formidable` | Design and craft on **any** UI stack — web, native mobile, desktop, terminal/TUI, CLI output, game HUD, embedded and e-ink, XR, email, print/PDF, voice and chat, dense data. Multi-file: 12 stack playbooks + 12 command references |
| `impeccable` † | Frontend/web design, the original this adapts |
| `design-system` † | Establishing a coherent visual language in a codebase |

## Judgment and meta

| Skill | For |
|---|---|
| `using-tbaguette` | Force-injected at every session start: check the Atelier's own skills before every response, for the whole conversation |
| `orchestrating-work-end-to-end` | The spine the rest of the library hangs off: which of seven tracks a request is on, the envelope the run executes in, the phase order, the evidence that opens each gate, what a gate becomes when nobody is there to answer it, one run record that survives compaction. Multi-file: envelope dials, the express lane, full phase-to-skill routing index |
| `calibrating-confidence` | Marking verified vs inferred vs assumed; false precision; saying you don't know |
| `estimating-effort` | Reference classes, ranges over points, the planning fallacy |
| `deciding-reversibility` | One-way vs two-way doors; matching decision cost to decision weight |
| `steelmanning-alternatives` | Escaping first-idea lock-in; recommending rather than surveying |
| `managing-scope-drift` | Necessary vs adjacent vs discovered work; silent widening and narrowing |
| `revalidating-decisions` | Separating a decision's principle from its premise; which premises expire with no commit; overturning an old call on evidence |
| `reading-specifications` | Turning ambiguous prose into testable requirements |
| `scoping-before-building` | Turning an idea into an approved design before any code; one clarifying question at a time |

## Planning and delegation

| Skill | For |
|---|---|
| `structuring-an-implementation-plan` | Turning a settled spec into bite-sized, placeholder-free tasks |
| `working-a-plan-task-by-task` | Executing an already-written plan inline, in the current session |
| `delegating-tasks-with-review-gates` | Fresh subagent per task, gated by a two-stage review, with a bounded fix loop |
| `fanning-out-independent-work` | Telling genuine independence from work that only looks independent; avoiding collisions |
| `routing-around-capability-gaps` | When this model or harness can't do it: surveying what else is on the machine, deterministic tools before a second model, consent before data crosses a provider |
| `bounding-autonomous-work` | Work that finishes before anyone reads it: substituting each gate rather than skipping it, four pre-committed stop conditions, the actions no confidence licenses without a human |
| `checkpointing-long-runs` | Work that outlives the context holding it: what to write and when, the seams worth a checkpoint, re-reading rather than recalling, the brief a successor can act on |

## Reading code

| Skill | For |
|---|---|
| `orienting-in-unfamiliar-code` | The first pass through a codebase nobody present wrote |
| `tracing-data-flow` | Following one value from source to sink |
| `code-archaeology` | Recovering intent from history: blame, bisect, pickaxe, reverts |
| `recovering-agent-context` | What prior AI sessions across every tool already learned here — transcripts, instruction files, dead ends already paid for |
| `mapping-dependencies` | The real graph; cycles, layering, blast radius |
| `finding-the-seam` | Where to make a change so the blast radius is smallest |
| `naming-things` | Names as the cheapest documentation and the costliest to change late |

## Landing changes

| Skill | For |
|---|---|
| `atomic-commits` | One logical change per commit; splitting a grown working tree |
| `writing-commit-messages` | The subject line as an index entry for a future bisector |
| `incremental-migration` | Strangler fig, expand–migrate–contract, never big-bang |
| `refactoring-safely` | Behavior preservation; refactor vs rewrite |
| `judging-duplication` | What would notice if these stopped differing; duplication as contract vs debt; the rule of three |
| `deleting-code` | Proving code is dead; deprecation with a deadline |
| `feature-flagging` | The four flag types, flag debt, combinatorics |
| `resolving-merge-conflicts` | Resolving by intent; semantic conflicts that never conflict textually |
| `isolating-work-with-worktrees` | Judging whether isolation is worth its cost; native isolated-workspace tool vs. a manual git worktree |
| `landing-a-finished-branch` | Merge, rebase, or squash, and what each does to history; branch and worktree cleanup |

## Testing

| Skill | For |
|---|---|
| `designing-test-data` | Builders over fixtures; the one-obvious-difference rule |
| `auditing-new-input-categories` | A new category vs. a new instance; why prior categories' green tests are not evidence; sourcing known trouble spots from expert knowledge |
| `property-based-testing` | Invariants, generators, shrinking |
| `testing-the-untestable` | Time, randomness, network, filesystem, identifiers, concurrency |
| `flaky-test-triage` | The cause taxonomy; quarantine with an expiry; why retry is not a fix |
| `regression-test-from-bug` | Fail first, name the test after the defect |
| `characterization-testing` | Pinning legacy behavior before changing it |
| `choosing-test-scope` | Unit, integration, contract, end-to-end — and what only each catches |
| `grounding-test-doubles` | Fixture provenance; capture over compose; one live test per integration; unrecognized shapes must raise |
| `writing-the-failing-test-first` | The red-green-refactor cycle itself — failing test first, minimal code, refactor on green |

## Debugging and performance

| Skill | For |
|---|---|
| `reproducing-bugs` | Report to on-demand failure; minimal reproductions |
| `bisecting-failures` | Binary search over commits, input, config, versions |
| `reading-stack-traces` | Which frame identifies the defect; traces that lie |
| `debugging-concurrency` | Races, deadlocks, heisenbugs, happens-before reasoning |
| `observing-production-safely` | Diagnosing live systems without becoming the incident |
| `performance-profiling` | Baselines, percentiles, flame graphs, benchmark traps |
| `finding-resource-leaks` | Growth over time; retention vs allocation; error-path leaks |
| `diagnosing-before-fixing` | The general hypothesis-driven loop; tracing a symptom back to where it actually originates |
| `responding-to-incidents` | Something is broken now: mitigating before diagnosing, preserving the evidence the mitigation would destroy, holding all three roles alone, handing back once users are safe |

## Designing systems

| Skill | For |
|---|---|
| `designing-apis` | The interface as the part you cannot change later |
| `modeling-errors` | Classifying failure before choosing a mechanism |
| `designing-for-idempotency` | At-least-once everywhere; idempotency keys and dedup windows |
| `choosing-concurrency-model` | Picking by workload shape, not by fashion |
| `modeling-state-machines` | Replacing boolean soup; illegal states unrepresentable |
| `drawing-boundaries` | Coupling and cohesion; when not to split |
| `caching-strategy` | Correctness traded for latency; invalidation and stampedes |
| `schema-evolution` | Changing a contract already in production |
| `data-migrations` | Backfills that are batched, resumable, and verified |
| `configuration-management` | Config vs code vs secret; fail fast at startup |
| `instrumenting-for-observability` | Deciding what to emit before the incident |
| `rate-limiting-and-backpressure` | Shedding vs queueing vs slowing down; retry storms |
| `tracking-data-provenance` | Observed vs imported vs inferred vs defaulted; confidence laundering; one write path per provenance |

## Security (defensive)

| Skill | For |
|---|---|
| `threat-modeling` | A model that fits in a design review |
| `handling-untrusted-input` | Parse don't validate; the injection family as one bug |
| `validating-numeric-input` | NaN and infinity defeating every comparison; overflow, precision loss, locale ambiguity |
| `secrets-hygiene` | Where secrets must never be; revoke-then-rotate |
| `redacting-sensitive-output` | A replacement that quotes the match; allowlist fields over denylist patterns; assert the input is gone |
| `auditing-dependencies` | Code you ship and did not review; triage by reachability |
| `least-privilege-design` | Default deny; blast radius as the design metric |

## Finishing and proving

| Skill | For |
|---|---|
| `finishing-what-you-started` | The near side of the finish line: an acceptance ledger written before the work, checks watched failing first, numbers re-measured at report time |
| `confirming-before-claiming-done` | Evidence before claiming done — running the check that proves it, not trusting a stale run or a subagent's report |
| `red-teaming-your-own-work` | A bounded adversarial pass before handing work off |
| `karen-and-the-manager` | Persona-forced finishing pass: never-satisfied critique, then a triage pass on it. Invoke right after `knowing-when-to-stop` |
| `knowing-when-to-stop` | Diminishing returns, gold-plating, bounded passes, explicit handoff |
| `offering-the-next-move` | Closing a run with the next move already assembled: options harvested from the ledger and the record rather than invented, ranked, and recommended |

## Communicating

| Skill | For |
|---|---|
| `writing-durable-docs` | The four doc types; documenting why; deleting stale docs |
| `writing-adrs` | Recording a decision so a future reader can tell if it still applies |
| `writing-release-notes` | Written for the person deciding whether to upgrade |
| `writing-postmortems` | Blameless in mechanism; contributing factors over root cause |
| `reviewing-code-deeply` | Reviewing in priority order; finding what is absent |
| `explaining-technical-work` | Conclusion first; altitude chosen by what the reader will do |
| `crouton` | Terse on purpose: what really costs tokens, which words survive any cut, where compression stops |
| `handing-off-for-review` | What a reviewer needs up front; when a request is premature |
| `verifying-review-feedback` | Verifying feedback before acting on it; fix vs. pushback vs. clarifying question |

## Environment and tooling

| Skill | For |
|---|---|
| `portable-shell-scripting` | Quoting, `set -e` exemptions, GNU vs BSD, when to stop using shell |
| `reproducible-environments` | The "works on my machine" taxonomy; verifying by building twice |
| `designing-ci-pipelines` | Cost-to-signal ordering; cache correctness; required vs advisory |
| `upgrading-dependencies` | Routine and cheap, or one forced upgrade during an incident |
| `keeping-tbaguette-current` | Checks the installed Atelier plugin against the published repo, updates it if it can fast-forward cleanly, keeps a local changelog |
| `tending-tbaguette` | Capturing a project-agnostic lesson while it is still in the transcript, and contributing it back as a pull request |
| `automating-repetition` | When a manual sequence should become a tool, and when it should not |
| `play-console` † | Google Play Console workflows |
