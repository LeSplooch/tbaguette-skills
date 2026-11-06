//! Were the built WASM modules made from the sources in this tree?
//!
//! `docs/assets/*.wasm` are committed, because Pages serves `docs/` as it stands, and only
//! `site/build.sh` writes them: the pre-commit hook's `site/atelier build` makes the pages and
//! leaves the modules alone. So a change to `atelier-web` committed through the hooks would
//! publish the old module while every other gate passed.
//!
//! `site/build.sh` writes a stamp beside the modules: a hash of every source that goes into
//! them, and the versions of the tools that built them. The `wasm-fresh` gate recomputes the
//! hash and reads the wanted versions from `rust-toolchain.toml` and `Cargo.lock`. It starts
//! no tool and reads a few dozen small files, because it runs on every check and every commit.
//!
//! `atelier-core/src/model.rs` stays out of the hash. It holds the skill registry, which every
//! new skill edits and neither module reaches, and filing a skill must not ask for a WASM build.

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

use anyhow::{Context, Result, anyhow, bail};
use sha2::{Digest, Sha256};

use super::crumb::{Scratch, resolve};
use super::{Repo, git, root_from};

/// Beside the modules it speaks for.
pub const STAMP: &str = "docs/assets/wasm-build.stamp";
const TOOLCHAIN: &str = "rust-toolchain.toml";
const LOCKFILE: &str = "site/Cargo.lock";
const REBUILD: &str = "run site/build.sh";

const SOURCE_FILES: [&str; 6] = [
    "site/Cargo.lock",
    "site/Cargo.toml",
    "site/build.sh",
    "site/crates/atelier-core/Cargo.toml",
    "site/crates/atelier-graph/Cargo.toml",
    "site/crates/atelier-web/Cargo.toml",
];
const SOURCE_DIRS: [&str; 3] = [
    "site/crates/atelier-core/src",
    "site/crates/atelier-graph/src",
    "site/crates/atelier-web/src",
];
const NEVER_IN_A_MODULE: &str = "site/crates/atelier-core/src/model.rs";

fn walk(tree: &Path, dir: &str, found: &mut Vec<String>) -> Result<()> {
    let path = tree.join(dir);
    for entry in fs::read_dir(&path).with_context(|| format!("reading {}", path.display()))? {
        let entry = entry?;
        let rel = format!("{dir}/{}", entry.file_name().to_string_lossy());
        if entry.file_type()?.is_dir() {
            walk(tree, &rel, found)?;
        } else {
            found.push(rel);
        }
    }
    Ok(())
}

/// The hash of the sources under `tree`, by name and content. Line endings are normalised
/// because Git for Windows checks text out with CRLF by default, and the hash must agree on
/// every checkout.
fn sources_hash(tree: &Path) -> Result<String> {
    let mut files: Vec<String> = SOURCE_FILES.map(String::from).to_vec();
    for dir in SOURCE_DIRS {
        walk(tree, dir, &mut files)?;
    }
    files.retain(|rel| rel != NEVER_IN_A_MODULE);
    files.sort();
    let mut hash = Sha256::new();
    for rel in &files {
        let path = tree.join(rel);
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        hash.update(rel.as_bytes());
        hash.update([0]);
        hash.update(String::from_utf8_lossy(&bytes).replace("\r\n", "\n").as_bytes());
        hash.update([0]);
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The value after `key` on the stamp's line for it.
fn field<'a>(stamp: &'a str, key: &str) -> Option<&'a str> {
    stamp
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(' '))
}

/// `channel = "1.99.1"` from the toolchain pin.
fn pinned_rustc(tree: &Path) -> Result<String> {
    let path = tree.join(TOOLCHAIN);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    text.lines()
        .find_map(|line| {
            let value = line.trim().strip_prefix("channel")?.trim().strip_prefix('=')?;
            Some(value.trim().trim_matches('"').to_string())
        })
        .ok_or_else(|| anyhow!("{TOOLCHAIN} has no channel"))
}

/// The wasm-bindgen crate's locked version: the CLI refuses a module built against another.
pub fn pinned_wasm_bindgen(tree: &Path) -> Result<String> {
    let path = tree.join(LOCKFILE);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?
        .replace("\r\n", "\n");
    let (_, rest) = text
        .split_once("name = \"wasm-bindgen\"\nversion = \"")
        .ok_or_else(|| anyhow!("{LOCKFILE} locks no wasm-bindgen"))?;
    rest.split_once('"')
        .map(|(version, _)| version.to_string())
        .ok_or_else(|| anyhow!("{LOCKFILE} has an unfinished wasm-bindgen version"))
}

