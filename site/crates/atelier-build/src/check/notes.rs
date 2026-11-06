//! UPDATES.md and updates-archive/ (the repository half of
//! scripts/test_archive_updates.py): the record stays whole, parses, runs
//! newest first across every file, and every file stays under the plugin
//! directory's 256 KiB.

use std::path::PathBuf;

use anyhow::{Context, Result};
use atelier_core::updates::{self, ARCHIVE_DIR, KEEP};

use super::Repo;
use super::archive::split_entries;

const PLUGIN_DIRECTORY_LIMIT: u64 = 256 * 1024;

/// How many notes the landing page shows (templates.UPDATE_NOTES_LIMIT). The
/// file must keep more than the page shows, or the archive would eat notes the
/// page still displays. Checked when this compiles; nothing in a repo can
/// change it at run time.
const LANDING_PAGE_NOTES: usize = 20;
const _: () = assert!(KEEP >= LANDING_PAGE_NOTES);

pub fn updates_valid(repo: &Repo) -> Result<Vec<String>> {
    Ok(match updates::parse(&repo.read("UPDATES.md")?) {
        Ok(_) => Vec::new(),
        Err(error) => vec![format!("UPDATES.md: {error}")],
    })
}

pub fn updates_size(repo: &Repo) -> Result<Vec<String>> {
    let size = std::fs::metadata(repo.path("UPDATES.md"))
        .context("reading the size of UPDATES.md")?
        .len();
    Ok(updates::size_guard("UPDATES.md", size)
        .into_iter()
        .collect())
}

/// The archive files, oldest month first.
fn archives(repo: &Repo) -> Result<Vec<PathBuf>> {
    let dir = repo.path(ARCHIVE_DIR);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();
    Ok(files)
}

fn label(file: &std::path::Path) -> String {
    format!(
        "{ARCHIVE_DIR}/{}",
        file.file_name().unwrap_or_default().to_string_lossy()
    )
}

pub fn archives_hold_their_month(repo: &Repo) -> Result<Vec<String>> {
    let files = archives(repo)?;
    if files.is_empty() {
        return Ok(vec![format!("{ARCHIVE_DIR}/ holds no archive")]);
    }
    let mut problems = Vec::new();
    for file in files {
        let name = label(&file);
        let size = std::fs::metadata(&file)?.len();
        if size >= PLUGIN_DIRECTORY_LIMIT {
            problems.push(format!(
                "{name}: {size} bytes, past the 256 KiB the plugin directory allows"
            ));
        }
        let notes = match updates::parse(&std::fs::read_to_string(&file)?) {
            Ok(notes) => notes,
            Err(error) => {
                problems.push(format!("{name}: {error}"));
                continue;
            }
        };
        let month = file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if notes.is_empty() {
            problems.push(format!("{name}: holds no notes"));
        }
        for note in notes.iter().filter(|n| !n.date.starts_with(&month)) {
            problems.push(format!(
                "{name}: holds a note dated {}, outside {month}",
                note.date
            ));
        }
    }
    Ok(problems)
}

/// Dates of every entry across UPDATES.md then the archives, newest file
/// first, with the file each came from.
pub fn order(repo: &Repo) -> Result<Vec<String>> {
    let mut sources = vec![("UPDATES.md".to_string(), repo.read("UPDATES.md")?)];
    let mut files = archives(repo)?;
    files.reverse();
    for file in files {
        sources.push((label(&file), std::fs::read_to_string(&file)?));
    }
    let mut problems = Vec::new();
    let mut previous: Option<(String, String)> = None;
    for (name, text) in &sources {
        for entry in split_entries(text).1 {
            let date: String = entry.chars().skip(3).take(10).collect();
            if let Some((prev_name, prev_date)) = &previous
                && date.as_str() > prev_date.as_str()
            {
                problems.push(format!(
                    "{name}: {date} comes after {prev_name}'s {prev_date}, but the record runs newest first"
                ));
            }
            previous = Some((name.clone(), date));
        }
    }
    Ok(problems)
}
