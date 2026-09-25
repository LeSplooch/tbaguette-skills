"""Does the Crumb a commit publishes describe the skills it publishes?

The Crumb is the skill graph: docs/graph/graph.json, the /graph/ page that
draws it, and the landing page's banner that summarises it. All three are
generated from skills/ by scripts/generate.py (see skill_graph.py), and the
pre-commit hook regenerates them on every commit, so in the ordinary flow they
cannot fall behind. Three routes still publish a Crumb describing some other
set of skills:

- a commit made with hooks off -- the nightly routine lands its source-only
  commits that way, ahead of one closing site commit;
- a merge, where git runs pre-merge-commit and never pre-commit;
- a commit whose hook regenerated from a working tree holding edits that were
  never staged -- in this shared checkout, often another session's.

Pushing to master is publishing here, so .githooks/pre-push runs this on
exactly what is about to go live: the pushed commit's own skills, built by
that commit's own code, against that commit's own generated files. Nothing in
the working tree is read, which is what makes it safe to run while another
session is mid-edit in it, and what keeps a Crumb built by older code from
being judged by newer code.

It also checks the one hand-written thing the Crumb needs per family: a
colour. The palette in docs/assets/styles.css is validated as a set (see the
comment above it), one entry per category and theme. A category added without
one is drawn in the fallback accent, and nothing else would ever say so.

Usage:
    python3 scripts/crumb_check.py [REV] [--live]

REV defaults to HEAD. --live also fetches the published graph.json and checks
that the deploy caught up with REV -- run it after the Pages build finishes.
Exits 1 and names every mismatch, 2 when the check could not run at all.

It executes REV's own scripts/ to build the graph. That is the point, and it
is also why REV must be a commit you would run anyway -- your own, or one
already merged -- and never an unreviewed pull request's head.

Stdlib only, like the rest of the generator.
"""

from __future__ import annotations

import argparse
import html
import io
import json
import re
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The base path of the build GitHub Pages serves -- the same literal
# .githooks/pre-commit hands generate.py. A Crumb built for any other base
# path links every skill to a page that does not exist on the live site.
PAGES_BASE_PATH = "/tbaguette-skills"
LIVE_GRAPH_URL = "https://lesplooch.github.io/tbaguette-skills/graph/graph.json"

GRAPH_JSON = "docs/graph/graph.json"
GRAPH_PAGE = "docs/graph/index.html"
LANDING_PAGE = "docs/index.html"
STYLES = "docs/assets/styles.css"

# Stamped from git history against the clock of whichever build wrote them,
# so two builds of identical skills disagree on them as the hours pass.
CLOCK_FIELDS = ("change_status", "change_at")

# Run inside a copy of the revision's scripts/, beside its skills/. Prints one
# JSON object: the graph plus the summary numbers the pages carry, or null for
# a revision that predates the graph.
_BUILD_SNIPPET = r"""
import json, sys
sys.path.insert(0, ".")
try:
    import skill_graph
except ImportError:
    print("null")
    raise SystemExit(0)
import content_pipeline, generate, locales, templates

base_path = sys.argv[1]
content = content_pipeline.build_content(
    "../skills", locale=None, locale_root=None,
    resolve_skill_link=generate._skill_link_resolver(base_path, locales.DEFAULT_LOCALE))
graph = skill_graph.build_graph(
    content, skill_url_template=templates.skill_url("{slug}", base_path, locales.DEFAULT_LOCALE))
pair_count = len(skill_graph.edge_weights(graph))
lede = getattr(templates.ENGLISH_STRINGS, "graph_page_lede_template", "")
if lede:
    lede = lede.format(skill_count=len(content["skills"]), pair_count=pair_count)
banner = skill_graph.banner_summary(graph) if hasattr(skill_graph, "banner_summary") else None
print(json.dumps({"graph": graph, "pair_count": pair_count, "lede": lede, "banner": banner},
                 ensure_ascii=False))
"""

_CSS_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
_CSS_BLOCK_RE = re.compile(r"([^{}]+)\{([^{}]*)\}")
_PALETTE_ENTRY_RE = re.compile(r"--graph-cat-(\d+)\s*:")
_BANNER_TAG_RE = re.compile(r'<aside class="graph-banner"[^>]*>')


class CheckError(Exception):
    """The check could not run at all -- a bad revision, no git, a build that
    crashed. Distinct from a mismatch, which is a finding."""


def _git(*args: str, root: Path = REPO_ROOT) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=root, capture_output=True, check=False)


