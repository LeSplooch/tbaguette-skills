"""Self-test for templates.py, run before any real content exists.

Renders the locked fixture from the design brief through both public
functions, writes the output to _preview/ so it can be opened in a browser,
and asserts a handful of sanity checks. Stdlib only.

Usage:
    python3 scripts/test_templates.py
"""

from pathlib import Path

from templates import (
    INSTALL_COMMAND,
    INSTALL_TEST_GITHUB_URL,
    escape_html,
    render_index,
    render_skill_page,
    render_verify_install_page,
)

# ---------------------------------------------------------------------------
# The fixture from the original design brief. Only additive edits belong
# here (a new key that doesn't disturb an existing substring check) — this
# is why formidable_craft_floor_html was added directly below rather than
# given its own fixture. Anything that needs deliberately dangerous or
# structurally different input (special characters, a different base_path)
# gets its own small dedicated fixture instead, in check_escaping() and
# check_base_path() below, so a targeted test can't accidentally weaken an
# existing one.
# ---------------------------------------------------------------------------

FIXTURE = {
    "categories": [
        {"slug": "ui-and-design", "title": "UI and design", "skill_slugs": ["formidable"]},
        {"slug": "testing", "title": "Testing", "skill_slugs": ["designing-test-data", "flaky-test-triage"]},
    ],
    "skills": {
        "formidable": {
            "slug": "formidable", "name": "formidable",
            "category_slug": "ui-and-design", "category_title": "UI and design",
            "description": "Use when designing, redesigning, critiquing, auditing, polishing, or hardening any user interface on any stack.",
            "summary": "Design craft for every UI stack, not just web.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Design that earns to be called <strong>out-of-distribution</strong> craft.</p>",
            "is_formidable": True,
            "formidable_craft_floor_html": "<p>Load immediately before editing UI.</p>",
            "formidable_stacks": [
                {"id": "stack-web", "title": "Web", "html": "<p>Effectively unlimited color, type, and motion.</p>"},
                {"id": "stack-terminal-tui", "title": "Terminal / TUI", "html": "<p>A grid of character cells.</p>"},
            ],
            "formidable_commands": [
                {"id": "cmd-shape", "title": "Shape", "html": "<p>Decide before you build.</p>"},
                {"id": "cmd-critique", "title": "Critique", "html": "<p>Design review with a verdict.</p>"},
            ],
        },
        "designing-test-data": {
            "slug": "designing-test-data", "name": "designing-test-data",
            "category_slug": "testing", "category_title": "Testing",
            "description": "Use when a test's setup is longer than its assertions, when fixtures are shared across files.",
            "summary": "Builders over shared fixtures; the one-obvious-difference rule.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Build test data so the reason a test exists is visible in its setup.</p><table><thead><tr><th>Symptom</th><th>Real cause</th></tr></thead><tbody><tr><td>Order-dependent failures</td><td>Shared mutable fixtures</td></tr></tbody></table>",
            "is_formidable": False,
        },
        "flaky-test-triage": {
            "slug": "flaky-test-triage", "name": "flaky-test-triage",
            "category_slug": "testing", "category_title": "Testing",
            "description": "Use when a test passes on rerun, fails only in CI, fails only when the whole suite runs.",
            "summary": "The cause taxonomy; quarantine with an expiry.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Treat flakiness as a defect report, not a nuisance.</p>",
            "is_formidable": False,
        },
    },
}

PREVIEW_DIR = Path(__file__).resolve().parent.parent / "_preview"

_checks_run = 0


def check(label: str, condition: bool) -> None:
    global _checks_run
    _checks_run += 1
    if not condition:
        raise AssertionError(f"FAILED: {label}")
    print(f"  ok  {label}")


def write_preview(filename: str, html: str) -> Path:
    PREVIEW_DIR.mkdir(parents=True, exist_ok=True)
    path = PREVIEW_DIR / filename
    path.write_text(html, encoding="utf-8")
    return path


def check_escaping() -> None:
    """The FIXTURE above deliberately contains no HTML-special characters in
    any plain-text field, so it cannot catch a regression where an escape_html
    call is removed — every substring check in main() would pass identically
    either way. This is a separate, dedicated fixture built specifically to
    prove escaping actually happens: it asserts both that the dangerous raw
    form is absent and that the escaped form is present, and that body_html's
    verbatim-injection contract (pre-rendered HTML is NOT re-escaped) still
    holds at the same time."""
    print("escaping regression check")
    categories = [{"slug": "cat", "title": "Cat", "skill_slugs": ["x"]}]
    skill = {
        "slug": "x",
        "name": 'A & B <em>"quoted"</em>',
        "category_slug": "cat",
        "category_title": "Cat",
        "description": "Uses <script>alert(1)</script> & \"quotes\" & 'apostrophes'.",
        "summary": "Contains <b>bold-looking</b> text & ampersands.",
        "body_html": "<p>pre-rendered, injected verbatim on purpose</p>",
        "is_formidable": False,
    }
    skills = {"x": skill}

    index_html = render_index(categories, skills)
    check("index: raw <em> from a name field never appears unescaped", "<em>" not in index_html)
    check("index: escaped name form is present", "&lt;em&gt;" in index_html)
    check("index: & in a summary field is escaped", "bold-looking&lt;/b&gt; text &amp; ampersands" in index_html)

    page_html = render_skill_page(skill, None, None, [], categories)
    check("page: raw <script> from a description field never appears unescaped", "<script>alert(1)</script>" not in page_html)
    check("page: escaped description form is present", "&lt;script&gt;alert(1)&lt;/script&gt;" in page_html)
    check("page: body_html's own tags are NOT double-escaped (verbatim-injection contract)", "<p>pre-rendered, injected verbatim on purpose</p>" in page_html)
    print("  escaping check passed")


