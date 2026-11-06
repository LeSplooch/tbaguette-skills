//! Anthropic's plugin directory rules (scripts/test_directory_readiness.py),
//! and the Codex size report (scripts/run_tests.py).
//!
//! The directory reads this repository at every commit on the branch it
//! tracks, validating and scanning each one, so a rule broken by one careless
//! push blocks or holds the version that push produced and every version after
//! it, while the listing quietly keeps serving the last published one. The
//! rules are the directory's own pre-submission checklist, limited to what a
//! repository can check without the portal. Three outcomes, as in the Python
//! suite: *stops validation*, *blocks a submission*, *holds every update for a
//! reviewer*.
//!
//! The plugin folder is the repository root, and the directory reads what the
//! commit contains, which is what git tracks rather than what is on disk.

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

use super::{Repo, git};

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;

// Held for a reviewer past these.
const MAX_TEXT_FILE: u64 = 256 * KIB;
const MAX_FILES: usize = 512;
// Validation stops past these.
const MAX_ANY_FILE: u64 = 5 * MIB;
const MAX_ARCHIVE: usize = 50 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;

/// Symbolic links are a warning where the plugin does not load them and a
/// block where it does. This one points AGENTS.md at CLAUDE.md for other agents
/// working on this repository; no Claude surface loads either file.
const ALLOWED_SYMLINKS: [&str; 1] = ["AGENTS.md"];

const RESERVED_WHOLE_NAMES: [&str; 6] =
    ["claude", "anthropic", "official", "plugin", "mcp", "test"];
const SYSTEM_FILES: [&str; 3] = [".DS_Store", "Thumbs.db", "desktop.ini"];
const PACKAGE_SOURCE_CONFIGS: [&str; 6] = [
    ".npmrc",
    "bunfig.toml",
    "uv.toml",
    ".yarnrc",
    ".yarnrc.yml",
    "pip.conf",
];
const HOOK_EVENTS: [&str; 12] = [
    "SessionStart",
    "SessionEnd",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Notification",
    "Stop",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PermissionRequest",
];

/// The WASM front end. GitHub Pages serves docs/ from master, so the modules are
/// on master, and the directory holds any version carrying a binary other than
/// an image or a font, and any file past 256 KiB, for a reviewer. The plugin is
/// not listed yet, so today that costs nothing. Ruling, the requester's, on
/// 2026-11-06: ship the WASM in docs/ on master and accept the hold. Named one by
/// one, by full path, so any other binary still fails both gates.
const ACCEPTED_WASM: [&str; 2] = [
    "docs/assets/atelier_web_bg.wasm",
    "docs/assets/atelier_graph_bg.wasm",
];

