//! SKILL.md frontmatter, with the build's own semantics (content_pipeline.py
//! `split_frontmatter`), not YAML's: only single-line `key: value` pairs are
//! read, so a value wrapped onto a second line is truncated to its first.
//! `test_skill_frontmatter.py` folds continuations and the build does not;
//! the site must follow the build.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontmatterError {
    /// The first line is not `---`.
    MissingOpening,
    /// No later line is `---`.
    Unclosed,
}

impl fmt::Display for FrontmatterError {
    // The wording is the Python ValueError text, which the fixtures pin.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOpening => {
                f.write_str("expected a \"---\" frontmatter delimiter on the first line")
            }
            Self::Unclosed => f.write_str("frontmatter opened with \"---\" is never closed"),
        }
    }
}

impl std::error::Error for FrontmatterError {}

/// Python's `str.strip()`: Unicode whitespace plus the four ASCII separator
/// controls U+001C..U+001F, which `char::is_whitespace` leaves out.
fn py_strip(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}

/// Split a SKILL.md into its frontmatter fields and its body: everything
/// after the line that closes the frontmatter, untouched.
///
/// Lines are split on `\n` only; a `\r` stays on the line, as in Python.
/// A later duplicate key overwrites an earlier one.
pub fn parse(text: &str) -> Result<(BTreeMap<String, String>, &str), FrontmatterError> {
    let mut lines = text.split('\n');
    if py_strip(lines.next().unwrap_or("")) != "---" {
        return Err(FrontmatterError::MissingOpening);
    }

    let mut fields = BTreeMap::new();
    let mut consumed = text.find('\n').map_or(text.len(), |i| i + 1);
    for line in lines {
        let next = consumed + line.len() + 1;
        if py_strip(line) == "---" {
            let body = text.get(next..).unwrap_or("");
            return Ok((fields, body));
        }
        if let Some((key, value)) = field(line) {
            fields.insert(
                key.to_string(),
                strip_matching_quotes(py_strip(value)).to_string(),
            );
        }
        consumed = next;
    }
    Err(FrontmatterError::Unclosed)
}

/// `^([A-Za-z][A-Za-z0-9_-]*):[ \t]?(.*)$`, returning (key, raw value).
fn field(line: &str) -> Option<(&str, &str)> {
    let mut chars = line.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let colon = chars
        .find(|&(_, c)| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .filter(|&(_, c)| c == ':')?
        .0;
    let rest = &line[colon + 1..];
    Some((
        &line[..colon],
        rest.strip_prefix([' ', '\t']).unwrap_or(rest),
    ))
}

/// One pair of identical `'` or `"` around the whole value, counted in code
/// points as Python's `len()` does.
fn strip_matching_quotes(value: &str) -> &str {
    let mut chars = value.chars();
    match (chars.next(), chars.next_back()) {
        (Some(first @ ('\'' | '"')), Some(last)) if first == last => {
            &value[first.len_utf8()..value.len() - last.len_utf8()]
        }
        _ => value,
    }
}
