# claim

A Claude Code mod for a working tree with more than one writer: another agent session, a person, a
script. A sweeping git command stages, stashes or destroys work that is not this session's. claim
notices which dirty files this session did not change, tells the model once, and refuses the sweeping
commands while any of them is still dirty. It is the tool half of TBaguette's `atomic-commits` skill.

## Install

At the prompt of a terminal session:

    /plugin install claim --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install claim --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /claim                what claim knows about this working tree
    /claim take <path>    accept one foreign file as this session's work
    /claim take all       accept every foreign file
    /claim off            switch detection and the guard off for this session
    /claim on             switch them back on

While a foreign file is dirty, these commands are refused: `git add -A` (and `--all`, `-u`, `.`),
`git commit -a`, a stash, `git reset --hard`, `git checkout .`, `git restore .` and `git clean -f`. The
refusal names the foreign files. A command that names explicit paths passes, and so does everything
else, and nothing is refused once no foreign file is dirty.

Only a person can take a file or switch the guard off or on, by typing `/claim`. The same command from
a plugin, a schedule or a peer prints the report and changes nothing, and the mod registers no tool for
the model.

## How it behaves

- **The rule.** This session changes the tree only while one of its own write-capable calls runs,
  subagents included. A dirty file whose size or modification time changed while none was running was
  changed by someone else, and is called foreign.
- **Telling the model.** Once for each new set of foreign files, attached to the next tool result and
  marked `[claim]`, with the paths given as data. You get a toast and a `claim: N foreign` status entry.
- **The guard.** It reads the Bash tool's command, including compound commands, `git -C` and options
  before the subcommand, and environment prefixes. It does not see a sweeping command inside a script,
  an alias or a Makefile target, and it does not see shells other than the Bash tool.
- **Background commands.** Detection pauses while a command this session moved to the background is
  running, because its writes cannot be told from another writer's. It resumes with a fresh baseline when
  a turn ends with none left, or at `/claim on`.
- **Taking a file.** `/claim take` and a commit both take a file off the list. A taken file returns only
  if it changes again while the session is idle.
- **What it can get wrong.** It can call a file foreign when a tool it does not know as a writer changed
  it between calls. It misses a foreign change that lands while one of this session's own write-capable
  calls is running. Above 300 dirty entries it compares status codes only, so a second change to an
  already-changed file is not seen.
- **Cost.** Two read-only `git status` runs around each write-capable call, each with a 5 second limit.
- **Outside git.** It does nothing, and `/claim` says so.

The mod API is early access, and nothing in the skills depends on this mod.