def check_base_path() -> None:
    """GitHub Pages serves a project site from a subpath
    (https://<user>.github.io/<repo>/), not the domain root, so every
    root-relative href/src must be prefixed. Confirms both that a non-default
    base_path is honored everywhere and that the default ("") reproduces the
    exact previous root-relative behavior other checks above rely on."""
    print("base_path check")
    categories = FIXTURE["categories"]
    skills = FIXTURE["skills"]
    base = "/tbaguette-skills"

    index_html = render_index(categories, skills, base_path=base)
    check("prefixed stylesheet href", f'"{base}/assets/styles.css"' in index_html)
    check("prefixed script src", f'"{base}/assets/site.js"' in index_html)
    check("prefixed skill card link", f'href="{base}/skills/formidable/"' in index_html)
    check("un-prefixed root-relative form is absent once base_path is set", '"/assets/styles.css"' not in index_html)

    formidable = skills["formidable"]
    page_html = render_skill_page(formidable, None, None, [], categories, base_path=base)
    check("prefixed breadcrumb home link", f'href="{base}/"' in page_html)
    check("prefixed icon sprite reference", f'{base}/assets/icons.svg#' in page_html)
    print("  base_path check passed")


def check_verify_install_page() -> None:
    """render_verify_install_page against the real, highlighted source of
    test_install_command.py — not a synthetic snippet, since the whole point
    of this page is displaying that exact file. Import is deferred to here
    (rather than the top of the file, next to the other imports) only
    because python_highlight is this module's own sibling under active
    development in the same change; the import itself is otherwise ordinary."""
    print("verify-install page check")
    from python_highlight import highlight_source

    real_source = (Path(__file__).resolve().parent / "test_install_command.py").read_text(encoding="utf-8")
    lines = highlight_source(real_source)
    categories = FIXTURE["categories"]

    html = render_verify_install_page(lines, categories)
    check("looks like a document", "<html" in html)
    check("title names what the page proves", "only touches one folder" in html)
    check("links out to the real file on GitHub as provenance",
          f'href="{INSTALL_TEST_GITHUB_URL}"' in html)
    check("renders one list item per source line",
          html.count('<li><span class="code-block__line-number">')
          == len(real_source.splitlines()))
    check("at least one comment span made it through from the real file",
          'class="tok-comment"' in html)
    check("at least one string span made it through from the real file",
          'class="tok-string"' in html)

    base = "/tbaguette-skills"
    prefixed = render_verify_install_page(lines, categories, base_path=base)
    check("code block content is base_path-independent (no hrefs inside code lines)",
          prefixed.count('class="code-block__line-code"')
          == html.count('class="code-block__line-code"'))
    check("but the page chrome around it is still prefixed like every other page",
          f'"{base}/assets/styles.css"' in prefixed)


