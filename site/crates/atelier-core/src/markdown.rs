//! The Markdown subset the site renders, ported from
//! `scripts/content_pipeline.py`. Hand-rolled and NOT CommonMark: the output,
//! and the graph built from it, depend on exactly this subset, so unsupported
//! syntax degrades to a paragraph the way the Python did.
//!
//! Blocks: `##`/`###` headings with deduplicated ids, fenced code (language
//! discarded, never highlighted), the `<EXTREMELY-IMPORTANT>` callout, pipe
//! tables, `-`/`N.` lists, paragraphs. Inline: `` `code` ``, `**bold**`,
//! `*italic*`, `[text](href)`, and skill cross-references.
//!
//! Python's regexes are replaced by hand scans (the `regex` crate has no
//! lookarounds; each scan names the Python pattern it replaces). Python's
//! character classes are Unicode-wide, and they are reproduced exactly, as
//! the Python 3.14 oracle (Unicode 16) defines them:
//!
//! - `\s`, `str.strip()` and `str.split()` are `str.isspace()`: `is_space`
//!   (it has U+001C..U+001F, which `char::is_whitespace` lacks).
//! - `\w` is `str.isalnum()` or `_`: `is_py_alnum` is
//!   `char::is_alphanumeric` minus the code points Python does not count
//!   (the `Other_Alphabetic` combining marks such as Devanagari vowel signs,
//!   circled letters, and letters assigned after Unicode 16).
//! - `\d` is `str.isdecimal()`, Unicode `Nd`: `is_py_decimal`.
//! - Lengths and positions in [`summarize_description`] are code points.
//!
//! The two tables at the bottom were generated from Python 3.14.8 against
//! this toolchain's `char::is_alphanumeric`. Like the highlighter's keyword
//! list they are frozen: a toolchain with newer Unicode tables may need the
//! exclusion table regenerated.

use std::collections::{BTreeMap, BTreeSet};

use crate::escape::{html_attr, html_text_pipeline as escape};
use crate::model::humanize_filename;

/// The card teaser's budget in code points (content_pipeline.py
/// `SUMMARY_MAX_LENGTH`).
pub const SUMMARY_MAX_LENGTH: usize = 140;

const CALLOUT_LABEL: &str = "Extremely important";

/// What one render call needs to know: the arguments content_pipeline's
/// renderers take, with its two resolver callbacks spelled out as data.
#[derive(Debug, Clone, Copy)]
pub struct RenderCtx<'a> {
    /// Every skill that exists. A cross-reference links only to one of these.
    pub known_slugs: &'a BTreeSet<String>,
    /// The site's base path (`/tbaguette-skills`, or empty at a domain
    /// root); skill links are `{base_path}/skills/{slug}/`, verbatim, so a
    /// trailing slash here doubles.
    pub base_path: &'a str,
    /// Whether skill cross-references are linked at all (Python: whether a
    /// `resolve_skill_link` was passed).
    pub link_slugs: bool,
    /// The page's own slug, which is never linked.
    pub current_slug: Option<&'a str>,
    /// For relative `.md` links: filename stem to anchor id (Python
    /// `make_reference_link_resolver`). `None`, or a stem missing from it,
    /// drops the href and keeps the text.
    pub relative_anchors: Option<&'a BTreeMap<String, String>>,
    /// Prepended to every heading id [`render`] mints. [`render_reference`]
    /// supplies its own and ignores this.
    pub heading_id_prefix: &'a str,
}

/// One rendered reference file (content_pipeline.py
/// `render_reference_subdocument`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSection {
    /// `{anchor_prefix}-{file stem}`: `ref-lenses`, `stack-web`, `cmd-color`.
    pub id: String,
    /// The file stem humanized, not the file's own H1.
    pub title: String,
    pub html: String,
}

// ---------------------------------------------------------------------------
// Link resolution
// ---------------------------------------------------------------------------

/// The two Python resolver callbacks.
#[derive(Clone, Copy)]
struct Links<'a> {
    /// `resolve_skill_link`, or `None` where Python passed none (and inside a
    /// link's text, which is always rendered without one).
    skills: Option<SkillLinks<'a>>,
    relative: Relative<'a>,
}

#[derive(Clone, Copy)]
struct SkillLinks<'a> {
    known: &'a BTreeSet<String>,
    base_path: &'a str,
    current: Option<&'a str>,
}

impl SkillLinks<'_> {
    /// content_pipeline.py `make_skill_mention_resolver` over
    /// templates.py `skill_url`.
    fn resolve(&self, slug: &str) -> Option<String> {
        if self.current == Some(slug) || !self.known.contains(slug) {
            return None;
        }
        Some(format!("{}/skills/{}/", self.base_path, html_attr(slug)))
    }
}

#[derive(Clone, Copy)]
struct Relative<'a> {
    anchors: Option<&'a BTreeMap<String, String>>,
    /// Inside a reference file, `#fragment` points at that file's own
    /// headings, which carry the section's prefix.
    section_id: Option<&'a str>,
}

impl Relative<'_> {
    fn resolve(&self, href: &str) -> Option<String> {
        if let (Some(section_id), Some(fragment)) = (self.section_id, href.strip_prefix('#')) {
            return Some(format!("#{section_id}-{}", slugify(fragment)));
        }
        let anchor = self.anchors?.get(path_stem(href))?;
        (!anchor.is_empty()).then(|| format!("#{anchor}"))
    }
}

