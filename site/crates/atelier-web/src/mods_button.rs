//! The header Mods link's approach animation: the word climbs.
//!
//! A mouse or pen within [`REACH`] px of the pill (distance to its rectangle's
//! edge) walks the four letters up a flight of stairs, each one step higher
//! than the last, on springs, and a faint dashed bar fades in above the pill.
//! At contact the last letter hops onto the bar: a ballistic arc, the bar
//! flares with sparks as it lands, it bounces once and stays perched while
//! the pointer is on the pill. Leaving, the letters tumble back down out of
//! step: each lets go at its own moment, with its own spin and its own
//! spring.
//!
//! Only the inner face's pieces move (the letters, the bar), never the link
//! box, so the click target is the pill's own size at every moment. The
//! Graph pill beside it has an approach effect of its own; whichever of the
//! two the pointer is nearer owns it, so they never move together. The "New"
//! tag steps aside from the first stair and comes back only once every
//! letter is at rest (`is-astir`). Springs
//! run on animation frames only until everything is at rest, then every
//! inline style is cleared. Off under reduced motion and forced colours, and
//! for touch (decided per event: a finger has no approach). Keyboard focus
//! shows the climbed letters statically; that is CSS, not this.

use crate::dom::{
    FrameCallback, Slot, distance_to, document, media, on, on_passive, owns_approach,
    prefers_reduced_motion, request_frame, set_timeout, to_fixed, window,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{DomRect, Element, Event, HtmlElement, PointerEvent};

/// How far from the pill's edge the climb starts, in CSS px.
const REACH: f64 = 120.0;
/// One stair, in CSS px: letter `i` rises `(i + 1)` of them at full approach.
/// Tall enough to read as stairs rather than a wobble; the stylesheet's
/// focus state climbs the same steps (styles.css `--mods-step`).
const STEP: f64 = 3.75;
/// Gravity for the hop, px/s².
const GRAVITY: f64 = 2600.0;
/// How far above the bar the hop peaks, px.
const HOP_APEX: f64 = 12.0;
/// The share of its speed the letter keeps on its one bounce.
const RESTITUTION: f64 = 0.7;
/// Letting go on the way down: each letter's delay (ms), spin kick (deg/s)
/// and spring stiffness, all different, so they land out of step.
const LET_GO_MS: [f64; 4] = [95.0, 0.0, 160.0, 45.0];
const SPIN_KICK: [f64; 4] = [-240.0, 300.0, -190.0, 360.0];
const FALL_K: [f64; 4] = [120.0, 175.0, 95.0, 150.0];
/// Below this much total motion (px, deg and their speeds summed) the
/// springs count as settled and every inline style is cleared.
const SETTLED: f64 = 0.05;
/// How long the bar's flare class stays on, ms.
const FLARE_MS: f64 = 700.0;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Following its stair on a spring.
    Spring,
    /// In the air (or on its one bounce) toward the bar.
    Hop { bounced: bool },
    /// Sitting on the bar.
    Perched,
}

#[derive(Clone, Copy)]
struct Letter {
    y: f64,
    vy: f64,
    target: f64,
    rot: f64,
    vrot: f64,
    squash: f64,
    vsquash: f64,
    /// When it lets go on the way down (performance.now ms), or 0.
    let_go_at: f64,
    mode: Mode,
}

impl Default for Letter {
    fn default() -> Self {
        Letter {
            y: 0.0,
            vy: 0.0,
            target: 0.0,
            rot: 0.0,
            vrot: 0.0,
            squash: 0.0,
            vsquash: 0.0,
            let_go_at: 0.0,
            mode: Mode::Spring,
        }
    }
}

#[derive(Default)]
struct State {
    rect: Option<DomRect>,
    /// The Graph link's rectangle, the rival for the pointer.
    rival_rect: Option<DomRect>,
    /// Where the last letter sits when perched: its offset from rest, px
    /// (negative is up). Measured from layout, so transforms never skew it.
    perch_y: Option<f64>,
    /// Approach, 0 (out of reach) to 1 (touching the pill).
    near: f64,
    inside: bool,
    letters: Vec<Letter>,
    bar: f64,
    vbar: f64,
    raf: i32,
    last: f64,
}

struct Climb {
    link: Element,
    /// The Graph link, whose own approach effect this one yields to when the
    /// pointer is nearer it.
    rival: Option<Element>,
    letters: Vec<HtmlElement>,
    bar: HtmlElement,
    state: RefCell<State>,
    frame: Slot<FrameCallback>,
}

