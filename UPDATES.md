# Update notes

What changed, newest first, written for someone who has TBaguette installed
rather than for someone reading the diff. The landing page renders the most
recent entries below the “Fresh from the oven” rail; this file is the whole
record.

Writing an entry is part of shipping a change here — see CLAUDE.md. The shape
is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

## 2026-08-24 — Skills link to each other, and the site says what changed

- Every mention one skill makes of another is now a link to that skill’s page
  — 362 of them across the site. The library is densely cross-referenced and
  none of it was navigable before; `diagnosing-before-fixing` naming
  `regression-test-from-bug` was a dead string.
- Both forms link: the `code`-span mentions that run through the bodies and
  tables, and the bare ones in the “Not for: … (other-skill)” line at the top
  of most skills. A page never links to itself, and a code span that is not a
  skill stays plain.
- The landing page carries an **Update notes** section under the “Fresh from
  the oven” rail: the newest entry’s bullets in full, everything earlier
  folded behind one disclosure that needs no JavaScript to open.
- Notes come from `UPDATES.md` at the repo root, not from the commit log, so
  an entry can say what is different for you rather than which files moved.
- The rail and the notes answer different questions side by side. The rail
  names *which* skills changed in the last 48 hours; the notes say *what*
  changed, including work that touches no skill at all.

## 2026-08-23 — Three skills sharpened, and English-only again

- `least-privilege-design` now covers being asked to justify a permission
  something already holds: check what actually calls it before defending it
  in the abstract.
- `confirming-before-claiming-done` closes the gap between “pushed” and
  “reached the user” — a deploy nothing has fetched yet is not evidence the
  change landed.
- `feature-flagging` says to pin the value of a flag that suspends a rule, so
  a test proves the suspension instead of inheriting whatever the environment
  happened to have set.
- The site dropped its eleven translated locales and serves English only.
  Translated URLs (`/fr/`, `/de/`, and the rest) no longer exist. Nothing
  about installing TBaguette or about skill content changed.

## 2026-08-22 — The spine the rest of the library hangs off

- New skill: `orchestrating-work-end-to-end`. Invoke it when a request will
  take more than one edit — it routes the work to a track, names the phase
  you are in, and says what evidence opens the next one. It is the skill that
  sequences the other ninety-one.
- The install prompt on the site works out which harness it was pasted into
  and takes that route, instead of assuming Claude Code and falling through
  to a filesystem clone.
- `rate-limiting-and-backpressure`, `instrumenting-for-observability`,
  `deleting-code`, and `authoring-a-new-skill` each gained a case they
  previously got wrong.

## 2026-08-21 — When the model itself is the blocker

- New skill: `routing-around-capability-gaps` — what to do when the thing
  being asked for is outside what this model or harness can actually do,
  rather than quietly delivering something adjacent and calling it done.
- The install instructions warn that a conversation already open when you
  install will not see the new skills. Start a fresh one.
- The “Fresh from the oven” carousel works again: a click on a tile reaches
  its skill, dragging no longer swallows that click, the fan sits level in
  its rail, and it stopped pushing the whole page sideways at narrow widths.

## 2026-08-20 — Finishing, and a version you can read

- New skill: `finishing-what-you-started`, for long runs and multi-part
  requests where stopping short would go unnoticed. Write the acceptance
  ledger to a file before starting, and re-measure every number at report
  time rather than quoting it from memory.
- The installed plugin’s version shows next to the wordmark in the site
  header, so you can tell at a glance whether what you have matches what is
  published.
- The skill check is re-asserted every turn rather than only at the start of
  a session, which is where it used to quietly lapse in long conversations.
