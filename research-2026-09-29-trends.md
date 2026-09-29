# Trends research — 2026-09-29

A pass over what the agent-skills ecosystem is installing, what practitioners
now recommend for working with coding agents, which agent incidents of
June–September 2026 carry a lesson, and what engineers shipping LLM-backed
features have converged on. Each finding was checked against the 98 skills
on disk before anything was written; the verdicts are below, followed by what
shipped and what was deliberately left for later. The previous report of this
kind is `research-2026-09-16-harness-ecosystem.md`, and nothing it covers is
repeated here (Deadbugz, SEP-2640, Codex's listing budget).

## What shipped from this pass

Three new skills, all gaps with strong, multi-source evidence:

| Skill | Why it is a gap | Strongest evidence |
|---|---|---|
| `writing-agent-instructions` | Nothing said what belongs in an instruction file; `recovering-agent-context` even told agents to append recovered history to one | Gloaguen et al., "Evaluating AGENTS.md" (arXiv 2602.11988, rev. 2026-06-23): context files do not generally raise success, cost +20%, overviews ineffective, instructions followed well. dos Santos et al., "Configuration Smells in AGENTS.md Files" (arXiv 2606.15828): lint leakage 62%, bloat 42%, skill leakage 35% of 100 popular repos. superpowers v6.2.0 (2026-07-24): removing arguments dropped test-first compliance 8/10 → 5/10 |
| `evaluating-llm-output` | No skill mentioned evals, judges, or model output at all | Husain & Shankar evals FAQ (hamel.dev/blog/posts/evals-faq); Anthropic, "Demystifying evals for AI agents" (2026-01-09); Shankar et al., criteria drift (arXiv 2404.12272); Thinking Machines on inference nondeterminism (2025-09-10); Anthropic, statistical approach to evals (2024-11-19) |
| `building-llm-features` | The general forms existed (pinning, parse-don't-validate, budgets, redaction) but none was applied to a model call | Provider deprecation pages (retirement = hard failure date); structured-output docs (shape, not bounds; no guarantee on refusal/truncation); OpenTelemetry GenAI semconv (still "Development", content opt-in); OWASP LLM Top 10 2025 (LLM05, LLM06, LLM07, LLM10) |

Targeted additions to existing skills:

| Skill | Addition | Evidence |
|---|---|---|
| `secrets-hygiene` | A thief still resident on the machine changes "revoke first": isolate, remove persistence, then revoke from elsewhere | Shai-Hulud 2.0 (GitLab, 2025-11-24; token-watcher wiping `$HOME` on 4xx); Keyv/ChainDrop (Datadog, THN, 2026-08) with the same watcher |
| `designing-ci-pipelines` | Agent jobs reading attacker-writable text get the fork rules; agent config in a checkout is a pipeline definition | Comment and Control (2026-04), GitLost (Noma, 2026-07), Clinejection (2026-02); TrustFall (Adversa, 2026-05-07), Keyv worm committing hooks |
| `deciding-reversibility`, `bounding-autonomous-work` | Read the target out of the credential before writing; a found credential and a credential mismatch are stops | PocketOS (2026-04-24; korben.info, incidentdatabase.ai report 7311); Prisma shadow-database incident (single vendor source) |
| `auditing-dependencies` | Attestation proves origin, not intent; a skill's package names must resolve | TanStack / Mini Shai-Hulud with valid SLSA L3 provenance (slsa.dev blog 2026-05, CSA); react-codeshift in 237 repos via two skills (Aikido, CSO Online) |
| `confirming-before-claiming-done`, `reviewing-code-deeply` | Diff the checks, not only the code; the list of ways a check is made to pass | ImpossibleBench (arXiv 2510.20270); SpecBench (arXiv 2605.21384); Anthropic prompting guidance on hard-coding to tests; Osmani, "Agentic Code Review" (2026-06-15) |
| `designing-apis` | A section for callers that are models | Anthropic, "Writing effective tools for AI agents" (2025-09-11); Carlini, parallel-Claude C compiler (2026-02-05); OpenAI, "Harness engineering" (2026-02-11) |
| `handing-off-for-review` | Read an upstream project's AI-contribution policy before sending it anything | Hora, Robbes & Zacchiroli (arXiv 2609.07542): 281 policies, 67.3% require substantial human involvement, 48.8% require disclosure |
| `fanning-out-independent-work` | Width of a fan-out is set by review capacity | Osmani, "The Orchestration Tax" (2026-05-24); CodeRabbit AI-vs-human PR study (2025-12-17) |
| `testing-the-untestable`, `upgrading-dependencies`, `handling-untrusted-input` | Hosted model is not seedable; a model id is a dependency with an expiry; model output is an untrusted source | As for `building-llm-features` |

## Checked and already covered

| Finding | Where it already lives |
|---|---|
| Keep the checker separate from the builder (Anthropic harness-design post, 2026-03-24; Böckeler, guides vs sensors) | `delegating-tasks-with-review-gates`, `confirming-before-claiming-done` |
| Spec before building, sized to the change (Kiro, spec-kit, Tessl) | `scoping-before-building`, `finishing-what-you-started` |
| Long-running agents keep state outside the context | `checkpointing-long-runs` |
| Context engineering and sub-agent isolation | `fanning-out-independent-work`, `crouton` |
| An agent rewriting its own boundary (Kiro MCP config, sandbox-escape week) | `threat-modeling`, `bounding-autonomous-work` |
| Hidden Unicode in skills and rules files | `auditing-dependencies`, `handling-untrusted-input` |
| Prompt caching needs stable-first ordering | `caching-strategy` |
| Popular skills — tdd, diagnosing-bugs, code-review, handoff, brainstorming, frontend-design, caveman | `writing-the-failing-test-first`, `diagnosing-before-fixing`, `reviewing-code-deeply`, `checkpointing-long-runs`, `scoping-before-building`, `formidable`, `crouton` |

## Candidates left for a later pass, with what would change the answer

- **Prose without machine tells** (humanizer ~53k stars, stop-slop, unslop).
  Strong demand from several authors. It overlaps `crouton` and
  `explaining-technical-work` enough that the first question is whether it is
  a section in one of them or a skill of its own. Would move on: a reading of
  those two skills against the tells catalogues.
- **Technical diagrams** (diagram-design ~43k stars, Anthropic's own
  diagramming skill). Agnostic core: choose the diagram by the question, not
  the layout, and make every node earn its place. Would move on: a second
  vendor-neutral source that isn't a single repository.
- **A shared domain glossary** and **deep modules / architecture passes**
  (mattpocock/skills, about 1M installs each, but one author). Partly in
  `naming-things` and `drawing-boundaries`. Would move on: the same idea
  from an unrelated author.
- **Handing over understanding, not just a diff**: cognitive and intent debt
  (Anthropic RCT, 2026-01-29; Storey, arXiv 2603.22106; Radar Vol 34 lists
  codebase cognitive debt under Caution). Nothing in the library treats the
  owner's understanding as a deliverable. Probably a section in
  `explaining-technical-work`.
- **TDD inside the agent loop**: Böckeler (martinfowler.com, 2026-08-10)
  measured no quality gain at 3–8.5× the tokens, and agents skipping the red
  step; Willison and Anthropic still recommend it. One small study against
  standing practice. `writing-the-failing-test-first` already requires
  watching the test fail. A revalidation should wait for a second
  controlled result.

## Findings about the library itself (tooling, not skills)

- **Description budget.** Claude Code gives the whole skill listing about 1%
  of the context window and drops the least-used descriptions first; the 101
  descriptions total roughly 70,000 characters. Codex's round-robin budget
  (previous report) is the same pressure. Shorter descriptions would help more
  than further workarounds.
- **Body length versus compaction.** After a compaction Claude Code
  re-attaches only the first ~5,000 tokens of each invoked skill (25,000
  total). Eighteen skill bodies are longer than that by a characters/4
  estimate, `tending-tbaguette` (~14k tokens) most of all. `writing-durable-docs`
  already says what survives is the opening; these numbers say where the cut
  falls.
- **No skill has an eval set.** `claude plugin eval` runs cases with and
  without the plugin, three times each, and can grade triggering. A small set
  per high-traffic skill would let the library measure whether a skill still
  changes behavior on current models — the test `writing-agent-instructions`
  now asks of every instruction file, applied to this one.
- **Side-effecting skills could opt out of automatic invocation**
  (`disable-model-invocation`), which would also take them out of the listing
  budget. Candidates: `keeping-tbaguette-current`, the contribution half of
  `tending-tbaguette`. Needs checking against how `using-tbaguette` invokes
  them before changing anything.
- **Porting lead, unverified:** one source reports Gemini CLI stopped
  serving individual accounts on 2026-06-18 in favour of Antigravity CLI, and
  superpowers dropped Gemini support in v6.1.0. `gemini-extension.json` and
  `GEMINI.md` still ship here. Worth a check against Google's own docs before
  touching `PORTING.md`.
