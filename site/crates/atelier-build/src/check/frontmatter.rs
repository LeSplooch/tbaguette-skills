//! Skill frontmatter against the Agent Skills platform limits
//! (scripts/test_skill_frontmatter.py).
//!
//! These are the format's limits, not house style: `name` at most 64
//! characters of lowercase letters, digits and hyphens, free of the reserved
//! words; `description` non-empty, at most 1024 characters; neither field
//! holding an XML tag.
//!
//! The parser here is deliberately the test's, not the build's. It folds a
//! wrapped value (a line that does not open a new `key:`) into one string, so
//! a description that someday wraps in YAML is measured whole, where the
//! site's single-line parser would truncate it and still pass.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use regex::Regex;

use super::Repo;

const MAX_NAME: usize = 64;
const MAX_DESCRIPTION: usize = 1024;
const RESERVED_WORDS: [&str; 2] = ["anthropic", "claude"];

/// `name` and `description` (and any other key) of a SKILL.md, wrapped lines
/// folded; empty when there is no frontmatter block.
fn parse(text: &str) -> BTreeMap<String, String> {
    // \A---\n(.*?)\n---\n, dot matching newlines.
    let Some(rest) = text.strip_prefix("---\n") else {
        return BTreeMap::new();
    };
    let Some(end) = rest.find("\n---\n") else {
        return BTreeMap::new();
    };
    let key = Regex::new(r"^([A-Za-z_][A-Za-z0-9_-]*):\s?(.*)$").expect("static regex");
    let mut fields: Vec<(String, Vec<String>)> = Vec::new();
    for line in rest[..end].split('\n') {
        let opens_key = !line.starts_with([' ', '\t']);
        match key.captures(line).filter(|_| opens_key) {
            Some(c) => fields.push((c[1].to_string(), vec![c[2].to_string()])),
            None => {
                if let Some((_, parts)) = fields.last_mut() {
                    parts.push(line.trim().to_string());
                }
            }
        }
    }
    fields
        .into_iter()
        .map(|(k, parts)| {
            let joined = parts
                .iter()
                .filter(|p| !p.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            (k, joined.trim().to_string())
        })
        .collect()
}

/// (slug, frontmatter) for every directory under skills/ that has a SKILL.md.
fn skills(repo: &Repo) -> Result<Vec<(String, BTreeMap<String, String>)>> {
    let mut out = Vec::new();
    for slug in repo.skill_dirs()? {
        let rel = format!("skills/{slug}/SKILL.md");
        if repo.is_file(&rel) {
            let text = repo.read(&rel).with_context(|| format!("reading {rel}"))?;
            out.push((slug, parse(&text)));
        }
    }
    Ok(out)
}

fn field<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    fields.get(key).map_or("", String::as_str)
}

pub fn required(repo: &Repo) -> Result<Vec<String>> {
    Ok(skills(repo)?
        .into_iter()
        .filter(|(_, fm)| field(fm, "name").is_empty() || field(fm, "description").is_empty())
        .map(|(slug, _)| format!("{slug}: SKILL.md without a name or a description"))
        .collect())
}

pub fn name_matches_dir(repo: &Repo) -> Result<Vec<String>> {
    Ok(skills(repo)?
        .into_iter()
        .filter(|(slug, fm)| field(fm, "name") != slug)
        .map(|(slug, fm)| {
            format!(
                "{slug}: name is {:?}, differs from the directory name",
                field(&fm, "name")
            )
        })
        .collect())
}

pub fn name_limits(repo: &Repo) -> Result<Vec<String>> {
    let charset = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("static regex");
    let mut bad = Vec::new();
    for (slug, fm) in skills(repo)? {
        let name = field(&fm, "name");
        let length = name.chars().count();
        if length > MAX_NAME {
            bad.push(format!("{slug}: {length} chars, limit {MAX_NAME}"));
        }
        if !charset.is_match(name) {
            bad.push(format!("{slug}: not lowercase-hyphen-alphanumeric"));
        }
        for word in RESERVED_WORDS {
            if name.contains(word) {
                bad.push(format!("{slug}: contains reserved word {word:?}"));
            }
        }
    }
    Ok(bad)
}

pub fn description_limit(repo: &Repo) -> Result<Vec<String>> {
    Ok(skills(repo)?
        .into_iter()
        .filter_map(|(slug, fm)| {
            let length = field(&fm, "description").chars().count();
            (length > MAX_DESCRIPTION).then(|| {
                format!(
                    "{slug}: {length} chars, limit {MAX_DESCRIPTION} -- displace an older routing \
                     trigger rather than appending another one"
                )
            })
        })
        .collect())
}

pub fn no_xml(repo: &Repo) -> Result<Vec<String>> {
    let tag = Regex::new(r"<[A-Za-z/][^>]*>").expect("static regex");
    let mut bad = Vec::new();
    for (slug, fm) in skills(repo)? {
        for key in ["name", "description"] {
            if tag.is_match(field(&fm, key)) {
                bad.push(format!(
                    "{slug}: {key} holds an XML tag; the platform rejects it"
                ));
            }
        }
    }
    Ok(bad)
}