/// What is wrong with the stamp under `tree`; empty when the modules are current.
fn problems_at(tree: &Path) -> Result<Vec<String>> {
    let Ok(stamp) = fs::read_to_string(tree.join(STAMP)) else {
        return Ok(vec![format!("{STAMP} is missing: {REBUILD}")]);
    };
    let mut problems = Vec::new();
    if field(&stamp, "sources") != Some(sources_hash(tree)?.as_str()) {
        problems.push(format!(
            "the WASM modules in docs/assets were built from other sources than site/crates \
             holds: {REBUILD}"
        ));
    }
    let tools = [
        ("rustc", pinned_rustc(tree)?, TOOLCHAIN),
        ("wasm-bindgen", pinned_wasm_bindgen(tree)?, LOCKFILE),
    ];
    for (tool, wanted, source) in tools {
        let built = field(&stamp, tool).unwrap_or("nothing named");
        if built != wanted {
            problems.push(format!(
                "the WASM modules were built with {tool} {built} and {source} pins {wanted}: \
                 {REBUILD}"
            ));
        }
    }
    Ok(problems)
}

/// The gate.
pub fn fresh(repo: &Repo) -> Result<Vec<String>> {
    problems_at(&repo.root)
}

/// The second word of a tool's `--version` line, or the whole line if it has none.
fn version_of(line: &str) -> &str {
    let mut words = line.split_whitespace();
    let first = words.next().unwrap_or_default();
    words.next().unwrap_or(first)
}

fn write(root: &Path, args: &[String]) -> Result<()> {
    let value = |name: &str| -> Result<&str> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
            .ok_or_else(|| anyhow!("wasm-stamp write needs {name}"))
    };
    let stamp = format!(
        "sources {}\nrustc {}\nwasm-bindgen {}\nwasm-opt {}\n",
        sources_hash(root)?,
        version_of(value("--rustc")?),
        version_of(value("--wasm-bindgen")?),
        value("--wasm-opt")?,
    );
    let path = root.join(STAMP);
    fs::write(&path, stamp).with_context(|| format!("writing {}", path.display()))
}

/// The problems at commit `sha`, from that commit's files, or `None` if it has no stamp.
fn problems_at_commit(root: &Path, sha: &str) -> Result<Option<Vec<String>>> {
    if git(root, &["cat-file", "-e", &format!("{sha}:{STAMP}")]).is_err() {
        return Ok(None);
    }
    let scratch = Scratch::new()?;
    let tar = scratch.0.join("commit.tar");
    git(
        root,
        &[
            "archive",
            "--format=tar",
            &format!("--output={}", tar.display()),
            sha,
            "--",
            STAMP,
            TOOLCHAIN,
            "site",
        ],
    )?;
    let extract = Command::new("tar")
        .arg("-xf")
        .arg(&tar)
        .arg("-C")
        .arg(&scratch.0)
        .output()
        .context("running tar to unpack the commit")?;
    if !extract.status.success() {
        bail!(
            "tar failed: {}",
            String::from_utf8_lossy(&extract.stderr).trim()
        );
    }
    problems_at(&scratch.0).map(Some)
}

fn check(root: &Path, rev: Option<&str>) -> Result<ExitCode> {
    let (label, problems) = match rev {
        None => ("this tree".to_string(), problems_at(root)?),
        Some(rev) => {
            let sha = resolve(root, rev)?;
            let label = sha[..10].to_string();
            match problems_at_commit(root, &sha)? {
                Some(problems) => (label, problems),
                None => {
                    println!("wasm-stamp: {label}: no stamp at this commit, nothing to check");
                    return Ok(ExitCode::SUCCESS);
                }
            }
        }
    };
    if problems.is_empty() {
        println!("wasm-stamp: {label}: the WASM modules match their sources");
        return Ok(ExitCode::SUCCESS);
    }
    for problem in &problems {
        eprintln!("wasm-stamp: {label}: {problem}");
    }
    Ok(ExitCode::from(1))
}

/// `atelier-build wasm-stamp write --rustc V --wasm-bindgen V --wasm-opt V [--root DIR]` and
/// `atelier-build wasm-stamp check [REV] [--root DIR]`. Exit 1 and every problem named; 2 when
/// the check could not run at all.
pub fn run_wasm_stamp(args: Vec<String>) -> ExitCode {
    let root = root_from(&args);
    let mut words = args.iter().map(String::as_str);
    let outcome = root.and_then(|root| match words.next() {
        Some("write") => write(&root, &args).map(|()| ExitCode::SUCCESS),
        Some("check") => {
            let rev = words.next().filter(|word| !word.starts_with("--"));
            check(&root, rev)
        }
        _ => Err(anyhow!("usage: wasm-stamp <write|check> ...")),
    });
    outcome.unwrap_or_else(|error| {
        eprintln!("wasm-stamp: could not check: {error:#}");
        ExitCode::from(2)
    })
}
