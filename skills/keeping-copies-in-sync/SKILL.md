---
name: keeping-copies-in-sync
description: Use when the same fact — a version number, a constant, a policy document, a generated file, a lockfile entry, a manifest field — has to be recorded in more than one place; when a change updates one copy and a sibling copy is easy to forget; when two files that describe the same thing are found to disagree, weeks or releases apart, and someone catches it by hand; when a change introduces a second location for a value that used to live in exactly one; when the plan for keeping copies aligned is "remember to update both" rather than something that runs, or when a public statement of what the software does — a policy, a store listing — is updated before the build that does it has reached anyone. Covers why the number of places to update is the number of ways to disagree, replacing memory with a comparison that fails loud, preferring to generate a copy over hand-maintaining it, where the check belongs in the pipeline, and publishing a behavior-describing copy with the release.
---

# Keeping copies in sync

## Overview

Any fact that has to be true in more than one place starts drifting the day
it becomes two facts instead of one. Nobody decides to let a version number
fall out of step or a mirrored policy document go stale — every individual
edit that skips the sibling copy looks small and unrelated to the drift it
causes, and the drift is invisible until something reads the wrong copy or a
human happens to compare them. A version number pasted into eight manifest
files gets bumped in seven of them without anyone noticing the eighth, right
up until a build reads that one file and ships the old number — and the next
release repeats it, because the fix that time was to bump the eighth file by
hand, not to stop having eight places to bump. The fix is never "be more
careful next time"; it is a check that reads every copy and fails the moment
two disagree.

## When to use

- A fact — a version, a constant, a URL, a policy document, a schema, a
  generated file — is about to be recorded somewhere it is already recorded
  elsewhere.
- A change updated one copy of something and a reviewer, or a later incident,
  found a sibling copy that was not.
- Two files that describe the same thing are found to disagree, and nobody
  can say for how long.
- Introducing an integration, a second consumer, or a second target that
  needs its own copy of a fact your project already has one of.
- A public document that says what the software does — a policy, a store
  listing, a help page — is being updated alongside the change that alters
  the behavior.
- Not for: choosing whether a value should have more than one writer at all
  — `mapping-dependencies`' data-edge material is the diagnostic step that
  finds a fact already living in more than one place; this skill is what to
  do once it does, deliberately or not.
- Not for: a document specifically (a README, a guide, a reference page)
  duplicating what the code already states — `writing-durable-docs` owns
  writing prose so it cannot drift, including its own note on two mirrored
  documents. This skill covers every other kind of duplicated fact:
  manifests, lockfiles, generated artifacts, constants, machine-readable
  config — and applies to a duplicated *document* too, wherever the fix is a
  comparison rather than a rewrite.
- Not for: a dependency's own declared version against what actually got
  installed — `upgrading-dependencies` owns reading that diff on upgrade.
  This skill is about facts your own project chose to record more than once.

## The number of places is the number of ways to disagree

A fact recorded once cannot disagree with itself. A fact recorded in two
places can be wrong in either direction, and a fact recorded in N places has
N-choose-2 pairs that can silently diverge, each drifting on its own schedule
— one copy changes at a release, another at a deploy, a third only when
someone happens to open the file it lives in. Nothing about the mechanism
that lets a fact live in two places also keeps them equal; equality was never
enforced, it just happened to hold on the day someone wrote the second copy
by transcribing the first.

The failure is quiet by construction. Neither copy is malformed, neither
fails its own local validation, and each one, read in isolation, looks
correct. The only observation that catches it is a comparison — reading both
copies and asking whether they agree — and that observation has to be built,
because nothing produces it as a side effect of anything else running.

## Memory is not a plan

"We'll remember to update both" is not a mitigation; it is a description of
the exact mechanism that produces drift. It works until it doesn't, and it
fails silently precisely because everyone involved genuinely believed they
were being careful. It does not fail because someone was careless once — it
fails because "update the other copy too" is a step with nothing checking
whether it happened, in a change that is usually about something else
entirely, made by whoever happened to be editing that day and may not have
known the sibling copy existed at all.

Treat "there are now two of these" as the moment the real work starts, not
the moment it ends. The question that matters is not *did I remember this
time* but **what enforces that these stay equal, and on what schedule does
that check actually run** — every time either copy could change, not on a
best-effort audit sometime later.

## Prefer generating over copying, and checking over trusting

Two remedies, in order of strength:

