Content the site build reads that has no other home.

- `climb-live-run.tsv`: the `.climb-log.tsv` of the first live climb, run on
  2026-10-29 against a scratch project whose `to_roman` lacked the subtractive
  forms (check: 12 cases, score = cases passing). The climb was restarted once
  after a fix to the mod, so iteration 1 here is the second climb's first
  change (score 5 because the first climb had already added CM and CD); it met
  the bar at iteration 2. Shown verbatim on /mods/climb/.
- `climb-live-run.meta.tsv`: what that climb was started with, from the session
  that ran it: budget 6, the check `python3 check.py` (run in the project
  folder), and `check.py` protected. The replay on /mods/climb/ shows these
  instead of stand-ins. The same file holds what the log's caption says about
  the run and the log cannot: `date`, `project` (what it ran against),
  `checked_by` (what the check counted and how it scored), and the restart
  before it, `restarted` (how often, and why) with `carried` (what the earlier
  attempt had already added). The page reads every one of them; drop the
  restart pair together if a future run had none.
