//! The repo gates: everything `scripts/run_tests.py` held that is about the
//! plugin's files rather than the site's pages, plus the tools that sit beside
//! them (`crumb-check`, `agents`, `archive-updates`, `wasm-stamp`).
//!
//! A gate is a function from a [`Repo`] to a list of problems; an empty list
//! means it holds. `atelier-build check` runs every one, prints one line per
//! gate, and exits non-zero naming each problem. Gates read the working tree
//! (and `git ls-files` for the directory-limit rules, which concern what a
//! commit will contain), so a test can break one thing in a temp copy and
//! watch exactly that gate say so.

use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;

pub mod agents;
pub mod archive;
pub mod catalog;
pub mod crumb;
pub mod frontmatter;
pub mod limits;
pub mod manifests;
pub mod notes;
pub mod references;
pub mod router;
pub mod wasm_stamp;

/// One tracked file as `git ls-files -s` lists it.
#[derive(Debug, Clone)]
pub struct Tracked {
    /// The octal mode: `100644`, `120000` for a symlink, `160000` for a submodule.
    pub mode: String,
    pub path: String,
}

/// The repository the gates look at.
pub struct Repo {
    pub root: PathBuf,
    tracked: OnceCell<Vec<Tracked>>,
}

impl Repo {
    pub fn new(root: impl Into<PathBuf>) -> Repo {
        Repo {
            root: root.into(),
            tracked: OnceCell::new(),
        }
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub fn read(&self, rel: &str) -> Result<String> {
        let path = self.path(rel);
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    pub fn json(&self, rel: &str) -> Result<Value> {
        let text = self.read(rel)?;
        serde_json::from_str(&text).map_err(|e| anyhow!("{rel}: not valid JSON: {e}"))
    }

    pub fn is_file(&self, rel: &str) -> bool {
        self.path(rel).is_file()
    }

    /// Every directory under `skills/`, sorted (the library's slugs).
    pub fn skill_dirs(&self) -> Result<Vec<String>> {
        let dir = self.path("skills");
        let mut slugs = Vec::new();
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
        {
            let entry = entry?;
            if entry.path().is_dir() {
                slugs.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        slugs.sort();
        Ok(slugs)
    }

    /// Every file git will put in the next commit, with its mode.
    pub fn tracked(&self) -> Result<&[Tracked]> {
        if self.tracked.get().is_none() {
            let out = git(&self.root, &["ls-files", "-s", "-z"])?;
            let mut entries = Vec::new();
            for record in out.split(|b| *b == 0).filter(|r| !r.is_empty()) {
                let record = String::from_utf8_lossy(record);
                let (meta, path) = record
                    .split_once('\t')
                    .ok_or_else(|| anyhow!("unexpected `git ls-files -s` record {record:?}"))?;
                let mode = meta.split(' ').next().unwrap_or_default().to_string();
                entries.push(Tracked {
                    mode,
                    path: path.to_string(),
                });
            }
            let _ = self.tracked.set(entries);
        }
        Ok(self.tracked.get().expect("just set").as_slice())
    }

    /// Tracked paths that are regular files (not symlinks or submodules).
    pub fn regular_files(&self) -> Result<Vec<&str>> {
        Ok(self
            .tracked()?
            .iter()
            .filter(|t| t.mode != "120000" && t.mode != "160000")
            .map(|t| t.path.as_str())
            .collect())
    }
}

/// Run `git args...` in `dir` and return its stdout, or an error carrying its
/// stderr.
pub fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// The top of the work tree containing the current directory.
pub fn current_repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let out = git(&cwd, &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(String::from_utf8_lossy(&out).trim()))
}

/// A gate: a stable id (what tests and failure lines name), a one-line
/// statement of what it holds, and the check itself.
pub struct Gate {
    pub id: &'static str,
    pub what: &'static str,
    pub run: fn(&Repo) -> Result<Vec<String>>,
}

macro_rules! gate {
    ($id:literal, $what:literal, $run:path) => {
        Gate {
            id: $id,
            what: $what,
            run: $run,
        }
    };
}

/// Every gate, in the order `check` runs and lists them.
pub fn gates() -> Vec<Gate> {
    vec![
        gate!(
            "manifest-json",
            "every harness manifest exists and is valid JSON",
            manifests::json_valid
        ),
        gate!(
            "manifest-versions",
            "the ten manifests carry .claude-plugin/plugin.json's version",
            manifests::versions
        ),
        gate!(
            "manifest-agents-md-symlink",
            "AGENTS.md is a symlink to CLAUDE.md",
            manifests::agents_md_symlink
        ),
        gate!(
            "manifest-copilot-hooks-field",
            "the Copilot manifest names its own hook file and the skills directory",
            manifests::copilot_hooks_field
        ),
        gate!(
            "manifest-copilot-root",
            "the Copilot manifest sits at the repo root, not a tidier-looking directory",
            manifests::copilot_root
        ),
        gate!(
            "manifest-hermes-layout",
            "plugin.yaml and __init__.py are siblings at the repo root",
            manifests::hermes_layout
        ),
        gate!(
            "manifest-names",
            "every manifest names the plugin the same kebab-case way",
            manifests::names
        ),
        gate!(
            "manifest-agent-plugins-schema",
            "plugin.json keeps the Agent Plugins schema line VS Code routes hooks by",
            manifests::agent_plugins_schema
        ),
        gate!(
            "manifest-copilot-hook-casing",
            "the two Copilot hook files keep their own event casing",
            manifests::copilot_hook_casing
        ),
        gate!(
            "manifest-copilot-install-spec",
            "the published Copilot install command names a real marketplace",
            manifests::copilot_install_spec
        ),
        gate!(
            "manifest-codex-hooks",
            "the Codex manifest names its hook file",
            manifests::codex_hooks
        ),
        gate!(
            "manifest-hook-scripts",
            "every harness hook file reaches a script that exists",
            manifests::hook_scripts
        ),
        gate!(
            "manifest-package-files",
            "package.json points at files that exist",
            manifests::package_files
        ),
        gate!(
            "manifest-shared-fields",
            "repository, homepage and keywords agree across manifests",
            manifests::shared_fields
        ),
        gate!(
            "frontmatter-required",
            "every skill has a name and a description",
            frontmatter::required
        ),
        gate!(
            "frontmatter-name-matches-dir",
            "each skill's name is its directory name",
            frontmatter::name_matches_dir
        ),
        gate!(
            "frontmatter-name-limits",
            "names are at most 64 characters, lowercase-hyphen, no reserved words",
            frontmatter::name_limits
        ),
        gate!(
            "frontmatter-description-limit",
            "descriptions are at most 1024 characters",
            frontmatter::description_limit
        ),
        gate!(
            "frontmatter-no-xml",
            "no XML tag in a name or description",
            frontmatter::no_xml
        ),
        gate!(
            "catalog-headings",
            "CATALOG.md headings are the categories, in order",
            catalog::headings
        ),
        gate!(
            "catalog-membership",
            "each CATALOG.md section lists exactly its category's skills",
            catalog::membership
        ),
        gate!(
            "catalog-no-duplicates",
            "no skill is listed twice in CATALOG.md",
            catalog::no_duplicates
        ),
        gate!(
            "catalog-foreign",
            "rows marked as foreign are not skills in this repo",
            catalog::foreign
        ),
        gate!(
            "catalog-prose-counts",
            "the spelled-out skill count in manifests and README is the count on disk",
            catalog::prose_counts
        ),
        gate!(
            "catalog-stated-count",
            "CATALOG.md's opening count is the count on disk",
            catalog::stated_count
        ),
        gate!(
            "references-resolve",
            "every parenthetical skill reference names a real skill",
            references::resolve
        ),
        gate!(
            "references-reachable",
            "every skill is cited by at least one other skill",
            references::reachable
        ),
        gate!(
            "references-phase-index",
            "phase-routing.md names every skill",
            references::phase_index
        ),
        gate!(
            "references-exemptions",
            "the reference exemption lists name no real skill",
            references::exemptions
        ),
        gate!(
            "agents-fresh",
            "the committed Copilot and Claude Code agents equal what the renderer writes",
            agents::fresh
        ),
        gate!(
            "agents-copilot-profiles",
            "Copilot agent profiles are well formed, read-only roles cannot edit, no model pins",
            agents::copilot_profiles
        ),
        gate!(
            "agents-claude-profiles",
            "Claude Code agents share the Copilot body and use Claude tool names",
            agents::claude_profiles
        ),
        gate!(
            "hooks-skill-index",
            "hooks/skill-index.tsv is what the router builds from the skills on disk",
            router::fresh
        ),
        gate!(
            "wasm-fresh",
            "the built WASM modules come from the sources in this tree",
            wasm_stamp::fresh
        ),
        gate!(
            "limits-manifest-root",
            "the plugin manifest is at the plugin root",
            limits::manifest_root
        ),
        gate!(
            "limits-five-mib",
            "no tracked file reaches 5 MiB",
            limits::five_mib
        ),
        gate!(
            "limits-entry-count",
            "fewer than ten thousand tracked entries",
            limits::entry_count
        ),
        gate!(
            "limits-archive-size",
            "the archive GitHub serves is under 50 MiB",
            limits::archive_size
        ),
        gate!(
            "limits-portable-names",
            "names are valid on Windows and macOS",
            limits::portable_names
        ),
        gate!(
            "limits-case-clash",
            "no two names differ only by case",
            limits::case_clash
        ),
        gate!(
            "limits-gitattributes",
            ".gitattributes never rewrites or drops content",
            limits::gitattributes
        ),
        gate!(
            "limits-plugin-name",
            "the plugin name is lowercase ASCII kebab-case",
            limits::plugin_name
        ),
        gate!(
            "limits-reserved-name",
            "the plugin name is not a reserved word",
            limits::reserved_name
        ),
        gate!(
            "limits-ascii-names",
            "display and author names are plain ASCII",
            limits::ascii_names
        ),
        gate!(
            "limits-required-fields",
            "description, author and version are set",
            limits::required_fields
        ),
        gate!("limits-licence", "a licence ships", limits::licence),
        gate!(
            "limits-readme",
            "the README has forty words outside code",
            limits::readme
        ),
        gate!(
            "limits-litter",
            "no operating-system litter is tracked",
            limits::litter
        ),
        gate!(
            "limits-hook-events",
            "hooks.json uses only known events",
            limits::hook_events
        ),
        gate!(
            "limits-package-launchers",
            "no hook runs a package launcher",
            limits::package_launchers
        ),
        gate!(
            "limits-package-configs",
            "no file redirects package installs",
            limits::package_configs
        ),
        gate!(
            "limits-text-256kib",
            "every text file is under 256 KiB, the two accepted WASM modules aside",
            limits::text_256kib
        ),
        gate!(
            "limits-file-count",
            "no more than 512 tracked files",
            limits::file_count
        ),
        gate!(
            "limits-binaries",
            "the only binaries are images, fonts and the two accepted WASM modules",
            limits::binaries
        ),
        gate!(
            "limits-symlinks",
            "symlinks only where nothing loads them",
            limits::symlinks
        ),
        gate!("limits-lfs", "no Git LFS pointers", limits::lfs),
        gate!(
            "notes-updates-valid",
            "UPDATES.md parses",
            notes::updates_valid
        ),
        gate!(
            "notes-updates-size",
            "UPDATES.md is under its ceiling",
            notes::updates_size
        ),
        gate!(
            "notes-archives",
            "every archive parses, holds its month and fits the directory",
            notes::archives_hold_their_month
        ),
        gate!(
            "notes-order",
            "the record runs newest first across every file",
            notes::order
        ),
    ]
}

/// Run one gate by id. A gate that could not run at all reports that as its
/// problem, so a missing file fails the gate rather than the process.
///
/// # Panics
///
/// On an id no gate has; that is a typo in a test, not a repo problem.
pub fn run_gate(root: &Path, id: &str) -> Vec<String> {
    let gate = gates()
        .into_iter()
        .find(|g| g.id == id)
        .unwrap_or_else(|| panic!("no gate named {id:?}"));
    run(&Repo::new(root), &gate)
}

fn run(repo: &Repo, gate: &Gate) -> Vec<String> {
    match (gate.run)(repo) {
        Ok(problems) => problems,
        Err(error) => vec![format!("could not run: {error:#}")],
    }
}

/// The outcome of one gate.
pub struct Outcome {
    pub id: &'static str,
    pub what: &'static str,
    pub problems: Vec<String>,
}

/// Run every gate against the repository at `root`.
pub fn run_all(root: &Path) -> Vec<Outcome> {
    let repo = Repo::new(root);
    gates()
        .into_iter()
        .map(|gate| Outcome {
            id: gate.id,
            what: gate.what,
            problems: run(&repo, &gate),
        })
        .collect()
}

/// `atelier-build check [--root DIR]`: run every gate, list each by name,
/// print the Codex size report, and exit 1 when any gate has a problem.
pub fn run_check(args: Vec<String>) -> ExitCode {
    let root = match root_from(&args) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("atelier-build check: {error:#}");
            return ExitCode::from(2);
        }
    };
    let outcomes = run_all(&root);
    let mut failed = 0;
    for outcome in &outcomes {
        if outcome.problems.is_empty() {
            println!("ok    {:<32} {}", outcome.id, outcome.what);
        } else {
            failed += 1;
            println!("FAIL  {:<32} {}", outcome.id, outcome.what);
            for problem in &outcome.problems {
                println!("        - {problem}");
            }
        }
    }
    match limits::codex_report(&Repo::new(&root)) {
        Ok(report) => print!("\n{report}"),
        Err(error) => println!("\n(skill size report unavailable: {error:#})"),
    }
    println!();
    if failed > 0 {
        println!("FAILED: {failed}/{} gates failed.", outcomes.len());
        return ExitCode::from(1);
    }
    println!("All {} gates passed.", outcomes.len());
    ExitCode::SUCCESS
}

/// `--root DIR`, or the work tree containing the current directory.
pub(crate) fn root_from(args: &[String]) -> Result<PathBuf> {
    match args.iter().position(|a| a == "--root") {
        Some(i) => args
            .get(i + 1)
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("--root needs a directory")),
        None => current_repo_root(),
    }
}
