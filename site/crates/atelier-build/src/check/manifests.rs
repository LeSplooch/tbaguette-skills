//! The harness manifests (scripts/test_harness_manifests.py): every
//! integration this repo ships is well formed and agrees with
//! `.claude-plugin/plugin.json`, the source of truth for the plugin's name
//! and version.

use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

use super::Repo;

const SOURCE_OF_TRUTH: &str = ".claude-plugin/plugin.json";

const JSON_MANIFESTS: &[&str] = &[
    ".claude-plugin/plugin.json",
    ".claude-plugin/marketplace.json",
    ".agents/plugins/marketplace.json",
    "plugin.json",
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "com.github.copilot/hooks/hooks.json",
    "gemini-extension.json",
    "hooks/hooks.json",
    "hooks/hooks-codex.json",
    "hooks/hooks-copilot.json",
    "hooks/hooks-cursor.json",
    "package.json",
];

/// Followers of the source of truth. `package.json` is here because it drifted
/// five minor versions once when nothing compared it.
const VERSIONED_MANIFESTS: &[&str] = &[
    ".codex-plugin/plugin.json",
    ".cursor-plugin/plugin.json",
    ".devin-plugin/plugin.json",
    ".kimi-plugin/plugin.json",
    "gemini-extension.json",
    "package.json",
    "plugin.json",
];

const YAML_MANIFEST: &str = "plugin.yaml";

