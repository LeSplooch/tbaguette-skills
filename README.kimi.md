# TBaguette's Atelier for Kimi Code

Complete guide for using the Atelier with [Kimi Code](https://github.com/MoonshotAI/kimi-code).

## Installation

Install directly from this repository:

```text
/plugins install https://github.com/LeSplooch/tbaguette-skills
```

Kimi Code applies plugin changes to new sessions. After installing, updating,
enabling, disabling, or reloading a plugin, start a fresh session with `/new`.

## How It Works

The Kimi plugin manifest lives at `.kimi-plugin/plugin.json`.

The manifest does three things:

1. Points Kimi Code at the existing `skills/` directory.
2. Loads `using-tbaguette` at session start through `sessionStart.skill`.
3. Provides Kimi-specific tool mapping through `skillInstructions`.

Kimi Code reads the Atelier's skills from this repository. There are no copied
skills, symlinks, hooks, or extra runtime dependencies.

## Tool Mapping

Skills describe actions instead of hard-coding one runtime's tool names.
The Atelier's skills are, with one exception, written this way already — a
grep across the whole library found only `using-tbaguette` naming a
specific tool (`Skill`). On Kimi Code:

- "Invoke a skill" -> Kimi Code's native `Skill` tool
- "Read a file" / "write a file" / "edit a file" -> `Read`, `Write`, `Edit`
- "Run a shell command" -> `Bash`
- "Search file contents" -> `Grep`
- "Find files by path or pattern" -> `Glob`

If a future Atelier skill needs to dispatch a subagent, track a todo, or
ask the user a multiple-choice question, `.kimi-plugin/plugin.json`'s
`skillInstructions` already covers the mapping (`Agent`, `TodoList`,
`AskUserQuestion` respectively) — extend that block rather than this file
if the mapping ever needs to grow.

## Updating

Use Kimi Code's plugin manager:

```text
/plugins
```

Select TBaguette's Atelier and update it from there. Start a fresh session with
`/new` after updating.

## Troubleshooting

### Plugin not loading

1. Run `/plugins info TBaguette` and check diagnostics.
2. Make sure the plugin is enabled.
3. Start a fresh session with `/new` after install or update.

### Skills not triggering

1. Confirm `/plugins info TBaguette` shows the plugin enabled.
2. Start a fresh session with `/new`.
3. Try an acceptance prompt that should trigger a skill by description —
   e.g. asking to review unfamiliar code should load
   `orienting-in-unfamiliar-code` before any other action.
