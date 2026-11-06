//! The `wasm-fresh` gate and `wasm-stamp`: a change to the crates behind the WASM modules
//! cannot publish past the gates with the old modules still in `docs/assets`.

mod common;

use std::path::Path;
use std::process::Command;

use atelier_build::check::wasm_stamp::{STAMP, pinned_wasm_bindgen};
use common::{Copy, breaks, passes, real_root};

const WEB: &str = "site/crates/atelier-web/src/updated.rs";
const REGISTRY: &str = "site/crates/atelier-core/src/model.rs";

fn stamp(root: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .arg("wasm-stamp")
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("atelier-build runs");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn append(c: &Copy, rel: &str, text: &str) {
    let body = c.read(rel);
    c.write(rel, &format!("{body}{text}"));
}

#[test]
fn the_committed_modules_match_this_tree() {
    passes("wasm-fresh");
}

#[test]
fn a_change_to_a_crate_behind_the_modules_names_the_rebuild() {
    breaks("wasm-fresh", "run site/build.sh", |c| {
        append(c, WEB, "// a changed line\n");
    });
}

#[test]
fn a_new_source_file_counts() {
    breaks("wasm-fresh", "run site/build.sh", |c| {
        c.write("site/crates/atelier-graph/src/graph/added.rs", "// new\n");
    });
}

#[test]
fn a_changed_build_script_counts() {
    breaks("wasm-fresh", "run site/build.sh", |c| {
        append(c, "site/build.sh", "# a changed flag\n");
    });
}

#[test]
fn a_missing_stamp_is_named() {
    breaks("wasm-fresh", "wasm-build.stamp is missing", |c| {
        c.remove(STAMP);
    });
}

#[test]
fn a_moved_toolchain_pin_is_named() {
    breaks("wasm-fresh", "rust-toolchain.toml pins 0.0.1", |c| {
        let pin = c.read("rust-toolchain.toml");
        let line = pin.lines().find(|l| l.starts_with("channel")).unwrap();
        c.replace("rust-toolchain.toml", line, "channel = \"0.0.1\"");
    });
}

#[test]
fn a_moved_wasm_bindgen_pin_is_named() {
    let pinned = pinned_wasm_bindgen(&real_root()).unwrap();
    breaks("wasm-fresh", "site/Cargo.lock pins 0.0.1", |c| {
        c.replace(
            "site/Cargo.lock",
            &format!("name = \"wasm-bindgen\"\nversion = \"{pinned}\""),
            "name = \"wasm-bindgen\"\nversion = \"0.0.1\"",
        );
    });
}

/// Filing a skill edits the registry, which neither module reaches. It needs Rust and no
/// WASM toolchain, so it must not ask for a WASM build.
#[test]
fn filing_a_skill_in_the_registry_does_not_ask_for_a_rebuild() {
    let copy = Copy::new();
    copy.replace(REGISTRY, "\"naming-things\",", "\"naming-things\",\n\"a-new-skill\",");
    assert_eq!(copy.problems("wasm-fresh"), Vec::<String>::new());
}

#[test]
fn the_same_sources_hash_the_same_on_a_crlf_checkout() {
    let copy = Copy::new();
    let body = copy.read(WEB);
    copy.write(WEB, &body.replace('\n', "\r\n"));
    assert_eq!(copy.problems("wasm-fresh"), Vec::<String>::new());
}

#[test]
fn check_names_the_rebuild_and_exits_one_when_the_sources_moved() {
    let copy = Copy::new();
    let (code, out) = stamp(copy.root(), &["check"]);
    assert_eq!(code, 0, "{out}");
    append(&copy, WEB, "// a changed line\n");
    let (code, out) = stamp(copy.root(), &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("site/build.sh"), "{out}");
}

#[test]
fn check_of_a_commit_reads_that_commit_and_not_this_tree() {
    let copy = Copy::new();
    append(&copy, WEB, "// uncommitted\n");
    let (code, out) = stamp(copy.root(), &["check", "HEAD"]);
    assert_eq!(code, 0, "an uncommitted edit is not what the push publishes: {out}");
    copy.commit();
    let (code, out) = stamp(copy.root(), &["check", "HEAD"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("site/build.sh"), "{out}");
}

#[test]
fn a_commit_with_no_stamp_has_nothing_to_check() {
    let copy = Copy::new();
    copy.remove(STAMP);
    copy.commit();
    let (code, out) = stamp(copy.root(), &["check", "HEAD"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("nothing to check"), "{out}");
}

#[test]
fn a_revision_that_is_not_a_commit_could_not_be_checked() {
    let (code, out) = stamp(&real_root(), &["check", "no-such-revision"]);
    assert_eq!(code, 2, "{out}");
}

#[test]
fn writing_the_stamp_cures_the_sources_and_records_the_tools() {
    let copy = Copy::new();
    append(&copy, WEB, "// a changed line\n");
    let (code, out) = stamp(
        copy.root(),
        &[
            "write",
            "--rustc",
            "rustc 0.0.1 (abc 2026-01-01)",
            "--wasm-bindgen",
            "wasm-bindgen 0.0.2",
            "--wasm-opt",
            "none",
        ],
    );
    assert_eq!(code, 0, "{out}");
    let written = copy.read(STAMP);
    assert!(written.contains("\nrustc 0.0.1\n"), "{written}");
    assert!(written.contains("\nwasm-bindgen 0.0.2\n"), "{written}");
    assert!(written.contains("\nwasm-opt none\n"), "{written}");
    let problems = copy.problems("wasm-fresh");
    assert!(
        !problems.iter().any(|p| p.contains("other sources")),
        "the sources now match: {problems:#?}"
    );
    assert!(
        problems.iter().any(|p| p.contains("rustc 0.0.1")),
        "the tools do not: {problems:#?}"
    );
}

/// The README's install commands are a copy of a pin; a copy that drifts sends a contributor
/// to a CLI that refuses the module.
#[test]
fn the_readme_gives_the_install_commands_at_the_pinned_version() {
    let readme = std::fs::read_to_string(real_root().join("README.md")).unwrap();
    assert!(readme.contains("rustup target add wasm32-unknown-unknown"));
    let pinned = pinned_wasm_bindgen(&real_root()).unwrap();
    let install = format!("cargo install wasm-bindgen-cli --version {pinned} --locked");
    assert!(readme.contains(&install), "the README lacks `{install}`");
}
