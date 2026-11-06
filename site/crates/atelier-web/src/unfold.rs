//! The release card at the top of the landing page. It starts small and centred, and about a
//! second after it is in view opens on its own to the whole width of the page. At this
//! layout's wide widths the band's height never changes and only the window cut through it
//! does, so nothing else on the page moves, which is what lets it happen unasked.
//!
//! The markup says everything without this module. Here: `--bleed` and the open state, a
//! braid with one bead per skill, a spring on each bead, a wave of light on opening, beams to
//! the skills a request names, a dial, and a demo that types until a person takes the field.
//! The router's table is fetched when the details first show, and if it cannot be had the
//! field goes and everything else carries on.
//!
//! Frames run while the card is on screen, the tab is visible, and there is something to draw:
//! a folded card with nobody pointing at it stops once its braid has settled, and opening it or
//! pointing at it starts the frames again. Reduced motion is open at once, with a still braid, no
//! demo and a still dial; the field still answers. Forced colours hide the canvas in CSS, and
//! nothing is drawn for it.
//!
//! The toggle shows only once this module has attached to it (`data-unfold-live`), so it is never
//! a button that does nothing.

use crate::dom::{
    FrameCallback, Slot, clear_timeout, document, format_template, js_num, media, num, on, on_resize,
    on_theme, on_visible, performance_now, prefers_reduced_motion, request_frame, set_timeout,
    then, truthy, when_fonts_ready, window,
};
use atelier_core::router::{Routed, Table};
use js_sys::{Array, Object, Reflect};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d as Ctx, Element, Event, HtmlCanvasElement, HtmlElement,
    HtmlInputElement, IntersectionObserver, IntersectionObserverInit, PointerEvent, Response,
};

const TAU: f64 = PI * 2.0;
/// Holds the release's `new_until`: a fold is remembered per release.
const FOLD_KEY: &str = "tbaguette-unfold-folded";
const FAMILY_COLOURS: usize = 12;

/// The same query the stylesheet stacks the card on, so the two can never disagree between
/// 74.99rem and 75rem at a fractional width.
const NARROW: &str = "(max-width: 74.99rem)";

/// After it is in view, how long before it opens.
const OPEN_AFTER_MS: f64 = 1250.0;
/// How much of it must be in view to arm that.
const ARM_THRESHOLD: f64 = 0.4;
/// How often something paused looks again.
const PAUSED_POLL_MS: f64 = 800.0;

/// Seconds a beam takes to reach its bead.
const BEAM_FLIGHT: f64 = 0.55;
const BEAM_STEPS: usize = 26;
/// Seconds the wave of the opening is drawn for.
const WAVE_LIFE: f64 = 2.2;
/// How close a pointer must come to name a bead, squared, in CSS px.
const HIT_RADIUS_SQUARED: f64 = 15.0 * 15.0;
/// A bead this close to where it belongs, and this slow, in CSS px and px/s, has settled.
const AT_REST: f64 = 0.05;
/// A label's chip is 19px tall; a stacked one sits this far above the one below it.
const LABEL_STEP: f64 = 21.0;
/// A press on the toggle this soon after the card opened by itself is someone who reached for the
/// button while it was still closed. They meant to open it, so it does not fold.
const SAME_GESTURE_MS: f64 = 400.0;

const SAID_NOTHING: &str = "Nothing named here. This one gets the general reminder.";
const SAID_SHORT: &str = "Too short to route, like \u{201c}Yes, go\u{201d}.";
const SAID_EMPTY: &str = "Type a request and see which skills it would name.";

/// Where the braid goes, measured from the page by [`Unfold::measure`].
#[derive(Debug, Clone, Copy, PartialEq)]
struct Stage {
    w: f64,
    h: f64,
    wide: bool,
    /// The small card's width, its braid's inset from that, and the stacked layout's gutter.
    card: f64,
    inset: f64,
    gutter: f64,
    /// The stacked layout's lane, from the canvas's top.
    lane_mid: f64,
    lane_h: f64,
}

/// Where bead `i` of `n` belongs: two strands, in catalog order so the families read as bands.
fn braid(s: &Stage, n: usize, i: usize, open: bool, t: f64) -> (f64, f64) {
    let f = (i as f64 + 0.5) / n as f64;
    let (x0, x1, cy, amp, turns) = if s.wide {
        if open {
            (
                14.0,
                s.w - 14.0,
                s.h * 0.585,
                (s.h * 0.105).min(46.0),
                (s.w / 230.0).max(4.0),
            )
        } else {
            let left = (s.w - s.card) / 2.0 + s.inset;
            (
                left,
                (s.w + s.card) / 2.0 - s.inset,
                s.h * 0.72,
                s.h * 0.13,
                2.5,
            )
        }
    } else {
        let g = if open { 10.0 } else { s.gutter + 14.0 };
        (
            g,
            s.w - g,
            s.lane_mid,
            s.lane_h * 0.3,
            if open { 3.0 } else { 2.2 },
        )
    };
    let phase = f * turns * TAU + t * 0.22;
    let strand = if i % 2 == 1 { PI } else { 0.0 };
    let wobble = 0.86 + 0.14 * (f * 19.0).sin();
    (
        x0 + (x1 - x0) * f,
        cy + (phase + strand).sin() * amp * wobble,
    )
}

/// One step of a bead's spring towards `target`: a spring and not a tween, so it answers an
/// interruption from wherever the bead is. Stiffer at the centre, which staggers the opening
/// outward. Returns the new position and velocity.
fn spring(at: f64, velocity: f64, target: f64, from_centre: f64, dt: f64) -> (f64, f64) {
    let k = 34.0 - from_centre * 14.0;
    let velocity = velocity + (target - at) * k * dt - velocity * 7.5 * dt;
    (at + velocity * dt, velocity)
}

/// How bright the opening's wave is on bead `i`, `since` seconds in: it runs out from the
/// centre to both edges, once.
fn wave(i: usize, n: usize, since: f64) -> f64 {
    if since >= WAVE_LIFE {
        return 0.0;
    }
    let off = ((i as f64 + 0.5) / n as f64 - 0.5).abs() * 2.0 - since * 1.15;
    (-off * off * 42.0).exp()
}

/// Whether the details are showing: at wide widths whenever the window is open, and on request
/// below that.
fn details_showing(wide: bool, open: bool, detailed: bool) -> bool {
    if wide { open } else { detailed }
}

