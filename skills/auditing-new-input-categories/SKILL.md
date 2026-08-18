---
name: auditing-new-input-categories
description: Use when extending a system to handle a new category of input — a new script or writing system, file format, payment method, protocol version, device type, currency, or a language with more plural forms than the ones already handled — rather than a new value within a category it already supports; when a phased rollout keeps turning up bugs in old shared code instead of in the new content itself; or when tempted to treat an existing green test suite as coverage for a category it has never actually run. Covers sourcing a category's well-known trouble spots from expert knowledge, auditing shared code paths before real content arrives, and why a clean run against prior categories is not evidence for the next one.
---

# Auditing new input categories

## Overview

A test suite is only ever evidence for the categories of input it has actually run. Add a category it has never seen — a new script, a new file format, a new payment rail — and every shared code path that category will flow through is unproven again, however green the suite already looks. The bugs that surface are almost never in the new content itself; they're in generic code that quietly assumed the shape of whatever categories came before it.

## When to use

- Adding a new script, writing system, writing direction, or punctuation convention to text-handling code.
- Adding a new file format, payment method, protocol version, device type, currency, or unit system to code that already handles several others.
- A phased rollout — languages, regions, integrations, formats — keeps finding real bugs in old shared code, never in the new content.
- About to treat a passing test suite as coverage for a category it has never actually exercised.
- Not for: a new *value* within a category already supported — another customer, another SKU, another user writing in a language the system already handles. Ordinary test data for that is `designing-test-data`.
- Not for: a double standing in for a system you do not control — `grounding-test-doubles` owns fixtures whose real shape lives outside your own code.
- Not for: hunting for unknown edge cases inside a category already in production, via random generation — that is `property-based-testing`. This skill is about a category's *known* trouble spots, sourced deliberately — ideally before the category goes live, but the same targeted pass is exactly what to run the moment a rollout starts surfacing shared-code bugs instead.

## A new category is not a new instance

A second Portuguese-language string is a new *instance* of a category the system already handles — ordinary content. A first Thai string is a new *category*: no whitespace between words. A first Arabic string is a different new category again: right-to-left, letterforms that change shape by position in a word. Each one exercises shared code — word wrap, truncation, search, sort, case-folding, length limits — in a way no earlier category did, no matter how many Portuguese and Spanish strings came before it.

The same split shows up outside text. A new file format is a new category for a parser. A new payment method is a new category for a ledger. A new protocol version is a new category for a compatibility layer. A language with six plural forms is a new category for anything that pluralizes a count and was only ever tested against languages with two. A system that has only ever converted metric units is a new category away from its first imperial input, and the reverse. Anywhere a system is generic *over* a set of categories, adding a member to that set is the trigger — not routine content growth.

## A clean run on prior categories is not evidence

A suite that has passed for every category tried so far says nothing about the next one — those runs only ever exercised the code paths and edge cases those particular categories happen to have. Reading a long green streak as proof the shared code is "solid" confuses solid-for-what-was-tried with solid-in-general. Each new category resets the count to zero and earns its own dedicated pass, however well-tested the system looks going in.

## Source the trouble spots from expertise, not invention

A synthetic fixture for a new category reliably misses that category's real, well-known trouble spots, because whoever writes the fixture usually isn't fluent in what actually breaks for it. A made-up test string in an unfamiliar script tests nothing a fluent reader would recognize as dangerous; the specific characters that fluent reader already knows to distrust do.

The fast way to find the real trouble spots without already being a domain expert: **ask what a fluent expert in this category would immediately flag as commonly mishandled by code written by non-experts.** That question surfaces documented, well-known failure modes directly — combining characters and grapheme clusters for text shaping, a currency with three decimal digits or none at all for money, a variable-length year or a leap second for calendars, a device with no persistent storage for a sync protocol — instead of whatever edge case happens to occur to someone unfamiliar with the domain.

No fluent expert on hand is not a dead end. The question above is common enough that someone has usually already written the answer down: a script's own standards body, a file format's spec, or a payment network's integration guide typically carries a "common mistakes" or "gotchas" section aimed at exactly this audience. Search for that existing answer before inventing trouble spots from scratch.

## Run the known trouble spots before real content finds them

1. **Identify the category's own well-known trouble spots** — cases fluent or expert sources already document, not ones guessed at. Prefer a canonical stress source for the domain over invented examples: for scripts, a real problem-character set (public compilations of exactly this exist, such as the "Big List of Naughty Strings"); for a file format, its own conformance suite; for a payment rail, the provider's documented edge cases.
2. **Run those specific examples through every shared or generic code path the new category will exercise** — deliberately, before real content does it by accident. This is a targeted pass against known trouble spots, not a volume fuzz: a handful of the right examples finds more than a large batch of arbitrary ones.
3. **Do it before the category has any real content**, whenever that's possible at all. The strongest version of this is proactive: a case-folding function's documented quirk for one language's letterforms gets caught by running that language's own real problem characters through the function ahead of a rollout — not by waiting for the first translated string to surface it by accident. The same move works outside text: a new payment provider's documented chargeback-reason-code quirks get caught by running its published sandbox test cards through the ledger before the provider goes live, not by waiting for the first real chargeback to surface them.
4. **When content arrives before an audit was possible** — an urgent partner integration, an unplanned locale request — run the same trouble-spot examples immediately once it lands, rather than treating the first live bug report as the audit. A bug a user found is a missed audit, not a substitute for one.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The audit only ran the example the requester happened to provide | The requester's own example is content they already know works, not the category's documented trouble spots |
| "The suite's green, we've shipped a dozen of these already" offered as confidence about an untried category | Green proves the categories tried; it is silent on the untried one |
| Fixture for the new category was written by whoever wrote the code, not someone fluent in it | Self-authored synthetic fixtures encode the author's own blind spots — exactly what needed catching |
| The same class of bug recurs with each new category, just with different specifics | No dedicated audit step exists; each rollout rediscovers the same shared-code gap from scratch |
| A trouble spot looked "obvious in hindsight" only after shipping | Nobody asked what a fluent expert in the category would flag before shipping |

## Red flags

- "It's basically the same as the last one we shipped" said about a category that has never actually run through the code.
- Nobody on the team can read, write, or verify the new category's content well enough to judge whether a fixture is realistic.
- "We'll find out what breaks once real users start using it."
- Treating a new-category rollout as routine content addition rather than a new code path.
- No review step for the new category beyond the pre-existing test suite.
