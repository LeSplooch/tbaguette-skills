//! A skill's page (templates.py `render_skill_page` and the parts it calls):
//! breadcrumb, head with badge and graph link, the body, the reference tabs,
//! prev/next within the category, and the category's other skills.

use atelier_core::content::References;
use atelier_core::graph::CitationList;
use atelier_core::markdown::ReferenceSection;

use super::index::PLUGIN_NAME;
use super::shell::{self, Page};
use super::{
    BRAND_ATELIER_TEXT, GRAPH_PATH, PageCtx, SkillEntry, change_badge, escape_html, join, strings,
};

/// A description longer than this many characters is clamped to three lines
/// behind a "Show the full trigger" disclosure. Three lines of the measure
/// hold about this much at desktop widths; on a phone a description under it
/// is shown whole rather than clamped with no way to open it.
const TRIGGER_CLAMP_CHARS: usize = 220;

/// The full HTML document for the skill `slug`. `citations` is who cites it
/// and whom it cites (the graph's edges), for the strip and the rail; a skill
/// the graph does not have gets neither.
///
/// # Panics
///
/// If `slug` is not one of `ctx.skills`.
pub fn render(ctx: &PageCtx, slug: &str, citations: Option<&CitationList>) -> String {
    let entry = &ctx.skills[slug];
    let (prev, next, siblings) = prev_next_and_siblings(ctx, entry);
    // The body is pre-rendered HTML from the content pipeline, inserted
    // verbatim, as the reference panels are. The head and body share a
    // column with the rail beside them at desktop widths; the reference tabs
    // below take the full width.
    let article = format!(
        "<article class=\"container skill-article\"><div class=\"page-layout\"><div class=\"page-layout__main\">{head}{strip}<div class=\"prose\">{body}</div></div>{rail}</div>{extras}{references}</article>",
        head = skill_head(ctx, entry),
        strip = citation_strip(ctx, slug, citations),
        body = entry.rendered.body_html,
        rail = skill_rail(ctx, entry, citations.map_or(0, |c| c.cited_by.len())),
        extras = formidable_extras(&entry.rendered.references),
        references = reference_sections(&entry.rendered.references),
    );
    let main_html = join(&[
        breadcrumb(ctx, entry),
        article,
        prevnext(ctx, prev, next),
        see_also(ctx, entry, &siblings),
    ]);
    let summary = if entry.rendered.summary.is_empty() {
        &entry.skill.description
    } else {
        &entry.rendered.summary
    };
    shell::document(
        ctx,
        &Page {
            title: &format!(
                "{} — {} — {BRAND_ATELIER_TEXT}",
                entry.skill.name, entry.category_title
            ),
            meta_description: summary,
            body_class: "page-skill",
            path_suffix: &format!("skills/{slug}/"),
            main_html: &main_html,
        },
    )
}

/// The skills before and after this one in its category's declared order,
/// and the whole category (generate.py `_prev_next_and_siblings`).
fn prev_next_and_siblings<'c>(
    ctx: &'c PageCtx,
    entry: &SkillEntry,
) -> (
    Option<&'c SkillEntry>,
    Option<&'c SkillEntry>,
    Vec<&'c SkillEntry>,
) {
    let Some(category) = ctx
        .categories
        .iter()
        .find(|c| c.slug == entry.category_slug)
    else {
        return (None, None, Vec::new());
    };
    let ordered = ctx.skills_in(category);
    let Some(i) = ordered
        .iter()
        .position(|s| s.skill.slug == entry.skill.slug)
    else {
        return (None, None, ordered);
    };
    let prev = i.checked_sub(1).map(|p| ordered[p]);
    let next = ordered.get(i + 1).copied();
    (prev, next, ordered)
}

/// templates.py `_render_breadcrumb`: Home / category / this skill.
fn breadcrumb(ctx: &PageCtx, entry: &SkillEntry) -> String {
    let home = ctx.url("");
    format!(
        r#"<nav class="container breadcrumb" aria-label="{aria}">
  <a href="{home}">{home_label}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <a href="{home}#{category_slug}">{category_title}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">{name}</span>
</nav>"#,
        aria = escape_html(strings::BREADCRUMB_ARIA_LABEL),
        home_label = escape_html(strings::BREADCRUMB_HOME),
        category_slug = escape_html(entry.category_slug),
        category_title = escape_html(entry.category_title),
        name = escape_html(&entry.skill.name),
    )
}

