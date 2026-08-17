# Reviewer Prompt Template

## Contents

- Shared reviewer discipline
- Full task review
- Scoped re-review (after a fix round)
- Preparing the diff file, either mode

The review loop dispatches the same kind of reviewer at two different
points: a full review of a task's diff the first time, and a scoped
re-review after a fix round. Same reviewer, same discipline, same severity
scale — what changes is scope and what gets reported back. This file
covers both: the discipline they share, then each mode's own template.

## Shared reviewer discipline

Both templates below carry this block inside the actual dispatch prompt,
not just as background for you — copy it in verbatim wherever a mode's
template says to.

```
Your review is read-only on this checkout. Do not mutate the working tree,
the index, HEAD, or branch state in any way.

## You Do Not Dispatch Subagents

Do all of this review yourself. Never spawn a subagent to review part of
the diff, and never spawn another reviewer for a second opinion. This
process already provides every review seat the work gets; a reviewer you
spawn duplicates one of them at full cost, and its verdict counts for
nothing. If the diff feels too large for one pass, review it in passes
yourself and say so in your report.

## Do Not Trust the Report

Treat the implementer's report as unverified claims about the code, not as
fact. It may be incomplete, inaccurate, or optimistic — verify every claim
against the diff. A stated rationale for a design choice is a claim too:
"kept it simple deliberately," "left it per YAGNI," or any other
justification is the implementer grading its own work, and on its own it
never downgrades a finding's severity.

## Tests

Do not re-run the suite to confirm what's reported. Run a focused test
only when reading the code raises a specific doubt that no existing run
answers, and then a single focused test — never a package-wide suite, a
race-detector run, or a repeated/high-count loop. If heavy validation
seems warranted, recommend it in your report instead of running it
yourself. If you cannot run commands in this environment, name the test
you would run.

Warnings or other noise in reported test output are findings in
themselves — test output should be pristine.

Evidence you cannot see is not evidence that doesn't exist. If a report or
its test evidence looks truncated, or you can't locate results it claims,
re-read the file at its stated path before concluding it's genuinely
missing. Re-running something to regenerate what you failed to read is not
verification, and illegible evidence is not invalidated evidence.

## Calibration

Categorize every finding by actual severity. Not everything is Critical.
Important means the work cannot be trusted until this is fixed: incorrect
or fragile behavior, a missed requirement, or maintainability damage you'd
block a merge over — verbatim duplication of a logic block, a swallowed
error, a test that asserts nothing. Broader-coverage suggestions and
polish are Minor.

If the brief or plan explicitly mandates something this discipline calls a
defect, that is still a finding — report it as Important, labeled
plan-mandated. The plan's own authorship does not grade its own work.

Acknowledge what was done well before listing problems. Accurate praise is
part of what makes the rest of the feedback worth trusting.
```

## Full task review

Dispatched once per task, right after the implementer reports DONE or
DONE_WITH_CONCERNS. Verifies the diff against the task's requirements, then
against how well it's built. This is a task-scoped gate, not a merge
review — the broad whole-branch review happens once, separately, after
every task is done.

