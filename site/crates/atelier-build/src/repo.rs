//! Reading the repository: the skills on disk, as the Python build's
//! `content_pipeline.build_content` loads them (before any rendering), the
//! plugin version, the update notes, the install test's source, and which
//! skills changed recently according to git.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use atelier_core::frontmatter;
use atelier_core::graph::ChangeStamp;
use atelier_core::model::{Reference, Skill, humanize_filename};
use atelier_core::updates::{self, UpdateEntry};
use chrono::{DateTime, TimeDelta, Utc};

const SKILL_FILE_NAME: &str = "SKILL.md";

/// Every skill under `<root>/skills/`, keyed and ordered by slug.
///
/// `root` is the repository root. A directory counts as a skill only if it
/// has a `SKILL.md`. Each needs `name` and `description` in its frontmatter;
/// a missing one fails naming the file, where the Python build died on a
/// `KeyError`. References are `reference/*.md` by name, then
/// `reference/stacks/*.md` by name (only formidable has stacks today).
pub fn load_skills(root: &Path) -> Result<BTreeMap<String, Skill>> {
    let skills_dir = root.join("skills");
    let mut skills = BTreeMap::new();
    for entry in
        fs::read_dir(&skills_dir).with_context(|| format!("reading {}", skills_dir.display()))?
    {
        let dir = entry?.path();
        if !dir.is_dir() || !dir.join(SKILL_FILE_NAME).is_file() {
            continue;
        }
        let slug = dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("{}: directory name is not UTF-8", dir.display()))?
            .to_string();
        let skill = load_skill(&dir, &slug)?;
        skills.insert(slug, skill);
    }
    Ok(skills)
}

fn load_skill(dir: &Path, slug: &str) -> Result<Skill> {
    let skill_file = dir.join(SKILL_FILE_NAME);
    let text = read(&skill_file)?;
    let (mut fields, body) =
        frontmatter::parse(&text).map_err(|e| anyhow!("{}: {e}", skill_file.display()))?;
    let mut take = |key: &str| {
        fields
            .remove(key)
            .ok_or_else(|| anyhow!("{}: frontmatter has no `{key}`", skill_file.display()))
    };
    let name = take("name")?;
    let description = take("description")?;

    let mut references = Vec::new();
    for sub in ["reference", "reference/stacks"] {
        for path in markdown_files(&dir.join(sub))? {
            let rel_path = path
                .strip_prefix(dir)
                .expect("a file under the skill directory")
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            references.push(Reference {
                rel_path,
                title: humanize_filename(stem),
                body: read(&path)?,
            });
        }
    }

    Ok(Skill {
        slug: slug.to_string(),
        name,
        description,
        body: body.to_string(),
        references,
    })
}

/// `dir/*.md` regular files, sorted by name; empty if `dir` does not exist.
fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            files.push(path);
        }
    }
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files)
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

