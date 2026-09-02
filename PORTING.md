# Porting TBaguette's Atelier to a new harness

This is a condensed adaptation of the `superpowers` plugin's own
[porting guide](https://github.com/obra/superpowers/blob/main/docs/porting-to-a-new-harness.md)
— credit there for the methodology and the vocabulary this document reuses
(shapes A/B/C, the capability checklist). The Atelier's own harness layer was
built by following it. If you're adding harness #11, read the source guide
in full for the parts condensed away here (live-instance verification via a
driven TUI, distribution/release mechanics, Windows specifics, PR process);
this file covers the invariants and points you at the Atelier's own reference
implementations to copy instead of superpowers'.

## How this works across harnesses

The Atelier's content is the same everywhere. What changes per harness is a
thin delivery layer:

1. **Skills (harness-agnostic).** Everything in `skills/` is the source of
   truth, shared verbatim. Skills describe *actions* — "invoke a skill",
   "read a file", "run a shell command" — not specific tool names. A grep
   across all 93 confirms this holds almost without exception (`using-tbaguette`
   is the one skill that names a tool, "the Skill tool" — see
   `.kimi-plugin/plugin.json`'s `skillInstructions`, or
   `skills/using-tbaguette/references/copilot-tools.md`, for how that one
   sentence gets mapped on a harness that has no such tool).
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
per-session opt-in from the user. If the only way to get the Atelier in
front of the model is to paste a prompt or flip a mode by hand each
session, it can't be properly supported — full stop, before writing
anything.

**Strongly wanted, not required: a per-turn re-injection point.** Session
start alone is measurably not enough. Across real Claude Code sessions,
the start-of-session notice landed 95% of the time, yet only ~42% of
substantive sessions ever invoked an Atelier skill — sessions running
hundreds of turns would invoke one, or none. A single message at position
zero loses against a long context no matter how forcefully it's worded, so
where a harness offers a per-prompt hook, use it: Claude Code does this
with `UserPromptSubmit` → `hooks/user-prompt-submit`, a short nudge rather
than a second copy of the full `SKILL.md`. A harness without one is still
supportable, just weaker in long sessions.

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
| runs a shell command at session start and reads its stdout | A (shell-hook) | `.cursor-plugin/` + `hooks/hooks-cursor.json`, or `.github/plugin/` + `hooks/hooks-copilot.json` |
| is a JS/TS plugin host with session/message lifecycle callbacks | B (in-process) | `.opencode/` — or `.pi/` if it has no native skill tool |
| ships an extension-declared context file it always loads | C (instructions-file) | `gemini-extension.json` + `GEMINI.md` |

Most harnesses fit one row cleanly. Shape B additionally needs the
repo-root `package.json` (`main` for an OpenCode-style loader, the `pi`
field for a Pi-style one) — a JS/TS adapter file that nothing declares is
never loaded.

## The Atelier's current reference integrations

| Harness | Entry point | Bootstrap mechanism | Tool mapping |
|---|---|---|---|
| Claude Code | `.claude-plugin/plugin.json` + `hooks/hooks.json` | shell hook → `hooks/session-start`, plus per-turn `hooks/user-prompt-submit` | native `Skill` tool; no adapter needed |
| Codex | `.codex-plugin/plugin.json` (empty `hooks`) | native skill discovery, no session-start hook | none shipped yet — no Atelier skill has needed one so far |
| Cursor | `.cursor-plugin/plugin.json` + `hooks/hooks-cursor.json` | shell hook → `hooks/session-start`; no per-turn hook wired up yet | none needed (Claude Code–compatible tool surface) |
| GitHub Copilot CLI | `.github/plugin/plugin.json` + `hooks/hooks-copilot.json` (installed with `copilot plugin marketplace add LeSplooch/tbaguette-skills` then `copilot plugin install TBaguette@tbaguette-dev`, reusing `.claude-plugin/marketplace.json` — Copilot CLI reads that location too) | shell hook → `hooks/session-start copilot`, plus per-turn `hooks/user-prompt-submit copilot` | `skills/using-tbaguette/references/copilot-tools.md` |
| Devin | `.devin-plugin/plugin.json` | Devin's own `skills/` convention | none shipped |
| Gemini CLI | `gemini-extension.json` + `GEMINI.md` | instructions file `@`-include of `using-tbaguette` | none shipped |
| Kimi Code | `.kimi-plugin/plugin.json` | manifest `sessionStart.skill` loads `using-tbaguette` | inline `skillInstructions` |
| OpenCode | `.opencode/plugins/tbaguette.js` (declared via root `package.json` `main`) | in-process: `config` hook registers skills dir, `experimental.chat.messages.transform` injects context | inline in `tbaguette.js` |
| Pi | `.pi/extensions/tbaguette.ts` (declared via root `package.json`'s `pi` field) | in-process: resource discovery registers skills, a context event injects bootstrap | inline in `tbaguette.ts` |
| Hermes Agent | `.hermes-plugin/plugin.yaml` + `.hermes-plugin/__init__.py` (installed with `hermes plugins install LeSplooch/tbaguette-skills`) | in-process: `register()` registers every skill with the native loader, a `pre_llm_call` hook injects the bootstrap on the first turn | `skills/using-tbaguette/references/hermes-tools.md` |

When in doubt, read the files, not this table — same rule the source guide
gives, still true here.

Two things about the Copilot row are built from GitHub's documentation rather
than from a driven live instance, which is what the source guide asks for and
what this port could not do. Both are worth re-checking the first time anyone
runs it for real. One: `${PLUGIN_ROOT}` in a plugin-shipped hook command —
documented as both a token the harness expands and an environment variable it
sets, and written here as `${PLUGIN_ROOT:-.}` so that the failure mode, if
neither turns out to be true, is Cursor's cwd-relative assumption rather than a
path rooted at `/`. Two: whether Copilot accepts a manifest `name` with capitals
in it. `TBaguette` is kept because the plugin name is what prefixes every skill
(`/TBaguette:naming-things`), and because `.claude-plugin/plugin.json` — a
manifest location Copilot documents reading — has always carried it.

Worth knowing before copying the per-turn hook: because Copilot's route in is
`modifiedPrompt`, the nudge becomes part of the prompt rather than sitting
beside it, so a Copilot user can see it in their own transcript. Claude Code's
`additionalContext` is invisible in the same position. That is a real cost, paid
because the alternative is no per-turn reminder at all.

## Gotchas carried over from the source guide

- **Opt-in isn't a port.** If a person has to do anything per session to
  get the Atelier in front of the model, it doesn't count.
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
- **The harness's manifest search order beats this repo's naming
  convention.** Every other integration here lives in `.<harness>-plugin/`,
  and a `.copilot-plugin/` would have been the consistent choice — Copilot
  CLI would simply never have looked in it. Its search order is fixed
  (`.plugin/plugin.json`, `plugin.json`, `.github/plugin/plugin.json`,
  `.claude-plugin/plugin.json`), so the manifest goes in `.github/plugin/`.
  Read the harness's own discovery rules before picking a directory name;
  consistency you can't be found in isn't consistency.
- **A harness that reads `.claude-plugin/plugin.json` is not thereby
  installed.** Copilot CLI reads that file, which makes "it already works"
  tempting and wrong. That manifest declares no `hooks`, so Copilot falls
  through to its own default hook location — `hooks/hooks.json`, which
  exists here in *Claude Code's* schema. The result isn't a harness with no
  bootstrap; it's a harness pointed at a config file it cannot read. Give
  the harness its own manifest naming its own hook file, so default
  discovery never reaches the wrong one.
- **Check what each hook event lets you return, not just that the event
  exists.** Copilot CLI has a per-turn hook, which looked like a free win
  over Cursor. It does not accept `additionalContext` — `userPromptSubmitted`
  returns `modifiedPrompt`, so the only way to add a nudge is to hand back
  the user's own prompt with the nudge prepended. That turns a hook that
  could previously only fail to help into one that can actively destroy a
  turn, which is why `hooks/user-prompt-submit`'s copilot branch treats every
  parse failure as "emit `{}` and exit 0." Any port that reaches for a
  prompt-rewriting hook owes the same fail-open.
