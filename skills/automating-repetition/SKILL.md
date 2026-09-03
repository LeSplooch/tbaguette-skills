---
name: automating-repetition
description: Use when a manual sequence has been repeated often enough to consider scripting it, when deciding whether a task is worth automating at all, when a script exists but nobody knows about it or trusts it, when automation half-succeeded and left the system in a middle state, when a scheduled job has been failing silently, when a scripted edit, codemod, or migration reported success and changed nothing, when a manual step is risky, irreversible, or easy to get wrong by hand, when a person is being asked to report a state that changes faster than they can reply, or when the thing to be noticed happens on its own schedule rather than inside your procedure and no polling interval feels right. Covers the ladder from reporting to unattended, measuring a proposed rule against recorded history before arming it, and when a habit needs a watcher rather than a step.
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

Where the trigger's inputs are **already recorded** — logs, transcripts, ticket history, a metrics
store — the two-to-four-week wait is avoidable, and skipping it is not a shortcut. Replay the record
and measure how often the proposed rule would have fired. A guard, alert, lint or hook that would
have gone off on a fifth of all occasions is furniture before it is ever armed, and furniture is
ignored, which is the same as absent while looking like coverage. That is knowable in seconds
against history, and only knowable after a month of watching without it.

Replay answers a second question the alongside-run answers slowly or not at all: **does the
condition separate anything?** A rule fires on cases meeting some test, and it is worth confirming
that the cases *not* meeting it exist at all. One measured example: a proposed check keyed on a
threshold, and replay showed every occasion above that threshold shared the outcome the check was
looking for — so the threshold was not selecting the interesting cases, it was selecting all of
them, and the discriminator distinguished nothing. Watching would eventually have shown a stream of
alerts; it would not have shown *why*.


## Some conditions arise on the world's schedule, not on yours

Everything above assumes the automation runs when you invoke it, or on a
schedule you chose. A whole class of checks cannot work that way, and the tell is
specific: the honest answer to *how often should this run* is **the moment it
happens**, and every interval you can actually name is a compromise between
noticing late and looking a thousand times for nothing.

Those are watchers, not steps. A watcher runs for the lifetime of the work and
reports when the condition occurs — a file changing, a line appearing in a log, an
external state flipping, a long job finishing. Reaching for a poll there is not a
cheaper version of the same thing; it converts a fact you could have been told
into a question you now have to keep asking, and it is the reason so many of these
end up either noisy or useless.

Two costs come with it, and both are the kind that arrive quietly.

- **A watcher is a process you now own, and its silence looks exactly like the
  good case.** This is the silent-scheduled-job failure with its last symptom
  removed: a job that stops running at least leaves a gap where its output used to
  be, and a watcher's correct output on a quiet day is nothing at all, so a dead
  one and a working one produce the same record. It needs a heartbeat, or some
  periodic assertion that it is still alive, or the thing it was watching for will
  happen unobserved and the afternoon will read as calm.
- **Its output arrives out of order with whatever you were doing.** A message
  that made sense at the moment it was written is being read by someone in the
  middle of something else, possibly hours later. It has to carry enough context to
  be actionable away from the moment that produced it — which of the things being
  watched, what changed, and what the reader is expected to do — because the
  surrounding state that would have disambiguated it is gone.

One substitute for a watcher is worth naming because it is the default and almost never checked: **asking a person to tell you when it happens.** It reads as the cheap option — no process to own, no heartbeat, no stale message — and it quietly imports the round trip as a sampling interval. A human observer can only report at the rate a conversation turns, so anything changing faster than a reply is invisible to them by construction. They see the transition; by the time the message is written the state has moved on. What arrives is not late information but information about a state that no longer exists, which is worse, because it reads as current.

The tell is in the shape of the request. "Tell me when X" is fine for something that holds still once it happens. For a transition that *passes* — a screen that flashes by, a window that is open for two seconds, a process that is briefly killable — the person is being asked to be an instrument with a latency floor nobody measured. Put a watcher on it, and leave the person the part only a person can do — which is the physical action, not the observation.