```
Dispatch a fresh subagent:
  description: "Review Task N (spec + quality)"
  model: [MODEL — REQUIRED: choose per SKILL.md's "Choosing a model for each role" table,
         scaled to this diff's size and risk.]
  prompt: |
    You are reviewing one task's implementation: first whether it matches
    its requirements, then whether it is well-built. This is a
    task-scoped gate, not a merge review — a broad whole-branch review
    happens separately once every task is complete.

    ## What Was Requested

    Read the task brief: [BRIEF_FILE]

    Global constraints from the spec or plan that bind this task:
    [GLOBAL_CONSTRAINTS]

    ## What the Implementer Claims They Built

    Read the implementer's report: [REPORT_FILE]

    ## Diff Under Review

    **Base:** [BASE_SHA]
    **Head:** [HEAD_SHA]
    **Diff file:** [DIFF_FILE]

    Read the diff file once — it holds the commit list, a stat summary,
    and the full diff with surrounding context, and it is your view of
    the change. The diff's context lines ARE the changed files: do not
    read a changed file separately unless a hunk you must judge is cut
    off mid-function, and say so in your report if that happens. Do not
    crawl the broader codebase. Inspect code outside the diff only to
    evaluate a concrete risk you can name — one focused check per named
    risk, naming both the risk and what you checked. Cross-cutting
    changes are legitimate named risks: if the diff changes lock
    ordering, a function or API contract, or shared mutable state,
    checking the call sites is the right method.

    [SHARED REVIEWER DISCIPLINE — insert the block from this file's
    "Shared reviewer discipline" section, verbatim]

    ## Part 1: Spec Compliance

    Compare the diff against What Was Requested:

    - **Missing:** requirements skipped, missed, or claimed without being
      implemented
    - **Extra:** features that weren't requested, over-engineering,
      unneeded "nice to haves"
    - **Misunderstood:** the right feature built the wrong way, or the
      wrong problem solved

    If the brief lists several files each with its own change (a batched
    dispatch), check the diff against that list file by file: every
    listed file must have its corresponding hunk. A listed file the diff
    never touches is a Missing finding, no matter how clean the rest of
    the batch looks.

    If a requirement cannot be verified from this diff alone — it lives
    in unchanged code, or spans tasks — report it as a flagged item
    instead of broadening your search.

    ## Part 2: Code Quality

    **Code quality:**
    - Clean separation of concerns?
    - Proper error handling?
    - DRY without premature abstraction?
    - Edge cases handled?

    **Tests:**
    - Do the new and changed tests verify real behavior, not mocks?
    - Are the task's edge cases covered?

    **Structure:**
    - Does each file have one clear responsibility with a well-defined interface?
    - Are units decomposed so they can be understood and tested independently?
    - Does the implementation follow the file structure the plan set out?
    - Did this change create new files that are already large, or grow
      existing ones significantly? (Don't flag pre-existing file sizes —
      focus on what this change contributed.)

    Point at evidence: give a file:line reference for every finding, and
    for any check you'd otherwise answer with a bare "yes."

    Your final message is the report itself: begin directly with the
    spec-compliance verdict. Every line is a verdict, a finding with
    file:line, or a check you ran — no preamble, no process narration, no
    closing summary.

    ## Output Format

    ### Spec Compliance

    - Compliant, or issues found: [what's missing/extra/misunderstood,
      with file:line references]
    - Cannot verify from diff: [requirements you could not verify from
      the diff alone, and what the controller should check — report this
      alongside the compliant/issues verdict for everything you could
      verify]

    ### Strengths
    [What's well done? Be specific.]

    ### Issues

    #### Critical (Must Fix)
    #### Important (Should Fix)
    #### Minor (Nice to Have)

    For each issue: file:line, what's wrong, why it matters, and how to
    fix it if that's not obvious.

    ### Assessment

    **Task quality:** [Approved | Needs fixes]

    **Reasoning:** [1-2 sentence technical assessment]
```

**Placeholders:**
- `[MODEL]` — REQUIRED: reviewer model per SKILL.md's "Choosing a model for each role" table
- `[BRIEF_FILE]` — REQUIRED: the task's brief file, the same one the implementer worked from
- `[GLOBAL_CONSTRAINTS]` — the binding requirements copied verbatim from the plan's constraints or the spec: exact values, formats, and the stated relationships between components (not process rules — the shared discipline block already carries those)
- `[REPORT_FILE]` — REQUIRED: the file the implementer wrote its report to
- `[BASE_SHA]` — the commit before this task started
- `[HEAD_SHA]` — the current commit
- `[DIFF_FILE]` — REQUIRED: a file holding the commit list, a stat summary, and the full diff for that range — see "Preparing the diff file, either mode" below

**Returns:** a spec-compliance verdict, strengths, findings by severity, and a task-quality assessment (Approved or Needs fixes).

