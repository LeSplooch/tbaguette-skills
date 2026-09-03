---
name: tending-tbaguette
description: Use at the start of every conversation, in any project or repo, and keep watching for the rest of it — the moment a genuinely project-agnostic lesson shows up while using TBaguette, capture it. Triggers include a correction that generalizes past this one codebase, a gap or wrong assumption found in a skill while that skill was running, a recurring judgment call nothing covers yet, a TBaguette skill that looks wrong enough to want editing, an installed plugin that already carries hand-edits, or any question about how to contribute to TBaguette. Covers the bar a candidate has to clear, capturing one without derailing the current task, scrubbing it of anything project-specific, choosing which existing skill a lesson belongs in, the approval gate that no absent human lifts, opening the pull request from a fork and answering the review it gets, restoring an install that was edited in place, and how a merged change comes back through keeping-tbaguette-current.
---

# Tending TBaguette

## What this is

TBaguette is a library of project-, stack-, and language-agnostic skills.
It only gets better if it keeps absorbing what actually gets learned in
real work — and that work happens in *your* conversations, about *your*
projects, almost none of which have anything to do with TBaguette itself.
The lesson is right there in the transcript for about ten minutes, and
then it is gone.

This skill is the always-on watcher that catches those moments, and the
route that turns a caught moment into a pull request against the published
repo.

Two clearly separate jobs, on two different clocks:

- **Capture** — cheap, constant, every conversation. Notice something,
  write two sentences to a queue file, get back to whatever you were
  actually doing. This must never derail the current task.
- **Contribute** — occasional, at a natural breakpoint, and never without
  an explicit yes. Turn queued candidates into a real pull request.

Capture is meant to start at conversation turn one. When this skill gets
invoked partway through instead, it runs a one-time catch-up first — see
"Starting mid-conversation" — before settling into the same ongoing watch.

## Why a pull request, and never an edit in place

<EXTREMELY-IMPORTANT>
No TBaguette skill gets changed by editing the installed copy. A change to
a skill goes upstream as a pull request, or it does not happen. There is no
"just this once" branch of this rule and no size of change that is below
it.
</EXTREMELY-IMPORTANT>

Three separate things make that the rule, and they stack:

**The licence.** TBaguette is free software under the GNU General Public
License, version 2. Be precise about what that does and does not compel,
because a rule defended with a false reason is a rule that collapses the
first time someone checks. GPLv2 says that if you pass a modified copy on
to anyone else, you must pass it on under these same terms, with source.
It does not oblige you to send anything upstream; a change you make and
keep entirely to yourself is yours to keep. What the licence establishes is
the *direction* the project runs in — improvements stay free and stay
shared — and this skill is how TBaguette makes that real rather than
nominal. It ships inside the plugin on purpose: every recipient of the
software also receives the means to send changes back, so the share-alike
promise has a working route rather than only a legal one.

**The freeze.** This is the mechanical one, and it bites immediately. The
installed plugin is a git clone, and `keeping-tbaguette-current` refuses to
update a clone with local changes — it will not discard your work to force
an update through. So a hand-edit does not merely sit there. It pins your
entire install at that commit, silently, for as long as it exists. You
traded every future skill in the library for one local edit, and nothing
announces the trade.

One specific wrong belief is what makes people comfortable doing this, and it
is worth naming because it is the one they actually hold: *the next update
will quietly overwrite my edit anyway, so it is temporary.* It will not, and
it is not. The update reads the tree before it touches anything and backs off
rather than discarding your work. The edit is not a sandcastle waiting for
the tide — it is a wedge holding the door shut, and it holds until somebody
removes it by hand.

**The direction of travel.** An edit that stays local decays. Upstream
moves, your copy does not, and the improvement you were proud of becomes a
merge conflict nobody remembers the reason for. The same edit, merged, comes
back to you automatically — and to everyone else — the next time
`keeping-tbaguette-current` runs. That is the whole point: **your
improvements reach you through the update path, not through your working
tree.**

## Where a project-specific rule actually belongs