/// Images and fonts are exempt from the 256 KiB rule, and are the only
/// binaries the validator reads.
fn is_exempt_binary(path: &str) -> bool {
    let lower = path.to_lowercase();
    [
        "png", "jpg", "jpeg", "gif", "webp", "svg", "woff", "woff2", "ttf", "otf",
    ]
    .iter()
    .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn size_of(repo: &Repo, rel: &str) -> Result<u64> {
    Ok(std::fs::metadata(repo.path(rel))
        .with_context(|| format!("reading the size of {rel}"))?
        .len())
}

fn manifest(repo: &Repo) -> Result<Value> {
    repo.json(".claude-plugin/plugin.json")
}

/// Python truthiness of a JSON value.
fn is_set(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

// ---- layout that stops validation ----

pub fn manifest_root(repo: &Repo) -> Result<Vec<String>> {
    Ok(if repo.is_file(".claude-plugin/plugin.json") {
        Vec::new()
    } else {
        vec!["the manifest is not at the plugin root: .claude-plugin/plugin.json".to_string()]
    })
}

pub fn five_mib(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for path in repo.regular_files()? {
        let size = size_of(repo, path)?;
        if size >= MAX_ANY_FILE {
            problems.push(format!("{path}: {size} bytes, 5 MiB or more"));
        }
    }
    Ok(problems)
}

pub fn entry_count(repo: &Repo) -> Result<Vec<String>> {
    let tracked = repo.tracked()?;
    let mut folders: Vec<&str> = tracked
        .iter()
        .map(|t| t.path.rsplit_once('/').map_or(".", |(parent, _)| parent))
        .collect();
    folders.sort_unstable();
    folders.dedup();
    let entries = tracked.len() + folders.len();
    Ok(if entries < MAX_ENTRIES {
        Vec::new()
    } else {
        vec![format!("{entries} entries, past {MAX_ENTRIES}")]
    })
}

pub fn archive_size(repo: &Repo) -> Result<Vec<String>> {
    let size = git(&repo.root, &["archive", "--format=tar.gz", "HEAD"])?.len();
    Ok(if size < MAX_ARCHIVE {
        Vec::new()
    } else {
        vec![format!(
            "the archive GitHub serves is {size} bytes, past 50 MiB"
        )]
    })
}

pub fn portable_names(repo: &Repo) -> Result<Vec<String>> {
    let forbidden = Regex::new(r#"[:<>"|?*\\]"#).expect("static regex");
    let devices: Vec<String> = ["con", "prn", "aux", "nul"]
        .iter()
        .map(|d| (*d).to_string())
        .chain((1..=9).flat_map(|i| [format!("com{i}"), format!("lpt{i}")]))
        .collect();
    let mut problems = Vec::new();
    for tracked in repo.tracked()? {
        let bad = tracked.path.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or_default().to_lowercase();
            forbidden.is_match(part) || part.ends_with(['.', ' ']) || devices.contains(&stem)
        });
        if bad {
            problems.push(format!(
                "{}: not a valid name on Windows and macOS",
                tracked.path
            ));
        }
    }
    Ok(problems)
}

pub fn case_clash(repo: &Repo) -> Result<Vec<String>> {
    let mut seen: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
    let mut problems = Vec::new();
    for tracked in repo.tracked()? {
        if let Some(earlier) = seen.insert(tracked.path.to_lowercase(), &tracked.path) {
            problems.push(format!(
                "{earlier} and {} differ only by case",
                tracked.path
            ));
        }
    }
    Ok(problems)
}

pub fn gitattributes(repo: &Repo) -> Result<Vec<String>> {
    let rewriting =
        Regex::new(r"\bexport-ignore\b|\bexport-subst\b|\bfilter\b").expect("static regex");
    let mut problems = Vec::new();
    for tracked in repo.tracked()? {
        if tracked.path.rsplit('/').next() != Some(".gitattributes") {
            continue;
        }
        for line in repo.read(&tracked.path)?.lines() {
            if !line.trim_start().starts_with('#') && rewriting.is_match(line) {
                problems.push(format!("{}: {line}", tracked.path));
            }
        }
    }
    Ok(problems)
}

// ---- what blocks a submission ----

pub fn plugin_name(repo: &Repo) -> Result<Vec<String>> {
    let manifest = manifest(repo)?;
    let name = manifest["name"].as_str().unwrap_or_default();
    let shape = Regex::new(r"^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$").expect("static regex");
    Ok(if shape.is_match(name) {
        Vec::new()
    } else {
        vec![format!(
            "the plugin name {name:?} is not lowercase ASCII kebab-case (letters, digits, hyphens, at most 64)"
        )]
    })
}

pub fn reserved_name(repo: &Repo) -> Result<Vec<String>> {
    let manifest = manifest(repo)?;
    let name = manifest["name"].as_str().unwrap_or_default();
    Ok(if RESERVED_WHOLE_NAMES.contains(&name) {
        vec![format!("the plugin name {name:?} is a reserved word")]
    } else {
        Vec::new()
    })
}

pub fn ascii_names(repo: &Repo) -> Result<Vec<String>> {
    let manifest = manifest(repo)?;
    let mut problems = Vec::new();
    for (label, value) in [
        (
            "displayName",
            manifest["displayName"].as_str().unwrap_or_default(),
        ),
        (
            "author.name",
            manifest["author"]["name"].as_str().unwrap_or_default(),
        ),
    ] {
        // str.isascii() and str.isprintable(): printable ASCII only.
        if !value.chars().all(|c| (' '..='~').contains(&c)) {
            problems.push(format!("{label} {value:?} is not plain printable ASCII"));
        }
    }
    Ok(problems)
}

pub fn required_fields(repo: &Repo) -> Result<Vec<String>> {
    let manifest = manifest(repo)?;
    Ok(["description", "author", "version"]
        .iter()
        .filter(|key| !is_set(&manifest[**key]))
        .map(|key| format!("{key} is not set in .claude-plugin/plugin.json"))
        .collect())
}

pub fn licence(repo: &Repo) -> Result<Vec<String>> {
    let manifest = manifest(repo)?;
    Ok(if is_set(&manifest["license"]) || repo.is_file("LICENSE") {
        Vec::new()
    } else {
        vec!["no licence: neither a license field in the manifest nor a LICENSE file".to_string()]
    })
}

pub fn readme(repo: &Repo) -> Result<Vec<String>> {
    let fenced = Regex::new(r"(?s)```.*?```").expect("static regex");
    let word = Regex::new(r"\w+").expect("static regex");
    let text = repo.read("README.md")?;
    let words = word.find_iter(&fenced.replace_all(&text, "")).count();
    Ok(if words >= 40 {
        Vec::new()
    } else {
        vec![format!(
            "README.md has {words} words outside code, needs 40"
        )]
    })
}

pub fn litter(repo: &Repo) -> Result<Vec<String>> {
    Ok(repo
        .tracked()?
        .iter()
        .filter(|t| {
            SYSTEM_FILES.contains(&t.path.rsplit('/').next().unwrap_or_default())
                || t.path.split('/').any(|part| part == "__MACOSX")
        })
        .map(|t| format!("{} is operating-system litter", t.path))
        .collect())
}

pub fn hook_events(repo: &Repo) -> Result<Vec<String>> {
    let hooks = repo.json("hooks/hooks.json")?;
    let Some(events) = hooks["hooks"].as_object() else {
        return Ok(vec![
            "hooks/hooks.json: \"hooks\" is not an object".to_string(),
        ]);
    };
    let unknown: Vec<&String> = events
        .keys()
        .filter(|e| !HOOK_EVENTS.contains(&e.as_str()))
        .collect();
    Ok(if unknown.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "hooks/hooks.json uses events the directory does not know: {unknown:?}"
        )]
    })
}

