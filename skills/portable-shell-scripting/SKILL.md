---
name: portable-shell-scripting
description: Use when writing or reviewing a shell script, when a script works in one shell but fails in another, when it breaks on filenames with spaces or newlines, when a failing command does not stop the script, when a `cd` fails or persists and later relative paths resolve against a directory nobody meant, or when a script kills a process by matching its command line or loops waiting for one to disappear. Also use when a long-running command that changes state is about to be piped into `head`, `tail`, a pager, or any reader that stops early. Covers quoting and word splitting, set -e exemptions, lost variable assignments after a pipeline, exit codes and pipeline status, the working directory as state that outlives one command, relative reads and writes that land in the wrong tree, cleanup traps, matching processes by pattern for kills and wait loops, POSIX sh versus bash-isms, GNU versus BSD tool differences, and when a script has outgrown shell.
---

# Portable shell scripting

## Overview

Shell expands text and then splits and globs the result; quoting is not politeness, it is the mechanism that turns that off. `set -e` is a seatbelt with documented holes rather than error handling, and the shell you tested in is rarely the shell that will run the script.

## When to use

- Writing a script that runs somewhere you do not control: a CI runner, a container image, a BSD or Alpine host, a colleague's laptop
- A script fails only in CI, only on macOS, or only for one person
- Symptoms: `[: too many arguments`, `unbound variable`, `bad substitution`, a loop whose variables are unchanged afterwards, `rm` treating a filename as an option
- Auditing a script that outgrew its one-off origin

Not for: deciding whether the automation should exist at all (`automating-repetition`), or pinning the tools it invokes (`reproducible-environments`).

## Quoting, splitting, globbing

An unquoted expansion is split on `IFS` (space, tab, newline) and every resulting field is then glob-expanded against the filesystem. Quote every expansion and treat an unquoted one as a deliberate, commented request for splitting.

| Form | Behavior |
|---|---|
| `"$@"` | Each argument stays one word; an empty list expands to nothing. The only correct argument forwarding. |
| `$@`, `$*` | Split and globbed. `"$*"` joins into one word with the first `IFS` character. |
| `x=$y`, `case $y in` | The two contexts that never split. Quoting is optional there, and harmless. |
| `[ $x = y ]` | A syntax error when `$x` is empty or contains a space. Write `[ "$x" = y ]`. |
| `for f in *.log` | POSIX leaves the literal `*.log` when nothing matches; guard with `[ -e "$f" ] \|\| continue`. |

`set -u` turns a typo'd variable name into an error instead of an empty string; pair it with `${1:-}` for genuinely optional parameters.

## Know which shell is actually running

`#!/bin/sh` is dash on Debian-family systems, busybox ash on Alpine, and bash in POSIX mode elsewhere. Writing bash while declaring `sh` is the most common portability failure in existence, and it passes on the author's machine every single time.

Not POSIX: arrays, `[[ ]]`, `+=`, `$'...'`, `${x^^}`, `<(...)`, `set -o pipefail`, `echo -n` and `echo -e`. Use `printf '%s\n'` for anything but a literal flagless string. `local` is universal in practice and still not standardized.

Commit in the shebang and hold to it: `#!/usr/bin/env bash` with bash features, or `#!/bin/sh` with the POSIX subset only. Verify by running the script under `dash` or `busybox sh` in CI and by linting with the dialect set explicitly — never by reading. `$SHELL` is the user's login shell and says nothing about the interpreter you are inside; check `$BASH_VERSION` / `$ZSH_VERSION` if code must branch. Interactive shells such as fish, zsh, and nushell are not sh-compatible and belong in nobody's shebang.

## `set -e` and its documented holes

`set -e` exits on an unchecked non-zero status, with exemptions that are surprising and are the reason scripts silently continue after a failure:

- Any command in a condition context: `if cmd`, `while cmd`, `! cmd`, and every element but the last of an `&&` / `||` chain.
- A function invoked from a condition context runs with errexit disabled **for its entire body**, recursively. This is the hole that hides real failures.
- Only a pipeline's last command counts, so `false | true` succeeds. `set -o pipefail` fixes it in bash/ksh/zsh; POSIX sh has no equivalent.
- `x=$(false)` exits, but `local x=$(false)` and `export x=$(false)` do not — the status reported is `local`'s. Declare on one line, assign on the next.
- `((i++))` returns 1 when the result is zero, so `set -e` kills the script on an ordinary counter increment.

Run with `set -eu`, plus `pipefail` where the shebang permits it, and still check explicitly anywhere you want a diagnostic: `cmd || { printf '%s\n' "what failed, what to do" >&2; exit 1; }`. A script whose entire error handling is line 2 has none.

## A pipeline's reader can kill its writer