/// `(open, detailed)` after a press on the toggle: a fold closes the window as well as the
/// details at every width, and a press to show opens both.
fn after_toggle(showing: bool) -> (bool, bool) {
    if showing {
        (false, false)
    } else {
        (true, true)
    }
}

/// Whether a press would fold what the card opened by itself a moment ago: `since` ms ago, and
/// showing, because below the wide layout a press shows the details and is what they meant.
fn press_meant_to_open(showing: bool, since_auto_open: f64) -> bool {
    showing && since_auto_open < SAME_GESTURE_MS
}

/// A bead has settled when it sits still where it belongs.
fn bead_settled(at: (f64, f64), velocity: (f64, f64), target: (f64, f64)) -> bool {
    (at.0 - target.0).abs() < AT_REST
        && (at.1 - target.1).abs() < AT_REST
        && velocity.0.abs() < AT_REST
        && velocity.1.abs() < AT_REST
}

/// Where a label's chip goes, given the chips already placed as `(x, width, y)`: at `y`, or above
/// whichever it would overlap, so the second of two neighbours sits above the first.
fn stacked_y(x: f64, width: f64, y: f64, placed: &[(f64, f64, f64)]) -> f64 {
    let mut y = y;
    while let Some(under) = placed
        .iter()
        .filter(|p| x < p.0 + p.1 && p.0 < x + width && (y - p.2).abs() < LABEL_STEP)
        .map(|p| p.2)
        .reduce(f64::min)
    {
        y = under - LABEL_STEP;
    }
    y
}

/// `text` with its last `count` characters gone.
fn without_last(text: &str, count: usize) -> String {
    let keep = text.chars().count().saturating_sub(count);
    text.chars().take(keep).collect()
}

struct Bead {
    slug: String,
    family: usize,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    glow: f64,
    /// How far its label sits above its place, eased so a label moving up a stack glides.
    lift: f64,
}

/// A line from the field to a bead it named: it stays until the field says something else.
struct Beam {
    bead: usize,
    age: f64,
}

/// What a press on a touch screen is waiting for: a second tap on the bead it named.
#[derive(Default)]
struct Tap {
    last_pointer: String,
    armed: Option<usize>,
}

struct Palette {
    colours: Vec<String>,
    ink: String,
    accent: String,
    ground: String,
    line: String,
    font: String,
}

struct State {
    stage: Stage,
    open: bool,
    detailed: bool,
    beads: Vec<Bead>,
    by_slug: HashMap<String, usize>,
    palette: Palette,
    /// Seconds drawn, which the wave of the opening is timed by.
    t: f64,
    /// Seconds the braid has swayed: this one stops while the card is folded and left alone, so
    /// the braid can settle.
    sway: f64,
    last: f64,
    visible: bool,
    in_view: bool,
    /// The opening has been scheduled, or never will be: a person has spoken.
    armed: bool,
    auto_open: JsValue,
    /// When the card last opened by itself, in `performance.now()` terms.
    auto_opened_at: f64,
    /// A mouse or pen is over the card.
    pointer_over: bool,
    wave_at: f64,
    /// Lit beads, in the order they were lit.
    lit: Vec<usize>,
    beams: Vec<Beam>,
    hover: Option<usize>,
    tap: Tap,
    demo_on: bool,
    demo_timer: JsValue,
    phrase_at: usize,
    /// What is in the field is the demo's and goes when a person arrives.
    field_is_demo: bool,
    dial: u32,
}

struct Dial {
    from: u32,
    at: u32,
    silent: String,
    told: String,
}

impl Dial {
    fn says(&self, n: u32) -> String {
        let template = if n >= self.at {
            &self.told
        } else {
            &self.silent
        };
        format_template(template, &[("n", n.to_string())])
    }
}

struct Unfold {
    root: HtmlElement,
    toggle: Element,
    more: HtmlElement,
    canvas: HtmlCanvasElement,
    ctx: Ctx,
    form: HtmlElement,
    input: HtmlInputElement,
    named: Element,
    watch: HtmlElement,
    used: Element,
    lane: Element,
    href: String,
    release: String,
    phrases: Vec<String>,
    dial: Dial,
    reduced: bool,
    forced: bool,
    table: RefCell<Option<Table>>,
    table_asked: Cell<bool>,
    raf: Cell<i32>,
    frame: Slot<FrameCallback>,
    st: RefCell<State>,
}

impl Unfold {
    // --- geometry --------------------------------------------------------------------------

    /// Pins `--bleed` to the document's real width, which 100vw is not beside a scrollbar or
    /// the gutter this site reserves for one, and sizes the canvas to what that gives. Returns
    /// whether the backing store changed, which clears it, and whether the layout crossed the
    /// wide breakpoint.
    fn measure(&self) -> (bool, bool) {
        let page = document()
            .ok()
            .and_then(|d| d.document_element())
            .map_or(0.0, |e| e.get_bounding_client_rect().width());
        // Through JavaScript's number formatting: Rust's would add 24 KB to the module.
        let _ = self
            .root
            .style()
            .set_property("--bleed", &format!("{}px", js_num(page)));
        let wide = !media(NARROW);
        let rect = self.canvas.get_bounding_client_rect();
        let dpr = window()
            .map_or(1.0, |w| w.device_pixel_ratio())
            .clamp(1.0, 2.0);
        let (width, height) = (
            (rect.width() * dpr).round() as u32,
            (rect.height() * dpr).round() as u32,
        );
        let resized = self.canvas.width() != width || self.canvas.height() != height;
        if resized {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
        }
        let _ = self.ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
        let rem = root_font_size();
        let lane = self.lane.get_bounding_client_rect();
        let gutter = computed(&self.root, "--gutter");
        let gutter = js_sys::parse_float(&gutter) * rem;
        let mut st = self.st.borrow_mut();
        let crossed = st.stage.wide != wide;
        st.stage = Stage {
            w: rect.width(),
            h: rect.height(),
            wide,
            card: rem * 37.0,
            inset: rem * 1.5,
            gutter: if gutter > 0.0 { gutter } else { rem * 1.5 },
            lane_mid: lane.top() - rect.top() + lane.height() / 2.0,
            lane_h: lane.height(),
        };
        (resized, crossed)
    }

    fn remeasure(self: &Rc<Self>) {
        let (resized, crossed) = self.measure();
        if crossed {
            // Not from inside a ResizeObserver: this can change the band's height.
            let c = self.clone();
            set_timeout(
                move || {
                    let (open, detailed, wide) = {
                        let st = c.st.borrow();
                        (st.open, st.detailed, st.stage.wide)
                    };
                    c.set_open(open, if wide { open } else { detailed });
                },
                0.0,
            );
        }
        if resized {
            // A cleared canvas would otherwise paint empty until the next frame, and on a phone
            // the details' opening resizes it on every frame.
            self.draw(0.0);
        }
        self.kick();
    }

