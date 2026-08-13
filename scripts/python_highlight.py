"""Minimal Python syntax highlighter — stdlib only, no dependency, built for
one real file (test_install_command.py) rather than as a general Python
parser. It handles exactly what that file actually contains: comments,
single- and double-quoted strings (including their triple-quoted and
prefixed forms — b"...", r"...", f"...", and combinations), decorators,
numeric literals, and every real keyword/constant via the stdlib `keyword`
module itself rather than a hand-curated guess at the list.

Replaces shell_highlight.py now that the file being displayed on the
verify-install page is Python rather than bash. The two aren't
interchangeable at a deeper level than syntax, though: shell_highlight.py
could tokenize strictly line-by-line because nothing in a small bash script
ever spans multiple lines. Python docstrings and triple-quoted strings
genuinely do — so this module tokenizes the *whole* source in one pass and
only splits back into per-line HTML afterward, distributing any token that
crosses a line boundary across each `<span>` it touches. Concretely, that
means there is no standalone `highlight_line(line)` here the way there was
for shell: a single line can't be colored correctly in isolation without
knowing whether it's inside a triple-quoted string that started three lines
above it.

Deliberate simplification, same spirit as shell_highlight.py's treatment of
`$(...)`: an f-string's `{expr}` interpolation is not recursed into — the
whole f-string, braces and all, is one opaque string span. Correctly
tokenizing an expression nested inside a string literal needs a real parser,
which is disproportionate for what this is — a pleasant-to-read display of
one known script.

Public API:

    highlight_source(source: str) -> list[str]   # whole file -> one HTML string per line
"""

from __future__ import annotations

import keyword as _keyword_module
from html import escape as _escape_html

_KEYWORDS = frozenset(_keyword_module.kwlist) - {"True", "False", "None"}
_CONSTANTS = frozenset({"True", "False", "None"})

_STRING_PREFIX_CHARS = frozenset("rRbBfFuU")
_IDENT_START_CHARS = frozenset(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"
)
_IDENT_CHARS = _IDENT_START_CHARS | frozenset("0123456789")
_DIGIT_CHARS = frozenset("0123456789")


def _match_string_start(text: str, i: int) -> tuple[str, str] | None:
    """If text[i:] opens a string literal (with an optional prefix like r,
    b, f, or a two-letter combination like rb), returns (prefix, quote)
    where quote is the delimiter run — one character, or three for a
    triple-quoted string. Otherwise None. Never advances i itself; the
    caller decides what to do with the answer."""
    n = len(text)
    j = i
    while j < n and j < i + 2 and text[j] in _STRING_PREFIX_CHARS:
        j += 1
    if j < n and text[j] in ("'", '"'):
        prefix = text[i:j]
        q = text[j]
        quote = q * 3 if text[j:j + 3] == q * 3 else q
        return prefix, quote
    return None


def _scan_string(text: str, start: int, quote: str) -> int:
    """text[start:] is the string body (just past the opening delimiter);
    `quote` is that same delimiter. Returns the index just past the closing
    delimiter, honoring backslash-escapes — including inside raw strings,
    where a backslash still isn't allowed to touch the closing quote even
    though it stays literal in the string's value; that's a real quirk of
    Python's own tokenizer, not a simplification here. A single-line string
    (quote length 1) that hits an unescaped newline first is treated as
    ending there, since that's a syntax error in real Python and can't occur
    in a source file that actually runs — this just guarantees the
    highlighter never runs away past it instead of raising. Reaching the end
    of the text with no close is handled the same way: returns len(text)."""
    n = len(text)
    qlen = len(quote)
    i = start
    while i < n:
        if qlen == 1 and text[i] == "\n":
            return i
        if text[i] == "\\" and i + 1 < n:
            i += 2
            continue
        if text[i:i + qlen] == quote:
            return i + qlen
        i += 1
    return n


def _scan_comment(text: str, start: int) -> int:
    end = text.find("\n", start)
    return end if end != -1 else len(text)


def _at_line_start(text: str, i: int) -> bool:
    """True if everything since the previous newline (or the start of the
    file) is just indentation — i.e. text[i] opens a new logical line rather
    than sitting mid-expression. Used only to tell a decorator's leading `@`
    apart from the `@` matrix-multiplication operator, which can only appear
    mid-expression."""
    line_start = text.rfind("\n", 0, i) + 1
    return text[line_start:i].strip(" \t") == ""