Most edits someone wants to make to an installed skill are not corrections to
that skill at all. They are a *local* fact — this estate is camelCase, this
service already validates upstream, this team does not use feature flags —
colliding with a library that is deliberately project-agnostic. The skill is
not wrong; it is general, and that generality is the entire reason it works
in a repo it has never seen.

A local fact has a home, and the home is not inside the plugin. It goes in
the project's own instructions: `CLAUDE.md` at the repo root, or a
project-level skill living in the repo. Either one sits *above* a library
skill rather than modifying it, and that buys four things a hand-edit cannot:

- It is version-controlled where the team can see it, rather than sitting in
  one person's home directory.
- It reaches everyone working in that repo, rather than one machine.
- It survives every update, because nothing overwrites it.
- The install stays clean and keeps updating.

**Reach for this first, every time.** Then ask the separate question: is the
skill *also* wrong in a way that would be wrong in any repo? If yes, that
part goes upstream as a pull request. If no, there was never an upstream
change to make here — only a project that needed to say something about
itself.

Getting this backwards is the expensive mistake in both directions. A
collision between one project and a general rule, sent upstream, asks a
maintainer to make the library worse for everyone else. The same collision
hand-edited into the install freezes the install. The project's own
instructions are where it belonged from the start.

## The bar: what actually belongs in TBaguette

Every skill here is a **reusable judgment call or technique**, stated so it
applies regardless of language, framework, or project.
`scripts/content_pipeline.py`'s `CATEGORIES` in the repo is the live list of
what the library already covers, and reading it beats guessing.

Worth capturing:

- A correction someone gave you that generalizes past this one codebase —
  "don't do X because Y", where Y is not specific to this stack.
- Using a TBaguette skill and finding it had a real gap, a wrong
  assumption, or a case it missed. This is usually higher-value than a new
  skill and much lower-risk to land.
- A judgment call that came up, felt like it *should* have had a skill
  already, and didn't.

Not worth capturing:

- Anything tied to a specific codebase, company, product, library, or API
  that doesn't generalize. That is a note for your own project, not a skill
  for a library that has to survive being read cold in a repo it has never
  seen.
- A one-off bug fix with no reusable lesson in it.
- A fact to remember rather than a technique or judgment call to apply.

Close calls get captured anyway. The contribute phase re-evaluates every
candidate with fresh eyes before touching anything, so a weak candidate
costs nothing to discard later. Capturing is cheap; opening a public pull
request on something half-baked is not.

## Non-negotiable: scrub before writing anything down

This skill runs in conversations about all kinds of things, some of them
private, proprietary, or under an agreement you did not read. **Before
capturing *and* again before contributing**, strip every identifying
detail: project names, company names, product names, internal URLs,
hostnames, business logic, anything that is not the generalizable technique
itself.

If the lesson cannot be stated without referencing specifics, it is not
agnostic enough for TBaguette. Leave it out entirely rather than sanitize
it down to something misleading. A pull request is public and permanent
from the moment it opens — there is no scrubbing it afterward, only
apologizing for it. `redacting-sensitive-output` covers the general
technique.

## Capture procedure

Queue file: `~/.claude/tbaguette-candidates.md` — deliberately outside the
plugin's own directory, which every update overwrites and which must stay
clean for the reasons above. Create it with **both** headings if it does
not exist — `## Pending` and, under it, `## Shipped`. Step 10 moves entries
between the two, and a file with only the first is one where the first
finished contribution has nowhere to go and invents a heading instead.

Append under `## Pending`:

```
### <YYYY-MM-DD> — <short title>
- Kind: improve `<existing-skill-slug>` | new skill
- Observed: what happened (scrubbed, per above)
- Why agnostic: the one-sentence generalization
- Sketch: rough shape of the fix or the new skill's angle
```

That is the whole capture step. Do not stop to draft the skill content
now, do not go looking at the repo now — two sentences, then straight back
to what the user actually asked for.

## Starting mid-conversation

This skill is meant to be watching from the first turn. Sometimes it is
not: it gets invoked explicitly, or its trigger finally fires, partway into
a conversation that already has real history. That history was never
watched, so catch up on it once before returning to the triggering request:

