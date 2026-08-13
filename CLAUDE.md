# Working conventions for this repo

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