def resolve(rev: str, root: Path = REPO_ROOT) -> str:
    result = _git("rev-parse", "--verify", "--quiet", f"{rev}^{{commit}}", root=root)
    if result.returncode != 0:
        raise CheckError(f"{rev!r} is not a commit in {root}")
    return result.stdout.decode().strip()


def read_at(sha: str, path: str, root: Path = REPO_ROOT) -> str | None:
    result = _git("show", f"{sha}:{path}", root=root)
    return result.stdout.decode("utf-8") if result.returncode == 0 else None


def build_at(sha: str, root: Path = REPO_ROOT, base_path: str = PAGES_BASE_PATH) -> dict | None:
    """What generate.py would write into the Crumb for this commit, computed
    by this commit's own code from this commit's own skills. None when the
    commit predates the graph.

    Only scripts/ and skills/ are extracted, because that is all the English
    build reads today. The day it reads anything else, the end-to-end test in
    test_crumb_check.py -- whose fixture holds the same two directories --
    fails before any push does."""
    archive = _git("archive", "--format=tar", sha, "--", "scripts", "skills", root=root)
    if archive.returncode != 0:
        raise CheckError(f"git archive {sha[:10]} failed: {archive.stderr.decode().strip()}")
    with tempfile.TemporaryDirectory(prefix="crumb-check-") as tmp:
        with tarfile.open(fileobj=io.BytesIO(archive.stdout)) as tar:
            if hasattr(tarfile, "data_filter"):
                tar.extractall(tmp, filter="data")
            else:  # Python before 3.12; the archive comes from our own object store.
                tar.extractall(tmp)
        result = subprocess.run(
            [sys.executable, "-c", _BUILD_SNIPPET, base_path],
            cwd=Path(tmp) / "scripts", capture_output=True, text=True, check=False,
        )
    if result.returncode != 0:
        tail = "\n".join(result.stderr.strip().splitlines()[-6:])
        raise CheckError(f"building the graph at {sha[:10]} failed:\n{tail}")
    return json.loads(result.stdout)


def without_clock(graph: dict) -> dict:
    clean = json.loads(json.dumps(graph))
    for skill in clean.get("skills", []):
        for field in CLOCK_FIELDS:
            skill.pop(field, None)
    return clean


def _names(items: list[str], limit: int = 8) -> str:
    shown = ", ".join(items[:limit])
    return shown + (f" and {len(items) - limit} more" if len(items) > limit else "")


def compare_graphs(fresh: dict, published: dict, *, label: str = GRAPH_JSON) -> list[str]:
    """Every way published differs from fresh, ignoring the clock fields.
    An empty list means they are the same graph."""
    a, b = without_clock(fresh), without_clock(published)
    if a == b:
        return []
    problems = []
    if a.get("skill_url_template") != b.get("skill_url_template"):
        problems.append(
            f"{label} links skills at {b.get('skill_url_template')!r}, but the Pages build "
            f"links them at {a.get('skill_url_template')!r} -- it was built for another base path"
        )
    fresh_families = [c.get("title") for c in a.get("categories", [])]
    published_families = [c.get("title") for c in b.get("categories", [])]
    if fresh_families != published_families:
        problems.append(
            f"{label} lists other families than the skills do -- "
            f"{len(published_families)} there, {len(fresh_families)} in the skills: "
            f"{_names(published_families)} vs {_names(fresh_families)}"
        )
    elif a.get("categories") != b.get("categories"):
        problems.append(f"{label} files some skills under a different family than the skills do")
    fresh_skills = {s["slug"]: s for s in a.get("skills", [])}
    published_skills = {s["slug"]: s for s in b.get("skills", [])}
    missing = [slug for slug in fresh_skills if slug not in published_skills]
    gone = [slug for slug in published_skills if slug not in fresh_skills]
    moved = [slug for slug in fresh_skills
             if slug in published_skills and fresh_skills[slug] != published_skills[slug]]
    if missing:
        problems.append(f"missing from {label}: {_names(missing)}")
    if gone:
        problems.append(f"in {label} but no longer skills: {_names(gone)}")
    if moved:
        problems.append(f"out of date in {label}: {_names(moved)}")
    if not problems:
        problems.append(f"{label} differs from a fresh build in its shape or its order")
    return problems


def page_problems(page_html: str | None, lede: str, *, label: str = GRAPH_PAGE) -> list[str]:
    if not lede:
        return []
    if page_html is None:
        return [f"{label} is missing"]
    if any(form in page_html for form in {lede, html.escape(lede, quote=False), html.escape(lede)}):
        return []
    return [f"{label} does not carry the summary a fresh build writes: {lede!r}"]


