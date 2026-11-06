//! The published install command (`pages::index::install::INSTALL_COMMAND`,
//! verify/test_install_command.py shows readers the same check) can never alter, overwrite, or merge into
//! anything a user already has under `~/.claude/skills/` that this repo does
//! not own. A user asked "won't this alter other skills?"; this is the answer,
//! re-checked on every run rather than trusted from memory.
//!
//! Each scenario runs the literal published string, under every shell of
//! bash, zsh, fish and sh that is on PATH, in its own throwaway `HOME` so a
//! real `~/.claude` is never touched:
//!   A. fresh install                          - clones cleanly
//!   B. re-running once already installed      - updates in place, no error
//!   C. an empty directory already named TBaguette - clones into it
//!   D. a NON-empty, non-git directory named TBaguette (the real collision)
//!                                             - refuses, leaves it untouched
//! Sibling directories (another skill, another plugin) are populated in every
//! scenario and compared byte for byte before and after.
//!
//! This needs no network, and neither does the run of that Python test at the
//! end of this file: git's config rewrites the published GitHub URL to a local
//! repository, so the clone and pull still go through the exact string shown on
//! the site. A reader running the Python test by hand does clone from GitHub.

#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use atelier_build::pages::index::install::{
    INSTALL_COMMAND, INSTALL_COMMAND_CMD, INSTALL_COMMAND_POWERSHELL,
};

const REPO_URL: &str = "https://github.com/LeSplooch/tbaguette-skills.git";

/// bash and zsh cover the macOS/Linux default, sh is whatever /bin/sh is, and
/// fish is included though it is not POSIX: fish 3.0+ accepts && and ||
/// precisely so commands copy-pasted from bash docs still work, which is this
/// page's use case, and worth proving rather than assuming.
const CANDIDATE_SHELLS: [&str; 4] = ["bash", "zsh", "fish", "sh"];

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

fn shells() -> Vec<&'static str> {
    let found: Vec<&str> = CANDIDATE_SHELLS
        .into_iter()
        .filter(|s| {
            let present = on_path(s);
            if !present {
                eprintln!("(skipped {s}: not on PATH)");
            }
            present
        })
        .collect();
    assert!(
        !found.is_empty(),
        "no POSIX-ish shell on PATH: nothing to run the published command in"
    );
    found
}

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A local stand-in for the published repository, with the files the install
/// is expected to bring (README.md, CATALOG.md and skills/ are what the
/// install prompt itself verifies).
fn make_origin(parent: &Path) -> PathBuf {
    let origin = parent.join("origin");
    fs::create_dir_all(origin.join("skills/example")).unwrap();
    fs::write(origin.join("README.md"), "TBaguette stand-in\n").unwrap();
    fs::write(origin.join("CATALOG.md"), "catalog\n").unwrap();
    fs::write(origin.join("skills/example/SKILL.md"), "skill\n").unwrap();
    git(&origin, &["init", "--quiet", "--initial-branch=master"]);
    git(&origin, &["config", "user.name", "t"]);
    git(&origin, &["config", "user.email", "t@t"]);
    git(&origin, &["add", "-A"]);
    git(&origin, &["commit", "--quiet", "-m", "initial"]);
    origin
}

/// A fresh HOME whose gitconfig sends the published URL to `origin`.
fn make_home(parent: &Path, name: &str, origin: &Path) -> PathBuf {
    let home = parent.join(name);
    fs::create_dir_all(&home).unwrap();
    fs::write(
        home.join(".gitconfig"),
        format!("[url \"{}\"]\n\tinsteadOf = {REPO_URL}\n", origin.display()),
    )
    .unwrap();
    home
}

fn target(home: &Path) -> PathBuf {
    home.join(".claude/skills/TBaguette")
}

/// Fake pre-existing content the install must never touch: a loose skill
/// directory and a separate plugin, the two shapes `~/.claude/skills/` holds.
fn seed_sibling_skills(home: &Path) {
    let skills = home.join(".claude/skills");
    fs::create_dir_all(skills.join("some-other-skill")).unwrap();
    fs::write(
        skills.join("some-other-skill/SKILL.md"),
        "precious user content that must survive\n",
    )
    .unwrap();
    fs::create_dir_all(skills.join("another-plugin/.claude-plugin")).unwrap();
    fs::write(
        skills.join("another-plugin/.claude-plugin/plugin.json"),
        "{\"name\": \"another-plugin\"}\n",
    )
    .unwrap();
    fs::create_dir_all(skills.join("another-plugin/skills/foo")).unwrap();
    fs::write(
        skills.join("another-plugin/skills/foo/SKILL.md"),
        "more precious user content\n",
    )
    .unwrap();
}

