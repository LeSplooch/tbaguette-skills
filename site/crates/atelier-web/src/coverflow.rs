//! The "Fresh from the oven" coverflow (site.js initFreshCoverflow).
//!
//! The server renders every tile with its `--cf-offset`; this is the clock
//! that moves which tile is centred every 3 s, plus prev/next, focus
//! recentring, and click-and-drag with release momentum. Off (the server's
//! static fan stays) with fewer than two tiles, under reduced motion, below
//! 640 px, or in forced colours, each checked once, mirroring the CSS gate.
//!
//! Two traps the code below keeps out of: pointer capture is taken only once
//! a press has moved far enough to be a drag (capturing at pointerdown
//! retargets the click away from the tile's link and silently kills it), and
//! only the click a drag leaves behind is swallowed, never one with detail 0
//! (a keyboard Enter).

use crate::dom::{document, html, js_num, js_round, media, num, on, prefers_reduced_motion, select_within, set_timeout, window};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, FocusEvent, HtmlElement, MouseEvent, PointerEvent};

const STEP_MS: i32 = 3000;
const DRAG_STEP_PX: f64 = 140.0;
const DRAG_CLICK_THRESHOLD_PX: f64 = 6.0;
const VELOCITY_WINDOW_MS: f64 = 100.0;
const MOMENTUM_TIME_CONSTANT_MS: f64 = 260.0;
const MAX_MOMENTUM_TILES: f64 = 5.0;
const STALL_MS: f64 = 60.0;
const SPIN_DURATION_PER_TILE_S: f64 = 0.18;
const MAX_SPIN_DURATION_S: f64 = 1.8;
const BASE_SPIN_DURATION_S: f64 = 0.7;

/// templates.py's `_fresh_signed_offset`, restated: the signed circular
/// distance of tile `index` from the centred position (fractional mid-drag).
fn signed_offset(index: f64, active: f64, count: f64) -> f64 {
    if count <= 1.0 {
        return 0.0;
    }
    let diff = ((index - active) % count + count) % count;
    if diff > count / 2.0 { diff - count } else { diff }
}

/// templates.py's `_fresh_group_shift_px`: depends only on the count.
fn group_shift_px(count: usize) -> f64 {
    if count <= 1 {
        return 0.0;
    }
    let half = (count / 2) as f64;
    let hi = half.min(2.0);
    let lo = (-((count as f64) - 1.0 - half)).max(-2.0);
    js_round(-(lo + hi) / 2.0 * 140.0)
}

#[derive(Default)]
struct Drag {
    active: f64,
    hovering: bool,
    pressing: bool,
    manual: bool,
    pointer_down: bool,
    moved: bool,
    pointer_id: Option<i32>,
    start_x: f64,
    start_index: f64,
    samples: Vec<(f64, f64)>,
}

struct Rail {
    rail: Element,
    tiles: Vec<HtmlElement>,
    count: f64,
    group_shift: f64,
    st: RefCell<Drag>,
}

/// A tile's own opacity at this distance from the centre: whole out to two
/// either side, gone by three (landing.css's rule for the same thing).
fn card_opacity(abs: f64) -> f64 {
    ((3.0 - abs) * 2.0).clamp(0.0, 1.0)
}

/// How far its content has dimmed at this distance.
fn content_dim(abs: f64) -> f64 {
    (1.0 - abs * abs * 0.115).clamp(0.0, 1.0)
}

impl Rail {
    fn paused(&self) -> bool {
        let s = self.st.borrow();
        s.hovering || s.pressing || s.manual
    }

    /// Writes `--cf-offset` and, redundantly on purpose, plain transform and
    /// opacity (transitions between plain values animate everywhere). The
    /// card stays opaque across the fan so a neighbour's text never shows
    /// through it; its content dims with distance (`--cf-dim`, landing.css).
    fn layout(&self, index: f64) {
        for (i, tile) in self.tiles.iter().enumerate() {
            let offset = signed_offset(i as f64, index, self.count);
            let style = tile.style();
            let _ = style.set_property("--cf-offset", &js_num(offset));
            let abs = offset.abs();
            let x = (-280f64).max(280f64.min(offset * 140.0)) + self.group_shift;
            let rotate = (-38f64).max(38f64.min(offset * -34.0));
            let scale = 0.55f64.max(1.0 - abs * 0.13);
            let transform = format!("translateX({}px) rotateY({}deg) scale({})", js_num(x), js_num(rotate), js_num(scale));
            let _ = style.set_property("transform", &transform);
            let _ = style.set_property("opacity", &js_num(card_opacity(abs)));
            let _ = style.set_property("--cf-dim", &js_num(content_dim(abs)));
        }
    }