/// The published plugin version (`.claude-plugin/plugin.json`'s `version`),
/// shown beside the wordmark.
pub fn plugin_version(root: &Path) -> Result<String> {
    let path = root.join(".claude-plugin/plugin.json");
    let manifest: serde_json::Value = serde_json::from_str(&read(&path)?)
        .with_context(|| format!("parsing {}", path.display()))?;
    manifest["version"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{}: no string `version`", path.display()))
}

/// The update notes file, at the repository root.
pub const UPDATE_NOTES_FILENAME: &str = "UPDATES.md";

/// UPDATES.md parsed into entries, newest first; none if the file is not
/// there (generate.py `_update_notes`).
///
/// Present but malformed is an error naming the file and line, and so is a
/// file past [`updates::MAX_BYTES`]: UPDATES.md ships in the plugin, and
/// Anthropic's plugin directory holds any text file over 256 KiB, so the
/// build stops at half that with the fix in the message.
pub fn update_notes(root: &Path) -> Result<Vec<UpdateEntry>> {
    let path = root.join(UPDATE_NOTES_FILENAME);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let size = fs::metadata(&path)
        .with_context(|| format!("reading {}", path.display()))?
        .len();
    if size > updates::MAX_BYTES {
        bail!(
            "{}: {} KiB, past the {} KiB this file is kept under. Run `atelier-build \
             archive-updates` to move everything past the newest {} entries into {}/, then \
             build again.",
            path.display(),
            size / 1024,
            updates::MAX_BYTES / 1024,
            updates::KEEP,
            updates::ARCHIVE_DIR,
        );
    }
    updates::parse(&read(&path)?).map_err(|e| anyhow!("{}: {e}", path.display()))
}

/// The install test /verify-install/ shows in full (`verify/
/// test_install_command.py`): editing that test changes the published page.
pub fn install_test_source(root: &Path) -> Result<String> {
    read(&root.join("verify/test_install_command.py"))
}

// ---------------------------------------------------------------------------
// Mods: separate plugins under mods/, and the live climb's log
// ---------------------------------------------------------------------------

/// Where the mods live, one directory each, below the repository root.
pub const MODS_DIR: &str = "mods";

/// A mod's plugin manifest, the fields the site shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    /// The page the manifest points readers to (for climb, its skill).
    pub homepage: String,
    pub repository: String,
    pub license: String,
}

/// One mod as the build reads it: its directory name, manifest and README,
/// the README as written (the pages take their facts from it).
#[derive(Debug, Clone)]
pub struct ModSource {
    pub slug: String,
    pub manifest: ModManifest,
    pub readme: String,
    /// `mods/<slug>/README.md`, for error messages.
    pub readme_path: PathBuf,
}

/// Every mod under `<root>/mods/`, by directory name. A directory is a mod
/// when it has `.claude-plugin/plugin.json`; it must also have a README.md,
/// and the manifest every field the pages show. Anything missing fails
/// naming the file.
pub fn mods(root: &Path) -> Result<Vec<ModSource>> {
    let dir = root.join(MODS_DIR);
    let mut found = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if !path.join(".claude-plugin/plugin.json").is_file() {
            continue;
        }
        let slug = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("{}: directory name is not UTF-8", path.display()))?
            .to_string();
        let readme_path = path.join("README.md");
        found.push(ModSource {
            manifest: mod_manifest(&path.join(".claude-plugin/plugin.json"))?,
            readme: read(&readme_path)?,
            readme_path,
            slug,
        });
    }
    found.sort_by(|a, b| a.slug.cmp(&b.slug));
    if found.is_empty() {
        bail!(
            "{}: no mod (a directory with .claude-plugin/plugin.json)",
            dir.display()
        );
    }
    Ok(found)
}

fn mod_manifest(path: &Path) -> Result<ModManifest> {
    let manifest: serde_json::Value = serde_json::from_str(&read(path)?)
        .with_context(|| format!("parsing {}", path.display()))?;
    let field = |key: &str| {
        manifest[key]
            .as_str()
            .filter(|v| !v.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("{}: no string `{key}`", path.display()))
    };
    Ok(ModManifest {
        name: field("name")?,
        version: field("version")?,
        description: field("description")?,
        homepage: field("homepage")?,
        repository: field("repository")?,
        license: field("license")?,
    })
}

/// The first live climb's `.climb-log.tsv`, kept verbatim (see
/// site/content/README.md for what that run was).
pub const CLIMB_LIVE_LOG_PATH: &str = "site/content/climb-live-run.tsv";

/// The columns the climb mod writes, in its order.
pub const CLIMB_LOG_COLUMNS: [&str; 5] = ["iteration", "exit", "score", "check_ms", "turn_ms"];

/// One row of a climb log: the cells as written, and their values.
#[derive(Debug, Clone, PartialEq)]
pub struct ClimbLogRow {
    pub cells: Vec<String>,
    pub iteration: u32,
    pub exit: i64,
    /// Empty when the check printed no `score:` line.
    pub score: Option<f64>,
    pub check_ms: u64,
    pub turn_ms: u64,
}