1. Read back over the conversation from its actual start — not just the
   triggering message — applying the bar above exactly as if capture had
   been running the whole time. A long conversation can hold several
   capturable moments, not just the most recent one. Find all of them.
2. Run the capture procedure on each moment that clears the bar. Same
   scrub, same file, same one-entry-per-moment format.
3. Finding nothing is a normal outcome, not a failed scan. Same rule as an
   empty `## Pending`: don't narrate it, don't report that the sweep came
   up empty. Silence is correct.

Run this inline, in the current conversation, never in a subagent — the
transcript *is* the input, and a fresh agent does not have it.

Read that rule with its reason attached, because the reason is what bounds it.
"Never in a subagent" follows from *this* conversation's transcript being the
input. A run genuinely working from records it fetches rather than from a
transcript it is inside — a scheduled pass over sessions that already ended, say
— is no longer the case the rule describes, and it is free to delegate. What
that run should expect to get back is the next paragraph's subject.

**The sweep works backward through one conversation, and not much further.**
Reading back over the conversation you are in is cheap and it works: the
transcript is right there, in order, with each correction still attached to
whatever provoked it. Do not read that as a general ability to recover capture
after the fact. Across conversations that have already ended it does not hold,
and the reason is worth knowing before you lean on it — a lesson has no
consistent surface form. Corrections rarely contain the words you would think to
search for, so a keyword sweep over finished sessions comes back nearly empty
while those same sessions were full of material at the time.

Which is what the queue file is actually for. It is not a convenience for
batching work; it is the only durable record that a moment happened, and the
transcript is not a backup for it. A retroactive pass over a body of *finished*
work therefore reads the queue and the artifacts still visible — the commits, the
diffs, the notes — rather than fishing transcripts for a shape they do not have.
And a moment nobody captured is, in practice, gone. That is the whole argument
for two sentences now rather than a better write-up later.

Once done, it is done. Go back to watching turn by turn like a skill that
started on turn one.

## When to contribute

At the start of a conversation if `## Pending` has anything in it; after
finishing the actual task, if there is a natural lull; or whenever the user
asks. Never mid-task — do not interrupt live work to go run a git pipeline.

If `## Pending` is empty there is nothing to do, and saying so is worse
than saying nothing.

## The approval gate

<EXTREMELY-IMPORTANT>
Nothing gets pushed, forked, or opened as a pull request without an
explicit yes from the user, obtained through the harness's own question
tool, for this specific contribution.
</EXTREMELY-IMPORTANT>

Ask through whatever structured question tool the harness gives you —
`AskUserQuestion`, an elicitation call, a multiple-choice prompt, whatever
yours calls it. Prose fallback is for the absence of such a tool, not for
convenience.

Check that absence rather than reading it off. A harness can defer its tools —
they exist and are callable, but their schemas are missing from the list until
something fetches them — so a question tool can be entirely available and still
not appear where you looked. Query the harness's own index for one by name and
then by keyword before concluding there is none; `routing-around-capability-gaps`
covers why reading a list is not the same as looking. It matters more here than
almost anywhere else, because this is the one gate this skill will not let an
unattended run substitute.

Ask **after** the edit exists and the suite is green, and **before** the
first command that reaches the network. That ordering is deliberate: asking
earlier means asking about a change nobody can see yet, and asking later
means the answer is decoration on something already published.

Put the actual content in the question — which skill, what changes, how
many lines — not "may I proceed?".

Then say what the yes *does*, because "open the pull request" names an
intention rather than an outcome. Short bullets, inside the question itself,
naming what will exist afterwards that does not exist now:

- a public fork of the repository under their account, if they have none yet
- a branch pushed to that fork, carrying every commit on it
- a pull request against `LeSplooch/tbaguette-skills`, open under their GitHub
  identity and readable by anyone
- a diff of roughly a hundred files, nearly all of them the regenerated site
- a maintainer who now has something to act on, which closing it later does
  not undo

