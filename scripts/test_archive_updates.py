"""UPDATES.md and updates-archive/: the record stays whole and every file stays
under the plugin directory's 256 KiB.

Two halves. The script's own behaviour, against throwaway roots: it moves
exactly the entries past the newest KEEP, word for word, into the right
month, newest first, and moving twice changes nothing. And the repository as
it is: every file parses, every archive holds only its own month, the order
runs newest first across all of them, and none is near the limit.
"""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import archive_updates
import content_pipeline
import generate
import templates

REPO_ROOT = Path(__file__).resolve().parent.parent
PLUGIN_DIRECTORY_LIMIT = 256 * 1024

PREAMBLE = "# Update notes\n\nPreamble.\n\n"


def _entry(date: str, n: int) -> str:
    return f"## {date} — Change {n}\n\n- What changed, number {n}.\n  Wrapped onto a second line.\n"


def _updates(dates: list[str]) -> str:
    return PREAMBLE + "\n".join(_entry(d, i) for i, d in enumerate(dates))


def _entries_in_order(root: Path) -> list[str]:
    """Every entry across UPDATES.md and the archives, newest first."""
    files = [root / "UPDATES.md"] + sorted((root / archive_updates.ARCHIVE_DIR).glob("*.md"),
                                           reverse=True)
    out = []
    for path in files:
        out += [e.rstrip("\n") for e in archive_updates.split_entries(path.read_text(encoding="utf-8"))[1]]
    return out


class ArchiveScriptTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="archive-updates-test-"))

    def _write(self, dates: list[str]) -> list[str]:
        text = _updates(dates)
        (self.root / "UPDATES.md").write_text(text, encoding="utf-8")
        return [e.rstrip("\n") for e in archive_updates.split_entries(text)[1]]

    def test_nothing_moves_at_or_under_keep(self):
        self._write(["2026-09-01"] * archive_updates.KEEP)
        self.assertEqual(archive_updates.archive(self.root), [])
        self.assertFalse((self.root / archive_updates.ARCHIVE_DIR).exists())

    def test_everything_past_keep_moves_word_for_word_into_its_month(self):
        dates = ["2026-10-02"] * (archive_updates.KEEP - 1) + ["2026-10-01", "2026-09-30", "2026-09-29", "2026-08-31"]
        original = self._write(dates)
        archive_updates.archive(self.root)
        self.assertEqual(_entries_in_order(self.root), original)
        kept = archive_updates.split_entries((self.root / "UPDATES.md").read_text(encoding="utf-8"))
        self.assertEqual(kept[0].rstrip("\n"), PREAMBLE.rstrip("\n"))
        self.assertEqual(len(kept[1]), archive_updates.KEEP)
        for month in ("2026-09", "2026-08"):
            text = (self.root / archive_updates.ARCHIVE_DIR / f"{month}.md").read_text(encoding="utf-8")
            notes = content_pipeline.parse_update_notes(text)
            self.assertTrue(notes and all(n["date"].startswith(month) for n in notes))
        # October's entries are all among the newest KEEP, so October has no file.
        self.assertFalse((self.root / archive_updates.ARCHIVE_DIR / "2026-10.md").exists())

    def test_a_second_move_lands_above_the_first_in_the_same_month(self):
        first = self._write(["2026-09-10"] * (archive_updates.KEEP + 2))
        archive_updates.archive(self.root)
        # New entries arrive on top, and the file passes KEEP again.
        kept = (self.root / "UPDATES.md").read_text(encoding="utf-8")
        newer = "".join(_entry("2026-09-20", 100 + i) + "\n" for i in range(3))
        (self.root / "UPDATES.md").write_text(kept.replace("## ", newer + "## ", 1), encoding="utf-8")
        expected = [e.rstrip("\n") for e in archive_updates.split_entries(newer)[1]] + first
        archive_updates.archive(self.root)
        self.assertEqual(_entries_in_order(self.root), expected)

    def test_running_it_twice_changes_nothing(self):
        self._write(["2026-09-01"] * (archive_updates.KEEP + 5))
        archive_updates.archive(self.root)
        snapshot = {p: p.read_bytes() for p in self.root.rglob("*.md")}
        self.assertEqual(archive_updates.archive(self.root), [])
        self.assertEqual({p: p.read_bytes() for p in self.root.rglob("*.md")}, snapshot)

    def test_the_build_refuses_an_oversized_file_and_names_the_fix(self):
        filler = "x" * 120
        entries = "".join(f"## 2026-09-01 — Big {i}\n\n- {filler}\n\n" for i in range(1200))
        (self.root / "UPDATES.md").write_text(PREAMBLE + entries, encoding="utf-8")
        with self.assertRaises(SystemExit) as raised:
            generate._update_notes(self.root)
        self.assertIn("archive_updates.py", str(raised.exception))


class RepositoryTests(unittest.TestCase):
    def test_the_file_keeps_more_than_the_page_shows(self):
        self.assertGreaterEqual(archive_updates.KEEP, templates.UPDATE_NOTES_LIMIT)

    def test_updates_md_is_under_its_ceiling(self):
        self.assertLessEqual((REPO_ROOT / "UPDATES.md").stat().st_size, archive_updates.MAX_BYTES)

    def test_every_archive_parses_holds_its_month_and_fits_the_directory(self):
        archives = sorted((REPO_ROOT / archive_updates.ARCHIVE_DIR).glob("*.md"))
        self.assertTrue(archives)
        for path in archives:
            with self.subTest(archive=path.name):
                self.assertLess(path.stat().st_size, PLUGIN_DIRECTORY_LIMIT)
                notes = content_pipeline.parse_update_notes(path.read_text(encoding="utf-8"))
                self.assertTrue(notes)
                self.assertTrue(all(n["date"].startswith(path.stem) for n in notes))

    def test_the_record_runs_newest_first_across_every_file(self):
        dates = [e[3:13] for e in _entries_in_order(REPO_ROOT)]
        self.assertEqual(dates, sorted(dates, reverse=True))


if __name__ == "__main__":
    unittest.main()
