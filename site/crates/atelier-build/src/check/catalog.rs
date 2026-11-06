//! CATALOG.md against `categories()` (scripts/test_catalog.py), and the skill
//! count written in prose in the manifests, README.md and CATALOG.md.
//!
//! `categories()` is the authoritative grouping: the site is built from it.
//! CATALOG.md is the hand-kept human mirror, and until this gate existed
//! nothing compared the two. The `†` marker in CATALOG.md means "not a skill
//! of this repo, listed only to show where one hands off", so those rows are
//! deliberately absent from the categories and are checked separately.

use anyhow::{Result, bail};
use atelier_core::model::categories;
use regex::Regex;

use super::Repo;

struct Section {
    title: String,
    own: Vec<String>,
    foreign: Vec<String>,
}

fn parse_catalog(text: &str) -> Vec<Section> {
    let row = Regex::new(r"^\|\s*`([a-z0-9-]+)`\s*(†?)\s*\|").expect("static regex");
    let mut sections: Vec<Section> = Vec::new();
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            sections.push(Section {
                title: title.trim().to_string(),
                own: Vec::new(),
                foreign: Vec::new(),
            });
        } else if let (Some(c), Some(section)) = (row.captures(line), sections.last_mut()) {
            let list = if c[2].is_empty() {
                &mut section.own
            } else {
                &mut section.foreign
            };
            list.push(c[1].to_string());
        }
    }
    sections
}

fn catalog(repo: &Repo) -> Result<Vec<Section>> {
    Ok(parse_catalog(&repo.read("CATALOG.md")?))
}

/// Display order on the site is category order; CATALOG.md must read the same
/// way, or a browser and a reader disagree about the library.
pub fn headings(repo: &Repo) -> Result<Vec<String>> {
    let found: Vec<String> = catalog(repo)?.into_iter().map(|s| s.title).collect();
    let expected: Vec<&str> = categories().iter().map(|c| c.title).collect();
    if found
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
    {
        return Ok(Vec::new());
    }
    Ok(vec![format!(
        "CATALOG.md headings are {found:?}, the categories are {expected:?}"
    )])
}

pub fn membership(repo: &Repo) -> Result<Vec<String>> {
    let sections = catalog(repo)?;
    let mut problems = Vec::new();
    for category in categories() {
        match sections.iter().find(|s| s.title == category.title) {
            None => problems.push(format!(
                "category {:?} has no section in CATALOG.md",
                category.title
            )),
            Some(section)
                if section
                    .own
                    .iter()
                    .map(String::as_str)
                    .ne(category.skill_slugs.iter().copied()) =>
            {
                problems.push(format!(
                    "category {:?}: CATALOG.md lists {:?}, the category lists {:?}",
                    category.title, section.own, category.skill_slugs
                ));
            }
            Some(_) => {}
        }
    }
    Ok(problems)
}

pub fn no_duplicates(repo: &Repo) -> Result<Vec<String>> {
    let listed: Vec<String> = catalog(repo)?.into_iter().flat_map(|s| s.own).collect();
    let mut dupes: Vec<&String> = listed
        .iter()
        .filter(|slug| listed.iter().filter(|other| other == slug).count() > 1)
        .collect();
    dupes.sort();
    dupes.dedup();
    Ok(dupes
        .into_iter()
        .map(|slug| format!("listed twice: {slug}"))
        .collect())
}

/// A † row claims the skill lives outside this repo. If one grows a directory
/// under skills/, the marker is a lie and the skill is missing from the
/// categories, which the build would then refuse to build.
pub fn foreign(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for section in catalog(repo)? {
        for slug in &section.foreign {
            if repo.path(&format!("skills/{slug}")).is_dir() {
                problems.push(format!(
                    "{}: {slug} is marked † (not a skill of this repo) but skills/{slug} exists",
                    section.title
                ));
            }
        }
    }
    Ok(problems)
}

const UNITS: [&str; 20] = [
    "",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase())
        .unwrap_or_default()
}

/// "Ninety-eight", "One hundred and two": the form the manifests open with.
pub fn spell(n: usize) -> Result<String> {
    const TENS: [(usize, &str); 8] = [
        (20, "twenty"),
        (30, "thirty"),
        (40, "forty"),
        (50, "fifty"),
        (60, "sixty"),
        (70, "seventy"),
        (80, "eighty"),
        (90, "ninety"),
    ];
    if n >= 200 {
        bail!("the spell-out needs extending for {n}");
    }
    if n >= 100 {
        let rest = n - 100;
        return Ok(if rest == 0 {
            "One hundred".to_string()
        } else {
            format!("One hundred and {}", spell(rest)?.to_lowercase())
        });
    }
    if n < 20 {
        return Ok(capitalize(UNITS[n]));
    }
    let (tens, unit) = (n / 10 * 10, n % 10);
    let word = TENS
        .iter()
        .find(|(t, _)| *t == tens)
        .map_or("", |(_, w)| *w);
    Ok(capitalize(&if unit == 0 {
        word.to_string()
    } else {
        format!("{word}-{}", UNITS[unit])
    }))
}

/// Whether `text` holds `stale` followed by a space that does not continue
/// into "and ": "One hundred" is a prefix of "One hundred and one", so a stale
/// form only counts when it stands alone.
fn carries_stale_count(text: &str, stale: &str) -> bool {
    let needle = format!("{stale} ");
    text.match_indices(&needle)
        .any(|(at, _)| !text[at + needle.len()..].starts_with("and "))
}

pub fn prose_counts(repo: &Repo) -> Result<Vec<String>> {
    let on_disk = repo.skill_dirs()?.len();
    let spelled = spell(on_disk)?;
    let digits = format!("{on_disk} skills");
    let mut problems = Vec::new();
    let files = [
        (".claude-plugin/plugin.json", &spelled),
        (".claude-plugin/marketplace.json", &spelled),
        (".codex-plugin/plugin.json", &spelled),
        (".cursor-plugin/plugin.json", &spelled),
        (".devin-plugin/plugin.json", &spelled),
        (".kimi-plugin/plugin.json", &spelled),
        ("gemini-extension.json", &spelled),
        ("plugin.json", &spelled),
        ("README.md", &digits),
    ];
    for (rel, form) in files {
        let text = repo.read(rel)?;
        if !text.contains(form.as_str()) {
            problems.push(format!(
                "{rel} does not contain {form:?}; {on_disk} skills are on disk"
            ));
        }
        for stale in [on_disk.saturating_sub(1), on_disk + 1] {
            let stale = spell(stale)?;
            if carries_stale_count(&text, &stale) {
                problems.push(format!("{rel} still carries the stale count {stale:?}"));
            }
        }
    }
    Ok(problems)
}

/// CATALOG.md opens with "N skills," written by hand. Only the preamble is
/// read, so a body line that starts "12 skills," cannot satisfy the header.
pub fn stated_count(repo: &Repo) -> Result<Vec<String>> {
    let text = repo.read("CATALOG.md")?;
    let preamble = text
        .split_once("## ")
        .map_or(text.as_str(), |(head, _)| head);
    let line = Regex::new(r"(?m)^(\d+) skills,").expect("static regex");
    let Some(found) = line.captures(preamble) else {
        return Ok(vec![
            "CATALOG.md has no 'N skills,' opening line".to_string(),
        ]);
    };
    let on_disk = repo.skill_dirs()?.len();
    let stated: usize = found[1].parse()?;
    Ok(if stated == on_disk {
        Vec::new()
    } else {
        vec![format!(
            "CATALOG.md says {stated} skills; {on_disk} are on disk"
        )]
    })
}
