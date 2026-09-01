---
name: automating-repetition
description: Use when a manual sequence has been repeated often enough to consider scripting it, when deciding whether a task is worth automating at all, when a script exists but nobody knows about it or trusts it, when automation half-succeeded and left the system in a middle state, when a scheduled job has been failing silently, or when a manual step is risky, irreversible, or easy to get wrong by hand.
---

# Automating repetition

## Overview

Automation is a bet that a sequence will stay the same, paid up front and settled over years. The losing outcome is not a script that fails loudly; it is a script that quietly stops doing what its name promises while everyone keeps trusting it.

## When to use

- The same sequence of steps has been performed by hand more than once
- A step is destructive, irreversible, or has to happen in a strict order
- A tool exists but people still do the task manually, or wrote a second one
- A scheduled or unattended job failed and nobody noticed for days
- Automation stopped partway and left the system somewhere undocumented

Not for: work that belongs in the build pipeline itself (`designing-ci-pipelines`), making the script's own shell code robust (`portable-shell-scripting`), or making the underlying operation safe to repeat at the system level (`designing-for-idempotency`).

## When a sequence earns a tool

| Situation | Automate at | Reason |
|---|---|---|
| Destructive or irreversible: deletes data, touches production, moves money, grants access | 1st repetition | one mistyped argument costs more than the whole tool |
| Frequent, stable, low risk | 3rd repetition | the rule of three; by then the shape has stopped moving |
| Strict ordering, or long gaps between steps | 2nd repetition | the failure mode is memory, not skill |
| Onboarding or environment setup | when a second person needs it | prose instructions decay faster than code does |
| Unattended and scheduled | before first run, with monitoring | nobody is present to catch the failure |
| Shape changes every run | never | you would be encoding one instance of a guess |
| Run once, or under twice a year | never | a checklist costs a twentieth as much |
| The judgment is the deliverable | never | see the last section |

The rule of three is about learning the shape, not about accumulating effort. Automating on the first repetition is correct when the shape is already fixed by something external — a documented API, a published release process, a regulator's procedure — and wrong while you are still discovering what the steps are.

## Total cost, including the silent break

Cost is authorship, plus maintenance scaled by how fast the thing underneath moves, plus the probability of silent failure times the cost of not noticing.

The arithmetic people do covers only the first two terms: a task taking time T, run N times a year, against a script costing roughly 10T to write and 2T a year to maintain, breaks even near N ≥ 12 in year one. Automation over a stable interface stays near that estimate. Automation that scrapes a UI or parses another tool's human-readable output is a subscription, not a purchase, and routinely costs more per year than the manual task it replaced.

The third term is the one that decides, and it is the one nobody prices. A backup script that stopped working six months ago is worse than never having had one, because it also sold false confidence for six months. Anything that runs unattended needs a heartbeat or a dead-man's switch, because otherwise its failure mode is silence and silence looks exactly like success.

## Properties that make automation worth keeping

- **Discoverable.** It lives next to what it operates on, appears in one index or task runner, and is named for the outcome (`release`, `restore-snapshot`) rather than the mechanism. An undiscoverable script is worse than no script: the next person writes a second one, and now two divergent implementations both claim to be authoritative.
- **Self-describing.** Invoked with no arguments or with a help flag, it states what it does, what it will change, what it needs, and who owns it. A header comment carries the date it was last verified against reality.
- **Idempotent and resumable.** Each step checks before acting, a second run is a no-op, and a run after a partial failure completes the remainder. This matters most in exactly the situation where it is used most — after something has already gone wrong.
- **Preflight, then mutate.** Validate every precondition, credential, and input before the first write. A tool that dies halfway leaves a state nobody has documented, and the recovery is manual, improvised, and performed under stress.
- **Loud, not partial.** Exit non-zero, name the step that failed, state what the system's condition now is and what to run next. Never press on past an error to get most of it done: partial success is the worst available outcome because it looks like progress.
- **Observable.** Log what changed, not that it ran. A run that changed nothing when it should have changed something must be distinguishable from a legitimate no-op, or the tool will report success for years while doing nothing. `instrumenting-for-observability` generalises the same distinction: emitting input volume alongside the result is what separates "processed zero items" from "processed nothing because nothing arrived".

## Check before you act

The safe first version reports and changes nothing. Ship that, run it alongside the manual process for two to four weeks, and compare its proposed plan against what a person actually did. Two things are earned: the logic is validated before it can do harm, and the checking version has standalone value as a drift detector even if the acting version is never written.

