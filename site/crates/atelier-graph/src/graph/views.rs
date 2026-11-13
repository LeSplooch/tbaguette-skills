//! Every piece of HTML the graph writes: the shell's bar and stage furniture,
//! the legend, the trail, the side panel (overview, one skill, a trace), the
//! tooltip, and the list and matrix pages. Strings are built exactly as
//! graph.js builds them, so the DOM is the same DOM, attribute for attribute.
//! Pure functions over the model; the app sets them as innerHTML.

use super::model::{Model, Path, Sec, quote_of, reach_in_two, reach_two, ref_count};
use super::text::*;

pub fn icon(href: &str) -> String {
    format!("<svg class=\"icon\" aria-hidden=\"true\"><use href=\"{href}\"></use></svg>")
}

/// graph.js `iconHref`, given the stylesheet link's and an icon use's hrefs.
pub fn icon_href(stylesheet: Option<&str>, used: Option<&str>, name: &str) -> String {
    let base = match stylesheet {
        Some(href) => match href.rfind("styles.css") {
            // `styles\.css(\?.*)?$`: the last "styles.css", then nothing or a query.
            Some(i) if href[i + 10..].is_empty() || href[i + 10..].starts_with('?') => {
                href[..i].to_string()
            }
            _ => href.to_string(),
        },
        None => "/assets/".to_string(),
    };
    let sprite = match used {
        // `^.*\/assets\/(icons\.svg[^#]*)#.*$`
        Some(href) => match href.rfind("/assets/icons.svg") {
            Some(i) => {
                let rest = &href[i + 8..];
                match rest.find('#') {
                    Some(h) => rest[..h].to_string(),
                    None => href.to_string(),
                }
            }
            None => href.to_string(),
        },
        None => "icons.svg".to_string(),
    };
    format!("{base}{sprite}#{name}")
}

fn chip_var(cat: usize) -> String {
    format!("--chip:var(--graph-cat-{})", cat + 1)
}

pub fn role_svg(role: &str) -> String {
    let core = "<circle cx=\"12\" cy=\"12\" r=\"5\" fill=\"currentColor\"/>";
    let parts = match role {
        "foundation" => format!("{core}<circle cx=\"12\" cy=\"12\" r=\"8.5\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\"/>"),
        "signpost" => format!("{core}<path d=\"M12 2.5V5.5M12 18.5V21.5M2.5 12H5.5M18.5 12H21.5M5.3 5.3L7.4 7.4M16.6 16.6L18.7 18.7M5.3 18.7L7.4 16.6M16.6 7.4L18.7 5.3\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\"/>"),
        "crossroads" => "<circle cx=\"12\" cy=\"12\" r=\"6\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"3.5\"/><circle cx=\"12\" cy=\"12\" r=\"9.5\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.2\"/>".to_string(),
        _ => core.to_string(),
    };
    format!(
        "<svg class=\"crumb-role-glyph\" viewBox=\"0 0 24 24\" aria-hidden=\"true\">{parts}</svg>"
    )
}

/// The bar's inner HTML (title, view tabs, search, trace button).
pub fn bar_html(count: u32, search_icon: &str, route_icon: &str) -> String {
    let mut h = format!(
        "<div class=\"crumb__heading\"><p class=\"crumb__title\" aria-hidden=\"true\">{}</p><p class=\"crumb__meta\" data-crumb-meta></p></div>",
        escape_html(TITLE)
    );
    h.push_str(&format!(
        "<div class=\"crumb__views\" role=\"tablist\" aria-label=\"{}\">",
        escape_html(VIEWS_LABEL)
    ));
    for v in ["constellation", "wheel", "matrix", "list"] {
        h.push_str(&format!(
            "<button type=\"button\" role=\"tab\" class=\"crumb__view\" data-crumb-view=\"{v}\" aria-selected=\"{}\">{}</button>",
            v == "constellation",
            escape_html(view_name(v))
        ));
    }
    h.push_str("</div>");
    h.push_str(&format!(
        "<form class=\"crumb__search\" role=\"search\" data-crumb-search><label class=\"visually-hidden\" for=\"crumb-search-{count}\">{}</label>{search_icon}<input id=\"crumb-search-{count}\" type=\"search\" autocomplete=\"off\" spellcheck=\"false\" placeholder=\"{}\" list=\"crumb-skills-{count}\"><datalist id=\"crumb-skills-{count}\"></datalist><span class=\"crumb__searchnote\" data-crumb-searchnote hidden></span></form>",
        escape_html(SEARCH_LABEL),
        escape_html(SEARCH_PLACEHOLDER)
    ));
    h.push_str(&format!("<button type=\"button\" class=\"crumb__btn\" data-crumb-trace>{route_icon}<span>{}</span></button>", escape_html(TRACE)));
    h
}