impl<'a> Links<'a> {
    fn new(ctx: &RenderCtx<'a>, section_id: Option<&'a str>) -> Self {
        Links {
            skills: ctx.link_slugs.then_some(SkillLinks {
                known: ctx.known_slugs,
                base_path: ctx.base_path,
                current: ctx.current_slug,
            }),
            relative: Relative {
                anchors: ctx.relative_anchors,
                section_id,
            },
        }
    }
}

/// Python 3.14 `PurePosixPath(href).stem`: the last component that is not
/// empty or `.`, minus its last suffix, unless what remains is only dots.
fn path_stem(href: &str) -> &str {
    let name = href
        .split('/')
        .rev()
        .find(|c| !c.is_empty() && *c != ".")
        .unwrap_or("");
    match name.rfind('.') {
        Some(i) if !name[..i].trim_start_matches('.').is_empty() => &name[..i],
        _ => name,
    }
}

// ---------------------------------------------------------------------------
// Character classes and small text helpers
// ---------------------------------------------------------------------------

/// Python's `str.isspace()`, which is also what `\s` and `str.strip()` use.
fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r' | '\u{1c}'..='\u{1f}' | ' ' | '\u{85}' | '\u{a0}'
            | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
            | '\u{205f}' | '\u{3000}'
    )
}

/// Python's `\w`.
fn is_word(c: char) -> bool {
    c == '_' || is_py_alnum(c)
}

