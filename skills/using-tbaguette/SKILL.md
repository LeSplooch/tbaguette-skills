---
name: using-tbaguette
description: Use at the start of every conversation, in every project, and keep it in mind for the rest of the conversation — not just the first message, and not only at the top of a long one. Establishes that TBaguette's own skills must be checked for relevance before every response, and again once a response has run long enough that its opening check has gone stale, the same way this notice itself got surfaced. Also use when a check for a covering skill has just come back empty, or when the skill listing looks shorter than the library it is supposed to describe.
---

# Using TBaguette

<EXTREMELY-IMPORTANT>
If there is even a small chance one of TBaguette's skills applies to what you're about to do, invoke it — with the Skill tool, as `TBaguette:<skill-name>`. This holds for every response for the rest of this conversation, not just the one right after this notice.
</EXTREMELY-IMPORTANT>

## The rule

Before responding — including a clarifying question, "let me look at the code first," or anything that feels too small to bother — check whether a TBaguette skill covers it. You don't need to open a file to see what exists: every `TBaguette:*` skill and its trigger description is normally already listed in your available-skills context — with one failure mode, immediately below, that this library is big enough to hit. `CATALOG.md` in this repo has the longer versions when a one-line trigger isn't enough to judge relevance.

If a skill turns out not to fit once you're in it, that's fine — drop it. But check first, every time.

## The listing may be shorter than the library

That rule rests on a claim which is usually true and quietly stops being true at
about the size this library has reached: that every skill's trigger description is
in front of you. The listing has a character budget — in Claude Code it is a fraction of the
context window, one percent by default — and a library of a hundred skills, each
carrying a deliberately trigger-rich description, is the shape that exceeds it.

What happens then is the part worth knowing. The **names** all survive. The
**descriptions** are what gets dropped, and they are dropped starting with the
skills invoked least — so the trigger text vanishes from precisely the skills
nobody has been reaching for, which are precisely the ones you were relying on the
listing to surface. Nothing announces it. Every skill is still listed, the check
still runs, and it returns "nothing here covers this" with complete confidence.
The failure is invisible to the inspection that would catch it, which is the same
shape `routing-around-capability-gaps` describes one level down for a tool whose
schema has not been fetched.

Two things follow. Entries that are bare names, or descriptions noticeably shorter
than the paragraph-length ones this library writes, are the tell — and `CATALOG.md`
is already the answer the rule above points at, so read it rather than concluding
the library is silent on a question it covers.

The other is that this is fixable at the source rather than worked around. On
Claude Code, `/doctor` estimates the listing's cost and names its biggest
contributors, `/context`'s Skills row reports its size after the budget is
applied, and `--debug` logs a warning when it overflows. Raise the budget with the
`skillListingBudgetFraction` setting or the `SLASH_COMMAND_TOOL_CHAR_BUDGET`
environment variable; where the listing is shared with other libraries, set the
entries you never route on to `"name-only"` in `skillOverrides` to free room for
the ones you do. Run one of those once rather than assuming it all fits.

## A long response needs a second check, not only the first

The rule above is per *response*, and it quietly assumes a response is short. Most are: in one
measured harness the median response ran four tool calls, which a check at the top governs
comfortably. The tail does not behave. The 90th percentile was 19 calls, the 99th was 49, and one
response reached 194. By call 190 the check that opened the response is not a guardrail, it is
scrollback.

That is where this discipline actually fails, and it fails quietly — nothing about a long response
announces that its opening judgment has gone stale. Fifteen consecutive substantive responses in
that harness invoked nothing at all, while the notice fired correctly at the top of every one of
them. The guidance was never missing; it was simply hundreds of actions old by the time each
decision it governs got made.

So the check is owed twice: before the response, and again once a response has grown past the point
where you would still call its opening recent. If you want a trigger that does not depend on
noticing, a response that has run a couple of dozen tool calls and invoked nothing has drifted,
whatever it feels like from the inside.

Where the harness fires an event at the *end* of a response, that is the place to wire this —
`automating-repetition` owns the judgment about turning a habit into machinery, and a check whose
whole failure mode is "the reminder scrolled away" is near the top of that list. Two cautions, both
from measuring one: fire only on a response that is **both** large and skill-free, or it becomes
furniture — the obvious threshold fired on 22% of all responses. And confirm the discriminator
separates anything before trusting it; in that harness *every* large response had invoked nothing,
so size alone distinguished nothing at all.

## When the work is bigger than one response

One skill covers one stretch of work. Anything that will take several — a feature, a bug with no known cause, a migration, an audit — needs them in an order, and that order is `orchestrating-work-end-to-end`: which track the request is on, which phase it is in, what evidence opens the next gate, and where the run record lives so a compaction doesn't cost the run. Invoke it before the first action, not after the first three.

When several skills apply at once, they go in this order: the spine first, because it says which phase this is; then that phase's owner skill; then whatever the work's own content calls for. "Build me X" starts at `scoping-before-building`, not in an editor. "Fix this bug" starts at `diagnosing-before-fixing`, not at a patch.

## Red flags

Thoughts that mean stop and check anyway:

| Thought | Reality |
|---|---|
| "This is too small for a skill" | Small changes are exactly where the guardrails (naming, commit hygiene, test scope) get skipped first. |
| "I already know how to do this" | Knowing the general shape of a task isn't the same as this library's specific judgment calls. |
| "I'll check after I've looked at the code" | Several skills (`orienting-in-unfamiliar-code`, `reading-specifications`) are about *how* to look, not what to do once you have. |
| "It's just a question, not a task" | Questions are tasks. Check for skills. |
| "I'm deep into this response, checking now breaks the flow" | A response long enough for that thought is exactly the one its opening check no longer covers. Invoking costs one call. |

## Alongside other plugins

If another plugin also injects a "check skills first" notice — Superpowers' `using-superpowers`, for example — both apply at once. This one governs TBaguette's own library specifically and says nothing about anyone else's.

## Platform adaptation

If you're running on a harness other than Claude Code, read its reference file for special instructions:

- GitHub Copilot — the CLI, VS Code, and the coding agent: `references/copilot-tools.md`
- Hermes Agent: `references/hermes-tools.md`

Copilot is on that list for one reason: it has no Skill tool on any of its three surfaces, so the sentence above telling you to invoke `TBaguette:<skill-name>` with one names something you cannot find. There a skill is a slash command, and skills also load on their own when a prompt matches their description.

Other harnesses TBaguette ships a manifest for (Codex, Cursor, Devin, Gemini CLI, Kimi Code, OpenCode, Pi) don't currently need a separate reference file here — their tool mapping either lives inline in that harness's own manifest (Kimi's `skillInstructions`) or needs none at all, since most TBaguette skills describe actions rather than naming a specific tool. See `PORTING.md` at the repo root for the full harness-by-harness breakdown.

## Automatic update check

If a `TBaguette:keeping-tbaguette-current` update-check block is attached below this notice, act on it per that skill's instructions — the network check already ran for this session, so don't repeat it.
