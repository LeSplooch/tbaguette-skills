---
name: evaluating-llm-output
description: Use when a feature's correctness depends on text, a label, a decision, or a tool call produced by a language model — when writing its first eval, when a prompt, model, retrieval step, or tool definition is about to change, when an eval dashboard shows scores nobody acts on, when one model is grading another's output, when deciding what a gate on model output should block, when a check on model output fails and then passes on rerun, when two prompts or models are being compared on a few dozen cases, or when a retrieval-backed answer is wrong and nobody knows whether retrieval or generation failed. Covers reading real outputs before choosing a metric, binary criteria over scales, code checks before model judges, measuring a judge against human labels, building the set from production, reporting a rate with its uncertainty, capability versus regression suites, and splitting retrieval failures from generation failures.
---

# Evaluating LLM output

## Overview

A feature built on a language model is not correct or incorrect. It is correct at a rate, on a distribution of inputs, and both move — the rate when a prompt, a model, or a retrieval step changes, the distribution when users do something new. An eval is the test suite for that rate. The common failure is not having no evals; it is having a dashboard of generic scores, chosen before anyone looked at what the feature actually gets wrong, that moves a little on every change and decides nothing.

So the order matters more than the tooling: look at real outputs, name the failures you see, write one pass/fail check per failure, measure the checks themselves, and report every number with the uncertainty it actually has.

## When to use

- A feature's output comes from a model and nobody can say how often it is right.
- A prompt, model identifier, retrieval step, tool definition, or output schema is about to change.
- A dashboard shows "helpfulness 4.1" or "quality 0.83" and nobody can say what to do when it moves.
- One model is grading another's output.
- A check on model output went red and then green on a rerun of the same commit.
- Two prompts or models are being compared on a few dozen examples.
- A retrieval-backed answer is wrong and the fix is being guessed at.
- Not for: wiring the call itself — pinning the model, handling refusals and truncation, budgets, telemetry (`building-llm-features`). Not for: a deterministic function with a right answer; ordinary tests do that better (`choosing-test-scope`).

## Read before you measure

Start with outputs, not metrics. Pull real inputs and the feature's responses — a hundred is a good first batch, and the right number is however many it takes before new kinds of failure stop appearing — and write a free-text note on each one that is wrong: what is wrong with it, in plain words. Then group the notes. The groups are your failure modes, and counting them tells you which to work on first.

Only now choose what to measure, and measure those failure modes. A metric picked before reading measures what someone imagined would go wrong, and a generic one — relevance, coherence, helpfulness — measures nothing a change can be aimed at. Scores like that create confidence without information: they drift by a point, nobody knows why, and the decision gets made on a feeling anyway.

Expect this to be most of the work. Teams that ship these features well report spending more time reading outputs and building checks than on the prompt itself, and the reading is the part that cannot be delegated to the model being evaluated.

## One binary check per failure mode

- **Pass or fail, with a definition someone else could apply.** "Cites a source that appears in the retrieved documents", "declines when the account is not the requester's", "the total matches the line items". A scale from one to five invites an argument about whether this one is a three or a four, needs far more examples to show a difference, and hides which failure changed.
- **Code before a model.** Whatever code can decide, code decides: the output parses, a required field is present, a quoted figure appears in the source, a forbidden string does not, the tool called was the right one with arguments in range. Deterministic checks are free, instant, and never disagree with themselves. Reach for a model judge only for what genuinely needs reading — and `routing-around-capability-gaps` has the general form of that preference.
- **Include the cases where the behavior should not happen.** A feature that should decline, abstain, ask, or leave a tool uncalled needs examples where that is the right answer; a set containing only cases where it should act rewards a feature that always acts.

## A judge is a classifier, so measure it like one

A model grading output is making a prediction, and its agreement with the person who owns quality is an unknown until you measure it.

1. Label a set of outputs by hand for the one criterion the judge will apply — pass or fail, with a sentence of reason. The labeler is whoever would be embarrassed by a wrong answer shipping, not whoever is free.
2. Split the labels: some to tune the judge's instructions against, the rest held back and never looked at while tuning.
3. Report, on the held-back set, how often the judge passes what the human passed and how often it fails what the human failed — the two separately, not one accuracy figure. When most outputs pass, a judge that passes everything scores high on accuracy and catches nothing.

