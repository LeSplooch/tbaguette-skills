//! The site's pages, ported from `scripts/templates.py`.
//!
//! Templates are plain string building (`format!` and `push_str`) laid out
//! like the Python f-strings they replace, so the whitespace, attribute order
//! and entity forms of every page stay byte-identical to the Python build's
//! output. Values interpolated into markup go through [`escape_html`]
//! (Python's `html.escape(quote=True)`); pre-rendered HTML from the content
//! pipeline is inserted verbatim.
//!
//! The site has one locale, English at the root. The Python build's locale
//! plumbing (URL prefixes, RTL arrow swaps, translation fallbacks) had no
//! second locale to serve and is not ported.

pub mod index;
pub mod mods;
pub mod other;
pub mod shell;
pub mod skill;
pub mod unfold;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{Result, bail};
use atelier_core::content::{self, RenderedSkill};
use atelier_core::graph::ChangeStamp;
use atelier_core::model::{Category, Skill};
use chrono::{DateTime, Utc};

pub use atelier_core::escape::html_attr as escape_html;

/// The UI strings the ported pages use (templates.py `ENGLISH_STRINGS`).
pub mod strings {
    pub const SKIP_LINK: &str = "Skip to content";
    pub const HEADER_UPDATED_LABEL: &str = "Updated";
    pub const HEADER_UPDATED_VALUE_TEMPLATE: &str = "{local} your time · {utc} UTC";
    pub const THEME_TOGGLE_SWITCH_TO_DARK: &str = "Switch to dark theme";
    pub const THEME_TOGGLE_SWITCH_TO_LIGHT: &str = "Switch to light theme";
    pub const NAV_ARIA_LABEL: &str = "Site";
    pub const NAV_GETTING_STARTED: &str = "Getting started";
    pub const NAV_GRAPH: &str = "Graph";
    pub const NAV_SKILLS: &str = "Skills";
    pub const NAV_MODS: &str = "Mods";
    pub const GRAPH_NEW_BADGE: &str = "New";
    pub const SEARCH_STATUS_NO_MATCH: &str = "No skills match.";
    pub const SEARCH_EMPTY_QUERY_OPEN: &str = "“";
    pub const SEARCH_EMPTY_QUERY_CLOSE: &str = "”";
    pub const INSTALL_COPY_COPIED: &str = "Copied!";
    /// `{brand_atelier}` is [`super::BRAND_ATELIER`]; the result is HTML.
    pub const FOOTER_BRAND_HTML_TEMPLATE: &str = "<strong>{brand_atelier}</strong> — the judgment calls that sit between the ticket and the commit, organized like a proper bench.";
    /// `{url}` is the getting-started page; the result is HTML.
    pub const FOOTER_START_HERE_HTML_TEMPLATE: &str =
        "New here? Start with <a href=\"{url}\">Getting started</a>.";
    pub const FOOTER_CATEGORIES_LABEL: &str = "Families";
    pub const FOOTER_ATELIER_LABEL: &str = "The atelier";
    pub const FOOTER_VERIFY_INSTALL: &str = "What the install touches";
    pub const FOOTER_SOURCE_LABEL: &str = "Source";
    pub const FOOTER_REPOSITORY: &str = "Repository";
    pub const FOOTER_UPDATES: &str = "Every update note";
    pub const FOOTER_LICENCE: &str = "Licence";
    pub const BREADCRUMB_HOME: &str = "Home";
    pub const BREADCRUMB_ARIA_LABEL: &str = "Breadcrumb";
    pub const PREVNEXT_PREVIOUS: &str = "Previous";
    pub const PREVNEXT_NEXT: &str = "Next";
    pub const PREVNEXT_ARIA_LABEL: &str = "Skill navigation";
    /// `{category}` is the category's title.
    pub const SEE_ALSO_TITLE_TEMPLATE: &str = "More in {category}";
    pub const FORMIDABLE_STACKS_HEADING: &str = "Stacks";
    pub const FORMIDABLE_COMMANDS_HEADING: &str = "Commands";
    pub const FORMIDABLE_CRAFT_FLOOR_HEADING: &str = "Craft floor";
    pub const REFERENCE_HEADING: &str = "Reference";
    pub const SKILL_GRAPH_LINK: &str = "See its place in the graph";
    pub const SKILL_CITED_BY: &str = "Cited by";
    pub const SKILL_CITES: &str = "Cites";
    pub const SKILL_TRIGGER_MORE: &str = "Show the full trigger";
    pub const SKILL_TRIGGER_LESS: &str = "Show less";
    pub const SKILL_RAIL_LABEL: &str = "About this skill";
    pub const SKILL_RAIL_INVOKE: &str = "Invoke as";
    pub const SKILL_RAIL_FAMILY: &str = "Family";
    pub const SKILL_RAIL_CITED: &str = "Cited by";
    pub const SKILL_RAIL_CITED_ONE: &str = "1 other skill";
    /// `{count}` other skills.
    pub const SKILL_RAIL_CITED_MANY_TEMPLATE: &str = "{count} other skills";
    pub const SKILL_RAIL_CITED_NONE: &str = "no other skill yet";
    pub const PAGE_RAIL_CONTENTS: &str = "On this page";
    pub const CHANGE_BADGE_NEW: &str = "New";
    pub const CHANGE_BADGE_UPDATED: &str = "Updated";
    pub const UPDATE_MODAL_TITLE: &str = "New version available";
    pub const UPDATE_MODAL_BODY: &str = "This page has been updated. Reload to see the latest.";
    pub const UPDATE_MODAL_RELOAD_BUTTON: &str = "Reload";
}