/// templates.py `_render_skill_head`: the title row with the change badge,
/// the category tag with the graph link beside it (it is about where the
/// skill sits among the others), and the description, its cross-references
/// linked. A long description is clamped to three lines behind a native
/// `<details>` disclosure (styles.css opens the clamp with `:has`), so it
/// opens without the WASM module; the text itself is always all there for a
/// screen reader.
fn skill_head(ctx: &PageCtx, entry: &SkillEntry) -> String {
    let skill = &entry.skill;
    let graph_link = format!(
        "<a class=\"skill-article__graph-link\" href=\"{}#skill={}\">{}<span>{}</span></a>",
        ctx.url(GRAPH_PATH),
        escape_html(&skill.slug),
        ctx.icon("icon-graph", "icon"),
        escape_html(strings::SKILL_GRAPH_LINK),
    );
    let lede = if entry.rendered.description_html.is_empty() {
        escape_html(&skill.description)
    } else {
        entry.rendered.description_html.clone()
    };
    format!(
        r#"<div class="skill-article__head">
  <div class="skill-article__title-row">
    <h1 class="skill-article__title">{name}</h1>
    {badge}
  </div>
  <div class="skill-article__meta">
    <span class="tag skill-article__tag">{category_title}</span>
    {graph_link}
  </div>
  {trigger}
</div>"#,
        trigger = trigger(&skill.description, &lede),
        name = escape_html(&skill.name),
        badge = change_badge(ctx.fresh.get(&skill.slug)),
        category_title = escape_html(entry.category_title),
    )
}

/// The description as the page's trigger: roman body text, and past
/// [`TRIGGER_CLAMP_CHARS`] clamped with the disclosure that opens it.
fn trigger(plain: &str, html: &str) -> String {
    if plain.chars().count() <= TRIGGER_CLAMP_CHARS {
        return format!(
            "<div class=\"skill-trigger\"><p class=\"skill-trigger__text\">{html}</p></div>"
        );
    }
    format!(
        r#"<div class="skill-trigger skill-trigger--long">
    <p class="skill-trigger__text" id="skill-trigger-text">{html}</p>
    <details class="skill-trigger__more"><summary aria-controls="skill-trigger-text"><span class="skill-trigger__open">{more}</span><span class="skill-trigger__close">{less}</span></summary></details>
  </div>"#,
        more = escape_html(strings::SKILL_TRIGGER_MORE),
        less = escape_html(strings::SKILL_TRIGGER_LESS),
    )
}

/// templates.py `_render_citation_strip`: two rows under the description, who
/// cites this skill and whom it cites. The counts open the skill in the graph;
/// each chip is the other skill's page. A skill with neither gets no strip.
fn citation_strip(ctx: &PageCtx, slug: &str, citations: Option<&CitationList>) -> String {
    let Some(citations) = citations else {
        return String::new();
    };
    let graph_href = format!("{}#skill={}", ctx.url(GRAPH_PATH), escape_html(slug));
    let rows: String = [
        (strings::SKILL_CITED_BY, &citations.cited_by),
        (strings::SKILL_CITES, &citations.cites),
    ]
    .iter()
    .filter(|(_, slugs)| !slugs.is_empty())
    .map(|(label, slugs)| {
        let chips: String = slugs
            .iter()
            .map(|other| {
                format!(
                    "<a class=\"skill-cites__chip\" href=\"{}\">{}</a>",
                    ctx.skill_url(other),
                    escape_html(other)
                )
            })
            .collect();
        format!(
            "<div class=\"skill-cites__row\"><dt class=\"skill-cites__label\"><a href=\"{graph_href}\">{} {}</a></dt><dd class=\"skill-cites__chips\">{chips}</dd></div>",
            escape_html(label),
            slugs.len()
        )
    })
    .collect();
    if rows.is_empty() {
        return String::new();
    }
    format!("<dl class=\"skill-cites\">{rows}</dl>")
}

