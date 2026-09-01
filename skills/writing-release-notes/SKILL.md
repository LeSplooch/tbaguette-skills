---
name: writing-release-notes
description: Use when preparing release notes, a changelog, an upgrade guide, or a version announcement; when a release contains breaking changes, deprecations, removals, or anything a consumer must act on; when turning commit subjects into user-facing text; or when consumers keep asking what they must change to upgrade. Covers ordering by reader impact, per-audience tagging, migration steps, deprecation timelines, and what to leave out.
---

# Writing Release Notes

## Overview

Release notes are read by one person with one question: what must I do, and what do I get? They are a decision aid for an upgrade, not a record of the team's work. Anything that does not move that reader toward a yes or no is a tax they pay in scanning time.

## When to use

- Cutting a release of anything another team, customer, or process consumes
- The release contains a breaking change, a removal, a deprecation, or a required migration
- Turning a generated commit log into something a human can act on
- Consumers are asking in support channels what an upgrade requires
- **Not for:** why the change was made → `writing-adrs`. How to use the new feature → `writing-durable-docs`. Notes link to both; they do not contain them.

## Two artifacts, not one

| | Changelog | Release notes |
|---|---|---|
| Produced by | Generation from commits or merged changes | A human, deliberately |
| Coverage | Complete, every change | Selective, only what affects a reader |
| Order | Chronological or by type | By impact on the reader |
| Purpose | Archive and attribution | Upgrade decision and migration |

Merging them yields either a changelog nobody can act on or notes that are incomplete and therefore untrustworthy. Ship both; the notes link to the changelog for completeness so the notes are free to omit.

## Order by what it costs the reader

Strict. A reader who stops after any section has still received the most expensive thing they would have gotten.

1. **Breaking changes**, each with its migration step. When there are none, say "No breaking changes" explicitly — its presence is the single most valuable line in the document for an upgrade decision, and its absence forces a full read.
2. **Required actions that are not breaking** — data migration, new mandatory config, raised minimum runtime or dependency versions, a manual step during deploy.
3. **New capability** — what the reader can now do that they could not.
4. **Fixes** a user could have noticed. A fix to code shipped in this same release is not a fix; it is the absence of a bug and belongs nowhere.
5. **Deprecations**, each with a removal version and date.
6. **Internal, performance, and dependency changes** — one condensed block or a link, never interleaved with the above.

## Every entry names its audience

This is where most notes fail: every reader reads every line to discover that 90% of it does not apply to them.

- Tag each entry with who it affects — `[API consumers]`, `[self-hosted operators]`, `[plugin authors]`, `[CLI users]` — or split the document by audience with a nav line at the top.
- Target: a reader in one audience can skip to their part and read under ~20% of the document.
- An entry you cannot attribute to an audience is usually an internal change that should be omitted.

## A fix line claims the reader had the bug

"Fixed: X" carries a premise nobody states: that this reader could have run into X. The commit log cannot settle it — the log records when a bug was fixed, never whether anyone outside the team was exposed to it. Only one span can, and it is the span the notes are actually for: from the build this audience is on to the build they are getting.

Two kinds of entry fail that test while looking like real fixes in the diff.

- **Introduced and fixed inside the same window.** The ordering list above states the easy version of this — a fix to code shipped in this same release is not a fix. The window is the reader's, though, not the release's: an audience three versions behind never met a bug that appeared and died in the two versions between. Both commits are legitimately in range, so nothing in the log marks it.
- **The fix had no symptom.** The platform silently ignored the malformed attribute, the wrong value was never read, the broken branch was unreachable. The repair is genuine and there was nothing for the reader to have suffered.

The cheap tell for the first: **does the entry reference a feature this audience has never had?** If the fix only makes sense to someone who ran a build between two of yours, it is not news to them.

Keeping such a line anyway is a legitimate call — some projects want the record complete, and a fix with no symptom can still be worth stating where a reader is auditing rather than upgrading. Running the check is what makes that a decision instead of an accident.

## Breaking changes carry the step, not the difference

The entry contains the literal action, not a description of what is now different.

- Weak: "`parse` now returns a result type instead of throwing."
- Strong: "`parse` returns a result instead of throwing. Update call sites: `x = parse(s)` → `x = parse(s).or_default()`. Find them by searching `parse(`. A codemod is available at <link>."

Include, in order: the old form, the new form, how to find every affected site, and whether a mechanical fix exists. If the migration needs more than ~5 lines, the note carries a one-line statement of *what breaks* plus a link to a migration guide — never a bare "see the migration guide", which forces every reader to open it to find out whether they are affected.

## Deprecations need a version and a date

"Deprecated" with no removal target is ignored, because it is the ambient state of everything.

- State: deprecated in X, removed no earlier than Y (version and date), and the replacement.
- If you cannot name Y, you are not committed and this is not yet a deprecation — say "we expect to replace this" instead, and do not start a clock nobody will honor.
- Repeat every live deprecation in **every** release's notes between deprecation and removal. A reader upgrading across five versions reads one document, and a deprecation announced three releases ago is invisible to them.
- Removal itself is a breaking change and goes in section 1, even though it was announced.

## Translate commit language into reader language

| Commit says | Note says |
|---|---|
| "Refactor auth middleware" | Omit — or, if measurable, "sign-in latency down ~40ms" |
| "Fix null check in exporter" | "Exports no longer fail when a record has no owner" |
| "Bump lib to 3.2" | Omit, unless it raises a minimum version, fixes a published vulnerability, or changes behavior |
| "Add feature flag for X" | Omit until X is on for the reader |
| "Improve performance" | Quantify or omit: "cold start 2.1s → 0.9s" |

Each entry names the observable difference from outside the system, and where the effect is a number, gives the number with a baseline.

## What to omit

Internal refactors, test and CI changes, formatting, dependency bumps with no observable effect, changes behind an off flag, and fixes to unreleased code. Any entry you cannot finish without naming an internal module the reader has never heard of.

Omission is not concealment — the changelog has all of it. Padding the notes with internals trains readers to skim, and the first thing a skimming reader skips is section 1.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Users hit a break that "was in the notes" | It was, below 40 lines of internal changes |
| "What do I actually have to change?" in support | Differences described, migration steps never given |
| Notes are a bulleted list of commit subjects | Generated changelog shipped as notes with no human pass |
| Every release says "various fixes and improvements" | Written from the version number, not from the diff |
| Deprecation warnings ignored for a year, then removal breaks everyone | No removal version or date, and no repetition in intervening releases |
| Long notes for a patch, thin notes for a major | Length tracked commit count instead of reader impact |
| Notes assembled at tag time from memory | The user-facing line was not captured when the change was made |
| Readers on one platform read four screens that do not apply | No audience tagging |
| A fix entry nobody recognises, for a bug nobody reported | The bug was introduced and fixed between two of this audience's builds, so the log had it in range and the reader never met it |

## Red flags

- "It's a small change, they'll figure it out"
- Writing the notes from the commit log alone, without reading the public surface
- A breaking-change entry with no code in it
- Publishing "no breaking changes" without having checked the exported interface
- An entry only comprehensible to someone who read the pull request
- Announcing a deprecation with no removal date because the team has not agreed on one
- Writing the notes after the tag is already published
- A "Fixed:" line for a bug that never had a symptom, kept because the commit was real
