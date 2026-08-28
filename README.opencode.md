# TBaguette's Atelier for OpenCode

Complete guide for using the Atelier with [OpenCode.ai](https://opencode.ai).

## Installation

Add the Atelier to the `plugin` array in your `opencode.json` (global or
project-level):

```json
{
  "plugin": ["tbaguette-skills@git+https://github.com/LeSplooch/tbaguette-skills.git"]
}
```

Restart OpenCode. The plugin installs through OpenCode's plugin manager and
registers all skills.

Verify by asking: "Tell me about your TBaguette skills"

OpenCode uses its own plugin install. If you also use Claude Code, Codex, or
another harness, install the Atelier separately for each one — see
[README.md](README.md) for the Claude Code install command.

## Usage

### Finding Skills

Use OpenCode's native `skill` tool to list all available skills:

```
use skill tool to list skills
```

### Loading a Skill

```
use skill tool to load orienting-in-unfamiliar-code
```

### Project and Personal Skills

The Atelier's skills sit alongside whatever OpenCode already discovers in
`~/.config/opencode/skills/` (personal) or `.opencode/skills/` (project) —
those still take priority over the Atelier's if a name collides.

## Updating

OpenCode installs the Atelier through a git-backed package spec. Some
OpenCode and Bun versions pin that resolved git dependency in a lockfile or
cache, so a restart may not pick up the newest Atelier commit. If updates
do not appear, clear OpenCode's package cache or reinstall the plugin.

To pin a specific commit or tag instead of tracking `master`:

```json
{
  "plugin": ["tbaguette-skills@git+https://github.com/LeSplooch/tbaguette-skills.git#<ref>"]
}
```

## How It Works

The plugin does two things:

1. **Injects bootstrap context** via the `experimental.chat.messages.transform` hook, adding Atelier awareness to every conversation.
2. **Registers the skills directory** via the `config` hook, so OpenCode discovers all Atelier skills without symlinks or manual config.

### Tool Mapping

Skills speak in actions rather than naming any one runtime's tools. On
OpenCode these resolve to:

- "Invoke a skill" → OpenCode's native `skill` tool
- "Read a file" → `read`
- "Create a file" / "edit a file" / "delete a file" → `apply_patch`
- "Run a shell command" → `bash`
- "Search file contents" / "find files by name" → `grep`, `glob`

## Troubleshooting

### Plugin not loading

1. Check OpenCode logs: `opencode run --print-logs "hello" 2>&1 | grep -i tbaguette`
2. Verify the plugin line in your `opencode.json` is correct
3. Make sure you're running a recent version of OpenCode

### Skills not found

1. Use OpenCode's `skill` tool to list available skills
2. Check that the plugin is loading (see above)
3. Each skill needs a `SKILL.md` file with valid YAML frontmatter

### Bootstrap not appearing

1. Check OpenCode version supports `experimental.chat.messages.transform` hook
2. Restart OpenCode after config changes

## Getting Help

- Report issues: https://github.com/LeSplooch/tbaguette-skills/issues
- Main documentation: https://github.com/LeSplooch/tbaguette-skills
- Site: https://lesplooch.github.io/tbaguette-skills/
- OpenCode docs: https://opencode.ai/docs/
