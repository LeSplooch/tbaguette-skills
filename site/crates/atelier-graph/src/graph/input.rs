//! Input: pointers on the stage (pan, node drag, pinch, the touch two-tap),
//! the wheel, the keyboard (stage, root and document levels), and the
//! delegated clicks, hovers and submits on the app's own DOM (graph.js
//! `bindShell` .. `onKey`).

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Element, Event, HtmlElement, HtmlInputElement, KeyboardEvent, MouseEvent, PointerEvent,
    WheelEvent,
};

use super::layout::{clamp, js_hypot};
use super::model::{find_skill, shortest_path};
use super::{App, Drag, Hit, Pinch, Ptr, client_xy};

type R = Result<(), JsValue>;

pub(crate) fn closest(e: &Event, sel: &str) -> Option<Element> {
    let target: Element = e.target()?.dyn_into().ok()?;
    target.closest(sel).ok().flatten()
}

fn target_tag(e: &Event) -> String {
    e.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .map(|t| t.tag_name())
        .unwrap_or_default()
}

impl App {
    /// Open or close the phone sheet, keeping the handle's `aria-expanded` in step with the class.
    pub(super) fn set_sheet(&self, open: bool) -> Result<(), JsValue> {
        self.root
            .class_list()
            .toggle_with_force("crumb--sheet-open", open)?;
        if let Some(handle) = self.root.query_selector("[data-crumb-sheet]")? {
            handle.set_attribute("aria-expanded", if open { "true" } else { "false" })?;
        }
        Ok(())
    }

    fn local_point(&self, e: &MouseEvent) -> (f64, f64) {
        let rect = self.stage.get_bounding_client_rect();
        let (x, y) = client_xy(e);
        (x - rect.left(), y - rect.top())
    }

    fn target_is_canvas(&self, e: &Event) -> bool {
        e.target().is_some_and(|t| {
            let canvas: &JsValue = self.canvas.as_ref();
            let t: &JsValue = t.as_ref();
            t == canvas
        })
    }

    pub(crate) fn on_pointer_down(&mut self, e: &PointerEvent) -> R {
        if self.model.is_none() || self.is_page() || !self.target_is_canvas(e) {
            return Ok(());
        }
        let p = self.local_point(e);
        let id = e.pointer_id();
        self.pointers.insert(
            id,
            Ptr {
                x: p.0,
                y: p.1,
                x0: p.0,
                y0: p.1,
            },
        );
        // graph.js lets a failed capture throw out of the handler here.
        self.stage.set_pointer_capture(id)?;
        if self.pointers.len() == 2 {
            let mut it = self.pointers.values();
            let (a, b) = (
                it.next().unwrap_or(&Ptr {
                    x: 0.0,
                    y: 0.0,
                    x0: 0.0,
                    y0: 0.0,
                }),
                it.next().unwrap_or(&Ptr {
                    x: 0.0,
                    y: 0.0,
                    x0: 0.0,
                    y0: 0.0,
                }),
            );
            let (d, mx, my) = (
                js_hypot(a.x - b.x, a.y - b.y),
                (a.x + b.x) / 2.0,
                (a.y + b.y) / 2.0,
            );
            self.pinch = Some(Pinch {
                d,
                k: self.cam_t.k,
                w: self.to_world(mx, my),
            });
            self.drag = None;
            return Ok(());
        }
        let hit = self.pick(p.0, p.1);
        let pan = Drag {
            node: None,
            pan: true,
            cx: self.cam_t.x,
            cy: self.cam_t.y,
            moved: false,
        };
        match hit {
            Some(Hit::Node(n)) if self.view == "constellation" && e.pointer_type() == "touch" => {
                self.drag = Some(pan);
                self.start_hold(id, n);
            }
            Some(Hit::Node(n)) if self.view == "constellation" => {
                self.drag = Some(Drag {
                    node: Some(n),
                    pan: false,
                    cx: 0.0,
                    cy: 0.0,
                    moved: false,
                });
            }
            _ => self.drag = Some(pan),
        }
        self.dismiss_hint();
        Ok(())
    }

