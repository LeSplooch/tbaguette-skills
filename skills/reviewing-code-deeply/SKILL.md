---
name: reviewing-code-deeply
description: Use when reviewing someone else's change — a pull request, patch, diff, or commit series — deciding what deserves a comment, whether to block or approve, or how to phrase a concern. Also when a review is generating dozens of comments, when a diff is too large to review carefully, or when reviews keep passing code that later breaks. Covers review priority order, reviewing tests, finding absent cases, and blocking versus non-blocking.
---

# Reviewing Code Deeply

## Overview

Review in strict priority order and stop when your attention is spent. Comments about naming, spent on a diff whose failure handling is wrong, are worse than no review at all — they signal the change was examined when it was not.

## When to use

- Reviewing a pull request, patch series, or diff written by someone else
- Deciding whether a concern blocks a merge
- A review has produced more than a dozen comments and none of them are about behavior
- Reviewed code shipped a defect and the review process needs a look
- **Not for:** asking someone else to review your work → `handing-off-for-review`. Responding to feedback on your own change → `verifying-review-feedback`.

## Priority order — descend only when the level above is clear

| # | Level | Question | Cost of never reaching it |
|---|---|---|---|
| 1 | Correctness | Does it do what the change claims, at the boundaries and on the failure paths? | The review had no value |
| 2 | Security & data safety | Untrusted input, authorization at every entry, secrets, destructive or irreversible operations, migrations that cannot roll back, personal data in logs | The review had negative value |
| 3 | Design & boundaries | Is this the right seam? Does it leak state, create a cycle, or make the next change harder? | The cost arrives in six months |
| 4 | Readability & naming | Will the next reader understand this without the author present? | Mild, permanent |
| 5 | Style & formatting | — | None; this is a tool's job |

Level 5 is never a human's work. A comment about formatting is a bug report against the CI configuration, and should be filed as one.

Most reviewers work bottom-up without noticing, because levels 4–5 are the only ones visible without thinking. Read the diff once for level 1 alone, before you permit yourself to notice a name. A defect found at level 1 often deletes the code you were about to rename.

## Read the intent before the diff

- Read the stated intent — description, ticket, decision record — first, and review the diff *against* it. Without it you can only check internal consistency, which passes cleanly on code that solves the wrong problem.
- If the intent is not stated well enough to review against, that is comment one, and the rest of the review waits for the answer.
- Check the reverse direction: what is in the diff that the intent does not explain? Unrelated changes bundled into a diff are the most common way a real defect ships unexamined, because attention is spent on the part everyone came to look at.
- Past ~400 changed lines, defect detection drops sharply. Say so and ask for a split. A skim-and-approve on a 2,000-line diff is worse than declining, because it records that the change was checked.

## Review the tests as carefully as the code

- For behavioral changes, read the tests first. They state what the author believes the contract is, and the gap between that belief and the description is where defects live.
- For each test: **would it fail if the implementation were wrong?** Tests that assert against mocks they configured themselves, or that pass with the function body removed, are decoration and should be called out as such.
- Check the boundaries the tests skipped: empty, one, many, maximum; zero-length and maximum-length input; the error path of every fallible call; concurrent or reentrant entry; the second invocation.
- A new branch in the code with no new test is a specific comment naming that branch — not a general request for more tests, which the author cannot act on precisely.

## Look for what is absent

Diffs show what is present; most serious defects are a missing case.

| Category | What to check |
|---|---|
| Error paths | Every fallible call: handled, propagated, or knowingly swallowed — and is the swallow explained? |
| Partial failure | If step 3 of 5 fails, what state remains? Is it recoverable, and does a retry make it worse? |
| Concurrency | Two callers at once; the same caller twice; the operation interleaved with its own retry; check-then-act on shared state |
| Resources | Anything acquired must be released on the error path too — handles, locks, connections, transactions |
| Boundaries | Empty, single, negative, zero, overflow, unicode, timezone, clock skew |
| Compatibility | Old clients, old rows, old messages during the deploy window — is it safe while both versions run? |
| Reversibility | Can this be rolled back after it has written data? |

At every network or IPC boundary in the diff, ask what happens if the call never returns. Absence of a timeout is invisible in a diff and is a level-1 finding.

The table above is an enumeration from the domain rather than from the diff, and that is the only method that finds absence — you cannot review your way to a case nobody wrote, because review examines what is in front of you. When the change is large enough that the categories here do not cover its subject matter, `clairvoyance` generalizes the move: enumerate the states, actors, and lifecycle stages the *thing itself* has, then find each one in the diff. The ones with no counterpart are the finding.

## Mark every comment blocking or not

Unmarked comments make the author guess, and they guess "blocking" — which is why reviews with thirty unmarked nits take a week.

| Marker | Meaning | Use for |
|---|---|---|
| **blocking** | I will not approve until this changes | Levels 1–2, and genuine design damage |
| **non-blocking** | Take it or leave it; approving either way | Levels 3–4 |
| **nit** | Cosmetic, strictly optional | More than ~3 means fix the tooling instead |
| **question** | I genuinely do not know | Never as a rhetorical stand-in for "this is wrong" |

Ask rather than assert when you do not understand. "What happens if this runs during a config reload?" is answerable; "this breaks during a config reload" — when it does not — costs the author a defense and costs you the next reviewee's attention.

Give the *why* on every blocking comment. "Use a set" is a preference. "This is quadratic on a collection that reaches ~50k rows in production" is a defect report, and the author can evaluate it.

## Review your own review before sending

- Count the comments. Past ~15 on one change, or at any comment requesting a different architecture, this is a design conversation happening after the code was written. Say that in one summary comment, stop line-commenting, and move it to a conversation.
- Collapse repeats: one comment naming the pattern and "same in 6 other places", not seven comments.
- Delete every comment you could not defend as changing behavior, risk, or the next reader's time.
- Open with a summary of what you checked and what you did not. "I read the state machine closely and skimmed the generated client" is honest and calibrates the author's trust; silence implies you read everything.

## Approve on trust, and know the limits

- Approve when the remaining risk is acceptable to you — not when you have found nothing. Thoroughness measured in comment count is performance. A clean approval on a small change is a valid output.
- Review reliably catches: intent mismatch, missing cases, design damage, dangerous operations, unclear naming.
- Review reliably misses: races and interleavings, behavior under real load, resource growth over time, integration behavior, and most type-level mistakes. When those are the risk, the useful comment names the test, type, assertion, or canary that would catch it — because your reading will not.
- "Approve with comments" only when none of them block. Approving while leaving a blocking comment teaches the author that the markers mean nothing.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Thirty comments, all naming and formatting | Reviewed bottom-up; levels 1–2 never received attention |
| The review took four days | Comments unmarked; the author waited for a signal that never came |
| A defect shipped in reviewed code | Reviewed what was present; the defect was an absent case |
| Large diffs get fast approvals, small ones get scrutiny | Attention scaled with readability instead of risk |
| "This is wrong" and the reviewer was mistaken | Asserted where a question was warranted |
| Every review becomes an architecture argument | The design was never reviewed before implementation |
| The reviewer rewrites the change in comments | Reviewing against their own solution rather than the stated intent |
| The same style debate recurs on every change | No formatter in CI; a human is doing a tool's job |

## Red flags

- Reading the diff before reading what it is meant to do
- Commenting on a name before you have traced one error path
- "LGTM" on a diff you scrolled rather than read
- A comment you would withdraw the moment the author pushed back once
- Approving because the author is senior, or blocking because they are not
- More than three nits
- Requesting changes without saying which comment is the blocker
- Reviewing only the lines the diff shows, never opening the file around them
