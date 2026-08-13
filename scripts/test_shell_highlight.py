"""Self-tests for shell_highlight.py.

Fixtures are lines lifted verbatim from the actual test_install_command.sh
this was built for — including the two genuinely hard cases in that file:
a double-quoted string containing a command substitution that itself
contains a double-quoted string, and a command substitution nested inside
another command substitution. A highlighter that only handles toy examples
proves nothing about the file it's actually meant to display.

Usage:
    python3 scripts/test_shell_highlight.py
"""

from __future__ import annotations

from pathlib import Path

from shell_highlight import highlight_line, highlight_source

_checks_run = 0


def check(label: str, condition: bool) -> None:
    global _checks_run
    _checks_run += 1
    if not condition:
        raise AssertionError(f"FAILED: {label}")
    print(f"  ok  {label}")


def main() -> None:
    # --- comments -----------------------------------------------------
    print("comments")
    html = highlight_line('# Proves the published install command')
    check("whole comment line wrapped", html.startswith('<span class="tok-comment">'))
    check("comment text present", "Proves the published install command" in html)

    html = highlight_line('echo "value"  # trailing comment')
    check("trailing comment after code is still a comment",
          'class="tok-comment">  # trailing comment</span>' in html
          or 'class="tok-comment"># trailing comment</span>' in html)

    # A quote character appearing *inside* a comment (real line from the
    # file) must never be read as opening a string that swallows the rest
    # of the line — the whole thing is one comment, full stop.
    html = highlight_line('# ~/.claude/skills/ that this repo doesn\'t own — a user asked "won\'t this')
    check("a quote inside a comment does not start a string",
          html.count('class="tok-string"') == 0)
    check("the comment still contains the quote character verbatim",
          "&quot;won&#x27;t this" in html or '"won' in html or "won&#x27;t" in html)

    # --- strings --------------------------------------------------------
    print("strings")
    html = highlight_line('echo "precious user content that must survive" > "$home/.claude/skills/some-other-skill/SKILL.md"')
    check("two separate double-quoted strings both highlighted",
          html.count('class="tok-string"') == 2)
    check("a variable inside a double-quoted string is still visible in the output",
          "home" in html)

    html = highlight_line("trap 'rm -rf \"$WORKDIR\"' EXIT")
    check("single-quoted string is one opaque span",
          html.count('class="tok-string"') == 1)
    check("a double quote embedded in a single-quoted string does not start its own string",
          html.count('class="tok-string"') == 1)

    # --- the hard case: string containing a command substitution
    #     containing another string ------------------------------------
    print("nested: \"$(...)\" wrap containing its own quoted string")
    line = 'before_a="$(siblings_checksum "$home_a")"'
    html = highlight_line(line)
    check("produces valid, non-empty output", len(html) > 0)
    check("every character of the source survives somewhere in the output "
          "(strip tags and entities back down and compare)",
          _strip_markup(html) == line)
    check("the outer string's own closing quote is the last thing on the "
          "line, not swallowed early by the inner one",
          html.rstrip().endswith("</span>"))
    check("the whole \"$(...)\" wrap is one cmdsub span, not filed as a "
          "generic string, since the substitution is the interesting part",
          html.count('class="tok-cmdsub"') == 1 and 'class="tok-string"' not in html)

    # --- the hard case: command substitution nested inside another ------
    print("nested: $(...) inside $(...)")
    line2 = 'out_b="$([ "$before_b" = "$(siblings_checksum "$home_b")" ] && echo 0 || echo 1)"'
    html2 = highlight_line(line2)
    check("round-trips exactly (no characters lost or duplicated)",
          _strip_markup(html2) == line2)

    # --- command substitution as its own token, not inside a string -----
    print("bare command substitution")
    html = highlight_line('WORKDIR="$(mktemp -d)"')
    check("mktemp -d is present in the output", "mktemp" in html and "-d" in html)

    # --- variables --------------------------------------------------------
    print("variables")
    html = highlight_line('rc_b=$?')
    check("$? (special single-char variable) is recognized", 'class="tok-var"' in html)
    html = highlight_line('checks=$((checks + 1))')
    check("round-trips even for arithmetic expansion (not specially handled, "
          "but must not corrupt the text)",
          _strip_markup(html) == 'checks=$((checks + 1))')

    # --- keywords ---------------------------------------------------------
    print("keywords")
    html = highlight_line('    local home="$1"')
    check("local is highlighted as a keyword", 'class="tok-keyword">local</span>' in html)
    html = highlight_line('mkdir -p "$home_a"')
    check("mkdir is highlighted as a keyword", 'class="tok-keyword">mkdir</span>' in html)

    # --- escaping / safety --------------------------------------------------
    print("escaping")
    html = highlight_line('echo "<script>alert(1)</script>"')
    check("a literal < in source content is escaped, never raw",
          "<script>" not in html)
    check("the escaped form is present", "&lt;script&gt;" in html)

    # --- whole real file --------------------------------------------------
    print("the real file, in full")
    real_path = Path(__file__).resolve().parent / "test_install_command.sh"
    source = real_path.read_text(encoding="utf-8")
    lines = highlight_source(source)
    check(f"produces one HTML string per source line ({len(source.splitlines())} lines)",
          len(lines) == len(source.splitlines()))
    check("every line round-trips to its exact original text",
          all(_strip_markup(h) == s for h, s in zip(lines, source.splitlines())))
    check("no line is left completely unhighlighted-looking on a file this "
          "comment-heavy (at least half the lines carry some span)",
          sum(1 for h in lines if "<span" in h) >= len(lines) // 2)

    print(f"\n{_checks_run} checks passed.")


def _strip_markup(html: str) -> str:
    """Reverses highlight_line's output back to the original plain text:
    drop every <span ...> and </span> tag, then unescape entities. Used to
    prove round-tripping — the display must never lose or alter a
    character, only wrap it in color."""
    import re as _re
    from html import unescape

    without_tags = _re.sub(r"</?span[^>]*>", "", html)
    return unescape(without_tags)


if __name__ == "__main__":
    main()
