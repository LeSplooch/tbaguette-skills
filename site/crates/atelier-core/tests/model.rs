//! The model: category grouping against the skills on disk, and what loading
//! a malformed skill says.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use atelier_core::model::{categories, humanize_filename};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn categories_cover_every_skill_once() {
    let mut seen = BTreeSet::new();
    for category in categories() {
        for slug in category.skill_slugs {
            assert!(seen.insert(*slug), "{slug} listed twice");
        }
    }
    let skills = atelier_build::repo::load_skills(&repo_root()).unwrap();
    let on_disk: BTreeSet<&str> = skills.keys().map(String::as_str).collect();
    assert_eq!(seen, on_disk);
}

#[test]
fn load_skills_names_the_file_when_frontmatter_is_missing() {
    let root = std::env::temp_dir().join(format!("atelier-core-model-{}", std::process::id()));
    let dir = root.join("skills/broken");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), "---\nname: broken\n---\nbody").unwrap();
    let error = atelier_build::repo::load_skills(&root)
        .unwrap_err()
        .to_string();
    fs::remove_dir_all(&root).unwrap();
    assert!(error.contains("broken/SKILL.md"), "{error}");
    assert!(error.contains("`description`"), "{error}");
}

#[test]
fn humanize_filename_capitalises_words_and_upper_cases_acronyms() {
    assert_eq!(humanize_filename("web"), "Web");
    assert_eq!(humanize_filename("cli-output"), "CLI Output");
    assert_eq!(humanize_filename("review-checklist"), "Review Checklist");
}
