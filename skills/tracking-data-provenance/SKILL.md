---
name: tracking-data-provenance
description: Use when a record can be written from more than one kind of source — measured, imported, inferred, user-asserted, or backfilled — when a field encodes how strongly something is believed, when an outside recommendation could trip a threshold meant for first-party evidence, when designing an import or sync path, or when a value's origin has to survive into an audit, a model, or an automated decision. Covers separating write paths by origin, provenance fields, confidence laundering, and promotion rules.
---

# Tracking data provenance

## Overview

A row records what is believed. Provenance records **who believed it and on what basis**, and it is the part that gets dropped first, because at write time everybody knows where the data came from.

The failure is not that provenance is lost. It is that a weaker claim, once written through a path that means "we observed this", becomes indistinguishable from a stronger one — and every downstream reader treats it as the stronger one.

## When to use

- Building an import, sync, backfill, or migration that writes into tables an application also writes.
- A field encodes strength of belief: an observation count, a hit rate, a score, a confidence, a trust tier, a rating.
- A third party's data lands next to your own measurements.
- A value feeds an automated decision — a threshold, an alert, a ranking, a price, a model's training set.
- Not for: how a value moves at runtime (`tracing-data-flow`), the shape of the endpoint once you know how many you need (`designing-apis`), or migrating a live table to carry the new column (`schema-evolution`).

## Kinds of claim

These are different claims about the world. They are routinely stored in one column.

| Kind | Means | Trust decays when |
|---|---|---|
| **Observed** | We measured it directly | The world changes |
| **Reported** | A user or operator asserted it | It was never verified, so it has no trust to lose |
| **Imported** | A third party asserted it to us | Their method changes, silently |
| **Derived** | Computed from other values here | An input changes, or the formula does |
| **Inferred** | A model's output, with error | The model is replaced or drifts |
| **Defaulted** | Nobody supplied it; this is a fallback | It is mistaken for a real value |
| **Backfilled** | Reconstructed after the fact | The reconstruction's assumptions expire |

**Defaulted and backfilled are the two most often forgotten**, and both masquerade perfectly as observed data. A default that reaches storage is a fabricated measurement unless something records that nobody supplied it.

## One write path per provenance

Where fields encode how strongly something is believed, an imported value has different standing from a measured one, and reusing the entry point that means "we observed this" launders the weaker claim into the stronger.

Give the imported claim **its own operation and usually its own field.** Once the two share a column, nothing downstream can tell a third party's hypothesis from your own measurement — not with a query, not with a report, and not with an incident in progress.

The temptation is that the row shapes match. They do; that is exactly what makes it dangerous. **Separate write paths by provenance, not by row shape.** Sharing a path because the columns line up is the same class of mistake as sharing a table because two entities have the same fields.

Practical form:

- A distinct operation per source kind, named for the claim it makes rather than for the table it writes.
- A provenance field on the row: source kind, source identity, and the time the claim was made — which is not the time it was written.
- Where a value's meaning differs by source, a separate column, not a shared one with a type tag consumers are trusted to check.

## Confidence laundering

The characteristic failure has four steps, and each is individually reasonable:

1. A third party supplies a weak signal — a guess, a recommendation, a low-confidence match.
2. It is written through the path that also serves first-party measurement.
3. A downstream rule reads the field and cannot see the difference.
4. A promotion or escalation rule fires: the value crosses a threshold, gets aggregated into a score, is used to auto-approve something, or becomes a training label.

By step 4 the outside guess has the authority of your own evidence, and the chain that produced it is not reconstructable from the data.

**So the design step nobody takes is step 4's:** having separated the write paths, go read what consumes that field. An automatic promotion rule is how laundered provenance turns into authority, and it is usually in a different service, written by someone who never saw the import.

Aggregation deserves specific suspicion. A count, an average, or a score computed across mixed provenance produces a number whose meaning is undefined and whose confidence is unstated — and it looks exactly like a number computed over clean data.

## Keeping it honest over time

- **Record the claim time separately from the write time.** An import of last year's data written today is not a fact about today, and one timestamp cannot say both.
- **Preserve provenance through derivation.** A value computed from mixed sources is at best as trustworthy as its weakest input; if that cannot be carried, say so where the derived value is defined.
- **Make provenance non-nullable, with an explicit `unknown`.** A nullable field is filled in by whoever is in a hurry, and `unknown` that had to be chosen is information, while `null` is silence.
- **Do not let a re-derivation upgrade a claim.** Recomputing an inferred value from inferred inputs produces an inferred value, however many times it round-trips through storage.
- **Expect to be asked.** "Where did this number come from?" arrives during an incident, a dispute, an audit, or a regulatory question, and it is unanswerable retroactively.

## Common mistakes

| Symptom | Real cause |
|---|---|
| An outside recommendation tripped a threshold meant for first-party evidence | One write path served two provenances |
| A score cannot be explained to the person it affected | Provenance dropped at write time; the chain is unreconstructable |
| A default value is treated as a real measurement | Nothing recorded that the field was never supplied |
| A backfill made historical metrics look better | Reconstructed values written through the observed path |
| An average across sources means nothing in particular | Aggregated over mixed provenance without saying so |
| An import wrote "today" for year-old data | Claim time and write time collapsed into one column |
| A model trained on its own earlier output | Inferred values re-derived and stored as if measured |
| Import path reused because the columns matched | Separated by row shape rather than by provenance |

## Red flags

- "It's the same shape, so I'll reuse the endpoint."
- "We'll add a source column later if we need it."
- "It's just a default."
- "The data's in the table, that's what matters."
- "We can figure out where it came from if we have to."
- Any threshold, alert, or auto-approval reading a field that more than one kind of source writes.
- A confidence or score with no statement of what it was computed over.