/// What the stage holds after its canvas.
pub fn stage_furniture_html(coarse: bool, plus: &str, minus: &str, fit: &str) -> String {
    let mut h = String::new();
    h.push_str("<nav class=\"crumb__trail\" aria-label=\"Trail\" data-crumb-trail hidden></nav>");
    h.push_str(&format!(
        "<p class=\"crumb__hint\" data-crumb-hint>{}</p>",
        escape_html(if coarse { HINT_TOUCH } else { HINT })
    ));
    h.push_str("<div class=\"crumb__legend\" data-crumb-legend></div>");
    h.push_str("<div class=\"crumb__anatomy-key\" data-crumb-key hidden>");
    h.push_str(&format!("<span><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\"><circle cx=\"8\" cy=\"8\" r=\"5\" fill=\"currentColor\"/></svg>{}</span>", escape_html(LEGEND_CITES)));
    h.push_str(&format!("<span><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\"><circle cx=\"8\" cy=\"8\" r=\"4.5\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.6\"/></svg>{}</span>", escape_html(LEGEND_CITED_BY)));
    h.push_str(&format!("<span><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\"><circle cx=\"8\" cy=\"8\" r=\"3.5\" fill=\"currentColor\"/><circle cx=\"8\" cy=\"8\" r=\"6.5\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.3\"/></svg>{}</span></div>", escape_html(LEGEND_BOTH)));
    h.push_str("<div class=\"crumb__tools\">");
    h.push_str(&format!(
        "<label class=\"crumb__size\"><span>{}</span><select data-crumb-size>",
        escape_html(SIZE_BY)
    ));
    for k in ["in", "out", "words"] {
        h.push_str(&format!(
            "<option value=\"{k}\">{}</option>",
            escape_html(size_name(k))
        ));
    }
    h.push_str("</select></label>");
    for (how, label, ic) in [
        ("in", ZOOM_IN, plus),
        ("out", ZOOM_OUT, minus),
        ("fit", FIT, fit),
    ] {
        h.push_str(&format!(
            "<button type=\"button\" class=\"crumb__btn crumb__btn--icon\" data-crumb-zoom=\"{how}\" aria-label=\"{0}\" title=\"{0}\">{ic}</button>",
            escape_html(label)
        ));
    }
    h.push_str("</div>");
    h.push_str("<div class=\"crumb__tip\" data-crumb-tip hidden></div>");
    h.push_str("<div class=\"crumb__status\" data-crumb-status></div>");
    h.push_str("<div class=\"crumb__list\" data-crumb-list hidden></div>");
    h.push_str("<div class=\"crumb__list crumb__matrix\" data-crumb-matrix hidden></div>");
    h
}

pub fn panel_shell_html() -> String {
    format!(
        "<button type=\"button\" class=\"crumb__sheet-handle\" data-crumb-sheet aria-expanded=\"false\"><span class=\"crumb__sheet-grip\" aria-hidden=\"true\"></span><span data-crumb-sheet-label>{}</span></button><div class=\"crumb__panel-body\" data-crumb-panel></div>",
        escape_html(NOTES_TITLE)
    )
}

pub fn failed_html() -> String {
    format!(
        "<div class=\"crumb__failed\"><p>{}</p><button type=\"button\" class=\"crumb__btn\">{}</button></div>",
        escape_html(FAILED),
        escape_html(RETRY)
    )
}

pub fn loading_html() -> String {
    format!("<p class=\"crumb__loading\">{}</p>", escape_html(LOADING))
}

pub fn datalist_html(model: &Model) -> String {
    model
        .nodes
        .iter()
        .map(|n| format!("<option value=\"{}\"></option>", escape_html(&n.name)))
        .collect()
}

pub fn legend_html(model: &Model) -> String {
    let mut h = format!(
        "<p class=\"crumb__legend-title\">{}</p><div class=\"crumb__legend-list\">",
        escape_html(FAMILIES_TITLE)
    );
    for c in &model.categories {
        h.push_str(&format!(
            "<button type=\"button\" class=\"crumb-family\" data-crumb-act=\"family\" data-crumb-family=\"{}\" aria-pressed=\"false\" style=\"{}\"><span class=\"crumb-swatch\"></span>{}<span class=\"crumb-family__count\">{}</span></button>",
            escape_html(&c.slug),
            chip_var(c.index),
            escape_html(&c.title),
            c.nodes.len()
        ));
    }
    h.push_str("</div>");
    h
}

pub fn trail_html(model: &Model, trail: &[usize], focus: usize, back_icon: &str) -> String {
    let mut h = format!(
        "<button type=\"button\" class=\"crumb__crumb crumb__crumb--back\" data-crumb-act=\"back\" aria-label=\"{}\">{back_icon}</button><button type=\"button\" class=\"crumb__crumb\" data-crumb-act=\"crumb\" data-crumb-index=\"-1\">{}</button>",
        escape_html(BACK),
        escape_html(LIBRARY)
    );
    let shown = &trail[trail.len().saturating_sub(3)..];
    let offset = trail.len() - shown.len();
    if offset > 0 {
        h.push_str("<span class=\"crumb__crumb-sep\" aria-hidden=\"true\">…</span>");
    }
    for (i, &n) in shown.iter().enumerate() {
        h.push_str(&format!(
            "<span class=\"crumb__crumb-sep\" aria-hidden=\"true\">/</span><button type=\"button\" class=\"crumb__crumb\" data-crumb-act=\"crumb\" data-crumb-index=\"{}\">{}</button>",
            i + offset,
            escape_html(&model.nodes[n].name)
        ));
    }
    h.push_str(&format!(
        "<span class=\"crumb__crumb-sep\" aria-hidden=\"true\">/</span><span class=\"crumb__crumb crumb__crumb--here\" aria-current=\"location\">{}</span>",
        escape_html(&model.nodes[focus].name)
    ));
    h
}

