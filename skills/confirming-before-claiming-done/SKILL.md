---
name: confirming-before-claiming-done
description: Use when about to say a fix, a feature, or a test suite is done, fixed, or passing; when a change is about to be committed, pushed, or handed off on that claim; when a subagent's or tool's own success report is about to be repeated as fact; when the only thing behind the claim is that the code looks right and nothing has actually been run; when the claim is that something is absent — not installed, not applied, not registered — and the evidence is that the one location you knew to check is untouched; when the requirement is that something survives a restart, a cold start, or a fresh checkout and the only check to hand observes the present instead; or when a green suite only checks files you own. Covers naming the command that would prove the claim and running it fresh, treating a stale or partial run as current evidence, inducing the condition a requirement names rather than accepting a proxy, proving absence by searching the target surface, and telling internal agreement from an outside check.
---

# Confirming before claiming done

## Overview

"It should work now" and "it works" are different claims. The first is free — it falls out of having made the edit and having a plausible story for why the edit fixes things. The second costs one command: run the actual check, read what it actually printed, and only then use the word "done." Being careful, understanding the bug deeply, or having been right the last ten times does not substitute for that command. It only means the claim hasn't been checked yet.

The rule survives paraphrase. "Should be passing," "looks good," "that should do it," and "I'm fairly confident this is fixed" are the same unverified claim in different clothes. If the check hasn't run since the last change, none of them are earned.

## When to use

- About to say a fix, a feature, or a test suite is done, fixed, working, or passing.
- About to commit, push, open a PR, or hand work off on the strength of that claim.
- Reporting what a subagent, a CI run, or a tool said about itself, instead of what you independently checked.
- Moving on to the next task because this one feels finished.
- The only thing behind the claim is that the diff looks right and nothing has actually been executed.
- Not for: judging whether a past decision was actually correct in hindsight (see `revalidating-decisions`).
- `calibrating-confidence` is the adjacent concern: marking your own uncertainty honestly as verified, inferred, or assumed. This skill is the concrete act that earns the verified label in the first place — running the check before the claim.

## Name the check, then run it

Before typing the claim, name the exact command whose output would prove it. No candidate command means the claim isn't checkable yet — say that, instead of skipping straight to the confident version.

Run it in full: not the fast subset, not just the file that changed, the whole thing the claim is actually about. Read the whole output, not just the line that flatters the claim — a summary can say "34 passed" while the same output shows a separate failure the summary doesn't count. Then check what the output actually says against what the sentence is about to claim: a build that compiles is not a test suite that passes, and a linter with zero complaints hasn't touched runtime behavior at all.

Only once that comparison holds does the claim get made — and it gets made with the evidence next to it, not implied. "34/34 tests pass" carries different information than "tests pass."

When the check itself is unreliable — an intermittent bug that only reproduces some fraction of the time — a single clean run doesn't carry the same weight it would for a deterministic one. That's a reason to run it enough times to get real signal, or to report status honestly as still-in-progress, not a reason to fall back to a hedged claim instead. "Should be fixed, let me know if you still see it" spends the same unearned confidence a flat "it's fixed" would; softer wording doesn't make one weak attempt add up to evidence.

## The claim and what actually proves it

| Claim | Real evidence | Doesn't count |
|---|---|---|
| Tests pass | This session's run, fresh, zero failures | A run from before the last edit; "should still pass" |
| Lint is clean | This session's run, zero warnings | Spot-checking only the file you just touched |
| Build succeeds | Fresh build, exit code checked | Lint passing; no red squiggles in the editor |
| It starts up | Started the way it will actually be started, watched past the point the framework calls the hook | It compiles and the suite is green; the same function passes when a test calls it directly |
| It is live | A signed-out, cache-busted fetch of the published URL | It renders in your authoring session; the upload exited 0 |
| It is not installed | The whole target tree searched for the artifact's own name; the running process's open files and loaded modules read | The one location you knew to check is untouched |
| The encoded form is this | One real value put through the real encoder, and the output printed | The type's declaration read, annotations and all; a grep for the spelling |
| A bug is fixed | Reproduced the original symptom on the new code, and it's gone | The diff looks like the right fix |
| A regression test guards it | Red on the old code, green on the fix, both watched | Passes once, never run against the broken version |
| It comes back after a restart | A real restart, then the start time within seconds of boot | It is running; the registration reported success |
| It works from a clean checkout | A clone into an empty directory, built there | It builds in the tree you have been working in |
| The backup is good | A restore performed from it | The backup job exited 0 and the file is the right size |
| A subagent finished the task | The diff it actually produced, read | Its own summary of what it did |
| Requirements are met | Checked line by line against the spec | The tests pass, so it must be done |

