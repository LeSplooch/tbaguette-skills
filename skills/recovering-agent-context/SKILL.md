---
name: recovering-agent-context
description: Use when picking up an existing project for the first time, resuming work that someone or something else started, inheriting a repo that shows signs of prior AI agent work, or being told to continue or finish a task with no handoff attached. Covers sweeping every assistant that touched the repo rather than only your own, finding session transcripts and instruction files by shape when vendor paths have moved, searching huge transcript stores without reading them whole, telling an executed plan from an abandoned one, and turning what you find into a resume brief.
---

# Recovering agent context

## Overview

Someone worked on this repo before you, and on an existing project it was rarely a person alone. Git records what shipped. Agent sessions record what was tried and thrown away, which options were rejected and why, and every instruction the human gave that never became a file — the three things that separate resuming from restarting.

The same sessions also record plans that were never executed, APIs that never existed, and decisions reversed four turns later, in identical prose. What you recover is testimony, not ground truth.

## When to use

- First contact with an existing repo, running alongside `orienting-in-unfamiliar-code`.
- The task is phrased as *continue*, *finish*, or *pick up where it left off*, with no handoff attached.
- An instruction file, plan directory, or spec folder exists that nobody in this conversation wrote.
- You are about to re-derive a decision, retry an approach, or ask the human something they have plainly answered before.
- Not for: why one line exists (`code-archaeology`), what the code does (`orienting-in-unfamiliar-code`), or the coupling graph (`mapping-dependencies`).

## Check every provider, and not your own first

The default failure is searching your own tool's store, finding a session, and stopping there. Your own history is the *least* likely to hold what you are missing — if you had done this work, you would know about it. The gap is in the tool you do not use.

Enumerate every store before opening any. Then:

- **Do not stop at the first hit.** Two or three assistants on one repo is ordinary now, and they will contradict each other. The contradiction is the finding, not an obstacle to it.
- **Absence is a finding.** An `AGENTS.md` with no matching session store means the work happened on another machine, in a browser, or under someone else's account. Report that. It is a different fact from "no prior work exists", and only one of them means you can proceed as if the repo were new.
- **A store you cannot read still counts.** Hosted agents, web chats, and a colleague's laptop are unreachable from here. That is one specific question for the human, asked once: *was any of this driven from somewhere I can't see?* If nobody is there to answer, record it as a known gap rather than blocking on it — an unanswered question and a settled one are different, and only one of them is safe to forget.
- **The tool that left the least residue is often the one that did the most.** IDE assistants and cloud agents write far less into the repo than CLI agents do, and a clean tree is not evidence nothing happened in it.

## Cheapest signal first

