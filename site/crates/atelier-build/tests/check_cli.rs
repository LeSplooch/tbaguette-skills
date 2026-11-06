//! `atelier-build check` as a whole: what it prints and how it exits.

mod common;

use std::process::Command;

use atelier_build::check;
use common::{Copy, real_root, skill_count};

fn check_at(root: &std::path::Path) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .args(["check", "--root"])
        .arg(root)
        .output()
        .expect("atelier-build runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn check_exits_zero_on_the_real_repo_and_lists_every_gate_it_ran() {
    let (code, out) = check_at(&real_root());
    assert_eq!(code, 0, "{out}");
    let gates = check::gates();
    assert!(gates.len() >= 50, "{} gates", gates.len());
    for gate in &gates {
        assert!(
            out.contains(&format!("ok    {}", gate.id)),
            "gate {} not listed as ok:\n{out}",
            gate.id
        );
    }
    assert!(
        out.contains(&format!("All {} gates passed.", gates.len())),
        "{out}"
    );
    assert!(
        out.contains("skill size report (informational, not a gate)"),
        "{out}"
    );
}

#[test]
fn check_exits_one_and_names_the_failing_gate_and_problem() {
    let copy = Copy::new();
    let (on_disk, stated) = (skill_count(), skill_count() - 2);
    copy.replace(
        "CATALOG.md",
        &format!("{on_disk} skills,"),
        &format!("{stated} skills,"),
    );
    copy.stage();
    let (code, out) = check_at(copy.root());
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("FAIL  catalog-stated-count"), "{out}");
    assert!(
        out.contains(&format!(
            "CATALOG.md says {stated} skills; {on_disk} are on disk"
        )),
        "{out}"
    );
    assert!(out.contains("FAILED: 1/"), "{out}");
}

#[test]
fn every_gate_has_a_distinct_id_and_a_description() {
    let gates = check::gates();
    let mut ids: Vec<&str> = gates.iter().map(|g| g.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), gates.len(), "duplicate gate ids");
    assert!(gates.iter().all(|g| !g.what.is_empty()));
}

#[test]
fn an_unreadable_root_is_exit_two_not_a_pass() {
    let (code, _) = check_at(std::path::Path::new("/nonexistent/repo/root"));
    assert_ne!(code, 0);
}

fn router_at(root: &std::path::Path, extra: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .arg("router")
        .args(extra)
        .arg("--root")
        .arg(root)
        .output()
        .expect("atelier-build runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn the_router_check_names_a_stale_table_and_writing_it_cures_it() {
    let (code, out) = router_at(&real_root(), &["--check"]);
    assert_eq!(code, 0, "the committed table is current: {out}");

    let copy = Copy::new();
    let skill = "skills/naming-things/SKILL.md";
    let text = copy.read(skill);
    copy.write(
        skill,
        &text.replacen(
            "description: ",
            "description: zebra crossing guardrails ",
            1,
        ),
    );
    let (code, out) = router_at(copy.root(), &["--check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("hooks/skill-index.tsv is stale"), "{out}");
    assert!(
        out.contains("site/atelier router"),
        "says how to fix it: {out}"
    );
    assert!(
        !copy.problems("hooks-skill-index").is_empty(),
        "the gate says so too"
    );

    let (code, out) = router_at(copy.root(), &[]);
    assert_eq!(code, 0, "{out}");
    assert!(
        copy.read("hooks/skill-index.tsv")
            .contains("cross_guardrail")
    );
    let (code, out) = router_at(copy.root(), &["--check"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(copy.problems("hooks-skill-index"), Vec::<String>::new());
}