def main() -> None:
    categories = FIXTURE["categories"]
    skills = FIXTURE["skills"]

    # --- render_index -----------------------------------------------------
    print("render_index")
    index_html = render_index(categories, skills)
    index_path = write_preview("index.html", index_html)
    check("non-empty", len(index_html) > 0)
    check("looks like a document", "<html" in index_html)
    check("references the shared stylesheet root-relatively", '"/assets/styles.css"' in index_html)
    check("references the shared script root-relatively", '"/assets/site.js"' in index_html)
    for cat in categories:
        check(f"contains category title {cat['title']!r}", cat["title"] in index_html)
    for slug, skill in skills.items():
        check(f"contains skill name {slug!r}", skill["name"] in index_html)
        check(f"contains skill summary for {slug!r}", skill["summary"] in index_html)
        check(f"links to /skills/{slug}/", f'href="/skills/{slug}/"' in index_html)
    check("has a search input", 'data-search-input' in index_html)
    # The command contains && , which escape_html correctly turns into
    # &amp;&amp; — checking for the raw form here would either fail (proving
    # nothing) or, worse, pass by accident if escaping were ever silently
    # disabled. Checking the escaped form catches that regression directly.
    check("install command appears, correctly HTML-escaped",
          escape_html(INSTALL_COMMAND) in index_html)
    check("raw, un-escaped command never appears (would mean escaping broke)",
          INSTALL_COMMAND not in index_html)
    check("install command sits right after the headline, before the lede",
          index_html.index("hero__headline") < index_html.index('id="install-command"')
          < index_html.index("hero__lede"))
    check("has a copy button wired to the install command", 'data-copy-target="install-command"' in index_html)
    check("install command is wrapped in its labeled frame",
          index_html.index("install-frame") < index_html.index("Install TBaguette")
          < index_html.index('id="install-command"'))
    label_start = index_html.index('install-frame__label')
    label_end = index_html.index('</p>', label_start)
    check("frame label itself carries an icon (icon-crust also appears in category "
          "headers elsewhere on the page, so this checks the label's own slice, not "
          "just presence anywhere)",
          '#icon-crust' in index_html[label_start:label_end])
    check("verification note sits after the command and before the lede, inside the frame",
          index_html.index('id="install-command"') < index_html.index("install-frame__note")
          < index_html.index("hero__lede"))
    check("verification note links to the on-site explanation page, base_path-prefixed",
          'href="/verify-install/"' in index_html)
    print(f"  wrote {index_path}")

    # --- render_skill_page: formidable (the interesting one) --------------
    # No prev/next, no siblings: formidable is alone in its category in this
    # fixture, exactly like the real content it stands in for. This is a
    # real empty state, not a hypothetical one, so it's worth its own file.
    print("render_skill_page (formidable)")
    formidable = skills["formidable"]
    formidable_html = render_skill_page(formidable, None, None, [], categories)
    skill_path = write_preview("skill.html", formidable_html)
    check("non-empty", len(formidable_html) > 0)
    check("looks like a document", "<html" in formidable_html)
    check("contains the fixture skill name", "formidable" in formidable_html)
    check("contains the full description (trigger text)", formidable["description"] in formidable_html)
    check("injects body_html verbatim (h2#overview)", 'id="overview"' in formidable_html)
    check("injects body_html verbatim (strong)", "<strong>out-of-distribution</strong>" in formidable_html)
    for item in formidable["formidable_stacks"] + formidable["formidable_commands"]:
        check(f"contains tab panel id {item['id']!r}", f'id="{item["id"]}"' in formidable_html)
        check(f"contains tab title {item['title']!r}", item["title"] in formidable_html)
    check("first stack tab starts selected", 'aria-controls="stack-web" aria-selected="true"' in formidable_html)
    check("second stack tab starts unselected", 'aria-controls="stack-terminal-tui" aria-selected="false"' in formidable_html)
    check("no prev/next nav when both are None", 'class="container prevnext"' not in formidable_html)
    check("no see-also section with zero siblings", 'class="container see-also"' not in formidable_html)
    check("craft floor section id exists (the anchor formidable's own links target)", 'id="cmd-craft-floor"' in formidable_html)
    check("craft floor heading present", "Craft floor" in formidable_html)
    check("craft floor content injected verbatim", "Load immediately before editing UI" in formidable_html)
    print(f"  wrote {skill_path}")

    # --- render_skill_page: the two-skill "testing" category --------------
    # Extra coverage beyond the required two files: exercises a table inside
    # body_html, and both directions of prev/next (one skill has next-only,
    # the other has prev-only) plus a populated see-also list.
    print("render_skill_page (testing category, both directions)")
    dtd = skills["designing-test-data"]
    ftt = skills["flaky-test-triage"]
    testing_siblings = [dtd, ftt]

    dtd_html = render_skill_page(dtd, None, ftt, testing_siblings, categories)
    dtd_path = write_preview("skill-testing-1.html", dtd_html)
    check("table from body_html is present", "<table>" in dtd_html)
    check("next-only: has a next link", 'prevnext__link--next' in dtd_html)
    check("next-only: has no prev label", ">Previous<" not in dtd_html)
    check("see-also excludes self, includes sibling", "flaky-test-triage" in dtd_html)
    print(f"  wrote {dtd_path}")

    ftt_html = render_skill_page(ftt, dtd, None, testing_siblings, categories)
    ftt_path = write_preview("skill-testing-2.html", ftt_html)
    check("prev-only: has a previous label", ">Previous<" in ftt_html)
    check("prev-only: has no next-modifier link", 'prevnext__link--next' not in ftt_html)
    check("see-also excludes self, includes sibling", "designing-test-data" in ftt_html)
    print(f"  wrote {ftt_path}")

    check_escaping()
    check_base_path()
    check_verify_install_page()

    print(f"\n{_checks_run} checks passed.")
    print(f"Preview files written to {PREVIEW_DIR}")
    print("Serve from the project root (e.g. `python3 -m http.server 8000`) and open:")
    print("  /_preview/index.html")
    print("  /_preview/skill.html")
    print("  /_preview/skill-testing-1.html")
    print("  /_preview/skill-testing-2.html")


if __name__ == "__main__":
    main()