None of which argues for a watcher whenever one is possible. A poll is the right
answer whenever the condition changes slowly against how long you are willing to
wait — checked twice a day, a daily thing is not worth a process — and it fails
safe, because a poll that stops running stops producing the output you were
reading. Reach for a watcher when latency is the requirement rather than a
preference, and when nobody will be sitting there to ask again.

The ladder still applies inside the watcher: report first, act later, and measure
the firing rate against the record before arming anything.

## A bulk edit runs once, so the ladder does not apply

The ladder assumes a tool with a future — run it beside the manual process, compare, promote it a rung. A one-shot mechanical edit has no future to earn trust in. Thousands of generated values across dozens of files get exactly one run, and the review that would normally catch a mistake is precisely the review that a diff of that size defeats: nobody reads the four-thousandth line with the attention they gave the first.

So the trust has to be built into the applier instead, and it takes two checks that fail in unrelated ways.

**Structural — write the tool to refuse, not to report.** For each file, assert every property that must hold before a single byte is written: every key present in the reference set and absent from the target, every format placeholder surviving unchanged, no value using an escaping form the target renders literally. Then refuse the file outright when any assertion fails. Apply-and-report is the weaker shape by a wide margin, because it produces exactly the artifact nobody can review — a mostly-correct bulk diff with the failures narrated somewhere above it.

Such a validator commonly catches nothing on the day it runs, and that is not evidence it was unnecessary. Its value is that it made a whole class of error impossible to commit, which is a property of the tool rather than an event in the log.

**Semantic — then ask what that tool cannot see.** The structural check is blind by construction to anything shaped correctly and meaning the wrong thing: a value in the wrong language entirely, a description attached to the wrong key, a number in the right format and the wrong unit. Every assertion above passes on all of them. Catching that needs a second check aimed at content rather than shape, and it usually looks nothing like the first — scanning each file for characters, units, or vocabulary that have no business being in that file at all.

The transferable move is the question, not either check: **what class of error would pass every assertion I just wrote?** Ask it while the assertions are fresh, because that is the only moment their blind spot is obvious. A tool that validates and refuses, with nothing aimed at what it cannot see, feels like a safety net and is half of one.

## A replacement that matched nothing exits zero

Pattern-driven edits share a blind spot, and it is not in the part that writes. `sed -i`, a regex codemod, a scripted find-and-replace, an `UPDATE ... WHERE` — each is handed a pattern and a substitution, and each *succeeds* when the pattern is absent. Nothing matched, nothing was written, the file comes back byte-identical, and the exit status is zero because no error occurred. None did. The command did exactly what it was told, to nothing.

Whether that gets caught depends entirely on what is being treated as the signal. An empty diff is conspicuous to anyone who looks at it; the failure survives because exit zero already answered the question, so nobody does. And it arrives at the least suspicious moment — one edit inside a batch of twenty, or a pattern written against the file as it was remembered rather than as it is, after a rename, a reformat, or an earlier edit in the same session moved the anchor out from under it.

So an edit that is *supposed* to change something has to assert that it did. Count the matches and fail on zero rather than trusting the substitution's own status; capture the target before and after and refuse to continue on an empty diff; read a database's affected-row count back and compare it against what the predicate was meant to select. All of them are cheap, and each separates two claims that every mechanical editor reports with the same number: **ran without error**, and **did the thing**.

That is the asymmetry `writing-the-failing-test-first` is built on, met through a different instrument. A test never watched failing and a substitution never watched matching share one failure — nobody has confirmed the thing is pointed at anything, so the agreeable result it returns is worth nothing.

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
| A scripted edit reported success and left the file byte-identical | Its pattern matched nothing; a substitution that changes nothing still exits zero |
| Every file passed the bulk applier's checks and one was still wrong | the checks asserted shape; the error was in the content, which a structural assertion cannot see by construction |

## Red flags

- Running the same handful of commands "quickly" for the third time this month
- A destructive tool with no dry-run and no confirmation step
- "It's fine, I'll remember the order"
- Failure output going only to a log nobody reads on a schedule nobody watches
- Adding a retry loop around an operation that should not be failing
- Automation that only runs from one person's machine
- Asking someone to tell you the moment something happens, when the something is over in less time than a reply takes
- "The script ran fine" — said about an edit whose diff was never looked at
- A generated report nobody has opened in three months — the automation now produces waste on a schedule
