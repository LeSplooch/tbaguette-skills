//! `crumb-check` (scripts/crumb_check.py, scripts/test_crumb_check.py): does
//! the Crumb a commit publishes describe the skills that commit publishes?

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use atelier_core::model::categories;
use common::{Copy, real_root, skill_count};

/// The Crumb this tree's skills build to, once for the whole file, into Cargo's scratch
/// directory. Tests never read it from docs/: this suite runs after a skill is added and
/// before the hook has rebuilt the site, and then docs/ describes the skills before it.
fn fresh_crumb() -> &'static Path {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("check-crumb-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        let built = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
            .args(["build", "--base-path", "/tbaguette-skills", "--root"])
            .arg(real_root())
            .arg("--out")
            .arg(&out)
            .output()
            .expect("atelier-build runs");
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        out
    })
}

/// A copy of this tree, committed, whose Crumb was built from the skills it holds.
fn copy_with_fresh_crumb() -> Copy {
    let copy = Copy::new();
    for rel in ["graph/graph.json", "graph/index.html"] {
        let built = std::fs::read_to_string(fresh_crumb().join(rel)).unwrap();
        copy.write(&format!("docs/{rel}"), &built);
    }
    copy.commit();
    copy
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn crumb(root: &Path, args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .arg("crumb-check")
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("atelier-build runs");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// `crumb-check HEAD` on a copy after one committed breakage: exit 1 naming it.
fn names_after_breakage(expect: &str, break_it: impl FnOnce(&Copy)) {
    let copy = copy_with_fresh_crumb();
    let clean = crumb(copy.root(), &["HEAD"]);
    assert_eq!(
        clean.code, 0,
        "the unbroken copy should pass first: {}{}",
        clean.stdout, clean.stderr
    );
    break_it(&copy);
    copy.commit();
    let run = crumb(copy.root(), &["HEAD"]);
    assert_eq!(
        run.code, 1,
        "stdout: {}\nstderr: {}",
        run.stdout, run.stderr
    );
    assert!(
        run.stderr.contains(expect),
        "wanted {expect:?} in:\n{}",
        run.stderr
    );
}

#[test]
fn a_crumb_built_from_its_commit_s_skills_matches_them() {
    let copy = copy_with_fresh_crumb();
    let run = crumb(copy.root(), &["HEAD"]);
    assert_eq!(
        run.code, 0,
        "stdout: {}\nstderr: {}",
        run.stdout, run.stderr
    );
    let summary = format!("the Crumb matches its skills -- {} skills,", skill_count());
    assert!(run.stdout.contains(&summary), "{}", run.stdout);
    let families = format!("{} families", categories().len());
    assert!(run.stdout.contains(&families), "{}", run.stdout);
}

#[test]
fn a_skill_edited_after_the_graph_was_built_is_named() {
    names_after_breakage("out of date in docs/graph/graph.json: naming-things", |c| {
        let text = c.read("skills/naming-things/SKILL.md");
        c.write(
            "skills/naming-things/SKILL.md",
            &format!("{text}\n## Added\n\nSeveral more words that nobody counted.\n"),
        );
    });
}

#[test]
fn a_skill_missing_from_the_graph_is_named() {
    names_after_breakage("missing from docs/graph/graph.json: ", |c| {
        c.edit_json("docs/graph/graph.json", |v| {
            // Drop the last skill and its place in the last family.
            v["skills"].as_array_mut().unwrap().pop();
            let last = v["categories"].as_array_mut().unwrap().last_mut().unwrap();
            last[2].as_array_mut().unwrap().pop();
        });
    });
}

#[test]
fn a_graph_built_for_another_base_path_is_named() {
    names_after_breakage("it was built for another base path", |c| {
        c.replace(
            "docs/graph/graph.json",
            "/tbaguette-skills/skills/{slug}/",
            "/elsewhere/skills/{slug}/",
        );
    });
}

#[test]
fn a_family_the_skills_no_longer_have_is_named() {
    names_after_breakage("lists other families than the skills do", |c| {
        c.edit_json("docs/graph/graph.json", |v| {
            v["categories"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!(["ghost", "Ghost family", []]));
        });
    });
}

#[test]
fn a_stale_summary_on_the_graph_page_is_named() {
    names_after_breakage(
        "docs/graph/index.html does not carry the summary a fresh build writes",
        |c| {
            let n = skill_count();
            c.replace_all(
                "docs/graph/index.html",
                &format!("How {n} skills"),
                &format!("How {} skills", n - 2),
            );
        },
    );
}

#[test]
fn a_family_without_a_palette_colour_is_named() {
    names_after_breakage("has no colour for family 12 of 12", |c| {
        c.replace_all(
            "docs/assets/styles.css",
            "--graph-cat-12:",
            "--graph-cat-99:",
        );
    });
}

#[test]
fn skills_reordered_within_a_family_are_named() {
    names_after_breakage(
        "files some skills under a different family than the skills do",
        |c| {
            c.edit_json("docs/graph/graph.json", |v| {
                let members = v["categories"][1][2].as_array_mut().unwrap();
                members.swap(0, 1);
            });
        },
    );
}

#[test]
fn a_missing_graph_page_is_named() {
    names_after_breakage("docs/graph/index.html is missing", |c| {
        c.remove("docs/graph/index.html");
    });
}

#[test]
fn a_palette_missing_from_only_one_theme_names_that_theme() {
    names_after_breakage("`", |c| {
        c.replace(
            "docs/assets/styles.css",
            "--graph-cat-12:",
            "--graph-cat-99:",
        );
    });
}

#[test]
fn a_commented_out_colour_does_not_count() {
    names_after_breakage("has no colour for family 12 of 12", |c| {
        c.replace_all(
            "docs/assets/styles.css",
            "--graph-cat-12:",
            "/*--graph-cat-12:*/ x:",
        );
    });
}

#[test]
fn no_palette_at_all_is_named() {
    names_after_breakage("defines no --graph-cat-N family colours at all", |c| {
        c.replace_all("docs/assets/styles.css", "--graph-cat-", "--graph-ink-");
    });
}

#[test]
fn the_clock_fields_are_not_compared() {
    let copy = copy_with_fresh_crumb();
    copy.edit_json("docs/graph/graph.json", |v| {
        for skill in v["skills"].as_array_mut().unwrap().iter_mut().take(5) {
            let skill = skill.as_array_mut().unwrap();
            while skill.len() < 9 {
                skill.push(match skill.len() {
                    6 => false.into(),
                    _ => serde_json::Value::Null,
                });
            }
            skill[7] = "updated".into();
            skill[8] = "2031-01-01T00:00:00+00:00".into();
        }
    });
    copy.commit();
    let run = crumb(copy.root(), &["HEAD"]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
}

#[test]
fn only_the_revision_is_read_never_the_working_tree() {
    let copy = copy_with_fresh_crumb();
    // Uncommitted edit that would fail the check if the working tree were read.
    let text = copy.read("skills/naming-things/SKILL.md");
    copy.write(
        "skills/naming-things/SKILL.md",
        &format!("{text}\nmore words, uncommitted\n"),
    );
    copy.write("docs/graph/graph.json", "not even json");
    let run = crumb(copy.root(), &["HEAD"]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
}

#[test]
fn a_commit_with_no_crumb_has_nothing_to_check() {
    let copy = copy_with_fresh_crumb();
    copy.remove("docs/graph/graph.json");
    copy.commit();
    let run = crumb(copy.root(), &["HEAD"]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains("no Crumb at this commit, nothing to check"),
        "{}",
        run.stdout
    );
}

#[test]
fn a_revision_that_is_not_a_commit_could_not_be_checked() {
    let run = crumb(&real_root(), &["no-such-revision"]);
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("'no-such-revision' is not a commit"),
        "{}",
        run.stderr
    );
}

/// Serve `body` as the response to the next request on a local port.
fn serve_once(body: String) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/graph/graph.json", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0u8; 2048];
        let n = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..n]).into_owned();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        request
    });
    (url, handle)
}

