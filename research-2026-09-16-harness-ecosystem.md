# Harness ecosystem research — 2026-09-16

Window covered: 2026-09-14 through 2026-09-16. No prior-format report exists in
this directory to match against (`research-2026-09-14-harness-ecosystem.md`
was not found on disk despite the brief referencing it), so this one sets its
own structure: headline events, then leads ordered by strength, then
coverage-check results against existing skills, then non-skill proposals.

## Headline events

1. **SEP-2640 ("Skills over MCP") merged — confirmed via GitHub API, not a
   search summary.** `gh`/`curl` against
   `api.github.com/repos/modelcontextprotocol/modelcontextprotocol/pulls/2640`
   returns `"state": "closed", "merged": true, "merged_at":
   "2026-09-13T21:27:51Z", "merged_by": "localden"`. This resolves the
   09-12/13 "still a draft" status from prior passes — it is no longer a
   draft. Caveat worth keeping: the WG's own charter page
   (`modelcontextprotocol.io/community/working-groups/skills-over-mcp`)
   still lists the item as "In Review" in its tracking table, but that page's
   changelog hasn't been touched since 2026-04-25 — it's a stale secondary
   page, not a contradicting authority. Trust the PR API state over the
   charter page. Practical meaning for TBaguette: MCP now has a ratified
   Extensions-Track convention (`skill://` resources, `skills/list`,
   `skills/get`) for serving skills *from an MCP server* to any compliant
   client — a fundamentally different distribution path than the
   plugin-manifest path every current TBaguette integration uses. Nothing to
   build yet (no evidence any of Claude Code/Codex/Copilot/Antigravity/Devin
   consume it as a client), but it's the first time this mechanism has been
   spec-final rather than experimental, so it's worth a listing check again
   in ~2-4 weeks for client adoption.

