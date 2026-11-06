//! The document shell every page shares: `<head>` with the inline theme
//! bootstrap, the header, the footer and the WASM module that enhances the
//! page (templates.py `_render_document` and the chrome it calls; the module
//! replaced site.js and graph.js).

use super::{
    BRAND_ATELIER, CLIMB_NEW_UNTIL, GETTING_STARTED_PATH, GRAPH_NEW_UNTIL, GRAPH_PATH, MODS_PATH,
    PageCtx, SKILLS_ANCHOR, escape_html, join, strings,
};

/// The repository, for the footer's source links.
const REPOSITORY_URL: &str = "https://github.com/LeSplooch/tbaguette-skills";

/// The localStorage key the theme toggle writes.
const THEME_STORAGE_KEY: &str = "tbaguette-theme";

/// What one page puts into the shell.
#[derive(Debug, Clone, Copy)]
pub struct Page<'a> {
    /// `<title>` and `og:title`, plain text.
    pub title: &'a str,
    /// `<meta name="description">` and `og:description`, plain text.
    pub meta_description: &'a str,
    /// The `<body>` class, e.g. `page-skill`.
    pub body_class: &'a str,
    /// The page's path below the base path (`""`, `skills/<slug>/`,
    /// `getting-started/`): the canonical URL, and which header link is
    /// `aria-current`.
    pub path_suffix: &'a str,
    /// Everything inside `<main>`, as HTML.
    pub main_html: &'a str,
}

/// The whole HTML document for `page`, without the generated-file header
/// comment (the writer adds that).
pub fn document(ctx: &PageCtx, page: &Page) -> String {
    format!(
        r##"<!doctype html>
<html lang="en" dir="ltr">
<head>
{head}
</head>
<body class="{body_class}"
      data-i18n-copied="{copied}"
      data-i18n-no-match="{no_match}"
      data-i18n-modal-title="{modal_title}"
      data-i18n-modal-body="{modal_body}"
      data-i18n-modal-reload="{modal_reload}"
      data-i18n-quote-open="{quote_open}"
      data-i18n-quote-close="{quote_close}">
<a class="skip-link" href="#main">{skip_link}</a>
{header}
<main id="main">
{main_html}
</main>
{footer}
{modules}
</body>
</html>
"##,
        head = head(ctx, page),
        body_class = page.body_class,
        copied = escape_html(strings::INSTALL_COPY_COPIED),
        no_match = escape_html(strings::SEARCH_STATUS_NO_MATCH),
        modal_title = escape_html(strings::UPDATE_MODAL_TITLE),
        modal_body = escape_html(strings::UPDATE_MODAL_BODY),
        modal_reload = escape_html(strings::UPDATE_MODAL_RELOAD_BUTTON),
        quote_open = escape_html(strings::SEARCH_EMPTY_QUERY_OPEN),
        quote_close = escape_html(strings::SEARCH_EMPTY_QUERY_CLOSE),
        skip_link = escape_html(strings::SKIP_LINK),
        header = header(ctx, page.path_suffix),
        main_html = page.main_html,
        footer = footer(ctx),
        modules = modules(ctx, page.path_suffix),
    )
}