/// The brand as HTML (templates.py `BRAND_ATELIER`).
pub const BRAND_ATELIER: &str = "TBaguette&rsquo;s Atelier";
/// The brand as plain text, for `<title>` (`BRAND_ATELIER_TEXT`).
pub const BRAND_ATELIER_TEXT: &str = "TBaguette’s Atelier";

/// Page paths below the base path (`GETTING_STARTED_PATH`, `GRAPH_PATH`).
pub const GETTING_STARTED_PATH: &str = "getting-started/";
pub const GRAPH_PATH: &str = "graph/";
pub const MODS_PATH: &str = "mods/";
/// The landing page's skills section, which the header's Skills link opens.
pub const SKILLS_ANCHOR: &str = "#skills";

/// The climb release's window, inclusive: the header's "New" tag on Mods and the
/// one on the landing page's Mods counter both carry it, and atelier-web's news
/// check retires both against the reader's clock.
pub const CLIMB_NEW_UNTIL: &str = "2026-11-25";

/// The header's Graph link carries a "New" tag until this date, inclusive
/// (`GRAPH_NEW_UNTIL`); site.js retires it against the reader's clock.
pub const GRAPH_NEW_UNTIL: &str = "2026-10-08";

/// One skill as the pages see it: the skill, where it sits, and its rendered
/// content (the Python build's content-dict entry).
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub skill: Skill,
    pub category_slug: &'static str,
    pub category_title: &'static str,
    pub rendered: RenderedSkill,
}

/// Everything a page render reads, built once per build.
#[derive(Debug, Clone)]
pub struct PageCtx {
    /// Prefixed verbatim to every root-relative URL (`/tbaguette-skills`, or
    /// empty at a domain root).
    pub base_path: String,
    /// The build instant: the header's "Updated" time, version.txt, the
    /// freshness cutoff and the announcement windows.
    pub now: DateTime<Utc>,
    /// Shown beside the wordmark; empty leaves it out.
    pub plugin_version: String,
    /// `?v=` fingerprints by asset file name (`styles.css` -> `a9d8126d81`);
    /// an asset missing here is linked without one.
    pub asset_versions: BTreeMap<String, String>,
    /// Display order, prev/next order and see-also order.
    pub categories: &'static [Category],
    /// Every skill by slug.
    pub skills: BTreeMap<String, SkillEntry>,
    /// The skills inside the freshness window (`repo::fresh_skills`).
    pub fresh: BTreeMap<String, ChangeStamp>,
}

impl PageCtx {
    /// Renders every skill's content and places it in its category.
    ///
    /// Fails, naming the slug, when a skill directory is in no category or
    /// a category lists a skill that does not exist (content_pipeline's
    /// catalog drift errors). A skill listed twice belongs to the last
    /// category listing it, as in the Python.
    pub fn new(
        skills: BTreeMap<String, Skill>,
        categories: &'static [Category],
        base_path: &str,
        now: DateTime<Utc>,
        plugin_version: String,
        asset_versions: BTreeMap<String, String>,
        fresh: BTreeMap<String, ChangeStamp>,
    ) -> Result<PageCtx> {
        let category_of: HashMap<&str, &'static Category> = categories
            .iter()
            .flat_map(|c| c.skill_slugs.iter().map(move |s| (*s, c)))
            .collect();
        for category in categories {
            for slug in category.skill_slugs {
                if !skills.contains_key(*slug) {
                    bail!(
                        "category \"{}\" lists skill_slug \"{slug}\" but no such skill directory (with a SKILL.md) exists",
                        category.slug
                    );
                }
            }
        }
        let known: BTreeSet<String> = skills.keys().cloned().collect();
        let mut entries = BTreeMap::new();
        for (slug, skill) in skills {
            let Some(category) = category_of.get(slug.as_str()) else {
                bail!(
                    "skill directory \"{slug}\" is not listed in any category (atelier_core::model)"
                );
            };
            let rendered = content::render_skill(&skill, &known, base_path);
            entries.insert(
                slug,
                SkillEntry {
                    skill,
                    category_slug: category.slug,
                    category_title: category.title,
                    rendered,
                },
            );
        }
        Ok(PageCtx {
            base_path: base_path.to_string(),
            now,
            plugin_version,
            asset_versions,
            categories,
            skills: entries,
            fresh,
        })
    }

