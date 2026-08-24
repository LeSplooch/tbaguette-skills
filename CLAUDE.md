# Working conventions for this repo

## Write the update note first

Every change that ships from this repo adds an entry to `UPDATES.md` at the
root, before `scripts/generate.py` runs. The landing page renders the newest
few below the "Fresh from the oven" rail; that file is the whole record.

Write it for someone who has TBaguette installed, not for someone reading the
diff — the observable difference, not which files moved. `writing-release-notes`
is the register: order by what it costs the reader, translate commit language
into reader language, and omit anything you cannot finish without naming an
internal that reader has never heard of. Internal refactors, test changes and
site plumbing with no visible effect belong in the commit log, not here.

The shape is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date at
the top of the file. A bullet may wrap across lines. The build refuses to run
if the shape or the ordering breaks, so a mistyped heading or an entry appended
to the bottom fails loudly instead of quietly costing the entry.

An existing entry for today gets extended rather than duplicated — same-day
entries are legal, but two of them in one session usually means one change.

## Commit and push automatically

Once a change here is complete and verified — `python3 scripts/run_tests.py`
passes, and the site was regenerated (`python3 scripts/generate.py --base-path
/tbaguette-skills`) if `docs/`, `skills/`, or `scripts/templates.py` changed —
commit and push it to `origin/master` without asking for confirmation first.

This repo's GitHub Pages site is served directly from `docs/` on `master`,
with no separate build or deploy step. Pushing *is* publishing here, and
that's intentional, not something to gate behind an extra confirmation.

This does not extend to destructive or hard-to-reverse git operations
(force-push, `reset --hard`, rewriting published history, deleting branches).
Those still need explicit confirmation, same as anywhere else.