/// The WASM front end: atelier_web on every page, and on /graph/ the graph
/// app's own module beside it, so no other page downloads the graph.
///
/// Each is a module script that imports the wasm-bindgen glue and hands it
/// the `.wasm` URL explicitly, both fingerprinted: the glue would otherwise
/// fetch its module by a URL derived from its own, and the `?v=` on the glue
/// does not reach it, so a cached old module could meet new glue. Module
/// scripts are deferred by default.
///
/// A browser with no WebAssembly, or a blocked request, marks the page
/// `html[data-nowasm]` and says so once, quietly. The stylesheets then do what
/// they do with JavaScript off: the page reads and navigates, and no control
/// that needs the module stays on it.
fn modules(ctx: &PageCtx, path_suffix: &str) -> String {
    let mut names = vec!["atelier_web"];
    if path_suffix == GRAPH_PATH {
        names.push("atelier_graph");
    }
    names
        .iter()
        .map(|name| module_script(ctx, name))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One module's loader. A dynamic import, so the catch hears about the glue
/// failing to load as well as the module.
pub fn module_script(ctx: &PageCtx, name: &str) -> String {
    format!(
        "<script type=\"module\">import('{glue}').then(function(m){{return m.default({{module_or_path:'{wasm}'}});}}).catch(function(){{var r=document.documentElement;if(!r.hasAttribute('data-nowasm')){{r.setAttribute('data-nowasm','');console.info('{NO_WASM_NOTE}');}}}});</script>",
        glue = ctx.asset_url(&format!("{name}.js")),
        wasm = ctx.asset_url(&format!("{name}_bg.wasm")),
    )
}

/// The one console line a failed load leaves. No apostrophes: it sits in a
/// single-quoted string in the page.
const NO_WASM_NOTE: &str = "TBaguette: WebAssembly did not load here, so the page stays static.";

/// The inline script that applies a stored theme before first paint (so a
/// reload never flashes the wrong one). Dark is the default; only a reader
/// who chose flour gets it.
fn theme_bootstrap_js() -> String {
    format!(
        "(function(){{try{{if(localStorage.getItem('{THEME_STORAGE_KEY}')==='flour')\
         {{document.documentElement.setAttribute('data-theme','flour');}}}}catch(e){{}}}})();"
    )
}

/// The stylesheet a page loads after styles.css, by its path: the shared core
/// is on every page, and the landing page, /graph/ and the mods' pages each
/// add their own, so no page downloads another's rules. Skill pages,
/// /getting-started/ and /verify-install/ need only the core.
pub fn page_stylesheet(path_suffix: &str) -> Option<&'static str> {
    if path_suffix.is_empty() {
        Some("landing.css")
    } else if path_suffix == GRAPH_PATH {
        Some("graph.css")
    } else if path_suffix.starts_with(MODS_PATH) {
        Some("mods.css")
    } else {
        None
    }
}

/// templates.py `_render_head`. The canonical URL is root-relative.
fn head(ctx: &PageCtx, page: &Page) -> String {
    let base = &ctx.base_path;
    let title = escape_html(page.title);
    let desc = escape_html(page.meta_description);
    format!(
        r#"<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="{desc}">
<meta property="og:type" content="website">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{desc}">
<link rel="canonical" href="{canonical}">
<link rel="icon" type="image/svg+xml" href="{base}/assets/favicon.svg">
<link rel="preload" href="{base}/assets/fonts/fraunces-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="{base}/assets/fonts/work-sans-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{styles}">{page_styles}
<script>{bootstrap}</script>"#,
        canonical = ctx.url(page.path_suffix),
        styles = ctx.asset_url("styles.css"),
        page_styles = match page_stylesheet(page.path_suffix) {
            Some(name) => format!(
                "\n<link rel=\"stylesheet\" href=\"{}\">",
                ctx.asset_url(name)
            ),
            None => String::new(),
        },
        bootstrap = theme_bootstrap_js(),
    )
}

/// The header's "Updated" time (templates.py `_render_updated_time`): plain
/// UTC so it reads true without JS; site.js rewrites it in the reader's
/// time, and polls `data-version-url` for a newer build.
fn updated_time(ctx: &PageCtx) -> String {
    let last_updated = ctx.last_updated_utc();
    let fallback = format!("{} UTC", escape_html(&last_updated.replace("+00:00", "Z")));
    format!(
        r#"<p class="site-header__updated">
      <span class="site-header__updated-label">{label}</span>
      <time class="site-header__updated-value" datetime="{datetime}" data-format-updated data-version-url="{base}/version.txt" data-i18n-updated-template="{template}">{fallback}</time>
    </p>"#,
        label = escape_html(strings::HEADER_UPDATED_LABEL),
        datetime = escape_html(&last_updated),
        base = ctx.base_path,
        template = escape_html(strings::HEADER_UPDATED_VALUE_TEMPLATE),
    )
}

