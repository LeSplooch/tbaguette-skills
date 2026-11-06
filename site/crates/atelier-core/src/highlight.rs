//! The /verify-install/ page's syntax highlighter, ported from
//! `scripts/python_highlight.py`. It is the site's only highlighter: fenced
//! blocks in skill bodies are never highlighted (the Markdown renderer
//! discards their language tag).
//!
//! It is built for one real file (scripts/test_install_command.py), not as a
//! general Python lexer: comments, strings with their prefixes and
//! triple-quoted forms, decorators, numbers, keywords and constants. The whole
//! source is tokenized in one pass, because a triple-quoted string spans
//! lines, and only then split back into one HTML string per line, re-wrapping
//! a multi-line token in its own `<span>` on every line it touches. An
//! f-string's `{expr}` is not recursed into.
//!
//! Positions are code points, as in Python, so the scanner works over a
//! `Vec<char>`.

use crate::escape::html_attr;
use crate::markdown::is_py_alnum;

/// Python's `keyword.kwlist` minus `True`/`False`/`None`, frozen at the
/// interpreter the oracle was dumped with (3.14.8; highlight.json records
/// it). The Python module read the running interpreter's list, so this was
/// the one output that moved with the Python version; here it does not.
pub const KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda",
    "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
];

/// Highlighted as `tok-const`, like a decorator.
pub const CONSTANTS: &[&str] = &["False", "None", "True"];