pub fn panel_overview(model: &Model) -> String {
    let mut counts = [0usize; 4];
    for n in &model.nodes {
        counts[match n.role.key() {
            "foundation" => 0,
            "signpost" => 1,
            "crossroads" => 2,
            _ => 3,
        }] += 1;
    }
    let count_of = |r: &str| {
        counts[match r {
            "foundation" => 0,
            "signpost" => 1,
            "crossroads" => 2,
            _ => 3,
        }]
    };
    let mut h = format!(
        "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><p class=\"crumb-panel__text\">{}</p></section>",
        escape_html(INTRO_TITLE),
        escape_html(INTRO)
    );
    h.push_str(&format!("<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><ul class=\"crumb-notes\">", escape_html(NOTES_TITLE)));
    for (i, note) in model.notes.iter().enumerate() {
        h.push_str(&format!("<li><button type=\"button\" class=\"crumb-note\" data-crumb-act=\"note\" data-crumb-note=\"{i}\" aria-pressed=\"false\">{}</button></li>", note.html));
    }
    h.push_str("</ul></section>");
    h.push_str(&format!("<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><ul class=\"crumb-roles\">", escape_html(ROLES_TITLE)));
    for r in ["foundation", "crossroads", "signpost", "specialist"] {
        let (name, blurb) = role_copy(r);
        h.push_str(&format!(
            "<li class=\"crumb-role\">{}<span><b>{}</b> <span class=\"crumb-role__count\">{}</span><br>{}</span></li>",
            role_svg(r),
            escape_html(name),
            count_of(r),
            escape_html(blurb)
        ));
    }
    h.push_str(&format!(
        "<li class=\"crumb-role\"><svg class=\"crumb-role-glyph\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"4\" fill=\"currentColor\"/><circle cx=\"12\" cy=\"12\" r=\"9\" fill=\"none\" stroke=\"var(--accent-solid)\" stroke-width=\"1.4\" stroke-dasharray=\"2.5 3\"/></svg><span>{}</span></li>",
        escape_html(ALWAYS_ON)
    ));
    h.push_str("</ul></section>");
    h
}

/// The one sentence that explains why two skills are joined, its mention marked.
pub fn quote_html(quote: Option<&str>, slug: &str) -> String {
    let Some(quote) = quote.filter(|q| !q.is_empty()) else {
        return String::new();
    };
    let mut safe = escape_html(quote);
    if let Some(i) = safe.find(slug) {
        safe = format!(
            "{}<mark>{slug}</mark>{}",
            &safe[..i],
            &safe[i + slug.len()..]
        );
    }
    format!("<blockquote class=\"crumb-quote\">{safe}</blockquote>")
}

pub struct Citing<'a> {
    pub title: &'a str,
    pub quote: Option<&'a str>,
    pub id: &'a str,
}

/// Where `from` first cites `to`: its trigger, else its first citing section.
pub fn citing_section(model: &Model, from: usize, to: usize) -> Option<Citing<'_>> {
    let f = &model.nodes[from];
    let slug = &model.nodes[to].slug;
    if ref_count(&f.trigger_refs, slug) != 0.0 {
        return Some(Citing {
            title: TRIGGER,
            quote: quote_of(&f.trigger_quotes, slug),
            id: "",
        });
    }
    f.sections
        .iter()
        .find(|s| ref_count(&s.refs, slug) != 0.0)
        .map(|s| Citing {
            title: &s.title,
            quote: quote_of(&s.quotes, slug),
            id: &s.id,
        })
}

fn cite_chip(model: &Model, slug: &str) -> String {
    let cat = model.node(slug).map_or(0, |n| model.nodes[n].cat);
    format!(
        "<button type=\"button\" class=\"crumb-chip crumb-chip--button\" data-crumb-act=\"open\" data-crumb-node=\"{0}\" style=\"{1}\">{0}</button>",
        escape_html(slug),
        chip_var(cat)
    )
}

fn section_item(model: &Model, s: &Sec, i: usize, on: bool, url: &str) -> String {
    let cites: Vec<&str> = s.refs.iter().map(|(k, _)| k.as_str()).collect();
    let mut detail = String::new();
    if on {
        detail.push_str("<div class=\"crumb-section__detail\">");
        if cites.is_empty() {
            detail.push_str(&format!(
                "<p class=\"crumb-panel__text\">{}</p>",
                escape_html(NO_CITES)
            ));
        } else {
            for slug in &cites {
                detail.push_str(&format!(
                    "<div class=\"crumb-cite\">{}{}</div>",
                    cite_chip(model, slug),
                    quote_html(quote_of(&s.quotes, slug), slug)
                ));
            }
        }
        let href = if s.id.is_empty() {
            url.to_string()
        } else {
            format!("{url}#{}", s.id)
        };
        detail.push_str(&format!(
            "<a class=\"crumb-link\" href=\"{}\">{} →</a></div>",
            escape_html(&href),
            escape_html(READ_SECTION)
        ));
    }
    let mut h = format!(
        "<li class=\"crumb-section{}{}\"><button type=\"button\" class=\"crumb-section__head\" data-crumb-act=\"section\" data-crumb-section=\"{i}\" aria-expanded=\"{on}\"><span class=\"crumb-section__num\">{}</span><span class=\"crumb-section__title\">{}</span>",
        if on { " crumb-section--on" } else { "" },
        if s.reference {
            " crumb-section--ref"
        } else {
            ""
        },
        i + 1,
        escape_html(&s.title)
    );
    if !cites.is_empty() {
        h.push_str(&format!(
            "<span class=\"crumb-section__cites\" title=\"{}\">{}</span>",
            escape_html(&fmt(CITES_COUNT, &[("n", cites.len().to_string())])),
            cites.len()
        ));
    }
    h.push_str("</button>");
    h.push_str(&detail);
    h.push_str("</li>");
    h
}

