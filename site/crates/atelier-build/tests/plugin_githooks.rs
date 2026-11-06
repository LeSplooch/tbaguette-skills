//! The repo's git hooks and how they get wired (scripts/test_githooks.py and
//! scripts/githooks.py's `ensure_wired`).
//!
//! `core.hooksPath` is local to a clone and cannot be committed, so
//! `.githooks/` ships inert until something sets it; `site/build.sh` does, on
//! every build. The hooks are thin: pre-commit refuses stale WASM modules, runs
//! `site/atelier build` and stages `docs/`; pre-push runs `site/atelier
//! crumb-check <sha>` and `wasm-stamp check <sha>` for pushes to master only. The behavior tests run the real hook scripts in a
//! throwaway repository whose `site/atelier` is a recording stand-in.

#![cfg(unix)]

mod common;

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, MutexGuard};

use common::real_root;

const HOOKS: [&str; 2] = ["pre-commit", "pre-push"];
const ZEROS: &str = "0000000000000000000000000000000000000000";
const SHA: &str = "1111111111111111111111111111111111111111";

/// Tests here write executable stubs and then run them. Another test thread
/// forking while a stub is still open for writing leaves that child holding the
/// write handle, and exec-ing the stub then fails with "Text file busy". Taking
/// this first in every test keeps writes and spawns from overlapping.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn git_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs")
}

fn git_ok(cwd: &Path, args: &[&str]) -> String {
    let out = git_in(cwd, args);
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn hook_text(name: &str) -> String {
    fs::read_to_string(real_root().join(".githooks").join(name)).unwrap()
}

// ---------------------------------------------------------------------------
// The real hook files
// ---------------------------------------------------------------------------

#[test]
fn hooks_exist_and_are_executable() {
    let _serial = serial();
    for name in HOOKS {
        let hook = real_root().join(".githooks").join(name);
        assert!(hook.is_file(), "{name} exists");
        let mode = fs::metadata(&hook).unwrap().permissions().mode();
        assert!(mode & 0o100 != 0, "{name} is not executable");
    }
}

/// A hook that loses its +x in the index is inert on every fresh clone, and
/// the symptom is silence rather than an error.
#[test]
fn hooks_are_committed_with_their_executable_bit() {
    let _serial = serial();
    for name in HOOKS {
        let out = git_ok(
            &real_root(),
            &["ls-files", "-s", &format!(".githooks/{name}")],
        );
        assert!(
            out.starts_with("100755"),
            "{name}: expected mode 100755, got {:?}",
            out.split_whitespace().next()
        );
    }
}

#[test]
fn pre_commit_regenerates_the_site() {
    let _serial = serial();
    let body = hook_text("pre-commit");
    assert!(
        body.contains("site/atelier build --base-path /tbaguette-skills"),
        "pre-commit runs the site build"
    );
    assert!(body.contains("git add docs/"), "pre-commit stages docs/");
}

#[test]
fn pre_push_checks_the_crumb_on_master_only() {
    let _serial = serial();
    let body = hook_text("pre-push");
    assert!(
        body.contains("crumb-check"),
        "pre-push runs the Crumb check"
    );
    assert!(
        body.contains("\"refs/heads/master\""),
        "pre-push looks at master only"
    );
}

/// The wiring: git reads hooks from `.githooks` only once `core.hooksPath`
/// points there, which no committed file can do. `site/build.sh` sets it on
/// every build.
#[test]
fn build_sh_wires_hooks_path() {
    let _serial = serial();
    let build_sh = fs::read_to_string(real_root().join("site/build.sh")).unwrap();
    let line = build_sh
        .lines()
        .find(|l| l.contains("config core.hooksPath"))
        .expect("site/build.sh sets core.hooksPath");
    assert_eq!(
        line.trim(),
        "git -C .. config core.hooksPath .githooks",
        "build.sh points core.hooksPath at .githooks (relative to the repo root above site/)"
    );
    assert!(
        real_root().join(".githooks").is_dir(),
        "...and the directory it names exists"
    );

    // Run that very line from a throwaway repo's site/ directory, twice: it
    // wires an unwired clone and is idempotent.
    let tmp = tempfile::tempdir().unwrap();
    git_ok(tmp.path(), &["init", "--quiet"]);
    fs::create_dir(tmp.path().join("site")).unwrap();
    for _ in 0..2 {
        let status = Command::new("sh")
            .args(["-c", line])
            .current_dir(tmp.path().join("site"))
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            git_ok(
                tmp.path(),
                &["config", "--local", "--get", "core.hooksPath"]
            )
            .trim(),
            ".githooks"
        );
    }
}

