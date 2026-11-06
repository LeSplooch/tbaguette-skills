//! Move all but the newest update notes from UPDATES.md into one file per
//! month (scripts/archive_updates.py).
//!
//! UPDATES.md ships inside the plugin folder, and Anthropic's plugin directory
//! holds any text file over 256 KiB for a reviewer. At one to four entries a
//! day the file would pass that within weeks, so the newest `KEEP` entries stay
//! in UPDATES.md (more than the landing page shows) and everything older goes
//! to `updates-archive/<YYYY-MM>.md`, newest first, exactly as written. Nothing
//! is reworded, merged or dropped: the entries across all the files, in order,
//! are the same text UPDATES.md held.
//!
//! Safe to run at any time; it does nothing when there is nothing to move. The
//! build refuses to run once UPDATES.md passes `MAX_BYTES`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use atelier_core::updates::{ARCHIVE_DIR, KEEP};
use regex::Regex;

use super::root_from;

/// (preamble, [entry, ...]): each entry is its heading through the line before
/// the next heading, as written. Python split on the zero-width
/// `(?m)^(?=## \d{4}-\d{2}-\d{2})`; the heading's own position is the cut.
pub fn split_entries(text: &str) -> (&str, Vec<&str>) {
    let heading = Regex::new(r"(?m)^## [0-9]{4}-[0-9]{2}-[0-9]{2}").expect("static regex");
    let starts: Vec<usize> = heading.find_iter(text).map(|m| m.start()).collect();
    let Some(&first) = starts.first() else {
        return (text, Vec::new());
    };
    let entries = starts
        .iter()
        .zip(
            starts
                .iter()
                .skip(1)
                .copied()
                .chain(std::iter::once(text.len())),
        )
        .map(|(&start, end)| &text[start..end])
        .collect();
    (&text[..first], entries)
}

fn join(preamble: &str, entries: &[&str]) -> String {
    let body: String = entries
        .iter()
        .map(|entry| format!("{}\n\n", entry.trim_end_matches('\n')))
        .collect();
    format!("{}\n\n{body}", preamble.trim_end_matches('\n'))
        .trim_end_matches('\n')
        .to_string()
        + "\n"
}

fn month_preamble(month: &str) -> String {
    format!(
        "# Update notes, {month}\n\n\
         Entries from this month that have moved out of UPDATES.md, which keeps\n\
         the newest {KEEP}. Newest first, exactly as they were written there, in\n\
         the same shape; `site/atelier archive-updates` moves them.\n"
    )
}

/// Move every entry past the newest `KEEP`. Returns the files written, with
/// UPDATES.md first; empty when there was nothing to move.
pub fn archive(root: &Path) -> Result<Vec<PathBuf>> {
    let updates_path = root.join("UPDATES.md");
    let text = std::fs::read_to_string(&updates_path)
        .with_context(|| format!("reading {}", updates_path.display()))?;
    let (preamble, entries) = split_entries(&text);
    if entries.len() <= KEEP {
        return Ok(Vec::new());
    }
    let (kept, moved) = entries.split_at(KEEP);

    // Months in the order they first appear, newest first.
    let mut by_month: Vec<(&str, Vec<&str>)> = Vec::new();
    for entry in moved {
        let month = &entry[3..10];
        match by_month.iter_mut().find(|(m, _)| *m == month) {
            Some((_, list)) => list.push(entry),
            None => by_month.push((month, vec![entry])),
        }
    }

    let mut written = Vec::new();
    for (month, month_entries) in &by_month {
        let path = root.join(ARCHIVE_DIR).join(format!("{month}.md"));
        let existing = std::fs::read_to_string(&path).ok();
        let (head, archived) = match &existing {
            Some(text) => split_entries(text),
            None => ("", Vec::new()),
        };
        let head = if existing.is_some() {
            head.to_string()
        } else {
            month_preamble(month)
        };
        // Whatever moves now is newer than whatever moved before it.
        let all: Vec<&str> = month_entries
            .iter()
            .chain(archived.iter())
            .copied()
            .collect();
        std::fs::create_dir_all(path.parent().expect("a parent"))?;
        std::fs::write(&path, join(&head, &all))
            .with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }

    std::fs::write(&updates_path, join(preamble, kept))?;
    written.insert(0, updates_path);
    Ok(written)
}

/// `atelier-build archive-updates [--root DIR]`.
pub fn run_archive_updates(args: Vec<String>) -> ExitCode {
    let result = root_from(&args).and_then(|root| Ok((archive(&root)?, root)));
    match result {
        Ok((written, _)) if written.is_empty() => {
            println!("archive-updates: UPDATES.md holds {KEEP} entries or fewer, nothing to move");
            ExitCode::SUCCESS
        }
        Ok((written, root)) => {
            for path in written {
                println!(
                    "archive-updates: wrote {}",
                    path.strip_prefix(&root).unwrap_or(&path).display()
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atelier-build archive-updates: {error:#}");
            ExitCode::from(2)
        }
    }
}