/// [`CLIMB_LIVE_LOG_PATH`], parsed. The header must be the mod's own, and
/// every row must hold numbers where the mod writes them: a log that is not
/// a climb log fails, naming the file and line.
pub fn climb_live_log(root: &Path) -> Result<Vec<ClimbLogRow>> {
    let path = root.join(CLIMB_LIVE_LOG_PATH);
    let text = read(&path)?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    if header != CLIMB_LOG_COLUMNS {
        bail!(
            "{}: line 1: expected the climb log header {:?}, found {header:?}",
            path.display(),
            CLIMB_LOG_COLUMNS
        );
    }
    let mut rows = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let at = || format!("{}: line {}", path.display(), index + 2);
        let cells: Vec<String> = line.split('\t').map(str::to_string).collect();
        if cells.len() != CLIMB_LOG_COLUMNS.len() {
            bail!(
                "{}: {} cells, expected {}",
                at(),
                cells.len(),
                CLIMB_LOG_COLUMNS.len()
            );
        }
        let number = |i: usize| -> Result<u64> {
            cells[i].parse().with_context(|| {
                format!(
                    "{}: `{}` is not a whole number ({})",
                    at(),
                    cells[i],
                    CLIMB_LOG_COLUMNS[i]
                )
            })
        };
        let score = if cells[2].is_empty() {
            None
        } else {
            Some(
                cells[2]
                    .parse()
                    .with_context(|| format!("{}: score `{}` is not a number", at(), cells[2]))?,
            )
        };
        rows.push(ClimbLogRow {
            iteration: number(0)? as u32,
            exit: cells[1]
                .parse()
                .with_context(|| format!("{}: exit `{}` is not a whole number", at(), cells[1]))?,
            score,
            check_ms: number(3)?,
            turn_ms: number(4)?,
            cells,
        });
    }
    if rows.is_empty() {
        bail!("{}: a header and no rows", path.display());
    }
    Ok(rows)
}

/// What the first live climb was started with, and what is known about the
/// run that its log does not record (see site/content/README.md).
pub const CLIMB_LIVE_META_PATH: &str = "site/content/climb-live-run.meta.tsv";

/// [`CLIMB_LIVE_META_PATH`], parsed: the budget, the check command and the
/// paths it protected, as typed at its prompt; then the facts the log's
/// caption states, so the page never types them itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClimbRunMeta {
    pub budget: u32,
    pub check: String,
    /// Space-separated in the file, as `/climb protect` takes them.
    pub protected: Vec<String>,
    /// The day it ran.
    pub date: chrono::NaiveDate,
    /// What it ran against, as a phrase ("a scratch project whose ...").
    pub project: String,
    /// What the check counted and how it scored ("twelve cases, ...").
    pub checked_by: String,
    /// The restart before the logged climb, when there was one: how often
    /// and why ("once, after a fix to the mod"), and what the earlier
    /// attempt had already done, which is why the first row does not start
    /// from nothing. Both or neither.
    pub restart: Option<(String, String)>,
}

/// Every key the meta file may hold.
const CLIMB_META_KEYS: [&str; 8] = [
    "budget",
    "check",
    "protected",
    "date",
    "project",
    "checked_by",
    "restarted",
    "carried",
];