1. **Eliminate the second copy.** Where one of the two locations can be
   produced *from* the other — a lockfile from a manifest, a generated
   client from a schema, a rendered page from its source — do that instead
   of hand-maintaining both. A generated copy cannot drift from its source
   because it has no independent existence; regenerating it is the only way
   it changes at all. This is the strongest fix because it removes the
   comparison rather than automating it.
2. **Where a second copy has to genuinely exist** — a value required by two
   unrelated integrations that cannot share a generation step, a fact that
   two independent teams or repos each need their own file for — add a
   check that reads every copy and fails loud the moment any two disagree.
   The check is small: it does not need to know which copy is right, only
   that they must match. Run it on the same cadence the fact can change: a
   pre-commit hook if any commit can change either copy, a release gate if
   only a release can, a scheduled job if the copies live in systems that
   change independently of each other's deploys.

Neither remedy is "write a comment reminding the next person." A comment is
read by whoever is already looking at that file, which is never the person
editing the sibling copy.

## Finding the copies before writing the check

Before building the comparison, enumerate every place the fact actually
lives — not just the two you already know disagree. A fact copied twice on
purpose is often copied a third time by accident: a value pasted into a
manifest and a lockfile is a plausible candidate for also being pasted into
a CI config, a Dockerfile, or a support document nobody thought to check.
Grep for the value itself across the repository and any sibling repository
it is known to be mirrored into, rather than trusting the two locations the
incident that prompted the search happened to surface.

## A copy that describes behavior is true of a build, not of a commit

A number is true the moment it is written. A copy that records behavior — what the software collects, where it sends it, what it stores, what it does for the person using it — is true of a *build*, and the build reaches the people the copy addresses well after the commit that describes it: cut, uploaded, reviewed, rolled out, installed. A public policy updated in the same hour as the commit tells every reader something false for as long as the shipped build still does the old thing, and the false statement is about them. The rest of this skill treats the source as the truth and the copy as the thing that lags; for a statement of behavior, the truth is what has actually been released, so the copy lags the release, not the branch.

So write the new text in the same change as the code, where the check can compare them, and publish it with the release: the same step, or the moment the build is confirmed to be in users' hands rather than merely submitted. While builds of both behaviors are in use — a store review, a staged rollout, clients that do not update — the copy has to be true of each, which usually means saying plainly which behavior applies to which version. Holding the text needs an owner and a trigger, a named release step that does the publishing, because a held statement nobody publishes is the stale mirror this skill exists to prevent, one release later. `writing-durable-docs` has the opposite direction, a document that lags a behavior change.

A push is not a deploy, either. A host that builds the public copy from the source has its own build, which can trail the push by minutes and can fail without the push noticing, so a green push, a clean hook and a failed host build produce the same silence. Verify after the host reports it finished: fetch what is actually served, from outside and with no cache, and compare it with the source, as a diff rather than a glance. `confirming-before-claiming-done`'s *The push is not the reach* has the general rule.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A release shipped two components at different versions | Each had its own file recording the version, and nothing compared them |
| A lockfile-recorded version sat several releases behind | The lockfile is hand-edited on the same cadence as the value it should track exactly, and nothing else touches it |
| A mirrored document was found stale more than once | Each staleness was fixed by hand-resyncing the copy, never by adding the check that would have caught the next drift too |
| The fix for a drifted copy was "update it and move on" | The actual defect — nothing enforces agreement — was never addressed, so the same drift recurs on the next change to either copy |
| A comment says "keep this in sync with X" and nothing does | A comment is read by whoever already has the file open, not by whoever edits the other one |
| The check exists but only runs on release day | The copies can change between releases, so the check's cadence is slower than the drift it exists to catch |
| Grepping for a stale value found a third copy nobody remembered | The search for copies stopped at the two the incident surfaced instead of enumerating every occurrence |
| A public policy described behavior that no user's build had yet | The copy was published with the commit, not with the release that delivered the behavior |
| The push succeeded and the live page still showed the old text, or never changed | A push is not a deploy: the host's own build lagged or failed silently, and nobody fetched what was served |

## Red flags

- "We'll remember to update both" said about a fact that is about to gain a
  second copy.
- A comment asking a future editor to keep two files aligned, with nothing
  that checks whether they did.
- "It's only out of sync by one release, it's not a big deal" — said about a
  gap that has already grown once and has nothing stopping it from growing
  again.
- A hand-written second copy of something a generator could produce instead.
- A sync check that only runs on a slower cadence than the thing it is
  checking can change on.
- A public statement of behavior merged, and published, the same day as the code that changes it, before any build with that code has reached anyone.
- "Pushed" reported as the last step for a copy that a separate host builds.
