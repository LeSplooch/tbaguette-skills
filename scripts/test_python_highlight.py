"""Self-tests for python_highlight.py.

Fixtures are drawn from the actual test_install_command.py this was built
for wherever possible — including its module and function docstrings, which
are the genuinely hard case a whole-source, multi-line-aware tokenizer
exists for in the first place (shell_highlight.py, which this replaces,
never had to deal with a token spanning more than one line). A few
constructs the real file doesn't happen to contain — decorators, and the
`@` matrix-multiplication operator that a decorator's leading `@` must be
told apart from — are still worth proving correct even though they're
synthetic here, since a highlighter that's only ever been run against its
one target file proves nothing about what happens the next time that file
changes.

Usage:
    python3 scripts/test_python_highlight.py
"""

from __future__ import annotations

import re as _re
from html import unescape as _unescape
from pathlib import Path

from checker import Checker
from python_highlight import highlight_source

checker = Checker()
check = checker.check


def _strip_markup(html_lines: list[str], original: str) -> str:
    """Reverses highlight_source's output back to the original plain text:
    drop every <span ...> and </span> tag, unescape entities, rejoin with
    \\n. Used to prove round-tripping — the display must never lose or
    alter a character, only wrap it in color. Takes `original` only to
    decide whether to add back the one trailing newline splitlines()-style
    output always drops."""
    joined = "\n".join(_re.sub(r"</?span[^>]*>", "", line) for line in html_lines)
    plain = _unescape(joined)
    return plain + "\n" if original.endswith("\n") and not plain.endswith("\n") else plain


