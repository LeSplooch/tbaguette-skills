//! Builds `hooks/skill-index.tsv`, the table the per-turn hook routes a prompt
//! with (`site/atelier router`), and the gate that holds the committed table to
//! what this builds.
//!
//! A generic "does a skill cover this?" fired on every turn stopped changing
//! behaviour, and the skill listing it pointed at keeps trigger text for only
//! the most-used skills. So `hooks/route-skills.awk` checks each prompt against
//! `hooks/skill-routes.tsv`, a short hand-written list of intents, then against
//! this table, built from every skill's full `description:`, and the hook names
//! the skills that fit.
//!
//! Each line: slug TAB one-line trigger TAB term=weight,term=weight,... A term
//! is a stem_stem pair of neighbouring words. The stemmer and the stopwords live
//! both in `atelier_core::router` and in the awk script, because awk has no import;
//! plugin_hooks.rs compares them.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::process::ExitCode;
use std::sync::LazyLock;

use anyhow::{Context, Result, anyhow};
use atelier_core::router::{Table, parse_index, parse_routes, stop_words};
use regex::Regex;
use serde_json::json;

use super::{Repo, root_from};

pub use atelier_core::router::{BOILERPLATE, STOPWORDS, stem};

/// The table, relative to the repository root.
pub const INDEX_PATH: &str = "hooks/skill-index.tsv";
/// The hand-written intents, relative to the repository root.
pub const ROUTES_PATH: &str = "hooks/skill-routes.tsv";

static IGNORED: LazyLock<BTreeSet<&'static str>> = LazyLock::new(|| {
    STOPWORDS
        .split_whitespace()
        .chain(BOILERPLATE.split_whitespace())
        .collect()
});

/// The one-line description at the top of a SKILL.md.
static DESCRIPTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^description:\s*(.+)$").expect("static regex"));

/// The opening a description's blurb drops.
static USE_WHEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Use (this )?(at|when|whenever|before|after|for)\b\s*").expect("static regex")
});

/// The longest a blurb runs, in characters, before it is cut at a word.
const BLURB_LIMIT: usize = 110;

/// Stems and neighbour pairs. A stopword breaks a pair, as it does in the awk
/// script.
fn terms(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase().replace('\'', "");
    let mut out = Vec::new();
    let mut previous = String::new();
    for word in lowered
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
        .filter(|w| !w.is_empty())
    {
        if word.len() < 3 || IGNORED.contains(word) {
            previous.clear();
            continue;
        }
        let current = stem(word);
        out.push(current.clone());
        if !previous.is_empty() {
            out.push(format!("{previous}_{current}"));
        }
        previous = current;
    }
    out
}

fn description_of(skill_md: &Path) -> Result<String> {
    let text = std::fs::read_to_string(skill_md)
        .with_context(|| format!("reading {}", skill_md.display()))?;
    let found = DESCRIPTION
        .captures(&text)
        .ok_or_else(|| anyhow!("{}: no one-line description", skill_md.display()))?;
    Ok(found[1]
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string())
}

/// The first sentence of a description, without its "Use when", cut to
/// [`BLURB_LIMIT`] at a word, and safe in a shell-built JSON string.
pub fn blurb(description: &str) -> String {
    let text = USE_WHEN.replace(description, "");
    let text = text.trim();
    // Up to the first white space that follows a `.` or a `;`.
    let mut first = text;
    let mut previous = '\0';
    for (at, c) in text.char_indices() {
        if c.is_whitespace() && (previous == '.' || previous == ';') {
            first = &text[..at];
            break;
        }
        previous = c;
    }
    let cleaned = first
        .trim_end_matches(['.', ';'])
        .replace('\t', " ")
        .replace(['"', '`'], "'")
        .replace('\\', "/");
    if cleaned.chars().count() <= BLURB_LIMIT {
        return cleaned;
    }
    let cut: String = cleaned.chars().take(BLURB_LIMIT).collect();
    let word = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{}...", word.trim_end_matches([',', ';', ':', ' ']))
}

/// One term's weight: a term in the skill's name counts most, then one in its
/// trigger, then one only in what it "Covers".
fn weigh(slug: &str, description: &str) -> BTreeMap<String, u32> {
    let (trigger, covers) = description
        .split_once(" Covers ")
        .unwrap_or((description, ""));
    let mut weights = BTreeMap::new();
    for term in terms(trigger) {
        weights.insert(term, 2);
    }
    for term in terms(covers) {
        weights.entry(term).or_insert(1);
    }
    for term in terms(&slug.replace('-', " ")) {
        weights.insert(term, 3);
    }
    weights
}