fn in_ranges(c: char, ranges: &[(u32, u32)]) -> bool {
    let c = u32::from(c);
    ranges
        .binary_search_by(|&(lo, hi)| {
            if hi < c {
                std::cmp::Ordering::Less
            } else if lo > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Python's `str.isalnum()` for one character.
pub(crate) fn is_py_alnum(c: char) -> bool {
    c.is_alphanumeric() && !in_ranges(c, NOT_PY_ALNUM)
}

/// Python's `str.isdecimal()` for one character, which is also `\d`.
fn is_py_decimal(c: char) -> bool {
    c.is_ascii_digit() || (!c.is_ascii() && in_ranges(c, PY_DECIMAL))
}

fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

fn is_blank(line: &str) -> bool {
    strip(line).is_empty()
}

fn char_len_at(text: &str, i: usize) -> usize {
    text[i..].chars().next().map_or(1, char::len_utf8)
}

/// content_pipeline.py `slugify`: kebab-case for an HTML id. ASCII-only, so
/// a heading with no ASCII letters or digits becomes `section`.
pub fn slugify(text: &str) -> String {
    let lowered = text.to_lowercase().replace(['\'', '’'], "");
    let mut slug = String::with_capacity(lowered.len());
    let mut in_gap = false;
    for c in lowered.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if in_gap && !slug.is_empty() {
                slug.push('-');
            }
            in_gap = false;
            slug.push(c);
        } else {
            in_gap = true;
        }
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

/// content_pipeline.py `strip_title_heading`: drops the first non-blank line
/// when it starts (after indentation) with `# `.
pub fn strip_title_heading(body: &str) -> &str {
    let mut offset = 0;
    for line in body.split('\n') {
        let next = offset + line.len() + 1;
        if is_blank(line) {
            offset = next;
            continue;
        }
        if line.trim_start_matches(is_space).starts_with("# ") {
            return body.get(next..).unwrap_or("");
        }
        break;
    }
    body
}

/// content_pipeline.py `summarize_description`: the first sentence, cut at a
/// clause mark or a space with an ellipsis when it exceeds `max_length` code
/// points. `max_length` must be at least 1 (the build passes 140).
pub fn summarize_description(description: &str, max_length: usize) -> String {
    const CLAUSE: &[char] = &[',', '，', '、', '；', '،', '؛'];
    const RSTRIP: &[char] = &[' ', ',', ';', ':', '，', '、', '；', '،', '؛', '：'];

    let normalized = description
        .split(is_space)
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>();
    let chars: Vec<char> = normalized.join(" ").chars().collect();

    // `^.*?(?:[.!?](?=\s|$)|[。！？؟])` (_SENTENCE_START_RE): the earliest
    // sentence end. After normalization the only space left is ' '.
    let end = (0..chars.len()).find(|&k| match chars[k] {
        '.' | '!' | '?' => chars.get(k + 1).is_none_or(|&c| is_space(c)),
        '。' | '！' | '？' | '؟' => true,
        _ => false,
    });
    let first = &chars[..end.map_or(chars.len(), |k| k + 1)];
    if first.len() <= max_length {
        return first.iter().collect();
    }

    let budget = max_length - 1; // one character for the ellipsis
    let truncated = &first[..budget];
    let comma_cut = truncated.iter().rposition(|c| CLAUSE.contains(c));
    let space_cut = truncated.iter().rposition(|&c| c == ' ');
    let clipped = match (comma_cut, space_cut) {
        (Some(cut), _) if cut as f64 > budget as f64 * 0.3 => &first[..cut],
        (_, Some(cut)) if cut > 0 => &first[..cut],
        _ => truncated,
    };
    let kept = clipped
        .iter()
        .rposition(|c| !RSTRIP.contains(c))
        .map_or(0, |k| k + 1);
    let mut out: String = clipped[..kept].iter().collect();
    out.push('…');
    out
}

// ---------------------------------------------------------------------------
// Inline
// ---------------------------------------------------------------------------

enum Inline<'t> {
    Code(&'t str),
    Bold(&'t str),
    Italic(&'t str),
    Link { text: &'t str, href: &'t str },
}

/// The `_INLINE_TOKEN_RE` alternation tried at byte `i`, in its order:
/// `` `([^`]+)` ``, `\*\*([^*]+)\*\*`, `\*([^*]+)\*`,
/// `\[([^\]]*)\]\(([^)]*)\)`. Each class excludes its own terminator, so the
/// greedy match is the only one and backtracking never changes it. Returns
/// the token and the byte index just past it.
fn inline_token_at(text: &str, i: usize) -> Option<(Inline<'_>, usize)> {
    let bytes = text.as_bytes();
    let next = |from: usize, c: char| text.get(from..)?.find(c).map(|p| from + p);
    match bytes[i] {
        b'`' => {
            let close = next(i + 1, '`')?;
            (close > i + 1).then(|| (Inline::Code(&text[i + 1..close]), close + 1))
        }
        b'*' => {
            if bytes.get(i + 1) == Some(&b'*')
                && let Some(close) = next(i + 2, '*')
                && close > i + 2
                && bytes.get(close + 1) == Some(&b'*')
            {
                return Some((Inline::Bold(&text[i + 2..close]), close + 2));
            }
            let close = next(i + 1, '*')?;
            (close > i + 1).then(|| (Inline::Italic(&text[i + 1..close]), close + 1))
        }
        b'[' => {
            let bracket = next(i + 1, ']')?;
            if bytes.get(bracket + 1) != Some(&b'(') {
                return None;
            }
            let paren = next(bracket + 2, ')')?;
            let token = Inline::Link {
                text: &text[i + 1..bracket],
                href: &text[bracket + 2..paren],
            };
            Some((token, paren + 1))
        }
        _ => None,
    }
}

/// content_pipeline.py `render_inline_markdown` with the resolvers in `ctx`
/// (`heading_id_prefix` is unused here).
pub fn render_inline(text: &str, ctx: &RenderCtx) -> String {
    inline(text, Links::new(ctx, None))
}

fn inline(text: &str, links: Links) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    let mut cursor = 0;
    let mut i = 0;
    while i < text.len() {
        let Some((token, end)) = inline_token_at(text, i) else {
            i += char_len_at(text, i);
            continue;
        };
        if i > cursor {
            out.push_str(&plain_run(&text[cursor..i], links.skills));
        }
        match token {
            Inline::Code(code) => out.push_str(&code_span(code, links.skills)),
            Inline::Bold(inner) => {
                out.push_str("<strong>");
                out.push_str(&inline(inner, links));
                out.push_str("</strong>");
            }
            Inline::Italic(inner) => {
                out.push_str("<em>");
                out.push_str(&inline(inner, links));
                out.push_str("</em>");
            }
            Inline::Link { text, href } => out.push_str(&markdown_link(text, href, links.relative)),
        }
        cursor = end;
        i = end;
    }
    if cursor < text.len() {
        out.push_str(&plain_run(&text[cursor..], links.skills));
    }
    out
}

/// A slug-shaped token starting at byte `i`, returning its end: hyphen-joined
/// runs of `part` characters, at least `min_hyphens` hyphens, not touching a
/// word character, `/` or `-` on either side. Replaces the lookarounds of
/// `_BARE_SLUG_RE` (`(?<![\w/-])([a-z0-9]+(?:-[a-z0-9]+)+)(?![\w/-])`) and
/// `_TITLE_WORD_RE` (`[A-Za-z0-9]` and `*` in place of `+`). Only the
/// maximal greedy run can pass the look-ahead: any shorter match ends before
/// a part character or a hyphen, both of which it rejects.
fn slug_shaped_at(text: &str, i: usize, part: fn(u8) -> bool, min_hyphens: usize) -> Option<usize> {
    let guard = |c: char| is_word(c) || c == '/' || c == '-';
    if text[..i].chars().next_back().is_some_and(guard) {
        return None;
    }
    let bytes = text.as_bytes();
    let run = |from: usize| from + bytes[from..].iter().take_while(|&&b| part(b)).count();
    let mut end = run(i);
    if end == i {
        return None;
    }
    let mut hyphens = 0;
    while bytes.get(end) == Some(&b'-') && bytes.get(end + 1).is_some_and(|&b| part(b)) {
        end = run(end + 1);
        hyphens += 1;
    }
    if hyphens < min_hyphens || text[end..].chars().next().is_some_and(guard) {
        return None;
    }
    Some(end)
}

fn bare_slug_part(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit()
}

fn title_word_part(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

fn bare_link(href: &str, label: &str) -> String {
    format!(
        "<a class=\"skill-link skill-link--bare\" href=\"{}\">{}</a>",
        escape(href),
        escape(label)
    )
}

/// content_pipeline.py `_render_plain_run`: escapes a run between inline
/// tokens, linking bare slugs the resolver knows.
fn plain_run(text: &str, skills: Option<SkillLinks>) -> String {
    let Some(skills) = skills else {
        return escape(text);
    };
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut i = 0;
    while i < text.len() {
        let Some(end) = slug_shaped_at(text, i, bare_slug_part, 1) else {
            i += char_len_at(text, i);
            continue;
        };
        if let Some(href) = skills.resolve(&text[i..end]) {
            out.push_str(&escape(&text[cursor..i]));
            out.push_str(&bare_link(&href, &text[i..end]));
            cursor = end;
        }
        i = end;
    }
    out.push_str(&escape(&text[cursor..]));
    out
}

/// content_pipeline.py `_render_code_span`: a code span whose whole content
/// is a known slug becomes a link around the `<code>`.
fn code_span(code: &str, skills: Option<SkillLinks>) -> String {
    let rendered = format!("<code>{}</code>", escape(code));
    match skills.and_then(|s| s.resolve(code)) {
        Some(href) => format!(
            "<a class=\"skill-link\" href=\"{}\">{rendered}</a>",
            escape(&href)
        ),
        None => rendered,
    }
}

/// content_pipeline.py `_render_markdown_link`. The text is rendered without
/// skill links; an unresolvable relative href is dropped, keeping the text.
fn markdown_link(text: &str, href: &str, relative: Relative) -> String {
    let rendered = inline(
        text,
        Links {
            skills: None,
            relative,
        },
    );
    let target = if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        match relative.resolve(href) {
            Some(resolved) => resolved,
            None => return rendered,
        }
    };
    format!("<a href=\"{}\">{rendered}</a>", escape(&target))
}

/// content_pipeline.py `render_note_title`: escapes an update-note title,
/// linking (bare, once each) the words that name a skill. Looser than body
/// prose: a single word links, case-insensitively.
pub fn render_note_title(title: &str, ctx: &RenderCtx) -> String {
    let Some(skills) = Links::new(ctx, None).skills else {
        return escape(title);
    };
    let mut out = String::with_capacity(title.len());
    let mut linked = BTreeSet::new();
    let mut cursor = 0;
    let mut i = 0;
    while i < title.len() {
        let Some(end) = slug_shaped_at(title, i, title_word_part, 0) else {
            i += char_len_at(title, i);
            continue;
        };
        let word = &title[i..end];
        let slug = word.to_ascii_lowercase();
        if !linked.contains(&slug)
            && let Some(href) = skills.resolve(&slug)
        {
            out.push_str(&escape(&title[cursor..i]));
            out.push_str(&bare_link(&href, word));
            linked.insert(slug);
            cursor = end;
        }
        i = end;
    }
    out.push_str(&escape(&title[cursor..]));
    out
}

// ---------------------------------------------------------------------------
// Block line classifiers (the module-level regexes of content_pipeline.py)
// ---------------------------------------------------------------------------

/// `^(#{2,3})\s+(.*)$` (_HEADING_RE): the level and the stripped text.
fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|&b| b == b'#').count();
    let rest = &line[level..];
    let ok = (level == 2 || level == 3) && rest.chars().next().is_some_and(is_space);
    ok.then(|| (level, strip(rest)))
}

/// `^```` matched against the stripped line (_FENCE_RE).
fn is_fence(line: &str) -> bool {
    strip(line).starts_with("```")
}

/// `^(\s*)(-|\d+\.)\s+(.*)$` (_LIST_ITEM_RE): indent in code points, whether
/// the marker is a number, and the stripped text.
fn list_item(line: &str) -> Option<(usize, bool, &str)> {
    let rest = line.trim_start_matches(is_space);
    let indent = line[..line.len() - rest.len()].chars().count();
    let digits: usize = rest
        .chars()
        .take_while(|&c| is_py_decimal(c))
        .map(char::len_utf8)
        .sum();
    let marker_len = if rest.starts_with('-') {
        1
    } else if digits > 0 && rest.as_bytes().get(digits) == Some(&b'.') {
        digits + 1
    } else {
        return None;
    };
    let after = &rest[marker_len..];
    after
        .chars()
        .next()
        .is_some_and(is_space)
        .then(|| (indent, digits > 0, strip(after)))
}

/// `^\s*\|.*\|\s*$` (_TABLE_ROW_RE).
fn is_table_row(line: &str) -> bool {
    let s = strip(line);
    s.len() >= 2 && s.starts_with('|') && s.ends_with('|')
}

/// `^\s*\|[\s:|-]+\|\s*$` (_TABLE_SEPARATOR_RE).
fn is_table_separator(line: &str) -> bool {
    let s = strip(line);
    s.len() >= 3
        && s.starts_with('|')
        && s.ends_with('|')
        && s[1..s.len() - 1]
            .chars()
            .all(|c| is_space(c) || matches!(c, ':' | '|' | '-'))
}

/// `^\s*<(/?)EXTREMELY[-_ ]IMPORTANT>\s*$` with re.IGNORECASE
/// (_CALLOUT_OPEN_RE / _CALLOUT_CLOSE_RE). Python's case-insensitive
/// matching also lets `I` match U+0130 and U+0131; nothing else beyond
/// ASCII case.
fn is_callout_tag(line: &str, closing: bool) -> bool {
    let s = strip(line);
    let Some(s) = s.strip_prefix(if closing { "</" } else { "<" }) else {
        return false;
    };
    // `~` stands for the separator class.
    const PATTERN: &str = "EXTREMELY~IMPORTANT>";
    s.chars().count() == PATTERN.len()
        && s.chars().zip(PATTERN.chars()).all(|(c, p)| match p {
            '~' => matches!(c, '-' | '_' | ' '),
            'I' => matches!(c, 'I' | 'i' | '\u{130}' | '\u{131}'),
            _ => c.to_ascii_uppercase() == p,
        })
}

fn is_table_start(lines: &[&str], index: usize) -> bool {
    is_table_row(lines[index]) && lines.get(index + 1).is_some_and(|l| is_table_separator(l))
}

/// content_pipeline.py `_is_block_start`: where a paragraph stops without a
/// blank line.
fn is_block_start(lines: &[&str], index: usize) -> bool {
    let line = lines[index];
    heading(line).is_some()
        || is_fence(line)
        || is_callout_tag(line, false)
        || is_callout_tag(line, true)
        || is_table_start(lines, index)
        || list_item(line).is_some()
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

/// content_pipeline.py `render_markdown_body`: a Markdown body (its leading
/// `# Title` already stripped) to HTML, blocks joined by `\n`. Heading ids get
/// `ctx.heading_id_prefix` and are deduplicated within the call.
pub fn render(md: &str, ctx: &RenderCtx) -> String {
    body(md, ctx.heading_id_prefix, Links::new(ctx, None))
}

/// content_pipeline.py `render_reference_subdocument` over the text of
/// `reference/<file_stem>.md` (or `reference/stacks/…`), as the build read
/// it. `anchor_prefix` is `ref`, `stack` or `cmd`. Heading ids are prefixed
/// with the section id, and `#fragment` links resolve to them.
pub fn render_reference(
    file_stem: &str,
    anchor_prefix: &str,
    text: &str,
    ctx: &RenderCtx,
) -> ReferenceSection {
    let id = format!("{anchor_prefix}-{file_stem}");
    let html = body(
        strip_title_heading(text),
        &format!("{id}-"),
        Links::new(ctx, Some(&id)),
    );
    ReferenceSection {
        title: humanize_filename(file_stem),
        html,
        id,
    }
}

fn body(md: &str, prefix: &str, links: Links) -> String {
    let lines: Vec<&str> = md.split('\n').collect();
    let mut used_ids = BTreeSet::new();
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        if is_blank(line) {
            index += 1;
            continue;
        }
        let (html, next) = if let Some((level, text)) = heading(line) {
            let id = dedupe_id(
                format!("{prefix}{}", slugify(&text.replace(['`', '*'], ""))),
                &mut used_ids,
            );
            (
                format!("<h{level} id=\"{id}\">{}</h{level}>", inline(text, links)),
                index + 1,
            )
        } else if is_callout_tag(line, false) {
            callout(&lines, index, prefix, links)
        } else if is_fence(line) {
            fenced_code(&lines, index)
        } else if is_table_start(&lines, index) {
            table(&lines, index, links)
        } else if list_item(line).is_some() {
            list(&lines, index, links)
        } else {
            paragraph(&lines, index, links)
        };
        blocks.push(html);
        index = next;
    }
    blocks.join("\n")
}

fn dedupe_id(candidate: String, used: &mut BTreeSet<String>) -> String {
    let mut unique = candidate.clone();
    let mut suffix = 2;
    while used.contains(&unique) {
        unique = format!("{candidate}-{suffix}");
        suffix += 1;
    }
    used.insert(unique.clone());
    unique
}

/// Up to the next line starting with ```` ``` ```` (any text after it, so a
/// "language" on a closing line still closes), or to the end of the body.
fn fenced_code(lines: &[&str], index: usize) -> (String, usize) {
    let close = (index + 1..lines.len())
        .find(|&i| is_fence(lines[i]))
        .unwrap_or(lines.len());
    let code = lines[index + 1..close].join("\n");
    let html = format!(
        "<pre class=\"prose-code-block\"><code>{}</code></pre>",
        escape(&code)
    );
    (html, (close + 1).min(lines.len()))
}

/// The inner lines are rendered as their own body, with a fresh id set and
/// the prefix `{prefix}callout-{opening line index}-`. Unclosed runs to the
/// end of the body.
fn callout(lines: &[&str], index: usize, prefix: &str, links: Links) -> (String, usize) {
    let close = (index + 1..lines.len())
        .find(|&i| is_callout_tag(lines[i], true))
        .unwrap_or(lines.len());
    let inner = body(
        &lines[index + 1..close].join("\n"),
        &format!("{prefix}callout-{index}-"),
        links,
    );
    let html = format!(
        "<div class=\"prose-callout\"><p class=\"prose-callout__label\">{}</p>\
         <div class=\"prose-callout__body\">{inner}</div></div>",
        escape(CALLOUT_LABEL)
    );
    (html, (close + 1).min(lines.len()))
}

/// Header row, separator (skipped; no alignment support), then rows until
/// the first line that is not a row.
fn table(lines: &[&str], index: usize, links: Links) -> (String, usize) {
    let mut html = String::from("<table><thead><tr>");
    for cell in split_table_row(lines[index]) {
        html.push_str(&format!("<th>{}</th>", inline(&cell, links)));
    }
    html.push_str("</tr></thead><tbody>");
    let mut row = index + 2;
    while row < lines.len() && is_table_row(lines[row]) {
        html.push_str("<tr>");
        for cell in split_table_row(lines[row]) {
            html.push_str(&format!("<td>{}</td>", inline(&cell, links)));
        }
        html.push_str("</tr>");
        row += 1;
    }
    html.push_str("</tbody></table>");
    (html, row)
}

/// Cells of `| a | b |`, splitting on `|` not preceded by a backslash
/// (`(?<!\\)\|`, _UNESCAPED_PIPE_RE) and turning `\|` into `|`.
fn split_table_row(line: &str) -> Vec<String> {
    let trimmed = strip(line);
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
    let bytes = trimmed.as_bytes();
    let mut cells = Vec::new();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'|' && (i == 0 || bytes[i - 1] != b'\\') {
            cells.push(&trimmed[start..i]);
            start = i + 1;
        }
    }
    cells.push(&trimmed[start..]);
    cells
        .into_iter()
        .map(|cell| strip(cell).replace("\\|", "|"))
        .collect()
}

struct ListEntry {
    indent: usize,
    ordered: bool,
    text: String,
}

/// Item lines until a blank line, heading, fence or table; an unmarked line
/// folds into the previous item with one space.
fn list(lines: &[&str], index: usize, links: Links) -> (String, usize) {
    let mut entries: Vec<ListEntry> = Vec::new();
    let mut cursor = index;
    while cursor < lines.len() {
        let line = lines[cursor];
        if is_blank(line)
            || heading(line).is_some()
            || is_fence(line)
            || is_table_start(lines, cursor)
        {
            break;
        }
        if let Some((indent, ordered, text)) = list_item(line) {
            entries.push(ListEntry {
                indent,
                ordered,
                text: text.to_string(),
            });
        } else if let Some(last) = entries.last_mut() {
            last.text.push(' ');
            last.text.push_str(strip(line));
        } else {
            break;
        }
        cursor += 1;
    }
    (list_entries(&entries, 0, links).0, cursor)
}

/// content_pipeline.py `_render_list_entries`: the group at
/// `entries[start]`'s indent, its tag decided by that first entry; each item
/// takes at most one deeper group. Stops at the first entry at another
/// indent, so entries left over at the top level are dropped, as in Python.
fn list_entries(entries: &[ListEntry], start: usize, links: Links) -> (String, usize) {
    let base = entries[start].indent;
    let tag = if entries[start].ordered { "ol" } else { "ul" };
    let mut html = format!("<{tag}>");
    let mut cursor = start;
    while cursor < entries.len() && entries[cursor].indent == base {
        let mut item = inline(&entries[cursor].text, links);
        cursor += 1;
        if cursor < entries.len() && entries[cursor].indent > base {
            let (nested, next) = list_entries(entries, cursor, links);
            item.push_str(&nested);
            cursor = next;
        }
        html.push_str(&format!("<li>{item}</li>"));
    }
    html.push_str(&format!("</{tag}>"));
    (html, cursor)
}

fn paragraph(lines: &[&str], index: usize, links: Links) -> (String, usize) {
    let mut parts = vec![strip(lines[index])];
    let mut cursor = index + 1;
    while cursor < lines.len() && !is_blank(lines[cursor]) && !is_block_start(lines, cursor) {
        parts.push(strip(lines[cursor]));
        cursor += 1;
    }
    (
        format!("<p>{}</p>", inline(&parts.join(" "), links)),
        cursor,
    )
}

// ---------------------------------------------------------------------------
// Python character-class tables (generated; see the module docs)
// ---------------------------------------------------------------------------

/// Inclusive ranges where `char::is_alphanumeric` is true and Python 3.14's
/// `str.isalnum()` is false.
#[rustfmt::skip]
const NOT_PY_ALNUM: &[(u32, u32)] = &[
    (0x345, 0x345), (0x363, 0x36F), (0x5B0, 0x5BD), (0x5BF, 0x5BF), (0x5C1, 0x5C2), (0x5C4, 0x5C5),
    (0x5C7, 0x5C7), (0x610, 0x61A), (0x64B, 0x657), (0x659, 0x65F), (0x670, 0x670), (0x6D6, 0x6DC),
    (0x6E1, 0x6E4), (0x6E7, 0x6E8), (0x6ED, 0x6ED), (0x711, 0x711), (0x730, 0x73F), (0x7A6, 0x7B0),
    (0x816, 0x817), (0x81B, 0x823), (0x825, 0x827), (0x829, 0x82C), (0x88F, 0x88F), (0x897, 0x897),
    (0x8D4, 0x8DF), (0x8E3, 0x8E9), (0x8F0, 0x903), (0x93A, 0x93B), (0x93E, 0x94C), (0x94E, 0x94F),
    (0x955, 0x957), (0x962, 0x963), (0x981, 0x983), (0x9BE, 0x9C4), (0x9C7, 0x9C8), (0x9CB, 0x9CC),
    (0x9D7, 0x9D7), (0x9E2, 0x9E3), (0xA01, 0xA03), (0xA3E, 0xA42), (0xA47, 0xA48), (0xA4B, 0xA4C),
    (0xA51, 0xA51), (0xA70, 0xA71), (0xA75, 0xA75), (0xA81, 0xA83), (0xABE, 0xAC5), (0xAC7, 0xAC9),
    (0xACB, 0xACC), (0xAE2, 0xAE3), (0xAFA, 0xAFC), (0xB01, 0xB03), (0xB3E, 0xB44), (0xB47, 0xB48),
    (0xB4B, 0xB4C), (0xB56, 0xB57), (0xB62, 0xB63), (0xB82, 0xB82), (0xBBE, 0xBC2), (0xBC6, 0xBC8),
    (0xBCA, 0xBCC), (0xBD7, 0xBD7), (0xC00, 0xC04), (0xC3E, 0xC44), (0xC46, 0xC48), (0xC4A, 0xC4C),
    (0xC55, 0xC56), (0xC5C, 0xC5C), (0xC62, 0xC63), (0xC81, 0xC83), (0xCBE, 0xCC4), (0xCC6, 0xCC8),
    (0xCCA, 0xCCC), (0xCD5, 0xCD6), (0xCDC, 0xCDC), (0xCE2, 0xCE3), (0xCF3, 0xCF3), (0xD00, 0xD03),
    (0xD3E, 0xD44), (0xD46, 0xD48), (0xD4A, 0xD4C), (0xD57, 0xD57), (0xD62, 0xD63), (0xD81, 0xD83),
    (0xDCF, 0xDD4), (0xDD6, 0xDD6), (0xDD8, 0xDDF), (0xDF2, 0xDF3), (0xE31, 0xE31), (0xE34, 0xE3A),
    (0xE4D, 0xE4D), (0xEB1, 0xEB1), (0xEB4, 0xEB9), (0xEBB, 0xEBC), (0xECD, 0xECD), (0xF71, 0xF83),
    (0xF8D, 0xF97), (0xF99, 0xFBC), (0x102B, 0x1036), (0x1038, 0x1038), (0x103B, 0x103E),
    (0x1056, 0x1059), (0x105E, 0x1060), (0x1062, 0x1064), (0x1067, 0x106D), (0x1071, 0x1074),
    (0x1082, 0x108D), (0x108F, 0x108F), (0x109A, 0x109D), (0x1712, 0x1713), (0x1732, 0x1733),
    (0x1752, 0x1753), (0x1772, 0x1773), (0x17B6, 0x17C8), (0x1885, 0x1886), (0x18A9, 0x18A9),
    (0x1920, 0x192B), (0x1930, 0x1938), (0x1A17, 0x1A1B), (0x1A55, 0x1A5E), (0x1A61, 0x1A74),
    (0x1ABF, 0x1AC0), (0x1ACC, 0x1ACE), (0x1B00, 0x1B04), (0x1B35, 0x1B43), (0x1B80, 0x1B82),
    (0x1BA1, 0x1BA9), (0x1BAC, 0x1BAD), (0x1BE7, 0x1BF1), (0x1C24, 0x1C36), (0x1DD3, 0x1DF4),
    (0x24B6, 0x24E9), (0x2DE0, 0x2DFF), (0xA674, 0xA67B), (0xA69E, 0xA69F), (0xA7CE, 0xA7CF),
    (0xA7D2, 0xA7D2), (0xA7D4, 0xA7D4), (0xA7F1, 0xA7F1), (0xA802, 0xA802), (0xA80B, 0xA80B),
    (0xA823, 0xA827), (0xA880, 0xA881), (0xA8B4, 0xA8C3), (0xA8C5, 0xA8C5), (0xA8FF, 0xA8FF),
    (0xA926, 0xA92A), (0xA947, 0xA952), (0xA980, 0xA983), (0xA9B4, 0xA9BF), (0xA9E5, 0xA9E5),
    (0xAA29, 0xAA36), (0xAA43, 0xAA43), (0xAA4C, 0xAA4D), (0xAA7B, 0xAA7D), (0xAAB0, 0xAAB0),
    (0xAAB2, 0xAAB4), (0xAAB7, 0xAAB8), (0xAABE, 0xAABE), (0xAAEB, 0xAAEF), (0xAAF5, 0xAAF5),
    (0xABE3, 0xABEA), (0xFB1E, 0xFB1E), (0x10376, 0x1037A), (0x10940, 0x10959), (0x10A01, 0x10A03),
    (0x10A05, 0x10A06), (0x10A0C, 0x10A0F), (0x10D24, 0x10D27), (0x10D69, 0x10D69),
    (0x10EAB, 0x10EAC), (0x10EC5, 0x10EC7), (0x10EFA, 0x10EFC), (0x11000, 0x11002),
    (0x11038, 0x11045), (0x11073, 0x11074), (0x11080, 0x11082), (0x110B0, 0x110B8),
    (0x110C2, 0x110C2), (0x11100, 0x11102), (0x11127, 0x11132), (0x11145, 0x11146),
    (0x11180, 0x11182), (0x111B3, 0x111BF), (0x111CE, 0x111CF), (0x1122C, 0x11234),
    (0x11237, 0x11237), (0x1123E, 0x1123E), (0x11241, 0x11241), (0x112DF, 0x112E8),
    (0x11300, 0x11303), (0x1133E, 0x11344), (0x11347, 0x11348), (0x1134B, 0x1134C),
    (0x11357, 0x11357), (0x11362, 0x11363), (0x113B8, 0x113C0), (0x113C2, 0x113C2),
    (0x113C5, 0x113C5), (0x113C7, 0x113CA), (0x113CC, 0x113CD), (0x11435, 0x11441),
    (0x11443, 0x11445), (0x114B0, 0x114C1), (0x115AF, 0x115B5), (0x115B8, 0x115BE),
    (0x115DC, 0x115DD), (0x11630, 0x1163E), (0x11640, 0x11640), (0x116AB, 0x116B5),
    (0x1171D, 0x1172A), (0x1182C, 0x11838), (0x11930, 0x11935), (0x11937, 0x11938),
    (0x1193B, 0x1193C), (0x11940, 0x11940), (0x11942, 0x11942), (0x119D1, 0x119D7),
    (0x119DA, 0x119DF), (0x119E4, 0x119E4), (0x11A01, 0x11A0A), (0x11A35, 0x11A39),
    (0x11A3B, 0x11A3E), (0x11A51, 0x11A5B), (0x11A8A, 0x11A97), (0x11B60, 0x11B67),
    (0x11C2F, 0x11C36), (0x11C38, 0x11C3E), (0x11C92, 0x11CA7), (0x11CA9, 0x11CB6),
    (0x11D31, 0x11D36), (0x11D3A, 0x11D3A), (0x11D3C, 0x11D3D), (0x11D3F, 0x11D41),
    (0x11D43, 0x11D43), (0x11D47, 0x11D47), (0x11D8A, 0x11D8E), (0x11D90, 0x11D91),
    (0x11D93, 0x11D96), (0x11DB0, 0x11DDB), (0x11DE0, 0x11DE9), (0x11EF3, 0x11EF6),
    (0x11F00, 0x11F01), (0x11F03, 0x11F03), (0x11F34, 0x11F3A), (0x11F3E, 0x11F40),
    (0x1611E, 0x1612E), (0x16EA0, 0x16EB8), (0x16EBB, 0x16ED3), (0x16F4F, 0x16F4F),
    (0x16F51, 0x16F87), (0x16F8F, 0x16F92), (0x16FF0, 0x16FF6), (0x187F8, 0x187FF),
    (0x18D09, 0x18D1E), (0x18D80, 0x18DF2), (0x1BC9E, 0x1BC9E), (0x1E000, 0x1E006),
    (0x1E008, 0x1E018), (0x1E01B, 0x1E021), (0x1E023, 0x1E024), (0x1E026, 0x1E02A),
    (0x1E08F, 0x1E08F), (0x1E6C0, 0x1E6DE), (0x1E6E0, 0x1E6F5), (0x1E6FE, 0x1E6FF),
    (0x1E947, 0x1E947), (0x1F130, 0x1F149), (0x1F150, 0x1F169), (0x1F170, 0x1F189),
    (0x2B73A, 0x2B73F), (0x2CEA2, 0x2CEAD), (0x323B0, 0x33479),
];

/// Inclusive ranges of Python 3.14's `str.isdecimal()` (Unicode 16 `Nd`).
#[rustfmt::skip]
const PY_DECIMAL: &[(u32, u32)] = &[
    (0x30, 0x39), (0x660, 0x669), (0x6F0, 0x6F9), (0x7C0, 0x7C9), (0x966, 0x96F), (0x9E6, 0x9EF),
    (0xA66, 0xA6F), (0xAE6, 0xAEF), (0xB66, 0xB6F), (0xBE6, 0xBEF), (0xC66, 0xC6F), (0xCE6, 0xCEF),
    (0xD66, 0xD6F), (0xDE6, 0xDEF), (0xE50, 0xE59), (0xED0, 0xED9), (0xF20, 0xF29),
    (0x1040, 0x1049), (0x1090, 0x1099), (0x17E0, 0x17E9), (0x1810, 0x1819), (0x1946, 0x194F),
    (0x19D0, 0x19D9), (0x1A80, 0x1A89), (0x1A90, 0x1A99), (0x1B50, 0x1B59), (0x1BB0, 0x1BB9),
    (0x1C40, 0x1C49), (0x1C50, 0x1C59), (0xA620, 0xA629), (0xA8D0, 0xA8D9), (0xA900, 0xA909),
    (0xA9D0, 0xA9D9), (0xA9F0, 0xA9F9), (0xAA50, 0xAA59), (0xABF0, 0xABF9), (0xFF10, 0xFF19),
    (0x104A0, 0x104A9), (0x10D30, 0x10D39), (0x10D40, 0x10D49), (0x11066, 0x1106F),
    (0x110F0, 0x110F9), (0x11136, 0x1113F), (0x111D0, 0x111D9), (0x112F0, 0x112F9),
    (0x11450, 0x11459), (0x114D0, 0x114D9), (0x11650, 0x11659), (0x116C0, 0x116C9),
    (0x116D0, 0x116E3), (0x11730, 0x11739), (0x118E0, 0x118E9), (0x11950, 0x11959),
    (0x11BF0, 0x11BF9), (0x11C50, 0x11C59), (0x11D50, 0x11D59), (0x11DA0, 0x11DA9),
    (0x11F50, 0x11F59), (0x16130, 0x16139), (0x16A60, 0x16A69), (0x16AC0, 0x16AC9),
    (0x16B50, 0x16B59), (0x16D70, 0x16D79), (0x1CCF0, 0x1CCF9), (0x1D7CE, 0x1D7FF),
    (0x1E140, 0x1E149), (0x1E2F0, 0x1E2F9), (0x1E4F0, 0x1E4F9), (0x1E5F1, 0x1E5FA),
    (0x1E950, 0x1E959), (0x1FBF0, 0x1FBF9),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_sorted_and_disjoint_for_binary_search() {
        for table in [NOT_PY_ALNUM, PY_DECIMAL] {
            assert!(table.iter().all(|&(lo, hi)| lo <= hi));
            assert!(table.windows(2).all(|w| w[0].1 < w[1].0));
        }
    }

    #[test]
    fn python_character_classes() {
        // A Devanagari vowel sign and a circled letter: Rust counts them as
        // alphanumeric, Python's str.isalnum() does not.
        assert!('\u{93F}'.is_alphanumeric() && !is_py_alnum('\u{93F}'));
        assert!('Ⓐ'.is_alphanumeric() && !is_py_alnum('Ⓐ'));
        assert!(is_py_alnum('é') && is_py_alnum('日') && is_py_alnum('٣'));
        assert!(is_py_decimal('7') && is_py_decimal('٣') && !is_py_decimal('½'));
        assert!(is_space('\u{1c}') && !'\u{1c}'.is_whitespace());
    }
}