def banner_problems(landing_html: str | None, banner: dict | None, *,
                    label: str = LANDING_PAGE) -> list[str]:
    """The landing page's graph banner, while it is up. Only the two numbers
    lists it carries are compared; anything else about it is layout."""
    if not landing_html or not banner:
        return []
    tag = _BANNER_TAG_RE.search(landing_html)
    if not tag:
        return []
    problems = []
    expected = {
        "families": [f["count"] for f in banner.get("families", [])],
        "links": banner.get("links", []),
    }
    for attribute, value in expected.items():
        found = re.search(rf'data-{attribute}="([^"]*)"', tag.group(0))
        if found and json.loads(html.unescape(found.group(1))) != value:
            problems.append(f"the graph banner on {label} carries stale {attribute}")
    return problems


def palette_blocks(css: str) -> dict[str, set[int]]:
    """Selector -> the family numbers its rule gives a --graph-cat-N colour,
    for every rule that gives any. One rule per theme, today."""
    blocks: dict[str, set[int]] = {}
    for selector, body in _CSS_BLOCK_RE.findall(_CSS_COMMENT_RE.sub("", css)):
        defined = {int(n) for n in _PALETTE_ENTRY_RE.findall(body)}
        if defined:
            blocks.setdefault(" ".join(selector.split()), set()).update(defined)
    return blocks


def palette_problems(css: str | None, family_count: int, *, label: str = STYLES) -> list[str]:
    """Every rule that sets the family palette must name a colour for every
    family -- a partial theme would mix two palettes, and a missing entry
    falls back to the accent colour without a word."""
    if family_count == 0:
        return []
    if css is None:
        return [f"{label} is missing"]
    blocks = palette_blocks(css)
    if not blocks:
        return [f"{label} defines no --graph-cat-N family colours at all"]
    problems = []
    for selector, defined in blocks.items():
        absent = [n for n in range(1, family_count + 1) if n not in defined]
        if absent:
            problems.append(
                f"{label}: `{selector}` has no colour for family "
                f"{', '.join(str(n) for n in absent)} of {family_count} -- extend the palette "
                "and revalidate it as a set, per the comment above it"
            )
    return problems


def check(rev: str = "HEAD", *, root: Path = REPO_ROOT, live_url: str | None = None) -> tuple[str, list[str]]:
    """(summary line, problems). No problems means the Crumb at rev matches
    the skills at rev -- and, with live_url, that the deployed one does too."""
    sha = resolve(rev, root)
    published = read_at(sha, GRAPH_JSON, root)
    if published is None:
        return f"{sha[:10]}: no Crumb at this commit, nothing to check", []
    built = build_at(sha, root)
    if built is None:
        return f"{sha[:10]}: this commit's code predates the graph, nothing to check", []

    graph = built["graph"]
    family_count = len(graph.get("categories", []))
    problems = compare_graphs(graph, json.loads(published))
    problems += page_problems(read_at(sha, GRAPH_PAGE, root), built.get("lede", ""))
    problems += banner_problems(read_at(sha, LANDING_PAGE, root), built.get("banner"))
    problems += palette_problems(read_at(sha, STYLES, root), family_count)

    if live_url:
        # The query string is only a cache key: without it, a CDN edge can
        # answer with the copy from before the deploy for several minutes.
        try:
            with urllib.request.urlopen(f"{live_url}?rev={sha[:12]}", timeout=30) as response:
                live = json.loads(response.read().decode("utf-8"))
        except (OSError, ValueError) as error:
            raise CheckError(f"could not read the live graph.json at {live_url}: {error}") from error
        problems += compare_graphs(graph, live, label=f"the live graph.json ({live_url})")

    summary = (f"{sha[:10]}: the Crumb matches its skills -- {len(graph.get('skills', []))} skills, "
               f"{built.get('pair_count', 0)} cross-references, {family_count} families"
               + (", live site included" if live_url else ""))
    return summary, problems


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("rev", nargs="?", default="HEAD", help="commit to check (default HEAD)")
    parser.add_argument("--live", action="store_true",
                        help="also compare the deployed graph.json with REV's")
    parser.add_argument("--live-url", default=LIVE_GRAPH_URL, metavar="URL",
                        help=f"where --live reads it from (default {LIVE_GRAPH_URL})")
    args = parser.parse_args(argv)
    try:
        sha = resolve(args.rev)
        summary, problems = check(sha, live_url=args.live_url if args.live else None)
    except CheckError as error:
        print(f"crumb_check: {error}", file=sys.stderr)
        return 2
    if problems:
        print(f"crumb_check: {sha[:10]}: the Crumb does not match its skills:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    print(f"crumb_check: {summary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