pub fn package_launchers(repo: &Repo) -> Result<Vec<String>> {
    let launcher = Regex::new(r"\b(npx|bunx|pnpm\s+dlx|yarn\s+dlx|uvx|pipx\s+run|uv\s+run)\b")
        .expect("static regex");
    let hooks = repo.json("hooks/hooks.json")?;
    let mut texts: Vec<(String, String)> = Vec::new();
    for groups in hooks["hooks"]
        .as_object()
        .into_iter()
        .flat_map(|o| o.values())
    {
        for group in groups.as_array().into_iter().flatten() {
            for hook in group["hooks"].as_array().into_iter().flatten() {
                if let Some(command) = hook["command"].as_str() {
                    texts.push((
                        format!("a hooks/hooks.json command ({command:?})"),
                        command.to_string(),
                    ));
                }
            }
        }
    }
    for script in ["run-hook.cmd", "session-start", "user-prompt-submit"] {
        let rel = format!("hooks/{script}");
        if repo.is_file(&rel) {
            texts.push((rel.clone(), repo.read(&rel)?));
        }
    }
    Ok(texts
        .into_iter()
        .filter_map(|(label, text)| {
            launcher
                .find(&text)
                .map(|m| format!("{label} runs a package launcher ({})", m.as_str()))
        })
        .collect())
}