/// `plugin.yaml` is flat `key: value` lines, parsed by hand like the Python
/// suite did: the repository has no YAML dependency.
fn yaml_scalar(repo: &Repo, key: &str) -> Result<Option<String>> {
    for line in repo.read(YAML_MANIFEST)?.lines() {
        if line.starts_with([' ', '-', '#']) {
            continue;
        }
        if let Some((k, v)) = line.split_once(':')
            && k.trim() == key
        {
            return Ok(Some(v.trim().to_string()));
        }
    }
    Ok(None)
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

pub fn json_valid(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for rel in JSON_MANIFESTS {
        if !repo.is_file(rel) {
            problems.push(format!("{rel} missing"));
        } else if let Err(error) = repo.json(rel) {
            problems.push(format!("{error:#}"));
        }
    }
    Ok(problems)
}

pub fn versions(repo: &Repo) -> Result<Vec<String>> {
    let source = repo.json(SOURCE_OF_TRUTH)?;
    let expected = text(&source, "version")
        .ok_or_else(|| anyhow::anyhow!("{SOURCE_OF_TRUTH} has no version"))?
        .to_string();
    let mut problems = Vec::new();
    let mut compare = |label: String, found: Option<String>| match found {
        Some(v) if v == expected => {}
        Some(v) => problems.push(format!(
            "{label}: version {v} but {SOURCE_OF_TRUTH} says {expected}"
        )),
        None => problems.push(format!("{label}: no version (expected {expected})")),
    };
    for rel in VERSIONED_MANIFESTS {
        let data = repo.json(rel)?;
        compare(
            (*rel).to_string(),
            text(&data, "version").map(str::to_string),
        );
    }
    let marketplace = repo.json(".claude-plugin/marketplace.json")?;
    compare(
        ".claude-plugin/marketplace.json (plugins[0])".to_string(),
        marketplace["plugins"][0]["version"]
            .as_str()
            .map(str::to_string),
    );
    compare(YAML_MANIFEST.to_string(), yaml_scalar(repo, "version")?);
    Ok(problems)
}

pub fn agents_md_symlink(repo: &Repo) -> Result<Vec<String>> {
    let agents = repo.path("AGENTS.md");
    let is_link = std::fs::symlink_metadata(&agents).is_ok_and(|m| m.file_type().is_symlink());
    if !is_link {
        return Ok(vec!["AGENTS.md must be a symlink".to_string()]);
    }
    let points_at = agents.canonicalize().ok();
    if points_at != repo.path("CLAUDE.md").canonicalize().ok() {
        return Ok(vec![
            "AGENTS.md is a symlink, but not to CLAUDE.md".to_string(),
        ]);
    }
    Ok(Vec::new())
}

/// Copilot CLI searches for hooks.json when the manifest does not name one, and
/// this repo's hooks/hooks.json is in Claude Code's incompatible schema.
pub fn copilot_hooks_field(repo: &Repo) -> Result<Vec<String>> {
    let manifest = repo.json("plugin.json")?;
    let mut problems = Vec::new();
    if text(&manifest, "hooks") != Some("./hooks/hooks-copilot.json") {
        problems.push(format!(
            "plugin.json: hooks is {}, expected \"./hooks/hooks-copilot.json\"",
            manifest
                .get("hooks")
                .map_or("absent".to_string(), Value::to_string)
        ));
    }
    if !repo.is_file("hooks/hooks-copilot.json") {
        problems.push("hooks/hooks-copilot.json is missing".to_string());
    }
    if text(&manifest, "skills") != Some("./skills/") {
        problems.push(format!(
            "plugin.json: skills is {}, expected \"./skills/\"",
            manifest
                .get("skills")
                .map_or("absent".to_string(), Value::to_string)
        ));
    }
    Ok(problems)
}

/// The repo root is the only location on both Copilot surfaces' search lists.
pub fn copilot_root(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    if !repo.is_file("plugin.json") {
        problems.push("plugin.json is missing from the repo root".to_string());
    }
    for tidier in [".copilot-plugin", ".github/plugin"] {
        if repo.path(tidier).exists() {
            problems.push(format!(
                "{tidier}/ is not read by both Copilot surfaces -- the manifest belongs at the repo root"
            ));
        }
    }
    Ok(problems)
}

/// Hermes loads `plugin.yaml` and `__init__.py` from the plugin directory root
/// and nowhere else; the tidier `.hermes-plugin/` once broke the install.
pub fn hermes_layout(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for rel in ["plugin.yaml", "__init__.py"] {
        if !repo.is_file(rel) {
            problems.push(format!("{rel} is missing from the repo root"));
        }
    }
    if repo.path(".hermes-plugin").exists() {
        problems.push(
            ".hermes-plugin/ is invisible to Hermes and shadows the install behind the portable \
             plugin.json reader -- both files belong at the repo root"
                .to_string(),
        );
    }
    // Hermes qualifies every skill as `<manifest name>:<skill>`.
    match yaml_scalar(repo, "name")?.as_deref() {
        Some("tbaguette-atelier") => {}
        other => problems.push(format!(
            "plugin.yaml: name is {other:?}, but every tbaguette-atelier:<skill> reference needs \"tbaguette-atelier\""
        )),
    }
    Ok(problems)
}

fn is_kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

pub fn names(repo: &Repo) -> Result<Vec<String>> {
    let source = repo.json(SOURCE_OF_TRUTH)?;
    let expected = text(&source, "name").unwrap_or_default().to_string();
    let mut problems = Vec::new();
    if !is_kebab(&expected) || expected.chars().count() > 64 {
        problems.push(format!(
            "{SOURCE_OF_TRUTH}: name {expected:?} is not kebab-case of at most 64 characters"
        ));
    }
    let mut named: Vec<(String, Option<String>)> = Vec::new();
    for rel in [
        "plugin.json",
        ".codex-plugin/plugin.json",
        ".cursor-plugin/plugin.json",
        ".devin-plugin/plugin.json",
        ".kimi-plugin/plugin.json",
        "gemini-extension.json",
    ] {
        named.push((
            rel.to_string(),
            text(&repo.json(rel)?, "name").map(str::to_string),
        ));
    }
    for rel in [
        ".claude-plugin/marketplace.json",
        ".agents/plugins/marketplace.json",
    ] {
        let data = repo.json(rel)?;
        named.push((
            format!("{rel} (plugins[0])"),
            data["plugins"][0]["name"].as_str().map(str::to_string),
        ));
    }
    named.push((YAML_MANIFEST.to_string(), yaml_scalar(repo, "name")?));
    for (rel, name) in named {
        if name.as_deref() != Some(expected.as_str()) {
            problems.push(format!(
                "{rel}: name {:?} but {SOURCE_OF_TRUTH} says {expected:?}",
                name.unwrap_or_default()
            ));
        }
    }
    Ok(problems)
}

/// VS Code ignores a manifest's hooks field and derives the path from the
/// detected plugin format; the `$schema` line is what routes it.
pub fn agent_plugins_schema(repo: &Repo) -> Result<Vec<String>> {
    let manifest = repo.json("plugin.json")?;
    let mut problems = Vec::new();
    if text(&manifest, "$schema")
        != Some("https://agent-plugins.org/schemas/1.0.0/plugin.schema.json")
    {
        problems.push(
            "plugin.json: $schema is not the Agent Plugins 1.0.0 line -- without it VS Code stops \
             finding com.github.copilot/hooks/hooks.json"
                .to_string(),
        );
    }
    if !repo.is_file("com.github.copilot/hooks/hooks.json") {
        problems.push("com.github.copilot/hooks/hooks.json is missing".to_string());
    }
    Ok(problems)
}

fn event_names(data: &Value) -> BTreeSet<String> {
    data["hooks"]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn set_of(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

/// The CLI's file is camelCase, VS Code's PascalCase; swapping them leaves
/// both valid and one surface without a bootstrap.
pub fn copilot_hook_casing(repo: &Repo) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for (rel, expected) in [
        (
            "hooks/hooks-copilot.json",
            set_of(&["sessionStart", "userPromptSubmitted"]),
        ),
        (
            "com.github.copilot/hooks/hooks.json",
            set_of(&["SessionStart", "UserPromptSubmit"]),
        ),
    ] {
        let found = event_names(&repo.json(rel)?);
        if found != expected {
            problems.push(format!(
                "{rel}: events are {found:?}, expected {expected:?}"
            ));
        }
    }
    Ok(problems)
}

/// `copilot plugin install tbaguette-atelier@tbaguette-dev` is printed on the
/// live site; both halves come from marketplace.json and are not derived.
pub fn copilot_install_spec(repo: &Repo) -> Result<Vec<String>> {
    let marketplace = repo.json(".claude-plugin/marketplace.json")?;
    let plugin = &marketplace["plugins"][0];
    let spec = format!(
        "{}@{}",
        plugin["name"].as_str().unwrap_or_default(),
        marketplace["name"].as_str().unwrap_or_default()
    );
    let mut problems = Vec::new();
    if spec != "tbaguette-atelier@tbaguette-dev" {
        problems.push(format!(
            ".claude-plugin/marketplace.json: the install spec is {spec:?}, but the site publishes \
             \"tbaguette-atelier@tbaguette-dev\""
        ));
    }
    let porting = repo.read("PORTING.md")?;
    let command = format!("copilot plugin install {spec}");
    if !porting.contains(&command) {
        problems.push(format!("PORTING.md does not contain {command:?}"));
    }
    let declarative = format!("\"{spec}\": true");
    if !porting.contains(&declarative) {
        problems.push(format!("PORTING.md does not contain {declarative:?}"));
    }
    let source = plugin["source"].as_str().unwrap_or_default();
    if source != "./" {
        problems.push(format!(
            ".claude-plugin/marketplace.json: source is {source:?}, expected \"./\""
        ));
    } else if !repo.is_file("plugin.json") {
        problems.push("the marketplace source ./ has no plugin.json".to_string());
    }
    Ok(problems)
}

/// An empty `"hooks": {}` once cost Codex its whole bootstrap.
pub fn codex_hooks(repo: &Repo) -> Result<Vec<String>> {
    let manifest = repo.json(".codex-plugin/plugin.json")?;
    let mut problems = Vec::new();
    if text(&manifest, "hooks") != Some("./hooks/hooks-codex.json") {
        problems.push(format!(
            ".codex-plugin/plugin.json: hooks is {}, expected \"./hooks/hooks-codex.json\"",
            manifest
                .get("hooks")
                .map_or("absent".to_string(), Value::to_string)
        ));
    }
    if !repo.is_file("hooks/hooks-codex.json") {
        problems.push("hooks/hooks-codex.json is missing".to_string());
    }
    Ok(problems)
}

/// Five hook configs in four schemas, all of which must name a script that exists.
pub fn hook_scripts(repo: &Repo) -> Result<Vec<String>> {
    let configs: [(&str, &[&str]); 5] = [
        (
            "hooks/hooks.json",
            &[
                "SessionStart",
                "UserPromptSubmit",
                "SubagentStart",
                "PostToolUse",
            ],
        ),
        (
            "hooks/hooks-codex.json",
            &["SessionStart", "UserPromptSubmit"],
        ),
        (
            "hooks/hooks-copilot.json",
            &["sessionStart", "userPromptSubmitted"],
        ),
        (
            "com.github.copilot/hooks/hooks.json",
            &["SessionStart", "UserPromptSubmit"],
        ),
        ("hooks/hooks-cursor.json", &["sessionStart", "postToolUse"]),
    ];
    let mut problems = Vec::new();
    for (rel, events) in configs {
        let data = repo.json(rel)?;
        let found = event_names(&data);
        if found != set_of(events) {
            problems.push(format!(
                "{rel}: events are {found:?}, expected {:?}",
                set_of(events)
            ));
        }
        let commands = data["hooks"].to_string();
        if !commands.contains("run-hook.cmd") {
            problems.push(format!("{rel} does not route through run-hook.cmd"));
        }
        for script in [
            "session-start",
            "user-prompt-submit",
            "subagent-start",
            "post-tool-use",
        ] {
            if commands.contains(script) && !repo.is_file(&format!("hooks/{script}")) {
                problems.push(format!("{rel} names hooks/{script}, which does not exist"));
            }
        }
    }
    Ok(problems)
}

pub fn package_files(repo: &Repo) -> Result<Vec<String>> {
    let package = repo.json("package.json")?;
    let mut problems = Vec::new();
    let main = text(&package, "main").unwrap_or_default();
    if !repo.is_file(main) {
        problems.push(format!("package.json main={main:?} does not exist"));
    }
    for ext in package["pi"]["extensions"].as_array().into_iter().flatten() {
        let ext = ext.as_str().unwrap_or_default();
        if !repo.is_file(ext) {
            problems.push(format!("pi extension {ext:?} does not exist"));
        }
    }
    Ok(problems)
}

/// `repository`, `homepage` and `keywords` were added to the manifests that
/// lacked them in 2026-09; one of the manifests that had them was already
/// missing keywords, because nothing compared them.
pub fn shared_fields(repo: &Repo) -> Result<Vec<String>> {
    const REPOSITORY: &str = "https://github.com/LeSplooch/tbaguette-skills";
    const HOMEPAGE: &str = "https://lesplooch.github.io/tbaguette-skills/";
    let keywords = set_of(&[
        "skills",
        "code-review",
        "debugging",
        "testing",
        "systems-design",
        "security",
        "naming",
        "workflow",
    ]);
    let mut problems = Vec::new();
    for rel in [
        ".claude-plugin/plugin.json",
        ".claude-plugin/marketplace.json",
        ".codex-plugin/plugin.json",
        ".cursor-plugin/plugin.json",
        ".devin-plugin/plugin.json",
        ".kimi-plugin/plugin.json",
        "plugin.json",
    ] {
        let data = repo.json(rel)?;
        let data = if rel == ".claude-plugin/marketplace.json" {
            data["plugins"][0].clone()
        } else {
            data
        };
        if text(&data, "repository") != Some(REPOSITORY) {
            problems.push(format!(
                "{rel}: repository is {:?}, expected {REPOSITORY:?}",
                text(&data, "repository")
            ));
        }
        if text(&data, "homepage") != Some(HOMEPAGE) {
            problems.push(format!(
                "{rel}: homepage is {:?}, expected {HOMEPAGE:?}",
                text(&data, "homepage")
            ));
        }
        let found: BTreeSet<String> = data["keywords"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect();
        if found != keywords {
            problems.push(format!(
                "{rel}: keywords are {found:?}, expected {keywords:?}"
            ));
        }
    }
    Ok(problems)
}
