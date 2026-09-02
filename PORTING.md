# Porting TBaguette's Atelier to a new harness

This is a condensed adaptation of the `superpowers` plugin's own
[porting guide](https://github.com/obra/superpowers/blob/main/docs/porting-to-a-new-harness.md)
— credit there for the methodology and the vocabulary this document reuses
(shapes A/B/C, the capability checklist). The Atelier's own harness layer was
built by following it. If you're adding harness #13, read the source guide
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
| runs a shell command at session start and reads its stdout | A (shell-hook) | `.cursor-plugin/` + `hooks/hooks-cursor.json`, or root `plugin.json` + `hooks/hooks-copilot.json` |
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
| Codex | `.codex-plugin/plugin.json` + `hooks/hooks-codex.json` (installed with `codex plugin marketplace add LeSplooch/tbaguette-skills`, reusing `.agents/plugins/marketplace.json`) | shell hook → `hooks/session-start`, plus per-turn `hooks/user-prompt-submit` — Codex's hook config, stdout shape and `CLAUDE_PLUGIN_ROOT` are all Claude Code's | none needed |
| Cursor | `.cursor-plugin/plugin.json` + `hooks/hooks-cursor.json` | shell hook → `hooks/session-start cursor`, plus a throttled re-assertion on `postToolUse` → `hooks/user-prompt-submit cursor` | none needed (Claude Code–compatible tool surface) |
| GitHub Copilot CLI | root `plugin.json` + `hooks/hooks-copilot.json` (installed with `copilot plugin marketplace add LeSplooch/tbaguette-skills` then `copilot plugin install TBaguette@tbaguette-dev`, reusing `.claude-plugin/marketplace.json` — the CLI reads that location too) | shell hook → `hooks/session-start copilot`, plus per-turn `hooks/user-prompt-submit copilot` | `skills/using-tbaguette/references/copilot-tools.md` |
| Copilot in VS Code | root `plugin.json` + `com.github.copilot/hooks/hooks.json` (installed with the **Chat: Install Plugin From Source** command and this repo's git URL) | shell hook → `hooks/session-start vscode`, plus per-turn `hooks/user-prompt-submit vscode` | same file as the CLI |
| Copilot coding agent | root `plugin.json`, enabled per repository in that repo's `.github/copilot/settings.json` (see below) | the CLI's `hooks/hooks-copilot.json`, run in the cloud sandbox — only the `bash` field is honored there | same file as the CLI |
| Devin | `.devin-plugin/plugin.json` | Devin's own `skills/` convention | none shipped |
| Gemini CLI | `gemini-extension.json` + `GEMINI.md` | instructions file `@`-include of `using-tbaguette` | none shipped |
| Kimi Code | `.kimi-plugin/plugin.json` | manifest `sessionStart.skill` loads `using-tbaguette` | inline `skillInstructions` |
| OpenCode | `.opencode/plugins/tbaguette.js` (declared via root `package.json` `main`) | in-process: `config` hook registers skills dir, `experimental.chat.messages.transform` injects context | inline in `tbaguette.js` |
| Pi | `.pi/extensions/tbaguette.ts` (declared via root `package.json`'s `pi` field) | in-process: resource discovery registers skills, a context event injects bootstrap | inline in `tbaguette.ts` |
| Hermes Agent | `.hermes-plugin/plugin.yaml` + `.hermes-plugin/__init__.py` (installed with `hermes plugins install LeSplooch/tbaguette-skills`) | in-process: `register()` registers every skill with the native loader, a `pre_llm_call` hook injects the bootstrap on the first turn | `skills/using-tbaguette/references/hermes-tools.md` |

When in doubt, read the files, not this table — same rule the source guide
gives, still true here.

### Enabling it for the Copilot coding agent

The coding agent installs declaratively, in the repository it will work on
rather than in this one. Both fields are objects keyed by name, not arrays,
and the marketplace has to be registered because this one is not known by
default:

```json
{
  "extraKnownMarketplaces": {
    "tbaguette-dev": {
      "source": { "source": "github", "repo": "LeSplooch/tbaguette-skills" },
      "autoUpdate": true
    }
  },
  "enabledPlugins": {
    "TBaguette@tbaguette-dev": true
  }
}
```

That goes in the target repository's `.github/copilot/settings.json`. This repo
deliberately does not ship one of its own: a settings file here would enable the
plugin for anyone whose coding agent touches *this* repository, which is a
decision for them to make in theirs.

### What a docs-only audit of every row turned up

Every integration in the table above was re-read against its harness's own
documentation, with no live instance to test against. Three of them were
delivering nothing, and none of the three looked broken from inside this
repository — the files existed, the JSON was valid, the hooks exited 0.

- **Cursor was inert.** Its `sessionStart` reads a flat, snake_case
  `{"additional_context": ...}`. It was being handed Claude Code's
  `hookSpecificOutput`, which it ignores. A hook ran on every session, exited
  0, and delivered nothing — for as long as the integration had existed.
- **Cursor cannot have a literal per-turn hook.** `beforeSubmitPrompt` returns
  `{continue, user_message}`: `continue` gates the turn and `user_message` is
  addressed to the human. Only `sessionStart` and `postToolUse` accept
  `additional_context`. So the re-assertion rides `postToolUse` and throttles
  itself per conversation, which is a different cadence rather than a worse
  one — the decay it exists to fix happens in long agentic runs, and long
  agentic runs are made of tool calls.
- **Codex had its own bootstrap switched off.** The manifest shipped
  `"hooks": {}`, empty on purpose, to stop Codex default-discovering Claude
  Code's `hooks/hooks.json`. It worked, and it cost the entire integration.
  Codex's hook config shape, its stdout shape and even its
  `CLAUDE_PLUGIN_ROOT` variable are Claude Code's, so the answer was a hook
  file of its own rather than no hooks at all. Codex now has both a
  session-start bootstrap and a per-turn nudge.

Three others were checked and left alone, which is worth recording so nobody
re-audits them from scratch:

- **Gemini CLI is the strongest of the lot.** `@./`-imports in a context file
  are real (the Memory Import Processor, relative paths, depth limit 5), so
  `GEMINI.md`'s one-line include resolves. Better still, Gemini concatenates
  its context files into *every prompt* — it has per-turn re-injection for
  free, without a hook.
- **Kimi Code's manifest is correct.** `skills`, `sessionStart.skill` and
  `interface` are all real fields; a second, older-looking Kimi plugin doc
  describes a tools-only format with none of them, and it is not the one that
  applies. `skillInstructions` is the one field that could not be confirmed
  either way.
- **OpenCode and Pi are the fragile pair.** Both adapters hang off APIs that
  are explicitly experimental — OpenCode's `experimental.chat.messages.transform`
  is not in the published plugin-hook list, and plugins registering
  `experimental.*` hooks have been reported silently breaking across minor
  versions. Nothing is wrong with them today. They are the two most likely to
  stop working without anyone noticing, because their failure mode is exactly
  the one this audit kept finding: a bootstrap that runs and delivers nothing.

And the last two, which is the whole table:

- **Devin discovers the skills and will never be told to check them.** Its
  `.devin-plugin/plugin.json` plus `SKILL.md` directories is a real format, and
  Devin reads skills from `.agents/skills/`, `.devin/skills/` and
  `~/.config/devin/skills/`. But its own documentation is explicit that skills
  "don't run automatically" — they are chosen from task context. There is no
  session-start hook to attach a bootstrap to, and the nearest thing, Devin's
  Knowledge, lives in the user's own repository, which rule 2 puts off limits.
  So Devin is the one row here that fails the hard requirement outright, and
  the honest description is skill discovery without a bootstrap. Nothing was
  shipped for it, because there is nothing to ship.
- **Hermes was a version, not a mechanism.** Its `pre_llm_call` bootstrap is
  sound and its `__init__.py` already refuses to start rather than silently
  skip when it cannot find the skills tree. What it had was
  `plugin.yaml` sitting at 0.6.0 against a plugin at 1.0.28 — silently, for
  exactly the reason `package.json` once drifted five minor versions, which is
  that nothing compared them. It is now compared, by the same test, with a
  hand-rolled reader rather than a new dependency.

The rule this audit produced, worth stating on its own: **the tests proved
consistency and never proved correctness.** Every one of these files was valid,
every version matched, every hook exited 0, and three integrations delivered
nothing to the model. A suite that only compares this repository against itself
cannot see any of that. Where a fact came from a harness's documentation, the
test that guards it now says so in its own docstring, so the next person can
tell a checked fact from a copied assumption.

### What the three Copilot rows are, and are not, verified against

All three are built from GitHub's documentation rather than from a driven live
instance, which is what the source guide asks for and what this port could not
do. Worth re-checking the first time anyone runs each for real:

- **`${PLUGIN_ROOT}` in a plugin-shipped hook command.** Documented both as a
  token the harness expands and as an environment variable it sets. Written
  here with a fallback chain, so that if neither turns out to be true the
  failure mode is Cursor's cwd-relative assumption rather than a path rooted at
  `/`. The VS Code file falls back through `CLAUDE_PLUGIN_ROOT` first, since
  VS Code is documented as setting that one.
- **A manifest `name` with capitals in it.** `TBaguette` is kept because the
  plugin name is what prefixes every skill (`/TBaguette:naming-things`), and
  because `.claude-plugin/plugin.json` — a manifest location both surfaces
  document reading — has always carried it.
- **What VS Code reads off a hook's stdout.** Its own docs describe the events
  and say nothing about the output shape. `session-start vscode` emits Claude
  Code's envelope and Copilot's side by side so either reader finds its key;
  `user-prompt-submit vscode` deliberately does not, for the reason below.

Worth knowing before copying the per-turn hook: because the CLI's route in is
`modifiedPrompt`, the nudge becomes part of the prompt rather than sitting
beside it, so a CLI user can see it in their own transcript. Claude Code's
`additionalContext` is invisible in the same position. That is a real cost, paid
because the alternative is no per-turn reminder at all — and it is exactly why
the VS Code branch, where the right shape is unknown, takes the shape that
cannot rewrite a prompt instead of emitting both and hoping.

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
  and a `.copilot-plugin/` would have been the consistent choice — no
  Copilot surface would ever have looked in it. Worse, the two lists are
  not the same list: the CLI searches `.plugin/plugin.json`, `plugin.json`,
  `.github/plugin/plugin.json`, `.claude-plugin/plugin.json`, while VS Code
  searches root `plugin.json`, `.claude-plugin/plugin.json`,
  `.plugin/plugin.json` and does **not** read `.github/plugin/` at all. The
  repo root is the only entry on both, which is the whole reason the
  manifest sits there. Intersect the discovery rules of every surface you
  mean to serve before picking a path; consistency you can't be found in
  isn't consistency.
- **Two surfaces of one product can resolve the same manifest differently.**
  The CLI honors an explicit `hooks` path in the manifest. VS Code ignores it
  entirely and derives the path from the detected plugin format — Agent
  Plugins 1.0 means `com.github.copilot/hooks/hooks.json`, Claude format
  means `hooks/hooks.json`, Copilot format means a root `hooks.json`. That is
  why the `$schema` line in `plugin.json` is load-bearing rather than
  decorative, and why there are two Copilot hook files with different event
  casing rather than one. Neither file is reachable from the other's
  surface.
- **A harness that reads `.claude-plugin/plugin.json` is not thereby
  installed.** Copilot CLI reads that file, which makes "it already works"
  tempting and wrong. That manifest declares no `hooks`, so Copilot falls
  through to its own default hook location — `hooks/hooks.json`, which
  exists here in *Claude Code's* schema. The result isn't a harness with no
  bootstrap; it's a harness pointed at a config file it cannot read. Give
  the harness its own manifest naming its own hook file, so default
  discovery never reaches the wrong one.
- **A hook that runs is not a hook that works.** This is the single failure
  this repository has now shipped three times, on three harnesses, and it
  never looks like a failure: the file exists, the JSON parses, the script
  exits 0, and the harness silently ignores an output shape it does not
  recognise. `additionalContext`, `additional_context`, nested under
  `hookSpecificOutput` or flat — four harnesses, four answers. Read the
  harness's own output schema before writing the hook, and treat "the hook
  ran" as worth nothing on its own.
- **Check what each hook event lets you return, not just that the event
  exists.** Copilot CLI has a per-turn hook, which looked like a free win
  over Cursor. It does not accept `additionalContext` — `userPromptSubmitted`
  returns `modifiedPrompt`, so the only way to add a nudge is to hand back
  the user's own prompt with the nudge prepended. That turns a hook that
  could previously only fail to help into one that can actively destroy a
  turn, which is why `hooks/user-prompt-submit`'s copilot branch treats every
  parse failure as "emit `{}` and exit 0." Any port that reaches for a
  prompt-rewriting hook owes the same fail-open. Cursor is the sharper case:
  its per-prompt event has no context field at all, so the only honest options
  were a different event or none.