`cmd | head -20` reads as a way to shorten *output*. It is also a way to shorten `cmd`. When `head` has taken its twenty lines it exits, the pipe's read end closes, and the next write from `cmd` raises `SIGPIPE` — which by default terminates it. The same happens with `sed 3q`, a pager the reader quits, a `grep -q` that has found its match, and any consumer that stops early by design.

For a command that only prints, that is the intended behaviour and costs nothing. The damage is that **the shell draws no distinction between a command that prints and a command that acts**, so the identical idiom applied to something that mutates state truncates the *work*, wherever it had got to. A tool that mutates in stages and writes its bookkeeping at the end can be killed in between, leaving a result its own cleanup and resume paths do not recognise — the recovery command reports nothing to recover while the wreckage sits in plain sight, reading as ordinary mess. The class is wide: a package manager mid-transaction, an archiver mid-extract, a formatter rewriting in place, a version-control operation that stages many paths and then writes one reference.

How visible this is depends entirely on settings made elsewhere in the script, which is why it is worth knowing rather than guessing. A bare pipeline reports only its **last** stage, so the status is the reader's `0` and nothing anywhere says the writer died. With `pipefail` the pipeline reports `141` — `128 + SIGPIPE` — and in shells that have `PIPESTATUS` / `pipestatus` the per-stage array names which stage it was. So the failure is detectable exactly where the `set -eu` section above has already been followed, and silent everywhere else: in a one-off command typed at a prompt, inside `$(…)`, in a `sh` script with no `pipefail` available, or under any caller that reads only the final status.

Two rules, and neither depends on remembering the arithmetic. **Never pipe a state-changing command into a reader that stops early** — `confirming-before-claiming-done` and `crouton` both already say to redirect a long run to a file and search the file, and that advice keeps the writer alive as a side effect, which is the more important half here. And when a mutating command's output looks truncated, check the pipeline's per-stage status rather than `$?`, then ask the tool what state it thinks it is in — and be ready for the answer that it thinks nothing happened.

## Hostile filenames and safe iteration

Only `/` and NUL cannot appear in a filename. Newlines, leading dashes, quotes, glob characters, and non-UTF-8 bytes are all legal and all occur in the wild.

- `for f in *` and `for f in dir/*` are safe — globs do not word-split. Prefer them to everything below.
- Never parse `ls`. Its output is ambiguous by construction and it silently substitutes non-printing bytes when stdout is not a terminal.
- For recursion, `find … -exec cmd {} +` is POSIX. `-print0 | xargs -0` is a GNU/BSD extension: fine when you require those, not portable.
- Reading a list: `while IFS= read -r line`. `IFS=` stops whitespace trimming, `-r` stops backslash mangling, and neither is the default.
- A leading dash is an option: use `rm -- "$f"` or `rm "./$f"`.
- Sorting and case folding follow the locale. Set `LC_ALL=C` when you need byte order or ASCII rules.

## Subshells, pipelines, and lost assignments

Pipeline stages run in subshells, so assignments inside `cmd | while read …; do n=$((n+1)); done` are discarded at `done`. Redirect instead — `while …; done < file` — or use a here-document, or bash's `lastpipe`. ksh and zsh run the last stage in the current shell, so this code works for its author and loses data in CI.

`$(…)` runs in a subshell and strips *all* trailing newlines, so a `cd` inside it does not escape and a captured file loses its final blank lines. Capture status immediately with `rc=$?`, because `$?` is overwritten by the very next command, including `[` and `echo`.

Exit codes are 0–255 and `exit 256` becomes 0. 126 means not executable, 127 not found, 128+N a fatal signal N. Per-stage pipeline status exists only as bash `PIPESTATUS` and zsh `pipestatus`; under POSIX sh, restructure so you do not need it.

## A `cd` outlives the command that ran it

The trap above has a mirror image, and knowing one gives no protection against the other. `$(cd x && pwd)` runs in a subshell and leaves you where you were. `cd x && pwd`, typed at a session that persists between commands, moves you and keeps you there.

Persistent sessions are now the common case rather than the exotic one: an agent driving a shell across many calls, a CI job whose steps share a working directory, a terminal someone is scripting against by hand. In all three, the working directory is state that outlives the command that set it, so one compound command beginning `cd build && …` silently re-roots every relative path used afterwards — by a different command, possibly written by someone who never saw the `cd`.

What makes this expensive is not the breakage but the plausibility of the result. `no such file or directory` is exactly what a genuine absence looks like, and a grep that matches nothing looks exactly like a codebase that does not contain the pattern. The output is a confident false negative that reads as a finding about the system rather than an artifact of where the command ran, and it will be reported as one.

In order of preference:

