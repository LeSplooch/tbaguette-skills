# watch

A Claude Code mod for waiting with no model running. It watches a usage ceiling, or runs a
check command on a timer, and stays silent. The session is told once, when the wait is over,
so there are no "nothing to report" messages and no agent polling on a schedule. It is the
tool half of TBaguette's `automating-repetition` skill.

## Install

At the prompt of a terminal session:

    /plugin install watch --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install watch --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /watch usage 42 finish the release, then pause    tell the session once at 42% of the weekly window
    /watch usage 80 5h                                the same for the 5-hour window
    /watch every 5m curl -sf https://example.com/up   run it now and every 5 minutes, wake once when it exits 0
    /watch every 1m --for 6h --note "install it" adb get-state
    /watch test                                       send one real wake
    /watch stop 2                                     stop one watch; `stop` alone stops them all
    /watch                                            list what is being watched

The model can start a usage ceiling itself, with the `mcp__watch__usage_ceiling` tool, rather
than checking usage or dispatching an agent to check it. It cannot start a command watch. A
check is a shell command run on a timer outside the permission system, so only a person typing
`/watch` can start one. To wait on a command, the model has its own permissioned tools.

Only you can change a watch: `/watch` starts, stops or tests one only when you type it (at the
terminal, in an app, or through Remote Control). A scheduled prompt, another plugin or another
session running `/watch` gets the list and nothing else.

## How it behaves

- **Usage.** The mod reads what the host pushes when a window moves a whole point, so nothing
  polls. During a running turn the news arrives with the next tool result, and in an idle session
  as a new turn. Readings come from this session's own responses, so an idle session may not see
  usage spent elsewhere until its next reply. Off a subscription there are no windows, and some
  plans have no 5-hour window. If there is no reading, the mod says so when you start the watch,
  and that watch may never fire. A ceiling already passed starts nothing and says so.
  A usage watch ends after two weeks, checked hourly, so it ends even when no reading ever comes.
- **Commands** run with `sh -c` in the session's folder, as you, once at the start and then on the
  timer. A check gets two minutes. A failing check is silent, and only an exit of 0 wakes the
  session. A command that cannot run (exit 126 or 127), or that times out three times running,
  stops the watch and says so. A command watch ends after 24 hours unless `--for` says otherwise,
  and says so when it gives up. Its output reaches the model labelled as output, not instructions.
- **`/watch test`** proves a wake can reach this session. It says nothing about whether a command
  can run or a window is reported.
- Watches last as long as the session, at most 8 at a time. Closing the session ends them. Other
  harnesses ignore this mod.

The mod API is early access, and nothing in the skills depends on this mod.
