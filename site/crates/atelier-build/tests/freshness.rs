//! The New/Updated badges' input, read from a scratch git repository built
//! to exercise each rule of generate.py's `_fresh_skills`: the 48-hour window
//! on the committer date (commits out of date order included), "new" only
//! when the newest touching commit added the SKILL.md, the working tree
//! replacing commit history at HEAD, and an older revision ignoring the
//! working tree.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use atelier_build::repo::{HEAD, fresh_skills};
use atelier_core::graph::ChangeStamp;
use chrono::{DateTime, Utc};

const NOW: &str = "2026-10-29T14:11:17+00:00";

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(NOW)
        .unwrap()
        .with_timezone(&Utc)
}

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Repo {
        let dir =
            Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let repo = Repo(dir);
        repo.git(&["init", "--quiet", "--initial-branch=main"], None);
        repo
    }

    fn git(&self, args: &[&str], date: Option<&str>) -> String {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.0)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@example.com",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        if let Some(date) = date {
            command
                .env("GIT_COMMITTER_DATE", date)
                .env("GIT_AUTHOR_DATE", date);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Commits everything at `date`; returns the commit and its committer
    /// date as `%cI` prints it (git writes UTC as `Z`), which is the form
    /// the badge carries.
    fn commit(&self, message: &str, date: &str) -> (String, String) {
        self.git(&["add", "-A"], None);
        self.git(&["commit", "--quiet", "-m", message], Some(date));
        let shown = self.git(&["log", "-1", "--format=%H %cI"], None);
        let (rev, at) = shown.trim().split_once(' ').unwrap();
        (rev.to_string(), at.to_string())
    }
}

fn stamp(status: &str, at: &str) -> ChangeStamp {
    ChangeStamp {
        status: Some(status.to_string()),
        at: Some(at.to_string()),
    }
}

#[test]
fn badges_follow_the_window_the_newest_touch_and_the_working_tree() {
    let repo = Repo::new("freshness");
    // Out of the window: three days old.
    repo.write("skills/old/SKILL.md", "old");
    repo.write("skills/edited/SKILL.md", "edited");
    repo.commit("old", "2026-10-26T10:00:00+00:00");
    // Added inside the window: new.
    repo.write("skills/added/SKILL.md", "added");
    repo.write("skills/added/reference/a.md", "a");
    let (_, added_at) = repo.commit("added", "2026-10-28T09:00:00+02:00");
    // An older skill edited inside the window: updated.
    repo.write("skills/edited/SKILL.md", "edited again");
    // A commit dated before its parent, still inside the window: the walk
    // must not stop at it, and the one after it must still be seen.
    let (_, edited_at) = repo.commit("edited", "2026-10-28T08:00:00+00:00");
    repo.write("skills/late/SKILL.md", "late");
    repo.write("README", "not a skill");
    let (late, late_at) = repo.commit("late", "2026-10-29T12:00:00+00:00");

    let expected: BTreeMap<String, ChangeStamp> = [
        ("added", stamp("new", &added_at)),
        ("edited", stamp("updated", &edited_at)),
        ("late", stamp("new", &late_at)),
    ]
    .into_iter()
    .map(|(slug, s)| (slug.to_string(), s))
    .collect();
    assert_eq!(fresh_skills(&repo.0, now(), HEAD), expected);

    // Created inside the window, then edited later: the newest touch wins,
    // and it was not the one that added the SKILL.md.
    repo.write("skills/added/reference/a.md", "a, revised");
    let (revised, revised_at) = repo.commit("revise added", "2026-10-29T13:00:00+00:00");
    let at_head = fresh_skills(&repo.0, now(), HEAD);
    assert_eq!(at_head["added"], stamp("updated", &revised_at));

    // The working tree replaces history at HEAD: untracked is new, a
    // modification is updated, both stamped with the build instant.
    repo.write("skills/fresh/SKILL.md", "untracked");
    repo.write("skills/old/SKILL.md", "old, modified");
    let with_worktree = fresh_skills(&repo.0, now(), HEAD);
    assert_eq!(with_worktree["fresh"], stamp("new", NOW));
    assert_eq!(with_worktree["old"], stamp("updated", NOW));
    assert_eq!(with_worktree["late"], stamp("new", &late_at));

    // An older revision reads only its own history: no working tree, and
    // nothing committed after it.
    let at_late = fresh_skills(&repo.0, now(), &late);
    assert_eq!(at_late, expected);
    assert_ne!(late, revised);

    // git failing (here, on a revision it cannot resolve) means no badges,
    // and no error: the build fails open, as the Python one does.
    assert!(fresh_skills(&repo.0, now(), "no-such-rev").is_empty());

    fs::remove_dir_all(&repo.0).unwrap();
}
