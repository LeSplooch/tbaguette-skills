---
name: handling-untrusted-input
description: Use when code accepts or parses data from outside its own trust boundary — requests, files, uploads, webhooks, message queues, config, or another internal service — when building a query, command, path, URL, template, or markup that embeds a variable, when reviewing a sanitize or escape helper, or when handling deserialization, path traversal, injection, XSS, SSRF, XXE, unicode normalization, or parser resource exhaustion.
---

# Handling Untrusted Input

## Overview

Untrusted means "did not originate inside this trust boundary" — which includes your own database, your own config, and the service your own team wrote. Validity is established once, at the boundary, by constructing a type that cannot hold an invalid value; it is not a check repeated inward and eventually forgotten.

## When to use

- Parsing anything from a request, file, upload, header, filename, queue message, environment, or config.
- Assembling a query, shell command, filesystem path, URL, template, or markup that contains a variable.
- Reviewing code where a function is named `sanitize`, `clean`, `escape`, or `isValid`, or accepting nested structures, archives, images, or any format with a third-party parser behind it.
- Not for: deciding what privilege the parsing process runs with (`least-privilege-design`); deciding whether the boundary should exist (`threat-modeling`).

## What is untrusted (the surprising entries)

Your database — earlier writes passed an older, buggier validator, which is the entire mechanism of stored XSS. Another internal service — its identity is authenticated, its *caller's intent* is not. Config and environment, because deploy templates interpolate. Queue payloads. Filenames, headers, cookies, and declared content types. Data you produced yourself that round-tripped through a client. Error strings from downstream systems, the moment they are logged or reflected.

The trusted set is short and enumerable: values this process constructed, from constants, since the last boundary crossing.

## Parse, don't validate

At the boundary, convert bytes into a value whose type makes invalid states unrepresentable, then pass the typed value inward. Never pass the raw string onward next to a boolean asserting it was checked.

```
BAD:   if is_valid_email(s) { send(s) }        // s is still a string; the next call site forgets
GOOD:  let addr = Email::parse(s)?             // Email cannot exist unless parsing succeeded
       send(addr)                              // no re-check possible, none needed
```