fn is_string_prefix(c: char) -> bool {
    matches!(c, 'r' | 'R' | 'b' | 'B' | 'f' | 'F' | 'u' | 'U')
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// One highlighted HTML string per line of `source`, a single trailing
/// newline dropped first (python_highlight.py `highlight_source`). Lines are
/// split on `\n` only, so a `\r` stays in the text.
pub fn highlight_source(source: &str) -> Vec<String> {
    let core = source.strip_suffix('\n').unwrap_or(source);
    let text: Vec<char> = core.chars().collect();

    let mut lines: Vec<String> = vec![String::new()];
    for (class, token) in tokenize(&text) {
        for (k, part) in token.split('\n').enumerate() {
            if k > 0 {
                lines.push(String::new());
            }
            if part.is_empty() {
                continue;
            }
            let line = lines.last_mut().expect("lines starts non-empty");
            match class {
                Some(class) => {
                    line.push_str("<span class=\"");
                    line.push_str(class);
                    line.push_str("\">");
                    line.push_str(&html_attr(part));
                    line.push_str("</span>");
                }
                None => line.push_str(&html_attr(part)),
            }
        }
    }
    lines
}

/// The source as `(css class or None, text)` chunks that concatenate back to
/// it exactly (python_highlight.py `_tokenize`).
fn tokenize(text: &[char]) -> Vec<(Option<&'static str>, String)> {
    let mut tokens = Vec::new();
    let n = text.len();
    let mut i = 0;
    let piece = |a: usize, b: usize| text[a..b].iter().collect::<String>();

    while i < n {
        let ch = text[i];

        if ch == '#' {
            let end = scan_comment(text, i);
            tokens.push((Some("tok-comment"), piece(i, end)));
            i = end;
            continue;
        }

        if ch == '@' && at_line_start(text, i) {
            let end = scan_decorator(text, i);
            if end > i + 1 {
                tokens.push((Some("tok-const"), piece(i, end)));
                i = end;
                continue;
            }
        }

        if let Some((prefix_len, quote_len)) = match_string_start(text, i) {
            let quote = text[i + prefix_len];
            let end = scan_string(text, i + prefix_len + quote_len, quote, quote_len);
            tokens.push((Some("tok-string"), piece(i, end)));
            i = end;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && digit_at(text, i + 1)) {
            let end = scan_number(text, i);
            tokens.push((Some("tok-number"), piece(i, end)));
            i = end;
            continue;
        }

        if is_ident_start(ch) {
            let mut j = i + 1;
            while j < n && is_ident(text[j]) {
                j += 1;
            }
            let word = piece(i, j);
            let class = if CONSTANTS.contains(&word.as_str()) {
                Some("tok-const")
            } else if KEYWORDS.contains(&word.as_str()) {
                Some("tok-keyword")
            } else {
                None
            };
            tokens.push((class, word));
            i = j;
            continue;
        }

        let mut j = i + 1;
        while j < n && !is_token_start(text, j) {
            j += 1;
        }
        tokens.push((None, piece(i, j)));
        i = j;
    }
    tokens
}

fn digit_at(text: &[char], i: usize) -> bool {
    text.get(i).is_some_and(char::is_ascii_digit)
}

/// If `text[i..]` opens a string literal (an optional prefix of up to two of
/// `rRbBfFuU`, then a quote), its `(prefix length, delimiter length)`: the
/// delimiter is one quote, or three for a triple-quoted string.
fn match_string_start(text: &[char], i: usize) -> Option<(usize, usize)> {
    let n = text.len();
    let mut j = i;
    while j < n && j < i + 2 && is_string_prefix(text[j]) {
        j += 1;
    }
    let q = *text.get(j)?;
    if q != '\'' && q != '"' {
        return None;
    }
    let triple = j + 3 <= n && text[j + 1] == q && text[j + 2] == q;
    Some((j - i, if triple { 3 } else { 1 }))
}

/// The index just past the closing delimiter of a string whose body starts
/// at `start`, honouring backslash escapes (even in raw strings, as Python's
/// tokenizer does). A single-quoted string ends at an unescaped newline; an
/// unclosed string ends at the end of the text.
fn scan_string(text: &[char], start: usize, quote: char, quote_len: usize) -> usize {
    let n = text.len();
    let mut i = start;
    while i < n {
        if quote_len == 1 && text[i] == '\n' {
            return i;
        }
        if text[i] == '\\' && i + 1 < n {
            i += 2;
            continue;
        }
        if i + quote_len <= n && text[i..i + quote_len].iter().all(|&c| c == quote) {
            return i + quote_len;
        }
        i += 1;
    }
    n
}

fn scan_comment(text: &[char], start: usize) -> usize {
    text[start..]
        .iter()
        .position(|&c| c == '\n')
        .map_or(text.len(), |p| start + p)
}

/// Whether only spaces and tabs precede `i` on its line: tells a decorator's
/// `@` from the matrix-multiplication operator.
fn at_line_start(text: &[char], i: usize) -> bool {
    let line_start = text[..i]
        .iter()
        .rposition(|&c| c == '\n')
        .map_or(0, |p| p + 1);
    text[line_start..i].iter().all(|&c| c == ' ' || c == '\t')
}

fn scan_number(text: &[char], i: usize) -> usize {
    let n = text.len();
    let mut j = i;
    if text[j] == '0' && j + 1 < n && "xXoObB".contains(text[j + 1]) {
        j += 2;
        // Python's str.isalnum, which is Unicode-wide.
        while j < n && (is_py_alnum(text[j]) || text[j] == '_') {
            j += 1;
        }
        return j;
    }
    while j < n && (text[j].is_ascii_digit() || text[j] == '_') {
        j += 1;
    }
    if j < n && text[j] == '.' {
        j += 1;
        while j < n && (text[j].is_ascii_digit() || text[j] == '_') {
            j += 1;
        }
    }
    let signed_digit =
        |k: usize| k < n && (text[k] == '+' || text[k] == '-') && digit_at(text, k + 1);
    if j < n && (text[j] == 'e' || text[j] == 'E') && (digit_at(text, j + 1) || signed_digit(j + 1))
    {
        j += 1;
        if text[j] == '+' || text[j] == '-' {
            j += 1;
        }
        while digit_at(text, j) {
            j += 1;
        }
    }
    if j < n && (text[j] == 'j' || text[j] == 'J') {
        j += 1;
    }
    j
}

fn scan_decorator(text: &[char], i: usize) -> usize {
    let mut j = i + 1;
    while j < text.len() && (is_ident(text[j]) || text[j] == '.') {
        j += 1;
    }
    j
}

/// Whether `i` could open something other than plain text, so a run of
/// plain text is emitted as one chunk.
fn is_token_start(text: &[char], i: usize) -> bool {
    let ch = text[i];
    matches!(ch, '#' | '\'' | '"' | '@') || is_ident(ch) || (ch == '.' && digit_at(text, i + 1))
}
