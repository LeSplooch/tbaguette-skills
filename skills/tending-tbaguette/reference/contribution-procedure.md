# Contribution procedure

The full pipeline for turning a queued candidate into a pull request on
`LeSplooch/tbaguette-skills`, plus what to do when the install itself has
already been edited. Read it before the first candidate of a session and work
from it rather than from memory.

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

   One gate fires on that easy path too, and it is the only one that does.
   `description` is capped at 1024 characters by the Agent Skills format
   itself, not by this repo, and the suite enforces it. Descriptions are
   the field under permanent pressure to grow, because the natural way to
   make a skill fire on a newly-noticed trigger is to append that trigger
   to the list. Past the cap, the next trigger has to **displace** an older
   one rather than join it — so a description change is sometimes an edit
   to a sentence you did not come here to touch. Compressing the "Covers"
   half is usually where the room is, since it tends to restate triggers
   the "Use when" half already made.

   **First check the library does not already say it.** The strongest reason
   to drop a candidate is that some skill already covers it, and that reason is
   invisible from the candidate itself — a capture records what you noticed,
   never what the corpus already knew. The search is harder than it looks for
   the same reason the ownership rule below is: the skill that already states
   the lesson is filed by the family of judgments it belongs to, so it is
   usually *not* the file the candidate's subject points at. Search for the
   idea in the words the covering skill would have used rather than the words
   the capture used, and search wider than the two or three files you expect.
   Do this before drafting, not after. A section you have already written is
   expensive to abandon and very easy to keep on the grounds that it says the
   same thing more sharply — which is how the corpus acquires two statements of
   one idea, in two files, neither aware of the other.

   **Decide which skill owns it before writing a word of it.** A candidate
   usually arrives naming a topic, and the topic points at the wrong file
   surprisingly often. File by the *family of judgments the lesson belongs to*,
   not by its subject matter: a lesson about an installer that refused to run
   because a version check fired is not a dependency lesson, it is a
   reading-the-instrument lesson, and it belongs next to the other
   reading-the-instrument judgments even though nothing in it mentions
   debugging. The test that settles it is neighbourliness — read the section
   headings of the candidate file and ask whether yours would look like a
   sibling or like a visitor. A lesson filed into the least-wrong skill is not
   safely parked; it is invisible to everyone who would have needed it, which
   is the same failure as filing a new skill under the least-wrong category and
   is much easier to commit because no registry complains.

   **Then check the description would actually route someone to it.** This is
   the step with no gate behind it, and it is the one that quietly wastes the
   work. A skill's description is the only part always loaded, so it is the
   whole of the routing: a section added to a file whose description never
   mentions the question it answers is unreachable, and the suite is perfectly
   happy — the prose is there, it is correct, and nothing will ever surface it.
   After the edit, read the description back and ask whether the sentence that
   made you write this would land on this file. If not, the description is part
   of the change, under the cap and its displacement rule above.

   **Grep the library for your key noun before you adopt it.** A captured
   observation arrives in the words you happened to use at the time, and
   some of those words are already load-bearing here with a narrower
   meaning — `seam` means a place where behavior can be changed without
   editing the thing whose behavior changes, and `finding-the-seam` owns
   it. Shipping a capture's wording verbatim is how a second, vaguer sense
   of a defined term enters the corpus, and nothing in the suite can see
   that. One `grep -rn` over `skills/` costs seconds and is the difference
   between a reviewer reading your change and a reviewer teaching you the
   glossary.

   A genuinely new skill has more to satisfy, because the build gates it.
   The suite tells you which of these you missed, so run it early rather
   than working from this table:

   | What the repo demands | Where |
   |---|---|
   | Frontmatter with exactly `name` and `description`, in the "Use when A, B, or C. Covers D, E, F." register | `skills/<slug>/SKILL.md` |
   | A `name` matching the directory exactly, and a `description` under 1024 characters | `skills/<slug>/SKILL.md`, per the note above |
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

   A check that fails for reasons that are not yours still has to be handled,
   and the handling is evidence rather than assertion. Run the same suite on a
   pristine checkout of upstream at the commit you branched from, and report
   that the same suites fail there identically, naming your platform. Without
   that baseline, "three of twelve suites fail on my machine" is
   indistinguishable from "this change breaks the build", and a reviewer has
   no way to tell which — so it reads as the second one. With it, you have
   handed them a platform bug report they did not have, which is worth more
   than the silence would have been.

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
   conversation. Say explicitly which files are the change and which
   are the regenerated site, because from the outside the two are one
   hundred-file diff and a reviewer who cannot separate them is reviewing
   either all of it or none of it.

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
