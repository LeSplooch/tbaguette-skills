"""The About page: its content, its markup, and the contract between the three
files that make it (about_page.py writes the HTML, about.css dresses it,
about.js animates it).

    python3 scripts/test_about.py

Three kinds of thing are checked, because three kinds of thing went wrong
while it was being built, and each was found only by looking at the page in a
browser:

  * What the page says. The résumé is public, and most of what it describes
    lives in private repositories, so the content file is scanned for the
    things that must never reach it (an email, an address, a key-shaped
    string, a credential word) and for the quieter ways a claim goes stale: a
    count that no longer matches the library, a project listed under a family
    or an ingredient that does not exist.
  * That the page is a whole résumé without script, and that every address it
    writes carries the deployment base path. The seal's wheat and the search
    field's glass were first written without it and pointed at /assets/, which
    is a 404 on the published site (served under /tbaguette-skills/) and fine
    on a local one.
  * That the hooks agree. about.js finds things by data attribute and class
    name; about.css styles them by class name. A rename in any one of the
    three that misses the others breaks the page silently, so each side is
    checked against the others.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

import about_page
import about_profile as profile
import content_pipeline
import generate
import templates

REPO_ROOT = Path(__file__).resolve().parent.parent
ASSETS = REPO_ROOT / "docs" / "assets"
BASE = "/tbaguette-skills"

CATEGORIES = [{"slug": "testing", "title": "Testing", "skill_slugs": []}]


def _strings(value):
    """Every string inside a nested tuple/list/dict, for scanning."""
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from _strings(item)
    elif isinstance(value, (tuple, list)):
        for item in value:
            yield from _strings(item)


def _profile_strings():
    for name in dir(profile):
        if name.isupper():
            yield from _strings(getattr(profile, name))


def _keyframes(css: str, name: str) -> str:
    """The body of one @keyframes block, braces balanced."""
    start = css.index(f"@keyframes {name} {{") + len(f"@keyframes {name} {{")
    depth, i = 1, start
    while depth:
        depth += {"{": 1, "}": -1}.get(css[i], 0)
        i += 1
    return css[start:i - 1]


def _page(base_path: str = "") -> str:
    return about_page.render_about_page(CATEGORIES, base_path, skill_count=101)


def _main_of(html: str) -> str:
    return html.split("<main", 1)[1].split("</main>", 1)[0]


def _header_of(html: str) -> str:
    return html.split('<header class="site-header"', 1)[1].split("</header>", 1)[0]


class TestWhatThePageSays(unittest.TestCase):
    def test_slugs_are_unique_kebab_case(self):
        slugs = [p["slug"] for p in profile.PROJECTS]
        self.assertEqual(len(slugs), len(set(slugs)))
        for slug in slugs:
            self.assertRegex(slug, r"^[a-z0-9]+(-[a-z0-9]+)*$")

    def test_every_project_is_complete_and_well_formed(self):
        domains = profile.domain_by_slug()
        for p in profile.PROJECTS:
            with self.subTest(project=p["slug"]):
                self.assertIn(p["domain"], domains)
                self.assertIn(p["vis"], ("public", "private"))
                self.assertIn(p["scale"], (1, 2, 3, 4, 5))
                self.assertTrue(p["name"] and p["pitch"] and p["method"])
                for tag in p["tags"]:
                    self.assertIn(tag, profile.TAGS)
                self.assertEqual(len(p["tags"]), len(set(p["tags"])))
                for key in ("first", "last"):
                    self.assertRegex(p[key], r"^\d{4}-(0[1-9]|1[0-2])$")
                self.assertLessEqual(p["first"], p["last"])

    def test_only_a_public_project_carries_an_address(self):
        for p in profile.PROJECTS:
            with self.subTest(project=p["slug"]):
                if p["vis"] == "public":
                    self.assertTrue(p["url"].startswith("https://"))
                else:
                    self.assertNotIn("url", p)

    def test_every_domain_and_every_tag_is_used(self):
        used_domains = {p["domain"] for p in profile.PROJECTS}
        self.assertEqual(used_domains, {d[0] for d in profile.DOMAINS})
        used_tags = {t for p in profile.PROJECTS for t in p["tags"]}
        self.assertEqual(used_tags, set(profile.TAGS), "a tag nothing uses would never reach the pantry")

    def test_domain_colours_are_the_librarys_twelve(self):
        colours = [d[2] for d in profile.DOMAINS]
        self.assertEqual(len(colours), len(set(colours)))
        for colour in colours:
            self.assertIn(colour, range(1, 13))

    def test_claims_and_rules_cite_projects_that_exist(self):
        slugs = set(profile.project_by_slug())
        for _, text, evidence in list(profile.TITLE_CLAIMS) + list(profile.HOUSE_RULES):
            self.assertTrue(evidence)
            for slug in evidence:
                self.assertIn(slug, slugs)

    def test_the_title_is_its_three_claims(self):
        self.assertEqual([c[0] for c in profile.TITLE_CLAIMS],
                         [profile.TITLE_FR, profile.TITLE_ROLE, profile.TITLE_FIELD])

    def test_the_counters_are_derived_not_typed(self):
        stats = profile.stats()
        self.assertEqual(stats["projects"], len(profile.PROJECTS))
        self.assertEqual(profile.SINCE, int(min(p["first"] for p in profile.PROJECTS)[:4]))
        self.assertEqual(stats["languages"], sum(1 for t in profile.TAGS.values() if t[1] == "language"))
        self.assertEqual(stats["surfaces"], sum(1 for t in profile.TAGS.values() if t[1] == "platform"))
        self.assertGreaterEqual(stats["years"], 1)

    def test_the_eras_tile_the_record_in_order(self):
        eras = profile.ERAS
        for (start, end, name, text), nxt in zip(eras, eras[1:]):
            self.assertLessEqual(start, end)
            self.assertLess(end, nxt[0], f"{name!r} overlaps the era after it")
        self.assertLessEqual(eras[0][0], min(p["first"] for p in profile.PROJECTS))
        self.assertGreaterEqual(eras[-1][1], max(p["last"] for p in profile.PROJECTS))

    def test_the_library_counts_it_quotes_are_the_librarys(self):
        content = content_pipeline.build_content(str(REPO_ROOT / "skills"))
        first = profile.project_by_slug()["tbaguette-atelier"]["method"][0]
        self.assertIn(f"{generate.EXPECTED_SKILL_COUNT} skills", first)
        self.assertIn(f"{len(content['categories'])} families", first)

    def test_the_claim_with_numbers_in_it_formats_from_the_derived_stats(self):
        stats = profile.stats()
        claim = profile.TITLE_CLAIMS[0][1].format(
            Years=about_page._word(stats["years"]).capitalize(),
            languages=about_page._word(stats["languages"]),
            surfaces=about_page._word(stats["surfaces"]))
        self.assertIn(f"{about_page._word(stats['languages'])} languages", claim)
        self.assertNotIn("{", claim)

    # -- the things that must never reach a public page ---------------------

    ALLOWED_HOSTS = {"github.com", "lesplooch.github.io", "daydreamcorp.github.io"}

    def test_no_email_address(self):
        for s in _profile_strings():
            self.assertNotRegex(s, r"[\w.+-]+@[\w-]+\.[\w.]+", s)

    def test_no_ip_address(self):
        for s in _profile_strings():
            self.assertNotRegex(s, r"\b\d{1,3}(\.\d{1,3}){3}\b", s)

    def test_no_key_shaped_string_and_no_credential_word(self):
        for s in _profile_strings():
            self.assertNotRegex(s, r"[A-Za-z0-9_\-]{32,}", f"long token-like run in {s!r}")
            self.assertNotRegex(s, r"\b(sk|ghp|gho|AKIA|xox[bap])[-_][A-Za-z0-9]{8,}", s)
            self.assertNotRegex(s.lower(), r"\b(password|passwd|credential|private key|seed phrase|wallet address)", s)

    def test_every_address_is_a_known_host(self):
        for s in _profile_strings():
            for url in re.findall(r"https?://[^\s\"']+", s):
                host = re.match(r"https?://([^/]+)", url).group(1)
                self.assertIn(host, self.ALLOWED_HOSTS, url)
        self.assertTrue(profile.GITHUB_URL.startswith("https://github.com/"))

    def test_nothing_describes_how_to_get_around_anything(self):
        banned = ("bypass", "inject", "memory offset", "anti-cheat", "premium", "blocklist",
                  "telemetry", "expiry", "obfuscat")
        for s in _profile_strings():
            for word in banned:
                self.assertNotIn(word, s.lower(), s)

    def test_no_project_claims_a_working_relationship_nobody_observed(self):
        # "Built for X" says a client exists. What the record shows is where a
        # thing is published and whose name is on its policy; say that.
        for s in _profile_strings():
            for phrase in ("built for", "commissioned", "on behalf of", "client of", "employer"):
                self.assertNotIn(phrase, s.lower(), s)

    def test_no_third_party_work_is_listed_as_this_persons(self):
        # Repositories that are copies of other people's projects or stock
        # templates, found while reading the account. Their names would be an
        # honest-looking way to claim them.
        names = " ".join(p["name"].lower() + " " + p["slug"] for p in profile.PROJECTS)
        for copied in ("auto-gpt", "autogpt", "rotnomorecompose", "rotnomorereact", "voiceecho",
                       "food expiration", "material kit", "shinyfirefly"):
            self.assertNotIn(copied, names)


class TestThePageIsAWholeResumeWithoutScript(unittest.TestCase):
    def setUp(self):
        self.html = _page()
        self.main = _main_of(self.html)

    def _entry(self, slug):
        return re.search(rf'<article class="entry" id="project={re.escape(slug)}".*?</article>', self.main, re.S).group(0)

    def test_it_is_a_document_with_its_own_title_and_description(self):
        self.assertIn("<title>About TBaguette — ", self.html)
        self.assertRegex(self.html, r'<meta name="description" content="[^"]*26 projects')

    def test_the_name_is_one_accessible_heading_and_the_headings_nest(self):
        self.assertEqual(self.main.count("<h1"), 1)
        self.assertIn('aria-label="TBaguette"', self.main)
        for ch in re.findall(r'<span class="open__ch"[^>]*>', self.main):
            self.assertIn('aria-hidden="true"', ch)
        levels = [int(m) for m in re.findall(r"<h([1-4])[ >]", self.main)]
        self.assertEqual(levels[0], 1)
        for before, after in zip(levels, levels[1:]):
            self.assertLessEqual(after - before, 1, "a heading level was skipped")

    def test_nothing_sits_above_a_heading_as_a_kicker(self):
        # Formidable bans the eyebrow: the heading carries itself. No heading
        # in the page is directly preceded by a label-sized paragraph or span.
        for heading in re.finditer(r"<h[234][^>]*>", self.main):
            before = self.main[:heading.start()].rstrip()
            self.assertNotRegex(before, r"<(p|span)[^>]*class=\"[^\"]*(eyebrow|kicker|stage|__label|__domain|__when|__k)[^\"]*\"[^>]*>[^<]*</(p|span)>$")
        for banned in ("eyebrow", "kicker", "about-chapter__stage", "about-chapter__num"):
            self.assertNotIn(banned, self.main)

    def test_every_project_is_on_the_page_with_its_method(self):
        for p in profile.PROJECTS:
            with self.subTest(project=p["slug"]):
                entry = self._entry(p["slug"])
                for step in p["method"]:
                    self.assertIn(templates.escape_html(step), entry)

    def test_text_is_escaped(self):
        self.assertIn("Deckhand site &amp; feeds", self.main)
        self.assertNotIn("Deckhand site & feeds", self.main)

    def test_every_private_project_says_so_and_links_nowhere(self):
        for p in profile.PROJECTS:
            entry = self._entry(p["slug"])
            if p["vis"] == "private":
                self.assertIn("Private repository", entry)
                self.assertNotIn("<a ", entry)
            else:
                self.assertIn(f'href="{p["url"]}"', entry)

    def test_the_cuts_on_each_loaf_are_its_scale(self):
        for p in profile.PROJECTS:
            self.assertEqual(self._entry(p["slug"]).count('class="crown__cut"'), p["scale"], p["slug"])

    def test_the_family_is_said_in_words_as_well_as_colour(self):
        domains = profile.domain_by_slug()
        for p in profile.PROJECTS:
            self.assertIn(f'<span class="entry__family">{templates.escape_html(domains[p["domain"]][1])}</span>',
                          self._entry(p["slug"]))

    def test_every_project_appears_once_and_in_exactly_one_era(self):
        self.assertEqual(self.main.count('<article class="entry"'), len(profile.PROJECTS))
        for era in re.findall(r'<section class="era".*?</section>', self.main, re.S):
            self.assertGreaterEqual(era.count('<article class="entry"'), 1)
        self.assertEqual(self.main.count('class="era"'), len(profile.ERAS))

    def test_eras_run_newest_first(self):
        years = re.findall(r'<span class="era__years">(\d{4})', self.main)
        self.assertEqual(years, sorted(years, reverse=True))

    def test_the_method_is_a_native_disclosure_not_a_dialog(self):
        self.assertEqual(self.main.count("<details class=\"entry__more\""), len(profile.PROJECTS))
        self.assertNotIn("<dialog", self.html)
        self.assertNotIn("aria-haspopup", self.html)

    def test_a_link_to_a_project_works_with_no_script_because_its_id_is_the_fragment(self):
        for slug in re.findall(r'href="#project=([a-z0-9-]+)"', self.main):
            self.assertIn(f'id="project={slug}"', self.main)

    def test_the_title_and_the_rules_cite_projects_by_link(self):
        for _, _, evidence in list(profile.TITLE_CLAIMS) + list(profile.HOUSE_RULES):
            for slug in evidence:
                self.assertIn(f'href="#project={slug}"', self.main)

    def test_what_a_stranger_can_open_is_said_first(self):
        opening = self.main.split('id="title"', 1)[0]
        self.assertIn("Open today:", opening)
        for _, url in profile.OPEN_TODAY:
            self.assertIn(f'href="{url}"', opening)

    def test_no_stat_strip_and_no_modal_survive_the_rework(self):
        for gone in ("about-stats", "data-count", "recipe", "pantry", "anatomy", "about-seal", "about-hero__dust"):
            self.assertNotIn(gone, self.html)

    def test_it_goes_live_before_first_paint_and_can_take_it_back(self):
        script = re.search(r"<script>(\(function\(\)\{var r=document.*?)</script>", self.html).group(1)
        self.assertIn("classList.add('about-live')", script)
        self.assertIn("classList.remove('about-live')", script)
        self.assertIn("window.TBaguetteAbout", script)
        self.assertLess(self.html.index(script), self.html.index('class="open"'))

    def test_it_says_what_was_left_off_the_shelf(self):
        # A reader who asked for "all the projects" and counts 26 should be told
        # what the rule for leaving one out is, not left to wonder.
        self.assertIn("Forks and unmodified templates are not listed", self.main)

    def test_the_colophon_does_not_call_a_page_made_in_a_session_hand_baked(self):
        colophon = re.search(r'<p class="counter__colophon">(.*?)</p>', self.main, re.S).group(1)
        self.assertIn("Claude Code", colophon)
        self.assertNotIn("by hand", colophon)

    def test_the_counter_names_the_librarys_size_or_says_nothing_false(self):
        self.assertIn("101 skills for AI agents", self.main)
        self.assertIn("Skills for AI agents", about_page.render_about_page(CATEGORIES, "", skill_count=0))


class TestEveryAddressCarriesTheBasePath(unittest.TestCase):
    """The bug that started this class: pointing at /assets/ is right on a
    local server and a 404 under /tbaguette-skills/."""

    def setUp(self):
        self.html = _page(BASE)

    def test_internal_links_and_assets_are_prefixed(self):
        internal = re.findall(r'(?:href|src)="(/[^"]*)"', self.html)
        self.assertTrue(internal)
        self.assertEqual([h for h in internal if not h.startswith(BASE + "/")], [])

    def test_every_sprite_icon_is_prefixed(self):
        uses = re.findall(r'<use href="([^"#]*)#', self.html)
        self.assertGreaterEqual(len(uses), 4)  # wordmark, theme toggle x2, search
        self.assertEqual([u for u in uses if not u.startswith(BASE + "/assets/icons.svg")], [])

    def test_the_page_loads_its_own_files_and_nobody_else_does(self):
        self.assertIn(f'<link rel="stylesheet" href="{BASE}/assets/about.css">', self.html)
        self.assertLess(self.html.index("/assets/styles.css"), self.html.index("/assets/about.css"))
        self.assertIn(f'<script src="{BASE}/assets/about.js" defer></script>', self.html)
        other = templates.render_getting_started_page(CATEGORIES, BASE, skill_count=101)
        self.assertNotIn("about.css", other)
        self.assertNotIn("about.js", other)

    def test_files_are_fingerprinted_when_the_build_knows_them(self):
        saved = dict(templates.ASSET_VERSIONS)
        try:
            templates.ASSET_VERSIONS.update({"about.css": "c0ffee", "about.js": "facade"})
            html = _page(BASE)
        finally:
            templates.ASSET_VERSIONS.clear()
            templates.ASSET_VERSIONS.update(saved)
        self.assertIn("about.css?v=c0ffee", html)
        self.assertIn("about.js?v=facade", html)
        self.assertIn("about.css", generate.VERSIONED_ASSETS)
        self.assertIn("about.js", generate.VERSIONED_ASSETS)


class TestTheHeaderButton(unittest.TestCase):
    def test_every_page_carries_it_after_the_graph(self):
        pages = {
            "getting-started": templates.render_getting_started_page(CATEGORIES, skill_count=101),
            "about": _page(),
        }
        for name, html in pages.items():
            with self.subTest(page=name):
                header = _header_of(html)
                match = re.search(r'<a class="site-header__nav-link site-header__nav-link--about" href="([^"]*)"', header)
                self.assertIsNotNone(match)
                self.assertEqual(match.group(1), "/about/")
                self.assertIn("About TBaguette</span>", header)
                self.assertLess(header.index(">Graph<"), header.index("About TBaguette"))

    def test_it_marks_itself_current_on_its_own_page_only(self):
        about = _header_of(_page())
        self.assertIn('href="/about/" aria-current="page"', about)
        self.assertNotIn('href="/getting-started/" aria-current', about)
        other = _header_of(templates.render_getting_started_page(CATEGORIES, skill_count=101))
        self.assertNotIn('href="/about/" aria-current', other)

    def test_its_mark_is_decorative(self):
        header = _header_of(_page())
        self.assertRegex(header, r'<svg class="icon site-header__nav-icon about-mark"[^>]*aria-hidden="true"')


class TestTheTimeline(unittest.TestCase):
    def setUp(self):
        self.first_year = int(min(p["first"] for p in profile.PROJECTS)[:4])
        self.last_year = int(max(p["last"] for p in profile.PROJECTS)[:4])
        self.edges = about_page._warp(list(profile.PROJECTS), self.first_year, self.last_year)
        self.placed = about_page._place_dots(list(profile.PROJECTS), self.first_year, self.last_year, self.edges)

    def test_the_axis_runs_from_zero_to_a_hundred_and_never_backwards(self):
        self.assertEqual(len(self.edges), (self.last_year - self.first_year + 1) * 12 + 1)
        self.assertEqual(self.edges[0], 0.0)
        self.assertAlmostEqual(self.edges[-1], 100.0)
        self.assertTrue(all(b > a for a, b in zip(self.edges, self.edges[1:])))

    def test_a_month_with_work_gets_more_room_than_a_quiet_one(self):
        start = self.first_year * 12
        busy = about_page._month_index("2026-08") - start
        quiet = about_page._month_index("2020-08") - start
        self.assertGreater(self.edges[busy + 1] - self.edges[busy], self.edges[quiet + 1] - self.edges[quiet])

    def test_every_dot_lies_in_its_own_month(self):
        start = self.first_year * 12
        for project, x, _ in self.placed:
            m = about_page._month_index(project["first"]) - start
            self.assertTrue(self.edges[m] <= x <= self.edges[m + 1], project["slug"])

    def test_no_two_dots_in_a_lane_touch(self):
        lanes = {}
        for project, x, lane in self.placed:
            lanes.setdefault(lane, []).append((x, about_page._dot_size_rem(project["scale"])))
        for lane, dots in lanes.items():
            for (xa, sa), (xb, sb) in zip(dots, dots[1:]):
                gap_rem = (xb - xa) / 100 * about_page._TRACK_REM
                self.assertGreaterEqual(gap_rem, (sa + sb) / 2, f"lane {lane}")

    def test_the_stack_stays_short_enough_to_read(self):
        self.assertLessEqual(max(lane for _, _, lane in self.placed) + 1, 4)

    def test_every_project_has_a_dot_and_every_dot_is_a_link_to_its_entry(self):
        html = _page()
        self.assertEqual(html.count("data-rise-slot"), len(profile.PROJECTS))
        for p in profile.PROJECTS:
            self.assertIn(f'<a class="rise__dot" href="#project={p["slug"]}">', html)

    def test_an_open_era_does_not_promise_a_year_that_has_not_happened(self):
        last_year = max(p["last"] for p in profile.PROJECTS)[:4]
        label = about_page._era_years(profile.ERAS[-1][0], profile.ERAS[-1][1], max(p["last"] for p in profile.PROJECTS))
        self.assertTrue(label.endswith(last_year))


class TestTheThreeFilesAgree(unittest.TestCase):
    """about.js finds things by data attribute and class; about.css styles by
    class. Each side is checked against what about_page.py really writes."""

    DYNAMIC = {"rise__now"}  # built by about.js itself

    @classmethod
    def setUpClass(cls):
        cls.html = _page()
        cls.main = _main_of(cls.html)
        cls.js = (ASSETS / "about.js").read_text(encoding="utf-8")
        cls.css = (ASSETS / "about.css").read_text(encoding="utf-8")
        cls.shared_css = (ASSETS / "styles.css").read_text(encoding="utf-8")

    def _selectors_in_js(self):
        out = []
        for match in re.finditer(r"""(?:\$\$?|querySelector(?:All)?)\(\s*(['"])(.+?)\1""", self.js):
            out.append(match.group(2))
        return out

    def test_search_reads_the_ingredient_names_as_well_as_the_prose(self):
        # "wasm" is the ingredient's name but appears nowhere in its label,
        # "WebAssembly": a search built from the prose alone finds nothing.
        haystack = re.search(r"hay:\s*(.+?),\n", self.js).group(1)
        self.assertIn("data-tags", haystack)

    def test_every_hook_the_script_looks_for_exists_in_the_markup(self):
        selectors = self._selectors_in_js()
        self.assertGreater(len(selectors), 20)
        for selector in selectors:
            for attr in re.findall(r"\[data-([a-z-]+)", selector):
                self.assertIn(f"data-{attr}", self.main + self.html, f"{selector!r} finds nothing")
            for cls in re.findall(r"\.([A-Za-z_][\w-]*)", selector):
                if cls in self.DYNAMIC or cls.startswith("is-"):
                    continue
                self.assertRegex(self.main, rf'class="[^"]*\b{re.escape(cls)}\b', f"{selector!r} finds nothing")

    def test_every_class_the_markup_uses_is_styled_somewhere(self):
        used = set()
        for value in re.findall(r'class="([^"]+)"', self.main):
            used.update(value.split())
        self.assertGreater(len(used), 60)
        unstyled = [c for c in sorted(used) if f".{c}" not in self.css and f".{c}" not in self.shared_css]
        self.assertEqual(unstyled, [], "written by about_page.py and styled nowhere")

    def test_every_state_class_the_script_sets_has_a_rule(self):
        for state in ("about-live", "loaf-seen", "is-unborn", "is-born", "is-target"):
            self.assertIn(state, self.js, f"{state} is no longer set by about.js")
            self.assertIn(state, self.css, f"{state} is set by about.js and styled nowhere")

    def test_every_entrance_degrades_to_appearing_not_to_absence(self):
        # motion.md: put the visible state in the base rule and let the
        # animation supply only the from-state. Written the other way round it
        # looks identical while the motion runs and inverts the failure: skip
        # the animation and the element is left at a hidden resting style.
        entrances = ("about-letter", "about-rise", "about-bake", "about-score", "about-loaf-in")
        uses = re.findall(r"animation:\s*([^;]+);", self.css)
        used = [u for u in uses if any(name in u for name in entrances)]
        self.assertGreaterEqual(len(used), 7)
        for value in used:
            self.assertIn("backwards", value, f"{value!r} fills forwards: the end state would be the animation's, not the page's")
            self.assertNotRegex(value, r"\b(both|forwards)\b")
        for name in entrances:
            body = _keyframes(self.css, name)
            self.assertIn("from", body)
            self.assertNotRegex(body, r"\bto\s*\{", f"{name} defines a to-frame: the resting style should be the end")
        # and nothing in the base rules starts the page hidden
        for match in re.finditer(r"\.about-live[^{,]*\{[^}]*opacity:\s*0[;\s]", self.css):
            self.assertNotRegex(match.group(0), r"open__|loaf-art|about-stats", "an entrance starts hidden at rest")

    def test_the_stylesheet_never_hides_anything_outside_the_live_layer(self):
        for match in re.finditer(r"[^{}]+\{[^{}]*opacity:\s*0[;\s][^{}]*\}", self.css):
            rule = match.group(0)
            selector = rule.split("{", 1)[0].strip()
            if re.fullmatch(r"(from|to|\d+%(\s*,\s*\d+%)*)", selector):
                continue  # a keyframe's frame, not a rule that applies to anything
            if ".rise__tip" in selector:
                continue  # a tooltip: shown on hover and focus, and its name is in the dot's own label
            self.fail(f"hidden outside the live layer: {selector}")

    def test_motion_is_screen_only_and_respects_the_readers_preference(self):
        self.assertNotIn("@media (prefers-reduced-motion: no-preference)", self.css)
        self.assertIn("@media screen and (prefers-reduced-motion: no-preference)", self.css)
        self.assertIn("@media print", self.css)

    def test_both_files_stay_under_the_plugin_directorys_ceiling(self):
        for path in (ASSETS / "about.css", ASSETS / "about.js"):
            self.assertLess(path.stat().st_size, 256 * 1024, path.name)

    def test_the_script_is_one_guarded_closure(self):
        self.assertTrue(self.js.rstrip().endswith("})();"))
        self.assertIn("'use strict'", self.js)
        self.assertIn("if (window.TBaguetteAbout) { return; }", self.js)


if __name__ == "__main__":
    unittest.main(verbosity=2)
