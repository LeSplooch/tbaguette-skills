//! UPDATES.md: the entry grammar and its validation, ported from
//! `parse_update_notes` in scripts/content_pipeline.py.
//!
//! The grammar is a deliberate subset of markdown: an entry is a level-2
//! heading carrying an ISO date and an optional title, and its content is a
//! flat list of `-` bullets. A bullet may wrap across lines; the continuation
//! lines are joined back with a single space. Everything above the first
//! heading is preamble and is dropped. Every way the file can be wrong is an
//! error rather than a quietly dropped entry.
//!
//! Bullets come back as raw text. Rendering them is the page renderer's job.
//!
//! Python's `re` and `str` treat a few characters differently from Rust's
//! standard library (`\x1c`-`\x1f` are whitespace and line breaks there), so
//! the helpers below follow Python rather than Rust where the two differ.

use std::fmt;

/// The build refuses an UPDATES.md past this many bytes (128 KiB).
pub const MAX_BYTES: u64 = 128 * 1024;
/// Entries `archive-updates` leaves in UPDATES.md.
pub const KEEP: usize = 30;
/// Where `archive-updates` moves the rest.
pub const ARCHIVE_DIR: &str = "updates-archive";

/// One dated entry, with its bullets as written (wrapped lines already joined).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEntry {
    pub date: String,
    pub title: String,
    pub bullets: Vec<String>,
}

/// A malformed UPDATES.md. `Display` is exactly the text Python raises; the
/// build prefixes `path: ` to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatesError {
    /// 1-based line the problem was found on; `None` for the end-of-file check.
    pub line: Option<usize>,
    pub reason: String,
}

impl fmt::Display for UpdatesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for UpdatesError {}

fn no_bullets(date: &str) -> String {
    format!(
        "the {date} entry has no bullets \u{2014} an entry with a heading and nothing under it \
         says a change shipped without saying what it was"
    )
}

/// `str.isspace`, which (unlike `char::is_whitespace`) includes `\x1c`-`\x1f`.
fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

/// `str.splitlines()`: breaks on more than `\n`, and no empty last line.
fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let breaks = matches!(
            c,
            '\n' | '\r' | '\x0b' | '\x0c' | '\x1c'..='\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}'
        );
        if !breaks {
            continue;
        }
        lines.push(&text[start..i]);
        start = i + c.len_utf8();
        if c == '\r' && matches!(chars.peek(), Some((_, '\n'))) {
            chars.next();
            start += 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

fn is_dash(c: char) -> bool {
    matches!(c, '\u{2014}' | '\u{2013}' | '-')
}

/// `^##\s+(\S+)(?:\s*[—–-]\s*(.*?))?\s*$` on a right-trimmed line: the
/// candidate date and the title (untrimmed), or `None` when it is not a
/// heading at all. `(\S+)` is greedy and backtracks, so a token with a dash in
/// it can split there ("2026-08-24—Titled" yields the whole token as date, but
/// "2026-08-24x - y" is cut at the first dash that lets the line match).
fn match_heading(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("##")?;
    let after_space = rest.trim_start_matches(is_space);
    if after_space.len() == rest.len() {
        return None; // `\s+` needs at least one whitespace char
    }
    let token_len = after_space.find(is_space).unwrap_or(after_space.len());
    if token_len == 0 {
        return None;
    }
    let ends: Vec<usize> = after_space[..token_len]
        .char_indices()
        .map(|(i, c)| i + c.len_utf8())
        .collect();
    for &k in ends.iter().rev() {
        let tail = after_space[k..].trim_start_matches(is_space);
        if let Some(c) = tail.chars().next().filter(|c| is_dash(*c)) {
            return Some((&after_space[..k], &tail[c.len_utf8()..]));
        }
        if k == token_len && tail.is_empty() {
            return Some((&after_space[..k], ""));
        }
    }
    None
}

/// `^\d{4}-\d{2}-\d{2}$` (ASCII digits only; Python's `\d` also admits other
/// scripts' digits, which no UPDATES.md has ever used).
fn is_iso_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
}

/// `^-\s+(.*)$` on a trimmed line: the bullet text, or `None`.
fn match_bullet(stripped: &str) -> Option<&str> {
    let rest = stripped.strip_prefix('-')?;
    let text = rest.trim_start_matches(is_space);
    (text.len() < rest.len()).then_some(text)
}

/// Parse UPDATES.md into entries in the order the file declares (newest
/// first). Errors carry the line number and the exact text the Python build
/// raises.
pub fn parse(text: &str) -> Result<Vec<UpdateEntry>, UpdatesError> {
    let mut entries: Vec<UpdateEntry> = Vec::new();
    let fail = |line: usize, reason: String| UpdatesError {
        line: Some(line),
        reason,
    };

    for (index, raw_line) in split_lines(text).into_iter().enumerate() {
        let number = index + 1;
        let line = raw_line.trim_end_matches(is_space);

        if let Some((date, title)) = match_heading(line) {
            if !is_iso_date(date) {
                return Err(fail(
                    number,
                    format!(
                        "update-note headings start with an ISO date \
                         (## YYYY-MM-DD \u{2014} Title), got \"{line}\""
                    ),
                ));
            }
            // Same-day entries are allowed; file order is all that orders them.
            if let Some(previous) = entries.last() {
                if date > previous.date.as_str() {
                    return Err(fail(
                        number,
                        format!(
                            "{date} is newer than the {} entry above it \u{2014} update notes run \
                             newest first, and the top entry is the one the site shows expanded",
                            previous.date
                        ),
                    ));
                }
                if previous.bullets.is_empty() {
                    return Err(fail(number, no_bullets(&previous.date)));
                }
            }
            entries.push(UpdateEntry {
                date: date.to_owned(),
                title: title.trim_matches(is_space).to_owned(),
                bullets: Vec::new(),
            });
            continue;
        }

        // Preamble, including the file's own H1 and its notes to whoever
        // appends next: never rendered.
        let Some(current) = entries.last_mut() else {
            continue;
        };

        let stripped = line.trim_start_matches(is_space);
        if stripped.is_empty() {
            continue;
        }
        if let Some(bullet) = match_bullet(stripped) {
            current
                .bullets
                .push(bullet.trim_matches(is_space).to_owned());
        } else if let Some(last) = current.bullets.last_mut() {
            // A wrapped bullet, joined with a space.
            last.push(' ');
            last.push_str(stripped);
        } else {
            return Err(fail(
                number,
                format!(
                    "prose between the {} heading and its first bullet is not rendered anywhere \
                     \u{2014} got \"{stripped}\". Entries are bullets only.",
                    current.date
                ),
            ));
        }
    }

    if let Some(last) = entries.last()
        && last.bullets.is_empty()
    {
        return Err(UpdatesError {
            line: None,
            reason: no_bullets(&last.date),
        });
    }
    Ok(entries)
}

/// The build's refusal for an UPDATES.md past [`MAX_BYTES`], or `None` when the
/// file is small enough. `path` is how the build names the file.
pub fn size_guard(path: &str, size: u64) -> Option<String> {
    (size > MAX_BYTES).then(|| {
        format!(
            "{path}: {} KiB, past the {} KiB this file is kept under. Run \
             `site/atelier archive-updates` to move everything past the newest {KEEP} \
             entries into {ARCHIVE_DIR}/, then build again.",
            size / 1024,
            MAX_BYTES / 1024
        )
    })
}