    fn read_palette(&self) {
        let mut st = self.st.borrow_mut();
        st.palette = Palette {
            colours: (1..=FAMILY_COLOURS)
                .map(|i| {
                    let colour = computed(&self.root, &format!("--graph-cat-{i}"));
                    if colour.is_empty() {
                        "#c96".to_string()
                    } else {
                        colour
                    }
                })
                .collect(),
            ink: computed(&self.root, "--text-primary"),
            accent: computed(&self.root, "--accent"),
            ground: computed(&self.root, "--bg"),
            line: computed(&self.root, "--border-strong"),
            font: format!("500 12px {}", computed(&self.root, "--font-mono")),
        };
    }

    // --- drawing ---------------------------------------------------------------------------

    /// Draws one frame and says whether that was the last one needed: the card is folded, nobody
    /// is pointing at it, and nothing on it is still moving.
    fn draw(&self, dt: f64) -> bool {
        if self.forced {
            return true;
        }
        let open01 = js_sys::parse_float(&computed(&self.root, "--open"));
        let open01 = if open01.is_nan() { 0.0 } else { open01 };
        let (from_x, from_y) = self.beam_origin();
        let c = &self.ctx;
        let st = &mut *self.st.borrow_mut();
        st.t += dt;
        let awake = st.open || st.pointer_over;
        if awake {
            st.sway += dt;
        }
        let (stage, open, t, sway) = (st.stage, st.open, st.t, st.sway);
        c.clear_rect(0.0, 0.0, stage.w, stage.h);
        let n = st.beads.len();
        let mut settled = !awake;
        for (i, b) in st.beads.iter_mut().enumerate() {
            let (x, y) = braid(&stage, n, i, open, sway);
            if self.reduced {
                (b.x, b.y) = (x, y);
                continue;
            }
            let from_centre = (i as f64 / n as f64 - 0.5).abs() * 2.0;
            (b.x, b.vx) = spring(b.x, b.vx, x, from_centre, dt);
            (b.y, b.vy) = spring(b.y, b.vy, y, from_centre, dt);
            settled = settled && bead_settled((b.x, b.y), (b.vx, b.vy), (x, y));
        }
        // Strands first, faint, so the two read as one braid.
        c.set_line_width(1.0);
        c.set_stroke_style_str(&st.palette.line);
        c.set_global_alpha(0.28 + 0.12 * open01);
        for strand in 0..2 {
            c.begin_path();
            for b in st.beads.iter().skip(strand).step_by(2) {
                c.line_to(b.x, b.y);
            }
            c.stroke();
        }
        c.set_global_alpha(1.0);

        // Beams from the field to the skills it named.
        let mut beams = std::mem::take(&mut st.beams);
        for beam in &mut beams {
            beam.age += dt;
            let (to_x, to_y) = (st.beads[beam.bead].x, st.beads[beam.bead].y);
            let flight = (beam.age / BEAM_FLIGHT).min(1.0);
            let eased = 1.0 - (1.0 - flight).powi(3);
            let mid = (
                (from_x + to_x) / 2.0,
                from_y.min(to_y) + (to_y - from_y).abs() * 0.18,
            );
            if flight >= 1.0 && !st.lit.contains(&beam.bead) {
                st.beads[beam.bead].glow = 1.0;
                st.lit.push(beam.bead);
            }
            let along = |u: f64| {
                let v = 1.0 - u;
                (
                    v * v * from_x + 2.0 * v * u * mid.0 + u * u * to_x,
                    v * v * from_y + 2.0 * v * u * mid.1 + u * u * to_y,
                )
            };
            c.set_global_alpha(0.75);
            c.set_stroke_style_str(&st.palette.accent);
            c.set_line_width(1.4);
            c.begin_path();
            c.move_to(from_x, from_y);
            let steps = (BEAM_STEPS as f64 * eased) as usize;
            for j in 1..=steps {
                let (x, y) = along(j as f64 / BEAM_STEPS as f64);
                c.line_to(x, y);
            }
            c.stroke();
            if flight < 1.0 {
                let (x, y) = along(eased);
                c.set_global_alpha(1.0);
                c.set_fill_style_str(&st.palette.accent);
                c.begin_path();
                let _ = c.arc(x, y, 3.0, 0.0, TAU);
                c.fill();
            }
            c.set_global_alpha(1.0);
        }
        st.beams = beams;

        // Beads.
        let base = if stage.wide && open { 3.6 } else { 2.7 };
        let since = t - st.wave_at;
        for i in 0..n {
            let want = if st.lit.contains(&i) || st.hover == Some(i) {
                1.0
            } else {
                0.0
            };
            let b = &mut st.beads[i];
            b.glow = if self.reduced {
                want
            } else {
                b.glow + (want - b.glow) * (dt * 5.0).min(1.0)
            };
            settled = settled && b.glow < AT_REST;
            let colour = &st.palette.colours[b.family % FAMILY_COLOURS];
            let bright = wave(i, n, since);
            let radius = base + b.glow * 3.4 + bright * 3.2;
            c.set_fill_style_str(colour);
            if bright > 0.05 {
                c.set_global_alpha(0.38 * bright);
                c.begin_path();
                let _ = c.arc(b.x, b.y, radius * 3.4, 0.0, TAU);
                c.fill();
            }
            if b.glow > 0.02 {
                c.set_global_alpha(0.28 * b.glow);
                c.begin_path();
                let _ = c.arc(b.x, b.y, radius * 3.0, 0.0, TAU);
                c.fill();
            }
            c.set_global_alpha(0.82 + 0.18 * b.glow);
            c.begin_path();
            let _ = c.arc(b.x, b.y, radius, 0.0, TAU);
            c.fill();
        }
        c.set_global_alpha(1.0);

        // A lit bead's name beside it, on a chip of ground so it reads over the braid.
        c.set_font(&st.palette.font);
        let hovered = st.hover.filter(|h| !st.lit.contains(h));
        let mut placed = Vec::new();
        for &i in st.lit.iter().chain(hovered.iter()) {
            let b = &mut st.beads[i];
            if b.glow < 0.2 {
                b.lift = 0.0;
                continue;
            }
            let width = c.measure_text(&b.slug).map_or(0.0, |m| m.width());
            let x = (b.x - width / 2.0 - 5.0)
                .min(stage.w - width - 16.0)
                .max(6.0);
            let place = b.y - 30.0;
            let lift = place - stacked_y(x, width + 10.0, place, &placed);
            b.lift = if self.reduced {
                lift
            } else {
                b.lift + (lift - b.lift) * (dt * 12.0).min(1.0)
            };
            let y = place - b.lift;
            placed.push((x, width + 10.0, y));
            c.set_global_alpha(b.glow);
            c.set_fill_style_str(&st.palette.ground);
            c.begin_path();
            let _ = c.round_rect_with_f64(x, y - 13.0, width + 10.0, 19.0, 5.0);
            c.fill();
            c.set_fill_style_str(&st.palette.ink);
            let _ = c.fill_text(&b.slug, x + 5.0, y + 1.0);
            c.set_global_alpha(1.0);
        }
        settled && st.hover.is_none() && st.lit.is_empty() && st.beams.is_empty() && since >= WAVE_LIFE
    }