/// The panel for one opened skill. `fresh` is its live freshness status.
pub fn panel_skill(
    model: &Model,
    focus: usize,
    pin_section: i32,
    fresh: Option<&str>,
    route_icon: &str,
) -> String {
    let f = &model.nodes[focus];
    let url = model.skill_url(&f.slug);
    let reach = reach_two(model, focus);
    let sections: String = f
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| section_item(model, s, i, i as i32 == pin_section, &url))
        .collect();
    let mut inbound: Vec<usize> = f.inn.iter().map(|(s, _)| *s).collect();
    inbound.sort_by(|&a, &b| {
        let (na, nb) = (&model.nodes[a], &model.nodes[b]);
        na.cat.cmp(&nb.cat).then(na.index.cmp(&nb.index))
    });
    let inbound_html: String = inbound
        .iter()
        .map(|&n| {
            let node = &model.nodes[n];
            let src = citing_section(model, n, focus);
            format!(
                "<li><button type=\"button\" class=\"crumb-inbound\" data-crumb-act=\"open\" data-crumb-node=\"{}\"><span class=\"crumb-swatch\" style=\"{}\"></span><span class=\"crumb-inbound__name\">{}</span>{}</button></li>",
                escape_html(&node.slug),
                chip_var(node.cat),
                escape_html(&node.name),
                src.map_or(String::new(), |s| format!(
                    "<span class=\"crumb-inbound__from\">{}</span>",
                    escape_html(&fmt(FROM_SECTION, &[("title", s.title.to_string())]))
                ))
            )
        })
        .collect();
    let role = f.role.key();
    let mut h = String::new();
    h.push_str("<section class=\"crumb-panel__section crumb-skill\">");
    h.push_str(&format!(
        "<h3 class=\"crumb-skill__name\">{}</h3>",
        escape_html(&f.name)
    ));
    h.push_str(&format!(
        "<p class=\"crumb-skill__meta\"><span class=\"crumb-chip crumb-chip--family\" style=\"{}\">{}</span><span class=\"crumb-skill__role\">{}{}</span></p>",
        chip_var(f.cat),
        escape_html(&model.categories[f.cat].title),
        role_svg(role),
        escape_html(role_copy(role).0)
    ));
    if f.always_on {
        h.push_str(&format!(
            "<p class=\"crumb-skill__flag\">{}</p>",
            escape_html(ALWAYS_ON)
        ));
    }
    if let Some(fresh) = fresh {
        let copy = match fresh {
            "new" => FRESH_NEW,
            "updated" => FRESH_UPDATED,
            _ => "",
        };
        h.push_str(&format!(
            "<p class=\"crumb-skill__flag crumb-skill__flag--fresh\">{}</p>",
            escape_html(copy)
        ));
    }
    h.push_str(&format!(
        "<p class=\"crumb-panel__text\">{}</p>",
        escape_html(&f.summary)
    ));
    h.push_str(&format!(
        "<dl class=\"crumb-stats\"><div><dt>{}</dt><dd>{}</dd></div><div><dt>{}</dt><dd>{}</dd></div><div><dt>{}</dt><dd>{}</dd></div><div><dt>{}</dt><dd>{}</dd></div></dl>",
        escape_html(SECTIONS),
        f.sections.len(),
        escape_html(WORDS),
        grouped(f.words),
        escape_html(CITES),
        f.out_deg,
        escape_html(CITED_BY),
        f.in_deg
    ));
    if !reach.nodes.is_empty() {
        let fams = fmt(
            REACH_FAMILIES[usize::from(reach.families != 1)],
            &[("k", reach.families.to_string())],
        );
        h.push_str(&format!(
            "<p class=\"crumb-panel__text crumb-skill__reach\">{} <button type=\"button\" class=\"crumb-link crumb-link--button\" data-crumb-act=\"reach\">{}</button></p>",
            fmt(REACH, &[("n", reach.nodes.len().to_string()), ("f", fams)]),
            escape_html(REACH_LIGHT)
        ));
    }
    let reached = reach_in_two(model, focus);
    if reached > 0 {
        let pct = (100.0 * reached as f64 / (model.nodes.len().max(2) - 1) as f64).round();
        h.push_str(&format!(
            "<p class=\"crumb-panel__text crumb-skill__reached\">{}</p>",
            fmt(REACHED_BY, &[("n", reached.to_string()), ("p", format!("{pct}"))])
        ));
    }
    h.push_str(&format!(
        "<div class=\"crumb-actions\"><a class=\"crumb__btn crumb__btn--solid\" href=\"{}\">{}</a><button type=\"button\" class=\"crumb__btn\" data-crumb-act=\"trace-from\">{route_icon}<span>{}</span></button></div>",
        escape_html(&url),
        escape_html(OPEN_SKILL),
        escape_html(TRACE_FROM)
    ));
    h.push_str(&format!(
        "<p class=\"crumb-panel__hint\">{}</p></section>",
        escape_html(ANATOMY_HINT)
    ));
    if !f.trigger_refs.is_empty() {
        h.push_str(&format!(
            "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><p class=\"crumb-panel__text\">{}</p>",
            escape_html(TRIGGER),
            escape_html(TRIGGER_NOTE)
        ));
        for (slug, _) in &f.trigger_refs {
            h.push_str(&format!(
                "<div class=\"crumb-cite\">{}{}</div>",
                cite_chip(model, slug),
                quote_html(quote_of(&f.trigger_quotes, slug), slug)
            ));
        }
        h.push_str("</section>");
    }
    h.push_str(&format!(
        "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><ol class=\"crumb-sections\">{sections}</ol></section>",
        escape_html(SECTIONS)
    ));
    if !inbound.is_empty() {
        h.push_str(&format!(
            "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{} <span class=\"crumb-panel__count\">{}</span></h3><ul class=\"crumb-inbounds\">{inbound_html}</ul></section>",
            escape_html(CITED_BY),
            inbound.len()
        ));
    }
    h
}