/// (path relative to home, bytes) for every file under the two sibling
/// directories.
fn siblings(home: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, home: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, home, into);
            } else {
                into.insert(
                    path.strip_prefix(home).unwrap().display().to_string(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut map = BTreeMap::new();
    for name in ["some-other-skill", "another-plugin"] {
        walk(&home.join(".claude/skills").join(name), home, &mut map);
    }
    assert!(!map.is_empty(), "the seeded siblings exist");
    map
}

/// Run the literal published command string under `shell` with `home` as HOME.
fn install(shell: &str, home: &Path) -> Output {
    Command::new(shell)
        .args(["-c", INSTALL_COMMAND])
        .env("HOME", home)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_CONFIG_GLOBAL")
        .env_remove("XDG_CONFIG_HOME")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap_or_else(|e| panic!("{shell} runs: {e}"))
}

fn combined(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

// ---------------------------------------------------------------------------
// The command itself
// ---------------------------------------------------------------------------

/// The URL rewrite above only proves anything while the published command
/// clones from the URL it rewrites.
#[test]
fn published_commands_name_the_repo_and_the_one_install_path() {
    assert!(
        INSTALL_COMMAND.contains(REPO_URL),
        "the shell command clones from {REPO_URL}"
    );
    assert!(INSTALL_COMMAND_POWERSHELL.contains(REPO_URL));
    assert!(INSTALL_COMMAND_CMD.contains(REPO_URL));
    assert!(INSTALL_COMMAND.contains("~/.claude/skills/TBaguette"));
    // Windows variants: the same clone-or-pull, gated on the same `.git` test.
    for (label, command) in [
        ("powershell", INSTALL_COMMAND_POWERSHELL),
        ("cmd", INSTALL_COMMAND_CMD),
    ] {
        assert!(command.contains("pull"), "{label} pulls when installed");
        assert!(command.contains("git clone"), "{label} clones otherwise");
        assert!(
            command.contains(r"\.claude\skills\TBaguette\.git"),
            "{label} gates on the install's own .git"
        );
    }
}

#[test]
fn scenario_a_fresh_install() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = make_origin(tmp.path());
    for shell in shells() {
        let home = make_home(tmp.path(), &format!("a-{shell}"), &origin);
        seed_sibling_skills(&home);
        let before = siblings(&home);
        let out = install(shell, &home);
        assert!(
            out.status.success(),
            "{shell}: clone exits 0: {}",
            combined(&out)
        );
        assert!(
            target(&home).join("README.md").is_file(),
            "{shell}: TBaguette content actually present"
        );
        assert_eq!(
            before,
            siblings(&home),
            "{shell}: siblings byte-identical after install"
        );
    }
}

#[test]
fn scenario_b_rerun_when_already_installed_updates_without_error() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = make_origin(tmp.path());
    for shell in shells() {
        let home = make_home(tmp.path(), &format!("b-{shell}"), &origin);
        seed_sibling_skills(&home);
        assert!(
            install(shell, &home).status.success(),
            "{shell}: first install"
        );
        let before = siblings(&home);
        let out = install(shell, &home);
        let text = combined(&out);
        assert!(
            out.status.success(),
            "{shell}: re-run exits 0 (does not error): {text}"
        );
        assert!(
            ["Already up to date", "Updating", "Fast-forward"]
                .iter()
                .any(|marker| text.contains(marker)),
            "{shell}: re-run reports a pull, not a clone-refused error: {text}"
        );
        assert_eq!(
            before,
            siblings(&home),
            "{shell}: siblings still byte-identical after re-run"
        );
    }
}

/// ...and a re-run really does bring in what the published repo gained.
#[test]
fn scenario_b_rerun_picks_up_new_commits() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = make_origin(tmp.path());
    let home = make_home(tmp.path(), "b-new", &origin);
    assert!(install("sh", &home).status.success());
    fs::write(origin.join("NEW.md"), "new\n").unwrap();
    git(&origin, &["add", "-A"]);
    git(&origin, &["commit", "--quiet", "-m", "second"]);
    let out = install("sh", &home);
    assert!(out.status.success(), "{}", combined(&out));
    assert!(target(&home).join("NEW.md").is_file(), "the update arrived");
}

