//! UPDATES.md and updates-archive/ (scripts/test_archive_updates.py), and the
//! archive-updates tool itself.

mod common;

use std::path::Path;

use atelier_build::check::archive::{self, split_entries};
use atelier_core::updates::{ARCHIVE_DIR, KEEP};
use common::{Copy, breaks, passes};

// ---- the repository's own notes ----

#[test]
fn notes_updates_valid_holds() {
    passes("notes-updates-valid");
}

#[test]
fn notes_updates_valid_names_a_heading_without_a_date() {
    breaks("notes-updates-valid", "UPDATES.md: line", |c| {
        c.replace("UPDATES.md", "\n## 2026-", "\n## sometime — ");
    });
}

#[test]
fn notes_updates_valid_names_an_entry_appended_to_the_bottom() {
    breaks("notes-updates-valid", "is newer than the", |c| {
        let text = c.read("UPDATES.md");
        c.write(
            "UPDATES.md",
            &format!(
                "{}\n## 2031-01-01 — From the future\n\n- Last, not first.\n",
                text.trim_end()
            ),
        );
    });
}

#[test]
fn notes_updates_size_holds() {
    passes("notes-updates-size");
}

#[test]
fn notes_updates_size_names_the_fix() {
    breaks("notes-updates-size", "past the 128 KiB", |c| {
        let text = c.read("UPDATES.md");
        let filler = format!("- {}\n", "x".repeat(120));
        c.write(
            "UPDATES.md",
            &format!(
                "{}\n## 2000-01-01 — Big\n\n{}",
                text.trim_end(),
                filler.repeat(1200)
            ),
        );
    });
}

#[test]
fn notes_archives_hold() {
    passes("notes-archives");
}

#[test]
fn notes_archives_names_an_entry_from_another_month() {
    breaks("notes-archives", "updates-archive/2026-09.md", |c| {
        c.replace(
            "updates-archive/2026-09.md",
            "\n## 2026-09-",
            "\n## 2026-10-",
        );
    });
}

#[test]
fn notes_archives_names_an_archive_past_the_directory_limit() {
    breaks("notes-archives", "updates-archive/2026-07.md", |c| {
        let entry = format!("## 2026-07-01 — Big\n\n- {}\n\n", "y".repeat(2000));
        c.write(
            "updates-archive/2026-07.md",
            &format!("# Update notes, 2026-07\n\n{}", entry.repeat(140)),
        );
    });
}

#[test]
fn notes_order_holds() {
    passes("notes-order");
}

#[test]
fn notes_order_names_a_file_out_of_sequence() {
    breaks("notes-order", "runs newest first", |c| {
        // Still sorted inside UPDATES.md, but now older than September's archive.
        let text = c.read("UPDATES.md");
        let at = text.rfind("\n## 2026-").expect("an entry");
        let (head, tail) = text.split_at(at + 1);
        let rest = &tail["## 2026-09-09".len()..];
        c.write("UPDATES.md", &format!("{head}## 2026-01-01{rest}"));
    });
}

// ---- the archive-updates tool, against throwaway roots ----

const PREAMBLE: &str = "# Update notes\n\nPreamble.\n\n";

fn entry(date: &str, n: usize) -> String {
    format!(
        "## {date} \u{2014} Change {n}\n\n- What changed, number {n}.\n  Wrapped onto a second line.\n"
    )
}

fn updates(dates: &[&str]) -> String {
    let body: Vec<String> = dates.iter().enumerate().map(|(i, d)| entry(d, i)).collect();
    format!("{PREAMBLE}{}", body.join("\n"))
}

