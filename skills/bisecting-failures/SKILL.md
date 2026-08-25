---
name: bisecting-failures
description: Use when something worked before and is broken now, when the culprit is one of many commits, config keys, dependency upgrades, data rows, or uncommitted edits, when `git bisect` runs into build failures, flaky tests, or merge-heavy history, or when there is no version control to search at all. Covers binary search over any axis of change and writing an unattended bisect script.
---

# Bisecting failures

## Overview

Binary search applies to any set of differences you can split in half and test, not just commit history. Bisection converts an unbounded "what broke it" question into ⌈log₂ n⌉ mechanical runs, which is why it beats reading the diff whenever n exceeds about 20.

## When to use

- A known-good state and a known-bad state exist and something between them is the cause
- The diff between them is too large to read, or spans repos you do not control
- A dependency upgrade, config change, or data import preceded the failure
- The working tree has accumulated many edits and one of them breaks the build
- Reading the change set has already failed once

Not for: a failure with no known-good state (nothing to bisect between — find the defect directly), or a test that fails at an unmeasured rate (`reproducing-bugs` first, then come back with a number).

## Preconditions, in order

1. **Verify both ends by running the test.** Assuming the old version was good is the most common way a whole search is wasted; roughly one time in five it was never good and you are searching an empty interval.
2. **Have a test that discriminates.** Deterministic, or with a measured reproduction rate. Narrow: a single case, not the suite.
3. **Make each step independent.** Clean build outputs, reset database and caches, reinstall dependencies from the candidate's own lockfile. Stale artifacts leaking between steps is the number one cause of a confidently wrong bisect answer.
4. **Do the cost arithmetic.** ⌈log₂ n⌉ runs: 1,000 candidates is 10 runs, 1,000,000 is 20. At 20 minutes per run that is 3+ hours — reduce the test before starting, not after run 6.

## The axes

| Axis | How to halve | Gotcha |
|---|---|---|
| Commits | `git bisect start bad good`, then `git bisect run <script>` | needs clean builds at every point |
| Uncommitted working-tree edits | split the diff, apply half, test, recurse | hunks are often not independently applicable — split by file first, hunk second |
| Input data | halve rows, records, or bytes | both halves passing means an interaction; split by a different dimension |
| Configuration | start from the known-good config, apply half the deltas | ordering and defaults matter; some keys only take effect together |
| Dependency versions | pin one dependency at a time, halve its release list | transitive resolution changes underneath you; lock the whole graph |
| Environment | halve the environment variable or feature-flag set | some variables are only read at process start |
| Time | halve over nightly builds, dated artifacts, or backups | works with no version control at all |
| Deployment fleet | halve the set of hosts, regions, or shards | tells you it is environmental, not code |

Bisecting the input and bisecting the code are different searches with different answers. Run the input search first when the failure is data-dependent: it is usually faster and it hands you a minimal reproduction as a side effect.

## The bisect script

The script is what makes bisection unattended, and its exit codes are what make it correct.

```
build || exit 125          # cannot test this candidate — skip, do not judge
setup_clean_state
timeout 300 run_one_test   # hard timeout, or one hang stalls the whole search
case $? in
  0)   exit 0 ;;           # good
  124) exit 125 ;;         # timed out: skip unless the hang IS the bug
  *)   exit 1 ;;           # bad
esac
```

- `0` = good, `1`–`124` and `126`–`127` = bad, `125` = untestable/skip, `128`–`255` = abort the run. Getting `125` right is the difference between a broken intermediate build being skipped and it being scored as good, which lands you on an innocent commit.
- A script that exits 0 when the build fails marks every unbuildable candidate "good" and reliably blames the wrong change. Fail loudly toward `125`, never toward `0`.
- Decide once, in a comment at the top, which direction is "good". When the bug is "a feature stopped working" rather than "a crash appeared", the polarity inverts and everyone gets it backwards at least once.
- Append revision and result to a log file. Bisect state is easy to lose and expensive to recreate.
- Keep the test script outside the tree being bisected (a copy in a temp path), or checkouts will replace it mid-search.

## When bisect misbehaves

| Symptom | Cause | Move |
|---|---|---|
| Answer is a plausible-looking innocent commit | stale artifacts, or a build failure scored as good | rerun with a clean-build guard and verify the accused commit by hand |
| Flaky test | each step's verdict is a coin flip | run k times per step, call good only if all k pass. At reproduction rate p, k = ⌈ln 0.05 / ln(1−p)⌉ — 5 at p=0.5, 14 at p=0.2. If k > 10, fix the flake first |
| Long unbuildable stretch | broken intermediate history | skip individually; if a whole region is dead, cherry-pick the test onto each candidate instead |
| Merge-heavy history | commits from a branch interleave with mainline | bisect first-parent only to find the offending merge, then bisect inside that branch. Two cheap searches beat one confusing one |
| Culprit is a merge commit | the defect is an interaction between two branches | diff the merge against each parent; neither side is wrong alone |
| Lands on a revert or a "fix the test" commit | introduced, reverted, reintroduced | abandon bisect; search the history for every touch of the relevant symbol or file |
| Result contradicts the good end | the good end was never good | re-verify both ends and restart |
| Every step is bad | the bug predates your "good" | move the good end further back by doubling the distance |

## Bisecting without version control

The change set is still a set. Halve it directly.

- Two artifacts, one working and one not: move half the files from good to bad, test, recurse. Works on deployed bundles, containers, and firmware images.
- Configuration: start from the shipped default (known good by construction) and apply half the local settings.
- Dependencies: diff the two installed manifests and bisect the version differences.
- Compare the running artifacts, not the sources. What is deployed and what is in the tree diverge more often than anyone expects, and that divergence is itself frequently the answer.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Bisect gave a nonsense answer" | build artifacts, caches, or dependencies not reset between steps |
| Manual bisect abandoned halfway | no script, so each step cost human attention and the search was never finished |
| Bisect run takes all day | full suite per step instead of the one failing case |
| Bisect blames a whitespace or formatting commit | that commit rebuilt something the incremental build had been serving stale |
| Found the commit, still do not know why | bisection names the change, not the mechanism — the commit's diff is the start of the real investigation, and `diagnosing-before-fixing` is what that investigation looks like |
| Bisecting a bug reported today across a year of history | the failure may be environmental; verify the old build fails *now* before assuming a code cause |

## Red flags

- "I'll just read the diff" when the range is more than about 20 commits
- Starting the search without running the test on the good end
- Marking a candidate by hand because "that one obviously isn't it"
- A bisect script with no timeout, or with no path that exits 125
- Accepting the answer without reverting the accused change and confirming the failure disappears — `confirming-before-claiming-done` on the one claim a whole search exists to produce
