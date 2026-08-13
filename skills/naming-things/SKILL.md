---
name: naming-things
description: Use when choosing an identifier for a variable, function, type, module, flag, config key, table, or event; when a name is misleading, abbreviated, inconsistent with the rest of the codebase, or inherited from something it no longer describes; when weighing whether a rename is worth its cost; or when code needs a comment to explain what a thing holds or what a function actually does.
---

# Naming things

## Overview

Names are the cheapest documentation to write and the most expensive to change once they reach a wire format, a database column, or another team's code. A name is read hundreds of times at sites that have none of the context of its definition, and it is the only documentation that cannot go stale silently — it goes wrong loudly, in every reader's head.

## When to use

- Introducing any identifier that will outlive the function it appears in.
- A reviewer, a colleague, or you had to explain what something holds.
- The same concept has different names in the API, the database, and the UI.
- Deciding whether a bad name is worth a rename now.
- Not for: mechanics of executing a large rename safely (refactoring-safely), or designing the shape of an interface rather than its vocabulary (designing-apis).

## Length is proportional to scope

| Scope of use | Length | Shape |
|---|---|---|
| Loop index, 1–3 line lifetime | 1–2 characters | `i`, `k`, `v` |
| Local in a function under 20 lines | 4–12 characters | `row`, `pending`, `cutoff` |
| Parameter or field read by callers | 8–20 characters | `retryBudget`, `sourceEncoding` |
| Exported or module-level | 12–30 characters, fully spelled | `maxConcurrentUploads` |
| Wire format, column, config key, metric | As long as needed, namespaced, never abbreviated | `billing.invoice.line_item_id` |

The principle underneath: a name must be readable at its furthest point of use, not at its definition. A field called `n` is fine inside a six-line function and a defect on a public type — not as a matter of taste, but because the distant reader has none of the context the definition site has.

The inverse holds and is less often said: long names in tiny scopes are noise. `currentIterationIndexValue` makes a three-line loop harder to read, not easier. Verbosity is not clarity.

## What a name should encode

In priority order: **what it is in the domain > what role it plays here > its type > how it is implemented.**

- Type-encoding names (`strName`, `userList`, `arr`, `IFoo`, `objData`) go stale the instant the type changes, and the type is already stated by the language or shown by the tooling. The one case that earns it: genuinely unshaped data where the encoding is the only fact worth carrying, such as `raw_json_body` at a wire edge.
- Implementation-encoding names (`redisSession`, `linkedListQueue`, `s3Path`) become lies at exactly the moment you change the implementation — the moment you are least able to also update every reference. Name the role: `sessionStore`, `pendingWork`, `artifactLocation`.
- Names built from `Manager`, `Handler`, `Processor`, `Util`, `Helper`, `Data`, `Info`, `Service`, or `Wrapper` usually name nothing; they name a decision that was postponed. The test: can you say what it does *not* do. If not, the name is empty and the design behind it probably is too.

## Booleans, functions, collections

**Booleans name the true state, affirmatively.** `isEnabled`, `hasPendingWrites`, `shouldRetry` read as English at the call site. Never name the negative — `notReady`, `disableCache`, `isNotValid` produce a double negative at every use, and inverted-logic bugs cluster there. Two further rules:

- A boolean parameter is unreadable at the call site no matter how well named: a call reading `render(doc, true, false)` conveys nothing. The fix is an enum, named arguments, or two functions — not a better name.
- Three booleans that describe one thing (`isLoading`, `isError`, `isEmpty`) permit eight combinations of which half are nonsense. Name the state, not the flags.

**Functions are named for their effect if they have one, and for their return if they do not.** A command that mutates takes an imperative verb: `flushBuffer`, `revokeToken`. A query takes a noun or a `get`/`find`/`is` form: `activeSessions`, `findUserByEmail`. A query whose name promises no effect but mutates is the most damaging misnomer class there is, because callers legitimately assume they can call it twice, cache it, reorder it, or delete the call.

Prefixes carry a contract that callers rely on without reading the body. Keep them honest:

| Prefix | Promises | Broken by |
|---|---|---|
| `get`, `is`, `has` | Cheap, in-memory, no failure | Hiding I/O behind it, so callers put it in a loop |
| `fetch`, `load`, `read` | I/O, latency, can fail | Returning stale cache with no indication |
| `compute`, `calculate` | Expensive but pure and repeatable | Caching into shared state |
| `try` | Returns failure rather than raising | Raising anyway |
| `create`, `ensure` | Creates / creates-if-absent | Conflating the two, so retries fail |

A function whose honest name needs "And" does two things. Split it, or keep the honest name — but do not shorten the name to hide it. The name is diagnosing the design.

**Collections are plural and name the element**: `pendingInvoices`, not `invoiceList` or `data`. A map is named for the relation it encodes: `sessionsByUserId` beats `sessionMap`, and it removes the question of which side is the key.

## Abbreviations, jargon, and inherited misnomers

Spell it out unless the abbreviation is more recognizable than the expansion within the domain (`id`, `url`, `http`, `db`, `io`, `max`, `min`). The test is not "do I understand it" but "would a new reader and a repo-wide search both find it" — `usrMgr` fails both.

Domain jargon the business itself uses is not an abbreviation; it is the correct name. Translating a domain term into clearer English installs a permanent translation cost between the code and every conversation about the code.

**When the codebase already calls a thing by a wrong name consistently, consistency wins.** A codebase with one correct name and forty wrong ones has forty-one names for one thing and zero searchability. Exactly two options: use the existing name, or rename all forty-one in one commit. Introducing the forty-second variant "correctly" is the worst of the three and is the most common choice.

The corollary is one word per concept across code, tests, logs, docs, tickets, and speech. When the same entity is a customer in the API, a user in the database, and an account in the UI, every conversation pays a translation tax and every bug report is ambiguous. Collapsing that to one word is worth a dedicated commit.

## Renaming has a real price

| The name lives in | Cost | When to do it |
|---|---|---|
| A local or private function | Seconds, tool-assisted | Immediately, whenever it is wrong |
| A module's internal API | Minutes, plus conflicts in every open branch | With the next change to that module |
| An exported or public API | Every consumer, plus a deprecation window | Only with an alias and a version bump |
| A wire field, column, config key, metric, or log field | A migration and a compatibility period; dashboards and alerts break silently and nobody notices for weeks | Rarely, never as a drive-by |

Rename in its own commit, containing nothing else, so review and revert both stay possible.

**The reliable signal that a name is wrong: you keep having to explain it.** Every comment clarifying what a variable holds, every "actually, this is…" in review, every time the same question is asked twice — the name is doing negative work and the comment is a patch on it. Delete the comment, fix the name.

**The second signal: you cannot think of a name at all.** Five minutes of not naming something usually means the thing is two things. That is a design problem presenting as a vocabulary problem, and no amount of thesaurus work resolves it.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A comment explaining what a variable holds | The name should have said it |
| `data`, `info`, `obj`, `temp`, `result2` | Naming postponed and never returned to |
| A condition reading as a double negative | Boolean named for the false state |
| One concept, three names across layers | No shared vocabulary; each layer named it locally |
| Renamed one occurrence "correctly" | Added the Nth variant; searchability now worse than before |
| A `get` accessor that makes a network call | Prefix contract broken; callers put it in a loop |
| Rename buried inside a behavior change | Neither reviewable; the revert takes both |
| Five minutes stuck on a name | The thing is two things |

## Red flags

- "I'll name it properly later."
- "Everyone here knows what that abbreviation means."
- "It's just a temp variable."
- "I'll add a comment explaining what it holds."
- "The existing name is wrong, so I'll use the right one for mine."
