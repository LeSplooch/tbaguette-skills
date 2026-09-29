---
name: writing-agent-instructions
description: Use when writing, generating, trimming, or reviewing a file a coding agent reads as standing instructions — AGENTS.md, CLAUDE.md, GEMINI.md, a rules file, a skill's body or description, a persistent subagent prompt — when an init command has just produced one, when the file has grown past what anyone rereads, when an agent keeps breaking a rule that is written down, when deciding whether a correction should become a permanent line, or when a skill never triggers or triggers on the wrong tasks. Covers the test a line has to pass to earn its place, moving rules that must hold into tools that enforce them, the file as a map rather than an encyclopedia, why generated overviews cost more than they return, writing for a literal reader, descriptions as triggers, and testing the file against a run without it.
---

# Writing agent instructions

## Overview

An instruction file is read on every turn of every session in its scope, so each line carries a standing price — tokens, and a share of the reader's attention — paid whether or not the task needed it. And the reader obeys it. That combination is why these files so often cost more than they return: a 2026 study across several agents and models found that repository context files did not generally raise task success, raised inference cost by about a fifth, and that the content most often recommended for them — an overview of the repository — did nothing measurable. The instructions themselves were followed well. That is the problem, not the consolation: an unnecessary requirement, faithfully followed, makes every task harder.

So the job is not to tell the agent everything true about the project. It is to tell it the few things it cannot find out and would get wrong without being told, and to make everything else either discoverable or enforced.

## When to use

- Writing or revising AGENTS.md, CLAUDE.md, GEMINI.md, a rules file, a skill, or any prompt an agent loads at the start of work rather than per request.
- An init or bootstrap command has just generated one, and it is about to be committed.
- The file has grown until nobody rereads it before adding to it.
- An agent keeps breaking a rule that is plainly written down.
- A correction just happened and it is tempting to make it permanent.
- A skill never fires, or fires on tasks it was not written for.
- Not for: reading the instruction files and session history someone else left (`recovering-agent-context`). Not for: where in a long document a rule has to sit to survive truncation and compaction — `writing-durable-docs` owns ordering. Not for: vetting a third-party skill or rules file before installing it (`auditing-dependencies`). Not for: this library's own contribution path (`tending-tbaguette`).

## What earns a line

One test, applied to every line: **could the agent find this out from the repository in the time it would take to look, and is anything already enforcing it?** A line passes only if the answer to both is no.

| Earns its place | Fails the test |
|---|---|
| The commands: how to build, the fast test invocation, the one that needs a flag nobody would guess | The language, framework, and directory listing — the code states them, and a manifest states them exactly |
| Operations that are slow, costly, or destructive, and what to run instead | Style rules a formatter or linter already enforces |
| What must not be touched, and why — generated files, a vendored tree, a directory another team owns | Generic virtue: "write clean code", "be careful", "follow best practices" |
| Rationale the code cannot show — "no ORM here, because…", "this looks dead and is loaded by name" | Session history, progress notes, what was tried last week |
| Where something lives when the layout misleads | Procedures used in one kind of task, loaded into every task |

A study of instruction files in a hundred popular repositories in 2026 found the failing column everywhere: rules a linter already enforced in about three files in five, bloat in two in five, and procedures that belonged somewhere loaded on demand in a third — usually together, and usually alongside instructions contradicting each other.

History has a home, and it is not here. What happened and where work stopped goes in a handoff note beside the work (`checkpointing-long-runs`); a decision and its reasons go in a decision record (`writing-adrs`). The instruction file may point at either in one line.

## A rule that must hold goes in a tool

Prose is probabilistic. It is followed most of the time, less often late in a long session than early, and never in the one run where it mattered and the context had drifted. So every rule that *must* hold gets moved out of the file and into something that runs: a formatter, a linter rule, a test, a pre-commit hook, a permission the agent does not have. The file keeps at most the reason, in one line, so the agent understands the refusal when it meets it.

The tool then becomes the best-timed instruction there is. A linter message that says how to fix what it flagged is read at the exact moment it applies, by a reader who is looking at the offending line — no standing file can do that. Write enforcement messages as instructions: what is wrong, what to do instead, where the rule's reason lives.

## A map, not an encyclopedia

What loads on every turn should mostly be pointers. Keep the always-loaded file short — the commands, the prohibitions, the non-obvious reasons — and let it name where detail lives: a docs directory, a skill for a procedure, a nested instruction file in the subdirectory it governs. Detail then enters the context when a task needs it and at no other time. A file that grows past what its owner rereads before editing has stopped being maintained and started being appended to, and the contradictions arrive next.

