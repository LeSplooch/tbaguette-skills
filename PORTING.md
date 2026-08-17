# Porting TBaguette to a new harness

This is a condensed adaptation of the `superpowers` plugin's own
[porting guide](https://github.com/obra/superpowers/blob/main/docs/porting-to-a-new-harness.md)
— credit there for the methodology and the vocabulary this document reuses
(shapes A/B/C, the capability checklist). TBaguette's own harness layer was
built by following it. If you're adding harness #10, read the source guide
in full for the parts condensed away here (live-instance verification via a
driven TUI, distribution/release mechanics, Windows specifics, PR process);
this file covers the invariants and points you at TBaguette's own reference
implementations to copy instead of superpowers'.

## How this works across harnesses

TBaguette's content is the same everywhere. What changes per harness is a
thin delivery layer:

1. **Skills (harness-agnostic).** Everything in `skills/` is the source of
   truth, shared verbatim. Skills describe *actions* — "invoke a skill",
   "read a file", "run a shell command" — not specific tool names. A grep
   across all 87 confirms this holds almost without exception (`using-tbaguette`
   is the one skill that names a tool, "the Skill tool" — see
   `.kimi-plugin/plugin.json`'s `skillInstructions` for how that gets
   mapped per harness).
2. **Tool mapping (per-harness).** Translates the action vocabulary into
   the harness's real tool names — inline in a manifest
   (`.kimi-plugin/plugin.json`) or the bootstrap injector itself.
3. **Bootstrap (per-harness).** At session start, `using-tbaguette`'s full
   `SKILL.md` gets injected into the model's context. **The bootstrap is
   the entire integration** — without it, the skill files are inert, present
   on disk but never invoked.

### Two rules

1. **Skills name actions, not tools.** Never edit `skills/*/SKILL.md` to
   fit a harness. Porting adds a tool-mapping layer; it doesn't rewrite
   skill content.
2. **Ship through the harness's own install mechanism. Never edit a user's
   personal files.** The bootstrap and tool mapping travel as part of what
   the harness installs (a plugin, an extension, a marketplace entry) — never
   as an edit to `~/.bashrc`, a global config file, or anything outside
   what this repo's own install artifacts declare.

## Can a harness be supported?

**Hard requirement: automatic session-start injection**, with no
per-session opt-in from the user. If the only way to get TBaguette in
front of the model is to paste a prompt or flip a mode by hand each
session, it can't be properly supported — full stop, before writing
anything.

Beyond that, check the harness has: file read/write/edit (essential), shell
command execution (essential), and ideally skill discovery + on-demand
loading (if absent, the fallback is reading the relevant `SKILL.md`
directly). Subagent dispatch, todo tracking, and web fetch are all
*degradable* — skills that use them already carry fallback wording for
when the capability is missing.

Before building anything: check whether the harness can simply load an
existing manifest instead of needing a new one (e.g. a harness that
consumes the Claude Code plugin format directly via its own install
command needs nothing new here).

## Choosing a shape

| If the harness… | Use shape | Copy from |
|---|---|---|
| runs a shell command at session start and reads its stdout | A (shell-hook) | `.cursor-plugin/` + `hooks/hooks-cursor.json` |
| is a JS/TS plugin host with session/message lifecycle callbacks | B (in-process) | `.opencode/` — or `.pi/` if it has no native skill tool |
| ships an extension-declared context file it always loads | C (instructions-file) | `gemini-extension.json` + `GEMINI.md` |

Most harnesses fit one row cleanly. Shape B additionally needs the
repo-root `package.json` (`main` for an OpenCode-style loader, the `pi`
field for a Pi-style one) — a JS/TS adapter file that nothing declares is
never loaded.

## TBaguette's current reference integrations

| Harness | Entry point | Bootstrap mechanism | Tool mapping |
|---|---|---|---|
| Claude Code | `.claude-plugin/plugin.json` + `hooks/hooks.json` | shell hook → `hooks/session-start` | native `Skill` tool; no adapter needed |
| Codex | `.codex-plugin/plugin.json` (empty `hooks`) | native skill discovery, no session-start hook | none shipped yet — no TBaguette skill has needed one so far |
| Cursor | `.cursor-plugin/plugin.json` + `hooks/hooks-cursor.json` | shell hook → `hooks/session-start` | none needed (Claude Code–compatible tool surface) |
| Copilot CLI | shares the Claude Code hook path | shell hook → `hooks/session-start` | none needed | 
| Devin | `.devin-plugin/plugin.json` | Devin's own `skills/` convention | none shipped |
| Gemini CLI | `gemini-extension.json` + `GEMINI.md` | instructions file `@`-include of `using-tbaguette` | none shipped |
| Kimi Code | `.kimi-plugin/plugin.json` | manifest `sessionStart.skill` loads `using-tbaguette` | inline `skillInstructions` |
| OpenCode | `.opencode/plugins/tbaguette.js` (declared via root `package.json` `main`) | in-process: `config` hook registers skills dir, `experimental.chat.messages.transform` injects context | inline in `tbaguette.js` |
| Pi | `.pi/extensions/tbaguette.ts` (declared via root `package.json`'s `pi` field) | in-process: resource discovery registers skills, a context event injects bootstrap | inline in `tbaguette.ts` |

When in doubt, read the files, not this table — same rule the source guide
gives, still true here.

## Gotchas carried over from the source guide

- **Opt-in isn't a port.** If a person has to do anything per session to
  get TBaguette in front of the model, it doesn't count.
- **Hook-config schema varies per harness.** Cursor's `hooks-cursor.json`
  looks nothing like Claude Code's (`version`, lowercase `sessionStart`,
  relative command, no `matcher`/`type`/`async`). Match the closest
  existing file, not a different one.
- **Message-object shape is per-harness** for in-process plugins (Shape B).
  OpenCode and Pi use incompatible shapes — discover yours, don't copy the
  other's object literal.
- **A harness with no skill system** (not just no `Skill` tool) has nothing
  to register — the model reads `SKILL.md` on demand instead. Don't go
  looking for a registration API that doesn't exist.
