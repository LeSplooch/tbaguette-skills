"""Minimal shell syntax highlighter — stdlib only, no dependency, built for
one real file (test_install_command.sh) rather than as a general bash
parser. It handles exactly the constructs that file actually contains:
comments, single- and double-quoted strings, command substitution (`$(...)`,
correctly balanced to arbitrary nesting depth — this script genuinely has
`$(...)` inside `$(...)`), variable expansion (`$VAR`, `${VAR}`, `$?` and
friends), and a curated keyword/builtin list.

Deliberate simplification: content inside `$(...)` is highlighted as one
opaque "command substitution" span rather than recursed into. Real bash
would let a quoted string or another `$(...)` appear inside there with full
meaning; correctly highlighting that in general requires a real quote-aware
parser, which is disproportionate for what this is — a pleasant-to-read
display of one known script, not a bash IDE. The text inside is still
rendered exactly, just in one color rather than several.

Public API:

    highlight_line(line: str) -> str   # one line of shell source -> HTML

Line-oriented rather than whole-file: every construct in the target file is
single-line (no multi-line strings), so tokenizing per line sidesteps ever
needing to carry quote state across a line boundary.
"""

from __future__ import annotations

import re
from html import escape as _escape_html

_VAR_RE = re.compile(r"\$\{[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]")

# Standard bash keywords (most unused in this specific file, kept for when
# it grows) plus the builtins/commands this file actually calls.
_KEYWORDS = (
    "if", "then", "elif", "else", "fi", "for", "while", "until", "do",
    "done", "case", "esac", "function", "select", "in", "return", "break",
    "continue", "local", "export", "trap", "exit", "set", "echo", "printf",
    "read", "shift", "unset", "mkdir", "rm", "cd", "test", "find", "sort",
    "grep", "sha256sum",
)
_KEYWORD_ALTERNATION = "|".join(_KEYWORDS)
_KEYWORD_RE = re.compile(r"\b(?:" + _KEYWORD_ALTERNATION + r")\b")

# Any character that could start a token the main loop treats specially —
# used to find how far a run of "definitely plain" text extends, so plain
# text isn't emitted one character at a time.
_PLAIN_STOP_RE = re.compile(r"""[#"'$]|\b(?:""" + _KEYWORD_ALTERNATION + r")\b")


def _skip_balanced_parens(s: str, open_index: int) -> int:
    """s[open_index] must be '('. Returns the index just after the matching
    ')', counting nested parens correctly. Does not account for a paren
    appearing inside a quoted string within the span — undefeated by
    anything in the one file this is built for (see module docstring), but
    worth knowing if this is ever pointed at a different script."""
    depth = 0
    i = open_index
    n = len(s)
    while i < n:
        if s[i] == "(":
            depth += 1
        elif s[i] == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def _span(css_class: str, text: str) -> str:
    return f'<span class="{css_class}">{_escape_html(text)}</span>'


def highlight_line(line: str) -> str:
    """HTML for one line of shell source. Never raises on malformed input —
    worst case for a construct outside what this is built for is imperfect
    color boundaries, never wrong displayed text, since every branch below
    still emits the real source characters, escaped, either way."""
    out: list[str] = []
    i = 0
    n = len(line)

    while i < n:
        ch = line[i]

        if ch == "#":
            out.append(_span("tok-comment", line[i:]))
            break

        if ch == '"':
            # Common idiom: "$(...)" — the entire quoted string is just a
            # command substitution wrapped to suppress word-splitting, e.g.
            # WORKDIR="$(mktemp -d)". That's the overwhelming majority of
            # double-quoted content in this file. Give it its own cmdsub
            # color (quotes included) rather than filing it under generic
            # "string" — the substitution is the part worth the reader's
            # attention, the quotes are just plumbing.
            if i + 2 < n and line[i + 1] == "$" and line[i + 2] == "(":
                close_paren = _skip_balanced_parens(line, i + 2)
                if close_paren < n and line[close_paren] == '"':
                    j = close_paren + 1
                    out.append(_span("tok-cmdsub", line[i:j]))
                    i = j
                    continue

            # General double-quoted string: scan to the matching close
            # quote, treating any $(...) found along the way as an opaque,
            # not-further-highlighted unit — so a quote embedded inside it
            # (this file has one: a string inside a $(...) inside a string)
            # can never be mistaken for closing the outer string early.
            j = i + 1
            while j < n:
                if line[j] == "\\" and j + 1 < n:
                    j += 2
                    continue
                if line[j] == "$" and j + 1 < n and line[j + 1] == "(":
                    j = _skip_balanced_parens(line, j + 1)
                    continue
                if line[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(_span("tok-string", line[i:j]))
            i = j
            continue

        if ch == "'":
            # Single quotes are literal in shell — no interpolation, no
            # escapes, ends at the very next quote. Nothing inside is
            # further tokenized, which is more correct than highlighting a
            # $VAR in there as if it expanded — it doesn't.
            close = line.find("'", i + 1)
            j = close + 1 if close != -1 else n
            out.append(_span("tok-string", line[i:j]))
            i = j
            continue

        if ch == "$" and i + 1 < n and line[i + 1] == "(":
            j = _skip_balanced_parens(line, i + 1)
            out.append(_span("tok-cmdsub", line[i:j]))
            i = j
            continue

        m = _VAR_RE.match(line, i)
        if m:
            out.append(_span("tok-var", m.group()))
            i = m.end()
            continue

        m = _KEYWORD_RE.match(line, i)
        if m:
            out.append(_span("tok-keyword", m.group()))
            i = m.end()
            continue

        j = i + 1
        while j < n and not _PLAIN_STOP_RE.match(line, j):
            j += 1
        out.append(_escape_html(line[i:j]))
        i = j

    return "".join(out)


def highlight_source(source: str) -> list[str]:
    """One highlighted HTML string per line of source (trailing newline
    dropped, matching str.splitlines())."""
    return [highlight_line(line) for line in source.splitlines()]