    /// Where the beams leave the field: under the last character typed. Layout is read only
    /// when there are beams to draw.
    fn beam_origin(&self) -> (f64, f64) {
        if self.st.borrow().beams.is_empty() {
            return (0.0, 0.0);
        }
        let field = self.input.get_bounding_client_rect();
        let canvas = self.canvas.get_bounding_client_rect();
        let typed = self.input.value().chars().count() as f64;
        let x = (field.right() - 12.0).min(field.left() + 16.0 + typed * 7.8) - canvas.left();
        (x, field.bottom() - canvas.top())
    }

    fn on_frame(self: &Rc<Self>, now: f64) {
        self.raf.set(0);
        if !self.running() {
            return;
        }
        let last = self.st.borrow().last;
        self.st.borrow_mut().last = now;
        let finished = self.draw(((now - last) / 1000.0).clamp(0.0, 0.05));
        if !self.reduced && !finished {
            self.request();
        }
    }

    fn running(&self) -> bool {
        self.st.borrow().visible && !hidden()
    }

    fn request(&self) {
        if let Some(cb) = self.frame.borrow().as_ref() {
            self.raf.set(request_frame(cb));
        }
    }

    fn kick(&self) {
        if self.raf.get() != 0 || self.forced || !self.running() {
            return;
        }
        self.st.borrow_mut().last = performance_now();
        self.request();
    }

    // --- the router ------------------------------------------------------------------------

    /// The request's answer, or `None` until the table is here.
    fn route(&self, text: &str) -> Option<Routed> {
        self.table.borrow().as_ref().map(|table| table.route(text))
    }

    /// Puts an answer on the page. `quiet` keeps the status region from announcing it: only
    /// what a person types is spoken, never the demo and never the hint a focus brings up.
    fn say(&self, result: Option<&Routed>, quiet: bool) {
        let _ = self
            .named
            .set_attribute("aria-live", if quiet { "off" } else { "polite" });
        self.named.set_text_content(None);
        {
            let mut st = self.st.borrow_mut();
            st.lit.clear();
            st.beams.clear();
        }
        match result {
            Some(Routed::Named(names)) => {
                for slug in names {
                    self.chip(slug, quiet);
                }
            }
            Some(Routed::Nothing) => self.named.set_text_content(Some(SAID_NOTHING)),
            Some(Routed::Short) => self.named.set_text_content(Some(SAID_SHORT)),
            Some(Routed::Empty) => self.named.set_text_content(Some(SAID_EMPTY)),
            None => {}
        }
        self.kick();
    }

    /// A chip for one named skill, and its beam. A demo's chip is not a tab stop: it will be
    /// gone in a moment, with the focus on it.
    fn chip(&self, slug: &str, quiet: bool) {
        let Ok(doc) = document() else { return };
        let Ok(chip) = doc.create_element("a") else {
            return;
        };
        chip.set_class_name("unfold__chip");
        chip.set_text_content(Some(slug));
        let _ = chip.set_attribute("href", &self.href_of(slug));
        if quiet {
            let _ = chip.set_attribute("tabindex", "-1");
        }
        let _ = self.named.append_child(&chip);
        let mut st = self.st.borrow_mut();
        let Some(&bead) = st.by_slug.get(slug) else {
            return;
        };
        // The variable, not its value: a chip stays right when the theme changes under it.
        let family = st.beads[bead].family % FAMILY_COLOURS + 1;
        let _ = chip.set_attribute("style", &format!("--dot: var(--graph-cat-{family})"));
        if self.reduced {
            st.lit.push(bead);
        } else {
            st.beams.push(Beam { bead, age: 0.0 });
        }
    }

    fn href_of(&self, slug: &str) -> String {
        let slug = String::from(js_sys::encode_uri_component(slug));
        self.href.replacen("{slug}", &slug, 1)
    }

    fn load_table(self: &Rc<Self>) {
        if self.table_asked.replace(true) {
            return;
        }
        let url = self.root.get_attribute("data-router").unwrap_or_default();
        let Ok(w) = window() else {
            return self.table_failed();
        };
        let failed = |c: &Rc<Self>| {
            let c = c.clone();
            Closure::once_into_js(move |_: JsValue| c.table_failed())
        };
        let c = self.clone();
        let on_response = Closure::once_into_js(move |response: Response| {
            let parsed = if response.ok() {
                response.json().ok()
            } else {
                None
            };
            let Some(parsed) = parsed else {
                return c.table_failed();
            };
            let on_failed = failed(&c);
            let on_json = Closure::once_into_js(move |value: JsValue| match table_from(&value) {
                Ok(table) => c.table_arrived(table),
                Err(_) => c.table_failed(),
            });
            then(&parsed, &on_json, Some(&on_failed));
        });
        then(&w.fetch_with_str(&url), &on_response, Some(&failed(self)));
    }

    fn table_arrived(self: &Rc<Self>, table: Table) {
        *self.table.borrow_mut() = Some(table);
        if self.field_has_focus() || !self.input.value().is_empty() {
            // A person got here before the table did.
            let result = self.route(&self.input.value());
            self.say(result.as_ref(), false);
        } else if self.showing() {
            self.demo_run();
        }
    }

    fn table_failed(&self) {
        self.form.set_hidden(true);
    }

    // --- the demo: types real phrases until a person takes the field ------------------------

    fn demo_stop(&self) {
        let mut st = self.st.borrow_mut();
        st.demo_on = false;
        clear_timeout(&st.demo_timer);
    }