Progress along this ladder deliberately: report only, then print a plan without applying it, then act with confirmation, then act unattended. Jumping straight to the last rung for anything destructive is the standard origin story of a cleanup job that deleted live data.

## A bulk edit runs once, so the ladder does not apply

The ladder assumes a tool with a future — run it beside the manual process, compare, promote it a rung. A one-shot mechanical edit has no future to earn trust in. Thousands of generated values across dozens of files get exactly one run, and the review that would normally catch a mistake is precisely the review that a diff of that size defeats: nobody reads the four-thousandth line with the attention they gave the first.

So the trust has to be built into the applier instead, and it takes two checks that fail in unrelated ways.

**Structural — write the tool to refuse, not to report.** For each file, assert every property that must hold before a single byte is written: every key present in the reference set and absent from the target, every format placeholder surviving unchanged, no value using an escaping form the target renders literally. Then refuse the file outright when any assertion fails. Apply-and-report is the weaker shape by a wide margin, because it produces exactly the artifact nobody can review — a mostly-correct bulk diff with the failures narrated somewhere above it.

Such a validator commonly catches nothing on the day it runs, and that is not evidence it was unnecessary. Its value is that it made a whole class of error impossible to commit, which is a property of the tool rather than an event in the log.

**Semantic — then ask what that tool cannot see.** The structural check is blind by construction to anything shaped correctly and meaning the wrong thing: a value in the wrong language entirely, a description attached to the wrong key, a number in the right format and the wrong unit. Every assertion above passes on all of them. Catching that needs a second check aimed at content rather than shape, and it usually looks nothing like the first — scanning each file for characters, units, or vocabulary that have no business being in that file at all.

The transferable move is the question, not either check: **what class of error would pass every assertion I just wrote?** Ask it while the assertions are fresh, because that is the only moment their blind spot is obvious. A tool that validates and refuses, with nothing aimed at what it cannot see, feels like a safety net and is half of one.

## Keep the manual path working

Document the manual sequence beside the tool and mark which steps the tool performs. The automation is the fast path, never the only path, and the day it breaks is disproportionately likely to be the day it is needed — both usually fail from the same upstream change.

Exercise the manual path at least annually, and whenever the tool changes owner. Once nobody can perform the task by hand, the script is not automation; it is the last remaining copy of the knowledge, and reading it is now the only documentation.

## When not to automate

- **The task's shape changes every run.** You would encode one instance and then spend more time on the exception path than the task ever took.
- **The judgment is the point.** Review, triage, prioritization, design. Automate the gathering and never the deciding: a tool that assembles every piece of evidence for a decision is valuable; the one that makes the decision becomes a rubber stamp that nobody dares contradict.
- **The automation would hide a problem worth fixing.** A wrapper that retries a flaky step is a bug report you decided not to file, and it makes the defect permanent by making it survivable.
- **The rate is under twice a year.** A checklist costs a fraction as much, fails visibly, and is read by a person who can notice that step 4 no longer matches reality.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Three scripts do nearly the same thing | none was discoverable; each author assumed nothing existed |
| The tool went unused by the people it was written for | not findable, named for its mechanism, or it never said what it does |
| A scheduled job had been failing for weeks | success was inferred from silence; no heartbeat, only failure output was wired up |
| Re-running it made things worse | steps act blindly instead of converging; not idempotent |
| Left the system in an undocumented middle state | preconditions were checked lazily, after the first mutation had run |
| Works only for the person who wrote it | it depends on their shell state, credentials, or local paths |
| Maintenance costs more than the manual task | it is coupled to something that changes weekly, usually another tool's human-readable output |
| It broke and nobody could do the task by hand | the manual path was deleted along with the tedium |
| A day of work saved five minutes a year | the rule of three was applied to effort rather than to frequency |
| Every file passed the bulk applier's checks and one was still wrong | the checks asserted shape; the error was in the content, which a structural assertion cannot see by construction |

## Red flags

- Running the same handful of commands "quickly" for the third time this month
- A destructive tool with no dry-run and no confirmation step
- "It's fine, I'll remember the order"
- Failure output going only to a log nobody reads on a schedule nobody watches
- Adding a retry loop around an operation that should not be failing
- Automation that only runs from one person's machine
- A generated report nobody has opened in three months — the automation now produces waste on a schedule