/// Reads [`CLIMB_LIVE_META_PATH`]: one `key<TAB>value` row each for
/// `budget`, `check`, `protected`, `date` (YYYY-MM-DD), `project` and
/// `checked_by`, and optionally `restarted` with `carried`. A missing,
/// repeated or unknown key, a budget that is not a positive whole number, a
/// date that is not one, or one of the restart pair without the other fails
/// naming the file (and the line, where there is one).
pub fn climb_live_meta(root: &Path) -> Result<ClimbRunMeta> {
    let path = root.join(CLIMB_LIVE_META_PATH);
    let text = read(&path)?;
    let mut values: BTreeMap<&str, &str> = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let at = format!("{}: line {}", path.display(), index + 1);
        let Some((key, value)) = line.split_once('\t') else {
            bail!("{at}: expected `key<TAB>value`, found {line:?}");
        };
        if !CLIMB_META_KEYS.contains(&key) {
            bail!("{at}: unknown key `{key}` ({})", CLIMB_META_KEYS.join(", "));
        }
        if value.trim().is_empty() {
            bail!("{at}: `{key}` has no value");
        }
        if values.insert(key, value.trim()).is_some() {
            bail!("{at}: `{key}` given twice");
        }
    }
    let get = |key: &str| {
        values
            .get(key)
            .copied()
            .ok_or_else(|| anyhow!("{}: no `{key}` row", path.display()))
    };
    let budget_text = get("budget")?;
    let budget = match budget_text.parse::<u32>() {
        Ok(n) if n > 0 => n,
        _ => bail!(
            "{}: budget `{budget_text}` is not a positive whole number",
            path.display()
        ),
    };
    let check = get("check")?.to_string();
    let protected = get("protected")?
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let date_text = get("date")?;
    let date = chrono::NaiveDate::parse_from_str(date_text, "%Y-%m-%d").map_err(|_| {
        anyhow!(
            "{}: date `{date_text}` is not a YYYY-MM-DD date",
            path.display()
        )
    })?;
    let restart = match (values.get("restarted"), values.get("carried")) {
        (Some(how), Some(carried)) => Some((how.to_string(), carried.to_string())),
        (None, None) => None,
        _ => bail!(
            "{}: `restarted` and `carried` go together; give both or neither",
            path.display()
        ),
    };
    Ok(ClimbRunMeta {
        budget,
        check,
        protected,
        date,
        project: get("project")?.to_string(),
        checked_by: get("checked_by")?.to_string(),
        restart,
    })
}

// ---------------------------------------------------------------------------
// Freshness: which skills changed recently, from git
// ---------------------------------------------------------------------------

/// How long a skill keeps its New/Updated badge after the change landed
/// (generate.py `FRESH_WINDOW_HOURS`). site.js enforces it again in the
/// browser, for a page read long after its build.
pub const FRESH_WINDOW_HOURS: i64 = 48;

/// How far back the log walk goes (generate.py `_LOG_SCAN_LIMIT`). It only
/// bites if more commits than this touched skills/ inside the window, and
/// then it fails toward a missing badge rather than a wrong one.
const LOG_SCAN_LIMIT: usize = 1000;

/// The revision whose working tree is the one being built.
pub const HEAD: &str = "HEAD";

/// Every skill inside the freshness window at `now`, as generate.py
/// `_fresh_skills` computes it: the newest commit touching `skills/<slug>/`
/// in the history of `rev` (status `new` only if that same commit added the
/// skill's SKILL.md), replaced by the working tree's own state for a skill
/// with uncommitted changes.
///
/// The working-tree half only applies when `rev` is [`HEAD`]: it describes
/// what is about to be committed on top of HEAD, which says nothing about an
/// older revision. Passing an older `rev` reproduces what a build at that
/// commit saw.
///
/// Fails open, as the Python build does: no git, or git failing, means no
/// badges rather than no build.
pub fn fresh_skills(root: &Path, now: DateTime<Utc>, rev: &str) -> BTreeMap<String, ChangeStamp> {
    let mut fresh = recent_skill_commits(root, now, rev);
    if rev == HEAD {
        // Uncommitted work is the newest touch by definition, so it replaces
        // a commit-derived entry outright, status included.
        let at = python_isoformat(now);
        for (slug, status) in changed_skill_slugs(root) {
            fresh.insert(
                slug,
                ChangeStamp {
                    status: Some(status.to_string()),
                    at: Some(at.clone()),
                },
            );
        }
    }
    fresh
}

