---
name: revalidating-decisions
description: Use when a recorded decision — an ADR, design doc, wontfix, runbook line, or a comment saying "we can't do X because Y" — appears to rule out the work in front of you, when a constraint is quoted as settled without a date, when a workaround has outlived the thing it worked around, or when deciding whether an old choice is still the right one. Covers separating a decision's principle from its premise, judging which premises decay, re-verifying cheaply, and overturning a colleague's call without treating it as an error.
---

# Revalidating decisions

## Overview

A durable decision record freezes two different things in one sentence: the **principle** its author applied, and the **premise** about the world they applied it to. The principle is usually stable. The premise decays, silently, on a schedule nobody in your repository controls.

Trusting an old record and distrusting it are both failures. The check is what separates them, and it is almost always cheaper than the work the record is currently blocking.

## When to use

- A record appears to forbid the approach you were about to take.
- Someone quotes a constraint as settled and nobody can say when it was last true.
- A workaround exists for a bug, limit, or missing feature in something you do not control.
- Before building something expensive *because* a cheaper route is documented as impossible.
- Reviewing your own decision from six months ago, on a system that has moved since.
- Not for: finding the record in the first place (`code-archaeology`), writing one so this is easy later (`writing-adrs`), or choosing between live options (`steelmanning-alternatives`, `deciding-reversibility`).

## Principle or premise

Split every record into the two before treating any of it as binding.

| | Principle | Premise |
|---|---|---|
| What it is | A rule the author chose to apply | A fact about the world they applied it to |
| Example | "We don't work around another party's access controls" | "There is no supported interface, so the only route is the unsupported one" |
| Decays? | Rarely, and visibly — by an explicit decision to change it | Constantly, silently, with nothing in the repo moving |
| Re-check | Only when values or obligations change | Every time the record is used to block work |

A record is binding when its principle holds *and* its premises still describe the world. Most stale records have a perfectly good principle sitting on a premise that expired years ago, which is why they read as convincing right up to the moment you check.

Nothing in the wording distinguishes **"we decided against this"** from **"this was impossible at the time"**. The two deserve opposite responses, and the sentence is identical.

## Which premises decay

Sorted by how fast, and by how invisible the decay is from inside the repository:

| Premise about | Typical decay | Visible in the repo? |
|---|---|---|
| A vendor's capabilities or API surface | Months | No |
| A platform's rules, quotas, or policies | Months | No |
| A dependency's limits or missing feature | Months to a year | Only in a lockfile bump nobody reads as relevant |
| A protocol or format version | Years | No |
| Hardware, cost, or scale economics | Years, then suddenly | No |
| A legal or compliance constraint | Unpredictable, sometimes overnight | No |
| Team size, skill, or available time | Continuously | No |
| Your own architecture | With each change | Yes — the commit is right there |

**Everything above the last row expires with no commit touching your repository.** That is the whole problem: your history is a complete record of your own changes and says nothing about the world your decision depended on. No amount of `git log` will surface it.

The corollary is a useful filter — a premise about your own code is usually still true, or provably false in one search. A premise about someone else's is the one to check.

## Re-verify before you comply

The check is usually one search, one API call, one release-notes page, or one line in a changelog. Compare that against what the record is blocking: a redesign, a workaround you are about to maintain forever, a feature declared impossible.

Run the check when the record is load-bearing for the decision in front of you. Skip it when the record merely agrees with what you were going to do anyway — revalidation is a tool for unblocking, not a ritual to run against every document you pass.

Three outcomes, all of them useful:

- **Premise still holds.** The record is now stronger than it was, carrying a second date. Add it: `Re-verified <date>: still no supported interface.` This is the outcome that makes the practice safe to trust rather than a license to overturn things.
- **Premise has expired.** The decision is now open. It was correctly reasoned and is wrong today; those are compatible.
- **Premise cannot be determined.** Say so explicitly and treat the record as holding. Unverifiable is not the same as false, and `calibrating-confidence` covers stating the difference.

## Overturning someone else's call

The perceived cost of contradicting a colleague's judgment is what keeps expired premises load-bearing for years. Nearly every long-lived stale constraint survives on politeness rather than on evidence.

Defuse it by stating the shape plainly: **the reasoning was right, the world moved.** A record that was correct when written, is still correctly reasoned, and is wrong now is the ordinary case — not an indictment of anyone. Where the original author is reachable, they are usually the fastest confirmation available and the least surprised by the news.

What makes this land is bringing the check, not the opinion. "The vendor shipped this endpoint in a release last spring, so the premise behind the 2023 decision no longer holds" ends the conversation. "I think this decision is outdated" starts an argument you cannot win, because you are contesting judgment instead of updating a fact.

Record the overturn where the original lives — a superseding record that names the old one and says which premise expired. A decision reversed with no trace invites the next person to reverse it back.

## Writing records that survive this

You will author the record someone revalidates in three years. Two habits make that cheap, and both belong to the authoring side — `writing-adrs` covers them: state each premise as its own dated, attributed claim rather than as a clause inside the rationale, and mark the ones you do not control.

One rule is this skill's own, because it is what makes a record revalidatable at all: **never write "impossible" when you mean "not currently supported".** The first closes the question permanently and gives a future reader nothing to check; the second carries its own expiry date. Where you expect movement, name the trigger — a version, an event, a date.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Built an expensive workaround for a limit lifted two releases ago | Read the record as a fact rather than as a dated claim |
| An old constraint quoted confidently, with no date behind it | The premise was never separable from the principle, so nobody could tell there was anything to re-check |
| Overturned a decision, broke the thing it protected | Checked the premise, ignored the principle — only one of the two had expired |
| Re-litigated a settled architectural choice at length | The premise was internal and unchanged; revalidation was never the issue |
| "We looked into that, it's not possible" with no date attached | An undated premise being quoted as a permanent property of the world |
| Same constraint re-investigated by three people in a year | Nobody wrote down the re-verification, so each check started cold |
| Kept a workaround because removing it felt risky | Never separated "the bug still exists" from "the workaround still works" |

## Red flags

- "That was already decided." — decided on a premise, which is a different claim from still true.
- "It's in the design doc." — as evidence about the present.
- Reading a document's present tense as a statement about now rather than about its writing.
- "It's not supported" with no version, date, or source attached.
- "I don't want to second-guess them." — the check is not a challenge to anyone.
- Treating a premise you cannot verify as false because verifying it was inconvenient.
- Overturning a record without recording that you did, or why.
