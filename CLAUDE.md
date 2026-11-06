# Working conventions for this repo

## Write the update note first

Every change that ships from this repo adds an entry to `UPDATES.md` at the
root, before the site build runs. The landing page renders the newest
few below the "Fresh from the oven" rail. The file keeps the newest thirty and
`updates-archive/` holds the rest, one file per month; when the build says
UPDATES.md is too big, run `site/atelier archive-updates` and build again.

Write it for someone who has TBaguette installed, not for someone reading the
diff — the observable difference, not which files moved. `writing-release-notes`
is the register: order by what it costs the reader, translate commit language
into reader language, and omit anything you cannot finish without naming an
internal that reader has never heard of.

**Scope is the plugin, and only what a user of it would notice.** A new skill, a
skill that now says something different, a change to how skills are named,
grouped, or installed. Not this repo's tooling — test suites, build gates,
registries, version bumps — and not the showcase site's furniture: its layout,
search, animations, or chrome. Those are real work and they are not news about
the plugin; they go in the commit log. The test is the reader rather than the
effort, so the hardest change of the week can be out of scope while a one-line
wording fix to a skill is in it. Shipping something that is entirely site or
tooling work means the correct number of update-note entries is zero.

The shape is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date at
the top of the file. A bullet may wrap across lines. The build refuses to run
if the shape or the ordering breaks, so a mistyped heading or an entry appended
to the bottom fails loudly instead of quietly costing the entry.

An existing entry for today gets extended rather than duplicated — same-day
entries are legal, but two of them in one session usually means one change.

## Adding a skill

`skills/tending-tbaguette/reference/contribution-procedure.md` has the filing table.
A new skill reddens only what is meant to be touched by hand, and each says so by name:

- `EXPECTED_SKILL_COUNT` in `site/crates/atelier-build/src/write.rs`, which stops the build
  when skills appear or vanish by accident.
- The count written out in every manifest description, `README.md` and `CATALOG.md`
  (`site/atelier check`: `catalog-prose-counts`, `catalog-stated-count`).
- `hooks/skill-index.tsv`, which `site/atelier router` rewrites (`hooks-skill-index`).

Nothing else in `cargo test` holds a count. Tests derive it from the skills on disk, so a new
literal there is a bug.

## Commit and push automatically

Once a change here is complete and verified — `cargo test --manifest-path
site/Cargo.toml --workspace` and `site/atelier check` pass, and the site was
regenerated (`site/build.sh`, which also rebuilds the WASM front end; `site/atelier
build --base-path /tbaguette-skills` alone when only `skills/` or the pages'
content changed) if `docs/`, `skills/`, or `site/` changed — commit and push it to `origin/master` without asking for confirmation first.
`site/atelier check` and both hooks refuse WASM modules older than the crates behind them;
`site/build.sh` is the answer.

This repo's GitHub Pages site is served directly from `docs/` on `master`,
with no separate build or deploy step. Pushing *is* publishing here, and
that's intentional, not something to gate behind an extra confirmation.

A push to `master` runs `.githooks/pre-push`, which refuses to publish a Crumb
(the skill graph at `/graph/`) that no longer matches the skills in the same
commit; `site/atelier crumb-check` names what is stale. That refusal is not a
permissions problem: rebuild the site in a tree that matches the commit, commit
`docs/`, and push again. Never `--no-verify` past it.

This does not extend to destructive or hard-to-reverse git operations
(force-push, `reset --hard`, rewriting published history, deleting branches).
Those still need explicit confirmation, same as anywhere else.