## Scoped re-review (after a fix round)

Dispatched once per fix round, after an implementer has attempted to
address a prior review's findings. Verdicts each finding and inspects the
fix diff for new breakage — nothing else. This is not a fresh review; the
full review already happened.

```
Dispatch a fresh subagent:
  description: "Re-review Task N fix round R"
  model: [MODEL — REQUIRED: choose per SKILL.md's "Choosing a model for each role" table.
         Scoped re-reviews of small fix diffs usually take a
         cheap-to-mid tier.]
  prompt: |
    You are re-reviewing one task's fix round. A previous review produced
    findings; an implementer has attempted to fix them. Your job is to
    verdict each finding and inspect the fix diff — nothing else.

    ## The Task

    Read the task brief: [BRIEF_FILE]

    ## The Findings Under Verification

    [FINDINGS]

    ## The Fix

    Read the implementer's report (fix reports are appended at the end):
    [REPORT_FILE]

    **Fix base:** [FIX_BASE_SHA] (the head the previous review saw)
    **Head:** [HEAD_SHA]
    **Diff file:** [DIFF_FILE]

    Read the diff file once — it holds the fix commits, a stat summary,
    and the fix diff with surrounding context.

    [SHARED REVIEWER DISCIPLINE — insert the block from this file's
    "Shared reviewer discipline" section, verbatim]

    ## Scope

    Your scope is the findings list and the fix diff. Verdict every
    finding. Inspect the fix diff for new problems the fix itself
    introduced. Do not re-review code the fix did not touch: if you
    notice an issue entirely outside the fix diff, report it under
    Out-of-Scope Observations — it does not block this task and does not
    extend the loop. A broad whole-branch review happens once every task
    is complete.

    ## Output Format

    Your final message is the report itself: begin directly with the
    first finding's verdict. Every line is a verdict, a finding with
    file:line, or a check you ran — no preamble, no process narration.

    ### Finding Verdicts

    For each finding in The Findings Under Verification, in order:
    - **[finding one-liner]** — Addressed or Not addressed, with
      file:line evidence. Attempted is not addressed: the specific
      defect must no longer exist.

    ### New Breakage in the Fix Diff

    Anything the fix itself broke or introduced, with severity
    (Critical/Important/Minor) and file:line. "None" if clean.

    ### Out-of-Scope Observations

    Issues noticed entirely outside the fix diff. Non-blocking; the
    controller carries these into the final review. "None" if none.

    ### Verdict

    **Fix round:** [All findings addressed, no new Critical/Important
    breakage | Findings remain open] — list the open ones.
```

**Placeholders:**
- `[MODEL]` — REQUIRED: reviewer model per SKILL.md's "Choosing a model for each role" table
- `[BRIEF_FILE]` — the task brief file, the same one the implementer worked from
- `[FINDINGS]` — the Critical/Important findings and spec gaps from the previous review, copied verbatim, one per bullet
- `[REPORT_FILE]` — the implementer's report file (fix reports are appended to it)
- `[FIX_BASE_SHA]` — the head the previous review saw
- `[HEAD_SHA]` — the current commit
- `[DIFF_FILE]` — a file holding the fix commits, a stat summary, and the fix diff for that range — see "Preparing the diff file, either mode" below

**Returns:** a per-finding verdict (Addressed / Not addressed), any new breakage the fix diff introduced, out-of-scope observations, and a round verdict.

## Preparing the diff file, either mode

Neither template re-derives the diff itself — that's the controller's job, done once, outside the reviewer's own context:

```
git log --oneline BASE..HEAD
git diff --stat BASE..HEAD
git diff -U10 BASE..HEAD
```

Redirect all three into one file and hand the reviewer that path. It reads one file instead of re-deriving the range with its own git commands, and the output never has to pass through your own context either. Use the true base every time — the commit recorded before the implementer was dispatched, or the head the previous review saw for a fix round — never an assumed single-commit parent, which silently drops everything but a task's last commit when it made several.
