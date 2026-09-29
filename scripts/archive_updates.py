#!/usr/bin/env python3
"""Move all but the newest update notes from UPDATES.md into one file per month.

UPDATES.md ships inside the plugin folder, and Anthropic's plugin directory
holds any text file over 256 KiB for a reviewer. At one to four entries a day
the file would pass that within weeks, and a single archive for the year
would follow it soon after -- September 2026 alone was 200 KB. So the newest
KEEP entries stay in UPDATES.md, which is more than the landing page shows
(templates.UPDATE_NOTES_LIMIT), and everything older goes to
updates-archive/<YYYY-MM>.md, newest first, exactly as written. Nothing is
reworded, merged or dropped: the entries across all the files, in order, are
the same text UPDATES.md held.

Run it from the repository root, or from anywhere:

    python3 scripts/archive_updates.py

It is safe to run at any time and does nothing when there is nothing to move.
The build refuses to run once UPDATES.md passes MAX_BYTES, with this command
in the message, so it never has to be remembered.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

KEEP = 30
MAX_BYTES = 128 * 1024
ARCHIVE_DIR = "updates-archive"

_ENTRY_START = re.compile(r"(?m)^(?=## \d{4}-\d{2}-\d{2})")


def split_entries(text: str) -> tuple[str, list[str]]:
    """(preamble, [entry, ...]), each entry its heading through the line
    before the next heading, as written."""
    parts = _ENTRY_START.split(text)
    return parts[0], parts[1:]


def _join(preamble: str, entries: list[str]) -> str:
    body = "".join(entry.rstrip("\n") + "\n\n" for entry in entries)
    return (preamble.rstrip("\n") + "\n\n" + body).rstrip("\n") + "\n"


def _month_preamble(month: str) -> str:
    return (
        f"# Update notes, {month}\n\n"
        f"Entries from this month that have moved out of UPDATES.md, which keeps\n"
        f"the newest {KEEP}. Newest first, exactly as they were written there, in\n"
        f"the same shape; scripts/archive_updates.py moves them.\n"
    )


def archive(root: Path) -> list[Path]:
    """Move every entry past the newest KEEP. Returns the files it wrote."""
    updates = root / "UPDATES.md"
    preamble, entries = split_entries(updates.read_text(encoding="utf-8"))
    if len(entries) <= KEEP:
        return []
    kept, moved = entries[:KEEP], entries[KEEP:]

    by_month: dict[str, list[str]] = {}
    for entry in moved:
        by_month.setdefault(entry[3:10], []).append(entry)

    written = []
    for month, month_entries in by_month.items():
        path = root / ARCHIVE_DIR / f"{month}.md"
        if path.is_file():
            month_head, archived = split_entries(path.read_text(encoding="utf-8"))
        else:
            month_head, archived = _month_preamble(month), []
        # Whatever moves now is newer than whatever moved before it.
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(_join(month_head, month_entries + archived), encoding="utf-8")
        written.append(path)

    updates.write_text(_join(preamble, kept), encoding="utf-8")
    return [updates] + written


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    written = archive(root)
    if not written:
        print(f"archive_updates: UPDATES.md holds {KEEP} entries or fewer, nothing to move")
        return 0
    for path in written:
        print(f"archive_updates: wrote {path.relative_to(root)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