    fn step(&self) {
        if self.paused() {
            return;
        }
        let next = {
            let mut s = self.st.borrow_mut();
            s.active = (s.active + 1.0) % self.count;
            s.active
        };
        self.layout(next);
    }

    fn go_to(&self, index: f64) {
        let next = {
            let mut s = self.st.borrow_mut();
            s.manual = true;
            s.active = (index % self.count + self.count) % self.count;
            s.active
        };
        self.layout(next);
    }

    fn pointer_down(&self, e: &PointerEvent) {
        if e.button() != 0 {
            return;
        }
        let mut s = self.st.borrow_mut();
        let x = num(e, "clientX");
        s.pointer_down = true;
        s.pressing = true;
        s.moved = false;
        s.pointer_id = Some(e.pointer_id());
        s.start_x = x;
        s.start_index = s.active;
        s.samples = vec![(e.time_stamp(), x)];
        // No setPointerCapture here: see the module comment.
    }

    fn pointer_move(&self, e: &PointerEvent) {
        let index = {
            let mut s = self.st.borrow_mut();
            if !s.pointer_down || s.pointer_id != Some(e.pointer_id()) {
                return;
            }
            let x = num(e, "clientX");
            let delta = x - s.start_x;
            if !s.moved {
                if delta.abs() < DRAG_CLICK_THRESHOLD_PX {
                    return;
                }
                s.moved = true;
                s.manual = true;
                // The first instant this is known to be a drag.
                let _ = self.rail.set_pointer_capture(e.pointer_id());
                for tile in &self.tiles {
                    let _ = tile.style().set_property_with_priority("transition", "none", "important");
                }
            }
            let t = e.time_stamp();
            s.samples.push((t, x));
            while s.samples.len() > 2 && t - s.samples[0].0 > VELOCITY_WINDOW_MS {
                s.samples.remove(0);
            }
            // Dragging left steps forward, like next and the clock.
            s.start_index - delta / DRAG_STEP_PX
        };
        self.layout(index);
    }

    fn end_drag(self: &Rc<Self>, e: &PointerEvent) {
        let (index, spin) = {
            let mut s = self.st.borrow_mut();
            if !s.pointer_down || s.pointer_id != Some(e.pointer_id()) {
                return;
            }
            s.pointer_down = false;
            s.pressing = false;
            let _ = self.rail.release_pointer_capture(e.pointer_id());
            if !s.moved {
                return;
            }
            // No click follows a cancel: hand the suppression back now, or
            // the next keyboard Enter would be swallowed.
            if e.type_() == "pointercancel" {
                s.moved = false;
            }
            let delta = num(e, "clientX") - s.start_x;
            let live = s.start_index - delta / DRAG_STEP_PX;
            let (oldest, newest) = (s.samples[0], s.samples[s.samples.len() - 1]);
            let elapsed = newest.0 - oldest.0;
            let stalled = e.time_stamp() - newest.0 > STALL_MS;
            let velocity = if !stalled && elapsed > 0.0 { -(newest.1 - oldest.1) / elapsed / DRAG_STEP_PX } else { 0.0 };
            let momentum = (-MAX_MOMENTUM_TILES).max(MAX_MOMENTUM_TILES.min(velocity * MOMENTUM_TIME_CONSTANT_MS));
            let spin = MAX_SPIN_DURATION_S.min(BASE_SPIN_DURATION_S + momentum.abs() * SPIN_DURATION_PER_TILE_S);
            s.active = (js_round(live + momentum) % self.count + self.count) % self.count;
            (s.active, spin)
        };
        for tile in &self.tiles {
            let style = tile.style();
            let _ = style.remove_property("transition");
            let _ = style.set_property("--fresh-spin-duration", &format!("{}s", js_num(spin)));
        }
        self.layout(index);
        let rail = self.clone();
        set_timeout(
            move || {
                for tile in &rail.tiles {
                    let _ = tile.style().remove_property("--fresh-spin-duration");
                }
            },
            spin * 1000.0 + 50.0,
        );
    }