    fn demo_after(self: &Rc<Self>, ms: f64, step: impl FnOnce(&Rc<Self>) + 'static) {
        let c = self.clone();
        let timer = set_timeout(move || step(&c), ms);
        self.st.borrow_mut().demo_timer = timer;
    }

    fn demo_run(self: &Rc<Self>) {
        if self.reduced
            || self.phrases.is_empty()
            || self.table.borrow().is_none()
            || !self.showing()
            || self.field_has_focus()
        {
            return;
        }
        self.demo_stop();
        let text = {
            let mut st = self.st.borrow_mut();
            st.demo_on = true;
            st.phrase_at += 1;
            self.phrases[(st.phrase_at - 1) % self.phrases.len()].clone()
        };
        self.demo_type(text, 0);
    }

    fn demo_type(self: &Rc<Self>, text: String, at: usize) {
        if !self.st.borrow().demo_on {
            return;
        }
        if !self.running() {
            return self.demo_after(PAUSED_POLL_MS, move |c| c.demo_type(text, at));
        }
        let at = at + 1;
        self.input
            .set_value(&text.chars().take(at).collect::<String>());
        self.st.borrow_mut().field_is_demo = true;
        if at < text.chars().count() {
            let pause = 26.0 + js_sys::Math::random() * 34.0;
            return self.demo_after(pause, move |c| c.demo_type(text, at));
        }
        let result = self.route(&text);
        self.say(result.as_ref(), true);
        self.demo_after(3400.0, |c| c.demo_clear());
    }

    fn demo_clear(self: &Rc<Self>) {
        if !self.st.borrow().demo_on {
            return;
        }
        if !self.running() {
            return self.demo_after(PAUSED_POLL_MS, |c| c.demo_clear());
        }
        let value = self.input.value();
        if !value.is_empty() {
            self.input.set_value(&without_last(&value, 2));
            return self.demo_after(12.0, |c| c.demo_clear());
        }
        self.say(None, true);
        self.demo_after(500.0, |c| c.demo_run());
    }

    fn field_has_focus(&self) -> bool {
        let field: &Element = self.input.as_ref();
        document()
            .ok()
            .and_then(|d| d.active_element())
            .is_some_and(|active| &active == field)
    }

    // --- the dial: silence, then one word ---------------------------------------------------

    fn dial_tick(self: &Rc<Self>) {
        let c = self.clone();
        if !self.running() || !self.showing() {
            set_timeout(move || c.dial_tick(), PAUSED_POLL_MS);
            return;
        }
        let n = self.st.borrow().dial;
        let _ = self.watch.style().set_property("--used", &n.to_string());
        self.used.set_text_content(Some(&self.dial.says(n)));
        let delay = if n >= self.dial.at {
            let _ = self.watch.set_attribute("data-woke", "");
            self.st.borrow_mut().dial = self.dial.from;
            3400.0
        } else {
            let _ = self.watch.remove_attribute("data-woke");
            self.st.borrow_mut().dial = n + 1;
            620.0
        };
        set_timeout(move || c.dial_tick(), delay);
    }

    // --- open and fold ---------------------------------------------------------------------

    fn showing(&self) -> bool {
        let st = self.st.borrow();
        details_showing(st.stage.wide, st.open, st.detailed)
    }

    fn set_open(self: &Rc<Self>, open: bool, detailed: bool) {
        {
            let mut st = self.st.borrow_mut();
            if open && !st.open && !self.reduced {
                st.wave_at = st.t;
            }
            st.open = open;
            st.detailed = detailed;
        }
        let flag = |name: &str, on: bool| {
            let _ = self.root.toggle_attribute_with_force(name, on);
        };
        flag("data-open", open);
        flag("data-expanded", detailed);
        let showing = self.showing();
        let _ = self
            .toggle
            .set_attribute("aria-expanded", if showing { "true" } else { "false" });
        self.more.set_inert(!showing);
        if showing {
            self.load_table();
            if self.table.borrow().is_some()
                && !self.st.borrow().demo_on
                && self.input.value().is_empty()
            {
                self.demo_run();
            }
        } else {
            self.demo_stop();
            self.input.set_value("");
            let mut st = self.st.borrow_mut();
            st.field_is_demo = false;
            st.lit.clear();
            st.beams.clear();
            drop(st);
            self.named.set_text_content(None);
        }
        self.kick();
    }

    /// A press on the toggle. It speaks for the reader, so any opening still on its way is off.
    fn on_toggle(self: &Rc<Self>) {
        let showing = self.toggle.get_attribute("aria-expanded").as_deref() == Some("true");
        let since_auto_open = performance_now() - self.st.borrow().auto_opened_at;
        if press_meant_to_open(showing, since_auto_open) {
            return;
        }
        {
            let mut st = self.st.borrow_mut();
            st.armed = true;
            clear_timeout(&st.auto_open);
        }
        if let Ok(Some(storage)) = window().map(|w| w.local_storage()).unwrap_or(Ok(None)) {
            let _ = if showing {
                storage.set_item(FOLD_KEY, &self.release)
            } else {
                storage.remove_item(FOLD_KEY)
            };
        }
        let (open, detailed) = after_toggle(showing);
        self.set_open(open, detailed);
    }

    fn folded(&self) -> bool {
        let stored = window()
            .map(|w| w.local_storage())
            .unwrap_or(Ok(None))
            .ok()
            .flatten()
            .and_then(|s| s.get_item(FOLD_KEY).ok().flatten());
        stored.as_deref() == Some(self.release.as_str())
    }

    /// The surprise: nothing else on the page moves, so it costs the reader nothing to have it
    /// happen unasked.
    fn arm(self: &Rc<Self>) {
        {
            let st = self.st.borrow();
            if st.armed || !st.in_view || hidden() {
                return;
            }
        }
        self.st.borrow_mut().armed = true;
        let c = self.clone();
        let timer = set_timeout(
            move || {
                let (wide, in_view) = {
                    let st = c.st.borrow();
                    (st.stage.wide, st.in_view)
                };
                if in_view {
                    c.st.borrow_mut().auto_opened_at = performance_now();
                    c.set_open(true, wide);
                } else {
                    // It went out of view on the way: a page scrolled to a fragment, say.
                    c.st.borrow_mut().armed = false;
                }
            },
            OPEN_AFTER_MS,
        );
        self.st.borrow_mut().auto_open = timer;
    }

    // --- point at a bead to name it, click to open it --------------------------------------

