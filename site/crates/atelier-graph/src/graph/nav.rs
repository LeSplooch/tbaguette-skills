//! Views and navigation: switching views, opening a skill, the way back,
//! spotlights and family filters, the path tracer, and putting the panel,
//! trail, legend, list, matrix and tooltip on the page (graph.js `setView`
//! .. `renderMatrix`).

use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement};

use super::input::js_number;
use super::model::{Action, Spot, ask_hits, reach_two, shortest_path};
use super::text::{self, escape_html, fmt};
use super::views;
use super::{
    App, Ev, Hit, Trace, dispatch, document, on_settle, perf_now, prop_f64, set_prop_f64, window,
};

type R = Result<(), JsValue>;

fn replace_state(url: &str) {
    if let Ok(h) = window().and_then(|w| w.history()) {
        let _ = h.replace_state_with_url(&JsValue::NULL, "", Some(url));
    }
}

/// The address with no hash (`location.pathname + location.search`).
fn bare_url() -> String {
    window().map(|w| w.location()).map_or_else(
        |_| String::new(),
        |l| {
            format!(
                "{}{}",
                l.pathname().unwrap_or_default(),
                l.search().unwrap_or_default()
            )
        },
    )
}

fn write_view_hash(view: &str) {
    if view == "constellation" {
        replace_state(&bare_url());
    } else {
        replace_state(&format!("#view={view}"));
    }
}

fn focus_quietly(el: &HtmlElement) {
    let opts = web_sys::FocusOptions::new();
    opts.set_prevent_scroll(true);
    let _ = el.focus_with_options(&opts);
}

impl App {
    pub(crate) fn is_page(&self) -> bool {
        self.view == "list" || self.view == "matrix"
    }

    pub(crate) fn set_view_named(&mut self, view: &str) -> R {
        match ["constellation", "wheel", "matrix", "list"]
            .into_iter()
            .find(|v| *v == view)
        {
            Some(v) => self.set_view(v),
            None => Ok(()),
        }
    }

