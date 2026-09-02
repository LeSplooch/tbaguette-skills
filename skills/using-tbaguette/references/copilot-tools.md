# GitHub Copilot CLI Tool Mapping

Almost nothing in this library needs mapping. TBaguette's skills describe
*actions* — read a file, run a command, dispatch a subagent — and Copilot CLI
has all of those under its own names, which it already tells you about. The one
place the abstraction leaks is the sentence that matters most: how to reach
another skill. Claude Code has a `Skill` tool. Copilot CLI does not. That
difference, and the few things downstream of it, is what this file is for.

## Invoking a skill

Two routes, and the first one is doing most of the work already:

1. **Automatic.** Copilot CLI loads a skill when the prompt matches its
   `description:` frontmatter. Every TBaguette skill is written so that its
   description *is* its trigger, which means the library is largely
   self-dispatching here. This is why `using-tbaguette`'s rule — check before
   responding — still holds even with no tool to call.
2. **Deliberate.** Invoke one by name as a slash command:

   ```
   /TBaguette:orienting-in-unfamiliar-code
   /TBaguette:karen-and-the-manager
   ```

   The `TBaguette:` prefix comes from the plugin name and is added by Copilot
   itself. Do not go looking for it inside any `SKILL.md` — a skill that writes
   a prefix into its own `name:` field fails to load, silently.

Where a TBaguette skill says "invoke `TBaguette:<skill-name>` with the Skill
tool," read it as route 2.

## If a skill will not load

Fall back to reading it. Copilot CLI has file read and shell, so this always
works and needs nothing registered:

```
skills/<skill-name>/SKILL.md
```

relative to wherever the plugin was installed. `CATALOG.md` at the plugin root
lists every skill with its full trigger description, which is the better file to
read when the question is *which* skill rather than *what does this one say*.

## Instructions file

When a skill refers to "your instructions file," on Copilot CLI that is
`AGENTS.md` or `.github/copilot-instructions.md` in the repository, and
`copilot-instructions.md` under `~/.copilot/` for the global one. If
`COPILOT_HOME` is set, the global one lives there instead.

One constraint worth knowing before you write an `@`-include into any of them:
Copilot CLI will not follow an absolute path or a `~/`-rooted one, and the
target has to stay inside the repository (or inside the custom-instructions
directory, for the global file). An include pointing at a plugin installed
elsewhere on the machine does not resolve.

## Subagents

Copilot CLI has custom agents — `*.agent.md` files, dispatched as subagents.
Where a skill asks for a subagent (`fanning-out-independent-work`,
`delegating-tasks-with-review-gates`), use that mechanism. Where it is
unavailable, every one of those skills already carries its own fallback: do the
work inline, in sequence, rather than inventing a dispatch that will not run.
Same rule for todo tracking and web fetch — degrade, don't improvise.

## What is deliberately not here

There is no table of Copilot CLI's tool names in this file, because writing one
would be guessing at names that the harness already puts in front of you
accurately, and a stale mapping is worse than none. Use the tools you actually
have, by the names you are actually given.