    fn hit(&self, e: &PointerEvent) -> Option<usize> {
        let rect = self.canvas.get_bounding_client_rect();
        let (x, y) = (
            num(e, "clientX") - rect.left(),
            num(e, "clientY") - rect.top(),
        );
        let st = self.st.borrow();
        let mut best = None;
        let mut nearest = HIT_RADIUS_SQUARED;
        for (i, b) in st.beads.iter().enumerate() {
            let d = (b.x - x) * (b.x - x) + (b.y - y) * (b.y - y);
            if d < nearest {
                nearest = d;
                best = Some(i);
            }
        }
        best
    }

    fn set_hover(&self, bead: Option<usize>) {
        self.st.borrow_mut().hover = bead;
        let _ = self
            .canvas
            .class_list()
            .toggle_with_force("is-pointing", bead.is_some());
        self.kick();
    }

    fn pointer_events(self: &Rc<Self>) -> Result<(), JsValue> {
        let c = self.clone();
        on(&self.root, "pointerenter", move |e: PointerEvent| {
            if e.pointer_type() != "touch" {
                c.st.borrow_mut().pointer_over = true;
                c.kick();
            }
        })?;
        let c = self.clone();
        on(&self.root, "pointerleave", move |_: PointerEvent| {
            c.st.borrow_mut().pointer_over = false;
            c.kick();
        })?;
        let canvas = &self.canvas;
        let c = self.clone();
        on(canvas, "pointermove", move |e: PointerEvent| {
            if e.pointer_type() != "touch" {
                c.set_hover(c.hit(&e));
            }
        })?;
        let c = self.clone();
        on(canvas, "pointerleave", move |e: PointerEvent| {
            if e.pointer_type() != "touch" {
                c.set_hover(None);
            }
        })?;
        let c = self.clone();
        on(canvas, "pointerdown", move |e: PointerEvent| {
            c.st.borrow_mut().tap.last_pointer = e.pointer_type();
            if e.pointer_type() != "touch" {
                return;
            }
            // A finger has no hover: the first tap names a bead, a second on it opens it.
            let bead = c.hit(&e);
            {
                let mut st = c.st.borrow_mut();
                st.tap.armed = if bead.is_some() && bead == st.hover {
                    bead
                } else {
                    None
                };
            }
            c.set_hover(bead);
        })?;
        let c = self.clone();
        on(canvas, "click", move |e: PointerEvent| {
            let Some(bead) = c.hit(&e) else { return };
            let slug = {
                let st = c.st.borrow();
                if st.tap.last_pointer == "touch" && st.tap.armed != Some(bead) {
                    return;
                }
                st.beads[bead].slug.clone()
            };
            if let Ok(w) = window() {
                let _ = w.location().set_href(&c.href_of(&slug));
            }
        })
    }

    fn field_events(self: &Rc<Self>) -> Result<(), JsValue> {
        let input = &self.input;
        let c = self.clone();
        on(input, "focus", move |_: Event| {
            c.demo_stop();
            if std::mem::take(&mut c.st.borrow_mut().field_is_demo) {
                c.input.set_value("");
            }
            let result = c.route(&c.input.value());
            c.say(result.as_ref(), true);
        })?;
        let c = self.clone();
        on(input, "input", move |_: Event| {
            c.demo_stop();
            c.st.borrow_mut().field_is_demo = false;
            let result = c.route(&c.input.value());
            c.say(result.as_ref(), false);
        })?;
        let c = self.clone();
        on(input, "blur", move |_: Event| {
            if c.input.value().is_empty() {
                c.demo_after(3500.0, |c| c.demo_run());
            }
        })?;
        on(&self.form, "submit", |e: Event| e.prevent_default())
    }

    fn page_events(self: &Rc<Self>) -> Result<(), JsValue> {
        let w = window()?;
        let doc = document()?;
        let c = self.clone();
        on_visible(&self.root, move |visible| {
            c.st.borrow_mut().visible = visible;
            c.kick();
        })?;
        let c = self.clone();
        on(&doc, "visibilitychange", move |_: Event| {
            c.kick();
            c.arm();
        })?;
        let c = self.clone();
        on(&w, "resize", move |_: Event| c.remeasure())?;
        let c = self.clone();
        on_resize(&self.canvas, move || c.remeasure())?;
        let c = self.clone();
        on_theme(move || {
            c.read_palette();
            c.kick();
        })?;
        let c = self.clone();
        when_fonts_ready(move || c.remeasure())
    }

    /// Opens the card once enough of it is in view; at once if the browser cannot say.
    fn arm_when_in_view(self: &Rc<Self>) -> Result<(), JsValue> {
        if !truthy(&window()?, "IntersectionObserver") {
            self.st.borrow_mut().in_view = true;
            self.arm();
            return Ok(());
        }
        let c = self.clone();
        let cb = Closure::<dyn FnMut(Array)>::new(move |entries: Array| {
            let seen = Reflect::get(&entries.get(0), &"isIntersecting".into())
                .is_ok_and(|v| v.is_truthy());
            c.st.borrow_mut().in_view = seen;
            c.arm();
        });
        let init = IntersectionObserverInit::new();
        init.set_threshold_f64(ARM_THRESHOLD);
        IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init)?
            .observe(&self.root);
        cb.forget();
        Ok(())
    }
}

fn hidden() -> bool {
    document().map(|d| d.hidden()).unwrap_or(false)
}