    pub(crate) fn set_view(&mut self, view: &'static str) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        if self.view != view {
            let line = fmt(
                text::ANNOUNCE_VIEW,
                &[
                    ("view", text::view_name(view).to_string()),
                    ("n", model.nodes.len().to_string()),
                    ("f", model.categories.len().to_string()),
                ],
            );
            self.say(&line);
        }
        let tabs = self.root.query_selector_all("[data-crumb-view]")?;
        for i in 0..tabs.length() {
            if let Some(b) = tabs.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let on = b.get_attribute("data-crumb-view").as_deref() == Some(view);
                b.set_attribute("aria-selected", if on { "true" } else { "false" })?;
            }
        }
        if let Some(list) = self.q("[data-crumb-list]") {
            list.set_hidden(view != "list");
        }
        if let Some(matrix) = self.q("[data-crumb-matrix]") {
            matrix.set_hidden(view != "matrix");
        }
        if view == "list" || view == "matrix" {
            self.view = view;
            if view == "list" {
                self.render_list();
            } else {
                self.render_matrix()?;
            }
            self.root.set_attribute("data-view", view)?;
            write_view_hash(view);
            return Ok(());
        }
        if self.focus.is_some() {
            self.leave_anatomy();
        }
        write_view_hash(view);
        self.overview = view;
        self.view = view;
        self.mix_t = if view == "wheel" { 1.0 } else { 0.0 };
        if view == "wheel" {
            self.layout_wheel();
        }
        self.root.set_attribute("data-view", view)?;
        self.user_cam = false;
        self.fit_camera(false, None);
        self.render_panel()?;
        self.kick();
        Ok(())
    }

    pub(crate) fn open_skill(&mut self, node: usize, from_outside: bool) -> R {
        if self.model.is_none() {
            return Ok(());
        }
        self.clear_emphasis(true)?;
        if self.is_page() {
            if let Some(l) = self.q("[data-crumb-list]") {
                l.set_hidden(true);
            }
            if let Some(m) = self.q("[data-crumb-matrix]") {
                m.set_hidden(true);
            }
        }
        if self.view != "anatomy" {
            self.trail.clear();
            let tabs = self.root.query_selector_all("[data-crumb-view]")?;
            for i in 0..tabs.length() {
                if let Some(b) = tabs.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    b.set_attribute("aria-selected", "false")?;
                }
            }
        }
        if let Some(f) = self.focus
            && f != node
        {
            self.trail.push(f);
        }
        self.trail.retain(|&n| n != node);
        self.focus = Some(node);
        self.view = "anatomy";
        self.pin_section = -1;
        self.hover_section = -1;
        self.hover = None;
        self.kb_node = None;
        self.hide_tip();
        self.root.set_attribute("data-view", "anatomy")?;
        self.layout_anatomy();
        self.user_cam = false;
        self.fit_camera(false, None);
        if from_outside && let Some(m) = self.model.as_mut() {
            m.nodes[node].px = 0.0;
            m.nodes[node].py = 0.0;
        }
        self.ring = f64::min(self.ring, 0.15);
        self.render_trail();
        self.render_panel()?;
        self.dismiss_hint();
        self.announce(Some(node));
        let name = self
            .model
            .as_ref()
            .map(|m| m.nodes[node].name.clone())
            .unwrap_or_default();
        self.set_title(&name);
        let doc = document()?;
        let body: Option<Element> = doc.body().map(Into::into);
        let active = doc.active_element();
        if !from_outside && (active.is_none() || active == body) {
            focus_quietly(&self.stage);
        }
        if from_outside {
            if let Some(name_el) = self.q("[data-crumb-panel] .crumb-skill__name") {
                name_el.set_attribute("tabindex", "-1")?;
                focus_quietly(&name_el);
            }
            let active = doc.active_element();
            if (active.is_none() || active == body || active == doc.document_element())
                && let Some(handle) = self.q("[data-crumb-sheet]")
            {
                focus_quietly(&handle);
            }
        }
        let slug = self
            .model
            .as_ref()
            .map(|m| m.nodes[node].slug.clone())
            .unwrap_or_default();
        replace_state(&format!("#skill={slug}"));
        self.kick();
        Ok(())
    }

    fn set_title(&self, part: &str) {
        let Some(base) = &self.base_title else { return };
        if let Ok(doc) = document() {
            doc.set_title(&if part.is_empty() {
                base.clone()
            } else {
                format!("{part} \u{2014} {base}")
            });
        }
    }

    pub(crate) fn missing_skill(&mut self, asked: &str) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let mut best = None;
        let mut best_d = 4;
        for n in &model.nodes {
            let d = super::model::edit_distance(asked, &n.slug);
            if d < best_d {
                best_d = d;
                best = Some(n.name.clone());
            }
        }
        let msg = format!(
            "There is no skill called \u{201c}{asked}\u{201d}.{}",
            best.map_or(String::new(), |b| format!(" Did you mean {b}?"))
        );
        if let Some(hint) = self.q("[data-crumb-hint]") {
            hint.set_text_content(Some(&msg));
            hint.class_list().remove_1("crumb__hint--gone")?;
        }
        self.say(&msg);
        replace_state(&bare_url());
        Ok(())
    }

    fn leave_anatomy(&mut self) {
        self.set_title("");
        self.focus = None;
        self.trail.clear();
        self.pin_section = -1;
        self.render_trail();
        replace_state(&bare_url());
    }

    /// Step back out of whatever is open; false when there is nothing to leave.
    pub(crate) fn back(&mut self) -> Result<bool, JsValue> {
        self.hide_tip();
        if self.view == "anatomy" {
            if self.pin_section >= 0 {
                self.pin_section_at(-1)?;
                return Ok(true);
            }
            if let Some(prev) = self.trail.pop() {
                self.focus = None;
                let keep = self.trail.clone();
                self.open_skill(prev, false)?;
                self.trail = keep;
                self.render_trail();
                return Ok(true);
            }
            self.set_view(self.overview)?;
            return Ok(true);
        }
        if self.is_page() {
            self.set_view(self.overview)?;
            return Ok(true);
        }
        if self.spot.as_ref().is_some_and(|s| s.kind == "flow") {
            self.clear_emphasis(true)?;
            self.set_view("matrix")?;
            if let Some(cell) = self
                .root
                .query_selector("[data-crumb-matrix] [tabindex=\"0\"]")?
                && let Some(cell) = cell.dyn_ref::<HtmlElement>()
            {
                cell.focus()?;
            }
            return Ok(true);
        }
        if self.trace.is_some() || self.spot.is_some() || self.families.is_some() {
            self.clear_emphasis(false)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn clear_emphasis(&mut self, quiet: bool) -> R {
        // Leaving a walk, its address goes too.
        if self.trace.is_some() && super::hash_trace(&window()?.location().hash()?).is_some() {
            write_view_hash(if self.view == "anatomy" {
                "constellation"
            } else {
                self.view
            });
        }
        self.spot = None;
        self.families = None;
        self.trace = None;
        let on = self
            .root
            .query_selector_all(".crumb-note--on, .crumb-family--on")?;
        for i in 0..on.length() {
            if let Some(b) = on.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                b.class_list()
                    .remove_2("crumb-note--on", "crumb-family--on")?;
                b.set_attribute("aria-pressed", "false")?;
            }
        }
        if !quiet {
            self.user_cam = false;
            self.fit_camera(false, None);
            self.render_panel()?;
        }
        self.kick();
        Ok(())
    }

    pub(crate) fn preview_family(&mut self, on: bool) {
        if self.families.is_some() && !self.families_preview {
            return;
        }
        if on {
            self.families = Some(vec![self.hover_family.clone().unwrap_or_default()]);
            self.families_preview = true;
        } else if self.families_preview {
            self.families = None;
            self.families_preview = false;
        }
        self.kick();
    }

    fn sync_family_chips(&self) -> R {
        let chips = self
            .root
            .query_selector_all("[data-crumb-act=\"family\"]")?;
        for i in 0..chips.length() {
            let Some(b) = chips.item(i).and_then(|n| n.dyn_into::<Element>().ok()) else {
                continue;
            };
            let slug = b.get_attribute("data-crumb-family").unwrap_or_default();
            let on = self.families.as_ref().is_some_and(|f| f.contains(&slug));
            b.class_list().toggle_with_force("crumb-family--on", on)?;
            b.set_attribute("aria-pressed", if on { "true" } else { "false" })?;
        }
        Ok(())
    }

    /// Toggle one family in the chosen set (a preview starts a fresh set).
    fn toggle_family(&mut self, slug: &str) -> bool {
        let mut fam = if self.families_preview {
            Vec::new()
        } else {
            self.families.take().unwrap_or_default()
        };
        self.families_preview = false;
        self.spot = None;
        self.trace = None;
        let added = !fam.iter().any(|s| s == slug);
        if added {
            fam.push(slug.to_string());
        } else {
            fam.retain(|s| s != slug);
        }
        self.families = if fam.is_empty() { None } else { Some(fam) };
        added
    }

    pub(crate) fn act(&mut self, btn: &Element) -> R {
        let attr = |name: &str| btn.get_attribute(name).unwrap_or_default();
        let Some(model) = &self.model else {
            return Ok(());
        };
        match attr("data-crumb-act").as_str() {
            "note" => {
                let index = attr("data-crumb-note");
                let Some(note) = model.notes.get(js_number(&index).max(0) as usize).cloned() else {
                    return Ok(());
                };
                let was_on = btn.class_list().contains("crumb-note--on");
                if self.view != "constellation" && self.view != "wheel" {
                    self.set_view(self.overview)?;
                }
                self.clear_emphasis(true)?;
                if was_on {
                    self.user_cam = false;
                    self.fit_camera(false, None);
                    return self.render_panel();
                }
                match note.action {
                    Action::Trace { from, to } => {
                        let (a, b) = (
                            self.model.as_ref().and_then(|m| m.node(&from)),
                            self.model.as_ref().and_then(|m| m.node(&to)),
                        );
                        if let (Some(a), Some(b)) = (a, b) {
                            self.run_trace(a, b)?;
                        }
                    }
                    Action::Family(cats) => {
                        self.families = Some(cats);
                        self.families_preview = false;
                    }
                    Action::Spot(spot) => self.spot = Some(spot),
                }
                if let Some(target) = self
                    .root
                    .query_selector(&format!("[data-crumb-note=\"{index}\"]"))?
                {
                    target.class_list().add_1("crumb-note--on")?;
                    target.set_attribute("aria-pressed", "true")?;
                }
                self.focus_emphasis();
                self.kick();
            }
            "family" => {
                let slug = attr("data-crumb-family");
                if self.view == "anatomy" || self.is_page() {
                    self.set_view(self.overview)?;
                }
                self.toggle_family(&slug);
                self.sync_family_chips()?;
                self.kick();
            }
            "row" => {
                let slug = attr("data-crumb-row");
                let title = model
                    .categories
                    .iter()
                    .find(|c| c.slug == slug)
                    .map_or_else(|| slug.clone(), |c| c.title.clone());
                let added = self.toggle_family(&slug);
                self.sync_family_chips()?;
                self.matrix_at = None;
                let was = btn.get_bounding_client_rect().top();
                self.render_matrix()?;
                if let (Some(again), Some(matrix)) = (
                    self.root
                        .query_selector(&format!("[data-crumb-row=\"{slug}\"]"))?,
                    self.q("[data-crumb-matrix]"),
                ) {
                    let shift = again.get_bounding_client_rect().top() - was;
                    set_prop_f64(&matrix, "scrollTop", prop_f64(&matrix, "scrollTop") + shift);
                    if let Some(again) = again.dyn_ref::<HtmlElement>() {
                        focus_quietly(again);
                    }
                }
                self.say(&fmt(
                    if added {
                        text::MATRIX_ROW_ON
                    } else {
                        text::MATRIX_ROW_OFF
                    },
                    &[("a", title)],
                ));
                self.kick();
            }
            "flow" => {
                let (from, to) = (
                    js_number(&attr("data-crumb-from")),
                    js_number(&attr("data-crumb-to")),
                );
                let (Some(from), Some(to)) = (
                    model.categories.get(from.max(0) as usize),
                    model.categories.get(to.max(0) as usize),
                ) else {
                    return Ok(());
                };
                let (fi, ti, fs, ts, ft, tt) = (
                    from.index,
                    to.index,
                    from.slug.clone(),
                    to.slug.clone(),
                    from.title.clone(),
                    to.title.clone(),
                );
                let count = attr("data-crumb-count");
                self.matrix_at = Some((fi, ti));
                self.set_view(self.overview)?;
                self.clear_emphasis(true)?;
                self.spot = Some(Spot {
                    kind: "flow",
                    nodes: Vec::new(),
                    dir: None,
                    from: Some(fs),
                    to: Some(ts),
                });
                self.focus_emphasis();
                self.say(&fmt(
                    text::MATRIX_SHOWN,
                    &[("a", ft), ("b", tt), ("n", js_number(&count).to_string())],
                ));
                self.kick();
            }
            "open" => {
                if let Some(n) = model.node(&attr("data-crumb-node")) {
                    self.open_skill(n, false)?;
                }
            }
            "crumb" => {
                let i = js_number(&attr("data-crumb-index"));
                if i < 0 {
                    return self.set_view(self.overview);
                }
                let Some(&target) = self.trail.get(i as usize) else {
                    return Ok(());
                };
                self.trail.truncate(i as usize);
                self.focus = None;
                let keep = self.trail.clone();
                self.open_skill(target, false)?;
                self.trail = keep;
                self.render_trail();
            }
            "section" => {
                let si = js_number(&attr("data-crumb-section"));
                self.pin_section_at(if si == self.pin_section { -1 } else { si })?;
            }
            "reach" => {
                let Some(src) = self.focus else { return Ok(()) };
                let rr = reach_two(model, src);
                let mut nodes = vec![model.nodes[src].slug.clone()];
                nodes.extend(rr.nodes.iter().map(|&n| model.nodes[n].slug.clone()));
                let line = fmt(
                    text::REACH_SHOWN,
                    &[
                        ("name", model.nodes[src].name.clone()),
                        ("n", rr.nodes.len().to_string()),
                    ],
                );
                self.set_view(self.overview)?;
                self.clear_emphasis(true)?;
                self.spot = Some(Spot::nodes(nodes, "pair"));
                self.focus_emphasis();
                self.say(&line);
                self.kick();
            }
            "trace-from" => self.open_trace(self.focus)?,
            "back" => {
                self.back()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn focus_emphasis(&mut self) {
        let Some(emph) = self.compute_emphasis() else {
            return;
        };
        let total = self.model.as_ref().map_or(0, |m| m.nodes.len());
        let list: Vec<usize> = (0..emph.nodes.len())
            .filter(|&i| emph.nodes[i] != 0.0)
            .collect();
        if !list.is_empty() && (list.len() as f64) < total as f64 * 0.9 {
            self.user_cam = true;
            self.fit_camera(false, Some(&list));
        }
    }

    pub(crate) fn open_trace(&mut self, from: Option<usize>) -> R {
        if self.model.is_none() {
            return Ok(());
        }
        if self.view == "anatomy" || self.is_page() {
            self.set_view(self.overview)?;
        }
        self.clear_emphasis(true)?;
        self.trace = Some(Trace {
            from,
            to: None,
            path: None,
            at: 0.0,
        });
        self.render_panel()?;
        self.root.class_list().add_1("crumb--sheet-open")?;
        let sel = if from.is_some() {
            "[data-crumb-trace-to]"
        } else {
            "[data-crumb-trace-from]"
        };
        if let Some(input) = self.panel.query_selector(sel)?
            && let Some(input) = input.dyn_ref::<HtmlElement>()
        {
            input.focus()?;
        }
        Ok(())
    }

    pub(crate) fn run_trace(&mut self, a: usize, b: usize) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let path = shortest_path(model, a, b);
        let line = match &path {
            Some(p) => fmt(
                text::ANNOUNCE_TRACE,
                &[
                    ("a", model.nodes[a].name.clone()),
                    ("b", model.nodes[b].name.clone()),
                    ("n", (p.nodes.len() - 1).to_string()),
                    (
                        "names",
                        p.nodes
                            .iter()
                            .map(|&n| model.nodes[n].name.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                ],
            ),
            None => fmt(
                text::ANNOUNCE_NO_TRACE,
                &[
                    ("a", model.nodes[a].name.clone()),
                    ("b", model.nodes[b].name.clone()),
                ],
            ),
        };
        if path.is_some() {
            replace_state(&format!(
                "#trace={}..{}",
                model.nodes[a].slug, model.nodes[b].slug
            ));
        }
        let path_nodes = path.as_ref().map(|p| p.nodes.clone());
        self.trace = Some(Trace {
            from: Some(a),
            to: Some(b),
            path,
            at: (perf_now() - self.t0) / 1000.0,
        });
        if let Some(nodes) = path_nodes {
            self.user_cam = true;
            self.fit_camera(false, Some(&nodes));
        }
        self.say(&line);
        self.render_panel()?;
        self.kick();
        Ok(())
    }

    /// Copies the page's address, which names the walk: the button says so,
    /// and so does the live region. A browser with no clipboard changes
    /// nothing, and the address bar still has the link.
    pub(crate) fn copy_trace_link(&mut self, button: Element) -> R {
        let w = window()?;
        let url = w.location().href()?;
        let clipboard = Reflect::get(&w.navigator(), &"clipboard".into())?;
        if !clipboard.is_truthy() {
            return Ok(());
        }
        let Ok(write) = Reflect::get(&clipboard, &"writeText".into())?.dyn_into::<Function>()
        else {
            return Ok(());
        };
        let Ok(promise) = write
            .call1(&clipboard, &JsValue::from_str(&url))?
            .dyn_into::<js_sys::Promise>()
        else {
            return Ok(());
        };
        let shared = self.shared();
        on_settle(
            &promise,
            move |_| {
                button.set_text_content(Some(text::COPIED_LINK));
                if let Some(shared) = shared {
                    dispatch(&shared, Ev::TraceLinkCopied);
                }
            },
            |_| {},
        );
        Ok(())
    }

    pub(crate) fn render_legend(&self) {
        if let (Some(legend), Some(model)) = (self.q("[data-crumb-legend]"), &self.model) {
            legend.set_inner_html(&views::legend_html(model));
        }
    }

    pub(crate) fn render_trail(&self) {
        let (Some(nav), Some(key)) = (self.q("[data-crumb-trail]"), self.q("[data-crumb-key]"))
        else {
            return;
        };
        match (self.view, self.focus, &self.model) {
            ("anatomy", Some(focus), Some(model)) => {
                key.set_hidden(false);
                nav.set_inner_html(&views::trail_html(
                    model,
                    &self.trail,
                    focus,
                    &self.icon("icon-back"),
                ));
                nav.set_hidden(false);
            }
            _ => {
                nav.set_hidden(true);
                nav.set_inner_html("");
                key.set_hidden(true);
            }
        }
    }

    pub(crate) fn render_panel(&mut self) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let (Some(body), Some(label)) = (
            self.q("[data-crumb-panel]"),
            self.q("[data-crumb-sheet-label]"),
        ) else {
            return Ok(());
        };
        let html = match (self.view, self.focus, &self.trace) {
            ("anatomy", Some(f), _) => {
                label.set_text_content(Some(&model.nodes[f].name));
                views::panel_skill(
                    model,
                    f,
                    self.pin_section,
                    self.fresh_status(f),
                    &self.icon("icon-route"),
                )
            }
            (_, _, Some(tr)) => {
                label.set_text_content(Some(text::TRACE_TITLE));
                let id = self.q("datalist").map(|d| d.id()).unwrap_or_default();
                views::panel_trace(model, tr.from, tr.to, tr.path.as_ref(), &id)
            }
            _ => {
                label.set_text_content(Some(text::NOTES_TITLE));
                views::panel_overview(model)
            }
        };
        let key = format!(
            "{}:{}:{}",
            self.view,
            self.focus.map_or("", |f| model.nodes[f].slug.as_str()),
            if self.trace.is_some() { "t" } else { "" }
        );
        let keep = if key == self.panel_key {
            prop_f64(&body, "scrollTop")
        } else {
            0.0
        };
        let doc = document()?;
        let held = doc.active_element();
        let body_node: &web_sys::Node = body.as_ref();
        let mut sel: Option<String> = None;
        if let Some(h) = &held {
            let hn: &web_sys::Node = h.as_ref();
            if hn != body_node && body.contains(Some(hn)) {
                let attrs = h.attributes();
                let mut s = String::new();
                for i in 0..attrs.length() {
                    if let Some(a) = attrs.item(i) {
                        let name = a.name();
                        if name.starts_with("data-crumb-") {
                            let value = a.value().replace('\\', "\\\\").replace('"', "\\\"");
                            s.push_str(&format!("[{name}=\"{value}\"]"));
                        }
                    }
                }
                sel = Some(s);
            }
        }
        body.set_inner_html(&html);
        set_prop_f64(&body, "scrollTop", keep);
        self.panel_key = key;
        if let (Some(h), Some(sel)) = (held, sel) {
            let hn: &web_sys::Node = h.as_ref();
            if !body.contains(Some(hn)) {
                let back = if sel.is_empty() {
                    None
                } else {
                    body.query_selector(&format!("{}{sel}", h.tag_name().to_lowercase()))
                        .ok()
                        .flatten()
                };
                match back.and_then(|b| b.dyn_into::<HtmlElement>().ok()) {
                    Some(b) => focus_quietly(&b),
                    None => {
                        body.set_attribute("tabindex", "-1")?;
                        focus_quietly(&body);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn render_list(&self) {
        if let (Some(list), Some(model)) = (self.q("[data-crumb-list]"), &self.model) {
            list.set_inner_html(&views::list_html(model, &self.icon("icon-graph")));
        }
    }

    pub(crate) fn render_matrix(&mut self) -> R {
        let (Some(matrix), Some(model)) = (self.q("[data-crumb-matrix]"), &self.model) else {
            return Ok(());
        };
        let (html, from) = views::matrix_html(model, self.families.as_deref());
        self.matrix_from = from;
        matrix.set_inner_html(&html);
        let cells = matrix.query_selector_all("[data-crumb-act=\"flow\"]")?;
        if cells.length() == 0 {
            return Ok(());
        }
        let current = match self.matrix_at.or(self.matrix_from) {
            Some((a, b)) => matrix
                .query_selector(&format!("[data-crumb-from=\"{a}\"][data-crumb-to=\"{b}\"]"))?,
            None => None,
        };
        let first = current.or_else(|| cells.item(0).and_then(|n| n.dyn_into::<Element>().ok()));
        if let Some(cell) = first.and_then(|c| c.dyn_into::<HtmlElement>().ok()) {
            cell.set_tab_index(0);
        }
        self.matrix_renders += 1;
        Ok(())
    }

    pub(crate) fn ask_library(&mut self, typed: &str) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let q = super::model::js_trim(typed);
        if q.is_empty() {
            return Ok(());
        }
        let hits = ask_hits(model, typed);
        let shown: Vec<usize> = hits.iter().take(12).copied().collect();
        if shown.is_empty() {
            return self.ask_note(&fmt(text::ASK_MISS, &[("q", escape_html(q))]));
        }
        let names = shown
            .iter()
            .take(5)
            .map(|&n| model.nodes[n].name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let template = if hits.len() == 1 {
            text::ASK_HITS_ONE
        } else if hits.len() > 5 {
            text::ASK_HITS_MORE
        } else {
            text::ASK_HITS
        };
        let note = fmt(
            template,
            &[
                ("n", hits.len().to_string()),
                ("q", escape_html(q)),
                ("names", names),
            ],
        );
        let spot_nodes: Vec<String> = shown.iter().map(|&n| model.nodes[n].slug.clone()).collect();
        if self.view != "constellation" && self.view != "wheel" {
            self.set_view(self.overview)?;
        }
        self.clear_emphasis(true)?;
        self.spot = Some(Spot::nodes(spot_nodes, "pair"));
        self.focus_emphasis();
        self.kick();
        self.ask_note(&note)
    }

    pub(crate) fn ask_note(&mut self, html: &str) -> R {
        let Some(note) = self.q("[data-crumb-searchnote]") else {
            return Ok(());
        };
        note.set_inner_html(html);
        note.set_hidden(html.is_empty());
        if !html.is_empty() {
            let said = note.text_content().unwrap_or_default();
            self.say(&said);
        }
        Ok(())
    }

    pub(crate) fn pin_section_at(&mut self, i: i32) -> R {
        self.pin_section = i;
        self.render_panel()?;
        if let Some(head) = self
            .panel
            .query_selector(&format!("[data-crumb-section=\"{i}\"]"))?
            && let Some(li) = head.closest("li")?
        {
            let opts = web_sys::ScrollIntoViewOptions::new();
            opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
            opts.set_behavior(if self.reduced {
                web_sys::ScrollBehavior::Auto
            } else {
                web_sys::ScrollBehavior::Smooth
            });
            li.scroll_into_view_with_scroll_into_view_options(&opts);
        }
        if i >= 0
            && let (Some(model), Some(f)) = (&self.model, self.focus)
            && let Some(s) = model.nodes[f].sections.get(i as usize)
        {
            let line = fmt(
                text::ANNOUNCE_SECTION,
                &[
                    ("n", (i + 1).to_string()),
                    ("title", s.title.clone()),
                    ("words", text::num(s.words)),
                    ("cites", s.refs.len().to_string()),
                ],
            );
            self.say(&line);
        }
        self.kick();
        Ok(())
    }

    pub(crate) fn show_tip(&mut self, hit: &Hit, x: f64, y: f64) -> R {
        let (Some(tip), Some(model)) = (self.q("[data-crumb-tip]"), &self.model) else {
            return Ok(());
        };
        let html = match (hit, self.focus) {
            (Hit::Node(n), Some(f)) if self.view == "anatomy" && *n != f => {
                views::relation_html(model, f, *n)
            }
            (Hit::Node(n), _) => views::tip_node(model, *n),
            (Hit::Section(s), Some(f)) => views::tip_section(model, f, *s),
            (Hit::Hub, Some(f)) => views::tip_hub(model, f),
            _ => String::new(),
        };
        tip.set_inner_html(&html);
        tip.set_hidden(false);
        let (tw, th) = (
            f64::from(tip.offset_width()),
            f64::from(tip.offset_height()),
        );
        let mut left = x + 16.0;
        let mut top = y + 16.0;
        if left + tw > self.w - 8.0 {
            left = x - tw - 16.0;
        }
        if top + th > self.h - 8.0 {
            top = y - th - 16.0;
        }
        tip.style().set_property(
            "transform",
            &format!(
                "translate({}px,{}px)",
                text::num(f64::max(8.0, left)),
                text::num(f64::max(8.0, top))
            ),
        )?;
        Ok(())
    }

    pub(crate) fn hide_tip(&self) {
        if let Some(tip) = self.q("[data-crumb-tip]") {
            tip.set_hidden(true);
        }
    }
}
