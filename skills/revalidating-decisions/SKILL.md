---
name: revalidating-decisions
description: Use when a recorded decision — an ADR, design doc, wontfix, runbook line, or a comment saying "we can't do X because Y" — appears to rule out the work in front of you, when a constraint is quoted as settled without a date, when a workaround has outlived the thing it worked around, when deciding whether an old choice is still the right one, when an item has sat in a backlog across several passes and each pass re-reads it and re-queues it, or when a recurring or scheduled run keeps re-deriving a question it has already answered. Also use when a step that has always worked is about to be relied on again, when an outcome arrived but the step that was supposed to produce it was refused. Covers separating a decision's principle from its premise, judging which premises decay, re-verifying cheaply, revalidating a deferral before repeating it, and overturning a colleague's call without treating it as an error.
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

## Only the premises that hurt get a review date

Every row above decays. Only some of them are ever re-read, and which ones is decided by something other than importance: **a premise that blocks you gets recorded with a way to reopen it, and a premise that permits you gets recorded nowhere at all.**

The asymmetry is entirely in the bookkeeping, and it comes from when each one costs something. "This cannot be done — the platform does not support it" is written down at the moment it hurts, usually with the workaround it forced and sometimes with the one check that would overturn it, so a later pass pays one search and occasionally gets a large refund. "This works" costs nothing when it is true, so it goes in no record, acquires no date, and is inherited by every later reader as a property of the world rather than as an observation somebody made once. It is then discovered to have expired at the moment it is finally needed — which, for anything sitting at the end of a sequence, is after everything before it has already been spent.

So give the enabling premises the same treatment as the blocking ones: **write down what worked, dated, beside what did not.** It costs a line. The next pass then opens with a lookup instead of an assumption, and can spend its one cheap re-check on whichever premise has the oldest date rather than on whichever one it happens to remember.

**And a result you can observe is not a capability you have.** The two are indistinguishable afterwards: the branch is merged, the file is deployed, the record exists — while the step that was supposed to produce it was refused, and something else produced it instead. Someone ran it by hand, a different process did it, a colleague landed the same change from another direction. A later reader takes the outcome as proof the route is open and writes down a measurement nobody took. The discriminator is narrow and worth applying literally: **credit a capability only if you watched it work.** Where the result arrived by another route, record *that*, in those words, because nothing in the end state will ever say so.

## A rule that agrees with you has stopped being a premise

A decision can also stop mattering without ever becoming wrong. A rule, convention, or piece of standing guidance that tells an actor to do what it would have done anyway is obeyed perfectly, forever, and changes nothing — and from outside, full compliance and complete irrelevance produce the identical observation.

`choosing-test-scope` owns this for a check running against real traffic, with the instrument: sample the *distribution* of its verdicts rather than their value, and treat a check whose output has never varied as unproven rather than as passing. Two things carry past that.

The first is direction. The world moves *toward* a rule — a language gains the check a convention was invented to enforce, a tool starts doing by default what a policy used to demand, a team absorbs an old correction into habit. So this is a premise decaying like any other in the table above, except that the decay makes the rule look *better*: nothing signals the day it happens, and an overall compliance figure rises as it does, because the rules that no longer matter are the easiest ones to satisfy.

The second is that standing guidance has no verdicts to sample. Where the thing in force is a sentence rather than a branch, the instrument that remains is **withholding it** — run the case with the rule removed and see whether the outcome changes. That is worth doing for the rules it would be expensive to be wrong about, and it is not an argument for deleting whatever fails: cheap insurance against a regression is a legitimate reason to keep something inert. It is an argument for knowing which kind you hold, because a body of rules nobody has tested this way reports a health it has not earned.

None of which contradicts the advice below to skip revalidation when a record merely agrees with what you were going to do anyway. That is about a record you are *consulting* and are free to walk past. This is about a rule already *in force*, which you are not walking past — you are obeying it, and the question is whether the obedience is doing anything.

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

## The decision that was never made

A record freezes a decision. A backlog freezes a *non*-decision, and that is the
harder one to revalidate, because nothing in it announces itself as a claim about
the world. "Worth doing, not now" reads as scheduling. It is a judgment, and it
rests on a premise like any other: *I do not yet have what I need to settle this.*

That premise is unlike the ones tabled above in one respect: it is supposed to
expire from the inside. A vendor's capability expires because the world moved; a
deferral's expires because *you* moved, since every subsequent look at the item
adds information. The failure case is therefore not a stale premise but a
motionless one — nothing new arrived, because nobody went to get anything.
**A deferral repeated on inputs identical to last time is not a deferral. It is a
decline that nobody had to own.**

The mechanism is an asymmetry in cost, not a failure of attention. Deferring is
free and takes no argument; deciding costs a justification and can be wrong. So a
marginal item gets re-read, re-agreed to be marginal, and re-queued — by a careful
reader, behaving reasonably, every time. The queue reads as a backlog and
functions as a graveyard, and the give-away is an entry that is *complete*: the
analysis finished, the cost known, the only missing piece somebody willing to say
yes or no.

So apply one step earlier the discipline the section above applies before you
comply with a record — **re-verify before you defer again.**

- **Name what would change the answer, and when.** A version, an event, a
  measurement, a second sighting. That is the deferral's premise, written so it
  can expire. Put it next to the item, not in your head.
- **A second deferral cites what changed since the first.** If nothing did, the
  inputs are the ones you already declined to act on, and the honest output is a
  decision rather than another pass.
- **The third reading settles it, and there is no fourth.** Closed as done or as
  declined, with the reason. `knowing-when-to-stop` owns the general form of this;
  here the bounded thing is the number of times one item may be reconsidered.

**An item whose stated trigger has not fired is not being deferred — it is
waiting, and it does not count against any of this.** A candidate that says "when
this shape turns up in a second project" and has been seen once is exactly where
it should be, for as long as that takes. The rule bites only where no trigger was
ever written, which is why writing one is the first bullet rather than the last.

The mirror-image failure belongs to recurring and unattended passes, and costs the
same for the opposite reason: a question already answered *no* gets re-derived from
scratch every cycle, at full price, because the answer was never written anywhere
the next pass would look. An open question and a settled one look identical from a
cold start. A negative finding is a finding — record it with its date and what
would reopen it, exactly as you would record a positive one.

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
| An item re-read and re-queued every cycle, always "worth doing, not now" | Deferring cost nothing and deciding cost a justification; the premise behind the deferral was never stated, so it could never expire |
| A recurring pass re-investigates the same settled question every time | The answer was negative, and negative findings were never written down as findings |
| Kept a workaround because removing it felt risky | Never separated "the bug still exists" from "the workaround still works" |

## Red flags

- "That was already decided." — decided on a premise, which is a different claim from still true.
- "It's in the design doc." — as evidence about the present.
- Reading a document's present tense as a statement about now rather than about its writing.
- "It's not supported" with no version, date, or source attached.
- "Let's look at it again next time." — said for the third time, about inputs that have not moved.
- A backlog item whose analysis is complete and whose only missing piece is somebody willing to say no.
- "I don't want to second-guess them." — the check is not a challenge to anyone.
- Treating a premise you cannot verify as false because verifying it was inconvenient.
- Overturning a record without recording that you did, or why.