    pub(crate) fn on_pointer_move(&mut self, e: &PointerEvent) -> R {
        if self.model.is_none() || self.is_page() {
            return Ok(());
        }
        let p = self.local_point(e);
        let id = e.pointer_id();
        let ptr0 = self.pointers.get_mut(&id).map(|ptr| {
            ptr.x = p.0;
            ptr.y = p.1;
            (ptr.x0, ptr.y0)
        });
        if self.pointers.len() == 2
            && let Some(pinch) = &self.pinch
        {
            let mut it = self.pointers.values();
            let (Some(a), Some(b)) = (it.next(), it.next()) else {
                return Ok(());
            };
            let d = js_hypot(a.x - b.x, a.y - b.y);
            let (mx, my) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
            let pk = clamp(pinch.k * d / pinch.d, 0.2, 5.0);
            let w = pinch.w;
            self.cam_t.k = pk;
            self.cam_t.x = w.0 - (mx - self.w / 2.0) / pk;
            self.cam_t.y = w.1 - (my - self.h / 2.0) / pk;
            self.user_cam = true;
            if self.reduced {
                self.cam = self.cam_t;
            }
            self.kick();
            return Ok(());
        }
        if let (Some(drag), Some((x0, y0))) = (self.drag.as_mut(), ptr0) {
            let (dx, dy) = (p.0 - x0, p.1 - y0);
            let mut hide = false;
            if !drag.moved && js_hypot(dx, dy) > 5.0 {
                drag.moved = true;
                hide = true;
            }
            let (moved, node, pan, cx, cy) = (drag.moved, drag.node, drag.pan, drag.cx, drag.cy);
            if hide {
                self.hide_tip();
            }
            if moved {
                if let Some(n) = node {
                    let w = self.to_world(p.0, p.1);
                    if let Some(model) = self.model.as_mut() {
                        let node = &mut model.nodes[n];
                        node.fx = Some(w.0);
                        node.fy = Some(w.1);
                        node.x = w.0;
                        node.y = w.1;
                    }
                    if let Some(sim) = self.sim.as_mut() {
                        sim.alpha_target = 0.28;
                        sim.reheat(0.28);
                    }
                    self.stage.class_list().add_1("crumb__stage--grabbing")?;
                } else if pan {
                    self.cam_t.x = cx - dx / self.cam.k;
                    self.cam_t.y = cy - dy / self.cam.k;
                    self.cam.x = self.cam_t.x;
                    self.cam.y = self.cam_t.y;
                    self.user_cam = true;
                    self.stage.class_list().add_1("crumb__stage--grabbing")?;
                }
                self.kick();
                return Ok(());
            }
        }
        if e.pointer_type() != "touch" {
            let hit = self.pick(p.0, p.1);
            let (node, section) = match &hit {
                Some(Hit::Node(n)) => (Some(*n), -1),
                Some(Hit::Hub) => (self.focus, -1),
                Some(Hit::Section(s)) => (None, *s as i32),
                None => (None, -1),
            };
            self.set_hover(node, section);
            let _ = self
                .stage
                .style()
                .set_property("cursor", if hit.is_some() { "pointer" } else { "" });
            match hit {
                Some(h) => self.show_tip(&h, p.0, p.1)?,
                None => self.hide_tip(),
            }
        }
        Ok(())
    }

    pub(crate) fn on_pointer_up(&mut self, e: &PointerEvent, cancelled: bool) -> R {
        let id = e.pointer_id();
        let had = self.pointers.remove(&id).is_some();
        if self.pointers.len() < 2 {
            self.pinch = None;
        }
        self.stage.class_list().remove_1("crumb__stage--grabbing")?;
        self.cancel_hold();
        let drag = self.drag.take();
        let Some(drag) = drag else { return Ok(()) };
        if !had || cancelled {
            return Ok(());
        }
        if let Some(n) = drag.node {
            if let Some(model) = self.model.as_mut() {
                model.nodes[n].fx = None;
                model.nodes[n].fy = None;
            }
            if let Some(sim) = self.sim.as_mut() {
                sim.alpha_target = 0.0;
            }
        }
        if drag.moved {
            return Ok(());
        }
        let p = self.local_point(e);
        let hit = self.pick(p.0, p.1);
        if let Some(Hit::Node(n)) = hit
            && e.pointer_type() == "touch"
            && self.touch_armed != Some(n)
            && self.view != "anatomy"
        {
            self.touch_armed = Some(n);
            self.set_hover(Some(n), -1);
            self.show_tip(&Hit::Node(n), p.0, p.1)?;
            return Ok(());
        }
        self.touch_armed = None;
        self.on_tap(hit)
    }