/// The version beside the wordmark: `dir="ltr"` so no locale reorders it
/// around its dots, and outside the wordmark link (templates.py
/// `_render_plugin_version`).
fn plugin_version(version: &str) -> String {
    format!(
        "<span class=\"wordmark-version\" dir=\"ltr\">v{}</span>",
        escape_html(version)
    )
}

/// The Graph link's mark, inline so its ties and nodes can move (icon-graph's
/// geometry). The node coordinates print as Python printed them: `6`, `5.5`.
const GRAPH_MARK_TIES: [&str; 5] = [
    "M8.2 6.6 L15.8 5.9",
    "M6.1 9.25 L6.4 15.25",
    "M8.75 17.6 L14.75 17.9",
    "M17.8 7.75 L17.2 15.75",
    "M7.7 8.8 L15.4 16.3",
];
const GRAPH_MARK_NODES: [(&str, &str); 4] =
    [("6", "7"), ("18", "5.5"), ("17", "18"), ("6.5", "17.5")];

fn graph_mark() -> String {
    let mut ties = String::new();
    for (i, d) in GRAPH_MARK_TIES.iter().enumerate() {
        ties.push_str(&format!(
            "<path class=\"graph-mark__tie\" d=\"{d}\"/>\
             <path class=\"graph-mark__flow\" d=\"{d}\" pathLength=\"10\" style=\"--i:{i}\"/>"
        ));
    }
    let mut nodes = String::new();
    for (i, (x, y)) in GRAPH_MARK_NODES.iter().enumerate() {
        nodes.push_str(&format!(
            "<circle class=\"graph-mark__node\" cx=\"{x}\" cy=\"{y}\" r=\"2.25\" style=\"--i:{i}\"/>"
        ));
    }
    format!(
        "<svg class=\"icon site-header__nav-icon graph-mark\" viewBox=\"0 0 24 24\" aria-hidden=\"true\">{ties}{nodes}</svg>"
    )
}

/// The value of `aria-current` for a header link, or nothing: `page` on the
/// page itself, `true` on a page inside its section.
fn current(on_page: bool, in_section: bool) -> &'static str {
    if on_page {
        " aria-current=\"page\""
    } else if in_section {
        " aria-current=\"true\""
    } else {
        ""
    }
}

/// A header link's "New" tag text. The link's accessible name is its text
/// run together, so a visually hidden separator keeps it from reading
/// "ModsNew": it reads "Mods, New". The separator lives inside the tag, so
/// the news check that removes the tag takes it along.
fn new_tag_text() -> String {
    format!(
        "<span class=\"visually-hidden\">, </span>{}",
        escape_html(strings::GRAPH_NEW_BADGE)
    )
}

/// The header's Mods link. The pill is an inner face, as on Graph, so the
/// link box never moves: atelier-web's mods_button walks the four letters up
/// a flight of stairs (each letter its own box, so assistive tech reads the
/// word from a visually hidden copy rather than spelling it out) as a mouse or pen comes near, and at contact the "s"
/// hops onto the dashed bar above the pill. Keyboard focus shows the climbed
/// letters still (CSS). While the climb release is new it carries a "New"
/// tag, which the news check retires against the reader's clock.
fn mods_link(ctx: &PageCtx, path_suffix: &str) -> String {
    let is_new = ctx.climb_is_new();
    let letters: String = strings::NAV_MODS
        .chars()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "<span class=\"mods-climb__letter\" style=\"--i:{i}\">{}</span>",
                escape_html(&c.to_string())
            )
        })
        .collect();
    let sparks: String = (0..5)
        .map(|k| format!("<span class=\"mods-climb__spark\" style=\"--k:{k}\"></span>"))
        .collect();
    let new_tag = if is_new {
        format!(
            "<span class=\"site-header__new\" data-climb-new-until=\"{CLIMB_NEW_UNTIL}\">{}</span>",
            new_tag_text()
        )
    } else {
        String::new()
    };
    format!(
        "<a class=\"site-header__nav-link site-header__nav-link--mods{new_class}\" href=\"{href}\"{current} data-mods-climb><span class=\"site-header__mods-face\"><span class=\"mods-climb__bar\" aria-hidden=\"true\">{sparks}</span><span class=\"visually-hidden\">{word}</span><span class=\"mods-climb__word\" aria-hidden=\"true\">{letters}</span>{new_tag}</span></a>",
        word = escape_html(strings::NAV_MODS),
        new_class = if is_new {
            " site-header__nav-link--new"
        } else {
            ""
        },
        href = ctx.url(MODS_PATH),
        current = current(path_suffix == MODS_PATH, path_suffix.starts_with(MODS_PATH)),
    )
}

