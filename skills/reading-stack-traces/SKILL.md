---
name: reading-stack-traces
description: Use when a crash, exception, panic, traceback, core dump, or error log must be interpreted, when the top frame sits inside a framework or standard library, when a trace is truncated, async, minified, inlined, or symbol-stripped, or when an error carries a wrapped cause chain or a list of aggregate errors. Covers which frame identifies the defect and what each frame does and does not prove.
---

# Reading stack traces

## Overview

The frame that threw is rarely the frame that is wrong. Read for the boundary where code you control handed a value to code you do not, because that is where the defect almost always lives.

## When to use

- A crash, exception, panic, or error log needs to be turned into a specific line of code
- The top of the trace is inside a standard library, framework, or runtime
- The trace is async, truncated, minified, stripped, or full of generated names
- An error message has a "caused by" tail, a suppressed exception, or a list of sub-errors
- The trace has been read once and produced no candidate

## Reading order

Not top to bottom. Do this instead.

1. **Read the type and the full message first**, including every "caused by" line to the very bottom. The type names the category; the message names the violated invariant. A surprising number of traces are solved here and never read further.
2. **Find the innermost frame in code you own.** This is the boundary. Everything below it is a consequence of a value you supplied.
3. **Find the outermost frame you own.** It names the request, job, or feature — the "when", which the message never carries.
4. **The defect lies between those two frames, or in the value passed at the boundary.** Frames above the boundary tell you what the callee objected to; they are the specification, not the bug.
5. **Read a cause chain bottom-up.** The last "caused by" is the original failure. The top is the outermost repackaging and is usually the least informative thing in the trace, which is exactly why it is the part people quote.

The output of reading a trace is one sentence: "*<caller>* passed *<value>* to *<callee>*, which requires *<invariant>*." If you cannot write that sentence, you have not finished reading.

## What a frame proves

| Observation | Proves | Does not prove |
|---|---|---|
| Frame present | this code was on the stack at throw time | it was reached the way you assume — check for retries and recursion |
| Line number | the build mapped that instruction to that line | the source you are reading matches the build; verify commit or build id first |
| Null dereference inside a framework frame | the framework got a value it does not permit | the framework is at fault |
| Same frames repeating | recursion or a retry loop | unbounded recursion; some depth is intentional |
| Frame absent | nothing on its own | the code did not run — it may be inlined, tail-called, or async |
| Top frame is an assert, panic, or throw helper | nothing about the defect | skip it and start at its caller |
| Thread name | which pool ran it | which code submitted it |

## Composed and non-linear errors

- **Wrapped chains:** fix the deepest cause. Teams routinely improve the wrapper's message for two releases without ever reading the bottom of the chain.
- **Aggregate errors** (parallel tasks, batch validation, fan-out): list order is arrival order, not causality. Cancellations, "context canceled", "connection closed", and "pool shutting down" are consequences. If most entries are those and one is not, the one that is not is the cause.
- **Suppressed exceptions** from cleanup or close blocks frequently carry the real failure while the primary is a knock-on of it. Always print them; many log formats drop them by default.
- **Rethrow without cause** severs the chain. That frame becomes the oldest evidence you have, and repairing it is a separate defect worth its own fix.
- **Error values rather than exceptions** (result and error-return styles): the wrap chain replaces the stack. Unwrap to the terminal error, and treat each wrap message as the line number substitute — which is why a wrap message with no context ("failed to process") is a defect in its own right.
- **Multiple threads in one dump:** the trace of the thread that crashed may not be the thread that caused it. See `debugging-concurrency`.

## Traces that lie

| Distortion | Signature | Recovery |
|---|---|---|
| Inlining | callers missing; line points inside a different function's body | reproduce on a build with optimization or inlining disabled |
| Tail calls | impossibly shallow stack; the caller is simply gone | reconstruct from logging or a non-optimized build |
| Stripped symbols | hex addresses, `??`, `<unknown>` | symbolicate against the *exact* matching build; a mismatched symbol file yields confidently wrong names |
| Minified or bundled code | one-letter names, everything on line 1 | source map from that exact build hash, not the current one |
| Proxies, interceptors, decorators, lazy loaders | your type name with a generated suffix; a thick reflective middle | ignore the reflective sandwich; the frames on either side of it are the real caller and callee |
| Generated code (parsers, schemas, macros, templates) | files you never wrote, line numbers past end of file | map back through the generator's input |
| Async, coroutines, futures | trace begins at an executor or event loop; the awaiting code is absent by construction | enable the runtime's async stack capture; otherwise carry a correlation id and reconstruct from logs |
| Cross-thread or queue handoff | producer never appears on the consumer's stack | capture the submission stack at enqueue time and attach it to the task |
| Signal handler or hard crash | the stack shown is the handler's | inspect the faulting thread's program counter and registers |
| Stack overflow | thousands of frames | the shortest repeating cycle is the defect, not the top frame |
| Truncated middle ("... 47 more") | elided common frames | the elided section is shared with the enclosing trace — read the enclosing one |

The general rule: a trace is a reconstruction produced by the runtime under whatever the compiler left behind. Optimized builds trade trace fidelity for speed, and the fidelity you lost is exactly the part you now want.

## When the trace ends inside a framework

1. Read the framework's source at the version in the lockfile, not the documentation for the current release. The gap between them is where a surprising share of these end.
2. Find the precondition on the call you made. In the overwhelming majority of cases you supplied something the contract excludes — null where absent means empty, an unsorted input, a closed resource, a wrong thread, an unregistered type.
3. Search the framework's issue tracker for the message with the variable parts removed. Exact-string search beats paraphrasing.
4. Check the trace against your own configuration: framework errors deep in initialization are usually configuration, not code.
5. If it really is a framework defect, pin the last working version (`bisecting-failures` over dependency versions) and record the reason next to the pin.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Fixed the top frame, bug persists | the top frame was the victim; the defect was at the boundary below it |
| "The framework is broken" | a contract was violated at the call site; framework bugs are rarer than the ratio of blame suggests |
| Cause chain never read | log format truncated it, or only the first line was pasted |
| Line number points at unrelated code | trace came from a different build than the source open in the editor |
| Async trace declared useless | async stack capture was never enabled, or no correlation id was propagated |
| Aggregate error debugged from entry one | entry one was the first cancellation, not the first failure |
| Symbolicated names look wrong | symbol file does not match the binary |
| Trace pasted into the ticket, nothing else | the message and the boundary frame were never extracted into a sentence |

## Red flags

- Reading only the first line of the error
- Copying a trace into a search engine before locating the boundary frame
- Concluding "null pointer" without asking which of the several values on that line was null
- Believing an absent frame proves code did not run
- Reasoning about optimized-build line numbers as if they were exact