pub fn panel_trace(
    model: &Model,
    from: Option<usize>,
    to: Option<usize>,
    path: Option<&Path>,
    datalist_id: &str,
) -> String {
    let name = |n: Option<usize>| n.map_or(String::new(), |n| escape_html(&model.nodes[n].name));
    let mut h = format!(
        "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><p class=\"crumb-panel__text\">{}</p><form class=\"crumb-trace\" data-crumb-trace-form>",
        escape_html(TRACE_TITLE),
        escape_html(TRACE_INTRO)
    );
    h.push_str(&format!(
        "<label><span>{}</span><input type=\"text\" data-crumb-trace-from list=\"{datalist_id}\" autocomplete=\"off\" spellcheck=\"false\" value=\"{}\"></label>",
        escape_html(TRACE_FROM_LABEL),
        name(from)
    ));
    h.push_str(&format!(
        "<label><span>{}</span><input type=\"text\" data-crumb-trace-to list=\"{datalist_id}\" autocomplete=\"off\" spellcheck=\"false\" value=\"{}\"></label>",
        escape_html(TRACE_TO_LABEL),
        name(to)
    ));
    h.push_str("<p class=\"crumb-trace__error\" data-crumb-trace-error role=\"alert\"></p>");
    h.push_str(&format!(
        "<div class=\"crumb-actions\"><button type=\"submit\" class=\"crumb__btn crumb__btn--solid\">{}</button><button type=\"button\" class=\"crumb__btn\" data-crumb-trace-random>{}</button><button type=\"button\" class=\"crumb__btn crumb__btn--quiet\" data-crumb-trace-done>{}</button></div></form></section>",
        escape_html(TRACE_GO),
        escape_html(TRACE_SURPRISE),
        escape_html(TRACE_DONE)
    ));
    if from.is_some() && to.is_some() {
        match path {
            None => h.push_str(&format!("<section class=\"crumb-panel__section\"><p class=\"crumb-panel__text\">{}</p></section>", escape_html(TRACE_NONE))),
            Some(path) => {
                let steps = path.edges.len();
                let title = if steps == 1 { TRACE_STEP.to_string() } else { fmt(TRACE_STEPS, &[("n", steps.to_string())]) };
                h.push_str(&format!(
                    "<section class=\"crumb-panel__section\"><h3 class=\"crumb-panel__title\">{}</h3><div class=\"crumb-actions\"><button type=\"button\" class=\"crumb__btn\" data-crumb-trace-copy>{}</button></div><ol class=\"crumb-route\">",
                    escape_html(&title),
                    escape_html(COPY_LINK)
                ));
                for (i, &n) in path.nodes.iter().enumerate() {
                    let node = &model.nodes[n];
                    h.push_str(&format!(
                        "<li class=\"crumb-route__stop\"><button type=\"button\" class=\"crumb-chip crumb-chip--button\" data-crumb-act=\"open\" data-crumb-node=\"{}\" style=\"{}\">{}</button>",
                        escape_html(&node.slug),
                        chip_var(node.cat),
                        escape_html(&node.name)
                    ));
                    if let Some(&next) = path.nodes.get(i + 1)
                        && let Some(src) = citing_section(model, n, next) {
                            let next_slug = &model.nodes[next].slug;
                            h.push_str(&format!(
                                "<p class=\"crumb-route__via\">{}</p>{}",
                                escape_html(&fmt(FROM_SECTION, &[("title", src.title.to_string())])),
                                quote_html(src.quote, next_slug)
                            ));
                            let url = model.skill_url(&node.slug);
                            let href = if src.id.is_empty() { url } else { format!("{url}#{}", src.id) };
                            h.push_str(&format!("<a class=\"crumb-link\" href=\"{}\">{} →</a>", escape_html(&href), escape_html(READ_SECTION)));
                        }
                    h.push_str("</li>");
                }
                h.push_str("</ol></section>");
            }
        }
    }
    h
}

pub fn tip_node(model: &Model, n: usize) -> String {
    let node = &model.nodes[n];
    format!(
        "<p class=\"crumb-tip__name\">{}</p><p class=\"crumb-tip__meta\"><span class=\"crumb-swatch\" style=\"{}\"></span>{} · {}</p><p class=\"crumb-tip__body\">{}</p><p class=\"crumb-tip__stats\">{} · {}</p>",
        escape_html(&node.name),
        chip_var(node.cat),
        escape_html(&model.categories[node.cat].title),
        escape_html(role_copy(node.role.key()).0),
        escape_html(&node.summary),
        escape_html(&fmt(CITED_BY_COUNT, &[("n", node.in_deg.to_string())])),
        escape_html(&fmt(CITES_COUNT, &[("n", node.out_deg.to_string())]))
    )
}