/// The header nav: Skills, Mods, Graph, Getting started (templates.py
/// `_render_header_nav`, grown from its two links). The page the reader is on
/// is `aria-current`. The Graph pill is an inner face so it can swell toward
/// the pointer while the link keeps its hit area; while the graph is new it
/// carries a "New" tag.
fn header_nav(ctx: &PageCtx, path_suffix: &str) -> String {
    let graph_new = ctx.graph_is_new();
    let new_tag = if graph_new {
        format!(
            "<span class=\"site-header__new\" data-graph-new-until=\"{GRAPH_NEW_UNTIL}\">{}</span>",
            new_tag_text()
        )
    } else {
        String::new()
    };
    let graph_class = if graph_new {
        "site-header__nav-link site-header__nav-link--graph site-header__nav-link--new"
    } else {
        "site-header__nav-link site-header__nav-link--graph"
    };
    format!(
        r#"<nav class="site-header__nav" aria-label="{aria}">
        <a class="site-header__nav-link" href="{home}{SKILLS_ANCHOR}"{skills_current}>{skills}</a>
        {mods}
        <a class="{graph_class}" href="{graph_href}"{graph_current} data-graph-magnet><span class="site-header__graph-face">{mark}<span>{graph}</span>{new_tag}</span></a>
        <a class="site-header__nav-link" href="{href}"{current}>{getting_started}</a>
      </nav>"#,
        aria = escape_html(strings::NAV_ARIA_LABEL),
        home = ctx.url(""),
        skills_current = current(false, path_suffix.starts_with("skills/")),
        skills = escape_html(strings::NAV_SKILLS),
        mods = mods_link(ctx, path_suffix),
        href = ctx.url(GETTING_STARTED_PATH),
        current = current(path_suffix == GETTING_STARTED_PATH, false),
        getting_started = escape_html(strings::NAV_GETTING_STARTED),
        graph_href = ctx.url(GRAPH_PATH),
        graph_current = current(path_suffix == GRAPH_PATH, false),
        mark = graph_mark(),
        graph = escape_html(strings::NAV_GRAPH),
    )
}

/// templates.py `_render_header`.
fn header(ctx: &PageCtx, path_suffix: &str) -> String {
    let version_html = if ctx.plugin_version.is_empty() {
        String::new()
    } else {
        plugin_version(&ctx.plugin_version)
    };
    format!(
        r#"<header class="site-header">
  <div class="site-header__band" aria-hidden="true"></div>
  <div class="container site-header__inner">
    <div class="site-header__brand">
      <a class="wordmark" href="{home}">
        {wheat}
        <span class="wordmark__text">TBaguette<span class="wordmark__suffix">&rsquo;s Atelier</span></span>
      </a>
      {version_html}
    </div>
    <div class="site-header__actions">
      {nav_html}
      {updated_html}
      <button class="theme-toggle" type="button" data-theme-toggle
              aria-label="{to_light}"
              data-i18n-theme-light="{to_light}"
              data-i18n-theme-dark="{to_dark}">
        {sun}
        {moon}
      </button>
    </div>
  </div>
</header>"#,
        home = ctx.url(""),
        wheat = ctx.icon("icon-wheat", "icon wordmark__icon"),
        nav_html = header_nav(ctx, path_suffix),
        updated_html = updated_time(ctx),
        to_light = escape_html(strings::THEME_TOGGLE_SWITCH_TO_LIGHT),
        to_dark = escape_html(strings::THEME_TOGGLE_SWITCH_TO_DARK),
        sun = ctx.icon(
            "icon-sun",
            "icon theme-toggle__icon theme-toggle__icon--sun"
        ),
        moon = ctx.icon(
            "icon-moon",
            "icon theme-toggle__icon theme-toggle__icon--moon"
        ),
    )
}