/// The rail beside a skill's body at desktop widths: how to invoke it, its
/// family, and how many other skills cite it (a way into the graph).
fn skill_rail(ctx: &PageCtx, entry: &SkillEntry, cited_by: usize) -> String {
    let slug = &entry.skill.slug;
    let cited = match cited_by {
        0 => strings::SKILL_RAIL_CITED_NONE.to_string(),
        1 => strings::SKILL_RAIL_CITED_ONE.to_string(),
        n => strings::SKILL_RAIL_CITED_MANY_TEMPLATE.replace("{count}", &n.to_string()),
    };
    let cited_html = if cited_by == 0 {
        escape_html(&cited)
    } else {
        format!(
            "<a href=\"{}#skill={}\">{}</a>",
            ctx.url(GRAPH_PATH),
            escape_html(slug),
            escape_html(&cited)
        )
    };
    format!(
        r#"<aside class="page-rail" aria-label="{label}">
    <dl class="page-rail__facts">
      <div><dt>{invoke}</dt><dd><code class="page-rail__invoke">{plugin}:{slug}</code></dd></div>
      <div><dt>{family}</dt><dd><a href="{home}#{category_slug}">{category_title}</a></dd></div>
      <div><dt>{cited_label}</dt><dd>{cited_html}</dd></div>
    </dl>
  </aside>"#,
        label = escape_html(strings::SKILL_RAIL_LABEL),
        invoke = escape_html(strings::SKILL_RAIL_INVOKE),
        plugin = escape_html(PLUGIN_NAME),
        slug = escape_html(slug),
        family = escape_html(strings::SKILL_RAIL_FAMILY),
        home = ctx.url(""),
        category_slug = escape_html(entry.category_slug),
        category_title = escape_html(entry.category_title),
        cited_label = escape_html(strings::SKILL_RAIL_CITED),
    )
}

/// templates.py `_render_tab_group`: an ARIA tab list with the first tab
/// selected and the other panels `hidden`. Panel ids are the sections' own
/// ids, so the pipeline's rewritten `.md` links land on them. `group_id` is a
/// fixed ASCII key, never derived from the heading.
fn tab_group(group_id: &str, heading: &str, items: &[ReferenceSection]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut tabs = String::new();
    let mut panels = String::new();
    for (i, item) in items.iter().enumerate() {
        let item_id = escape_html(&item.id);
        let tab_id = format!("tab-{item_id}");
        let (selected, tabindex, hidden) = if i == 0 {
            ("true", "0", "")
        } else {
            ("false", "-1", " hidden")
        };
        tabs.push_str(&format!(
            "<button class=\"tabs__tab\" type=\"button\" role=\"tab\" id=\"{tab_id}\" \
             aria-controls=\"{item_id}\" aria-selected=\"{selected}\" tabindex=\"{tabindex}\">{}</button>",
            escape_html(&item.title)
        ));
        panels.push_str(&format!(
            "<div class=\"tabs__panel\" role=\"tabpanel\" id=\"{item_id}\" \
             aria-labelledby=\"{tab_id}\" tabindex=\"0\"{hidden}>{}</div>",
            item.html
        ));
    }
    let heading = escape_html(heading);
    format!(
        r#"<section class="formidable-extra__group" aria-labelledby="{group_id}-heading">
  <h2 id="{group_id}-heading" class="formidable-extra__subtitle">{heading}</h2>
  <div class="tabs" data-tabs>
    <div class="tabs__list" role="tablist" aria-label="{heading}">
      {tabs}
    </div>
    <div class="tabs__panels">
      {panels}
    </div>
  </div>
</section>"#
    )
}

/// templates.py `_render_craft_floor`: not a tab, since it is the quality bar
/// rather than a command; links across formidable's content point at
/// `#cmd-craft-floor`, so the id must exist.
fn craft_floor(section: Option<&ReferenceSection>) -> String {
    let Some(section) = section.filter(|s| !s.html.is_empty()) else {
        return String::new();
    };
    format!(
        r#"<section class="formidable-extra__group" id="cmd-craft-floor" aria-labelledby="craft-floor-heading">
  <h2 id="craft-floor-heading" class="formidable-extra__subtitle">{heading}</h2>
  <div class="prose">{html}</div>
</section>"#,
        heading = escape_html(strings::FORMIDABLE_CRAFT_FLOOR_HEADING),
        html = section.html,
    )
}

/// templates.py `_render_formidable_extras`: Stacks and Commands tab groups
/// and the craft floor.
fn formidable_extras(references: &References) -> String {
    let References::Formidable {
        stacks,
        commands,
        craft_floor: floor,
    } = references
    else {
        return String::new();
    };
    let groups = join(&[
        tab_group("stacks", strings::FORMIDABLE_STACKS_HEADING, stacks),
        tab_group("commands", strings::FORMIDABLE_COMMANDS_HEADING, commands),
        craft_floor(floor.as_ref()),
    ]);
    if groups.is_empty() {
        return String::new();
    }
    format!("<div class=\"formidable-extra\">{groups}</div>")
}