pub fn tip_section(model: &Model, focus: usize, i: usize) -> String {
    let s = &model.nodes[focus].sections[i];
    let cites: Vec<String> = s.refs.iter().map(|(k, _)| escape_html(k)).collect();
    let mut h = format!(
        "<p class=\"crumb-tip__meta\"><span class=\"crumb-tip__num\">{}</span>{} · {}</p><p class=\"crumb-tip__name\">{}</p>",
        i + 1,
        escape_html(if s.reference { REFERENCE } else { "Section" }),
        escape_html(&fmt(WORDS_COUNT, &[("n", super::text::num(s.words))])),
        escape_html(&s.title)
    );
    if cites.is_empty() {
        h.push_str(&format!(
            "<p class=\"crumb-tip__body\">{}</p>",
            escape_html(NO_CITES)
        ));
    } else {
        h.push_str(&format!(
            "<p class=\"crumb-tip__body\">{} {}</p>",
            escape_html(CITES),
            cites.join(", ")
        ));
    }
    h
}

pub fn tip_hub(model: &Model, focus: usize) -> String {
    let f = &model.nodes[focus];
    format!(
        "<p class=\"crumb-tip__name\">{}</p><p class=\"crumb-tip__body\">{}</p>",
        escape_html(&f.name),
        escape_html(&f.summary)
    )
}

/// The tooltip for a neighbour in the anatomy: how the two are joined.
pub fn relation_html(model: &Model, focus: usize, n: usize) -> String {
    let f = &model.nodes[focus];
    let node = &model.nodes[n];
    let mut h = format!(
        "<p class=\"crumb-tip__name\">{}</p><p class=\"crumb-tip__meta\"><span class=\"crumb-swatch\" style=\"{}\"></span>{}</p>",
        escape_html(&node.name),
        chip_var(node.cat),
        escape_html(&model.categories[node.cat].title)
    );
    let mut shown = 0;
    if ref_count(&f.trigger_refs, &node.slug) != 0.0 {
        h.push_str(&format!(
            "<p class=\"crumb-tip__rel\">{}</p>{}",
            escape_html(TRIGGER_NOTE),
            quote_html(quote_of(&f.trigger_quotes, &node.slug), &node.slug)
        ));
        shown += 1;
    }
    for (i, s) in f.sections.iter().enumerate() {
        if ref_count(&s.refs, &node.slug) == 0.0 || shown > 1 {
            continue;
        }
        h.push_str(&format!(
            "<p class=\"crumb-tip__rel\"><span class=\"crumb-tip__num\">{}</span>{}</p>{}",
            i + 1,
            escape_html(&fmt(FROM_SECTION, &[("title", s.title.clone())])),
            quote_html(quote_of(&s.quotes, &node.slug), &node.slug)
        ));
        shown += 1;
    }
    if f.inn_edge(n).is_some()
        && let Some(back) = citing_section(model, n, focus)
    {
        h.push_str(&format!(
            "<p class=\"crumb-tip__rel crumb-tip__rel--in\">{}</p>{}",
            escape_html(&fmt(INBOUND_FROM, &[("title", back.title.to_string())])),
            quote_html(back.quote, &f.slug)
        ));
    }
    h
}

