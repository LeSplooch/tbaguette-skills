//! Does the Crumb a commit publishes describe the skills it publishes?
//! (scripts/crumb_check.py)
//!
//! The Crumb is the skill graph: `docs/graph/graph.json` and the `/graph/` page
//! that draws it. Both are generated from `skills/` by the build, and the
//! pre-commit hook rebuilds them on every commit, so in the ordinary flow they
//! cannot fall behind. Three routes still publish a Crumb describing some other
//! set of skills:
//!
//! - a commit made with hooks off (the nightly routine lands its source-only
//!   commits that way, ahead of one closing site commit);
//! - a merge, where git runs pre-merge-commit and never pre-commit;
//! - a commit whose hook rebuilt from a working tree holding edits that were
//!   never staged (in a shared checkout, often another session's).
//!
//! Pushing to master is publishing here, so `.githooks/pre-push` runs this on
//! exactly what is about to go live: the pushed commit's own `skills/`, against
//! that commit's own generated files. The data comes from the commit; the check
//! itself does not. The hook runs `site/atelier`, which compiles this binary from
//! the working tree's sources when they are newer, so its code and its category
//! list are the working tree's. Another session editing `skills/` or `docs/`
//! cannot change the answer; one editing `site/crates/` can.
//!
//! It also checks the one hand-written thing the Crumb needs per family: a
//! colour. The palette in `docs/assets/styles.css` is validated as a set, one
//! entry per category and theme; a category added without one is drawn in the
//! fallback accent and nothing else would ever say so.
//!
//! One difference from the Python tool: that one ran the revision's own
//! `scripts/` to build the graph. This builds it with the graph code compiled
//! into this binary, from the revision's `skills/`, so the binary's category
//! list is the one applied. A revision whose skills do not fit it cannot be
//! checked (exit 2) rather than judged.
//!
//! `--live` fetches the deployed graph.json with `curl`, which keeps an HTTP
//! and TLS client out of the build's dependencies.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use atelier_core::graph::{self, BannerSummary};
use atelier_core::model::categories;
use regex::Regex;
use serde_json::{Value, json};

use super::root_from;
use crate::repo::load_skills;

/// The base path of the build GitHub Pages serves: the literal the pre-commit
/// hook hands the build. A Crumb built for any other base path links every
/// skill to a page that does not exist on the live site.
pub const PAGES_BASE_PATH: &str = "/tbaguette-skills";
pub const LIVE_GRAPH_URL: &str = "https://lesplooch.github.io/tbaguette-skills/graph/graph.json";

const GRAPH_JSON: &str = "docs/graph/graph.json";
const GRAPH_PAGE: &str = "docs/graph/index.html";
const STYLES: &str = "docs/assets/styles.css";

/// templates.ENGLISH_STRINGS.graph_page_lede_template, which the graph page
/// renders and the check expects to find there.
fn lede(skill_count: usize, pair_count: usize) -> String {
    format!(
        "How {skill_count} skills lean on each other: {pair_count} cross-references, \
         traced section by section."
    )
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))
}

