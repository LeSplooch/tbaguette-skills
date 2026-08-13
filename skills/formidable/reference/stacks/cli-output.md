# Stack: CLI output

**Envelope.** A one-directional stream that may be a human's terminal, a pipe, a log file, or a CI panel. You do not control the width, the theme, the scrollback, or whether anyone is watching. Every line is permanent and greppable.

**The premise:** command-line output is interface, and most of it is designed by accident. A stack trace, a help screen, and a progress bar are all UI.

## The two-audience rule

Every command has a human reader and a machine reader, and they want opposite things. Serve both explicitly.

- Detect a TTY on stdout. Not a TTY → no color, no spinners, no cursor tricks, no boxes, stable line-oriented output.
- Honor `NO_COLOR`, `--no-color`, `--quiet`, `--verbose`, and `--json` (or the ecosystem's equivalent). `--json` output is a contract: stable keys, no decoration, no progress mixed in.
- **Diagnostics go to stderr. Data goes to stdout.** This one rule fixes most piping complaints.

## Craft

- **Errors are the most-read screen you ship.** Structure every one: what failed, where (file/line/key/id), why, and the single most likely fix — ideally a copy-pasteable command. Nothing costs a user more than an error naming only the exception class.
- **Help is a designed page.** One-line summary, usage line, the four or five flags people actually use, then examples. Full reference goes to a subcommand or man page. An alphabetized dump of 60 flags is a failure to prioritize.
- **Progress must be honest.** Known total → a bar with a count and a rate. Unknown total → an elapsed counter plus the current item, never a fake percentage. Under 300ms → nothing at all.
- **Alignment does the work color would.** Align keys, right-align numbers, pad to the widest label. A well-aligned monochrome block outreads a colorful ragged one.
- **Color carries severity, never information.** Red/yellow/green plus a word: `error:`, `warning:`, `ok:`. Piped output loses the color and must lose nothing else.
- **Verbosity is a ladder.** Default output is what a competent user needs; `-v` adds decisions; `-vv` adds internals. Never make the default level the debug level.
- **Summarize at the end.** Long-running commands close with counts, duration, and what to do next. Users scroll to the bottom first.
- **Silence on success** is the Unix default and it is correct for anything scripted. If it worked and there is nothing to say, say nothing.

## Failure modes

| Symptom | Real cause |
|---|---|
| Garbage in CI logs | Spinners and cursor codes emitted without a TTY check. |
| Users can't script it | Data and diagnostics both on stdout; unstable, decorated output. |
| "It just says failed" | Error names the exception, not the input, the cause, or the fix. |
| Nobody finds the flag | Help is exhaustive rather than ranked; no examples. |
| Wrapped, unreadable output | Assuming 80 columns instead of reading the real width, or hard-wrapping text that should flow. |

## Audit hooks

`cmd | cat`, `cmd > file`, `cmd 2>/dev/null`, `cmd --help | head -20`, `NO_COLOR=1`, a 40-column window, a failing run, an interrupted run, and one run whose output is consumed by another command.