Five lines, not five paragraphs — if it needs a paragraph it belongs in the
description of the change, not in the consequences. The reason to spell them
out is that the person answering is almost always approving the *change*: they
read the skill, they read the diff, they decided the lesson holds. The part
that is irreversible is none of that. It is the publication, under their name,
of a thing strangers can read and a maintainer has to answer. Someone can
genuinely want the change and still not want that today, and they can only
tell you so if the question distinguishes them.

The four options that earn their place:

| Option | What it does |
|---|---|
| Open the pull request | Push the branch and open the PR |
| Show me the diff first | Print the full diff, then ask again |
| Keep it queued | Leave the candidate in `## Pending`, push nothing |
| Drop it | Move it to `## Shipped` with a one-line reason |

Three things that are **not** approval, each of which has been mistaken for
it: enthusiasm earlier in the conversation about contributing to TBaguette;
a yes given for a previous contribution in this same session; and the user
having invoked this skill at all. Approval is per-pull-request. Getting one
yes does not bank a second.

**In a run with nobody there, this gate is not substituted — it is where the
run ends.** That is worth stating plainly, because the library's own
`bounding-autonomous-work` says an unanswered gate keeps its job and changes its
mechanism, and an autonomous run reading that rule alone could reason its way
into writing itself a self-approval here. It does not apply. Opening a public
pull request is outward-facing and permanent from the instant it happens, which
is the one category no confidence level and no envelope converts into a
self-answer. So an unattended run does everything up to it — the edit, the
suite, the adversarial pass, the commit on a local branch — and then stops with
the work staged and a report saying exactly what is waiting for a yes. Reaching
that point is the run finishing correctly, not failing.

## Contribution procedure

The full pipeline is [`reference/contribution-procedure.md`](reference/contribution-procedure.md) —
read it before the first candidate of a session and work from it, not from
memory. It also covers the case where the install itself has already been
edited in place, which is common and recoverable.

Run it once per candidate, fully, before starting the next. **Never batch
several candidates into one pull request** — a reviewer who wants two of three
changes then has to reject all three.

The shape, so this file still says what happens:

1. Fork and clone a working copy that is *not* the install, and branch from
   current `upstream/master` rather than from whatever the fork last saw.
2. Re-evaluate the candidate against the bar with fresh eyes, scrub applied
   again. One that no longer holds up gets dropped with a one-line note — that
   is the filter working, not a failure.
3. Make the edit, then write the update note in `UPDATES.md`.
4. Run `python3 scripts/run_tests.py`, fully green — and read what green
   means here. The suite checks the *filing*: registries agreeing, counts
   matching, manifests at one version, the update note well-formed. Nothing in
   it can see whether a section is true, belongs where you put it, or is
   reachable from the description. Green is a precondition for the next step,
   never a substitute for it (`confirming-before-claiming-done`).
5. `red-teaming-your-own-work`, then `karen-and-the-manager`.
6. Commit — then **ask**, which is the approval gate above and the one step
   nothing past this point runs without.
7. Push, open the pull request, record the URL in the queue file, and report it
   as *waiting on a maintainer* rather than as shipped.
8. Answer the review when it comes back. A maintainer arguing with a section is
   not a request to delete it and not a request to defend it — `verifying-review-feedback`
   owns which of those a given comment is, and the same rule applies to your own
   change as to anyone else's: fix it, refute it with a reason, or withdraw it,
   and never silently drop one. A contribution that goes quiet under review costs
   the maintainer more than one that was never opened.

Four decisions inside step 3 have no gate behind them, and a session that never
opens the reference can still get them wrong quietly:

**First check the library does not already say it.** The strongest reason to
drop a candidate is that some skill already covers it, and that is invisible
from the candidate — a capture records what you noticed, never what the corpus
already knew. Whichever skill states it is filed by the family of judgments the
lesson joins, so it is usually not the file the subject points at. Search in the
vocabulary the covering skill would have used, before drafting: a section
already written is expensive to abandon and easy to keep on the grounds that it
puts the point more sharply.