| Approach | What it buys |
|---|---|
| Use the tool's own path option | `git -C dir`, `make -C dir`, `tar -C dir`, `rsync`'s full paths — no directory change happens at all |
| Use absolute paths | Immune to whatever the session's directory happens to be |
| Scope the change to a subshell | `(cd dir && …)` cannot outlive its own command, which is the whole point of the parenthesis |
| Change directory and stay there | Only when every later command genuinely wants the new root, and only if you say so out loud |

And when a path check comes back with a surprising negative, print `pwd` before believing it. That is one command against a wrong conclusion about somebody's codebase.

A `cd` that *fails* re-roots nothing and stops nothing: the shell stays where it already was, and the next command runs there. So `cd "$dir" 2>/dev/null || cd "$fallback"` has one intended outcome and two unintended ones that are indistinguishable from each other afterwards — the fallback ran and put you somewhere unrelated, or neither ran and you never moved. The chain guarantees you end up *somewhere*; nothing in it guarantees that somewhere is the one you meant. Errexit is no help, because both `cd`s sit in an `||` chain, one of the documented exemptions above — and the persistent sessions this section is about mostly have no `set -e` at all, so there was never anything to catch it. Where the table's last row genuinely applies, the floor is `cd "$dir" || exit 1`; every row above it avoids the question instead of answering it.

The consequence is worse for a **write** than for the read above, and worse in a different way. A wrong-root read produces a wrong answer, which is expensive and stays inside your own conclusions. A wrong-root write *succeeds* — file created, status 0, output identical to the run that did what you meant — and it changes something in a tree nobody is currently looking at. Then the verification agrees with it, because the obvious check is to read the file back **by the same relative path**, and that resolves the identical path against the identical wrong root. It confirms the write happened. It cannot say where. The check has inherited the mistake's own assumption, so it will go on agreeing however many times it is run. **A claim about location has to be settled by something that does not share the assumption** — `pwd`, the absolute path printed and read back, a listing of the parent directory you actually named, or the destination's own status, checked in the tree you meant *and* in the neighbouring one you may have hit. Nothing else will surface it: a file written into somebody else's checkout is discovered out of band, by whoever next looks there.

## Cleanup that survives signals

```sh
tmp=$(mktemp -d) || exit 1
cleanup() { rm -rf "$tmp"; }
trap cleanup EXIT
trap 'cleanup; trap - INT; kill -INT $$' INT
trap 'cleanup; exit 143' TERM HUP
```

`mktemp` rather than a `$$`-derived name: a predictable path in a world-writable directory is a symlink attack and a collision. An EXIT trap does not reliably fire on a signal in every shell, so name the signals and re-raise `INT` so callers see a real interrupt. SIGKILL and power loss cannot be trapped, so keep scratch data under the system temp root where the OS reclaims it, and make cleanup idempotent.

## Killing and waiting on processes by pattern

`pkill -f` and `pgrep -f` match against a process's *entire* command line, not just its name — every argument, every embedded string, whatever ends up in argv. That's most dangerous when the caller and the target share a harness — an `eval`, a heredoc, a wrapper script — because the same substring that identifies the target then also shows up in the caller's own command line. The bullets below are about killing by pattern; waiting on one fails through the same mechanism in the opposite direction, and is covered after them.

