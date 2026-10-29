# climb

A Claude Code mod that turns a goal into a climb: you name a check command, and
after every turn the mod runs that check itself, records the result, and queues
the next iteration until the check passes or a stop condition fires. The run's
own report of how it did never counts; the check's exit code does. It is the
tool half of TBaguette's `designing-verifiable-goals` skill: write the check
first, then let the climb run against it.

## Install

At the prompt of a terminal session:

    /plugin install climb --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install climb --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace
with `LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /climb 20 npm test          start: up to 20 iterations, the check is `npm test`;
                                the mod queues the first iteration itself
    /climb 20 --in ../app make test   run the check in ../app; the log and
                                protected paths live there too
    /climb protect tests fixtures   refuse edits to these paths while it runs
    /climb stall 5              stop after 5 iterations with no better score
    /climb stop                 end it
    /climb                      show the climb and its last steps

- **Bar met** when the check exits 0. Anything else is another iteration.
- **Score**, optional: the last `score: <number>` the check prints, higher is
  better. With scores, the climb also stops once it goes the stall limit
  without beating its best.
- **Stops** on the bar, the budget, the stall limit, `/climb stop`, the Stop
  button, a turn you interrupt, or a check that runs past ten minutes or cannot
  start. Starting again is a fresh climb: the earlier one's history is discarded.
  An iteration prompt already queued when the climb stops is dropped.
- **Only its own prompts advance it.** A prompt you send yourself mid-climb is
  answered normally and does not count as an iteration or run the check.
- **Log**: `.climb-log.tsv` in the climb's folder (the session folder, or `--in`) holds the current climb, one
  row per iteration: exit code, score, check time and turn time, both measured
  by the mod and the engine rather than reported by the model. A climb whose turns are short
  and whose checks are long is waiting on the check, not short of ideas.
- **Protected**: `Edit`, `Write`, `MultiEdit` and `NotebookEdit` on a protected
  path, and on the log, are refused, matched by spelling and, for a file that
  exists, by its real path. A shell command is not inspected, so this limits the
  easy route rather than sealing the check; keep what the check reads somewhere
  the run cannot write if that matters.
- A band above the prompt shows the trail (filled dot for a pass), the last
  exit code, score, check time and turn time.

The check runs with `sh -c` in the climb's folder, with a ten-minute timeout.

## Limits

The mod API is early access and moves between Claude Code releases. Other
harnesses ignore this plugin; the TBaguette skills never depend on it.