/// Python's `datetime.isoformat()` for a UTC instant: microseconds only when
/// there are any.
fn python_isoformat(at: DateTime<Utc>) -> String {
    if at.timestamp_subsec_micros() == 0 {
        at.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
    } else {
        at.format("%Y-%m-%dT%H:%M:%S%.6f+00:00").to_string()
    }
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The slug a repository path belongs to, if it is under `skills/<slug>/`.
fn skill_slug_of(path: &str) -> Option<&str> {
    let mut parts = path.split('/').filter(|p| !p.is_empty() && *p != ".");
    (parts.next()? == "skills").then_some(())?;
    parts.next()
}

/// generate.py `_recent_skill_commits`. No `--since`: commit dates are not
/// monotonic, and `--since` stops the walk at the first old commit; the
/// window is applied here on the parsed committer date instead.
fn recent_skill_commits(
    root: &Path,
    now: DateTime<Utc>,
    rev: &str,
) -> BTreeMap<String, ChangeStamp> {
    let cutoff = now - TimeDelta::hours(FRESH_WINDOW_HOURS);
    let limit = format!("-n{LOG_SCAN_LIMIT}");
    let Some(log) = git_stdout(
        root,
        &[
            "log",
            &limit,
            "--no-renames",
            "--format=%x00%cI",
            "--name-status",
            // `rev` is a revision, never an option.
            "--end-of-options",
            rev,
            "--",
            "skills",
        ],
    ) else {
        return BTreeMap::new();
    };

    let mut fresh: BTreeMap<String, ChangeStamp> = BTreeMap::new();
    let mut commit_at: Option<&str> = None;
    for line in log.lines() {
        if let Some(stamp) = line.strip_prefix('\0') {
            let stamp = stamp.trim();
            commit_at = DateTime::parse_from_rfc3339(stamp)
                .is_ok_and(|at| at >= cutoff)
                .then_some(stamp);
            continue;
        }
        let Some(at) = commit_at else { continue };
        let Some((code, rest)) = line.split_once('\t') else {
            continue;
        };
        // A copy entry is "C<score>\told\tnew": the last field exists after
        // the commit either way.
        let path = rest.rsplit('\t').next().unwrap_or(rest);
        let Some(slug) = skill_slug_of(path) else {
            continue;
        };
        // Newest first: the first sighting of a slug is its latest touch,
        // and only that commit can call the skill new.
        let entry = fresh
            .entry(slug.to_string())
            .or_insert_with(|| ChangeStamp {
                status: Some("updated".to_string()),
                at: Some(at.to_string()),
            });
        if entry.at.as_deref() == Some(at) && code.starts_with('A') && path.ends_with("SKILL.md") {
            entry.status = Some("new".to_string());
        }
    }
    fresh
}

/// generate.py `_changed_skill_slugs`: skills whose directory differs between
/// the working tree and HEAD. Untracked (`??`) or staged-as-added (`A`) is
/// new, since the pre-commit build runs after `git add -A`.
fn changed_skill_slugs(root: &Path) -> BTreeMap<String, &'static str> {
    let Some(status) = git_stdout(root, &["status", "--porcelain", "--", "skills"]) else {
        return BTreeMap::new();
    };
    let mut changed = BTreeMap::new();
    for line in status.lines() {
        if line.len() < 4 || !line.is_char_boundary(3) {
            continue;
        }
        let (code, path) = (&line[..2], &line[3..]);
        // A rename's path is "old -> new"; the new one is what exists.
        let path = path.rsplit(" -> ").next().unwrap_or(path);
        let Some(slug) = skill_slug_of(path) else {
            continue;
        };
        let kind = if code == "??" || code.starts_with('A') {
            "new"
        } else {
            "updated"
        };
        if kind == "new" || !changed.contains_key(slug) {
            changed.insert(slug.to_string(), kind);
        }
    }
    changed
}