    fn on_tap(&mut self, hit: Option<Hit>) -> R {
        self.hide_tip();
        match hit {
            None => {
                if self.view == "anatomy" {
                    if self.pin_section >= 0 {
                        self.pin_section = -1;
                        self.render_panel()?;
                        self.kick();
                    }
                } else if self.spot.is_some() || self.families.is_some() || self.trace.is_some() {
                    self.clear_emphasis(false)?;
                }
                self.set_hover(None, -1);
                Ok(())
            }
            Some(Hit::Node(n)) => self.open_skill(n, false),
            Some(Hit::Section(s)) => self.pin_section_at(if s as i32 == self.pin_section {
                -1
            } else {
                s as i32
            }),
            Some(Hit::Hub) => Ok(()),
        }
    }

    pub(crate) fn set_hover(&mut self, node: Option<usize>, section: i32) {
        if self.hover == node && self.hover_section == section {
            return;
        }
        self.hover = node;
        self.hover_section = section;
        self.kick();
    }

    pub(crate) fn on_wheel(&mut self, e: &WheelEvent) -> R {
        if self.model.is_none() || self.is_page() {
            return Ok(());
        }
        e.prevent_default();
        let rect = self.stage.get_bounding_client_rect();
        let factor = (-e.delta_y() * if e.ctrl_key() { 0.01 } else { 0.0018 }).exp();
        let (x, y) = client_xy(e);
        self.zoom_at(x - rect.left(), y - rect.top(), factor);
        self.dismiss_hint();
        Ok(())
    }

    pub(crate) fn on_dblclick(&mut self, e: &MouseEvent) -> R {
        if !self.target_is_canvas(e) {
            return Ok(());
        }
        let rect = self.stage.get_bounding_client_rect();
        let (x, y) = client_xy(e);
        if self.pick(x - rect.left(), y - rect.top()).is_none() {
            self.user_cam = false;
            self.fit_camera(false, None);
        }
        Ok(())
    }