/// The table, from the skills under `root`.
pub fn build(root: &Path) -> Result<String> {
    let dir = root.join("skills");
    let mut skills: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let skill_md = path.join("SKILL.md");
        if skill_md.is_file() {
            let slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow!("{}: not UTF-8", path.display()))?
                .to_string();
            skills.push((slug, description_of(&skill_md)?));
        }
    }
    skills.sort();

    let per_skill: Vec<BTreeMap<String, u32>> = skills
        .iter()
        .map(|(slug, description)| weigh(slug, description))
        .collect();
    let mut df: HashMap<&str, usize> = HashMap::new();
    for weights in &per_skill {
        for term in weights.keys() {
            *df.entry(term).or_default() += 1;
        }
    }
    let n = skills.len() as f64;
    let mut table = String::new();
    for ((slug, description), weights) in skills.iter().zip(&per_skill) {
        let scored: Vec<String> = weights
            .iter()
            // A pair only. Measured on 565 real prompts, single words matched
            // 84% of them, "Continue" included, and most of those matches were
            // noise.
            .filter(|(term, _)| term.contains('_') && df[term.as_str()] as f64 <= n / 20.0)
            .map(|(term, base)| {
                let rarity = (n / df[term.as_str()] as f64).ln();
                (
                    term,
                    (f64::from(*base) * rarity * 10.0).round_ties_even() as i64,
                )
            })
            .filter(|(_, weight)| *weight > 0)
            .map(|(term, weight)| format!("{term}={weight}"))
            .collect();
        table.push_str(&format!(
            "{slug}\t{}\t{}\n",
            blurb(description),
            scored.join(",")
        ));
    }
    Ok(table)
}

/// The hook's tables as the landing page's card fetches them: routes as `[slug, pattern]`,
/// each skill's terms and weights, and the stopwords. Errs when a route uses more of POSIX
/// ERE than the card's matcher reads, naming it, rather than publishing a card that would
/// disagree with the hook.
pub fn site_table(root: &Path) -> Result<String> {
    let read = |rel: &str| {
        let path = root.join(rel);
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    };
    let (routes, index) = (read(ROUTES_PATH)?, read(INDEX_PATH)?);
    Table::from_hook_files(&routes, &index).map_err(|e| anyhow!("{ROUTES_PATH}: {e}"))?;
    let index: BTreeMap<String, BTreeMap<String, u32>> = parse_index(&index)
        .into_iter()
        .map(|(slug, weights)| (slug, weights.into_iter().collect()))
        .collect();
    let table = json!({
        "routes": parse_routes(&routes),
        "index": index,
        "stop": stop_words(),
    });
    Ok(table.to_string())
}

/// Gate: the committed table is what [`build`] writes.
pub fn fresh(repo: &Repo) -> Result<Vec<String>> {
    let built = build(&repo.root)?;
    let committed = std::fs::read_to_string(repo.path(INDEX_PATH)).unwrap_or_default();
    Ok(if committed == built {
        Vec::new()
    } else {
        vec![format!("{INDEX_PATH} is stale: run site/atelier router")]
    })
}

/// `atelier-build router [--check]`: writes the table, or with `--check` exits 1
/// if the committed one is stale.
pub fn run_router(args: Vec<String>) -> ExitCode {
    let result = (|| -> Result<ExitCode> {
        let root = root_from(&args)?;
        if args.iter().any(|a| a == "--check") {
            return Ok(match fresh(&Repo::new(&root))?.first() {
                None => ExitCode::SUCCESS,
                Some(problem) => {
                    println!("{problem}");
                    ExitCode::from(1)
                }
            });
        }
        let path = root.join(INDEX_PATH);
        std::fs::write(&path, build(&root)?)
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(ExitCode::SUCCESS)
    })();
    result.unwrap_or_else(|error| {
        eprintln!("atelier-build router: {error:#}");
        ExitCode::from(2)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stopword_breaks_a_pair_and_apostrophes_vanish() {
        assert_eq!(
            terms("Don't flush the cache layers"),
            vec![
                "dont",
                "flush",
                "dont_flush",
                "cache",
                "layer",
                "cache_layer"
            ]
        );
    }

    #[test]
    fn a_blurb_loses_its_opening_and_keeps_its_first_sentence() {
        assert_eq!(
            blurb("Use when a test flakes. Covers retries; and more."),
            "a test flakes"
        );
        assert_eq!(blurb("Use for x; then y."), "x");
        assert_eq!(
            blurb("Whenever the \"tab\"\tis `here`"),
            "Whenever the 'tab' is 'here'"
        );
    }

    #[test]
    fn a_long_blurb_is_cut_at_a_word_counted_in_characters() {
        let long = format!("{} tail", "é".repeat(108));
        assert_eq!(blurb(&long), format!("{}...", "é".repeat(108)));
    }

    #[test]
    fn the_name_outweighs_the_trigger_and_the_trigger_the_covers() {
        let weights = weigh(
            "flaky-tests",
            "flaky tests rerun quickly Covers rerun quickly and more words",
        );
        assert_eq!(weights["flaky_test"], 3);
        assert_eq!(weights["rerun_quickly"], 2);
        assert_eq!(weights["word"], 1);
    }
}