## Evidence goes stale the instant code moves

A test run is a claim about the exact code that existed the moment it ran. Change one more line afterward — even a line that "shouldn't touch" the part under test — and the run now describes a version of the code that no longer exists. "It passed ten minutes ago" and "it passes" stop being the same sentence the instant anything lands in between.

This is what makes "I fixed it" and "I confirmed the fix" different acts, not just different phrasings. Declaring something fixed the moment the edit is typed, without re-running the check against the post-edit code, fails for the identical reason a stale test run fails: the evidence on offer was gathered before the thing it's supposed to prove even existed.

## A report is not a check

A subagent reporting success, a CI badge sitting green, a teammate saying it should be fine — none of these are verification, they're claims, and repeating one as your own confirmed status launders someone else's unchecked belief into something that sounds checked. Read the diff the subagent actually produced instead of its summary of the diff. Open the CI log instead of trusting the badge. Run the command yourself instead of describing having run it. The report may well be accurate — that's a separate fact from whether it's been checked.

## The suite is not the contract

Where code has to satisfy something defined elsewhere — a wire protocol, a file format another tool reads, the shape a caller expects back, a vendor's API — a test suite can only compare the codebase against itself. It will confirm that the manifest parses, that the version matches everywhere it is written down, that the process exits 0, that the two registries agree. Every one of those can be true while the thing on the other side receives nothing it recognises, because not one of them ever consulted the document that says what it expects. The suite proves consistency, and correctness is not a fact about consistency.

The tell is mechanical, and worth running over any suite that guards an integration: **read the assertions and ask which of them would fail if the other side changed its mind.** If none would, the suite cannot fail for the reason that matters most — and its green is not weak evidence, it is evidence about a different claim. It will go on meaning "these parts agree with each other" for as long as they do, which is not the question.

Two things close that gap, and neither is a bigger suite. Exercise the real other side wherever you can: one round trip against the actual consumer settles what a hundred internal assertions cannot. Where you cannot, because there is no licence or no hardware or no account, make the test carry the external fact rather than a paraphrase of it — pin the exact shape the other side documents, and cite where that came from in the test's own name or docstring. A checked fact and a copied assumption are indistinguishable six months later, and which one it is decides whether the next person can trust it or has to re-derive it. Where the other side documents nothing at all, that is worth writing down too: say in the test that the shape is a guess, and prefer the guess whose failure is survivable.

The exit code is the most persuasive form this takes, and deserves naming on its own. A process that ran and returned 0 has told you it did not crash. It has said nothing about whether anything on the other end read what it wrote, and a consumer that silently drops a shape it does not recognise is the ordinary case rather than the exotic one. **An integration that runs is not an integration that arrives.**

## The push is not the reach

Where a check ran is part of what it proves. Confirming a published thing from the seat that published it — the authoring session, the signed-in browser, the tool that did the upload — establishes that the artifact exists and that you, specifically, can reach it; the claim is that its audience can. The two contexts differ along axes invisible from inside the authoring one, authentication and caching chief among them, and the two failures they produce run in opposite directions.

A default-private artifact is indistinguishable from a public one at the owner's seat, so a page that renders perfectly for its author returns nothing to anyone else — worse once the URL has already gone out as public. A stale cache is the mirror image: it serves the previous version after a genuinely successful deploy, so a correct check reads as a failure and invites a pointless re-push. One makes a broken thing look fine, the other makes a fine thing look broken, and a signed-out, cache-busted fetch from outside the publishing tool settles both.

## The run is not the return

Some requirements are not about the present at all. *Comes back after a restart. Survives a cold cache. Works from a fresh clone. Persists across a session. Recovers when the network drops.* Each of those is a claim about a condition that has not occurred yet — and the check that is conveniently available almost always measures the present instead: it is running, it answers, it is registered to start, the cache has entries in it.

Those checks pass. They would also pass, identically, in the world where the requirement is entirely unmet. That is the whole problem, and it is invisible from inside the check, because nothing about a green result advertises which of the two worlds produced it.

So put the check to a specific question before trusting it: **would this still pass if the condition the requirement names had never once occurred?** If the answer is yes, what you are holding is a proxy, and no amount of re-running it converts it into proof. A service started by hand reports itself running and answers on its port exactly as convincingly as one the machine brought up by itself, for as long as nobody restarts the machine.

