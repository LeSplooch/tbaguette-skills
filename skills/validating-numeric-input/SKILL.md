---
name: validating-numeric-input
description: Use when a number crosses a trust boundary — a form field, query parameter, JSON body, config value, CSV cell, sensor reading, or another service's response — when a range check passed and something broke far downstream, when clamping or bounding a value, when money or quantities are involved, or when a numeric parse silently produced zero. Covers NaN and infinity defeating comparisons, overflow and precision loss, locale and format ambiguity, and parsing a number rather than testing a relation.
---

# Validating numeric input

## Overview

A guard written as a comparison is not a parse, and it is not total over its own input domain. `value < MINIMUM` asks a question about a number; it silently answers "no" for inputs that are not numbers at all.

Parsing a number means asserting that it **is** one — finite first, then in range — rather than testing a relation and inferring the rest from the test not having fired.

## When to use

- Any external number: form, query string, JSON, header, config, CSV, spreadsheet, sensor, upstream API.
- Before clamping, bounding, summing, indexing, allocating, or scaling by a value.
- A value passed every check and surfaced as an exception somewhere unrelated.
- Anything involving money, quantities, durations, or identifiers that look numeric.
- Not for: the general parse-don't-validate argument and the injection family (`handling-untrusted-input`), or how to represent the resulting failure (`modeling-errors`).

## The order that works

1. **Assert it is a number of the expected kind** — integer versus real, and reject the string forms you did not intend to accept.
2. **Assert it is finite.** This is the step everyone skips, and it is the one that makes every later step meaningful.
3. **Assert the range**, now that comparison means something.
4. **Convert to the domain type** — cents, a duration, a bounded quantity — and let the rest of the program hold that type rather than a raw number.

Steps 2 and 3 are not interchangeable. Range-then-finite is the same bug in a different order.

## Values that defeat comparison

| Value | Why the guard misses it | What it does downstream |
|---|---|---|
| `NaN` | **Every** relation involving it is false, so `< MIN` and `> MAX` both decline to reject it | Propagates through arithmetic; a clamp built from min/max passes it straight through |
| `+Infinity` / `-Infinity` | Passes a lower bound, fails nothing if only one side is checked | Overflows conversions; becomes a huge allocation or an invalid duration |
| `-0` | Compares equal to `0`, so a zero check accepts it | Reappears in formatting, division sign, and serialization round-trips |
| Very large integers | Within range as a float, not representable exactly | Silently rounds; two distinct identifiers become one |
| Subnormals / tiny values | Pass a `> 0` check | Underflow to zero after one operation; division blows up |

NaN is the headline case because it is the one that makes a *pair* of checks fail together. A clamp assembled from a minimum and a maximum looks exhaustive — every value is below, above, or between — and NaN is in none of those three, so it takes the path meant for valid input.

The value then travels layers past its own validation and surfaces as an exception with no visible connection to where it came from, which is why this is expensive to diagnose rather than merely wrong.

**Where NaN comes from** is the reason a "we never produce NaN" argument fails: `0/0`, `∞−∞`, a failed string-to-float conversion that returns NaN instead of raising, a missing JSON key coerced to a number, an average over an empty set, or a sentinel written by an upstream that means "no reading".

## Integers have their own failures

- **Overflow and wraparound.** In a language that wraps, a bounds check performed *after* the arithmetic tests a value that has already been corrupted; the check must come before, on the operands. In a language that promotes to arbitrary precision, the failure moves to memory instead.
- **Division and modulo by zero**, and the asymmetric case where the most negative value divided by −1 overflows.
- **Precision loss at a boundary between systems.** A 64-bit integer identifier crossing a JSON boundary into a consumer whose number type holds 53 bits arrives *close to* correct — the worst failure mode, because it is not detectably wrong at the point of arrival. Send large identifiers as strings.
- **Signedness.** A length or count read as signed and passed to something expecting unsigned turns a negative into an enormous positive.

## Format and locale ambiguity

The bytes on the wire underdetermine the value more often than expected:

| Input | Reads as | Trap |
|---|---|---|
| `1,5` | 1.5 or 15 or 1 | Decimal comma versus thickness separator, by locale |
| `1.000` | 1000 or 1.0 | Same, inverted |
| `010` | 10 or 8 | Leading-zero octal in some parsers |
| `0x1F`, `1e3`, `Infinity`, `NaN` | Accepted by many float parsers | Literal strings that parse successfully |
| `١٢٣` | 123 | Non-ASCII digits, accepted by some locale-aware parsers |
| `" 42 "`, `"42abc"` | 42 | Lenient parsers that stop at the first bad character |
| `""` | 0 | Empty coerced to zero rather than rejected |

The last two are the ones that produce a wrong answer rather than an error. **A parser that stops at the first invalid character turns `"12abc"` into `12`**, and a parser that maps empty to zero turns a missing field into a real quantity. Choose strict parsing explicitly; the lenient one is usually the default.

Timestamps and durations arrive as numbers and carry an unstated unit — seconds, milliseconds, or microseconds — and a value that is plausible in one is wildly wrong in another. Range-check against the unit you expect rather than against a magnitude, since a millisecond timestamp read as seconds lands far in the future and passes any bound wide enough to be useful.

Never parse money as a float. Take it as an integer of minor units or a decimal type, reject anything else, and keep the currency next to the amount rather than implied.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A number passed every range check and broke something far downstream | The checks were relations, and NaN makes every relation false |
| A clamp returned a value outside its own bounds | Clamp built from min/max, which passes NaN through unchanged |
| Two distinct identifiers collided | Large integer crossed a boundary into a 53-bit number type |
| A missing field became a legitimate `0` | Empty string coerced instead of rejected |
| `"12abc"` was accepted as 12 | Lenient parse; no assertion that the whole input was consumed |
| A total came out cents short, sometimes | Money held as a binary float |
| A bounds check passed and the arithmetic still overflowed | Checked the result rather than the operands |
| A European user's `1,5` became 15 | Locale-dependent parse on an interchange format |

## Red flags

- "It's just an integer from the form."
- "I already check the range."
- "We never generate NaN" — about a value that arrives from outside.
- "The parser returns 0 if it fails, which is a safe default."
- Clamping without a finiteness assertion first.
- Storing money, or any exact quantity, in a floating-point type.
- A numeric parse whose failure path and whose zero path are the same path.