    /// The stage's own keys: arrows walk, Enter or Space opens, Backspace
    /// steps back in the anatomy.
    pub(crate) fn on_key(&mut self, e: &KeyboardEvent) -> R {
        if self.model.is_none() || self.is_page() {
            return Ok(());
        }
        let key = e.key();
        if key == "Escape" {
            return Ok(());
        }
        if self.view == "anatomy" {
            let list: Vec<usize> = self
                .anat
                .as_ref()
                .map_or_else(Vec::new, |l| l.neighbors.iter().map(|i| i.node).collect());
            match key.as_str() {
                "ArrowLeft" | "ArrowRight" => {
                    e.prevent_default();
                    let len = list.len() as i64;
                    if len == 0 {
                        self.kb_node = None;
                    } else {
                        let idx = list
                            .iter()
                            .position(|&n| Some(n) == self.kb_node)
                            .map_or(-1, |i| i as i64);
                        let idx = (idx + if key == "ArrowRight" { 1 } else { -1 } + len) % len;
                        self.kb_node = list.get(idx as usize).copied();
                    }
                    self.hover = self.kb_node;
                    self.announce(self.kb_node);
                }
                "ArrowUp" | "ArrowDown" => {
                    e.prevent_default();
                    let n = self
                        .focus
                        .and_then(|f| self.model.as_ref().map(|m| m.nodes[f].sections.len()))
                        .unwrap_or(0) as i32;
                    if n > 0 {
                        let down = key == "ArrowDown";
                        let cur = if self.pin_section < 0 {
                            if down { -1 } else { 0 }
                        } else {
                            self.pin_section
                        };
                        self.pin_section_at((cur + if down { 1 } else { -1 } + n) % n)?;
                    }
                }
                "Enter" | " " if self.kb_node.is_some() => {
                    e.prevent_default();
                    if let Some(kb) = self.kb_node {
                        self.open_skill(kb, false)?;
                    }
                }
                "Backspace" => {
                    e.prevent_default();
                    self.back()?;
                }
                _ => {}
            }
            self.kick();
            return Ok(());
        }
        let dir = match key.as_str() {
            "ArrowLeft" => Some((-1.0, 0.0)),
            "ArrowRight" => Some((1.0, 0.0)),
            "ArrowUp" => Some((0.0, -1.0)),
            "ArrowDown" => Some((0.0, 1.0)),
            _ => None,
        };
        if let Some(d) = dir {
            e.prevent_default();
            let from = self
                .kb_node
                .or_else(|| self.label_priority.first().copied());
            if let (Some(from), Some(model)) = (from, &self.model) {
                let f = &model.nodes[from];
                let mut best = None;
                let mut best_score = f64::INFINITY;
                for (i, n) in model.nodes.iter().enumerate() {
                    if i == from || n.pa < 0.3 {
                        continue;
                    }
                    let (dx, dy) = (n.sx - f.sx, n.sy - f.sy);
                    let along = dx * d.0 + dy * d.1;
                    if along <= 2.0 {
                        continue;
                    }
                    let across = (dx * d.1 - dy * d.0).abs();
                    let score = along + across * 2.2;
                    if score < best_score {
                        best_score = score;
                        best = Some(i);
                    }
                }
                if best.is_some() {
                    self.kb_node = best;
                    self.announce(best);
                }
            }
            self.kick();
        } else if (key == "Enter" || key == " ") && self.kb_node.is_some() {
            e.prevent_default();
            if let Some(kb) = self.kb_node {
                self.open_skill(kb, false)?;
            }
        }
        Ok(())
    }

    /// '/' anywhere in the app focuses the search, unless typing already.
    pub(crate) fn on_root_key(&mut self, e: &KeyboardEvent) -> R {
        let tag = target_tag(e).to_lowercase();
        if e.key() == "/"
            && !(tag.contains("input") || tag.contains("select") || tag.contains("textarea"))
        {
            e.prevent_default();
            if let Some(input) = self.q("[data-crumb-search] input") {
                input.focus()?;
            }
        }
        Ok(())
    }

    /// Escape, from anywhere on the page: clear a field first, else step back.
    pub(crate) fn on_doc_key(&mut self, e: &KeyboardEvent) -> R {
        if e.key() != "Escape" || e.default_prevented() {
            return Ok(());
        }
        if target_tag(e).to_lowercase().contains("input")
            && let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && !input.value().is_empty()
        {
            input.set_value("");
            self.ask_note("")?;
            e.prevent_default();
            return Ok(());
        }
        if self.back()? {
            e.prevent_default();
        }
        Ok(())
    }

    pub(crate) fn on_click(&mut self, e: &Event) -> R {
        // The trace form's own buttons (graph.js listens on them directly;
        // they run before the app's delegated click, which they never match).
        if closest(e, "[data-crumb-trace-random]").is_some() && self.panel_contains(e) {
            return self.trace_random();
        }
        if closest(e, "[data-crumb-trace-done]").is_some() && self.panel_contains(e) {
            return self.clear_emphasis(false);
        }
        if let Some(copy) = closest(e, "[data-crumb-trace-copy]")
            && self.panel_contains(e)
        {
            return self.copy_trace_link(copy);
        }
        if let Some(v) = closest(e, "[data-crumb-view]") {
            let view = v.get_attribute("data-crumb-view").unwrap_or_default();
            return self.set_view_named(&view);
        }
        if closest(e, "[data-crumb-trace]").is_some() {
            return self.open_trace(None);
        }
        if let Some(z) = closest(e, "[data-crumb-zoom]") {
            match z.get_attribute("data-crumb-zoom").as_deref() {
                Some("fit") => {
                    self.user_cam = false;
                    self.fit_camera(false, None);
                }
                how => self.zoom_at(
                    self.w / 2.0,
                    self.h / 2.0,
                    if how == Some("in") { 1.35 } else { 1.0 / 1.35 },
                ),
            }
            return Ok(());
        }
        if closest(e, "[data-crumb-sheet]").is_some() {
            let open = !self.root.class_list().contains("crumb--sheet-open");
            return self.set_sheet(open);
        }
        if let Some(act) = closest(e, "[data-crumb-act]") {
            return self.act(&act);
        }
        Ok(())
    }

