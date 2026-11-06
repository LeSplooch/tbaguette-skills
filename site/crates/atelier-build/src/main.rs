//! `atelier-build`: the site build and the repository's checks, one
//! subcommand each. One match arm per subcommand, each handing its own
//! arguments to a function below.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use atelier_build::check;
use atelier_build::write::{BuildOptions, build_site_with};
use chrono::{DateTime, SubsecRound, Utc};

const USAGE: &str =
    "usage: atelier-build <build|check|crumb-check|agents|router|archive-updates|wasm-stamp> [options]

  build [--base-path P] [--now T] [--history-rev REV] [--root DIR] [--out DIR]
      Build the site into DIR (default <root>/docs), swapping each output in
      by rename. --base-path prefixes every root-relative URL
      (/tbaguette-skills for the published site; default empty). --now is the
      build instant as RFC 3339 (default: the clock). --history-rev is the
      revision whose git history decides the New/Updated badges (default
      HEAD, with the working tree's uncommitted changes on top). --root is
      the repository (default: the git top level of the current directory).

  check                     every repo gate; non-zero exit names each failure
  crumb-check <rev> [--live]  the Crumb at <rev> matches its skills (and the live site)
  agents [--check]          render or verify the Copilot and Claude Code agents
  router [--check]          build or verify hooks/skill-index.tsv, the per-turn hook's table
  archive-updates           move update notes past the newest thirty into updates-archive/
  wasm-stamp write ...      site/build.sh's record of what built the WASM modules
  wasm-stamp check [REV]    the WASM modules match their sources, in this tree or at REV";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let rest: Vec<String> = args.collect();
    let result = match command.as_deref() {
        Some("build") => build(&rest),
        Some("check") => return check::run_check(rest),
        Some("crumb-check") => return check::crumb::run_crumb_check(rest),
        Some("agents") => return check::agents::run_agents(rest),
        Some("router") => return check::router::run_router(rest),
        Some("archive-updates") => return check::archive::run_archive_updates(rest),
        Some("wasm-stamp") => return check::wasm_stamp::run_wasm_stamp(rest),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("atelier-build: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// `--name value` or `--name=value` options from `names`, each at most once;
/// anything else is an error naming it.
fn parse_options(args: &[String], names: &[&str]) -> Result<Vec<(String, String)>> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (arg.as_str(), None),
        };
        let Some(name) = name.strip_prefix("--").filter(|n| names.contains(n)) else {
            bail!("unexpected argument `{arg}`\n\n{USAGE}");
        };
        let value = match inline {
            Some(value) => value,
            None => args
                .next()
                .cloned()
                .ok_or_else(|| anyhow!("--{name} needs a value"))?,
        };
        if found.iter().any(|(n, _)| n == name) {
            bail!("--{name} given twice");
        }
        found.push((name.to_string(), value));
    }
    Ok(found)
}

/// The repository the current directory is in, or the current directory
/// itself when git cannot say.
fn default_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(output) = output
        && output.status.success()
    {
        let top = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !top.is_empty() {
            return Ok(PathBuf::from(top));
        }
    }
    std::env::current_dir().context("reading the current directory")
}

/// `atelier-build build`.
fn build(args: &[String]) -> Result<ExitCode> {
    let options = parse_options(args, &["base-path", "now", "history-rev", "root", "out"])?;
    let get = |name: &str| {
        options
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    };
    let now = match get("now") {
        Some(text) => DateTime::parse_from_rfc3339(text)
            .with_context(|| format!("--now `{text}` is not an RFC 3339 instant"))?
            .with_timezone(&Utc),
        // Python's clock stops at microseconds, and a working-tree badge
        // carries the build instant at that precision.
        None => Utc::now().trunc_subsecs(6),
    };
    let root = match get("root") {
        Some(root) => PathBuf::from(root),
        None => default_root()?,
    };
    let out = get("out").map_or_else(|| root.join("docs"), PathBuf::from);
    let options = BuildOptions {
        base_path: get("base-path").unwrap_or_default().to_string(),
        now,
        history_rev: get("history-rev")
            .unwrap_or(atelier_build::repo::HEAD)
            .to_string(),
    };
    build_site_with(&root, &out, &options)?;
    println!(
        "built {} from {} (base path {:?}, instant {})",
        out.display(),
        root.display(),
        options.base_path,
        options.now.to_rfc3339(),
    );
    Ok(ExitCode::SUCCESS)
}
