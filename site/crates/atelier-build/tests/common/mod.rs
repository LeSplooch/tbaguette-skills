//! Shared by the check_*.rs suites: the real repository, and a throwaway copy
//! of it to break exactly one thing in.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use atelier_build::check;

/// The worktree this crate lives in (site/crates/atelier-build -> repo root).
pub fn real_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

/// How many skills are on disk, the number every count in this library answers to. A test
/// that needs one says this rather than a literal, so a new skill reddens only the counts
/// that are meant to be bumped by hand: `EXPECTED_SKILL_COUNT`, the manifests, the README and
/// CATALOG.md, each held by its own gate.
pub fn skill_count() -> usize {
    check::Repo::new(real_root())
        .skill_dirs()
        .expect("skills/ is readable")
        .len()
}

/// The gate holds on the real repository.
pub fn passes(id: &str) {
    let problems = check::run_gate(&real_root(), id);
    assert!(
        problems.is_empty(),
        "gate {id} should hold on the real repo but reports:\n  - {}",
        problems.join("\n  - ")
    );
}

/// A copy of every tracked file in its own git repository, committed, so
/// `git ls-files` and `git archive` see what the real repo shows.
pub struct Copy {
    dir: tempfile::TempDir,
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

impl Copy {
    pub fn new() -> Copy {
        let root = real_root();
        let listing = Command::new("git")
            .args([
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
            ])
            .current_dir(&root)
            .output()
            .expect("git ls-files");
        let dir = tempfile::tempdir().expect("temp dir");
        for rel in String::from_utf8(listing.stdout)
            .unwrap()
            .split('\0')
            .filter(|p| !p.is_empty())
        {
            // The built WASM modules are the heaviest files here and no test
            // that breaks one other thing reads them. The tests of the gates
            // that name them write their own.
            if rel.starts_with("docs/assets/") && rel.ends_with(".wasm") {
                continue;
            }
            let from = root.join(rel);
            let to = dir.path().join(rel);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            let meta = match fs::symlink_metadata(&from) {
                Ok(meta) => meta,
                Err(_) => continue, // tracked but deleted in the working tree
            };
            if meta.file_type().is_symlink() {
                std::os::unix::fs::symlink(fs::read_link(&from).unwrap(), &to).unwrap();
            } else {
                fs::copy(&from, &to).unwrap();
            }
        }
        git(dir.path(), &["init", "-q"]);
        let copy = Copy { dir };
        copy.commit();
        copy
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn commit(&self) {
        git(self.root(), &["add", "-A"]);
        git(
            self.root(),
            &["commit", "-q", "--allow-empty", "-m", "copy"],
        );
    }

    pub fn stage(&self) {
        git(self.root(), &["add", "-A"]);
    }

    pub fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
    }

    pub fn write(&self, rel: &str, text: &str) {
        let path = self.root().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Replace `from` with `to` in the file, which must contain `from`.
    pub fn replace(&self, rel: &str, from: &str, to: &str) {
        let text = self.read(rel);
        assert!(text.contains(from), "{rel} has no {from:?} to break");
        self.write(rel, &text.replacen(from, to, 1));
    }

    /// Replace every `from` with `to` in the file, which must contain `from`.
    pub fn replace_all(&self, rel: &str, from: &str, to: &str) {
        let text = self.read(rel);
        assert!(text.contains(from), "{rel} has no {from:?} to break");
        self.write(rel, &text.replace(from, to));
    }

    /// Parse, change and rewrite a JSON file.
    pub fn edit_json(&self, rel: &str, change: impl FnOnce(&mut serde_json::Value)) {
        let mut value: serde_json::Value = serde_json::from_str(&self.read(rel)).unwrap();
        change(&mut value);
        self.write(rel, &serde_json::to_string_pretty(&value).unwrap());
    }

    /// Write raw bytes (a binary, a large file).
    pub fn write_bytes(&self, rel: &str, bytes: &[u8]) {
        let path = self.root().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    pub fn symlink(&self, rel: &str, target: &str) {
        std::os::unix::fs::symlink(target, self.root().join(rel)).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        let path = self.root().join(rel);
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
    }

    pub fn problems(&self, id: &str) -> Vec<String> {
        self.stage();
        check::run_gate(self.root(), id)
    }
}

/// Break one thing in a copy and assert the gate reports a problem mentioning
/// `expect` (and, as a guard against the wrong reason, nothing is asserted
/// about other gates).
pub fn breaks(id: &str, expect: &str, break_it: impl FnOnce(&Copy)) {
    let copy = Copy::new();
    break_it(&copy);
    let problems = copy.problems(id);
    assert!(
        problems.iter().any(|p| p.contains(expect)),
        "gate {id} should name {expect:?} after the breakage, but reports: {problems:#?}"
    );
}

/// The same with the name checked on a gate that must stay quiet before the
/// breakage (the copy itself is clean).
pub fn copy_is_clean(id: &str) {
    let copy = Copy::new();
    assert_eq!(
        copy.problems(id),
        Vec::<String>::new(),
        "the unbroken copy trips {id}"
    );
}