/// One semi-implicit Euler step of a damped spring.
fn spring(x: f64, v: f64, target: f64, k: f64, c: f64, dt: f64) -> (f64, f64) {
    let a = (target - x) * k - v * c;
    let v = v + a * dt;
    (x + v * dt, v)
}

/// Smoothstep: the stairs ease in as the pointer comes, rather than
/// starting with a jump at the edge of reach.
fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn now() -> f64 {
    crate::dom::performance_now()
}

impl Climb {
    fn start(&self) {
        let mut st = self.state.borrow_mut();
        if st.raf == 0
            && let Some(cb) = self.frame.borrow().as_ref()
        {
            st.raf = request_frame(cb);
        }
    }

    /// The perch, from layout: the bar's top less the last letter's bottom,
    /// plus the letter's descent, so its baseline (not its box) meets the bar.
    fn perch_y(&self) -> f64 {
        let Some(s) = self.letters.last() else {
            return 0.0;
        };
        let bottom = f64::from(s.offset_top() + s.offset_height());
        let descent = f64::from(s.offset_height()) * 0.2;
        f64::from(self.bar.offset_top()) - bottom + descent
    }

    fn aim(&self, x: f64, y: f64) {
        {
            let mut st = self.state.borrow_mut();
            if st.rect.is_none() {
                st.rect = Some(self.link.get_bounding_client_rect());
            }
            if st.perch_y.is_none() {
                st.perch_y = Some(self.perch_y());
            }
            if st.rival_rect.is_none() {
                st.rival_rect = self.rival.as_ref().map(Element::get_bounding_client_rect);
            }
            let d = distance_to(st.rect.as_ref().expect("measured"), x, y);
            let rival = st.rival_rect.as_ref().map(|r| distance_to(r, x, y));
            // Nearer the Graph pill: that one moves, this one stays down.
            let near = if owns_approach(d, rival, true) {
                0f64.max(1.0 - d / REACH)
            } else {
                0.0
            };
            let inside = d == 0.0;
            let was_inside = st.inside;
            let perch = st.perch_y.unwrap_or(0.0);
            st.near = near;
            st.inside = inside;
            let n = st.letters.len();
            let lift = ease(near);
            let t = now();
            for (i, l) in st.letters.iter_mut().enumerate() {
                let stair = -STEP * (i as f64 + 1.0) * lift;
                if near == 0.0 {
                    // Out of reach: let go later, out of step (frame() applies it).
                    if (l.target < 0.0 || l.mode != Mode::Spring) && l.let_go_at == 0.0 {
                        l.let_go_at = t + LET_GO_MS[i % LET_GO_MS.len()];
                    }
                } else {
                    l.let_go_at = 0.0;
                    l.target = stair;
                }
                if i + 1 == n {
                    if inside && !was_inside && l.mode == Mode::Spring {
                        // The hop: fast enough to peak HOP_APEX above the bar.
                        let rise = (l.y - (perch - HOP_APEX)).max(1.0);
                        l.vy = -(2.0 * GRAVITY * rise).sqrt();
                        l.mode = Mode::Hop { bounced: false };
                    }
                    if !inside && was_inside && l.mode != Mode::Spring {
                        // Off the pill: it drops back to its stair, spinning a little.
                        l.mode = Mode::Spring;
                        l.vrot += SPIN_KICK[i % SPIN_KICK.len()] * 0.5;
                    }
                }
            }
            let _ = self
                .link
                .class_list()
                .toggle_with_force("is-climbing", near > 0.05);
            if near > 0.05 {
                let _ = self.link.class_list().add_1("is-astir");
            }
            // Far away with nothing moving: no frame to run.
            let busy = st.raf != 0
                || near > 0.0
                || st
                    .letters
                    .iter()
                    .any(|l| l.target != 0.0 || l.let_go_at != 0.0 || l.mode != Mode::Spring);
            if !busy {
                return;
            }
        }
        self.start();
    }

    /// The pointer left the page: everything lets go, out of step.
    fn leave(&self) {
        {
            let mut st = self.state.borrow_mut();
            st.near = 0.0;
            st.inside = false;
            let t = now();
            for (i, l) in st.letters.iter_mut().enumerate() {
                if l.target < 0.0 || l.mode != Mode::Spring {
                    l.let_go_at = t + LET_GO_MS[i % LET_GO_MS.len()];
                }
            }
            let _ = self.link.class_list().remove_1("is-climbing");
        }
        self.start();
    }

