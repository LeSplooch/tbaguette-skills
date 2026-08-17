---
name: verifying-review-feedback
description: Use when code review feedback lands on your own change — from a person, a bot, or an automated reviewer — before any of it gets implemented, especially when a suggestion is unclear, stated with more confidence than evidence, or hard to reconcile with what the code actually does. Covers verifying a claim against the codebase before acting on it, choosing a fix versus a pushback versus a clarifying question, and responding to correct feedback without performative agreement.
---

# Receiving code review

## Overview

A review comment is a claim about the code, not an instruction. The right first move is the one you'd use for any other claim: understand what it asserts, check it against what the code and tests actually do, then respond — with a fix, a question, or a reasoned no. Implementing on request without checking isn't deference; it's the same shortcut as dismissing the comment on principle. Both skip the evaluation and let something other than the code decide the outcome.

Agreement that arrives before verification costs you nothing to produce and is worth nothing to the reviewer — which is exactly why it's a habit worth breaking. "You're absolutely right" typed before the diff has been reread is a tell, not a courtesy.

## When to use

- Review comments, PR feedback, or an automated code-review report have landed on your own change.
- A suggestion is stated with more confidence than the evidence behind it, or doesn't match what you know the code does.
- A batch of feedback mixes items you follow with items you don't.
- Deciding whether to implement a comment as written, push back on it, or ask what it means.
- Not for: producing the review in the first place (see `reviewing-code-deeply`, `handing-off-for-review`). A self-review pass that comes back clean is a different risk, owned by `karen-and-the-manager` — that skill exists because your own satisfied review is suspect; this one exists because someone else's feedback earns the same scrutiny, not automatic agreement.

## Verify before you react

| Step | Question | Cost of skipping it |
|---|---|---|
| Understand | What does this comment actually claim? | You implement your first guess, which may not be the ask |
| Verify | Is that claim true of this code, right now? | You fix a problem the code doesn't have, or miss the one it does |
| Evaluate | Given what's actually true, is the suggested change right? | Compliance stands in for judgment |
| Respond | Fix it, ask, or push back with a reason | The reviewer can't tell whether you checked or complied |

Restating the comment — in your own words, back to the reviewer if needed — is what surfaces the gap between "I understood this" and "I'm about to guess." If the restatement doesn't come easily, that's the clarification path below, not a cue to implement your best guess and see what happens.

## Disagreement and confusion are both legitimate answers

Verification lands you in one of four places, and only one of them looks like blind compliance:

- **The comment is right.** Fix it. State what changed, not that the reviewer was correct to say so — the diff already proves that; thanking them for the observation is a sentence that carries no information.
- **The comment is wrong, and you can show why.** Say so, with the specific fact that makes it wrong — the test that already covers this case, the platform constraint that rules it out, the reason the code does the odd-looking thing on purpose. "I disagree" with no reason attached is exactly as performative as agreeing with no reason attached; it just points the other way.
- **You don't understand what's being asked.** Say which part, specifically. This is not the weaker move — guessing at an unclear comment and implementing the guess is the failure; asking is the correct response to a comment you can't yet act on.
- **You can't verify it from where you're sitting.** Name what's missing — access to a system, a benchmark, someone who remembers why — and ask how to proceed. Silent compliance and silent non-compliance are both worse than naming the gap.

Being wrong about a pushback isn't a special case. "Checked X — it does Y. Implementing" is the whole correction. No apology, no re-explaining why the first read seemed reasonable at the time; that spends more words defending a position you've already conceded.

## Clarify the whole batch before implementing any of it

A comment naming six problems is often describing one shape, seen six places. Implementing the four you follow while deferring the two you don't risks getting the four wrong too — the unclear ones may be what explains what the others are for. Resolve every unclear item before implementing any of them. "I follow 1, 2, 3, and 6; 4 and 5 need clarification first" is a complete response on its own, and a better one than four fixes plus two questions filed after the fact.

Once the batch is actually understood, order the work: what's blocking (breakage, security) before what's cosmetic, mechanical fixes (a typo, an import) before ones that touch logic, and each change tested on its own before the next lands. A batch applied and tested together hides which fix broke what.

## Check the claim, not the confidence

A suggestion stated flatly isn't evidence it's correct — reviewers, automated ones especially, comment from what a diff shows them, which is less than what the repository knows. Before implementing, check what the comment can't see from a diff alone: does this hold for the versions and platforms the code actually ships on? Does it break a behavior something else currently depends on? Is the current shape deliberate — a test pinning it, a comment explaining it, a decision on record — or genuinely just stale? A reviewer proposing to "implement this properly" is worth a grep for actual callers first; generalizing code nothing calls is waste with a review comment's blessing, not without one.

Scale the checking to how much context the source could plausibly have. A bot, or a reviewer working from the diff alone, knows less than you do sitting in the repository; someone who built the subsystem starts with a higher prior of being right, but still gets checked, not assumed. Either way, a suggestion that runs into a decision already on record — an ADR, a wontfix, a comment explaining why the code doesn't do the obvious thing — isn't yours to resolve alone in either direction; that's `revalidating-decisions`.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The reply agrees before the diff has been reread | Agreement produced on reflex, not after a check |
| A fix applied, then reverted, then reapplied differently | Implemented before verifying; the first version answered the wrong claim |
| Four items fixed, two questions asked afterward | Batch worked in arrival order instead of clarify-then-order |
| A generalized, configurable version of code nothing calls | "Implement it properly" taken literally instead of checked against actual usage |
| "Thanks for catching that!" followed by a two-line diff | Gratitude standing in for the fix, as if the sentence were the deliverable |
| Pushback that turns into a paragraph defending the original code | The technical reason is buried under the discomfort of disagreeing |
| An unclear item quietly dropped from the batch | Guessed silently, or hoped nobody would notice it wasn't addressed |
| The same suggestion resurfaces two reviews later, still wrong | Implemented without checking it against a decision already on record |

## Red flags

- "You're absolutely right" or a cousin of it, forming before the code has been reread
- Typing "thanks for catching that" where a diff would do
- Implementing the parts of a comment you followed while staying quiet about the parts you didn't
- A pushback with no fact in it — just a restatement that you think it's fine
- "I'll implement this properly" with no grep run first to check anyone calls it
- Every item in a batch applied in one commit, tested only at the end
- A correction to your own pushback that spends more words apologizing than stating what you checked
