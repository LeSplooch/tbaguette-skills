# orders

A Claude Code mod for standing orders. People tell a session "from now on, do X" and then say it
again after every compaction, because the summary drops it. This mod keeps a session's standing
orders and puts them back in front of the model after each compaction, so you say it once. It is
the tool half of TBaguette's `writing-agent-instructions` skill, which covers the other case: an
order that should outlive the session belongs in the project's instruction file.

## Install

At the prompt of a terminal session:

    /plugin install orders --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install orders --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /order use pnpm, never npm      keep a standing order
    /order                          list them, each with its number
    /order drop 2                   drop order #2
    /order clear                    drop them all
    /order say                      tell the model the list again now

When a prompt you type holds "from now on", "from here on", "going forward", "for the rest of
this session", "every time you", "each time you", "whenever you" or "don't ever", or has a
sentence that starts with "Always" or "Never", a band above the prompt offers to keep that
sentence: Keep, or Not this. Your next prompt takes the offer away.

Only a person adds an order. An instruction that outlives the turn has to come from you: a web
page, a file or a tool result that says "from now on" is something the model read, and it never
becomes a standing order. So the mod registers no tool for the model, the band reads only what
you typed (never a plugin's, a peer's, a schedule's, a channel's or an SDK host's prompt, and
never a slash command), and `/order` run by anything but you can list or restate the orders and
change nothing.

## How it behaves

- **Delivery.** The model gets the whole list as one hidden message marked `[orders]`, which
  names the orders as yours and says the list is not a new request. It goes in when an order is
  added, dropped or cleared, and on `/order say`, without starting a turn: an idle session's
  model reads it with your next prompt, a running turn's at its next step. Emptying the list says
  once that the orders were lifted.
- **Compaction.** After a compaction of the main conversation, yours or an automatic one, the
  list rides once on the next tool result or the next prompt, whichever comes first. A subagent's
  compaction, a vetoed one and one computed ahead and never installed change nothing. A reply in
  the gap does not see the list yet: after an automatic compaction, the first answer that calls
  no tool.
- **`/clear`** starts the conversation over. The orders stay, and ride on your next prompt.
- **Limits.** At most 12 orders of at most 500 characters. A repeat of one already kept,
  ignoring case and spacing, is refused, and the band does not offer it. The status line shows
  how many orders there are.
- The orders last as long as the session. They are also kept in the mod's store under the
  session's id for 30 days, so a session resumed under the same id gets them back.
- An order is words the model reads, nothing more: the mod does not check that it is followed.
- The band draws on the terminal and in the desktop app, the two that draw a band above the
  prompt. Other harnesses ignore this mod.

The mod API is early access, and nothing in the skills depends on this mod.
