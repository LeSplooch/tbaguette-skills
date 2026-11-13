# rest

A Claude Code mod that pauses every session on this machine with one command and resumes them
where they stopped. A resting session holds its next tool call with no model running, and nothing
is cancelled or lost: when the rest ends, the call runs as it would have, and the turn goes on. It
frees the machine and the usage window for a while (a game, a call, a battery) without closing a
session or interrupting a turn. It is the tool half of TBaguette's `bounding-autonomous-work` skill.

## Install

At the prompt of a terminal session:

    /plugin install rest --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install rest --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /rest                 rest every session with this mod for 8 hours
    /rest 45m             the same for 45 minutes; 2h, 90m work too (1 minute to 24 hours)
    /rest here 2h         rest this session alone
    /rest off             end the rest for every session, and this session's own
    /rest status          who rests, since when, until when, and what this session holds

`/rest` while a rest is in force says so and changes nothing; `/rest <length>` starts a new one.
A prompt you type in a resting session wakes that session alone, and the others keep resting. A slash
command you type there wakes it too, any command but `/rest` itself.

A length is a whole number and a lowercase unit: `90s`, `45m` or `2h`, from 1 minute to 24 hours. `1.5h`,
`2H`, `1h30m` and a bare `45` are refused with the usage text; write `90m` for an hour and a half.

The mod registers no tool for the model, and `/rest` changes nothing unless you typed it. A
session never pauses your other sessions, or wakes itself, on its own.

## How it behaves

- **Holding.** Each session checks the rest before every tool call, its subagents' included, and
  waits there while it rests, re-reading the rest every two seconds with the host's `sleep`. A
  prompt that is not yours (a plugin's, a schedule's, a task's notification, another session's, a
  channel's) waits the same way, so nothing starts a turn during a rest. Interrupting still works:
  the held call stops waiting and never runs.
- **Ending.** A rest ends by itself at its end time: 8 hours unless you gave a length, 24 hours at
  most, so a forgotten rest cannot freeze sessions you open tomorrow. `/rest off` ends it at once.
- **Showing.** A resting session's status line reads `resting until 18:30`, with `, 2 held` while
  calls wait. A session learns of a rest started elsewhere within five seconds, and says so once
  with a toast, and again when it wakes.
- **Not yet confirmed live.** Sessions learn of a rest by reading the mod's own store file. That
  every running session re-reads that file has not been checked with two live sessions. After
  starting a rest, `/rest status` in a second session shows whether it saw it.
- **What it cannot stop.** A model response that is already streaming finishes; the session stops
  at its next tool call. A turn that calls no tool simply ends. Shell commands already running in
  the background keep running. Sessions without the mod, and other machines, are not affected.
- **Waking costs.** After a long rest the prompt cache has usually expired, so the first request
  after waking reads the whole conversation again at full price.
- **Where it cannot hold.** Where `sleep` is not on the PATH (or not a CLI session), the mod cannot
  hold a call: `/rest` says so and starts nothing, and that session never waits. One failed round of
  `sleep` does not end a rest: the call stays held, and the session gives up holding only after three
  failed rounds in a row. `/rest status` and the next `/rest` try `sleep` again, so a session that gave
  up holds again once it works. Other harnesses ignore this mod.

The mod API is early access, and nothing in the skills depends on this mod.
