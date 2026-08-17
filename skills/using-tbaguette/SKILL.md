---
name: using-tbaguette
description: Use at the start of every conversation, in every project, and keep it in mind for the rest of the conversation — not just the first message. Establishes that TBaguette's own skills must be checked for relevance before every response, the same way this notice itself got surfaced.
---

# Using TBaguette

<EXTREMELY-IMPORTANT>
If there is even a small chance one of TBaguette's skills applies to what you're about to do, invoke it — with the Skill tool, as `TBaguette:<skill-name>`. This holds for every response for the rest of this conversation, not just the one right after this notice.
</EXTREMELY-IMPORTANT>

## The rule

Before responding — including a clarifying question, "let me look at the code first," or anything that feels too small to bother — check whether a TBaguette skill covers it. You don't need to open a file to see what exists: every `TBaguette:*` skill and its trigger description is already listed in your available-skills context. `CATALOG.md` in this repo has the longer versions when a one-line trigger isn't enough to judge relevance.

If a skill turns out not to fit once you're in it, that's fine — drop it. But check first, every time.

## Red flags

Thoughts that mean stop and check anyway:

| Thought | Reality |
|---|---|
| "This is too small for a skill" | Small changes are exactly where the guardrails (naming, commit hygiene, test scope) get skipped first. |
| "I already know how to do this" | Knowing the general shape of a task isn't the same as this library's specific judgment calls. |
| "I'll check after I've looked at the code" | Several skills (`orienting-in-unfamiliar-code`, `reading-specifications`) are about *how* to look, not what to do once you have. |
| "It's just a question, not a task" | Questions are tasks. Check for skills. |

## Alongside other plugins

If another plugin also injects a "check skills first" notice — Superpowers' `using-superpowers`, for example — both apply at once. This one governs TBaguette's own library specifically and says nothing about anyone else's.

## Platform adaptation

If you're running on a harness other than Claude Code, read its reference file for special instructions:

- Hermes Agent: `references/hermes-tools.md`

Other harnesses TBaguette ships a manifest for (Codex, Cursor, Devin, Gemini CLI, Kimi Code, OpenCode, Pi) don't currently need a separate reference file here — their tool mapping either lives inline in that harness's own manifest (Kimi's `skillInstructions`) or needs none at all, since most TBaguette skills describe actions rather than naming a specific tool. See `PORTING.md` at the repo root for the full harness-by-harness breakdown.

## Automatic update check

If a `TBaguette:keeping-tbaguette-current` update-check block is attached below this notice, act on it per that skill's instructions — the network check already ran for this session, so don't repeat it.
