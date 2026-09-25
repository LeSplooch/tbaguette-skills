# GitHub Copilot Tool Mapping

Covers the surfaces TBaguette installs into: Copilot CLI, the GitHub Copilot
app, Copilot in VS Code, and the Copilot coding agent. Where they differ, it
says so; where nothing below distinguishes them, they behave the same. The app
runs the same agent as the CLI, so everything said about the CLI holds there
too, and the section on the app adds what it has on top.

Almost nothing in this library needs mapping. TBaguette's skills describe
*actions* — read a file, run a command, dispatch a subagent — and Copilot CLI
has all of those under its own names, which it already tells you about. The one
place the abstraction leaks is the sentence that matters most: how to reach
another skill. Claude Code has a `Skill` tool. Copilot CLI does not. That
difference, and the few things downstream of it, is what this file is for.

## Invoking a skill

Two routes, and the first one is doing most of the work already. Both work the
same on every surface:

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
the same everywhere: AGENTS.md, or `.github/copilot-instructions.md`, or
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

Copilot has custom agents — `*.agent.md` files — and dispatches subagents from
them. On Copilot CLI and in the GitHub Copilot app the dispatching tool is
`task`; another surface may name its own differently. Where a skill asks for a
subagent (`fanning-out-independent-work`, `delegating-tasks-with-review-gates`,
the fanned crew of `orchestrating-work-end-to-end`), use whatever your surface
offers. Five things about it change how those skills read there; the first four
were observed on Copilot CLI 1.0.87 in September 2026.

**Parallel means one response.** Several `task` calls in the same response run
concurrently; the same calls spread over consecutive responses run one after
another, which is a queue, not a fan-out. A call with `mode: "background"`
returns at once and the harness reports when the agent finishes, so the
controller can work its own lane meanwhile instead of polling. An idle agent
takes a follow-up message with its context intact — that is the resume the fix
loop in `delegating-tasks-with-review-gates` asks for.

**An agent file is a role.** An `*.agent.md` under `~/.copilot/agents/`, under a
repository's `.github/agents/`, or shipped by a plugin, becomes an `agent_type`
the `task` tool accepts, with its own standing instructions, tool list, and
model. One with no `model:` line inherits the session's model — the expensive
default `delegating-tasks-with-review-gates` warns about — so either pin one in
the file or name the model in every dispatch. This plugin ships the three roles
the delegation skills dispatch, and Copilot CLI offers them as
`TBaguette:implementer`, `TBaguette:reviewer` and `TBaguette:investigator`. The
reviewer and investigator are given no edit tool — they keep a shell, so that
limits their tools rather than guaranteeing they cannot write — and none pins a
model, so name one per dispatch. A listed agent that nothing names tends to go
unused: name the role in the dispatch.

**A subagent starts without this plugin's context.** It gets no session-start
injection and no per-prompt nudge; it does get the skill tool and the file
tools. So the prompt has to stand alone, as `fanning-out-independent-work`
already demands, and a skill the subagent needs is named in the prompt rather
than assumed.

**Every subagent shares the session's checkout.** None gets a worktree of its
own. Disjoint write sets are the only isolation between parallel lanes unless
the controller creates worktrees itself (`isolating-work-with-worktrees`).

**Decide the fan-out before reading everything.** Copilot's own instructions
tell the model to keep small work inline, and a model that has already read
every unit into its context will nearly always find inline cheaper: the reading
is paid for by then, and only the dispatch overhead is still visible. So decide
the split from the orientation pass — the list of units and the files each one
writes — before opening each unit, and decide it on
`fanning-out-independent-work`'s own grounds: isolation and wall-clock time,
never the length of the list. One contributed measurement, on Copilot CLI with
four independent packages to fix: a soft "fan out if the work splits" rule,
delivered with every prompt, dispatched in none of three runs, while a firmer
rule — three or more units means one `task` each, decided before reading —
dispatched all four packages in three runs of three, at roughly twice the
credits and with no wall-clock gain on work that small. That measurement cannot
say whether the count or the timing did it; the count is the half this file
does not adopt.

`/fleet` — or `copilot --fleet`, or plan mode's option to build on autopilot
with fleet — hands the partitioning to the harness itself. What it reads to
partition is not documented, so give it a plan that states the split outright:
one that marks each task's dependencies and which tasks may run together
(`structuring-an-implementation-plan`) is legible to any partitioner, a model's
or a person's.

Where your surface offers no way to dispatch a subagent at all, every one of
those skills already carries its own fallback: do the work inline, in sequence,
rather than inventing a dispatch that will not run. Same rule for todo tracking
and web fetch — degrade, don't improvise.

## The GitHub Copilot app

The GitHub Copilot app hosts the same agent runtime as the CLI, and it reads
the same `~/.copilot/` home for global instructions, agents, skills and
extensions. Everything in the sections above therefore applies there
unchanged. Three things are new — observed on the app's agent runtime 1.0.87 in
September 2026 and not documented, so trust what the app actually offers you.

**A project session is a CLI session; the general chat is not.** Work in a
repository happens in a project session, bound to a checkout. There the
session-start context, the per-prompt reminder, the global instructions and
the custom agents all arrive, as they do on the CLI, provided this plugin's
hooks run. They are `bash` scripts, so on Windows the app needs a `bash` it can
find; without one the hooks exit silently and nothing this plugin injects
arrives. The session-start context arrives as a block prepended to the first
message rather than as a separate one. The app's general chat is a lighter
surface with no repository behind it. It was observed not to offer custom
agents as `task` types and not to carry the global instructions. There, a
fan-out goes to the built-in `general-purpose` and `explore` agents, and
repository changes are handed to a project session instead of being made
from the chat.

**A worker can be a whole session.** Besides `task`, the app gives the agent
`create_session`, which starts another project session. That session has its
own agent, its own context and, with `workspace_type: "worktree"`, its own git
worktree and branch. With `notify_on_idle` set, the creator is told when it
finishes, so it does not poll. It can read the worker's state with
`get_session`, message it with `send_session_message`, and archive it with
`archive_session`, which removes the worktree — so archive a worker only once
nothing on it is left to keep. This is the isolation `task` lacks. Use it for
the lanes that need it: lanes that would collide on a file, on the index lock
or on a build tree, lanes that run long, and lanes the user should get back as
their own branch. It costs a full session each, so `task` stays the default
for small disjoint lanes. The gate does not move: a worker's summary is a
claim, and its diff is reviewed before anything is integrated
(`delegating-tasks-with-review-gates`).

**Plan approval can fan out too.** Approving a plan in the app offers the same
build-on-autopilot-with-fleet choice as the CLI, so the same markup from
`structuring-an-implementation-plan` is what to hand it.

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

The names that do appear above — `task` and `create_session`, the session
tools beside it, and the built-in agents — are there because their shape
changes how a skill should behave:
one response means parallel, and a worker session has a worktree of its own.
Each is dated to the version it was observed on. Where your harness offers
something different, the harness is right.
