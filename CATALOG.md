# TBaguette

67 skills, shipped as the `TBaguette@skills-dir` plugin. Invoke any of them as
`TBaguette:<skill-name>` — they also load automatically when their `description:`
matches the situation. This file is for humans browsing what exists.

- Plugin root: `~/.claude/skills/TBaguette/`, skills in `skills/<name>/SKILL.md`
- Inventory and token cost: `claude plugin details TBaguette`
- Turn the whole group off: `claude plugin disable TBaguette@skills-dir`

Skills marked † are **not** part of TBaguette — they live loose in `~/.claude/skills/`
or come from the `superpowers` plugin, and are listed here only to show where a
TBaguette skill hands off to a neighbour.

## UI and design

| Skill | For |
|---|---|
| `formidable` | Design and craft on **any** UI stack — web, native mobile, desktop, terminal/TUI, CLI output, game HUD, embedded and e-ink, XR, email, print/PDF, voice and chat, dense data. Multi-file: 12 stack playbooks + 12 command references |
| `impeccable` † | Frontend/web design, the original this adapts |
| `design-system` † | Establishing a coherent visual language in a codebase |

## Judgment and meta

| Skill | For |
|---|---|
| `calibrating-confidence` | Marking verified vs inferred vs assumed; false precision; saying you don't know |
| `red-teaming-your-own-work` | A bounded adversarial pass before handing work off |
| `estimating-effort` | Reference classes, ranges over points, the planning fallacy |
| `deciding-reversibility` | One-way vs two-way doors; matching decision cost to decision weight |
| `steelmanning-alternatives` | Escaping first-idea lock-in; recommending rather than surveying |
| `managing-scope-drift` | Necessary vs adjacent vs discovered work; silent widening and narrowing |
| `knowing-when-to-stop` | Diminishing returns, gold-plating, bounded passes, explicit handoff |
| `karen-and-the-manager` | Persona-forced finishing pass: never-satisfied critique, then a triage pass on it. Invoke right after `knowing-when-to-stop` |
| `brainstorming` † | Exploring intent before building |
| `verification-before-completion` † | Evidence before success claims |

## Reading code

| Skill | For |
|---|---|
| `orienting-in-unfamiliar-code` | The first pass through a codebase nobody present wrote |
| `tracing-data-flow` | Following one value from source to sink |
| `code-archaeology` | Recovering intent from history: blame, bisect, pickaxe, reverts |
| `recovering-agent-context` | What prior AI sessions across every tool already learned here — transcripts, instruction files, dead ends already paid for |
| `mapping-dependencies` | The real graph; cycles, layering, blast radius |
| `finding-the-seam` | Where to make a change so the blast radius is smallest |
| `reading-specifications` | Turning ambiguous prose into testable requirements |
| `naming-things` | Names as the cheapest documentation and the costliest to change late |

## Landing changes

| Skill | For |
|---|---|
| `atomic-commits` | One logical change per commit; splitting a grown working tree |
| `writing-commit-messages` | The subject line as an index entry for a future bisector |
| `incremental-migration` | Strangler fig, expand–migrate–contract, never big-bang |
| `refactoring-safely` | Behavior preservation; refactor vs rewrite |
| `deleting-code` | Proving code is dead; deprecation with a deadline |
| `feature-flagging` | The four flag types, flag debt, combinatorics |
| `resolving-merge-conflicts` | Resolving by intent; semantic conflicts that never conflict textually |
| `using-git-worktrees` † | Isolated workspaces |
| `finishing-a-development-branch` † | Integration decisions |

## Testing

| Skill | For |
|---|---|
| `designing-test-data` | Builders over fixtures; the one-obvious-difference rule |
| `property-based-testing` | Invariants, generators, shrinking |
| `testing-the-untestable` | Time, randomness, network, filesystem, identifiers, concurrency |
| `flaky-test-triage` | The cause taxonomy; quarantine with an expiry; why retry is not a fix |
| `regression-test-from-bug` | Fail first, name the test after the defect |
| `characterization-testing` | Pinning legacy behavior before changing it |
| `choosing-test-scope` | Unit, integration, contract, end-to-end — and what only each catches |
| `test-driven-development` † | The red-green-refactor cycle itself |

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
| `systematic-debugging` † | The general hypothesis-driven loop |

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

## Security (defensive)

| Skill | For |
|---|---|
| `threat-modeling` | A model that fits in a design review |
| `handling-untrusted-input` | Parse don't validate; the injection family as one bug |
| `secrets-hygiene` | Where secrets must never be; revoke-then-rotate |
| `auditing-dependencies` | Code you ship and did not review; triage by reachability |
| `least-privilege-design` | Default deny; blast radius as the design metric |

## Communicating

| Skill | For |
|---|---|
| `writing-durable-docs` | The four doc types; documenting why; deleting stale docs |
| `writing-adrs` | Recording a decision so a future reader can tell if it still applies |
| `writing-release-notes` | Written for the person deciding whether to upgrade |
| `writing-postmortems` | Blameless in mechanism; contributing factors over root cause |
| `reviewing-code-deeply` | Reviewing in priority order; finding what is absent |
| `explaining-technical-work` | Conclusion first; altitude chosen by what the reader will do |
| `requesting-code-review` † · `receiving-code-review` † | The two sides of a review |
| `writing-plans` † · `executing-plans` † | Plan authoring and execution |

## Environment and tooling

| Skill | For |
|---|---|
| `portable-shell-scripting` | Quoting, `set -e` exemptions, GNU vs BSD, when to stop using shell |
| `reproducible-environments` | The "works on my machine" taxonomy; verifying by building twice |
| `designing-ci-pipelines` | Cost-to-signal ordering; cache correctness; required vs advisory |
| `upgrading-dependencies` | Routine and cheap, or one forced upgrade during an incident |
| `keeping-tbaguette-current` | Checks the installed TBaguette plugin against the published repo, updates it if it can fast-forward cleanly, keeps a local changelog |
| `automating-repetition` | When a manual sequence should become a tool, and when it should not |
| `dispatching-parallel-agents` † · `subagent-driven-development` † | Delegation patterns |
| `writing-skills` † · `using-superpowers` † | Authoring and using skills |
| `play-console` † | Google Play Console workflows |
