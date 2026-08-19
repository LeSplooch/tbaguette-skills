---
name: portable-shell-scripting
description: Use when writing or reviewing a shell script, when a script works in one shell but fails in another, when it breaks on filenames with spaces or newlines, or when a failing command does not stop the script. Covers quoting and word splitting, set -e exemptions, lost variable assignments after a pipeline, exit codes and pipeline status, cleanup traps, POSIX sh versus bash-isms, GNU versus BSD tool differences, and when a script has outgrown shell.
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

## Cleanup that survives signals

```sh
tmp=$(mktemp -d) || exit 1
cleanup() { rm -rf "$tmp"; }
trap cleanup EXIT
trap 'cleanup; trap - INT; kill -INT $$' INT
trap 'cleanup; exit 143' TERM HUP
```

`mktemp` rather than a `$$`-derived name: a predictable path in a world-writable directory is a symlink attack and a collision. An EXIT trap does not reliably fire on a signal in every shell, so name the signals and re-raise `INT` so callers see a real interrupt. SIGKILL and power loss cannot be trapped, so keep scratch data under the system temp root where the OS reclaims it, and make cleanup idempotent.

## Killing processes by pattern

`pkill -f` and `pgrep -f` match against a process's *entire* command line, not just its name — every argument, every embedded string, whatever ends up in argv. That's most dangerous when the caller and the target share a harness — an `eval`, a heredoc, a wrapper script — because the same substring that identifies the target then also shows up in the caller's own command line.

- Prefer a saved PID over pattern matching: write `$!` to a file right after starting the background process, then `kill "$(cat "$pidfile")"` to stop it later. No pattern, no risk of matching something else.
- If `-f` is unavoidable, check what it would hit before running it. `pgrep -f 'pattern'` lists the matching PIDs; `ps -o pid,args= -p <pids>` shows their exact command lines, including, potentially, the shell about to run the `pkill`. (GNU `pgrep -a` does both in one step; it's a procps extension, not available on BSD/macOS `pgrep`.)
- Anchor the pattern on something only the target has — a unique flag, a full path — not a bare project or script name.
- `pkill` without `-f` matches only the process's short kernel-tracked name (`comm`), which is immune to `argv[0]` spoofing but not to how the target was launched: for a script run directly (`./script.sh`), `comm` is the script's own basename, narrow enough to target; for a script run through an explicit interpreter (`bash script.sh`), `comm` is the *interpreter's* name, which matches every other script running under that interpreter too.

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

## Red flags

- "It works on my machine" about anything carrying a `#!/bin/sh` shebang
- Reaching for `ls | while read`, `for f in $(find …)`, or `eval` on a constructed string
- A second level of quoting inside `ssh`, `sudo sh -c`, or a generated command line
- Adding `|| true` to silence a failure rather than to declare it non-fatal
- The script now holds arrays of records, parses JSON or CSV with `sed`, retries with backoff, or passes 200 lines — it has outgrown shell and should become a program in a language with data structures and a test runner
- `pkill -f` or `pgrep -f` run with a pattern nobody checked against every process it could match, including the caller's own