    /// The build instant to the second, as the pages and version.txt carry
    /// it: `2026-10-29T14:11:17+00:00` (Python
    /// `isoformat(timespec="seconds")`).
    pub fn last_updated_utc(&self) -> String {
        self.now.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
    }

    /// A page's URL from its path below the base path (`""` for the landing
    /// page, `skills/<slug>/`, `graph/`): templates.py `_locale_url` for the
    /// one locale.
    pub fn url(&self, path_suffix: &str) -> String {
        format!("{}/{path_suffix}", self.base_path)
    }

    /// A skill page's URL (templates.py `skill_url`).
    pub fn skill_url(&self, slug: &str) -> String {
        self.url(&format!("skills/{}/", escape_html(slug)))
    }

    /// The URL of a file in docs/assets/, fingerprinted when the build hashed
    /// it (templates.py `asset_url`).
    pub fn asset_url(&self, name: &str) -> String {
        match self.asset_versions.get(name) {
            Some(version) => format!("{}/assets/{name}?v={version}", self.base_path),
            None => format!("{}/assets/{name}", self.base_path),
        }
    }

    /// An icon from the icons.svg sprite (templates.py `_icon`).
    pub fn icon(&self, symbol_id: &str, css_class: &str) -> String {
        format!(
            "<svg class=\"{css_class}\" aria-hidden=\"true\"><use href=\"{}#{symbol_id}\"></use></svg>",
            self.asset_url("icons.svg")
        )
    }

    /// Whether a page built now still announces the graph
    /// (templates.py `graph_is_new`).
    pub fn graph_is_new(&self) -> bool {
        self.last_updated_utc()[..10] <= *GRAPH_NEW_UNTIL
    }

    /// Whether a page built now still says "New" of the climb release, on the
    /// header's Mods link and the landing page's Mods counter.
    pub fn climb_is_new(&self) -> bool {
        self.last_updated_utc()[..10] <= *CLIMB_NEW_UNTIL
    }

    /// The skills of `category` in its declared order.
    pub fn skills_in(&self, category: &Category) -> Vec<&SkillEntry> {
        category
            .skill_slugs
            .iter()
            .filter_map(|slug| self.skills.get(*slug))
            .collect()
    }
}

/// Joins template fragments with newlines, dropping empty ones: an empty
/// fragment means "this part does not apply here" and leaves no trace
/// (templates.py `_join`).
pub fn join<S: AsRef<str>>(parts: &[S]) -> String {
    parts
        .iter()
        .map(AsRef::as_ref)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The "New"/"Updated" badge for a skill inside the freshness window, or
/// nothing (templates.py `_render_change_badge`). `data-fresh-at` lets
/// site.js retire it in the browser.
pub fn change_badge(stamp: Option<&ChangeStamp>) -> String {
    let Some(stamp) = stamp else {
        return String::new();
    };
    let (status, label) = match stamp.status.as_deref() {
        Some("new") => ("new", strings::CHANGE_BADGE_NEW),
        Some("updated") => ("updated", strings::CHANGE_BADGE_UPDATED),
        _ => return String::new(),
    };
    let fresh_at = match stamp.at.as_deref() {
        Some(at) if !at.is_empty() => format!(" data-fresh-at=\"{}\"", escape_html(at)),
        _ => String::new(),
    };
    format!(
        "<span class=\"change-badge change-badge--{status}\"{fresh_at}>{}</span>",
        escape_html(label)
    )
}

/// The right-hand rail of an interior page at desktop widths: an "On this
/// page" list of its sections, as links to their headings. Below desktop
/// widths it is not shown; the headings are all in the page.
pub(crate) fn contents_rail(sections: &[(&str, &str)]) -> String {
    let items: String = sections
        .iter()
        .map(|(id, heading)| {
            format!(
                "\n      <li><a href=\"#{}\">{}</a></li>",
                escape_html(id),
                escape_html(heading)
            )
        })
        .collect();
    format!(
        r#"<nav class="page-rail page-rail--contents" aria-labelledby="page-rail-title">
    <p class="page-rail__title" id="page-rail-title">{title}</p>
    <ol class="page-rail__contents">{items}
    </ol>
  </nav>"#,
        title = escape_html(strings::PAGE_RAIL_CONTENTS),
    )
}