#[test]
fn live_passes_when_the_deployed_graph_is_the_committed_one() {
    let copy = copy_with_fresh_crumb();
    let (url, server) = serve_once(copy.read("docs/graph/graph.json"));
    let run = crumb(copy.root(), &["HEAD", "--live", "--live-url", &url]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("live site included"), "{}", run.stdout);
    let request = server.join().unwrap();
    assert!(
        request.starts_with("GET /graph/graph.json?rev="),
        "the revision is a cache key: {request}"
    );
}

#[test]
fn live_names_a_deploy_that_has_not_caught_up() {
    let copy = copy_with_fresh_crumb();
    let mut graph: serde_json::Value =
        serde_json::from_str(&copy.read("docs/graph/graph.json")).unwrap();
    graph["skills"].as_array_mut().unwrap().pop();
    graph["categories"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()[2]
        .as_array_mut()
        .unwrap()
        .pop();
    let (url, _server) = serve_once(graph.to_string());
    let run = crumb(copy.root(), &["HEAD", "--live", "--live-url", &url]);
    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stderr.contains("missing from the live graph.json"),
        "{}",
        run.stderr
    );
}

#[test]
fn live_that_cannot_be_read_could_not_be_checked() {
    let copy = copy_with_fresh_crumb();
    let run = crumb(
        copy.root(),
        &[
            "HEAD",
            "--live",
            "--live-url",
            "http://127.0.0.1:1/graph.json",
        ],
    );
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("could not read the live graph.json"),
        "{}",
        run.stderr
    );
}
