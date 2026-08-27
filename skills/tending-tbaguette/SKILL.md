---
name: tending-tbaguette
description: Use at the start of every conversation, in any project or repo, and keep watching for the rest of it — the moment a genuinely project-agnostic lesson shows up while using TBaguette, capture it. Triggers include a correction that generalizes past this one codebase, a gap or wrong assumption found in a skill while that skill was running, a recurring judgment call nothing covers yet, a TBaguette skill that looks wrong enough to want editing, an installed plugin that already carries hand-edits, or any question about how to contribute to TBaguette. Covers the bar a candidate has to clear, capturing one without derailing the current task, scrubbing it of anything project-specific, the approval gate that runs before anything is pushed anywhere, opening the pull request from a fork, restoring an install that was edited in place, and how a merged change comes back through keeping-tbaguette-current.
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

**The direction of travel.** An edit that stays local decays. Upstream
moves, your copy does not, and the improvement you were proud of becomes a
merge conflict nobody remembers the reason for. The same edit, merged, comes
back to you automatically — and to everyone else — the next time
`keeping-tbaguette-current` runs. That is the whole point: **your
improvements reach you through the update path, not through your working
tree.**

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

Ask **after** the edit exists and the suite is green, and **before** the
first command that reaches the network. That ordering is deliberate: asking
earlier means asking about a change nobody can see yet, and asking later
means the answer is decoration on something already published.

Put the actual content in the question — which skill, what changes, how
many lines — not "may I proceed?". The four options that earn their place:

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

## Contribution procedure

Run once per candidate, fully, before starting the next. Never batch
several candidates into one pull request — a reviewer who wants two of
three changes then has to reject all three.

**One-time setup.** This path uses GitHub's `gh` CLI, which is not installed
by default on any platform. Run `gh auth status` first: it fails loudly both
when the tool is missing and when it is present but not logged in, which are
different problems with different fixes. Without it, everything below still
works from a browser — push the branch, then open the pull request from the
compare page GitHub offers on your fork. Only step 9 changes.

Then fork and clone a working copy that is *not* the install:

```bash
gh repo fork LeSplooch/tbaguette-skills --clone=false
git clone https://github.com/$(gh api user -q .login)/tbaguette-skills.git ~/.claude/tbaguette-contrib
git -C ~/.claude/tbaguette-contrib remote add upstream https://github.com/LeSplooch/tbaguette-skills.git
```

Everything below happens in `~/.claude/tbaguette-contrib`. The install at
`~/.claude/skills/TBaguette` is never touched, never edited, and stays
clean so it can keep updating.

1. **Re-evaluate the candidate** against the bar, with fresh eyes and the
   scrub applied again. If it does not hold up on a second look, move it to
   `## Shipped` with a one-line note on why it was dropped and stop. That
   is not a failure, it is the filter working.

2. **Start from current upstream**, not from whatever the fork last saw:

   ```bash
   git -C ~/.claude/tbaguette-contrib fetch upstream
   git -C ~/.claude/tbaguette-contrib checkout -B <branch-name> upstream/master
   ```

3. **Make the edit.** Improving an existing skill means editing
   `skills/<slug>/SKILL.md`, or a reference file under that skill's own
   directory. This is the common case and by far the easier review.

   A genuinely new skill has more to satisfy, because the build gates it.
   The suite tells you which of these you missed, so run it early rather
   than working from this table:

   | What the repo demands | Where |
   |---|---|
   | Frontmatter with exactly `name` and `description`, in the "Use when A, B, or C. Covers D, E, F." register | `skills/<slug>/SKILL.md` |
   | Filed under a category in the registry the build reads | `CATEGORIES` in `scripts/content_pipeline.py` |
   | Filed in the same category in the human-readable mirror | `CATALOG.md` |
   | The skill count, which is asserted, not derived | `EXPECTED_SKILL_COUNT` in `scripts/generate.py`, plus every manifest description |
   | An inbound cross-reference from a skill that already exists | some existing skill's "Not for:" line |

   That last row is the one that ambushes people, and it is a forcing
   function rather than a bug: a skill worth adding is one some existing
   skill should be handing off to, and writing that handoff is what proves
   it has a place rather than merely a topic. You cannot go green by
   touching only your own new files.