    fn panel_contains(&self, e: &Event) -> bool {
        e.target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
            .is_some_and(|n| self.panel.contains(Some(&n)))
    }

    pub(crate) fn on_mouse_over(&mut self, e: &Event) -> R {
        if let Some(fam) = closest(e, "[data-crumb-family]")
            && self.spot.is_none()
            && self.trace.is_none()
        {
            self.hover_family = fam.get_attribute("data-crumb-family");
            self.preview_family(true);
        }
        if let Some(sec) = closest(e, "[data-crumb-section]")
            && self.view == "anatomy"
        {
            self.hover_section =
                js_number(&sec.get_attribute("data-crumb-section").unwrap_or_default());
            self.kick();
        }
        if let Some(nb) = closest(e, "[data-crumb-node]")
            && let Some(model) = &self.model
        {
            self.hover = model.node(&nb.get_attribute("data-crumb-node").unwrap_or_default());
            self.kick();
        }
        Ok(())
    }

    pub(crate) fn on_mouse_out(&mut self, e: &Event) -> R {
        if closest(e, "[data-crumb-family]").is_some() && self.hover_family.is_some() {
            self.hover_family = None;
            self.preview_family(false);
        }
        if closest(e, "[data-crumb-section]").is_some() {
            self.hover_section = -1;
            self.kick();
        }
        if closest(e, "[data-crumb-node]").is_some() {
            self.hover = None;
            self.kick();
        }
        Ok(())
    }