/// The footer: the brand line with its pointer to Getting started (one grid
/// cell, so the pointer stays under the brand at every width), then the
/// families, the rest of the atelier, and where its source and licence live.
fn footer(ctx: &PageCtx) -> String {
    let home = ctx.url("");
    let families: Vec<String> = ctx
        .categories
        .iter()
        .map(|c| {
            format!(
                "<li><a href=\"{home}#{}\">{}</a></li>",
                escape_html(c.slug),
                escape_html(c.title)
            )
        })
        .collect();
    let atelier = [
        (format!("{home}{SKILLS_ANCHOR}"), strings::NAV_SKILLS),
        (ctx.url(MODS_PATH), strings::NAV_MODS),
        (ctx.url(GRAPH_PATH), strings::NAV_GRAPH),
        (ctx.url(GETTING_STARTED_PATH), strings::NAV_GETTING_STARTED),
        (ctx.url("verify-install/"), strings::FOOTER_VERIFY_INSTALL),
    ];
    let source = [
        (REPOSITORY_URL.to_string(), strings::FOOTER_REPOSITORY),
        (
            format!("{REPOSITORY_URL}/blob/master/UPDATES.md"),
            strings::FOOTER_UPDATES,
        ),
        (
            format!("{REPOSITORY_URL}/blob/master/LICENSE"),
            strings::FOOTER_LICENCE,
        ),
    ];
    let list = |links: &[(String, &str)]| -> String {
        links
            .iter()
            .map(|(href, text)| {
                format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    escape_html(href),
                    escape_html(text)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let brand_html = strings::FOOTER_BRAND_HTML_TEMPLATE.replace("{brand_atelier}", BRAND_ATELIER);
    let start_here_html =
        strings::FOOTER_START_HERE_HTML_TEMPLATE.replace("{url}", &ctx.url(GETTING_STARTED_PATH));
    format!(
        r#"<footer class="site-footer">
  <div class="container site-footer__inner">
    <div class="site-footer__brand-col">
      <p class="site-footer__brand">{brand_html}</p>
      <p class="site-footer__start">{start_here_html}</p>
    </div>
    <nav class="site-footer__families" aria-labelledby="footer-families">
      <p class="site-footer__nav-title" id="footer-families">{families_label}</p>
      <ul class="site-footer__categories">
{families}
      </ul>
    </nav>
    <nav class="site-footer__more" aria-labelledby="footer-atelier">
      <p class="site-footer__nav-title" id="footer-atelier">{atelier_label}</p>
      <ul class="site-footer__links">
{atelier}
      </ul>
      <p class="site-footer__nav-title" id="footer-source">{source_label}</p>
      <ul class="site-footer__links" aria-labelledby="footer-source">
{source}
      </ul>
    </nav>
  </div>
</footer>"#,
        families_label = escape_html(strings::FOOTER_CATEGORIES_LABEL),
        families = join(&families),
        atelier_label = escape_html(strings::FOOTER_ATELIER_LABEL),
        atelier = list(&atelier),
        source_label = escape_html(strings::FOOTER_SOURCE_LABEL),
        source = list(&source),
    )
}