Then keep it honest. Labeling changes the labeler: the criteria sharpen as examples are graded, so expect to rewrite the definition after the first batch and relabel. Judges have known leanings — toward whichever option came first in a pairwise comparison, toward longer answers, toward output from their own model family — so swap the order, and prefer a judge from a different family than the model being judged. And the judge's own model changes underneath it, so re-measure agreement on a schedule, not once.

## Build the set from what the feature actually receives

Composed examples describe the inputs someone imagined. Capture real ones — sampled from production or from the people already using the feature — and keep them, with the output and the verdict (`grounding-test-doubles` has the general case for capture over composition). Every failure found in production becomes a case, the same way a bug becomes a regression test (`regression-test-from-bug`). When users start sending a new kind of input, that is a new category rather than more of the same, and the existing pass rate says nothing about it (`auditing-new-input-categories`).

Keep the examples you tuned the prompt against apart from the ones you report on. A prompt adjusted until a set passes has learned that set; its score there is a measure of the tuning, not of the feature.

## A pass rate is an estimate

- **Run each case more than once.** Output varies between identical requests — even at a temperature of zero, since a hosted model's arithmetic depends on how the server batched your request with others. A single run is one sample.
- **Report what the user experiences.** "Passed at least once in five tries" and "passed all five times" can be far apart, and a user who sees one answer each time is living in the second number.
- **Say how uncertain it is.** Forty cases at 80% carry a standard error of about six points, so they cannot tell 80% from 85%. Compare two prompts or models on the same cases and look at the per-case differences; that pairing removes most of the noise that two separate runs would add.
- **Never rerun until green.** A check that passes on the second try has told you the failure rate is not zero; retrying hides exactly the rate you were measuring (`flaky-test-triage`).

## Two suites, two jobs

A **capability** suite holds what the feature cannot yet do reliably. Its pass rate is low on purpose, and it is where improvement is aimed. A **regression** suite holds what already works; it should sit near 100%, and a drop is a finding. A capability case that has passed consistently for a while graduates into the regression suite. Mixing them produces one number that improves when something breaks, provided something else got better at the same time.

Deterministic checks can block a merge like any test. A suite that calls a live model is slower, costs money, and varies, so run it as its own gate — on changes to the prompt, model, retrieval, or tools, compared against the last accepted baseline with a threshold rather than per-case red and green (`designing-ci-pipelines`). And write the failing case before changing the prompt to fix it (`writing-the-failing-test-first`); a prompt edit made without one is a guess you cannot tell apart from a fix.

After release, route a sample of production traffic through the same checks. Users' inputs drift, the provider's model is updated in place more often than its name changes, and a regression that only shows on this month's inputs is invisible offline (`observing-production-safely`).

## When the answer depends on retrieval, find which half failed

A retrieval-backed answer has two stages, and they fail differently. First ask whether retrieval returned the passage the answer needed — measurable against cases where you know which documents are relevant. Then ask, given what was retrieved, whether the answer stayed faithful to it. A wrong answer with the right passages is a generation failure; with the wrong passages, no change to the prompt will fix it, and weeks go into tuning the half that was working.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A score moves every week and nobody changes anything because of it | Generic metrics chosen before anyone read the outputs; none maps to a failure a change can target |
| The new prompt "won" by two points and was worse in production | Forty cases, one run each, two separate samples — the difference was inside the noise |
| The judge says 95% pass and users complain | The judge's agreement with a human was never measured, and it passes nearly everything |
| The eval set passes and the feature fails on real traffic | Composed examples, or the same set the prompt was tuned on |
| A model-graded check is red, then green on rerun | A varying measurement read as a single verdict |
| The feature acts when it should have declined | The set contained no cases where declining was the right answer |
| Weeks tuning a prompt for a retrieval-backed feature with no gain | Retrieval was returning the wrong passages; the generation half was fine |
| One number went up while a working behavior broke | Capability and regression cases averaged together |

## Red flags

- "Let's set up evals" followed by choosing metrics, before anyone has read fifty outputs.
- A one-to-five rubric with no written definition of what separates a three from a four.
- A model judge in the pipeline with no measured agreement against a human.
- "It passed" about a check on model output that ran once.
- A comparison of two prompts reported without the number of cases or the uncertainty.
- The prompt and the model changed in the same commit, and one eval run is meant to account for both.
- An eval set nobody has added a case to since launch.
