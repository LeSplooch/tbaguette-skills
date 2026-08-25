---
name: tracing-data-flow
description: Use when a value is wrong, missing, null, empty, or unexpectedly defaulted and its origin is unclear; when asking where a field comes from, who writes it, or everything it reaches; when following a request payload, identifier, config value, or flag across modules, threads, services, or serialization boundaries; or when searching for a name returns nothing useful and the path seems to vanish.
---

# Tracing data flow

## Overview

A trace is a written artifact — a chain of file:line hops with what changed at each one. Held in memory it is re-derived every twenty minutes and is wrong by the third hop. The goal is not to understand the module; it is to enumerate every place this value is produced, altered, and read.

## When to use

- A value arrives wrong somewhere far from where it was set.
- Assessing what a field change breaks: every writer and every reader must be named.
- A name appears in two places with no visible path between them.
- Auditing where sensitive or user-supplied data reaches — a forward trace is how `handling-untrusted-input` gets an inventory of sinks, and how `redacting-sensitive-output` finds the log line nobody remembered.
- Not for: diagnosing why a system misbehaves when the value is not yet identified (`diagnosing-before-fixing`), or module-level coupling (`mapping-dependencies`).

## Anatomy of a trace

Four node types. Miss one and the trace is wrong, not incomplete.

- **Source** — where the value enters the system: a wire payload, a database read, a config or env lookup, a clock, a random draw, a literal default. Every trace terminates upward at a source; if yours does not, you stopped early.
- **Transform** — parse, normalize, cast, round, truncate, encode, redact. Every transform is a chance for the value to change meaning without changing name.
- **Sink** — where it is observed: a query, a response, a log, a metric, a branch condition.
- **Silent event** — the four below. These are what make traces wrong.

| Silent event | Looks like | Find it by |
|---|---|---|
| Copy | a second variable, a struct clone, a spread, a snapshot into another object | searching the source expression, not the name |
| Mutate | in-place edit through an alias or a shared reference | searching the container, not the field |
| Serialize | rename at a wire, column, file, or env boundary | searching the serialized form: the JSON key, column name, protobuf field, env var |
| Default | `?? x`, `or x`, `getOrElse`, a zero value, a schema or column default, an optional's fallback | searching for the default's literal value as well as the name |

Defaulting costs the most: downstream, a value never set and a value deliberately set to the default are indistinguishable. If that difference matters, make it representable at the source (optional, sentinel, presence flag). No amount of further reading recovers it — which is `tracking-data-provenance`'s case for carrying *how a value was obtained* alongside the value itself, decided at the schema rather than recovered by a trace.

## Search ladder

Run in order. Stop when the hop is found; escalate when it is not.

| Step | Search for | Catches | Misses |
|---|---|---|---|
| 1 | The exact identifier, whole word, case-sensitive | direct reads and writes in one naming regime | every rename at a boundary |
| 2 | The type, struct, schema, or table that carries it | all construction sites regardless of local name | dynamically shaped containers, maps, generic payloads |
| 3 | Assignment and mutation forms: `=`, compound assign, setters, `update`, `merge`, `patch`, `with` | the write you assumed did not exist | mutation through an alias |
| 4 | The serialized spelling at each boundary it crosses | the rename that broke step 1 | fields renamed by a mapping table |
| 5 | Construction of the enclosing object, and its defaults | values injected at build time and never assigned by name | reflection-built objects |

**The rename rule.** A value changes name at every boundary: `user_id` → `userId` → `uid` → `subject` → `sub`. Step 1 finds the segment you are already standing in and nothing beyond it. When a search comes back empty, the default explanation is a rename, not an absence — assume that first, every time.

## Trace direction

- **Backward, from the sink** when a bad value is in hand. Finds the one path that produced it. Fastest, and the right default.
- **Forward, from the source** when assessing a change. Finds every path. Slower and mandatory — one missed fork is a missed caller.

Trace backward to the source, then forward from that source once. The forward pass exists to find the second branch the backward pass never saw.

## When static reading fails

| Opacity | Signal | Cheapest answer |
|---|---|---|
| Reflection, dynamic field access, metaprogramming | the name is assembled from a string at runtime | log every key at the dispatch point, one run |
| Polymorphism with many implementers | the interface has nine implementations, one executes | log or break on the concrete type at the call site |
| Message passing, queues, events, signals | producer and consumer share no import edge | search the topic or event name, trace each half separately, join at the payload schema |
| Config-driven wiring, DI containers, plugin registries | the graph lives in config, not code | read the config as the source and search the string keys |
| Generated code | the file has a "do not edit" banner | trace the schema or template, not the output |
| Concurrency | value differs between reads with no write between | log with a thread or task identity and a sequence number |

**The 15-minute rule.** After 15 minutes or three dead ends of static reading, one log line at the sink answers in one run what another hour of reading will not settle. Instrumentation is the cheaper tool past that threshold, not a concession. Log the value, its identity, and where it came from; remove it in the same change that lands the fix. A hop you have had to instrument twice is a signal the value deserves a permanent one — `instrumenting-for-observability` covers making that call before the next incident rather than during it.

## Write the chain

```
tenant_id
  [source]    header X-Tenant            ingress.*:44    absent -> ""
  [transform] lowercased, trimmed        middleware.*:91
  [copy]      into RequestContext        context.*:12    RENAMED: tenant
  [default]   "" -> "public"             resolver.*:33   <- the bug
  [sink]      WHERE tenant = ?           repo/query.*:210
  ? background jobs build the context elsewhere - unresolved
```

Every hop carries file, line, and what changed. Unresolved branches stay in the list, marked. A trace with no open question after ten minutes is usually a trace that stopped looking rather than one that finished.

## Common mistakes

| Symptom | Real cause |
|---|---|
| "Searching found nothing" | Renamed at a boundary; never left search step 1 |
| Trace looks complete, behavior still unexplained | A silent default or an in-place mutation between two hops assumed pure |
| Re-deriving the same hops repeatedly | Chain never written down |
| Read all nine implementations | One log line would have named the one that runs |
| Traced one path, another exists | Backward trace only; never ran the forward pass |
| Confident there is exactly one writer | Never searched the type or the serialized name, only the identifier |

## Red flags

- "Nothing else could possibly write to this."
- "The framework must set it somewhere."
- "I'll read one more file and it will be clear."
- "It's obviously null because it was never set" — before checking for a default.
- Reading past 15 minutes of dead ends rather than adding one log line.
