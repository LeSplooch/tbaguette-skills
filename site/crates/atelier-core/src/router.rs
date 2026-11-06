//! `hooks/route-skills.awk` in Rust: what the landing page's card runs on a request typed
//! into it, so a visitor sees what the hook would really name. `atelier-build`'s
//! `router_agreement` test runs the two over a corpus and compares the names.
//!
//! [`Table::route`] takes the prompt as typed, writes it as the JSON string the hook is
//! handed, then follows the script's `END` block one step at a time, in its order. Most of
//! the odd corners are that order showing through: a closing quote counted against
//! [`MAX_CHARS`], a typed backslash eating the character after it, a space left at each end
//! of the text, which is why the continuation rule misses a brief with anything before it.
//!
//! Two things the script leaves to its awk stay out. `tolower` is ASCII here, as on mawk;
//! gawk in a UTF-8 locale also folds the Kelvin sign and a dotted capital I to ASCII letters.
//! `length` counts characters here, as gawk does; mawk counts bytes, which only matters for
//! non-ASCII text within a few bytes of [`MAX_CHARS`], and the card's field stops far short
//! of that. The standing-instruction flag is left out on purpose: the card names skills.

use std::collections::{BTreeSet, HashMap, HashSet};

/// STOPWORDS in `route-skills.awk`, which has no import; a test holds the two equal.
pub const STOPWORDS: &str = "a about above after again all also am an and any are as at be been before being but by can could did \
do does doing done down each even every few for from further get gets getting had has have having he \
her here hers him his how i if in into is it its itself just let lets like make makes making may me \
might more most much must my need needs no nor not now of off on once one only or other our out over \
own please same she should so some still such than that the their them then there these they this \
those through to too under until up us use used uses using very was way we were what when where which \
while who why will with without would yes yet you your";

/// Words nearly every description uses to describe its own shape, or that say nothing about
/// the work. BOILERPLATE in `route-skills.awk`.
pub const BOILERPLATE: &str = "covers skill already someone something anything nothing thing things whatever whoever instead rather \
across another whether because around first second third next last once new old real long short small \
big good bad right wrong want wanted going back keep look looks show shows give gives take takes";

/// One rare pair in common can be chance: this takes two, or a slug's own words.
pub const MIN_SCORE: u32 = 120;
pub const MAX_NAMED: usize = 2;
/// Past this it is a pasted log or a document, not a request.
pub const MAX_CHARS: usize = 4000;
/// "Yes, deploy it" answers a question; the earlier turn was routed.
pub const MIN_WORDS: usize = 4;
/// A continuation brief mentions everything its session touched. Only resuming is the request.
const CONTINUATION: &str = " background from the session that started ";

/// `route-skills.awk`'s `stem`. `word` is ASCII: the text reads only `[a-z0-9%]+`.
pub fn stem(word: &str) -> String {
    let n = word.len();
    if n > 5 && word.ends_with("ies") {
        return format!("{}y", &word[..n - 3]);
    }
    if n > 5 && word.ends_with("ing") {
        return word[..n - 3].to_string();
    }
    if n > 4 && word.ends_with("ed") {
        return word[..n - 2].to_string();
    }
    if n > 4 && word.ends_with("es") {
        return word[..n - 2].to_string();
    }
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..n - 1].to_string();
    }
    word.to_string()
}

/// What a request comes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Routed {
    /// No words at all.
    Empty,
    /// Fewer than [`MIN_WORDS`]: the hook passes it over.
    Short,
    /// The hook would name nothing, and the general reminder goes out.
    Nothing,
    /// The skills the hook would name, best first.
    Named(Vec<String>),
}

impl Routed {
    /// The names, none unless it is [`Routed::Named`].
    pub fn names(&self) -> &[String] {
        match self {
            Routed::Named(names) => names,
            _ => &[],
        }
    }
}

/// `skill-routes.tsv`'s lines as written: slug, pattern.
pub fn parse_routes(tsv: &str) -> Vec<(String, String)> {
    tsv.lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split('\t');
            Some((fields.next()?.to_string(), fields.next()?.to_string()))
        })
        .collect()
}

/// `skill-index.tsv`'s lines as written: slug, then term and weight. The blurb stays out.
pub fn parse_index(tsv: &str) -> Vec<(String, Vec<(String, u32)>)> {
    tsv.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let slug = fields.next()?.to_string();
            let terms = fields.nth(1)?;
            let weights = terms
                .split(',')
                .filter_map(|pair| {
                    let (term, weight) = pair.split_once('=')?;
                    Some((term.to_string(), weight.parse().ok()?))
                })
                .collect();
            Some((slug, weights))
        })
        .collect()
}