    pub(crate) fn on_submit(&mut self, e: &Event) -> R {
        let Some(form) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return Ok(());
        };
        if form.has_attribute("data-crumb-search") {
            e.prevent_default();
            let Some(input) = form
                .query_selector("input")?
                .and_then(|i| i.dyn_into::<HtmlInputElement>().ok())
            else {
                return Ok(());
            };
            let found = self
                .model
                .as_ref()
                .and_then(|m| find_skill(m, &input.value()));
            if let Some(n) = found {
                input.set_value("");
                input.blur()?;
                return self.open_skill(n, false);
            }
            return self.ask_library(&input.value());
        }
        if form.has_attribute("data-crumb-trace-form") {
            e.prevent_default();
            let field = |sel: &str| {
                form.query_selector(sel)
                    .ok()
                    .flatten()
                    .and_then(|i| i.dyn_into::<HtmlInputElement>().ok())
            };
            let (Some(from), Some(to), Some(err)) = (
                field("[data-crumb-trace-from]"),
                field("[data-crumb-trace-to]"),
                form.query_selector("[data-crumb-trace-error]")?,
            ) else {
                return Ok(());
            };
            let Some(model) = &self.model else {
                return Ok(());
            };
            let (a, b) = (
                find_skill(model, &from.value()),
                find_skill(model, &to.value()),
            );
            match (a, b) {
                (Some(a), Some(b)) if a == b => {
                    err.set_text_content(Some(super::text::TRACE_SAME));
                    to.focus()?;
                }
                (Some(a), Some(b)) => {
                    err.set_text_content(Some(""));
                    self.run_trace(a, b)?;
                }
                (a, _) => {
                    err.set_text_content(Some(super::text::TRACE_UNKNOWN));
                    if a.is_some() {
                        to.focus()?
                    } else {
                        from.focus()?
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn on_search_input(&mut self, e: &Event) -> R {
        self.ask_note("")?;
        let Some(input) = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return Ok(());
        };
        let n = self
            .model
            .as_ref()
            .and_then(|m| m.node(super::model::js_trim(&input.value())));
        if let Some(n) = n {
            input.set_value("");
            self.open_skill(n, false)?;
        }
        Ok(())
    }

    /// "Surprise me": a random pair at least three steps apart, within sixty tries.
    fn trace_random(&mut self) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let len = model.nodes.len() as f64;
        let (mut a, mut b) = (0, 0);
        for _ in 0..60 {
            a = (js_sys::Math::random() * len).floor() as usize;
            b = (js_sys::Math::random() * len).floor() as usize;
            if a == b {
                continue;
            }
            if shortest_path(model, a, b).is_some_and(|p| p.edges.len() >= 3) {
                break;
            }
        }
        self.run_trace(a, b)
    }

    pub(crate) fn on_matrix_key(&mut self, e: &KeyboardEvent) -> R {
        let Some(b) = closest(e, "[data-crumb-act=\"flow\"]") else {
            return Ok(());
        };
        let key = e.key();
        if key == "Escape" {
            e.prevent_default();
            // One listener per render in graph.js, each stepping back.
            for _ in 0..self.matrix_renders {
                self.back()?;
            }
            return Ok(());
        }
        let Some(matrix) = self.q("[data-crumb-matrix]") else {
            return Ok(());
        };
        let cells = matrix.query_selector_all("[data-crumb-act=\"flow\"]")?;
        let at = |el: &Element| {
            (
                js_number(&el.get_attribute("data-crumb-from").unwrap_or_default()),
                js_number(&el.get_attribute("data-crumb-to").unwrap_or_default()),
            )
        };
        let p = at(&b);
        let (dr, dc, edge) = match key.as_str() {
            "ArrowRight" => (0, 1, ""),
            "ArrowLeft" => (0, -1, ""),
            "ArrowDown" => (1, 0, ""),
            "ArrowUp" => (-1, 0, ""),
            "Home" => (0, 0, "home"),
            "End" => (0, 0, "end"),
            _ => return Ok(()),
        };
        let mut best: Option<HtmlElement> = None;
        let mut best_d = f64::INFINITY;
        for i in 0..cells.length() {
            let Some(c) = cells.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else {
                continue;
            };
            let q = at(&c);
            let d = match edge {
                "home" => {
                    if q.0 == p.0 {
                        f64::from(q.1)
                    } else {
                        f64::INFINITY
                    }
                }
                "end" => {
                    if q.0 == p.0 {
                        -f64::from(q.1)
                    } else {
                        f64::INFINITY
                    }
                }
                _ => {
                    let along = if dr != 0 {
                        (q.0 - p.0) * dr
                    } else {
                        (q.1 - p.1) * dc
                    };
                    let across = if dr != 0 {
                        (q.1 - p.1).abs()
                    } else {
                        (q.0 - p.0).abs()
                    };
                    if along <= 0 {
                        continue;
                    }
                    f64::from(along * 100 + across)
                }
            };
            if d < best_d {
                best_d = d;
                best = Some(c);
            }
        }
        e.prevent_default();
        let Some(next) = best else { return Ok(()) };
        if let Some(b) = b.dyn_ref::<HtmlElement>() {
            b.set_tab_index(-1);
        }
        next.set_tab_index(0);
        next.focus()?;
        let q = at(&next);
        self.matrix_at = Some((q.0.max(0) as usize, q.1.max(0) as usize));
        Ok(())
    }

    pub(crate) fn on_matrix_focus_in(&mut self, e: &Event) -> R {
        if let Some(b) = closest(e, "[data-crumb-act=\"flow\"]") {
            let n = |a: &str| js_number(&b.get_attribute(a).unwrap_or_default()).max(0) as usize;
            self.matrix_at = Some((n("data-crumb-from"), n("data-crumb-to")));
        }
        Ok(())
    }
}

/// `+attribute` for the small integers these attributes carry.
pub(crate) fn js_number(s: &str) -> i32 {
    super::model::js_trim(s).parse().unwrap_or(0)
}