**Which skill owns it is decided by neighbours, not by subject.** Read the
candidate file's section headings and ask whether yours reads as a sibling or as
a visitor. Filing into the least-wrong skill is not parking it safely — it is
invisible to everyone who needed it.

**Then check the `description` would route someone to the section you wrote.**
It is the only always-loaded part of a skill, so it is the entire routing
surface, and a new section in a file whose description never mentions the
question it answers is unreachable while every suite stays green. The 1024
character cap is enforced by the format itself; past it, a new trigger has to
displace an older one rather than join it.

**Expect the commit to touch most of `docs/`.** The pre-commit hook regenerates
the site, every page carries a version and a build timestamp, so a one-file edit
lands as comfortably a hundred files. That is correct rather than damage — the
site is served straight out of `docs/`. Do not trim them back out, and do say in
the pull request body which files are the change and which are the regenerated
site.

## Safety rails

- **Never push to `LeSplooch/tbaguette-skills` directly.** Contributions go
  through a fork and a pull request. A push that unexpectedly succeeds is a
  reason to stop and check what account is active, not to carry on.
- Never force-push, never rewrite published history.
- Never open a pull request without the approval gate above.
- Never commit red, and never open a pull request on red.
- Always `fetch upstream` and branch from `upstream/master` immediately
  before editing — the repo moves.
- The queue file is personal state. It never gets committed to TBaguette.
- The install directory is a read-only dependency. Nothing writes to it
  except `keeping-tbaguette-current`.
- When genuinely unsure whether something clears the agnostic bar, leave it
  in `## Pending` for a later pass rather than pushing a borderline call
  under time pressure.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The install has not updated in months and nobody noticed | A hand-edit froze it; `keeping-tbaguette-current` has been declining to update ever since, exactly as designed |
| A pull request nobody can evaluate | The observation behind it was scrubbed into meaninglessness, or never included at all |
| Three unrelated changes in one pull request | Batched candidates; a reviewer who wants two of them now has to reject all three |
| The suite is red on a new skill and the new files all look right | The registry, the count, or the inbound cross-reference — none of which live in the new skill's own directory |
| A one-word description tweak turned the suite red | The description was already near the 1024-character cap the format sets; past it, a new trigger has to displace an old one |
| A pull request opened, then a correction pushed two minutes later | The approval gate was treated as the review, instead of running after one |
| A yes, then surprise at a public pull request under their own name | The question described the change and never said what approving it would create |
| A candidate captured mid-task, and the task never got finished | Capture is two sentences; anything longer is the contribute phase running at the wrong time |
| The same idea proposed twice | It was never moved to `## Shipped` with its URL |
| "Contributed to TBaguette" reported for an open pull request | Opened and merged are different states, and only one of them reaches other users |
| A well-written section that nobody ever seems to reach | It answers a question the skill's description does not mention, so nothing routes there |
| A lesson filed somewhere defensible that no reader would think to look in | Filed by its subject matter rather than beside the family of judgments it belongs to |

## Red flags

- "It's a one-line fix, I'll just edit the installed copy."
- "The next update will overwrite it anyway, so it's temporary." — it will
  not; it will stop updating instead.
- "It's only my machine, it affects nobody else." — it affects every future
  version of every skill you have, which is the opposite of nobody.
- "Keeping the install pristine is superstition." — it is not kept clean for
  its own sake; clean is the precondition for it ever changing again.
- "I'll open the PR and mention it to them after."
- "They said yes to the last one, so this one's fine."
- "I'll scrub the project name out of it later, before it merges."
- "The skill is wrong and I need it right *now*" — the fork takes ninety
  seconds and does not freeze your install.
- "I'll batch these three candidates so it's one review."
- "Nobody will notice the install is pinned."
- "The prose is in the file, so the change is done" — with a description that would never send anyone there.
- "Nobody is around to approve it, so I'll substitute the gate" — every other gate, yes; this one is the stop.
- "They already said the lesson was worth contributing" — that is a yes to the change. The fork, the branch, the pull request and their name on it are a different question, and it is the one that cannot be taken back.
