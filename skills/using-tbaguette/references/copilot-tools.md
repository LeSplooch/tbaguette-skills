# GitHub Copilot Tool Mapping

Covers all three surfaces TBaguette installs into: Copilot CLI, Copilot in
VS Code, and the Copilot coding agent. Where they differ, it says so; where
nothing below distinguishes them, they behave the same.

Almost nothing in this library needs mapping. TBaguette's skills describe
*actions* — read a file, run a command, dispatch a subagent — and Copilot CLI
has all of those under its own names, which it already tells you about. The one
place the abstraction leaks is the sentence that matters most: how to reach
another skill. Claude Code has a `Skill` tool. Copilot CLI does not. That
difference, and the few things downstream of it, is what this file is for.

## Invoking a skill

Two routes, and the first one is doing most of the work already. Both work the
same on all three surfaces:

1. **Automatic.** Copilot loads a skill when the prompt matches its
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

When a skill refers to "your instructions file," the repository-level answer is
the same everywhere: `AGENTS.md`, or `.github/copilot-instructions.md`, or
`.github/instructions/*.instructions.md`. The global one is where they part
company — the CLI reads `copilot-instructions.md` under `~/.copilot/` (or under
`COPILOT_HOME`, if that is set), VS Code has its own user-level equivalent, and
the coding agent has neither, because it runs with no user home to read from.
Write to the repository file when the instruction has to hold on all three.

One constraint worth knowing before you write an `@`-include into any of them:
an absolute path or a `~/`-rooted one is not followed, and the target has to
stay inside the repository (or inside the custom-instructions directory, for a
global file). An include pointing at a plugin installed elsewhere on the machine
does not resolve.

## Subagents

Copilot has custom agents — `*.agent.md` files, dispatched as subagents. Where a
skill asks for a subagent (`fanning-out-independent-work`,
`delegating-tasks-with-review-gates`), use that mechanism. Where it is
unavailable, every one of those skills already carries its own fallback: do the
work inline, in sequence, rather than inventing a dispatch that will not run.
Same rule for todo tracking and web fetch — degrade, don't improvise.

## One thing the coding agent changes about every other skill

The CLI and VS Code have someone sitting there. The coding agent does not — it
runs to completion and the first human to read a word of it is reading the pull
request afterwards. That is not a detail about Copilot; it is the `unattended`
setting of `orchestrating-work-end-to-end`'s presence dial, and it changes what
a gate means across the whole library.

So on the coding agent, read `bounding-autonomous-work` before the first action,
not after. Every gate that was a question becomes a written self-answer carrying
a stop condition. And the one rule no envelope relaxes still stands: an
irreversible action gets a human. Reaching that point is the run ending
correctly, with one step left for someone who can own it.

## What is deliberately not here

There is no table of Copilot's tool names in this file, because writing one
would be guessing at names that the harness already puts in front of you
accurately, and a stale mapping is worse than none. Use the tools you actually
have, by the names you are actually given.