| # | Look at | Why it ranks here | Cost |
|---|---|---|---|
| 1 | Root instruction files — `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `CONVENTIONS.md`, the `.*rules` family, `.github/copilot-instructions.md` | Already-distilled conversation; each line is there *because* it kept being repeated | One read each |
| 2 | In-repo agent artifacts — tool dotdirs, saved chat exports, spec and steering folders, plan directories under `docs/` | Deliberate, scoped to this repo, already summarized by whoever wrote them | Minutes |
| 3 | Git history for agent fingerprints — co-author trailers, generated-with footers, bot committers, bursts of same-minute commits | Tells you *which* tools ran and *when*, which is how you pick what to open in row 4 | One command |
| 4 | Host session stores keyed by this repo's absolute path | The full record, including everything the rows above chose not to write down | Search only, never read whole |
| 5 | Cloud-only histories | Unreachable from here | One question |

Rows 1–3 are usually enough. Reaching row 4 for everything means rows 1–3 were skipped, not that they were empty. Row 3 is this:

```bash
git log --format='%an <%ae>%n%b' | grep -iE 'co-authored-by|generated with|noreply' | sort -u
```

## Three shapes, not thirty paths

Any list of concrete paths starts rotting the day it is written; vendors rename directories and ship new tools constantly. Find stores by shape instead, and treat the map below as a starting point you verify rather than a fact you trust.

**In the repo.** Instruction files at the root, dotdirs named for the tool, and — with Aider in particular — a complete plain-text transcript sitting in the working directory. Read `.gitignore` as evidence in its own right: an ignored `.aider*`, `.specstory`, or `.claude` entry proves a tool ran here even after every file it wrote is gone, and it survives the cleanup that removed them.

**A tool dotdir under `$HOME`.** CLI agents keep one JSON or JSONL file per session, almost always tagged with the working directory it ran in. `~/.codex/sessions/`, `~/.claude/projects/`, `~/.gemini/tmp/`, `~/.continue/sessions/`, `~/.copilot/`, `~/.aws/amazonq/`, `~/.codeium/`, `~/.local/share/goose/sessions/`, `~/.local/share/opencode/`.

**A VS Code-family SQLite blob.** Nearly every agentic IDE — Cursor, Windsurf, Antigravity, Kiro, and the extensions Cline, Roo, and Copilot — is a VS Code fork or lives inside one, so chat lands in `…/User/workspaceStorage/<hash>/state.vscdb` and `…/User/globalStorage/<extension-id>/`. Application support directories are `~/.config/<App>/` on Linux, `~/Library/Application Support/<App>/` on macOS, `%APPDATA%\<App>\` on Windows.

The `<hash>` is opaque, and that stops most searches. It is not a dead end: each `workspaceStorage/<hash>/` holds a `workspace.json` naming the folder URI, which maps hash to repo directly.

```bash
grep -rl "$(pwd)" ~/.codex ~/.claude ~/.gemini ~/.continue ~/.copilot ~/.codeium \
  ~/.local/share ~/.config/*/User/workspaceStorage 2>/dev/null | head -50
sqlite3 state.vscdb .tables
sqlite3 state.vscdb "select key from ItemTable where key like '%chat%'"
```

List the tables before querying them: the schema differs between `workspaceStorage` and `globalStorage`, and naming a table that store does not have fails the whole statement rather than returning nothing — an empty result you would otherwise read as "no chat history here".

None of this depends on a vendor keeping a path stable, which is why it outlives the lists above.

## Searching a store without drowning in it

Session stores run to hundreds of megabytes of JSONL, and a single session can exceed your entire context. Reading one whole is not thoroughness, it is the mistake.

- **Newest first, and stop early.** Recency beats completeness here; the last two sessions usually contain the state you need, and older ones describe a codebase that no longer exists.
- **Grep for anchors, not topics.** A function name, a file path, an error string, a dependency version. Topic words match every session ever recorded.
- **Read the tail, not the head.** The opening of a session is the human's request, which you can usually infer. The end holds where it actually stopped, and that is the part with no other source.
- **Extract, then close.** Pull the specific exchange into your notes and drop the file. Never carry a transcript forward "in case".
- **Delegate a wide sweep.** When the search spans many stores or many sessions, delegate it if your harness can — the search is megabytes and the finding is a paragraph.

## An executed plan and an abandoned one read identically

Transcripts are a record of *intent and attempt*, and nothing in the format marks which attempts landed. Every one of these appears in confident, well-formed prose:

| What you read | What may actually be true |
|---|---|
| A detailed implementation plan | Never started; the session ended right after it was written |
| "I've updated the handler to…" | The edit failed, was reverted, or was made in a different branch |
| A firm architectural decision | Reversed by the human six turns later, in one line |
| An API, flag, or config key used throughout | Hallucinated, and the session ended before anyone ran it |
| A confident final summary | Written from the agent's intentions, not from the diff |

So: **every claim gets checked against the working tree before you act on it.** Does the file exist, does the symbol exist, does the test pass, is the commit in `git log`. This is fast — it is the cheapest verification in this whole skill — and skipping it is how you inherit a hallucination and then defend it as a prior decision.

Read the human's turns with more weight than the agent's — but know which turns those are. Most formats record tool output as a `user` turn as well, so filtering naively on the user role returns command output, file contents, and error text alongside the handful of messages a person actually typed. The human's real turns are the short ones between the long machine ones.

Corrections, refusals, and stated preferences are first-hand and durable; an agent's account of its own work is neither. When the two conflict, the human's last word wins. Carry the distinction into whatever you report — see `calibrating-confidence` for saying which is which.

## What you are extracting

Not a summary of the sessions. Five things; anything fitting none of them stays out of the brief:

1. **Where it stopped** — the last thing actually completed, verified against the tree, and the first thing that was not.
2. **Decided, with the reason** — choices already made and still standing in the code. Re-opening one of these without new information is pure waste.
3. **Already paid for** — approaches tried and abandoned, and what went wrong. The single highest-value category, and the only one that exists nowhere else.
4. **Standing corrections** — what the human told a previous agent to do or stop doing. These outlive the session and apply to you now.
5. **Open** — questions raised and never answered. Carry them forward as questions, not as assumptions.

Then write it into the repo's own instruction file, so the next session inherits the brief instead of repeating the dig. A recovery you keep to yourself is one you will pay for again — and you are, right now, generating exactly the residue the next agent will be looking for.

## Boundaries

These stores are the human's private conversations, most of them about other projects, other clients, other employers.

- Search scoped to this repo's path. Sessions matched to other directories are not yours to read, however easy the grep would be.
- Assume secrets are in there verbatim — pasted keys, tokens, `.env` dumps, customer data. Do not echo transcript content into a commit message, PR, issue, log, or any outbound message.
- Committed agent artifacts in a shared repo carry a colleague's half of a conversation. Use what they establish about the code; do not read their side of it back to them.
- Say which store each finding came from. "A prior Codex session decided X" and "the repo's `AGENTS.md` says X" carry very different weight.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Rebuilt something that already existed | Never looked; the tree had residue from a tool you don't use |
| Re-litigated a settled decision | Read the code, not the sessions where the alternatives were rejected |
| Implemented against an API that doesn't exist | Trusted a transcript claim without checking the tree |
| Repeated an approach that failed before | Dead ends are recorded only in sessions, and only if you read them |
| Context window gone before starting work | Read whole transcripts instead of grepping for anchors |
| Human repeats a correction they gave weeks ago | The correction was in another tool's session and never written to a file |
| "No prior agent work here" — and there was | Checked your own provider only |
| Confidently resumed at the wrong point | Trusted a final summary written from intentions rather than from a diff |

## Red flags

- "I checked my own history, there's nothing." — you checked one store out of a dozen.
- "There's an `AGENTS.md`, so I've got the context." — that file is the residue, not the record.
- "The transcript says it was implemented." — check the tree, then say it.
- "I'll read the whole session to be thorough."
- "That decision looks wrong, I'll redo it." — before finding out whether it was already tried the other way.
- "No point looking, the repo's clean." — IDE and cloud assistants leave nearly nothing behind.
- Quoting a plan without checking whether it survived the session it was written in.