/// The full hash of `rev`, or an error if it is not a commit in `root`.
pub fn resolve(root: &Path, rev: &str) -> Result<String> {
    let out = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ],
    )?;
    if !out.status.success() {
        bail!("'{rev}' is not a commit in {}", root.display());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn read_at(root: &Path, sha: &str, path: &str) -> Option<String> {
    let out = git(root, &["show", &format!("{sha}:{path}")]).ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A directory removed on drop.
pub(super) struct Scratch(pub(super) PathBuf);

impl Scratch {
    pub(super) fn new() -> Result<Scratch> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("crumb-check-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What the build writes into the Crumb for this commit, computed from this
/// commit's own skills.
struct Built {
    packed: Value,
    summary: BannerSummary,
}

fn build_at(root: &Path, sha: &str) -> Result<Built> {
    let scratch = Scratch::new()?;
    let tar = scratch.0.join("skills.tar");
    let archive = git(
        root,
        &[
            "archive",
            "--format=tar",
            &format!("--output={}", tar.display()),
            sha,
            "--",
            "skills",
        ],
    )?;
    if !archive.status.success() {
        bail!(
            "git archive {} failed: {}",
            &sha[..10],
            String::from_utf8_lossy(&archive.stderr).trim()
        );
    }
    let extract = Command::new("tar")
        .arg("-xf")
        .arg(&tar)
        .arg("-C")
        .arg(&scratch.0)
        .output()
        .context("running tar to unpack the commit's skills")?;
    if !extract.status.success() {
        bail!(
            "tar failed: {}",
            String::from_utf8_lossy(&extract.stderr).trim()
        );
    }
    let skills =
        load_skills(&scratch.0).with_context(|| format!("reading the skills at {}", &sha[..10]))?;

    // The graph code panics on a category naming a skill that is not there;
    // say so as a failure to check instead.
    let listed: BTreeSet<&str> = categories()
        .iter()
        .flat_map(|c| c.skill_slugs.iter().copied())
        .collect();
    let missing: Vec<&str> = listed
        .iter()
        .copied()
        .filter(|s| !skills.contains_key(*s))
        .collect();
    let uncategorised: Vec<&str> = skills
        .keys()
        .map(String::as_str)
        .filter(|s| !listed.contains(s))
        .collect();
    if !missing.is_empty() || !uncategorised.is_empty() {
        bail!(
            "the skills at {} do not fit this binary's categories (listed but absent: {missing:?}; \
             present but unlisted: {uncategorised:?})",
            &sha[..10]
        );
    }

    let graph = graph::build(
        &skills,
        categories(),
        &BTreeMap::new(),
        &format!("{PAGES_BASE_PATH}/skills/{{slug}}/"),
    );
    let packed = serde_json::from_str(&graph::to_json(&graph))
        .context("the graph this binary writes is not JSON")?;
    Ok(Built {
        packed,
        summary: graph::banner_summary(&graph),
    })
}

// ---- comparing graphs ----

/// A packed graph with citations resolved to skill slugs and the clock fields
/// (`change_status`, `change_at`: stamped from git history against the clock of
/// whichever build wrote them) left out. Two graphs are the same graph when
/// these are equal.
#[derive(PartialEq)]
struct Unpacked {
    template: String,
    categories: Vec<Value>,
    skills: Vec<(String, Value)>,
}

fn unpack(packed: &Value) -> Result<Unpacked> {
    if packed["schema"] != json!(graph::PACKED_SCHEMA) {
        bail!(
            "graph.json has schema {}, this check reads {}",
            packed["schema"],
            graph::PACKED_SCHEMA
        );
    }
    let empty = Vec::new();
    let skills = packed["skills"].as_array().unwrap_or(&empty);
    let cats = packed["categories"].as_array().unwrap_or(&empty);
    // A citation of a skill the graph does not list is itself a mismatch, so it
    // gets a placeholder name that no fresh build has, and the comparison names
    // the citing skill as out of date.
    let slug_at = |index: &Value| -> Result<String> {
        Ok(index
            .as_u64()
            .and_then(|i| skills.get(usize::try_from(i).ok()?))
            .and_then(|s| s[0].as_str())
            .map_or_else(|| format!("<no skill numbered {index}>"), str::to_string))
    };
    let family_slug = |index: &Value| -> Result<String> {
        index
            .as_u64()
            .and_then(|i| cats.get(usize::try_from(i).ok()?))
            .and_then(|c| c[0].as_str())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("graph.json points at family {index}, which it does not have"))
    };
    let cites = |list: &Value| -> Result<Value> {
        let mut out = Vec::new();
        for cite in list.as_array().unwrap_or(&empty) {
            let mut entry = cite.as_array().cloned().unwrap_or_default();
            if entry.is_empty() {
                bail!("graph.json has an empty citation");
            }
            entry[0] = json!(slug_at(&entry[0])?);
            out.push(Value::Array(entry));
        }
        Ok(Value::Array(out))
    };

    let mut unpacked_skills = Vec::new();
    for skill in skills {
        let slug = skill[0].as_str().unwrap_or_default().to_string();
        let mut sections = Vec::new();
        for section in skill[5].as_array().unwrap_or(&empty) {
            let mut subs = Vec::new();
            for sub in section[4].as_array().unwrap_or(&empty) {
                subs.push(json!({"title": sub[0], "words": sub[1], "refs": cites(&sub[2])?, "id": sub[3]}));
            }
            sections.push(json!({
                "title": section[0], "kind": section[1], "words": section[2],
                "refs": cites(&section[3])?, "subs": subs, "id": section[5],
            }));
        }
        let value = json!({
            "slug": slug,
            "family": family_slug(&skill[1])?,
            "summary": skill[2],
            "words": skill[3],
            "trigger": cites(&skill[4])?,
            "sections": sections,
            "always_on": skill[6].as_bool().unwrap_or(false),
            "name": skill[9],
        });
        unpacked_skills.push((slug, value));
    }
    let mut categories = Vec::new();
    for cat in cats {
        let members: Result<Vec<String>> = cat[2]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(&slug_at)
            .collect();
        categories.push(json!({"slug": cat[0], "title": cat[1], "skills": members?}));
    }
    Ok(Unpacked {
        template: packed["skill_url_template"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        categories,
        skills: unpacked_skills,
    })
}

fn names(items: &[&str]) -> String {
    const LIMIT: usize = 8;
    let shown = items
        .iter()
        .take(LIMIT)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if items.len() > LIMIT {
        format!("{shown} and {} more", items.len() - LIMIT)
    } else {
        shown
    }
}

/// Every way `published` differs from `fresh`, ignoring the clock fields. An
/// empty list means they are the same graph.
fn compare_graphs(fresh: &Value, published: &Value, label: &str) -> Result<Vec<String>> {
    let (a, b) = (
        unpack(fresh)?,
        unpack(published).with_context(|| format!("reading {label}"))?,
    );
    if a == b {
        return Ok(Vec::new());
    }
    let mut problems = Vec::new();
    if a.template != b.template {
        problems.push(format!(
            "{label} links skills at {:?}, but the Pages build links them at {:?} -- it was built \
             for another base path",
            b.template, a.template
        ));
    }
    let titles = |u: &Unpacked| -> Vec<String> {
        u.categories
            .iter()
            .map(|c| c["title"].as_str().unwrap_or_default().to_string())
            .collect()
    };
    let (fresh_families, published_families) = (titles(&a), titles(&b));
    if fresh_families != published_families {
        fn refs(v: &[String]) -> Vec<&str> {
            v.iter().map(String::as_str).collect()
        }
        problems.push(format!(
            "{label} lists other families than the skills do -- {} there, {} in the skills: {} vs {}",
            published_families.len(),
            fresh_families.len(),
            names(&refs(&published_families)),
            names(&refs(&fresh_families)),
        ));
    } else if a.categories != b.categories {
        problems.push(format!(
            "{label} files some skills under a different family than the skills do"
        ));
    }
    let fresh_skills: BTreeMap<&str, &Value> =
        a.skills.iter().map(|(s, v)| (s.as_str(), v)).collect();
    let published_skills: BTreeMap<&str, &Value> =
        b.skills.iter().map(|(s, v)| (s.as_str(), v)).collect();
    fn order(u: &Unpacked) -> Vec<&str> {
        u.skills.iter().map(|(s, _)| s.as_str()).collect()
    }
    let missing: Vec<&str> = order(&a)
        .into_iter()
        .filter(|s| !published_skills.contains_key(s))
        .collect();
    let gone: Vec<&str> = order(&b)
        .into_iter()
        .filter(|s| !fresh_skills.contains_key(s))
        .collect();
    let moved: Vec<&str> = order(&a)
        .into_iter()
        .filter(|s| {
            published_skills
                .get(s)
                .is_some_and(|p| *p != fresh_skills[s])
        })
        .collect();
    if !missing.is_empty() {
        problems.push(format!("missing from {label}: {}", names(&missing)));
    }
    if !gone.is_empty() {
        problems.push(format!("in {label} but no longer skills: {}", names(&gone)));
    }
    if !moved.is_empty() {
        problems.push(format!("out of date in {label}: {}", names(&moved)));
    }
    if problems.is_empty() {
        problems.push(format!(
            "{label} differs from a fresh build in its shape or its order"
        ));
    }
    Ok(problems)
}

// ---- the pages and the palette ----

fn escape(text: &str, quote: bool) -> String {
    let text = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    if quote {
        text.replace('"', "&quot;").replace('\'', "&#x27;")
    } else {
        text
    }
}

fn page_problems(page: Option<&str>, lede: &str) -> Vec<String> {
    if lede.is_empty() {
        return Vec::new();
    }
    let Some(page) = page else {
        return vec![format!("{GRAPH_PAGE} is missing")];
    };
    if [lede.to_string(), escape(lede, false), escape(lede, true)]
        .iter()
        .any(|form| page.contains(form.as_str()))
    {
        return Vec::new();
    }
    vec![format!(
        "{GRAPH_PAGE} does not carry the summary a fresh build writes: {lede:?}"
    )]
}

/// Selector to the family numbers its rule gives a `--graph-cat-N` colour, for
/// every rule that gives any (one rule per theme).
fn palette_blocks(css: &str) -> Vec<(String, BTreeSet<usize>)> {
    let comment = Regex::new(r"(?s)/\*.*?\*/").expect("static regex");
    let block = Regex::new(r"([^{}]+)\{([^{}]*)\}").expect("static regex");
    let entry = Regex::new(r"--graph-cat-(\d+)\s*:").expect("static regex");
    let css = comment.replace_all(css, "");
    let mut blocks: Vec<(String, BTreeSet<usize>)> = Vec::new();
    for c in block.captures_iter(&css) {
        let defined: BTreeSet<usize> = entry
            .captures_iter(&c[2])
            .filter_map(|e| e[1].parse().ok())
            .collect();
        if defined.is_empty() {
            continue;
        }
        let selector = c[1].split_whitespace().collect::<Vec<_>>().join(" ");
        match blocks.iter_mut().find(|(s, _)| *s == selector) {
            Some((_, existing)) => existing.extend(defined),
            None => blocks.push((selector, defined)),
        }
    }
    blocks
}

/// Every rule that sets the family palette must name a colour for every family:
/// a partial theme would mix two palettes, and a missing entry falls back to
/// the accent colour without a word.
fn palette_problems(css: Option<&str>, family_count: usize) -> Vec<String> {
    if family_count == 0 {
        return Vec::new();
    }
    let Some(css) = css else {
        return vec![format!("{STYLES} is missing")];
    };
    let blocks = palette_blocks(css);
    if blocks.is_empty() {
        return vec![format!(
            "{STYLES} defines no --graph-cat-N family colours at all"
        )];
    }
    blocks
        .into_iter()
        .filter_map(|(selector, defined)| {
            let absent: Vec<String> = (1..=family_count).filter(|n| !defined.contains(n)).map(|n| n.to_string()).collect();
            (!absent.is_empty()).then(|| {
                format!(
                    "{STYLES}: `{selector}` has no colour for family {} of {family_count} -- extend \
                     the palette and revalidate it as a set, per the comment above it",
                    absent.join(", ")
                )
            })
        })
        .collect()
}

// ---- the check ----

fn fetch_live(url: &str, sha: &str) -> Result<Value> {
    // The query string is only a cache key: without it, a CDN edge can answer
    // with the copy from before the deploy for several minutes.
    let out = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--max-time",
            "30",
        ])
        .arg(format!("{url}?rev={}", &sha[..12]))
        .output()
        .map_err(|e| {
            anyhow!("could not read the live graph.json at {url}: curl could not be run ({e})")
        })?;
    if !out.status.success() {
        bail!(
            "could not read the live graph.json at {url}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| anyhow!("could not read the live graph.json at {url}: {e}"))
}

/// (summary line, problems). No problems means the Crumb at `rev` matches the
/// skills at `rev` and, with `live_url`, that the deployed one does too.
pub fn check(root: &Path, rev: &str, live_url: Option<&str>) -> Result<(String, Vec<String>)> {
    let sha = resolve(root, rev)?;
    let short = &sha[..10];
    let Some(published) = read_at(root, &sha, GRAPH_JSON) else {
        return Ok((
            format!("{short}: no Crumb at this commit, nothing to check"),
            Vec::new(),
        ));
    };
    let published: Value = serde_json::from_str(&published)
        .map_err(|e| anyhow!("{GRAPH_JSON} at {short} is not JSON: {e}"))?;
    let built = build_at(root, &sha)?;
    let skill_count = built.packed["skills"].as_array().map_or(0, Vec::len);
    let family_count = built.packed["categories"].as_array().map_or(0, Vec::len);

    let mut problems = compare_graphs(&built.packed, &published, GRAPH_JSON)?;
    problems.extend(page_problems(
        read_at(root, &sha, GRAPH_PAGE).as_deref(),
        &lede(skill_count, built.summary.pair_count),
    ));
    problems.extend(palette_problems(
        read_at(root, &sha, STYLES).as_deref(),
        family_count,
    ));

    if let Some(url) = live_url {
        let live = fetch_live(url, &sha)?;
        problems.extend(compare_graphs(
            &built.packed,
            &live,
            &format!("the live graph.json ({url})"),
        )?);
    }

    let summary = format!(
        "{short}: the Crumb matches its skills -- {skill_count} skills, {} cross-references, {family_count} families{}",
        built.summary.pair_count,
        if live_url.is_some() {
            ", live site included"
        } else {
            ""
        }
    );
    Ok((summary, problems))
}

/// `atelier-build crumb-check [REV] [--live] [--live-url URL] [--root DIR]`.
/// Exit 1 and every mismatch named; 2 when the check could not run at all.
pub fn run_crumb_check(args: Vec<String>) -> ExitCode {
    let mut rev = "HEAD".to_string();
    let (mut live, mut live_url) = (false, LIVE_GRAPH_URL.to_string());
    let mut iter = args.iter();
    let mut parse_error = None;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--live" => live = true,
            "--live-url" => match iter.next() {
                Some(url) => live_url = url.clone(),
                None => parse_error = Some("--live-url needs a URL"),
            },
            "--root" => {
                iter.next();
            }
            other if other.starts_with("--") => parse_error = Some("unknown option"),
            other => rev = other.to_string(),
        }
    }
    let outcome = match parse_error {
        Some(message) => Err(anyhow!("{message}")),
        None => root_from(&args).and_then(|root| {
            let sha = resolve(&root, &rev)?;
            let (summary, problems) = check(&root, &sha, live.then_some(live_url.as_str()))?;
            Ok((sha, summary, problems))
        }),
    };
    match outcome {
        Ok((_, summary, problems)) if problems.is_empty() => {
            println!("crumb-check: {summary}");
            ExitCode::SUCCESS
        }
        Ok((sha, _, problems)) => {
            eprintln!(
                "crumb-check: {}: the Crumb does not match its skills:",
                &sha[..10]
            );
            for problem in problems {
                eprintln!("  - {problem}");
            }
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("crumb-check: {error:#}");
            ExitCode::from(2)
        }
    }
}