pub fn list_html(model: &Model, graph_icon: &str) -> String {
    let chips = |nodes: &[usize]| -> String {
        nodes
            .iter()
            .map(|&n| {
                let node = &model.nodes[n];
                format!(
                    "<a class=\"crumb-chip\" href=\"{}\" style=\"{}\">{}</a>",
                    escape_html(&model.skill_url(&node.slug)),
                    chip_var(node.cat),
                    escape_html(&node.slug)
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let jump = model
        .categories
        .iter()
        .map(|c| {
            format!(
                "<a href=\"#crumb-family-{}\">{}</a>",
                c.index,
                escape_html(&c.title)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut h = format!(
        "<p class=\"crumb-list__intro\">{}</p><nav class=\"crumb-list__jump\" aria-label=\"{}\">{jump}</nav>",
        escape_html(LIST_INTRO),
        escape_html(LIST_JUMP),
    );
    for c in &model.categories {
        h.push_str(&format!(
            "<section class=\"crumb-list__family\"><h3 class=\"crumb-list__title\" id=\"crumb-family-{}\" tabindex=\"-1\"><span class=\"crumb-swatch\" style=\"{}\"></span>{}</h3>",
            c.index,
            chip_var(c.index),
            escape_html(&c.title)
        ));
        for &n in &c.nodes {
            let node = &model.nodes[n];
            let outs: Vec<usize> = node.out.iter().map(|(t, _)| *t).collect();
            let ins: Vec<usize> = node.inn.iter().map(|(s, _)| *s).collect();
            let role = node.role.key();
            h.push_str(&format!(
                "<article class=\"crumb-list__skill\"><h4><a href=\"{}\">{}</a><button type=\"button\" class=\"crumb__btn crumb__btn--quiet crumb__btn--small\" data-crumb-act=\"open\" data-crumb-node=\"{}\" aria-label=\"{}: {}\">{graph_icon}<span>{}</span></button></h4><p class=\"crumb-list__role\">{}{}</p><details class=\"crumb-list__more\"><summary>{} {} · {} {}</summary><p class=\"crumb-list__refs\"><b>{} {}</b> {}</p><p class=\"crumb-list__refs\"><b>{} {}</b> {}</p></details></article>",
                escape_html(&model.skill_url(&node.slug)),
                escape_html(&node.name),
                escape_html(&node.slug),
                escape_html(LIST_OPEN),
                escape_html(&node.name),
                escape_html(LIST_OPEN),
                role_svg(role),
                escape_html(role_copy(role).0),
                escape_html(CITES),
                outs.len(),
                escape_html(CITED_BY),
                ins.len(),
                escape_html(CITES),
                outs.len(),
                chips(&outs),
                escape_html(CITED_BY),
                ins.len(),
                chips(&ins)
            ));
        }
        h.push_str("</section>");
    }
    h
}

pub fn matrix_counts(model: &Model) -> Vec<Vec<u32>> {
    let n = model.categories.len();
    let mut m = vec![vec![0u32; n]; n];
    for e in &model.edges {
        m[model.nodes[e.s].cat][model.nodes[e.t].cat] += 1;
    }
    m
}

/// The matrix page, and the first marked family's strongest pull
/// (graph.js `matrixFrom`) for the grid's initial tab stop.
pub fn matrix_html(model: &Model, families: Option<&[String]>) -> (String, Option<(usize, usize)>) {
    let cats = &model.categories;
    let m = matrix_counts(model);
    let mut max = 1u32;
    let mut best: Option<(usize, usize, u32)> = None;
    let mut inside = vec![0u32; cats.len()];
    for (i, row) in m.iter().enumerate() {
        for (j, &n) in row.iter().enumerate() {
            max = max.max(n);
            if i != j {
                inside[j] += n;
                if best.is_none_or(|b| n > b.2) {
                    best = Some((i, j, n));
                }
            }
        }
    }
    let top = inside.iter().copied().max().unwrap_or(0);
    let sink = inside.iter().position(|&v| v == top).unwrap_or(0);
    let on: Vec<bool> = cats
        .iter()
        .map(|c| families.is_some_and(|f| f.contains(&c.slug)))
        .collect();
    let mut matrix_from = None;
    let mut picked = Vec::new();
    for f in cats.iter().filter(|c| on[c.index]) {
        let i = f.index;
        let (mut out, mut inn): (Option<usize>, Option<usize>) = (None, None);
        for j in 0..cats.len() {
            if j == i {
                continue;
            }
            if m[i][j] > 0 && out.is_none_or(|o| m[i][j] > m[i][o]) {
                out = Some(j);
            }
            if m[j][i] > 0 && inn.is_none_or(|o| m[j][i] > m[o][i]) {
                inn = Some(j);
            }
        }
        if matrix_from.is_none() {
            matrix_from = out.map(|o| (i, o)).or(inn.map(|o| (o, i)));
        }
        let mut parts = Vec::new();
        if let Some(o) = out {
            parts.push(fmt(
                MATRIX_FAMILY_OUT,
                &[
                    ("a", f.title.clone()),
                    ("b", cats[o].title.clone()),
                    ("n", m[i][o].to_string()),
                ],
            ));
        }
        if let Some(o) = inn {
            let t = if parts.is_empty() {
                MATRIX_FAMILY_IN
            } else {
                MATRIX_FAMILY_IN_TOO
            };
            parts.push(fmt(
                t,
                &[
                    ("a", f.title.clone()),
                    ("b", cats[o].title.clone()),
                    ("n", m[o][i].to_string()),
                ],
            ));
        }
        picked.push(format!(
            "{}.",
            if parts.is_empty() {
                fmt(MATRIX_FAMILY_NONE, &[("a", f.title.clone())])
            } else {
                parts.join(", ")
            }
        ));
    }
    let swatch = |c: usize| {
        format!(
            "<span class=\"crumb-swatch\" style=\"{}\" aria-hidden=\"true\"></span>",
            chip_var(c)
        )
    };
    let mut head = format!(
        "<tr><td class=\"crumb-matrix__corner\" aria-hidden=\"true\"><span>{}</span> <span>{}</span></td>",
        escape_html(MATRIX_CORNER[0]),
        escape_html(MATRIX_CORNER[1])
    );
    for c in cats {
        head.push_str(&format!(
            "<th scope=\"col\" class=\"crumb-matrix__col{}\"><abbr title=\"{}\">{}<span class=\"crumb-matrix__num\">{}</span></abbr><span class=\"visually-hidden\">{}</span></th>",
            if on[c.index] { " crumb-matrix__col--on" } else { "" },
            escape_html(&c.title),
            swatch(c.index),
            c.index + 1,
            escape_html(&c.title)
        ));
    }
    head.push_str("</tr>");
    let mut body = String::new();
    for (i, a) in cats.iter().enumerate() {
        body.push_str(&format!(
            "<tr><th scope=\"row\" class=\"crumb-matrix__row{}\"><button type=\"button\" class=\"crumb-matrix__pick\" data-crumb-act=\"row\" data-crumb-row=\"{}\" aria-pressed=\"{}\" title=\"{}\">{}<span class=\"crumb-matrix__num\" aria-hidden=\"true\">{}</span><span class=\"crumb-matrix__name\">{}</span></button></th>",
            if on[i] { " crumb-matrix__row--on" } else { "" },
            escape_html(&a.slug),
            on[i],
            escape_html(&a.title),
            swatch(a.index),
            i + 1,
            escape_html(&a.title)
        ));
        for (j, b) in cats.iter().enumerate() {
            let n = m[i][j];
            let cls = format!(
                "crumb-matrix__cell{}{}",
                if i == j {
                    " crumb-matrix__cell--self"
                } else {
                    ""
                },
                if on[i] || on[j] {
                    " crumb-matrix__cell--on"
                } else {
                    ""
                }
            );
            if n == 0 {
                body.push_str(&format!(
                    "<td class=\"{cls}\"><span class=\"visually-hidden\">{}</span></td>",
                    escape_html(&fmt(
                        MATRIX_NONE,
                        &[("a", a.title.clone()), ("b", b.title.clone())]
                    ))
                ));
                continue;
            }
            let label = fmt(
                if i == j {
                    MATRIX_CELL_SELF
                } else {
                    MATRIX_CELL
                },
                &[
                    ("a", a.title.clone()),
                    ("b", b.title.clone()),
                    ("n", n.to_string()),
                ],
            );
            body.push_str(&format!(
                "<td class=\"{cls}\"><button type=\"button\" tabindex=\"-1\" data-crumb-act=\"flow\" data-crumb-from=\"{i}\" data-crumb-to=\"{j}\" data-crumb-count=\"{n}\" style=\"--m:{}\" aria-label=\"{1}\" title=\"{1}\">{n}</button></td>",
                to_fixed3((f64::from(n) / f64::from(max)).sqrt()),
                escape_html(&label)
            ));
        }
        body.push_str("</tr>");
    }
    let (ba, bb, bn) = best.unwrap_or((0, 0, 0));
    let mut h = format!(
        "<p class=\"crumb-list__intro\">{}</p>",
        escape_html(&fmt(
            MATRIX_INTRO,
            &[("pairs", model.edges.len().to_string())]
        ))
    );
    h.push_str(&format!(
        "<p class=\"crumb-matrix__lede\">{}</p>",
        escape_html(&fmt(
            MATRIX_STRONGEST,
            &[
                ("a", cats[ba].title.clone()),
                ("b", cats[bb].title.clone()),
                ("n", bn.to_string()),
                ("c", cats[sink].title.clone())
            ]
        ))
    ));
    for t in &picked {
        h.push_str(&format!(
            "<p class=\"crumb-matrix__lede crumb-matrix__lede--family\">{}</p>",
            escape_html(t)
        ));
    }
    h.push_str(&format!(
        "<div class=\"crumb-matrix__scroll\"><table class=\"crumb-matrix\"><caption class=\"visually-hidden\">{}</caption><thead>{head}</thead><tbody>{body}</tbody></table></div>",
        escape_html(MATRIX_CAPTION)
    ));
    (h, matrix_from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_href_follows_the_page_s_fingerprints() {
        assert_eq!(
            icon_href(
                Some("/tbaguette-skills/assets/styles.css?v=a9"),
                Some("/tbaguette-skills/assets/icons.svg?v=1a#icon-wheat"),
                "icon-plus"
            ),
            "/tbaguette-skills/assets/icons.svg?v=1a#icon-plus"
        );
        assert_eq!(
            icon_href(None, None, "icon-fit"),
            "/assets/icons.svg#icon-fit"
        );
    }

    #[test]
    fn a_quote_marks_its_first_mention() {
        assert_eq!(
            quote_html(Some("use a & b then a"), "a"),
            "<blockquote class=\"crumb-quote\">use <mark>a</mark> &amp; b then a</blockquote>"
        );
        assert_eq!(quote_html(None, "a"), "");
    }

    #[test]
    fn every_open_in_the_graph_button_names_its_skill() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/graph/graph.json"
        ))
        .expect("docs/graph/graph.json");
        let data = super::super::model::unpack(&text).expect("unpacks");
        let mut seed = 0.0;
        let model = Model::build(&data, &mut || {
            seed += 0.001;
            seed
        });
        let html = list_html(&model, "");
        let buttons = html.matches("data-crumb-act=\"open\"").count();
        assert_eq!(buttons, model.nodes.len());
        for node in &model.nodes {
            let label = format!("aria-label=\"{}: {}\"", LIST_OPEN, node.name);
            assert!(html.contains(&label), "no label for {}", node.name);
        }
    }

    #[test]
    fn a_skill_panel_says_how_many_skills_lead_to_it_within_two_citations() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/graph/graph.json"
        ))
        .expect("docs/graph/graph.json");
        let data = super::super::model::unpack(&text).expect("unpacks");
        let mut seed = 0.0;
        let model = Model::build(&data, &mut || {
            seed += 0.001;
            seed
        });
        for (n, node) in model.nodes.iter().enumerate() {
            let mut direct: Vec<usize> = node.inn.iter().map(|(s, _)| *s).collect();
            direct.sort();
            direct.dedup();
            let mut two = direct.clone();
            for &s in &direct {
                two.extend(model.nodes[s].inn.iter().map(|(t, _)| *t));
            }
            two.sort();
            two.dedup();
            two.retain(|&t| t != n);
            assert_eq!(reach_in_two(&model, n), two.len(), "{}", node.name);
            let html = panel_skill(&model, n, -1, None, "");
            assert_eq!(
                html.contains("crumb-skill__reached"),
                !two.is_empty(),
                "{}",
                node.name
            );
        }
    }
}