pub fn package_configs(repo: &Repo) -> Result<Vec<String>> {
    Ok(repo
        .tracked()?
        .iter()
        .filter(|t| PACKAGE_SOURCE_CONFIGS.contains(&t.path.rsplit('/').next().unwrap_or_default()))
        .map(|t| format!("{} redirects package installs", t.path))
        .collect())
}

// ---- what holds every update for a reviewer ----

pub fn text_256kib(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for path in repo.regular_files()? {
        if is_exempt_binary(path) || ACCEPTED_WASM.contains(&path) {
            continue;
        }
        let size = size_of(repo, path)?;
        if size >= MAX_TEXT_FILE {
            problems.push(format!("{path}: {size} bytes, 256 KiB or more"));
        }
    }
    Ok(problems)
}

pub fn file_count(repo: &Repo) -> Result<Vec<String>> {
    let count = repo.tracked()?.len();
    Ok(if count <= MAX_FILES {
        Vec::new()
    } else {
        vec![format!(
            "{count} files; past {MAX_FILES} every version waits for a reviewer"
        )]
    })
}

pub fn binaries(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for path in repo.regular_files()? {
        if is_exempt_binary(path) || ACCEPTED_WASM.contains(&path) {
            continue;
        }
        let bytes = std::fs::read(repo.path(path)).with_context(|| format!("reading {path}"))?;
        if bytes.iter().take(8192).any(|b| *b == 0) {
            problems.push(format!("{path} is a binary that is not an image or a font"));
        }
    }
    Ok(problems)
}

pub fn symlinks(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for tracked in repo.tracked()? {
        match tracked.mode.as_str() {
            "120000" if !ALLOWED_SYMLINKS.contains(&tracked.path.as_str()) => {
                problems.push(format!(
                    "{} is a symlink, which a plugin that loads it would block",
                    tracked.path
                ));
            }
            "160000" => problems.push(format!("{} is a git submodule", tracked.path)),
            _ => {}
        }
    }
    Ok(problems)
}

pub fn lfs(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for path in repo.regular_files()? {
        let bytes = std::fs::read(repo.path(path)).with_context(|| format!("reading {path}"))?;
        if bytes.starts_with(b"version https://git-lfs") {
            problems.push(format!("{path} is a Git LFS pointer"));
        }
    }
    Ok(problems)
}

// ---- the Codex size report ----

/// Codex truncates any SKILL.md loaded through an Agent Plugin manifest at
/// 8,000 bytes (codex-rs/ext/skills/src/render.rs, MAX_SKILL_PROMPT_BYTES);
/// this repo's .codex-plugin/plugin.json is exactly that path. Report-only:
/// nothing here is a defect this repo can fix by editing prose, and a hard gate
/// would turn "this skill grew" into a build failure for a limit this repo does
/// not control.
const SKILL_SIZE_CAP: u64 = 8000;

fn with_commas(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// How many SKILL.md files exceed the Codex cap, and the five largest.
pub fn codex_report(repo: &Repo) -> Result<String> {
    let mut total = 0;
    let mut over: Vec<(u64, String)> = Vec::new();
    for slug in repo.skill_dirs()? {
        let rel = format!("skills/{slug}/SKILL.md");
        if repo.is_file(&rel) {
            total += 1;
            let size = size_of(repo, &rel)?;
            if size > SKILL_SIZE_CAP {
                over.push((size, slug));
            }
        }
    }
    over.sort();
    over.reverse();
    let rule = "=".repeat(70);
    let mut report = format!(
        "{rule}\nskill size report (informational, not a gate)\n{rule}\n\
         {} / {total} SKILL.md files exceed {SKILL_SIZE_CAP} bytes \u{2014} truncated on Codex via the \
         documented Agent Plugin install.\n",
        over.len()
    );
    for (size, name) in over.iter().take(5) {
        report.push_str(&format!("  {:>7} {name}\n", with_commas(*size)));
    }
    Ok(report)
}