- One constructor. If any other code path can build the type, the guarantee is decorative — private constructors, smart-constructor modules, or a package boundary are how this is enforced.
- Realizations by paradigm: newtypes and refinement types (Rust, Haskell, branded types in TypeScript); value objects with private constructors (Java, C#, Ruby); a struct plus constructor discipline and no exported zero value (Go); a class validating in its initializer with frozen fields (Python); a struct with a `parse_` function returning a status and never a bare pointer (C).
- The typed value carries the *parsed* form, not the original text — keeping the raw alongside guarantees someone uses the raw. Parse at the outermost layer that understands the format; re-validating deeper in means the type did not carry the guarantee, so fix the type rather than adding the check.
- **A guard written as a comparison is not a parse, and it is not total over its own input domain.** Every relation involving a floating-point NaN evaluates false, so `value < MINIMUM` and `value <= 0` both decline to reject it, and a clamp assembled from min and max propagates it instead of bounding it. The value then travels layers past its own validation and surfaces as an exception with no visible connection to where it came from. Parsing a number means asserting that it *is* one — finite first, then in range — rather than testing a relation and inferring the rest from it not having fired.

## Injection is one bug with many names

Never build a string in a language you do not control out of data you do not control. Each destination has a mechanism that keeps data and code separate; use it instead of escaping.

| Destination | The concatenation form | The separating mechanism |
|---|---|---|
| Database query | query text built by concatenation | parameterized/prepared statements; identifiers cannot be parameterized, so map a caller token through an allowlist to a literal |
| OS command | a shell string | exec with an argument vector, no shell; if a shell is unavoidable the argument must come from an allowlist |
| Filesystem path | base + user segment | resolve to absolute, then assert the *resolved* value is under the base |
| Markup / DOM | interpolation into HTML | a template engine with contextual auto-escaping; text-content APIs, never raw-HTML APIs |
| URL | string joining | a URL builder that encodes each component in its own grammar |
| Document query languages | filter built from strings, or a caller-supplied object | a typed query builder; assert scalar values are scalars, which is what operator injection exploits |
| Logs, headers, email | concatenated fields and format strings | structured logging with fields, and library APIs that reject control characters — otherwise one newline forges a log entry or splits a header |
| Templates / expression languages | user data used as template **source** | user data may only ever be template **data**; the other way is remote code execution |

Escaping is the fallback, not the plan: it fails at nesting, where a URL inside an HTML attribute inside a script string needs three encodings applied in the right order. When an API forces string assembly, wrap it once and allowlist inside the wrapper.

## Canonicalize, then check, then use the canonical value

Check-then-canonicalize is exploitable because the transformation happens after the decision. The order is **decode → normalize → check → use**, and the value used must be the canonical one. Checking the canonical form and then acting on the original builds the bug twice.

- Decode exactly once, then reject any input that still contains encoded sequences. Loop-until-stable decoding is itself a vulnerability, and single-decode filters are what double-encoding is for.
- Paths: resolve to absolute *and* resolve symlinks, then prefix-compare including the separator — `/base` matches `/base-evil`, `/base/` does not.
- Hosts and URLs: parse with a real URL parser and compare the parsed host. Regex or prefix matching on a URL string is the standard SSRF allowlist bypass. Re-resolve at connect time or bind to the resolved address, since DNS answers change between check and use.
- Unicode: normalize (NFC generally; NFKC where confusables decide identity, as with usernames and hostnames) before uniqueness checks and before any authorization comparison, and case-fold afterwards, locale-independently. Comparisons that decide authorization run on canonical bytes — and constant-time whenever either side is a secret.

## Allowlists, and limits on every parse

A denylist enumerates what you thought of; the dangerous set is unbounded and grows with every downstream library update. An allowlist enumerates what you support, which is finite and known. Allowlist the *value* wherever the value space is finite: column names, sort directions, redirect targets, template names, locales, file types. Use a denylist only where the accepted set is genuinely open — free prose — and there the real defense is output encoding, not filtering. If the allowlist must be a pattern, anchor both ends and bound the length.

Every parser needs four numbers chosen deliberately, because every default is "unbounded":

| Limit | Starting point |
|---|---|
| Body size | 1 MB for API bodies; uploads on a separate path with their own cap |
| Nesting depth | ≤ 32 for JSON/XML/YAML-family structures |
| Element count | ≤ 10k entries; also cap archive members, multipart parts, headers, and page size |
| Time | a parse deadline ≤ 1s enforced by the caller, plus a 1–4 KB input bound on any backtracking pattern engine |
| Decompression | absolute output cap plus a ratio cap ≤ 100:1 |

Enforce at the streaming layer, before allocation — checking length after reading the body is not a limit. Disable DTD and external entity resolution in every XML-family parser; use the schema-restricted loader in every YAML parser.

## Output encoding is a separate obligation

Input validation cannot know the destination. Validate on the way in for *business* rules (is this a plausible quantity); encode on the way out for *interpretation* rules (how will this be read). One HTML document holds at least five contexts — element text, attribute value, URL inside an attribute, script string literal, style value — each with a different encoding.

Store raw, encode late. Encoding on input leaves the store holding a value that is wrong for every context except one and double-escaped in that one. The exception, stated as a predicate: store the *canonical parsed* form (a normalized phone number, a resolved path) — canonicalization is not encoding.

## Deserialization

Deserializing attacker-controlled bytes into arbitrary object graphs is remote code execution in every language that supports it; the gadget chain lives in your dependencies, so "we have no vulnerable class" is a statement about today's lockfile. Rule: untrusted bytes get a data-only format parsed into a schema you declared, and **no type information in the payload ever selects a type in your process**. Hazard set across ecosystems: native binary serializers, YAML loaders that construct arbitrary objects, JSON libraries with polymorphic type handling enabled, and any format with a type tag. Where a schema still permits subtypes, the permitted set is an allowlist you must write out. Signing changes who can deliver the payload, not what the parser does with it — and the signature is verified before parsing, never after.

## Where validation belongs across layers

Client-side validation is a UX feature with zero security value. A gateway or WAF is a coarse filter that any direct caller bypasses and that cannot know your business rules — never the only check. The real home is the outermost layer of the service that owns the invariant, expressed as a type flowing inward. Database constraints are the last line and the only one that also covers migrations, scripts, and the ops console — keep uniqueness, foreign keys, and check constraints there even when the application also enforces them. Duplicate checks are fine when each layer owns a *different* invariant; duplicated checks of the *same* invariant drift, and the weaker one silently becomes the effective one.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A `sanitize()` helper called at many sites, missing at one | The validator returned the same type it took, so nothing forces the call |
| Escaping added field by field after each report, and close variants of a blocked payload still work | Each injection treated as its own bug rather than as string assembly of a foreign grammar; the filter is a denylist |
| Path check passes, files outside the base are still read | Check ran before resolution; `..` and symlinks resolve afterwards |
| Stored values arrive already escaped | Encoding applied on input; the store now holds one context's encoding forever |
| One request exhausts the service | A parser with no size, depth, count, or time bound — every such default is unbounded |
| Two accounts collide, or authorization compares unequal-but-equivalent strings | Comparison ran before unicode normalization |
| A number passed every range check and still broke something far downstream | The checks were relations, and NaN makes every relation false; the guard was never total |
| A signed blob is fed to the full deserializer | Signature proves origin, not that the origin is honest or the key uncompromised |

## Red flags

- "It was already validated upstream" or "it came from our own database," with no line of code named.
- Reaching for an escape function when a parameterized API exists.
- Writing a pattern to *detect an attack* instead of to *define acceptance*, or adding one more case to a denylist after a report.
- Any string concatenation whose result is parsed by something else.
- A validator whose return type is a boolean.
- Bounding a float with a minimum and a maximum without first asserting that it is finite.
- Deciding to normalize after the check because the check is cheap.