4. **Write the update note.** Every change a user of the plugin would
   notice gets an entry at the top of `UPDATES.md`, newest first, shaped
   `## YYYY-MM-DD — Title` followed by `-` bullets. Write it for someone
   who has TBaguette installed — the observable difference, not a
   restatement of your diff. `writing-release-notes` is the register.

   Scope is the plugin and only what a user of it would notice. Repo
   tooling and site furniture are real work and are not news; they ship
   with no entry, and the build will not object.

5. **Run the suite.** From the repo root:

   ```bash
   python3 scripts/run_tests.py
   ```

   It must be fully green. Fix the real problem or abandon the candidate —
   never open a pull request on red, and never skip a suite to get around a
   failure. It is stdlib-only and needs no install step.

   The first run also wires this clone's pre-commit hook, which regenerates
   the site on every commit. Expect your commit to touch **most of `docs/`** —
   comfortably a hundred files — on top of the one file you actually edited.
   Every page carries the plugin version and a build timestamp, so any change
   at all rewrites nearly all of them. That is correct, not damage: the site
   is served straight out of `docs/`, and a commit that skipped it would ship
   a page that disagrees with the skill it is showing. Do not try to trim
   those files back out of the diff.

6. **Review your own work before anyone else has to.**
   `red-teaming-your-own-work` and then `karen-and-the-manager` are the
   standard adversarial close here, and they are most warranted exactly
   when the change looks obviously fine.

7. **Commit**, with a message that explains *why* rather than what — see
   `writing-commit-messages`, and `git log` for this repo's voice.

8. **Ask.** This is the approval gate above. Nothing past this point runs
   without a yes.

9. **Push and open the pull request:**

   ```bash
   git -C ~/.claude/tbaguette-contrib push -u origin <branch-name>
   gh pr create --repo LeSplooch/tbaguette-skills --base master \
     --head "$(gh api user -q .login):<branch-name>" \
     --title "<subject>" --body "<what changed, and the observation behind it>"
   ```

   `--head` is spelled out rather than left to inference. With `--repo`
   pointing at a repository you cannot push to, `gh` has to work out that the
   branch lives on your fork, and when it cannot it fails *after* the push —
   leaving a branch on your fork and no pull request, which reads like the
   push failed when it did not.

   The body is where the candidate's "Observed" and "Why agnostic" lines
   earn their keep — scrubbed. A maintainer reading it should be able to
   tell whether the lesson generalizes without having been in your
   conversation.

10. **Record it.** Move the candidate from `## Pending` to `## Shipped` in
    the queue file with the pull request's URL. That is the durable log,
    and it is what stops the same idea being proposed twice.

11. **Report the URL to the user**, and say plainly that it is now waiting
    on a maintainer. Do not describe an unmerged pull request as shipped,
    landed, or done — `confirming-before-claiming-done` covers why the
    distinction is worth keeping.

## If the install has already been edited

This is common and recoverable, and it is worth checking for whenever
`keeping-tbaguette-current` reports that it skipped an update because of
local changes.

```bash
git -C ~/.claude/skills/TBaguette status --porcelain
git -C ~/.claude/skills/TBaguette diff
```

Read the diff first — it is somebody's work and may be worth keeping. Then:

1. Save it: `git -C ~/.claude/skills/TBaguette diff > ~/.claude/tbaguette-local-edit.patch`.
2. Capture it as a candidate, applying the bar and the scrub like any other.
3. Restore the install to clean so it can update again. Ask before doing
   this — discarding it is the user's call, never yours, and the patch file
   from step 1 is what makes the choice reversible.
4. Run the contribution procedure on the candidate, applying the patch in
   the working clone rather than the install.

Never discard someone's local changes on their behalf to force an update
through. Save, ask, then restore.

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
| A pull request opened, then a correction pushed two minutes later | The approval gate was treated as the review, instead of running after one |
| A candidate captured mid-task, and the task never got finished | Capture is two sentences; anything longer is the contribute phase running at the wrong time |
| The same idea proposed twice | It was never moved to `## Shipped` with its URL |
| "Contributed to TBaguette" reported for an open pull request | Opened and merged are different states, and only one of them reaches other users |

## Red flags

- "It's a one-line fix, I'll just edit the installed copy."
- "I'll open the PR and mention it to them after."
- "They said yes to the last one, so this one's fine."
- "I'll scrub the project name out of it later, before it merges."
- "The skill is wrong and I need it right *now*" — the fork takes ninety
  seconds and does not freeze your install.
- "I'll batch these three candidates so it's one review."
- "Nobody will notice the install is pinned."