/// The stopwords, sorted and without repeats.
pub fn stop_words() -> Vec<&'static str> {
    STOPWORDS
        .split_whitespace()
        .chain(BOILERPLATE.split_whitespace())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Everything a request is checked against, in the order the hook checks it.
pub struct Table {
    routes: Vec<(String, Pattern)>,
    index: Vec<(String, HashMap<String, u32>)>,
    stop: HashSet<String>,
}

impl Table {
    /// Errs naming the route whose pattern [`Pattern::new`] refuses.
    pub fn new(
        routes: Vec<(String, String)>,
        index: Vec<(String, HashMap<String, u32>)>,
        stop: impl IntoIterator<Item = String>,
    ) -> Result<Self, String> {
        let routes = routes
            .into_iter()
            .map(|(slug, pattern)| {
                let compiled = Pattern::new(&pattern).map_err(|e| format!("{slug}: {e}"))?;
                Ok((slug, compiled))
            })
            .collect::<Result<_, String>>()?;
        Ok(Table {
            routes,
            index,
            stop: stop.into_iter().collect(),
        })
    }

    /// From the two files the hook reads.
    pub fn from_hook_files(routes_tsv: &str, index_tsv: &str) -> Result<Self, String> {
        let index = parse_index(index_tsv)
            .into_iter()
            .map(|(slug, weights)| (slug, weights.into_iter().collect()))
            .collect();
        let stop = stop_words().into_iter().map(String::from);
        Self::new(parse_routes(routes_tsv), index, stop)
    }

    /// The script's `END` block, for a prompt as typed.
    pub fn route(&self, prompt: &str) -> Routed {
        let text = hook_text(prompt);
        if text.trim_start_matches(' ').starts_with('<')
            || text.contains("[watch]")
            || text.contains("[climb]")
            || text.contains("plugin sent a message")
            || text.chars().count() > MAX_CHARS
        {
            return Routed::Nothing;
        }
        let text = normalise(&text);
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Routed::Empty;
        }
        if words.len() < MIN_WORDS {
            return Routed::Short;
        }
        let padded = format!(" {text} ");
        let limit = if padded.starts_with(CONTINUATION) {
            1
        } else {
            MAX_NAMED
        };

        let mut names: Vec<String> = Vec::new();
        for (slug, pattern) in &self.routes {
            if names.len() >= limit {
                break;
            }
            if pattern.is_match(padded.as_bytes()) && !names.contains(slug) {
                names.push(slug.clone());
            }
        }
        let seen = self.pairs(&words);
        while names.len() < limit {
            let mut best = MIN_SCORE - 1;
            let mut pick = None;
            for (slug, weights) in &self.index {
                if names.contains(slug) {
                    continue;
                }
                let total: u32 = seen.iter().filter_map(|term| weights.get(term)).sum();
                if total > best {
                    best = total;
                    pick = Some(slug);
                }
            }
            let Some(pick) = pick else { break };
            names.push(pick.clone());
        }
        if names.is_empty() {
            Routed::Nothing
        } else {
            Routed::Named(names)
        }
    }

    /// Neighbouring stems. A stopword or a word under three letters breaks a pair.
    fn pairs(&self, words: &[&str]) -> HashSet<String> {
        let mut seen = HashSet::new();
        let mut previous = String::new();
        for word in words {
            if word.len() < 3 || self.stop.contains(*word) {
                previous.clear();
                continue;
            }
            let current = stem(word);
            if !previous.is_empty() {
                seen.insert(format!("{previous}_{current}"));
            }
            previous = current;
        }
        seen
    }
}

/// The prompt as the script holds it after its `sub`: the inside of its JSON string, and the
/// closing quote the script's match took with it. JSON's own escapes: `JSON.stringify` and
/// serde_json write the same ones, and a harness's `’` is the script's own concern.
fn hook_text(prompt: &str) -> String {
    let mut out = String::with_capacity(prompt.len() + 1);
    for c in prompt.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The script's `tolower` and `gsub` lines, in its order. Nothing is trimmed: the text keeps
/// the single space its punctuation at either end turned into.
fn normalise(text: &str) -> String {
    let text = text.to_ascii_lowercase().replace("\\u2019", "");
    let text = pairs_to_space(&text, Some(&['n', 'r', 't']));
    let text = pairs_to_space(&text, None);
    let text = text.replace(['\'', '\u{2019}'], "");
    let text = tags_to_space(&text);
    let mut out = String::with_capacity(text.len());
    let mut in_gap = false;
    for c in text.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '%' {
            out.push(c);
            in_gap = false;
        } else if !in_gap {
            out.push(' ');
            in_gap = true;
        }
    }
    out
}