One statement per rule. A rule repeated in the user-level file, the repository file, and a directory file — slightly differently each time — resolves however the reader happens to resolve it that turn. Where several harnesses need the same file under different names, make one the source and the others links or generated copies (`keeping-copies-in-sync`), never three hand-edited siblings.

## A generated file is a draft to delete from

An init command that reads the repository and writes an instruction file produces, almost by construction, the failing column of the table above: a restatement of what the code already says, now paid for on every turn. Treat its output as a list of candidates. Keep the lines that pass the test, delete the rest, and expect to keep very few.

The same goes for a correction that feels like it should become permanent. Most corrections are about the task in front of you. Promote one only when it would have been needed in a different task too, and only if no tool can enforce it instead.

## Write for a literal reader

- **Say what to do.** "Run the fast suite with `-k unit` before committing" is followable; "don't forget the tests" is a mood. Where a prohibition is the point, pair it with the thing to do instead.
- **Attach the reason at the point of the rule.** A rule with its reason generalizes to the cases the rule did not list; a bare rule is applied to its letter, including where the letter is wrong. When one skill library cut its explanations to save tokens and tested the result, compliance with its central rule under pressure fell from eight runs in ten to five, so the arguments went back in — as short rows placed where the reader meets the temptation, rather than as an essay at the top.
- **Use words the reader already knows.** A private vocabulary costs a definition every time it appears, and the definition is the line that gets dropped.
- **Do not shout.** Capitals and "CRITICAL" on every rule make none of them critical, and current models over-apply emphasized rules to cases they were never meant for. Keep emphasis for the one rule where a violation cannot be undone, and put that one first.
- **Split by sequence when order matters.** A later step visible too early invites the reader to skip to it; a procedure whose phases must not be merged is safer as separate documents loaded in turn.

## A description is a trigger, not a summary

For a skill, a tool, or anything else chosen from a listing, the description decides whether the body is ever read — and the listing is budgeted, so descriptions are truncated, and the least-used are dropped first. Write it as the situations, symptoms, and phrasings in which the thing applies, most common first. Do not summarize the procedure in it: a reader that finds the procedure in the description follows the summary and skips the body, which is how a skill with a two-stage check gets run with one stage. State what it covers in a closing clause, as nouns, so a reader scanning the listing can tell neighbours apart.

## Test it against a run without it

An instruction file is code with no compiler, so its only check is behavior. Keep a handful of real tasks from the repository — the ones agents actually get — and run them with the file and without it, or with a line and without it, reading the transcripts rather than only the outcomes. A line whose removal changes nothing goes. Rerun the same set after a model upgrade, since a newer model often needs less of what an older one was told; `evaluating-llm-output` covers measuring a difference that varies from run to run.

A rule the agent keeps breaking is a finding about the rule before it is a finding about the agent: it is ambiguous, contradicted elsewhere, buried where compaction cuts it (`writing-durable-docs`), or asking prose to do a tool's job. Adding capitals to it answers none of those.

## It runs with your privileges

An instruction file is executed, not read: whatever it says, an agent does with the permissions of whoever started it. Review changes to it as code changes — including the ones an agent proposes to its own file — and treat one arriving in someone else's pull request the way `designing-ci-pipelines` treats a pipeline definition. Never put a credential in one; it will be sent to a model provider on every turn and committed with the project (`secrets-hygiene`).

## Common mistakes

| Symptom | Real cause |
|---|---|
| Every session is slower and costlier, and no task got easier | The file restates the repository — an overview, a stack list, a directory tree — and is paid for on every turn |
| The agent follows the file's style rules and the build still fails lint | The rule lived in prose where a linter configuration should have been |
| A rule is obeyed early in a session and ignored late | Prose compliance decays over a long run; the rule needed a tool, or a place near the top |
| The file contradicts itself and the agent picks a side at random | Rules appended for years with nobody rereading; the same rule stated twice, differently |
| A skill never fires | Its description summarizes what it does instead of naming when it applies |
| A skill fires and its body is skipped | The description contained the procedure, so the summary was followed instead |
| An instruction file doubles in size after a hard session | The session's history and corrections were poured in instead of filed where they belong |
| A committed init-generated file nobody has edited since | A draft kept as the final version |
| A line everyone is afraid to delete | Nobody has ever run the tasks without it to see whether it does anything |

## Red flags

- "Let me add that to CLAUDE.md so it doesn't happen again" — said about something a tool could enforce, or about this task only.
- A generated instruction file committed without a single line removed.
- An instruction file longer than the last time anyone read it end to end.
- "IMPORTANT" or "MUST" appearing more often than once a screen.
- A rule with no reason attached, which nobody can now explain.
- A description that could be pasted into the body as its first paragraph.
- An instruction file that has never been compared against a run without it.
- A secret, a token, or an internal hostname written into a file the agent sends on every turn.