#[test]
fn scenario_c_empty_existing_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = make_origin(tmp.path());
    for shell in shells() {
        let home = make_home(tmp.path(), &format!("c-{shell}"), &origin);
        fs::create_dir_all(target(&home)).unwrap();
        seed_sibling_skills(&home);
        let before = siblings(&home);
        let out = install(shell, &home);
        assert!(
            out.status.success(),
            "{shell}: clone into empty existing dir exits 0: {}",
            combined(&out)
        );
        assert!(
            target(&home).join("README.md").is_file(),
            "{shell}: TBaguette content actually present"
        );
        assert_eq!(before, siblings(&home), "{shell}: siblings byte-identical");
    }
}

#[test]
fn scenario_d_non_empty_non_git_directory_is_refused_and_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = make_origin(tmp.path());
    for shell in shells() {
        let home = make_home(tmp.path(), &format!("d-{shell}"), &origin);
        fs::create_dir_all(target(&home)).unwrap();
        let marker = target(&home).join("dont-touch-me.txt");
        let marker_text = "unrelated content some other tool put here\n";
        fs::write(&marker, marker_text).unwrap();
        seed_sibling_skills(&home);
        let before = siblings(&home);
        let out = install(shell, &home);
        assert!(
            !out.status.success(),
            "{shell}: refuses (nonzero exit) rather than merging into unrelated content"
        );
        assert_eq!(
            fs::read_to_string(&marker).unwrap(),
            marker_text,
            "{shell}: the colliding directory's own content is untouched"
        );
        assert!(
            !target(&home).join("README.md").exists() && !target(&home).join(".git").exists(),
            "{shell}: nothing was merged into the colliding directory"
        );
        assert_eq!(
            before,
            siblings(&home),
            "{shell}: siblings byte-identical even in the refusal case"
        );
    }
}

/// /verify-install/ shows `verify/test_install_command.py` and tells readers to
/// run it; it carries its own copy of the published command, because a reader
/// running it has no site build. The two must never differ.
#[test]
fn the_python_test_readers_run_carries_the_published_command_word_for_word() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../verify/test_install_command.py");
    let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let start = source
        .find("INSTALL_COMMAND = (")
        .expect("the test defines INSTALL_COMMAND as a parenthesised literal");
    let literal = &source[start..];
    let literal = &literal[..literal.find("\n)\n").expect("the literal closes")];
    let joined: String = literal
        .lines()
        .skip(1)
        .map(|line| {
            let line = line.trim();
            line.strip_prefix('"')
                .and_then(|l| l.strip_suffix('"'))
                .unwrap_or_else(|| panic!("not a plain string line: {line}"))
        })
        .collect();
    assert_eq!(joined, INSTALL_COMMAND);
}

/// /verify-install/ prints that file and calls it "the same file the test suite actually
/// runs". This is the run. No python3 is a skip rather than a failure, because nothing else
/// in the suite needs one; the notice goes straight to stderr, since libtest captures
/// `eprintln!` from a test that passes and a skip nobody sees reads as a pass.
#[test]
fn the_python_test_the_page_shows_is_run_by_this_suite() {
    if !on_path("python3") {
        let _ = writeln!(
            std::io::stderr(),
            "SKIPPED: python3 is not on PATH, so verify/test_install_command.py did not run"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let origin = make_origin(dir.path());
    let config = dir.path().join("gitconfig");
    fs::write(
        &config,
        format!("[url \"{}\"]\n\tinsteadOf = {REPO_URL}\n", origin.display()),
    )
    .unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../verify/test_install_command.py");
    // GIT_CONFIG_GLOBAL and not a HOME: the script gives each scenario a HOME of its own.
    let out = Command::new("python3")
        .arg(&script)
        .current_dir(dir.path())
        .env("GIT_CONFIG_GLOBAL", &config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("python3 runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("checks passed."), "{stdout}");
}