2. **Codex's skill-description truncation is a shared, catalogue-wide
   round-robin budget, not a fixed per-skill limit — this is new and
   materially changes the picture on the already-tracked 8,000-byte issue.**
   Confirmed via `github.com/eranroseman/agent-plugins/issues/49`: Codex
   allocates "2% of the model's context window (8,000 characters when
   unknown; `skills.max_context_tokens` caps it at 10,000 tokens)" across the
   *entire installed catalogue* of skills, not per skill. When the catalogue
   doesn't fit, "the remainder after each entry's name and path is handed
   out one character per skill per round-robin pass." The widely-cited
   "122-character" truncation figure people have been quoting is not a
   constant — it's what that budget happened to divide out to on a
   112-skill catalogue at one point in time; the same reporter measured 652
   untruncated characters on a 70-skill catalogue. **This is a different
   limit from the 8,000-byte `SKILL.md` body cap already tracked as P1** —
   that one truncates the file Codex loads on invocation; this one truncates
   the one-line `description` shown in the skill *listing* Codex uses to
   decide whether to invoke a skill at all, and it gets worse as TBaguette
   (or a user's total installed-skill count) grows, not better. A 97-skill
   plugin sitting in a user's Codex install alongside other plugins is
   exactly the worst case this mechanism describes.

## Leads, strongest first

### 1. Deadbugz: a malicious MCP server that passes review by delaying its payload past the review window (STRONG, multi-sourced)

Documented independently by Pillar Security, nhimg.org, and byteiota (three
separate write-ups converging on the same mechanism and timeline), plus an
adversa.ai security-digest roundup that treats it as the month's headline MCP
incident:

- The server ships two innocuous tools. Its tool metadata/descriptions stay
  clean for exactly the first three tool calls in a session, then rewrite
  themselves into instructions to hunt for SSH keys, AWS credentials, shell
  history, and Kubernetes config, while suppressing visibility of the
  change to the user.
- Distribution was via 23 GitHub PRs opened across a 74-minute window
  (2026-08-10, ~21:52–23:07 UTC) adding the server to unrelated repos' MCP
  config.
- The mechanism defeats the two most common lightweight review practices at
  once: a human skim of the tool description (clean at review time) and a
  brief automated smoke-test (passes, because it's under three calls).

**Agnostic core:** a trust decision made once, at connection or install time,
is being tested against an artifact that is allowed to have *state* — the
thing you approved and the thing that runs on call four are not
provably the same artifact, and nothing about "I reviewed it" or "I ran it
once and it looked fine" bounds what a stateful, call-counting payload can
still do later in the same session.

**Coverage check — this is the interesting part.** TBaguette's
`auditing-dependencies` already anticipates the general shape of this problem
closely, and better than I expected before checking. Its frontmatter
explicitly triggers on "a connected tool provider's descriptions can change
after you approved them," and the body (`## When the payload is prose rather
than code`) has a dedicated mitigation:

> "Record what the instructions said when you approved them and compare
> them on reconnect, so a change is something you see rather than something
> you obey... And keep the set connected at one time no larger than the
> task needs..."

That mitigation is keyed on **reconnect** as the event that triggers a
re-check. Deadbugz's trigger is **call count within one still-open
connection** — there is no reconnect for a "compare on reconnect" policy to
catch, because the session that gets compromised never drops. This is a real,
narrow gap rather than a restatement of something covered: the skill correctly
identifies that tool metadata is mutable and untrusted, and correctly proposes
diffing it against a recorded baseline, but the *event* it diffs on
(reconnection) doesn't fire for the attack that just appeared in the wild.
Worth a small addition to `auditing-dependencies` noting that the same diff
needs to run periodically or after N calls within a live session, not only on
reconnect — since "the connection never dropped" is precisely how Deadbugz
gets past a reconnect-triggered check. Flagging as a real gap, not proposing
the edit myself since this pass is research-only.

### 2. arXiv: MCP error responses don't carry enough structured data for a client to decide how to recover (MODERATE — single paper, but a clean empirical result)

`arxiv.org/abs/2609.00072`, "Can MCP Clients Decide What to Do After Failure?
A Result-Only Actionability Audit" (found via the arXiv listing-page route,
which worked where the API kept 429ing on prior passes — `arxiv.org/list/cs.SE/2026-09`
is the working path, worth reusing next time instead of retrying the API).
Across 21 induced MCP tool failures, typed error fields exposed the failure
in 18 cases, but "rarely make concrete recovery or safe replay
self-contained" — prose descriptions frequently carried more of the
actionable information than the structured fields did, pushing clients toward
parsing natural language to decide whether a retry is safe.

**Agnostic core:** a typed/structured error field that only says *that*
something failed, without also saying whether it's safe to retry and safe to
replay, forces every caller back onto parsing the prose message — which is
exactly the failure mode structured errors exist to prevent.

**Coverage check — already covered, not a gap.** `modeling-errors` already
teaches this as a first-class distinction. Its class table (line 26) has a
row for "Infrastructure / transient" errors defined as ones "carrying a
retryable marker and a hint," and its guidance on message shape (line 65)
explicitly models a good error as one that states what was retryable and
when: `` `connect to config store at <addr>: connection refused (retryable,
retry after 2s)` ``. Its red-flags section even names the failure mode this
paper measured: "Retry loop burns quota and never succeeds | Non-retryable
class retried; no retryable marker in the error." This arXiv result reads as
an empirical confirmation that real MCP implementations fail to follow advice
`modeling-errors` already gives, not as new territory — worth citing as a
concrete, dated instance if that skill ever wants a real-world example rather
than a gap to fill.

### 3. Devin: skills are now distributable as centrally-governed plugins, but the "no session-start hook" conclusion in PORTING.md still holds (WEAK-to-none, confirms rather than changes)

Devin's docs (`docs.devin.ai/product-guides/skills`) and release notes now
describe skills shippable "via Plugins that can be installed and governed
centrally, then applied consistently across Devin Cloud, Devin CLI, and Devin
Desktop." This sounds at first read like it might upgrade Devin from "skill
discovery without a bootstrap" to something portable — checked directly
against two sources (the docs page and a targeted search on Devin's hook
event types) and it doesn't change PORTING.md's verdict. Devin still has no
documented session-start-equivalent hook: activation is still either the
model's own relevance judgment ("Devin determines a skill is relevant to the
current task") or an explicit `@skills:skill-name` mention, and the only hook
events findable are task/worktree-scoped (`post_setup_worktree` etc.), not
session-scoped. **Second source found, and it confirms the existing PORTING.md
finding rather than contradicting it** — worth noting in the report per the
brief's ask, even though the answer is "no change."

## Coverage-check results, summarized

| Finding | Skill checked | Verdict |
|---|---|---|
| Deadbugz call-count-gated malicious MCP server | `auditing-dependencies` | Real gap: "compare on reconnect" doesn't fire for an attack that never triggers a reconnect. Worth a small addition, not made here. |
| MCP error responses lack recovery metadata | `modeling-errors` | Already covered — retryable-marker guidance and the exact red flag this paper measured are both already in the file. |
| Devin plugin-based skill distribution | `PORTING.md` (not a skill, the porting doc) | Confirms existing verdict (no session-start hook exists); no change needed. |
| "Enforcement belongs outside the model" (sandboxing/gateway consensus, several 2026 security write-ups) | `least-privilege-design` | Already covered — the skill's whole frame is annotate-don't-trust and containment tested by attempting to leave, matching the "don't trust tool descriptions/model behavior as the enforcement point" consensus cited this pass. |

## Not skill candidates — repo/tooling facts worth recording once

- **Codex catalogue-budget truncation (headline #2 above)** is a fact about
  Codex's own listing renderer, not a portable engineering lesson — it
  belongs in `PORTING.md`'s Codex row and possibly `scripts/run_tests.py`'s
  existing size-reporting logic (which already tracks the 8,000-byte body
  cap and could be extended to flag description length against catalogue
  size), not in a skill file skills describe actions, not vendor quirks.
- **AGENTS.md was donated to the Linux Foundation's Agentic AI Foundation**
  (alongside MCP and goose) in December 2025 and is now read natively by
  Codex, Cursor, Copilot, Gemini CLI, Aider, Windsurf, Zed, Factory, Jules,
  Devin, and 20+ others, with 60,000+ repos using it. Nothing actionable for
  TBaguette here — it's an instructions-file standard for a *target repo*,
  orthogonal to the skill-packaging problem TBaguette solves — but worth
  having the current number on hand since it keeps coming up as a reference
  point in porting discussions.
- **Antigravity CLI plugin-reinstall bugs were fixed** (reinstalling now
  replaces the managed directory cleanly; installing a plugin from its own
  installed directory is now refused instead of corrupting it; uninstall no
  longer leaves stale `config.json` entries). None of this affects the
  "no Antigravity manifest shipped yet" status PORTING.md already records —
  these are bug fixes to an install path TBaguette doesn't use yet, not new
  requirements for the manifest that still needs writing.
- **Copilot CLI's `copilot skill add`** confirmed still current
  (`<FILE | URL | DIRECTORY>`, plus `/skills add` and `/skills reload`
  in-session) — matches what PORTING.md already assumes for that row, no
  drift found.
- **Claude Code 2.1.268–2.1.273** shipped `claude plugin eval` (already
  known), plus `/output-style`, gateway pricing config, a startup
  security-issue warning, `gatewayInternalNetworks`, forkable remote-control
  sessions, and an MCP-server-disconnect notification improvement. Nothing
  here changes plugin/skill loading semantics that TBaguette's bootstrap
  depends on.

## Second-source check requested by the brief

- SEP-2640 status: **resolved definitively** via GitHub API (not just a
  second source — the authoritative one). See Headline #1.
- Devin skill-invocation model (task-context-triggered, no forced injection):
  **second source found** (docs page + targeted hook-event search), and it
  confirms rather than overturns the existing PORTING.md conclusion.
- Arxiv agent-reliability research access path: **fixed this pass** — the
  API's 429s are avoided by going through `arxiv.org/list/<category>/<YYMM>`
  listing pages instead, which returned real, current (2026-09) results on
  the first try. Worth using this path by default going forward instead of
  re-attempting the API.