The fix is not another check of the same kind. It is to induce the condition once — to actually make the thing happen, deliberately, in a window you choose: reboot the host, delete the cache, clone into an empty directory, kill the process, pull the network. That costs a real disruption, which is why it keeps being deferred — but the condition arrives on its own schedule eventually, and that schedule is reliably worse than the one you would have picked.

Which makes inducing it a decision that is not yours alone to take. On anything shared, live, or depended on by someone else, the disruption is the whole cost of the proof, so it needs the owner's go-ahead and a window agreed with them rather than the next convenient gap — and `deciding-reversibility` is the call to make first, because a restart you cannot undo from where you are sitting is a different act from one you can. Being unable to get that go-ahead is not a reason to fall back on the proxy and call it proven. It is a reason to say plainly that the requirement is configured but unverified, and to name the one test that would settle it.

Two things read the result more honestly than any status string. **Timestamps say who caused something; a status string only says that it is so.** A service's start time sitting seconds after the host's boot time means the machine started it; the same service reporting itself healthy with a start time hours after boot means a person did, and the boot path has never been exercised at all. And **a restart counter separates coming up cleanly from being caught and retried** — a supervisor configured to restart on failure makes a binary that crashes on every start look permanently healthy from outside, so the count of zero is part of the evidence, not a detail.

Away from services the shape is the same one every time: something was configured to happen later, and the configuring got recorded as the happening. A setting that says a thing will occur is a plan, and the only evidence it was a correct plan is the thing having occurred once. A backup is not a backup until a restore has been performed from it — the job exiting 0 attests to a file being written, which is a different claim entirely.

## A look is not a search

Not every claim is that something is there. *It is not installed. The patch never landed. That hook is not registered. The migration never ran.* A check can be fresh, first-hand, complete, and run from exactly the right seat and still answer a narrower question than the one asked — because the obvious way to check an absence is to open the one place the thing would live, and finding that place untouched establishes one fact about one location. The claim being made is about the whole target.

The failure is dangerous precisely because it feels like rigor: a real command, a real file, evidence gathered first-hand a minute ago, and a wrong answer at the end of it. What keeps the wrong location plausible is that it genuinely was the right one at some point — the thing being looked for moved, and the old location stayed behind, present and untouched, because nothing writes there any more. Installers relocate their hook between major versions; a patch targets a path that has since been renamed; a migration records itself in a ledger the current version no longer writes. The check has not become unreliable, which would at least be visible. It returns the same clean answer whether the thing is installed or not — the proxy problem one section up, moved from the moment a check runs to the place it points at.

A negative claim needs a different move: **search the target surface for the artifact itself, rather than interrogating the one location you can name.** Sweep the whole target tree for the artifact's own name, which is cheap and is what finds a relocated install. Ask the running process what it actually has open and loaded, the only one of these that separates present-on-disk from actually in effect. Read which candidate the runtime's resolution order really selects, rather than the path you remember it using. A location you know about settles the question the moment the thing turns up in it and can never settle it the other way, so an absence is only ever earned by a search — and the claim should name what was searched, because that is the part a reader can check.

## The call site is not the context

The usual complaint about a green build is that nothing actually ran. This failure survives the fix for that, because a test that runs the code does not necessarily run it where the code will run. Anything placed inside a hook a framework calls — a setup or init function, a registration callback, a plugin entry point, an installer's post-install step — is written and compiled as ordinary code and does not execute under the conditions the rest of the program executes under. It runs at the moment and in the surroundings the framework picked: possibly before the async runtime is up, before there is a window to draw into, on a thread that does not own the thing the line touches, or before configuration the identical line would have found in the program's entry point has been read.

Nothing at the call site carries any of that. The hook's signature looks like every other function's, so no static check can see the difference — and neither can a test that calls the hook directly, because calling it supplies the test's surroundings instead of the framework's. That is what makes this failure quiet: the suite is not wrong about what it measured, it measured a context the program will never be in. Compile, typecheck and every last test can be green on something that fails on every single start.

There is a tell, and it costs nothing to look for. **If the framework exports its own version of the thing you were about to call — its own spawn, its own timer, its own way onto the main thread, its own handle to the running application — that wrapper exists because the general-purpose one does not work here.** Reaching past that wrapper is the shape of this bug, and the only check that observes the framework's context is the expensive one it was tempting to skip: start the program the way it will actually be started, and watch it get past the point the hook runs.