/// templates.py `_render_reference_sections`: an ordinary skill's reference
/// files as one tab group, ids `ref-<stem>`.
fn reference_sections(references: &References) -> String {
    let References::Plain(sections) = references else {
        return String::new();
    };
    let group = tab_group("reference", strings::REFERENCE_HEADING, sections);
    if group.is_empty() {
        return String::new();
    }
    format!("<div class=\"formidable-extra\">{group}</div>")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Prev,
    Next,
}

/// templates.py `_render_prevnext_link`.
fn prevnext_link(ctx: &PageCtx, skill: Option<&SkillEntry>, direction: Direction) -> String {
    let Some(skill) = skill else {
        return String::new();
    };
    let name = escape_html(&skill.skill.name);
    let (label, modifier, name_html) = match direction {
        Direction::Prev => (strings::PREVNEXT_PREVIOUS, "", format!("← {name}")),
        Direction::Next => (
            strings::PREVNEXT_NEXT,
            " prevnext__link--next",
            format!("{name} →"),
        ),
    };
    format!(
        r#"<a class="prevnext__link{modifier}" href="{href}">
  <span class="prevnext__label">{label}</span>
  <span class="prevnext__name">{name_html}</span>
</a>"#,
        href = ctx.skill_url(&skill.skill.slug),
        label = escape_html(label),
    )
}

/// templates.py `_render_prevnext`.
fn prevnext(ctx: &PageCtx, prev: Option<&SkillEntry>, next: Option<&SkillEntry>) -> String {
    if prev.is_none() && next.is_none() {
        return String::new();
    }
    let links = join(&[
        prevnext_link(ctx, prev, Direction::Prev),
        prevnext_link(ctx, next, Direction::Next),
    ]);
    format!(
        "<nav class=\"container prevnext\" aria-label=\"{}\">{links}</nav>",
        escape_html(strings::PREVNEXT_ARIA_LABEL)
    )
}

/// templates.py `_render_see_also`: "More in <category>", every other skill
/// of the category in its declared order.
fn see_also(ctx: &PageCtx, entry: &SkillEntry, siblings: &[&SkillEntry]) -> String {
    let items: Vec<String> = siblings
        .iter()
        .filter(|s| s.skill.slug != entry.skill.slug)
        .map(|s| {
            format!(
                r#"<li><a class="see-also__link" href="{href}">
      <span class="see-also__name">{name}</span>
      <span class="see-also__summary">{summary}</span>
    </a></li>"#,
                href = ctx.skill_url(&s.skill.slug),
                name = escape_html(&s.skill.name),
                summary = escape_html(&s.rendered.summary),
            )
        })
        .collect();
    if items.is_empty() {
        return String::new();
    }
    let title = strings::SEE_ALSO_TITLE_TEMPLATE.replace("{category}", entry.category_title);
    format!(
        r#"<div class="container see-also">
  <h2 class="see-also__title">{title}</h2>
  <ul class="see-also__list">
{items}
  </ul>
</div>"#,
        title = escape_html(&title),
        items = join(&items),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skill_with_no_citations_gets_no_strip() {
        let skills = crate::repo::load_skills(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."),
        )
        .unwrap();
        let ctx = PageCtx::new(
            skills,
            atelier_core::model::categories(),
            "",
            chrono::Utc::now(),
            String::new(),
            Default::default(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(citation_strip(&ctx, "formidable", None), "");
        assert_eq!(
            citation_strip(&ctx, "formidable", Some(&CitationList::default())),
            ""
        );
        let html = citation_strip(
            &ctx,
            "formidable",
            Some(&CitationList {
                cites: vec!["c-skill".into()],
                cited_by: vec!["a-skill".into(), "b-skill".into()],
            }),
        );
        assert!(
            html.contains("href=\"/graph/#skill=formidable\">Cited by 2</a>"),
            "{html}"
        );
        assert!(
            html.contains("href=\"/skills/a-skill/\">a-skill</a>"),
            "{html}"
        );
        assert!(
            html.contains("href=\"/graph/#skill=formidable\">Cites 1</a>"),
            "{html}"
        );
    }
}