def _scan_number(text: str, i: int) -> int:
    n = len(text)
    j = i
    if text[j] == "0" and j + 1 < n and text[j + 1] in "xXoObB":
        j += 2
        while j < n and (text[j].isalnum() or text[j] == "_"):
            j += 1
        return j
    while j < n and (text[j] in _DIGIT_CHARS or text[j] == "_"):
        j += 1
    if j < n and text[j] == ".":
        j += 1
        while j < n and (text[j] in _DIGIT_CHARS or text[j] == "_"):
            j += 1
    if j < n and text[j] in "eE" and j + 1 < n and (
        text[j + 1] in _DIGIT_CHARS or (text[j + 1] in "+-" and j + 2 < n and text[j + 2] in _DIGIT_CHARS)
    ):
        j += 1
        if text[j] in "+-":
            j += 1
        while j < n and text[j] in _DIGIT_CHARS:
            j += 1
    if j < n and text[j] in "jJ":
        j += 1
    return j


def _is_decorator_char(ch: str) -> bool:
    return ch in _IDENT_CHARS or ch == "."


def _scan_decorator(text: str, i: int) -> int:
    n = len(text)
    j = i + 1
    while j < n and _is_decorator_char(text[j]):
        j += 1
    return j


def _is_token_start(text: str, i: int) -> bool:
    """Whether position i could open something other than plain text — used
    only to find how far a run of "definitely plain" text extends, so plain
    text (operators, punctuation, whitespace) isn't emitted one character at
    a time."""
    ch = text[i]
    return (
        ch in "#'\"@"
        or ch in _IDENT_CHARS
        or (ch == "." and i + 1 < len(text) and text[i + 1] in _DIGIT_CHARS)
    )


def _tokenize(text: str) -> list[tuple[str | None, str]]:
    """The whole source as a flat list of (css_class_or_None, text) chunks
    that reconstruct `text` exactly when concatenated. A chunk's text may
    contain embedded newlines (triple-quoted strings, mainly) — splitting
    those back into per-line HTML is highlight_source's job, not this one."""
    tokens: list[tuple[str | None, str]] = []
    i = 0
    n = len(text)

    while i < n:
        ch = text[i]

        if ch == "#":
            end = _scan_comment(text, i)
            tokens.append(("tok-comment", text[i:end]))
            i = end
            continue

        if ch == "@" and _at_line_start(text, i):
            end = _scan_decorator(text, i)
            if end > i + 1:
                tokens.append(("tok-const", text[i:end]))
                i = end
                continue

        prefix_info = _match_string_start(text, i)
        if prefix_info is not None:
            prefix, quote = prefix_info
            body_start = i + len(prefix) + len(quote)
            end = _scan_string(text, body_start, quote)
            tokens.append(("tok-string", text[i:end]))
            i = end
            continue

        if ch in _DIGIT_CHARS or (ch == "." and i + 1 < n and text[i + 1] in _DIGIT_CHARS):
            end = _scan_number(text, i)
            tokens.append(("tok-number", text[i:end]))
            i = end
            continue

        if ch in _IDENT_START_CHARS:
            j = i + 1
            while j < n and text[j] in _IDENT_CHARS:
                j += 1
            word = text[i:j]
            if word in _CONSTANTS:
                tokens.append(("tok-const", word))
            elif word in _KEYWORDS:
                tokens.append(("tok-keyword", word))
            else:
                tokens.append((None, word))
            i = j
            continue

        j = i + 1
        while j < n and not _is_token_start(text, j):
            j += 1
        tokens.append((None, text[i:j]))
        i = j

    return tokens


def _render_token(cls: str | None, text: str) -> str:
    escaped = _escape_html(text)
    if cls is None:
        return escaped
    return f'<span class="{cls}">{escaped}</span>'


def highlight_source(source: str) -> list[str]:
    """One highlighted HTML string per line of source (trailing newline
    dropped, matching str.splitlines()). A token that spans multiple physical
    lines — a triple-quoted string is the only real case — is split at each
    embedded newline and re-wrapped in its own <span> per line, since each
    line becomes its own list item in the rendered code block and a single
    <span> can't straddle two of those."""
    core = source.removesuffix("\n")
    tokens = _tokenize(core)

    lines: list[list[tuple[str | None, str]]] = [[]]
    for cls, text in tokens:
        parts = text.split("\n")
        for k, part in enumerate(parts):
            if k > 0:
                lines.append([])
            if part:
                lines[-1].append((cls, part))

    return ["".join(_render_token(cls, part) for cls, part in line) for line in lines]