def main() -> None:
    # --- comments -----------------------------------------------------
    print("comments")
    lines = highlight_source("# Proves the published install command\n")
    check("whole comment line wrapped", lines[0].startswith('<span class="tok-comment">'))
    check("comment text present", "Proves the published install command" in lines[0])

    lines = highlight_source('x = 1  # trailing comment\n')
    check("trailing comment after code is still a comment",
          'class="tok-comment"># trailing comment</span>' in lines[0])

    # A quote character appearing *inside* a comment must never be read as
    # opening a string that swallows the rest of the line.
    lines = highlight_source("# a user asked \"won't this alter other skills?\"\n")
    check("a quote inside a comment does not start a string",
          lines[0].count('class="tok-string"') == 0)
    check("the comment still contains the quote characters verbatim",
          "won" in lines[0] and "alter other skills" in lines[0])

    # --- strings, single line -------------------------------------------
    print("strings (single line)")
    lines = highlight_source('x = "precious content" + \'more content\'\n')
    check("a double-quoted and a single-quoted string are both highlighted",
          lines[0].count('class="tok-string"') == 2)

    lines = highlight_source("home = target / \".git\"\n")
    check("a dot inside a string is just part of the string, not a number",
          'class="tok-string">&quot;.git&quot;</span>' in lines[0])

    # --- string prefixes: raw, bytes, f-strings, and combinations --------
    print("string prefixes")
    lines = highlight_source('r = r"raw\\nstring"\n')
    check("a raw string is recognized as one string span including its prefix",
          'class="tok-string">r&quot;raw\\nstring&quot;</span>' in lines[0])

    lines = highlight_source('b = rb"""triple raw bytes"""\n')
    check("a two-letter prefix (rb) on a triple-quoted string is handled",
          'class="tok-string">rb&quot;&quot;&quot;triple raw bytes&quot;&quot;&quot;</span>' in lines[0])

    # f-strings are deliberately opaque: the {expr} inside is not recursed
    # into and highlighted separately, the same simplification precedent as
    # shell_highlight.py's treatment of $(...).
    lines = highlight_source('msg = f"exits 0 under {Path(shell).name}"\n')
    check("an f-string's {expr} is swallowed into the one opaque string span",
          lines[0].count('class="tok-string"') == 1
          and "{Path(shell).name}" in lines[0])

    # --- the real multi-line case: triple-quoted docstrings ---------------
    print("multi-line strings")
    source = 'def f():\n    """First line.\n\n    Third line.\n    """\n    return 1\n'
    lines = highlight_source(source)
    check("a docstring spanning 4 physical lines produces one output line each",
          len(lines) == len(source.splitlines()) == 6)
    check("each physical line of the docstring is independently wrapped in "
          "its own span (no span straddles two <li> lines)",
          # line 1 keeps its code indentation as plain text before the span
          # starts (the docstring's own leading indent on the *inside*,
          # lines 3-4, is part of the string's content instead, since the
          # opening triple-quote there is column 0 of the token)
          lines[1] == '    <span class="tok-string">&quot;&quot;&quot;First line.</span>'
          and lines[2] == ""
          and lines[3].startswith('<span class="tok-string">')
          and lines[4].startswith('<span class="tok-string">'))
    check("round-trips exactly across the whole multi-line construct",
          _strip_markup(lines, source) == source)

    # --- decorators vs. the @ matrix-multiplication operator --------------
    print("decorators")
    lines = highlight_source("@staticmethod\ndef f(): pass\n")
    check("a decorator at the start of a line is its own const span",
          lines[0] == '<span class="tok-const">@staticmethod</span>')

    lines = highlight_source("@app.route(\"/x\")\ndef f(): pass\n")
    check("a dotted decorator name is included in the one span",
          '<span class="tok-const">@app.route</span>' in lines[0])

    lines = highlight_source("result = a @ b\n")
    check("@ used as matrix-multiplication mid-expression is NOT treated as "
          "a decorator",
          'tok-const' not in lines[0] and "a @ b" in _strip_markup(lines, "a @ b\n"))

    # --- numbers ------------------------------------------------------------
    print("numbers")
    lines = highlight_source("x = 1_000 + 0x1F - 0b101 + .5e-3\n")
    check("int with underscore separator", 'class="tok-number">1_000</span>' in lines[0])
    check("hex literal", 'class="tok-number">0x1F</span>' in lines[0])
    check("binary literal", 'class="tok-number">0b101</span>' in lines[0])
    check("leading-dot float with negative exponent",
          'class="tok-number">.5e-3</span>' in lines[0])

    lines = highlight_source("sha256sum_result = home_a\n")
    check("digits inside an identifier are not mistaken for a number literal",
          'tok-number' not in lines[0])

    # --- keywords and constants --------------------------------------------
    print("keywords and constants")
    lines = highlight_source("if result.returncode != 0:\n    return True\nelse:\n    x = None\n")
    check("if is a keyword", 'class="tok-keyword">if</span>' in lines[0])
    check("return is a keyword", 'class="tok-keyword">return</span>' in lines[1])
    check("True is a constant, not a generic keyword",
          'class="tok-const">True</span>' in lines[1])
    check("else is a keyword", 'class="tok-keyword">else</span>' in lines[2])
    check("None is a constant", 'class="tok-const">None</span>' in lines[3])

    # --- escaping / safety ---------------------------------------------------
    print("escaping")
    lines = highlight_source('x = "<script>alert(1)</script>"\n')
    check("a literal < in source content is escaped, never raw",
          "<script>" not in lines[0])
    check("the escaped form is present", "&lt;script&gt;" in lines[0])

    # --- the real file, in full ----------------------------------------------
    print("the real file, in full")
    real_path = Path(__file__).resolve().parent / "test_install_command.py"
    source = real_path.read_text(encoding="utf-8")
    lines = highlight_source(source)
    check(f"produces one HTML string per source line ({len(source.splitlines())} lines)",
          len(lines) == len(source.splitlines()))
    check("the whole file round-trips exactly, including its module and "
          "function docstrings",
          _strip_markup(lines, source) == source)
    check("no line is left completely unhighlighted-looking on a file this "
          "comment-and-docstring-heavy (at least a third of lines carry a span)",
          sum(1 for h in lines if "<span" in h) >= len(lines) // 3)
    check("the file's own f-string (verifying a shell exit code) survives "
          "the round trip and is colored as one string",
          any('class="tok-string">f&quot;' in h for h in lines))

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