    /// Everything back at rest with no inline style left, so the stylesheet
    /// (the focus state among it) has the letters again.
    fn rest(&self, st: &mut State) {
        let _ = self.link.class_list().remove_1("is-astir");
        for el in &self.letters {
            let _ = el.style().set_property("transform", "");
        }
        let _ = self.bar.style().set_property("opacity", "");
        let _ = self.bar.style().set_property("transform", "");
        for l in st.letters.iter_mut() {
            *l = Letter::default();
        }
        st.bar = 0.0;
        st.vbar = 0.0;
        st.raf = 0;
        st.last = 0.0;
    }

    fn flare(&self) {
        let class = self.link.class_list();
        let _ = class.remove_1("is-flaring");
        // A layout read between remove and add restarts the CSS animation.
        let _ = self.bar.offset_width();
        let _ = class.add_1("is-flaring");
        let link = self.link.clone();
        set_timeout(
            move || drop(link.class_list().remove_1("is-flaring")),
            FLARE_MS,
        );
    }

    fn frame(&self, time: f64) {
        let mut flare = false;
        let mut st = self.state.borrow_mut();
        let since = if st.last != 0.0 { st.last } else { time };
        let dt = 0.033f64.min((time - since) / 1000.0);
        st.last = time;
        let perch = st.perch_y.unwrap_or(0.0);
        let t = now();
        let bar_target = if st.inside { 1.0 } else { 0.6 * ease(st.near) };
        (st.bar, st.vbar) = spring(st.bar, st.vbar, bar_target, 160.0, 20.0, dt);
        let mut moving = (st.bar - bar_target).abs() + st.vbar.abs();
        for (i, l) in st.letters.iter_mut().enumerate() {
            if l.let_go_at != 0.0 && t >= l.let_go_at {
                l.let_go_at = 0.0;
                l.target = 0.0;
                if l.mode != Mode::Spring {
                    l.mode = Mode::Spring;
                }
                l.vrot += SPIN_KICK[i % SPIN_KICK.len()];
            }
            match l.mode {
                Mode::Spring => {
                    let falling = l.target > l.y + 0.5;
                    let (k, c) = if falling {
                        (FALL_K[i % FALL_K.len()], 9.0)
                    } else {
                        (210.0, 16.0)
                    };
                    (l.y, l.vy) = spring(l.y, l.vy, l.target, k, c, dt);
                }
                Mode::Hop { bounced } => {
                    l.vy += GRAVITY * dt;
                    l.y += l.vy * dt;
                    if l.vy > 0.0 && l.y >= perch {
                        l.y = perch;
                        l.vsquash += 9.0 * (l.vy / 400.0).min(1.5);
                        if bounced {
                            l.vy = 0.0;
                            l.mode = Mode::Perched;
                        } else {
                            l.vy = -l.vy * RESTITUTION;
                            l.mode = Mode::Hop { bounced: true };
                            flare = true;
                        }
                    }
                }
                Mode::Perched => {
                    l.y = perch;
                    l.vy = 0.0;
                }
            }
            (l.rot, l.vrot) = spring(l.rot, l.vrot, 0.0, 150.0, 9.0, dt);
            (l.squash, l.vsquash) = spring(l.squash, l.vsquash, 0.0, 420.0, 20.0, dt);
            let rest = if l.mode == Mode::Spring {
                l.target
            } else {
                perch
            };
            moving += (l.y - rest).abs()
                + l.vy.abs()
                + l.rot.abs()
                + l.vrot.abs()
                + l.squash.abs()
                + l.vsquash.abs();
            if l.let_go_at != 0.0 || matches!(l.mode, Mode::Hop { .. }) {
                moving += 1.0;
            }
        }
        let at_rest = moving < SETTLED
            && st.near == 0.0
            && st
                .letters
                .iter()
                .all(|l| l.target == 0.0 && l.mode == Mode::Spring);
        if at_rest {
            self.rest(&mut st);
            return;
        }
        for (el, l) in self.letters.iter().zip(&st.letters) {
            let transform = format!(
                "translate(0px,{}px) rotate({}deg) scale({},{})",
                to_fixed(l.y, 2),
                to_fixed(l.rot, 2),
                to_fixed(1.0 + l.squash * 0.6, 4),
                to_fixed(1.0 - l.squash, 4)
            );
            let _ = el.style().set_property("transform", &transform);
        }
        let _ = self
            .bar
            .style()
            .set_property("opacity", &to_fixed(st.bar.clamp(0.0, 1.0), 3));
        let _ = self.bar.style().set_property(
            "transform",
            &format!(
                "scaleX({})",
                to_fixed(0.55 + 0.45 * st.bar.clamp(0.0, 1.2), 3)
            ),
        );
        st.raf = self.frame.borrow().as_ref().map(request_frame).unwrap_or(0);
        drop(st);
        if flare {
            self.flare();
        }
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(link) = doc.query_selector("[data-mods-climb]")? else {
        return Ok(());
    };
    if prefers_reduced_motion() || media("(forced-colors: active)") {
        return Ok(());
    }
    let letters: Vec<HtmlElement> = crate::dom::select_within(&link, ".mods-climb__letter")?
        .iter()
        .map(crate::dom::html)
        .collect();
    let Some(bar) = link.query_selector(".mods-climb__bar")? else {
        return Ok(());
    };
    if letters.is_empty() {
        return Ok(());
    }
    let climb = Rc::new(Climb {
        link: link.clone(),
        rival: doc.query_selector("[data-graph-magnet]")?,
        state: RefCell::new(State {
            letters: vec![Letter::default(); letters.len()],
            ..Default::default()
        }),
        letters,
        bar: bar.unchecked_into(),
        frame: RefCell::new(None),
    });
    let weak = Rc::downgrade(&climb);
    *climb.frame.borrow_mut() = Some(Closure::new(move |time: f64| {
        if let Some(c) = weak.upgrade() {
            c.frame(time);
        }
    }));

    let w = window()?;
    let forget = |c: &Rc<Climb>| {
        let mut st = c.state.borrow_mut();
        st.rect = None;
        st.rival_rect = None;
        st.perch_y = None;
    };
    let c = climb.clone();
    on_passive(&w, "resize", move |_: Event| forget(&c))?;
    let c = climb.clone();
    on_passive(&w, "scroll", move |_: Event| {
        let mut st = c.state.borrow_mut();
        st.rect = None;
        st.rival_rect = None;
    })?;
    let c = climb.clone();
    on_passive(&doc, "pointermove", move |e: PointerEvent| {
        let kind = e.pointer_type();
        if kind != "mouse" && kind != "pen" {
            return;
        }
        c.aim(
            crate::dom::num(&e, "clientX"),
            crate::dom::num(&e, "clientY"),
        );
    })?;
    let root = doc
        .document_element()
        .ok_or_else(|| JsValue::from_str("no root"))?;
    let c = climb.clone();
    on(&root, "pointerleave", move |_: Event| c.leave())?;
    // Keyboard focus belongs to the stylesheet's still, climbed word: with
    // no pointer near, whatever the springs were finishing stops here.
    let c = climb.clone();
    on(&link, "focus", move |_: Event| {
        let mut st = c.state.borrow_mut();
        if st.near == 0.0 {
            c.rest(&mut st);
        }
    })?;
    std::mem::forget(climb);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stairs_ease_in_and_reach_full_height_at_contact() {
        assert_eq!(ease(0.0), 0.0);
        assert_eq!(ease(1.0), 1.0);
        assert!(ease(0.25) < 0.25 && ease(0.75) > 0.75);
    }

    #[test]
    fn a_spring_settles_on_its_target() {
        let (mut x, mut v) = (0.0, 0.0);
        for _ in 0..600 {
            (x, v) = spring(x, v, -8.8, 210.0, 16.0, 1.0 / 60.0);
        }
        assert!((x + 8.8).abs() < 1e-3 && v.abs() < 1e-3);
    }

    #[test]
    fn the_hop_peaks_at_its_apex_above_the_bar() {
        // From rest on its stair (-8.8) to a perch 30 px up: the launch speed
        // carries it HOP_APEX past the bar before gravity turns it.
        let (start, perch) = (-8.8, -38.8);
        let rise = start - (perch - HOP_APEX);
        let v0 = (2.0 * GRAVITY * rise).sqrt();
        let peak = start - v0 * v0 / (2.0 * GRAVITY);
        assert!((peak - (perch - HOP_APEX)).abs() < 1e-9);
    }

    #[test]
    fn the_letters_let_go_out_of_step() {
        let mut delays = LET_GO_MS.to_vec();
        delays.sort_by(f64::total_cmp);
        delays.dedup();
        assert_eq!(delays.len(), 4);
        assert!(SPIN_KICK.windows(2).all(|w| w[0].signum() != w[1].signum()));
    }
}