fn computed(el: &Element, name: &str) -> String {
    window()
        .ok()
        .and_then(|w| w.get_computed_style(el).ok().flatten())
        .and_then(|style| style.get_property_value(name).ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn root_font_size() -> f64 {
    let root = document().ok().and_then(|d| d.document_element());
    let size = root.map_or(f64::NAN, |r| {
        js_sys::parse_float(&computed(&r, "font-size"))
    });
    if size > 0.0 { size } else { 16.0 }
}

/// The published table (`check::router::site_table`), as [`Table`] wants it. Object order is
/// the text's, which the score's tie-break needs.
fn table_from(json: &JsValue) -> Result<Table, JsValue> {
    let field = |name: &str| Reflect::get(json, &name.into());
    let routes = field("routes")?
        .dyn_into::<Array>()?
        .iter()
        .map(|route| {
            let route: Array = route.dyn_into()?;
            let text = |i: u32| route.get(i).as_string().ok_or(JsValue::NULL);
            Ok((text(0)?, text(1)?))
        })
        .collect::<Result<Vec<_>, JsValue>>()?;
    let entries = |value: &JsValue| -> Result<Vec<(String, JsValue)>, JsValue> {
        let object = value.clone().dyn_into::<Object>()?;
        Object::entries(&object)
            .iter()
            .map(|pair| {
                let pair: Array = pair.dyn_into()?;
                Ok((pair.get(0).as_string().ok_or(JsValue::NULL)?, pair.get(1)))
            })
            .collect()
    };
    let mut index = Vec::new();
    for (slug, terms) in entries(&field("index")?)? {
        let weights = entries(&terms)?
            .into_iter()
            .map(|(term, weight)| Ok((term, weight.as_f64().ok_or(JsValue::NULL)? as u32)))
            .collect::<Result<HashMap<_, _>, JsValue>>()?;
        index.push((slug, weights));
    }
    let stop: Array = field("stop")?.dyn_into()?;
    Table::new(
        routes,
        index,
        stop.iter().filter_map(|word| word.as_string()),
    )
    .map_err(|message| JsValue::from_str(&message))
}

fn json_attribute(root: &Element, name: &str) -> Array {
    root.get_attribute(name)
        .and_then(|raw| js_sys::JSON::parse(&raw).ok())
        .and_then(|value| value.dyn_into::<Array>().ok())
        .unwrap_or_else(Array::new)
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(root) = doc.query_selector("[data-unfold]")? else {
        return Ok(());
    };
    let part = |selector: &str| root.query_selector(selector);
    let (
        Some(toggle),
        Some(more),
        Some(canvas),
        Some(input),
        Some(named),
        Some(form),
        Some(watch),
        Some(lane),
    ) = (
        part(".unfold__toggle")?,
        part(".unfold__more")?,
        part(".unfold__canvas")?,
        part(".unfold__input")?,
        part(".unfold__named")?,
        part(".unfold__try")?,
        part(".unfold__watch")?,
        part(".unfold__braid-lane")?,
    )
    else {
        return Ok(());
    };
    let Some(used) = watch.query_selector("[data-used]")? else {
        return Ok(());
    };
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;
    let Some(ctx) = canvas
        .get_context("2d")?
        .and_then(|c| c.dyn_into::<Ctx>().ok())
    else {
        return Ok(());
    };
    let attribute = |name: &str| root.get_attribute(name).unwrap_or_default();
    let dial = Dial {
        from: attribute("data-dial-from").parse().unwrap_or(0),
        at: attribute("data-dial-at").parse().unwrap_or(0),
        silent: attribute("data-dial-silent"),
        told: attribute("data-dial-told"),
    };
    let beads: Vec<Bead> = json_attribute(&root, "data-beads")
        .iter()
        .filter_map(|bead| {
            let bead: Array = bead.dyn_into().ok()?;
            Some(Bead {
                slug: bead.get(0).as_string()?,
                family: bead.get(1).as_f64()? as usize,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                glow: 0.0,
                lift: 0.0,
            })
        })
        .collect();
    let by_slug = beads
        .iter()
        .enumerate()
        .map(|(i, b)| (b.slug.clone(), i))
        .collect();
    let phrases = json_attribute(&root, "data-phrases")
        .iter()
        .filter_map(|p| p.as_string())
        .collect();
    let reduced = prefers_reduced_motion();
    let start = dial.from;
    let card = Rc::new(Unfold {
        href: attribute("data-href"),
        release: attribute("data-unfold-new-until"),
        phrases,
        dial,
        reduced,
        forced: media("(forced-colors: active)"),
        toggle,
        more: more.dyn_into()?,
        canvas,
        ctx,
        form: form.dyn_into()?,
        input: input.dyn_into()?,
        named,
        watch: watch.dyn_into()?,
        used,
        lane,
        root: root.dyn_into()?,
        table: RefCell::new(None),
        table_asked: Cell::new(false),
        raf: Cell::new(0),
        frame: RefCell::new(None),
        st: RefCell::new(State {
            stage: Stage {
                w: 0.0,
                h: 0.0,
                wide: true,
                card: 0.0,
                inset: 0.0,
                gutter: 0.0,
                lane_mid: 0.0,
                lane_h: 0.0,
            },
            open: false,
            detailed: false,
            beads,
            by_slug,
            palette: Palette {
                colours: Vec::new(),
                ink: String::new(),
                accent: String::new(),
                ground: String::new(),
                line: String::new(),
                font: String::new(),
            },
            t: 0.0,
            sway: 0.0,
            last: 0.0,
            visible: true,
            in_view: false,
            armed: false,
            auto_open: JsValue::UNDEFINED,
            auto_opened_at: f64::NEG_INFINITY,
            pointer_over: false,
            wave_at: -99.0,
            lit: Vec::new(),
            beams: Vec::new(),
            hover: None,
            tap: Tap::default(),
            demo_on: false,
            demo_timer: JsValue::UNDEFINED,
            phrase_at: 0,
            field_is_demo: false,
            dial: start,
        }),
    });
    let weak = Rc::downgrade(&card);
    *card.frame.borrow_mut() = Some(Closure::new(move |now: f64| {
        if let Some(c) = weak.upgrade() {
            c.on_frame(now);
        }
    }));

    card.measure();
    card.read_palette();
    {
        let mut st = card.st.borrow_mut();
        let stage = st.stage;
        let n = st.beads.len();
        for (i, b) in st.beads.iter_mut().enumerate() {
            (b.x, b.y) = braid(&stage, n, i, false, 0.0);
        }
    }
    card.more.set_inert(true);

    let c = card.clone();
    on(&card.toggle, "click", move |_: Event| c.on_toggle())?;
    card.root.set_attribute("data-unfold-live", "")?;
    card.field_events()?;
    card.pointer_events()?;
    let folded = card.folded();
    card.page_events()?;
    if reduced {
        let wide = card.st.borrow().stage.wide;
        card.st.borrow_mut().auto_opened_at = performance_now();
        card.set_open(!folded, !folded && wide);
        card.draw(0.0);
    } else {
        card.dial_tick();
        if !folded {
            card.arm_when_in_view()?;
        }
        card.kick();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE: Stage = Stage {
        w: 1440.0,
        h: 440.0,
        wide: true,
        card: 592.0,
        inset: 24.0,
        gutter: 64.0,
        lane_mid: 0.0,
        lane_h: 0.0,
    };
    const PHONE: Stage = Stage {
        w: 390.0,
        h: 900.0,
        wide: false,
        card: 592.0,
        inset: 24.0,
        gutter: 24.0,
        lane_mid: 300.0,
        lane_h: 152.0,
    };

    fn extent(stage: &Stage, open: bool) -> (f64, f64, f64, f64) {
        let n = 103;
        let points: Vec<_> = (0..n).map(|i| braid(stage, n, i, open, 0.0)).collect();
        let fold = |f: fn(f64, f64) -> f64, start: f64, by: fn(&(f64, f64)) -> f64| {
            points.iter().map(by).fold(start, f)
        };
        (
            fold(f64::min, f64::MAX, |p| p.0),
            fold(f64::max, f64::MIN, |p| p.0),
            fold(f64::min, f64::MAX, |p| p.1),
            fold(f64::max, f64::MIN, |p| p.1),
        )
    }

    #[test]
    fn folded_the_braid_sits_inside_the_small_card_and_open_it_runs_edge_to_edge() {
        let (left, right, ..) = extent(&WIDE, false);
        assert!(
            left > (1440.0 - 592.0) / 2.0 && right < (1440.0 + 592.0) / 2.0,
            "{left} {right}"
        );
        let (left, right, ..) = extent(&WIDE, true);
        assert!(left < 30.0 && right > 1410.0, "{left} {right}");
    }

    #[test]
    fn open_the_braid_stays_in_the_lane_between_the_text_and_the_ships() {
        // The band is 27.5rem: the toggle ends about y 180 and the ships start about y 330.
        let (.., top, bottom) = extent(&WIDE, true);
        assert!(top > 200.0 && bottom < 310.0, "{top} {bottom}");
    }

    #[test]
    fn stacked_the_braid_keeps_to_its_lane_at_any_height() {
        let tall = Stage { h: 1400.0, ..PHONE };
        for stage in [PHONE, tall] {
            let (left, right, top, bottom) = extent(&stage, true);
            assert!(left >= 10.0 && right <= 380.0, "{left} {right}");
            assert!(top >= 224.0 && bottom <= 376.0, "{top} {bottom}");
        }
    }

    #[test]
    fn a_spring_settles_on_its_target_and_answers_an_interruption_from_where_it_is() {
        let run = |mut at: f64, mut velocity: f64, target: f64, seconds: f64| {
            let dt = 1.0 / 60.0;
            for _ in 0..(seconds / dt) as usize {
                (at, velocity) = spring(at, velocity, target, 0.5, dt);
            }
            (at, velocity)
        };
        let (at, velocity) = run(0.0, 0.0, 100.0, 4.0);
        assert!(
            (at - 100.0).abs() < 0.5 && velocity.abs() < 1.0,
            "{at} {velocity}"
        );
        // Part way there, then told to go back: it keeps its speed for a moment, and never jumps.
        let (mid, moving) = run(0.0, 0.0, 100.0, 0.2);
        assert!(mid > 1.0 && mid < 100.0 && moving > 0.0, "{mid} {moving}");
        let (next, _) = spring(mid, moving, 0.0, 0.5, 1.0 / 60.0);
        assert!((next - mid).abs() < 10.0, "{mid} to {next}");
        assert!(next > mid, "still going the way it was");
        let (back, _) = run(mid, moving, 0.0, 4.0);
        assert!(back.abs() < 0.5, "{back}");
    }

    #[test]
    fn the_wave_runs_out_from_the_centre_and_dies() {
        let n = 101;
        assert!(
            wave(50, n, 0.0) > 0.9 && wave(0, n, 0.0) < 0.01,
            "starts at the centre"
        );
        assert!(
            wave(0, n, 0.86) > 0.9 && wave(50, n, 0.86) < 0.01,
            "ends at the edges"
        );
        assert_eq!(wave(50, n, 2.2), 0.0);
    }

    #[test]
    fn a_toggle_folds_all_of_it_and_opens_what_the_layout_allows() {
        assert_eq!(after_toggle(true), (false, false));
        assert_eq!(after_toggle(false), (true, true));
        assert!(details_showing(true, true, false));
        assert!(!details_showing(false, true, false));
        assert!(details_showing(false, true, true));
        assert!(!details_showing(true, false, true));
    }

    #[test]
    fn a_press_just_after_the_card_opened_itself_does_not_fold_it() {
        assert!(press_meant_to_open(true, 0.0));
        assert!(press_meant_to_open(true, 399.0));
        assert!(!press_meant_to_open(true, 400.0));
        assert!(!press_meant_to_open(true, f64::INFINITY), "it never opened itself");
        assert!(!press_meant_to_open(false, 0.0), "a press that shows is what they meant");
    }

    #[test]
    fn a_folded_braid_settles_and_then_nothing_is_moving() {
        let dt = 1.0 / 60.0;
        let (mut at, mut velocity) = (0.0, 0.0);
        for _ in 0..(6.0 / dt) as usize {
            (at, velocity) = spring(at, velocity, 100.0, 1.0, dt);
        }
        assert!(bead_settled((at, at), (velocity, velocity), (100.0, 100.0)), "{at} {velocity}");
        assert!(!bead_settled((99.0, 100.0), (0.0, 0.0), (100.0, 100.0)));
        assert!(!bead_settled((100.0, 100.0), (0.0, 1.0), (100.0, 100.0)));
    }

    #[test]
    fn two_labels_that_would_overlap_stack_the_second_above_the_first() {
        let first = (100.0, 120.0, 200.0);
        // The same place, then close enough to touch from below and from above.
        assert_eq!(stacked_y(150.0, 120.0, 200.0, &[first]), 200.0 - LABEL_STEP);
        assert_eq!(stacked_y(150.0, 120.0, 210.0, &[first]), 200.0 - LABEL_STEP);
        assert_eq!(stacked_y(150.0, 120.0, 190.0, &[first]), 200.0 - LABEL_STEP);
        // Clear of it in either direction, it stays where it is.
        assert_eq!(stacked_y(150.0, 120.0, 200.0 - LABEL_STEP, &[first]), 200.0 - LABEL_STEP);
        assert_eq!(stacked_y(150.0, 120.0, 200.0 + LABEL_STEP, &[first]), 200.0 + LABEL_STEP);
        assert_eq!(stacked_y(220.0, 120.0, 200.0, &[first]), 200.0, "beside it, not over it");
        // A third goes above both, even when the space between would not hold it.
        let second = (150.0, 120.0, 200.0 - LABEL_STEP);
        assert_eq!(
            stacked_y(120.0, 120.0, 195.0, &[first, second]),
            200.0 - 2.0 * LABEL_STEP
        );
    }

    #[test]
    fn the_demo_clears_by_characters_and_stops_at_nothing() {
        assert_eq!(without_last("abc", 2), "a");
        assert_eq!(without_last("a", 2), "");
        assert_eq!(without_last("", 2), "");
        assert_eq!(without_last("caf\u{e9}s", 2), "caf");
    }
}