// ---------------------------------------------------------------------------
// The hooks, run for real
// ---------------------------------------------------------------------------

/// A throwaway repository with the real hooks copied to `.githooks/` and a
/// `site/atelier` that records its arguments (and any stdin) in `atelier.log`
/// and exits with `checker_exit`. `None` leaves `site/atelier` out.
struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(checker_exit: Option<i32>) -> Sandbox {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git_ok(root, &["init", "--quiet", "--initial-branch=master"]);
        git_ok(root, &["config", "user.name", "t"]);
        git_ok(root, &["config", "user.email", "t@t"]);
        fs::create_dir(root.join(".githooks")).unwrap();
        for name in HOOKS {
            let dest = root.join(".githooks").join(name);
            fs::copy(real_root().join(".githooks").join(name), &dest).unwrap();
            fs::set_permissions(&dest, fs::Permissions::from_mode(0o755)).unwrap();
        }
        if let Some(code) = checker_exit {
            fs::create_dir(root.join("site")).unwrap();
            let atelier = root.join("site/atelier");
            fs::write(
                &atelier,
                format!(
                    "#!/bin/sh\nprintf 'args:%s\\n' \"$*\" >> \"$(dirname \"$0\")/../atelier.log\"\n\
                     if [ \"$1\" = build ]; then mkdir -p docs && echo built > docs/index.html; fi\n\
                     stdin_bytes=$(cat | wc -c)\n\
                     printf 'stdin:%s\\n' \"$stdin_bytes\" >> \"$(dirname \"$0\")/../atelier.log\"\n\
                     exit {code}\n"
                ),
            )
            .unwrap();
            fs::set_permissions(&atelier, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Sandbox { dir }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn log(&self) -> String {
        fs::read_to_string(self.root().join("atelier.log")).unwrap_or_default()
    }

    /// Run `.githooks/pre-push` with these ref lines on stdin.
    fn pre_push(&self, ref_lines: &str) -> Output {
        let mut child = Command::new(self.root().join(".githooks/pre-push"))
            .current_dir(self.root())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(ref_lines.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

fn push_line(local_ref: &str, local_sha: &str, remote_ref: &str) -> String {
    format!("{local_ref} {local_sha} {remote_ref} {ZEROS}\n")
}

#[test]
fn pre_push_runs_crumb_check_on_the_pushed_sha_for_master() {
    let _serial = serial();
    let sb = Sandbox::new(Some(0));
    let out = sb.pre_push(&push_line("refs/heads/master", SHA, "refs/heads/master"));
    assert!(
        out.status.success(),
        "a passing checker lets the push through"
    );
    assert!(
        sb.log().contains(&format!("args:crumb-check {SHA}")),
        "the checker is asked about the pushed commit: {}",
        sb.log()
    );
}

#[test]
fn pre_push_refuses_when_the_checker_fails() {
    let _serial = serial();
    let sb = Sandbox::new(Some(3));
    let out = sb.pre_push(&push_line("refs/heads/master", SHA, "refs/heads/master"));
    assert_eq!(
        out.status.code(),
        Some(3),
        "the checker's status is the hook's"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("pre-push: refused"), "{stderr}");
    assert!(
        stderr.contains("site/atelier build --base-path /tbaguette-skills"),
        "the refusal names the rebuild command: {stderr}"
    );
}

#[test]
fn pre_push_ignores_other_branches() {
    let _serial = serial();
    let sb = Sandbox::new(Some(3));
    let out = sb.pre_push(&push_line("refs/heads/feature", SHA, "refs/heads/feature"));
    assert!(
        out.status.success(),
        "only master is served, so only master is checked"
    );
    assert!(sb.log().is_empty(), "the checker never ran: {}", sb.log());
}

#[test]
fn pre_push_passes_a_deletion() {
    let _serial = serial();
    let sb = Sandbox::new(Some(3));
    let out = sb.pre_push(&push_line("(delete)", ZEROS, "refs/heads/master"));
    assert!(out.status.success(), "a deletion publishes nothing");
    assert!(sb.log().is_empty(), "the checker never ran: {}", sb.log());
}

#[test]
fn pre_push_passes_when_the_checker_is_absent() {
    let _serial = serial();
    let sb = Sandbox::new(None);
    let out = sb.pre_push(&push_line("refs/heads/master", SHA, "refs/heads/master"));
    assert!(
        out.status.success(),
        "a checkout from before the check existed has nothing to run it with"
    );
}

/// The ref lines are the hook's loop input; the checker reading them would
/// swallow every ref after the first.
#[test]
fn pre_push_checks_every_master_ref_and_never_feeds_the_checker_its_stdin() {
    let _serial = serial();
    let sb = Sandbox::new(Some(0));
    let second = "2222222222222222222222222222222222222222";
    let lines = format!(
        "{}{}{}",
        push_line("refs/heads/feature", SHA, "refs/heads/feature"),
        push_line("refs/heads/master", SHA, "refs/heads/master"),
        push_line("refs/heads/other", second, "refs/heads/master"),
    );
    let out = sb.pre_push(&lines);
    assert!(out.status.success());
    let log = sb.log();
    assert!(log.contains(&format!("args:crumb-check {SHA}")), "{log}");
    assert!(log.contains(&format!("args:crumb-check {second}")), "{log}");
    assert_eq!(
        log.matches("stdin:").count(),
        log.matches("stdin:0").count(),
        "the checker was handed no stdin: {log}"
    );
}

#[test]
fn pre_push_passes_outside_a_work_tree() {
    let _serial = serial();
    let dir = tempfile::tempdir().unwrap();
    git_ok(dir.path(), &["init", "--quiet", "--bare"]);
    let mut child = Command::new(real_root().join(".githooks/pre-push"))
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(push_line("refs/heads/master", SHA, "refs/heads/master").as_bytes())
        .unwrap();
    assert!(
        child.wait().unwrap().success(),
        "a bare mirror has nothing to check with"
    );
}

/// End to end: with `core.hooksPath` pointing at `.githooks` (what build.sh
/// does), a real `git commit` runs the build and the commit carries `docs/`.
#[test]
fn a_wired_clone_builds_and_stages_docs_on_commit() {
    let _serial = serial();
    let sb = Sandbox::new(Some(0));
    git_ok(sb.root(), &["config", "core.hooksPath", ".githooks"]);
    fs::write(sb.root().join("README.md"), "hello\n").unwrap();
    git_ok(sb.root(), &["add", "README.md"]);
    git_ok(sb.root(), &["commit", "--quiet", "-m", "x"]);
    assert!(
        sb.log()
            .contains("args:build --base-path /tbaguette-skills"),
        "pre-commit ran the build: {}",
        sb.log()
    );
    let tracked = git_ok(sb.root(), &["ls-tree", "-r", "--name-only", "HEAD"]);
    assert!(
        tracked.lines().any(|p| p == "docs/index.html"),
        "the commit carries the regenerated docs/: {tracked}"
    );
}

#[test]
fn a_failing_build_stops_the_commit() {
    let _serial = serial();
    let sb = Sandbox::new(Some(1));
    git_ok(sb.root(), &["config", "core.hooksPath", ".githooks"]);
    fs::write(sb.root().join("README.md"), "hello\n").unwrap();
    git_ok(sb.root(), &["add", "README.md"]);
    let out = git_in(sb.root(), &["commit", "--quiet", "-m", "x"]);
    assert!(
        !out.status.success(),
        "a broken build must not be committed past"
    );
}

/// The real site/build.sh in a throwaway repository, with a `cargo` that fails and a
/// `wasm-bindgen` that is there, so it stops at its first build step. `preset` is what
/// `core.hooksPath` holds beforehand. Returns its exit code, stderr and the setting after.
fn build_sh_with_failing_cargo(preset: Option<&str>) -> (Option<i32>, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_ok(root, &["init", "--quiet", "--initial-branch=master"]);
    if let Some(path) = preset {
        git_ok(root, &["config", "core.hooksPath", path]);
    }
    fs::create_dir(root.join("site")).unwrap();
    fs::copy(real_root().join("site/build.sh"), root.join("site/build.sh")).unwrap();
    let stubs = root.join("stubs");
    fs::create_dir(&stubs).unwrap();
    for (name, body) in [("cargo", "exit 1"), ("wasm-bindgen", "exit 0")] {
        let stub = stubs.join(name);
        fs::write(&stub, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = format!("{}:{}", stubs.display(), std::env::var("PATH").unwrap());
    let out = Command::new("bash")
        .arg(root.join("site/build.sh"))
        .env("PATH", path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("bash runs");
    let after = git_in(root, &["config", "--local", "--get", "core.hooksPath"]);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        String::from_utf8_lossy(&after.stdout).trim().to_string(),
    )
}

#[test]
fn build_sh_wires_the_hooks_before_anything_can_fail() {
    let _serial = serial();
    let (code, stderr, hooks) = build_sh_with_failing_cargo(None);
    assert_ne!(code, Some(0), "the stand-in cargo stops it: {stderr}");
    assert_eq!(hooks, ".githooks", "{stderr}");
}

#[test]
fn build_sh_leaves_a_hooks_path_someone_chose() {
    let _serial = serial();
    let (code, stderr, hooks) = build_sh_with_failing_cargo(Some("my-hooks"));
    assert_ne!(code, Some(0), "the stand-in cargo stops it: {stderr}");
    assert_eq!(hooks, "my-hooks");
    assert!(stderr.contains("my-hooks"), "it says so: {stderr}");
}

#[test]
fn pre_commit_refuses_stale_modules_and_names_the_build_script() {
    let _serial = serial();
    let sb = Sandbox::new(Some(1));
    git_ok(sb.root(), &["config", "core.hooksPath", ".githooks"]);
    fs::write(sb.root().join("README.md"), "hello\n").unwrap();
    git_ok(sb.root(), &["add", "README.md"]);
    let out = git_in(sb.root(), &["commit", "--quiet", "-m", "x"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("site/build.sh"), "{stderr}");
    let log = sb.log();
    assert!(log.contains("args:wasm-stamp check"), "{log}");
    assert!(!log.contains("args:build"), "no build after a refusal: {log}");
}

#[test]
fn pre_push_checks_the_modules_of_the_pushed_commit() {
    let _serial = serial();
    let sb = Sandbox::new(Some(0));
    let out = sb.pre_push(&push_line("refs/heads/master", SHA, "refs/heads/master"));
    assert!(out.status.success());
    assert!(
        sb.log().contains(&format!("args:wasm-stamp check {SHA}")),
        "{}",
        sb.log()
    );
}

#[test]
fn pre_push_names_the_build_script_when_the_modules_are_stale() {
    let _serial = serial();
    let sb = Sandbox::new(Some(0));
    // Only the modules' check fails.
    let atelier = sb.root().join("site/atelier");
    fs::write(
        &atelier,
        "#!/bin/sh\ncase \"$1\" in wasm-stamp) exit 1;; esac\nexit 0\n",
    )
    .unwrap();
    let out = sb.pre_push(&push_line("refs/heads/master", SHA, "refs/heads/master"));
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("site/build.sh"), "{stderr}");
    assert!(!stderr.contains("Crumb"), "it was not the Crumb: {stderr}");
}