## The declaration is not the payload

Some questions are about a thing the system produces rather than about whether it works: what key this ends up under, what that enum looks like once it is serialized, what is actually in the column, what the header says. The reflex is to read the code that produces it, and for this class of question the code is the wrong place to look. An attribute or annotation renames and re-cases fields, a custom encoder overrides the declaration entirely, an inherited default supplies a policy written nowhere near the type, and a library changes its own default between releases. The declaration is a request. The bytes are the answer.

So produce one payload and read it. Serialize a single real value and print it; read one stored row; dump the header off one real request. That is usually one command; it costs less than the reading it replaces; and it settles the question outright instead of raising confidence in a guess. **Reading the code is a hypothesis; producing the payload is evidence.**

There is a second reason, and it is the sharper one. A grep, a regex, or a throwaway script written to answer the question is code that has existed for a minute and has never been tested, and it can be wrong in the direction that costs the most: reporting the absence of what is there. `diagnosing-before-fixing` covers separating a broken instrument from a refuted hypothesis once a result is in hand, and its discriminator is that a broken apparatus usually fails in several ways at once. A pattern that is quietly too narrow fails in exactly one way: it returns nothing, cleanly, and nothing about that looks like a malfunction. So the cheaper move is not to build the checker at all — and where one has already run, treat a negative result from it as unconfirmed until something independent agrees.

## A neighbouring number is not a baseline

A claim that something *helped* — the cache, the index, the retry policy, the
compression step, the tuned parameter — is a comparison against a world where
it was not applied. That world does not exist in the run being measured, so its
number has to be constructed. It is never lying around waiting to be read.

What is lying around is a different number, produced by the same treated run,
about roughly the same thing, and sitting close enough to the result to look
like the other half of a ratio. Divide by it and the arithmetic works, the
units cancel, and the figure that comes out is a confident measurement of
nothing. The direction it fails in is the expensive one: it reports "no effect"
for an intervention that is working, and the reasonable response to no effect
is to remove the intervention.

One question settles it, and it costs nothing to ask before writing the ratio:
**if the intervention were switched off, would this denominator change?** If it
would not, it is not a baseline — it is a second reading off the treated run.
Construct the real one instead: run the workload with the intervention disabled,
replay the same inputs through the previous version, or derive analytically what
the untreated result would have been. `performance-profiling`'s interleaved A/B
runs are the same discipline applied to timings, and for the same reason.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Should pass now" in a commit message or handoff note | The verification step was replaced with confidence in the fix |
| Tests declared passing based on a run from before the last edit | Evidence treated as durable when it's only valid for the code it ran against |
| "The agent said it completed the task" reported as the task being complete | A tool's self-report repeated as independently checked fact |
| A regression test added and trusted without ever seeing it fail | Never run against the broken code, so it's unknown whether it tests anything |
| Build green, shipped, runtime error in the first minute | Compilation was checked; behavior never was |
| "Looks right" standing in for "ran and confirmed" | Review of your own diff mistaken for verification of its behavior |
| One failing test out of many waved off as unrelated | A partial pass rate treated as a pass |
| Everything reports healthy for months, then nothing comes back after a power cut | Liveness checked repeatedly; durability never once induced |
| "It is set to start automatically" offered as evidence that it starts automatically | Configuration read as behavior, with nothing having exercised it |
| An improvement measured at "no change" and abandoned, then found later to have worked | The denominator of the ratio was another quantity from the treated run, so switching the intervention off would not have moved it |

## Red flags

- "Should," "probably," "looks like it," or "I'm fairly confident" appearing anywhere near a completion claim.
- Satisfaction expressed — "great," "done," "that's it" — before the check has run in this message.
- About to commit, push, or open a PR, and the last thing that ran was the edit, not a check.
- A claim that would change if reworded, because the wording was doing the work the evidence should be doing.
- "It's probably fine, I'm confident in the fix" as a reason to skip the command that would confirm it.
- Tired, near the end of a long task, and tempted to call it done to stop working rather than because it's verified.
- A requirement worded with "after," "across," "survives," or "from scratch," answered by a command that only observes right now.
- "It's enabled, so it'll come back" — enabling is the plan; coming back is the thing being claimed.
- Reluctance to induce the condition because it would be disruptive, on a system where the condition will occur anyway, unattended.
- A before/after ratio whose two terms were both produced by the "after" run.
