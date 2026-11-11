# meter

A Claude Code mod that puts four numbers on one quiet status line: how full the context is, how
much of each rate-limit window is used, and what the session has cost. A band appears above the
prompt only when one of them crosses a line. There is nothing to configure, nothing polls, and the
mod sends no pings and makes no model calls. Every figure is marked as measured or as an estimate.
It is the tool half of TBaguette's `checkpointing-long-runs` skill.

## Install

At the prompt of a terminal session:

    /plugin install meter --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install meter --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /meter          every figure, where it came from, and the cache lifetime
    /meter quiet    keep the status line and silence the band for this session
    /meter loud     bring the band back

The status line reads like `ctx 41% · 5h 62% · wk 18% · $1.84`. Only units with a reading appear,
so a session with no cost figure shows no cost, and a figure is never shown as zero when it is
missing.

The mod registers no tool for the model and never writes to the conversation. It informs you, and
what to do about a number stays yours.

## How it behaves

- **Readings.** The host pushes a new reading when a figure moves, and the mod reads once at
  start. Nothing polls.
- **The band.** It shows a line when context reaches 80%, when a rate-limit window reaches 80% or
  95%, or when the cache is probably cold. A line is announced once per crossing, and again only
  after its figure falls 5 points below the line and crosses it again. A 95% crossing replaces the
  80% one for the same window. When several lines are showing, the newest is drawn and the rest
  are counted. `Hide` hides the current lines until the next crossing. Under 60 columns the band
  uses a short form.
- **Cold cache.** This is an estimate. The cache counts as cold when the session has been idle
  longer than the cache lifetime with at least 50,000 tokens of context. It is recomputed when you
  start a draft and when you send, and once by a timer when the lifetime runs out. It never delays
  a send.
- **Cache lifetime.** It starts at 5 minutes, labelled assumed. If the first request of a turn of
  at least 50,000 input tokens comes after more than 5 minutes idle and reads more than half of
  its input from the cache, the lifetime becomes 60 minutes, labelled observed. If a turn comes
  after less than the current lifetime and writes at least 90% of its input fresh, it goes back to
  5 minutes. This is an inference from two numbers and can be wrong.
- **Limits.** Rate-limit readings come from this session's own responses, so usage spent
  elsewhere shows after this session's next reply. Off a subscription there are no windows.
  Nothing here keeps the cache warm, because a ping spends tokens and how cached reads count
  against plan limits is not documented. The band draws only where the host draws a band above
  the prompt.

The mod API is early access, and nothing in the skills depends on this mod.