    fn click(&self, e: &MouseEvent) {
        let mut s = self.st.borrow_mut();
        if !s.moved || e.detail() == 0 {
            return;
        }
        s.moved = false;
        e.prevent_default();
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(rail_el) = doc.query_selector("[data-fresh-coverflow]")? else { return Ok(()) };
    let tiles: Vec<HtmlElement> = select_within(&rail_el, ".fresh__tile")?.iter().map(html).collect();
    if tiles.len() < 2 {
        return Ok(());
    }
    if prefers_reduced_motion() || media("(max-width: 639px)") || media("(forced-colors: active)") {
        return Ok(());
    }
    let rail = Rc::new(Rail {
        rail: rail_el.clone(),
        count: tiles.len() as f64,
        group_shift: group_shift_px(tiles.len()),
        tiles,
        st: RefCell::new(Drag::default()),
    });

    let r = rail.clone();
    on(&rail_el, "mouseenter", move |_: Event| r.st.borrow_mut().hovering = true)?;
    let r = rail.clone();
    on(&rail_el, "mouseleave", move |_: Event| r.st.borrow_mut().hovering = false)?;
    let r = rail.clone();
    on(&rail_el, "focusin", move |e: FocusEvent| {
        r.st.borrow_mut().hovering = true;
        let target: Option<JsValue> = e.target().map(Into::into);
        let index = r.tiles.iter().position(|t| Some(JsValue::from(t)) == target);
        if let Some(i) = index {
            let changed = {
                let mut s = r.st.borrow_mut();
                let changed = i as f64 != s.active;
                if changed {
                    s.active = i as f64;
                }
                changed
            };
            if changed {
                r.layout(i as f64);
            }
        }
    })?;
    let r = rail.clone();
    on(&rail_el, "focusout", move |_: Event| r.st.borrow_mut().hovering = false)?;

    if let Some(nav) = doc.query_selector("[data-fresh-nav]")? {
        if let Some(prev) = nav.query_selector("[data-fresh-prev]")? {
            let r = rail.clone();
            on(&prev, "click", move |_: Event| {
                let at = r.st.borrow().active;
                r.go_to(at - 1.0);
            })?;
        }
        if let Some(next) = nav.query_selector("[data-fresh-next]")? {
            let r = rail.clone();
            on(&next, "click", move |_: Event| {
                let at = r.st.borrow().active;
                r.go_to(at + 1.0);
            })?;
        }
        html(&nav).set_hidden(false);
    }

    let r = rail.clone();
    on(&rail_el, "pointerdown", move |e: PointerEvent| r.pointer_down(&e))?;
    let r = rail.clone();
    on(&rail_el, "pointermove", move |e: PointerEvent| r.pointer_move(&e))?;
    let r = rail.clone();
    on(&rail_el, "pointerup", move |e: PointerEvent| r.end_drag(&e))?;
    let r = rail.clone();
    on(&rail_el, "pointercancel", move |e: PointerEvent| r.end_drag(&e))?;
    let r = rail.clone();
    on(&rail_el, "click", move |e: MouseEvent| r.click(&e))?;

    rail.layout(0.0);
    let r = rail.clone();
    let tick = Closure::<dyn FnMut()>::new(move || r.step());
    window()?.set_interval_with_callback_and_timeout_and_arguments_0(tick.as_ref().unchecked_ref(), STEP_MS)?;
    tick.forget();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::signed_offset;

    #[test]
    fn offsets_wrap_around_the_ring_like_templates_py() {
        let row: Vec<f64> = (0..8).map(|i| signed_offset(i as f64, 0.0, 8.0)).collect();
        assert_eq!(row, [0.0, 1.0, 2.0, 3.0, 4.0, -3.0, -2.0, -1.0]);
        let row: Vec<f64> = (0..8).map(|i| signed_offset(i as f64, 7.0, 8.0)).collect();
        assert_eq!(row, [1.0, 2.0, 3.0, 4.0, -3.0, -2.0, -1.0, 0.0]);
        assert_eq!(signed_offset(3.0, 0.0, 1.0), 0.0);
        // Mid-drag the index is fractional and so are the offsets.
        assert!((signed_offset(0.0, 0.5, 8.0) + 0.5).abs() < 1e-12);
    }
}