- Prefer a saved PID over pattern matching: write `$!` to a file right after starting the background process, then `kill "$(cat "$pidfile")"` to stop it later. No pattern, no risk of matching something else.
- If `-f` is unavoidable, check what it would hit before running it. `pgrep -f 'pattern'` lists the matching PIDs; `ps -o pid,args= -p <pids>` shows their exact command lines, including, potentially, the shell about to run the `pkill`. (GNU `pgrep -a` does both in one step; it's a procps extension, not available on BSD/macOS `pgrep`.)
- Anchor the pattern on something only the target has — a unique flag, a full path — not a bare project or script name.
- A precise pattern is still not a private one. Anchoring narrows the match to one program; it does not narrow it to *your copy* of that program. The process table is a single machine-wide namespace, shared with the user's editor, window manager, and browser, and with any other agent session running the same tooling — so a pattern can be exactly right and still name a process you did not start. Defeating self-match — the `'[p]attern'` bracket trick, a tighter anchor — closes that direction and leaves this one wide open, so on a shared or desktop machine assume a pattern naming a common binary also matches a stranger's copy of it. The saved PID above is the only handle that cannot.
- Never put the cleanup and the relaunch in one command. `pkill -f foo; start foo &` reads as stop-then-start and is not: by the time the pattern is evaluated the replacement can already be in the process table, so the kill takes the process it was run to make room for. The symptom is the reason this one costs so much — the job reports starting and then produces nothing, which reads as the job failing rather than as the cleanup having killed it, and every subsequent minute is spent debugging the wrong process. Kill, confirm the target is gone, then start.
- `pkill` without `-f` matches only the process's short kernel-tracked name (`comm`), which is immune to `argv[0]` spoofing but not to how the target was launched: for a script run directly (`./script.sh`), `comm` is the script's own basename, narrow enough to target; for a script run through an explicit interpreter (`bash script.sh`), `comm` is the *interpreter's* name, which matches every other script running under that interpreter too.

The same match used as a *wait condition* fails in the opposite direction, and is much easier to miss. `until ! pgrep -f foo; do sleep 20; done` never exits: the waiting shell's own command line contains the pattern, so the loop is waiting for itself to stop existing. Nothing is killed and nothing errors, so the shape reads as correct and gets written again — what a later observer finds is several loops that will never exit and that, from the process table, look exactly like several concurrent copies of the job. A kill that matches wrong is loud and gets debugged in the minute it happens; a wait that matches wrong is silent and accumulates for hours.

The fix is not a better pattern. Wait on the thing you actually care about: the artifact the job writes, a file it touches on completion, or — when this shell started the job — its own exit status, via `cmd & pid=$!` then `wait "$pid"`, which is POSIX and reports the job's own status. A completion file has to be removed before the job starts, or the previous run's copy satisfies the wait instantly. Whatever the condition, bound the loop with a counter, since `timeout` is not POSIX, and fail with a message naming what it waited for — so a wait that is wrong anyway gives up in minutes rather than at whatever hour someone happens to look. (`diagnosing-before-fixing` has the general form: poll the real condition, never a proxy, always with a bound.)

## Same name, different tool

The usual GNU-versus-BSD casualties: `sed -i` (BSD requires an argument, GNU forbids one), `readlink -f`, `date -d` versus `date -v`, `stat -c` versus `stat -f`, `grep -P`, `find -printf`, `xargs -r`, `sort -h`, `head -n -1`, `seq`, `tac`. `awk` splits three ways (gawk, mawk, busybox); the POSIX awk subset avoids nearly all of it.

Choose the POSIX subset first. Where an extension is genuinely needed, probe the capability at runtime rather than branching on `uname`, or check for the required tool once at startup and fail with a message naming what to install.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Works locally, `bad substitution` in CI | bash syntax under a `#!/bin/sh` shebang; CI's `sh` is dash |
| Script continues past an obvious failure | an errexit exemption — the failure sat in a function called from `if`, or upstream of a pipe |
| Counter is 0 after a `while read` loop | the loop body ran in a subshell on the right-hand side of a pipe |
| Loop runs once with a literal `*.txt` | the glob matched nothing and POSIX left the pattern unexpanded |
| `[: too many arguments` | an unquoted variable that was empty or contained spaces inside `[ ]` |
| Cleanup deleted the wrong tree | unquoted path, or an unset variable expanding to nothing with no `set -u` |
| Trailing newlines missing from captured output | `$(…)` strips every trailing newline; append a sentinel and remove it |
| Fails on exactly one machine | GNU versus BSD flags for `sed`, `date`, `readlink`, or `stat` |
| `pkill -f` killed the calling shell, not the target | the pattern also matched the shell's own wrapped or eval'd command line, not just the target's argv |
| A restarted background job reports starting, then does nothing | cleanup and relaunch ran in one command; the pattern matched the replacement, which was already in the process table |
| A `pgrep -f` wait loop never finishes, and there are now several of them | the pattern matched the waiting shell itself, so each loop waits for its own exit |
| A `pkill` aimed at the script's job took down one of the user's | the pattern named a program, not an instance; every copy on the machine matched |
| A file "does not exist" and then plainly does | A `cd` in an earlier command re-rooted every relative path after it |
| A file was written successfully, into the wrong tree | A `cd` failed or fell through to a fallback, and the relative write resolved against a root nobody chose |
| A tool reports nothing to recover from damage that is plainly there | It was killed by `SIGPIPE` before writing its own bookkeeping, because its output was piped into a reader that stopped early |

## Red flags

- A command that changes state, piped into `head`, `tail`, a pager, or `grep -q`.
- "It works on my machine" about anything carrying a `#!/bin/sh` shebang
- Reaching for `ls | while read`, `for f in $(find …)`, or `eval` on a constructed string
- A second level of quoting inside `ssh`, `sudo sh -c`, or a generated command line
- Adding `|| true` to silence a failure rather than to declare it non-fatal
- The script now holds arrays of records, parses JSON or CSV with `sed`, retries with backoff, or passes 200 lines — it has outgrown shell and should become a program in a language with data structures and a test runner
- `pkill -f` or `pgrep -f` run with a pattern nobody checked against every process it could match — the caller's own, and the user's
- A loop that waits for a process matching a string to disappear — the shell running the loop matches the string too
- A negative result from a relative path — nothing found, no such file — trusted without checking `pwd` first
- A `cd` whose failure is silenced or absorbed by a `||` fallback, with anything that writes after it
- A relative-path write checked by reading back the same relative path — that confirms the write and never the place