/// Every entry across UPDATES.md and the archives, newest first.
fn entries_in_order(root: &Path) -> Vec<String> {
    let mut files = vec![root.join("UPDATES.md")];
    let mut archives: Vec<_> = std::fs::read_dir(root.join(ARCHIVE_DIR))
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    archives.sort();
    archives.reverse();
    files.extend(archives);
    files
        .iter()
        .flat_map(|f| {
            let text = std::fs::read_to_string(f).unwrap();
            split_entries(&text)
                .1
                .iter()
                .map(|e| e.trim_end_matches('\n').to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn root_with(dates: &[&str]) -> (tempfile::TempDir, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let text = updates(dates);
    std::fs::write(dir.path().join("UPDATES.md"), &text).unwrap();
    let original = split_entries(&text)
        .1
        .iter()
        .map(|e| e.trim_end_matches('\n').to_string())
        .collect();
    (dir, original)
}

#[test]
fn nothing_moves_at_or_under_keep() {
    let (dir, _) = root_with(&vec!["2026-09-01"; KEEP]);
    assert!(archive::archive(dir.path()).unwrap().is_empty());
    assert!(!dir.path().join(ARCHIVE_DIR).exists());
}

#[test]
fn everything_past_keep_moves_word_for_word_into_its_month() {
    let mut dates = vec!["2026-10-02"; KEEP - 1];
    dates.extend(["2026-10-01", "2026-09-30", "2026-09-29", "2026-08-31"]);
    let (dir, original) = root_with(&dates);
    archive::archive(dir.path()).unwrap();
    assert_eq!(entries_in_order(dir.path()), original);
    let kept = std::fs::read_to_string(dir.path().join("UPDATES.md")).unwrap();
    let (preamble, kept_entries) = split_entries(&kept);
    assert_eq!(
        preamble.trim_end_matches('\n'),
        PREAMBLE.trim_end_matches('\n')
    );
    assert_eq!(kept_entries.len(), KEEP);
    for month in ["2026-09", "2026-08"] {
        let text =
            std::fs::read_to_string(dir.path().join(ARCHIVE_DIR).join(format!("{month}.md")))
                .unwrap();
        let notes = atelier_core::updates::parse(&text).unwrap();
        assert!(
            !notes.is_empty() && notes.iter().all(|n| n.date.starts_with(month)),
            "{month}"
        );
    }
    // October's entries are all among the newest KEEP, so October has no file.
    assert!(!dir.path().join(ARCHIVE_DIR).join("2026-10.md").exists());
}

#[test]
fn a_second_move_lands_above_the_first_in_the_same_month() {
    let (dir, first) = root_with(&vec!["2026-09-10"; KEEP + 2]);
    archive::archive(dir.path()).unwrap();
    let kept = std::fs::read_to_string(dir.path().join("UPDATES.md")).unwrap();
    let newer: String = (0..3)
        .map(|i| entry("2026-09-20", 100 + i) + "\n")
        .collect();
    std::fs::write(
        dir.path().join("UPDATES.md"),
        kept.replacen("## ", &format!("{newer}## "), 1),
    )
    .unwrap();
    let mut expected: Vec<String> = split_entries(&newer)
        .1
        .iter()
        .map(|e| e.trim_end_matches('\n').to_string())
        .collect();
    expected.extend(first);
    archive::archive(dir.path()).unwrap();
    assert_eq!(entries_in_order(dir.path()), expected);
}

#[test]
fn running_it_twice_changes_nothing() {
    let (dir, _) = root_with(&vec!["2026-09-01"; KEEP + 5]);
    archive::archive(dir.path()).unwrap();
    let snapshot = |root: &Path| -> Vec<(String, Vec<u8>)> {
        let mut files: Vec<_> = vec![root.join("UPDATES.md")];
        files.extend(
            std::fs::read_dir(root.join(ARCHIVE_DIR))
                .unwrap()
                .map(|e| e.unwrap().path()),
        );
        files.sort();
        files
            .into_iter()
            .map(|f| (f.display().to_string(), std::fs::read(&f).unwrap()))
            .collect()
    };
    let before = snapshot(dir.path());
    assert!(archive::archive(dir.path()).unwrap().is_empty());
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn the_command_moves_entries_in_a_repo_and_reports_what_it_wrote() {
    let copy = Copy::new();
    // The real UPDATES.md has not been archived yet, so entries are past KEEP.
    assert!(split_entries(&copy.read("UPDATES.md")).1.len() > KEEP);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .args(["archive-updates", "--root"])
        .arg(copy.root())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("archive-updates: wrote UPDATES.md"),
        "{stdout}"
    );
    let kept = split_entries(&copy.read("UPDATES.md")).1.len();
    assert_eq!(kept, KEEP);
    assert!(copy.problems("notes-updates-size").is_empty());
    assert!(
        copy.problems("notes-order").is_empty(),
        "{:?}",
        copy.problems("notes-order")
    );
}

#[test]
fn the_command_says_so_when_there_is_nothing_to_move() {
    let (dir, _) = root_with(&["2026-09-01"; 3]);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .args(["archive-updates", "--root"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("nothing to move"));
}