/// `gsub(/\\[nrt]/, " ")`, or `gsub(/\\./, " ")` when `only` is `None`: a backslash and the
/// character after it become one space, scanning left to right.
fn pairs_to_space(text: &str, only: Option<&[char]>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(next) = chars.peek()
            && only.is_none_or(|set| set.contains(next))
        {
            chars.next();
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

/// `gsub(/<[^>]*>/, " ")`.
fn tags_to_space(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push(' ');
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

/// The part of POSIX ERE `skill-routes.tsv` uses: literals, groups, alternatives, a bracket
/// of characters and ranges, and `+`. Anything else is refused, so a route that would be read
/// differently here than by awk fails the build, naming itself, instead of quietly
/// disagreeing. The browser's module carries no regex library: the card's whole matcher is
/// these few lines.
pub struct Pattern(Vec<Node>);

enum Node {
    Byte(u8),
    Class(Vec<(u8, u8)>),
    Group(Vec<Vec<Node>>),
    Plus(Box<Node>),
}

impl Pattern {
    pub fn new(source: &str) -> Result<Self, String> {
        let bytes = source.as_bytes();
        let mut at = 0;
        let alternatives =
            parse_alternatives(bytes, &mut at).map_err(|e| format!("`{source}`: {e}"))?;
        if at != bytes.len() {
            return Err(format!("`{source}`: an unmatched `)`"));
        }
        Ok(Pattern(vec![Node::Group(alternatives)]))
    }

    /// Whether it matches anywhere in `text`. Only ever a yes or no, so it does not matter
    /// that this takes the first way through and awk the longest.
    pub fn is_match(&self, text: &[u8]) -> bool {
        (0..=text.len()).any(|start| seq(&self.0, text, start, &mut |_| true))
    }
}

/// Alternatives up to a `)` or the end, which is left for the caller.
fn parse_alternatives(bytes: &[u8], at: &mut usize) -> Result<Vec<Vec<Node>>, String> {
    let mut alternatives = Vec::new();
    let mut current = Vec::new();
    while let Some(&c) = bytes.get(*at) {
        match c {
            b')' => break,
            b'|' => {
                *at += 1;
                alternatives.push(non_empty(std::mem::take(&mut current))?);
            }
            b'(' => {
                *at += 1;
                let inner = parse_alternatives(bytes, at)?;
                if bytes.get(*at) != Some(&b')') {
                    return Err("an unclosed `(`".into());
                }
                *at += 1;
                current.push(Node::Group(inner));
            }
            b'[' => {
                *at += 1;
                current.push(parse_class(bytes, at)?);
            }
            b'+' => {
                *at += 1;
                let Some(last) = current.pop() else {
                    return Err("`+` with nothing to repeat".into());
                };
                current.push(Node::Plus(Box::new(last)));
            }
            b'*' | b'?' | b'.' | b'^' | b'$' | b'{' | b'}' | b'\\' | b']' => {
                return Err(format!(
                    "`{}` is more than the card's matcher reads",
                    c as char
                ));
            }
            c => {
                *at += 1;
                current.push(Node::Byte(c));
            }
        }
    }
    alternatives.push(non_empty(current)?);
    Ok(alternatives)
}

/// An alternative that matches nothing has no meaning in POSIX, and would never end under `+`.
fn non_empty(nodes: Vec<Node>) -> Result<Vec<Node>, String> {
    if nodes.is_empty() {
        Err("an empty alternative".into())
    } else {
        Ok(nodes)
    }
}

/// A bracket's characters and `a-z` ranges, after its `[`. A negated one is refused.
fn parse_class(bytes: &[u8], at: &mut usize) -> Result<Node, String> {
    let mut ranges = Vec::new();
    if bytes.get(*at) == Some(&b'^') {
        return Err("a negated bracket is more than the card's matcher reads".into());
    }
    loop {
        match (bytes.get(*at), bytes.get(*at + 1), bytes.get(*at + 2)) {
            (Some(b']'), ..) if !ranges.is_empty() => {
                *at += 1;
                return Ok(Node::Class(ranges));
            }
            (Some(&lo), Some(b'-'), Some(&hi)) if hi != b']' => {
                ranges.push((lo, hi));
                *at += 3;
            }
            (Some(&c), ..) => {
                ranges.push((c, c));
                *at += 1;
            }
            (None, ..) => return Err("an unclosed `[`".into()),
        }
    }
}

fn seq(nodes: &[Node], text: &[u8], at: usize, then: &mut dyn FnMut(usize) -> bool) -> bool {
    let Some((first, rest)) = nodes.split_first() else {
        return then(at);
    };
    match first {
        Node::Byte(b) => text.get(at) == Some(b) && seq(rest, text, at + 1, then),
        Node::Class(ranges) => {
            text.get(at)
                .is_some_and(|c| ranges.iter().any(|(lo, hi)| lo <= c && c <= hi))
                && seq(rest, text, at + 1, then)
        }
        Node::Group(alternatives) => alternatives
            .iter()
            .any(|alt| seq(alt, text, at, &mut |end| seq(rest, text, end, then))),
        Node::Plus(inner) => plus(inner, rest, text, at, then),
    }
}

fn plus(
    inner: &Node,
    rest: &[Node],
    text: &[u8],
    at: usize,
    then: &mut dyn FnMut(usize) -> bool,
) -> bool {
    seq(std::slice::from_ref(inner), text, at, &mut |end| {
        plus(inner, rest, text, end, then) || seq(rest, text, end, then)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(pattern: &str, text: &str) -> bool {
        Pattern::new(pattern).unwrap().is_match(text.as_bytes())
    }

    #[test]
    fn groups_alternatives_ranges_and_plus_read_as_in_awk() {
        assert!(matches(" (bug|bugs) ", " a bug here "));
        assert!(!matches(" (bug|bugs) ", " debugging "));
        assert!(matches(
            " every [0-9]+ (min|hour) ",
            " run every 15 min ok "
        ));
        assert!(!matches(" every [0-9]+ (min|hour) ", " run every min ok "));
        assert!(matches("(a|b)+c", "xabac"));
        assert!(matches("[a-cx]+$".trim_end_matches('$'), "zzx"));
    }

    #[test]
    fn anything_else_is_refused_rather_than_misread() {
        for (pattern, says) in [
            ("a?", "`?`"),
            ("a*", "`*`"),
            ("a.b", "`.`"),
            ("^a", "`^`"),
            ("a$", "`$`"),
            ("a{2}", "`{`"),
            ("a\\b", "`\\`"),
            ("[^a]", "negated"),
            ("(a", "unclosed `(`"),
            ("a)", "unmatched `)`"),
            ("[a", "unclosed `[`"),
            ("(a|)", "empty alternative"),
            ("+a", "nothing to repeat"),
        ] {
            let error = Pattern::new(pattern)
                .err()
                .unwrap_or_else(|| panic!("{pattern}"));
            assert!(error.contains(says), "{pattern}: {error}");
        }
    }

    #[test]
    fn a_table_with_an_unreadable_route_names_the_skill() {
        let refused = Table::new(vec![("a-skill".into(), " x? ".into())], Vec::new(), []);
        assert!(refused.err().unwrap().starts_with("a-skill: "));
    }

    #[test]
    fn a_skill_with_two_routes_is_named_once_and_a_route_outranks_the_score() {
        let weights: HashMap<String, u32> = [("big_crash".to_string(), 500)].into();
        let table = Table::new(
            vec![
                ("twice".into(), " crash ".into()),
                ("twice".into(), " here ".into()),
                ("also".into(), " nope ".into()),
            ],
            vec![
                ("twice".into(), weights.clone()),
                ("scored".into(), weights),
            ],
            std::iter::empty(),
        )
        .unwrap();
        assert_eq!(table.route("a big crash here").names(), ["twice", "scored"]);
    }

    #[test]
    fn the_text_is_counted_as_the_script_counts_it() {
        let table = Table::new(
            vec![("s".into(), " bug ".into())],
            Vec::new(),
            std::iter::empty(),
        )
        .unwrap();
        // The closing quote counts: 3,999 typed is 4,000 held. Characters, not bytes.
        let at = |n: usize, fill: char| format!("a bug in it {}", fill.to_string().repeat(n - 12));
        assert_eq!(table.route(&at(3999, 'x')).names(), ["s"]);
        assert_eq!(table.route(&at(4000, 'x')), Routed::Nothing);
        assert_eq!(table.route(&at(3999, 'é')).names(), ["s"]);
    }

    #[test]
    fn the_empty_and_the_short_are_told_apart_from_nothing_found() {
        let table = Table::new(Vec::new(), Vec::new(), std::iter::empty()).unwrap();
        assert_eq!(table.route(""), Routed::Empty);
        assert_eq!(table.route("  ?! "), Routed::Empty);
        assert_eq!(table.route("Yes, go"), Routed::Short);
        assert_eq!(table.route("one two three four"), Routed::Nothing);
    }
}
