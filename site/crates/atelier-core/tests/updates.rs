//! The UPDATES.md parser: the entry grammar, every way the file can be wrong
//! and the exact text of each refusal, and the size guard's message.

use std::fs;
use std::path::{Path, PathBuf};

use atelier_core::updates::{self, UpdateEntry};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn entry(date: &str, title: &str, bullets: &[&str]) -> UpdateEntry {
    UpdateEntry {
        date: date.to_owned(),
        title: title.to_owned(),
        bullets: bullets.iter().map(|b| (*b).to_owned()).collect(),
    }
}

fn error_of(input: &str) -> String {
    updates::parse(input)
        .expect_err("this input should be refused")
        .to_string()
}

#[test]
fn entries_come_back_in_file_order_and_the_preamble_is_dropped() {
    let got = updates::parse(
        "# Update notes\n\nHow to append.\n\n## 2026-08-24 — Newest thing\n\n- First.\n- Second.\n\n## 2026-08-20\n- Older.\n",
    )
    .unwrap();
    assert_eq!(
        got,
        [
            entry("2026-08-24", "Newest thing", &["First.", "Second."]),
            entry("2026-08-20", "", &["Older."]),
        ]
    );
}

#[test]
fn the_title_separator_may_be_a_hyphen_an_en_dash_or_an_em_dash() {
    for separator in ["-", "–", "—"] {
        let got =
            updates::parse(&format!("## 2026-08-24 {separator} Titled\n- A bullet.\n")).unwrap();
        assert_eq!(
            got,
            [entry("2026-08-24", "Titled", &["A bullet."])],
            "{separator}"
        );
    }
}

#[test]
fn wrapped_bullets_are_joined_with_one_space_and_markdown_is_kept_raw() {
    let got = updates::parse(
        "## 2026-08-24\n- A bullet long enough that it\n  wrapped onto a second line.\n- `code`, **bold**, <not-a-tag>.\n",
    )
    .unwrap();
    assert_eq!(
        got[0].bullets,
        [
            "A bullet long enough that it wrapped onto a second line.",
            "`code`, **bold**, <not-a-tag>."
        ]
    );
}

#[test]
fn same_day_entries_and_crlf_line_endings_are_accepted() {
    let got =
        updates::parse("## 2026-08-24 — T\r\n- A.\r\n  wrapped\r\n\r\n## 2026-08-24\r\n- B.\r\n")
            .unwrap();
    assert_eq!(got[0].bullets, ["A. wrapped"]);
    assert_eq!(got.len(), 2);
    assert!(updates::parse("").unwrap().is_empty());
}

#[test]
fn a_heading_that_does_not_start_with_an_iso_date_is_refused() {
    for (input, shown) in [
        ("## Unreleased\n- A bullet.\n", "## Unreleased"),
        ("## 2026-8-24\n- A bullet.\n", "## 2026-8-24"),
        ("## 2026-08-24x\n- A bullet.\n", "## 2026-08-24x"),
        (
            "## 2026-08-24—Titled\n- A bullet.\n",
            "## 2026-08-24—Titled",
        ),
    ] {
        assert_eq!(
            error_of(input),
            format!(
                "line 1: update-note headings start with an ISO date (## YYYY-MM-DD — Title), got \"{shown}\""
            ),
            "{input:?}"
        );
    }
}

#[test]
fn a_newer_entry_below_an_older_one_names_its_line() {
    let message = error_of("# T\n\nintro\n\n## 2026-08-20\n- A.\n\n## 2026-08-24\n- B.\n");
    assert!(
        message.starts_with("line 8: 2026-08-24 is newer than the 2026-08-20 entry above it"),
        "{message}"
    );
}

#[test]
fn an_entry_without_bullets_is_refused_in_the_middle_and_at_the_end() {
    let middle = error_of("## 2026-08-24 — Empty\n\n## 2026-08-20\n- A bullet.\n");
    assert!(
        middle.starts_with("line 3: the 2026-08-24 entry has no bullets"),
        "{middle}"
    );
    let trailing = error_of("## 2026-08-24 — Trailing and empty\n");
    assert!(
        trailing.starts_with("the 2026-08-24 entry has no bullets"),
        "{trailing}"
    );
}

#[test]
fn prose_before_the_first_bullet_is_refused() {
    let message = error_of("## 2026-08-24\n\nA paragraph nobody would ever see.\n\n- A bullet.\n");
    assert!(
        message.starts_with("line 3: prose between the 2026-08-24 heading and its first bullet"),
        "{message}"
    );
}

#[test]
fn the_repositorys_own_updates_file_parses() {
    let text = fs::read_to_string(repo_root().join("UPDATES.md")).unwrap();
    let entries = updates::parse(&text).unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn the_size_guard_names_the_command_that_fixes_it() {
    assert_eq!(updates::size_guard("UPDATES.md", updates::MAX_BYTES), None);
    let message = updates::size_guard("UPDATES.md", updates::MAX_BYTES + 1).expect("past the cap");
    assert!(
        message.starts_with("UPDATES.md: 128 KiB, past the 128 KiB"),
        "{message}"
    );
    assert!(
        message.contains("`site/atelier archive-updates`"),
        "{message}"
    );
    assert!(message.contains("newest 30"), "{message}");
}

#[test]
fn unicode_line_breaks_and_whitespace_follow_python_semantics() {
    // The grammar was defined by Python's str.splitlines (which breaks on
    // \x0b, \x0c, \x1c-\x1e, \x85 and U+2028/9 as well as \n and \r) and
    // str.strip (which trims \x1c-\x1f); the parser keeps those semantics so
    // existing notes parse the way they always did.
    let text = "## 2026-08-24\u{2028}- A\u{85}  wrapped\u{b}\u{1f}\n";
    let got = updates::parse(text).unwrap();
    assert_eq!(got[0].bullets, ["A wrapped"]);
    // An impossible-but-shaped date is accepted.
    assert_eq!(
        updates::parse("## 2026-13-45\n- A.\n").unwrap()[0].date,
        "2026-13-45"
    );
}
