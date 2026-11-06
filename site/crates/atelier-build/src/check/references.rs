//! Cross-references between skills (scripts/test_skill_references.py).
//!
//! A reference in this library is not decoration: the site turns it into a
//! link and an agent reading the skill treats it as a routing instruction. So
//! a reference to a skill that is not here fails twice, and a skill nothing
//! points at is one a reader finds only by already knowing its name.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;

use super::Repo;

/// Parentheticals that match the shape but are ordinary prose. Every addition
/// is a place the guard stops looking, so keep this short.
const NOT_SKILL_REFERENCES: [&str; 2] = [
    "e-ink",       // formidable/reference/stacks/embedded-display.md, a display type
    "state-level", // threat-modeling/SKILL.md, an attacker tier
];

/// Skills belonging to other plugins, referenced on purpose to show a handoff.
const FOREIGN_SKILLS: [&str; 1] = ["using-superpowers"];

const PHASE_INDEX: &str = "skills/orchestrating-work-end-to-end/reference/phase-routing.md";

/// Every `*.md` under `dir`, in the order Python's sorted(Path.rglob) gives:
/// by path components.
fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in
            std::fs::read_dir(&next).with_context(|| format!("reading {}", next.display()))?
        {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                files.push(path);
            }
        }
    }
    files.sort_by(|a, b| a.components().cmp(b.components()));
    Ok(files)
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `(?<![\w/-])([a-z0-9]+(?:-[a-z0-9]+)+)(?![\w/-])`, scanned by hand: the
/// `regex` crate has no look-around. A token is a maximal run of
/// `[a-z0-9]+` groups joined by single hyphens, with at least one hyphen,
/// not touching a word character, `/` or `-` on either side. Backtracking
/// to a shorter token can never help: every shorter prefix is followed by a
/// letter, digit or hyphen, which the look-ahead forbids.
fn bare_slugs(text: &str) -> BTreeSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let alnum = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    let blocked = |c: char| is_word(c) || c == '/' || c == '-';
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i < chars.len() {
        if !alnum(chars[i]) || (i > 0 && blocked(chars[i - 1])) {
            i += 1;
            continue;
        }
        let mut end = i;
        let mut hyphens = 0;
        loop {
            while end < chars.len() && alnum(chars[end]) {
                end += 1;
            }
            if end + 1 < chars.len() && chars[end] == '-' && alnum(chars[end + 1]) {
                hyphens += 1;
                end += 1;
            } else {
                break;
            }
        }
        if hyphens > 0 && !chars.get(end).is_some_and(|c| blocked(*c)) {
            found.insert(chars[i..end].iter().collect());
            i = end;
        } else {
            i += 1;
        }
    }
    found
}

pub fn resolve(repo: &Repo) -> Result<Vec<String>> {
    let reference =
        Regex::new(r"\((?:see\s+)?`?([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`?\)").expect("static regex");
    let fenced = Regex::new(r"(?s)```.*?```").expect("static regex");
    let mut known: BTreeSet<String> = repo.skill_dirs()?.into_iter().collect();
    known.extend(
        FOREIGN_SKILLS
            .iter()
            .chain(NOT_SKILL_REFERENCES.iter())
            .map(|s| (*s).to_string()),
    );
    let mut dangling = Vec::new();
    for path in markdown_files(&repo.path("skills"))? {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let text = fenced.replace_all(&text, "");
        let shown = path
            .strip_prefix(&repo.root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for c in reference.captures_iter(&text) {
            if !known.contains(&c[1]) {
                dangling.push(format!("{shown}: ({}) is not a skill", &c[1]));
            }
        }
    }
    Ok(dangling)
}

/// Zero tolerance: a newly added skill turns this red until some existing
/// skill points at it, which is the forcing function, not a defect.
pub fn reachable(repo: &Repo) -> Result<Vec<String>> {
    let slugs = repo.skill_dirs()?;
    let frontmatter = Regex::new(r"(?ms)^---.*?^---\n").expect("static regex");
    let backticked = Regex::new(r"`([a-z0-9-]+)`").expect("static regex");
    let mut incoming: BTreeMap<&str, usize> = slugs.iter().map(|s| (s.as_str(), 0)).collect();
    for source in &slugs {
        let mut text = String::new();
        for (n, path) in markdown_files(&repo.path(&format!("skills/{source}")))?
            .iter()
            .enumerate()
        {
            if n > 0 {
                text.push('\n');
            }
            text.push_str(
                &std::fs::read_to_string(path)
                    .with_context(|| format!("reading {}", path.display()))?,
            );
        }
        let text = frontmatter.replace_all(&text, "");
        let mut targets = bare_slugs(&text);
        targets.extend(backticked.captures_iter(&text).map(|c| c[1].to_string()));
        for target in targets {
            if target != *source
                && let Some(count) = incoming.get_mut(target.as_str())
            {
                *count += 1;
            }
        }
    }
    Ok(incoming
        .into_iter()
        .filter(|(_, n)| *n == 0)
        .map(|(slug, _)| format!("{slug}: no other skill cites it"))
        .collect())
}

/// orchestrating-work-end-to-end tells every run that phase-routing.md
/// "indexes every skill in the library", so a skill missing from it is
/// invisible to exactly that lookup.
pub fn phase_index(repo: &Repo) -> Result<Vec<String>> {
    let named: BTreeSet<String> = Regex::new(r"`([a-z0-9]+(?:-[a-z0-9]+)*)`")
        .expect("static regex")
        .captures_iter(&repo.read(PHASE_INDEX)?)
        .map(|c| c[1].to_string())
        .collect();
    Ok(repo
        .skill_dirs()?
        .into_iter()
        .filter(|s| s != "orchestrating-work-end-to-end" && !named.contains(s))
        .map(|s| format!("{s}: not named in {PHASE_INDEX}, which claims to index every skill"))
        .collect())
}

/// An exemption that becomes a real skill is a silenced check.
pub fn exemptions(repo: &Repo) -> Result<Vec<String>> {
    let slugs: BTreeSet<String> = repo.skill_dirs()?.into_iter().collect();
    Ok(NOT_SKILL_REFERENCES
        .iter()
        .chain(FOREIGN_SKILLS.iter())
        .filter(|e| slugs.contains(**e))
        .map(|e| format!("{e} is on an exemption list but is now a real skill"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::bare_slugs;

    fn found(text: &str) -> Vec<String> {
        bare_slugs(text).into_iter().collect()
    }

    #[test]
    fn a_bare_slug_is_found_between_punctuation() {
        assert_eq!(
            found("Not for: (red-green-refactor), see a-b."),
            ["a-b", "red-green-refactor"]
        );
    }

    #[test]
    fn a_token_touching_a_slash_hyphen_or_word_character_is_skipped() {
        assert!(found("a/b-c d-e/f x-y- z_a-b Foo-bar and-Or").is_empty());
    }

    #[test]
    fn a_single_word_is_not_a_slug() {
        assert!(found("refactor plain words").is_empty());
    }
}
