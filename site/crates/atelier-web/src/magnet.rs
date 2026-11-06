//! The header Graph link's magnet (site.js initGraphMagnet). Only the inner
//! face is transformed, so the click target never moves. A mouse or pen
//! approaching the pill (distance to its rectangle's edge, within 120 px)
//! lifts it toward 1.1x, leans and drifts it toward the pointer; contact and a
//! press give it a small kick. Springs run on animation frames only until
//! settled, then the inline transform is cleared. Off under reduced motion.
//! The Mods pill beside it climbs toward an approaching pointer; whichever
//! of the two the pointer is nearer owns it, so the two never move together.

use crate::dom::{
    FrameCallback, Slot, distance_to, document, on, on_passive, owns_approach, prefers_reduced_motion, request_frame,
    to_fixed, window,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{DomRect, Element, Event, HtmlElement, PointerEvent};

const REACH: f64 = 120.0;
const GROWTH: f64 = 0.1;

#[derive(Default)]
struct Springs {
    rect: Option<DomRect>,
    /// The Mods link's rectangle, the rival for the pointer.
    rival_rect: Option<DomRect>,
    s: f64, sv: f64, st: f64,
    w: f64, wv: f64,
    tilt: f64, tilt_v: f64, tilt_t: f64,
    dx: f64, dx_v: f64, dx_t: f64,
    dy: f64, dy_v: f64, dy_t: f64,
    inside: bool,
    raf: i32,
    last: f64,
}

struct Magnet {
    link: Element,
    /// The Mods link: nearer it, the Mods climb has the pointer and this
    /// face stays at rest.
    rival: Option<Element>,
    face: HtmlElement,
    state: RefCell<Springs>,
    frame: Slot<FrameCallback>,
}

/// One semi-implicit Euler step of a damped spring.
fn spring(x: f64, v: f64, target: f64, k: f64, c: f64, dt: f64) -> (f64, f64) {
    let a = (target - x) * k - v * c;
    let v = v + a * dt;
    (x + v * dt, v)
}

impl Magnet {
    fn start(&self) {
        let mut st = self.state.borrow_mut();
        if st.raf == 0
            && let Some(cb) = self.frame.borrow().as_ref()
        {
            st.raf = request_frame(cb);
        }
    }

    fn aim(&self, x: f64, y: f64) {
        {
            let mut st = self.state.borrow_mut();
            if st.rect.is_none() {
                st.rect = Some(self.link.get_bounding_client_rect());
            }
            if st.rival_rect.is_none() {
                st.rival_rect = self.rival.as_ref().map(Element::get_bounding_client_rect);
            }
            let r = st.rect.as_ref().expect("measured");
            let (cx, cy) = (r.left() + r.width() / 2.0, r.top() + r.height() / 2.0);
            let (ox, oy) = (x - cx, y - cy);
            let d = distance_to(r, x, y);
            let rival = st.rival_rect.as_ref().map(|r| distance_to(r, x, y));
            let owns = owns_approach(d, rival, false);
            let t = if owns { 0f64.max(1.0 - d / REACH) } else { 0.0 };
            let now = owns && d == 0.0;
            if now && !st.inside {
                st.sv += 0.8;
                st.wv += 1.5;
            }
            if !now && st.inside {
                st.wv -= 1.0;
            }
            st.inside = now;
            st.st = 1.0 + GROWTH * js_sys::Math::pow(t, 2.4);
            st.tilt_t = (-1f64).max(1f64.min(ox / 90.0)) * 2.5 * t;
            st.dx_t = (-4f64).max(4f64.min(ox * 0.04)) * t;
            st.dy_t = (-2f64).max(2f64.min(oy * 0.04)) * t;
            let _ = self.link.class_list().toggle_with_force("site-header__nav-link--near", t > 0.35);
        }
        self.start();
    }

    fn frame(&self, time: f64) {
        let mut st = self.state.borrow_mut();
        let since = if st.last != 0.0 { st.last } else { time };
        let dt = 0.033f64.min((time - since) / 1000.0);
        st.last = time;
        (st.s, st.sv) = spring(st.s, st.sv, st.st, 170.0, 22.0, dt);
        let wt = -st.sv * 0.012;
        (st.w, st.wv) = spring(st.w, st.wv, wt, 260.0, 28.0, dt);
        (st.tilt, st.tilt_v) = spring(st.tilt, st.tilt_v, st.tilt_t, 120.0, 20.0, dt);
        (st.dx, st.dx_v) = spring(st.dx, st.dx_v, st.dx_t, 120.0, 20.0, dt);
        (st.dy, st.dy_v) = spring(st.dy, st.dy_v, st.dy_t, 120.0, 20.0, dt);
        let (sx, sy) = (st.s * (1.0 - st.w), st.s * (1.0 + st.w));
        let transform = format!(
            "translate({}px,{}px) rotate({}deg) scale({},{})",
            to_fixed(st.dx, 2),
            to_fixed(st.dy, 2),
            to_fixed(st.tilt, 2),
            to_fixed(sx, 4),
            to_fixed(sy, 4)
        );
        let _ = self.face.style().set_property("transform", &transform);
        let settled = (st.s - st.st).abs()
            + st.sv.abs()
            + st.w.abs()
            + st.wv.abs()
            + (st.tilt - st.tilt_t).abs()
            + (st.dx - st.dx_t).abs()
            + (st.dy - st.dy_t).abs()
            < 0.002;
        if settled && st.st == 1.0 {
            let _ = self.face.style().set_property("transform", "");
            st.raf = 0;
            st.last = 0.0;
            return;
        }
        st.raf = self.frame.borrow().as_ref().map(request_frame).unwrap_or(0);
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(link) = doc.query_selector("[data-graph-magnet]")? else { return Ok(()) };
    if prefers_reduced_motion() {
        return Ok(());
    }
    let Some(face) = link.query_selector(".site-header__graph-face")? else { return Ok(()) };
    let magnet = Rc::new(Magnet {
        link: link.clone(),
        rival: doc.query_selector("[data-mods-climb]")?,
        face: face.unchecked_into(),
        state: RefCell::new(Springs { s: 1.0, st: 1.0, ..Default::default() }),
        frame: RefCell::new(None),
    });
    let weak = Rc::downgrade(&magnet);
    *magnet.frame.borrow_mut() = Some(Closure::new(move |time: f64| {
        if let Some(m) = weak.upgrade() {
            m.frame(time);
        }
    }));

    let w = window()?;
    let m = magnet.clone();
    let forget = |m: &Magnet| {
        let mut st = m.state.borrow_mut();
        st.rect = None;
        st.rival_rect = None;
    };
    on_passive(&w, "resize", move |_: Event| forget(&m))?;
    let m = magnet.clone();
    on_passive(&w, "scroll", move |_: Event| forget(&m))?;
    // Mouse and pen only, decided per event: a finger has no approach.
    let m = magnet.clone();
    on_passive(&doc, "pointermove", move |e: PointerEvent| {
        let kind = e.pointer_type();
        if kind != "mouse" && kind != "pen" {
            return;
        }
        m.aim(crate::dom::num(&e, "clientX"), crate::dom::num(&e, "clientY"));
    })?;
    let root = doc.document_element().ok_or_else(|| JsValue::from_str("no root"))?;
    let m = magnet.clone();
    on(&root, "pointerleave", move |_: Event| {
        {
            let mut st = m.state.borrow_mut();
            st.st = 1.0;
            st.tilt_t = 0.0;
            st.dx_t = 0.0;
            st.dy_t = 0.0;
            st.inside = false;
        }
        let _ = m.link.class_list().remove_1("site-header__nav-link--near");
        m.start();
    })?;
    // A press dips it, and it comes straight back.
    let m = magnet.clone();
    on(&link, "pointerdown", move |_: Event| {
        {
            let mut st = m.state.borrow_mut();
            st.sv -= 1.2;
            st.wv -= 1.0;
        }
        m.start();
    })?;
    std::mem::forget(magnet);
    Ok(())
}
