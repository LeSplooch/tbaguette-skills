//! Drawing: the palette read from the stylesheet's tokens, and every layer of
//! a frame (dust, anatomy ring, family names, wheel bands, edges and their
//! travellers, nodes, labels, the trace comet), plus hit testing (graph.js
//! `readPalette` .. `pick`). Same canvas calls, same order, same numbers.

use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d as Ctx, HtmlCanvasElement};

use super::layout::{
    LinkKind, PI, TAU, angle_diff, angle_norm, clamp, cubic_at, ease_in_out_cubic, ease_out_cubic,
    js_hypot, lerp,
};
use super::model::{Role, ref_count};
use super::text::{js_round, num};
use super::{App, EmphKind, Hit, document};

type R = Result<(), JsValue>;

#[derive(Clone)]
pub(crate) struct Palette {
    pub bg: [f64; 4],
    pub surface: [f64; 4],
    pub text: [f64; 4],
    pub text2: [f64; 4],
    pub foil: [f64; 4],
    pub focus: [f64; 4],
    pub dark: bool,
    pub cats: Vec<[f64; 4]>,
}

pub(crate) fn rgba(c: &[f64; 4], a: Option<f64>) -> String {
    format!(
        "rgba({},{},{},{})",
        num(c[0]),
        num(c[1]),
        num(c[2]),
        num(a.unwrap_or(c[3]))
    )
}

fn dash(values: &[f64]) -> JsValue {
    let a = Array::new();
    for v in values {
        a.push(&JsValue::from_f64(*v));
    }
    a.into()
}

const DISPLAY: &str = "\"Fraunces\", ui-serif, Georgia, serif";
const MONO: &str = "\"IBM Plex Mono\", ui-monospace, monospace";
const BODY: &str = "\"Work Sans\", ui-sans-serif, system-ui, sans-serif";

pub(crate) fn label_font(weight: u32, size: f64, family: &str) -> String {
    let f = match family {
        "display" => DISPLAY,
        "mono" => MONO,
        _ => BODY,
    };
    format!("{weight} {}px {f}", num(size))
}

pub(crate) fn boxes_meet(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1]
}
pub(crate) fn box_meets_disc(b: &[f64; 4], x: f64, y: f64, r: f64) -> bool {
    let dx = x - clamp(x, b[0], b[2]);
    let dy = y - clamp(y, b[1], b[3]);
    dx * dx + dy * dy < r * r
}

fn width(ctx: &Ctx, s: &str) -> Result<f64, JsValue> {
    Ok(ctx.measure_text(s)?.width())
}

const FAMILY_RINGS: [f64; 5] = [0.0, 0.55, 1.0, 1.5, 2.1];
const FAMILY_TURNS: usize = 12;

struct FamLayout {
    lines: Vec<String>,
    w: f64,
    h: f64,
}

struct Fam {
    c: usize,
    cx: f64,
    cy: f64,
    layouts: Vec<FamLayout>,
    d: f64,
    show: bool,
    x: f64,
    y: f64,
    bx: [f64; 4],
    layout: usize,
}

struct FamSpot {
    k: usize,
    x: f64,
    y: f64,
    layout: usize,
    bias: f64,
    bx: [f64; 4],
}

// graph.js `drawRoleMark`, argument for argument.
#[allow(clippy::too_many_arguments)]
fn draw_role_mark(
    ctx: &Ctx,
    role: Role,
    x: f64,
    y: f64,
    r: f64,
    color: &str,
    bg: &str,
    alpha: f64,
) -> R {
    ctx.set_global_alpha(alpha);
    if role == Role::Crossroads {
        ctx.begin_path();
        ctx.arc(x, y, r, 0.0, TAU)?;
        ctx.set_fill_style_str(color);
        ctx.fill();
        ctx.begin_path();
        ctx.arc(x, y, r * 0.42, 0.0, TAU)?;
        ctx.set_fill_style_str(bg);
        ctx.fill();
        ctx.begin_path();
        ctx.arc(x, y, r + 3.2, 0.0, TAU)?;
        ctx.set_stroke_style_str(color);
        ctx.set_line_width(1.0);
        ctx.stroke();
    } else {
        ctx.begin_path();
        ctx.arc(x, y, r, 0.0, TAU)?;
        ctx.set_fill_style_str(color);
        ctx.fill();
        if role == Role::Foundation {
            ctx.begin_path();
            ctx.arc(x, y, r + 3.2, 0.0, TAU)?;
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.3);
            ctx.stroke();
        } else if role == Role::Signpost {
            ctx.begin_path();
            for i in 0..8 {
                let a = f64::from(i) * TAU / 8.0;
                ctx.move_to(x + a.cos() * (r + 2.0), y + a.sin() * (r + 2.0));
                ctx.line_to(x + a.cos() * (r + 5.0), y + a.sin() * (r + 5.0));
            }
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.2);
            ctx.set_line_cap("round");
            ctx.stroke();
        }
    }
    ctx.set_global_alpha(1.0);
    Ok(())
}

/// How far past its disc a skill's mark draws (outer edge, stroke included).
fn mark_reach(always_on: bool, sr: f64, role: Role) -> f64 {
    if always_on && sr > 2.0 {
        return 7.6;
    }
    match role {
        Role::Signpost => 5.6,
        Role::Foundation | Role::Crossroads => 3.9,
        Role::Specialist => 0.0,
    }
}

impl App {
    fn parse_color(&self, value: &str, fallback: &str) -> [f64; 4] {
        let p = &self.probe;
        p.set_fill_style_str("#000");
        let v = super::model::js_trim(value);
        p.set_fill_style_str(if v.is_empty() { fallback } else { v });
        let v = p.fill_style().as_string().unwrap_or_default();
        if let Some(hex) = v.strip_prefix('#') {
            let part = |i: usize| {
                hex.get(i..i + 2)
                    .and_then(|h| u8::from_str_radix(h, 16).ok())
                    .map_or(f64::NAN, f64::from)
            };
            return [part(0), part(2), part(4), 1.0];
        }
        let mut nums: Vec<f64> = Vec::new();
        let mut cur = String::new();
        for ch in v.chars().chain(std::iter::once(' ')) {
            if ch.is_ascii_digit() || ch == '.' {
                cur.push(ch);
            } else if !cur.is_empty() {
                nums.push(cur.parse().unwrap_or(f64::NAN));
                cur.clear();
            }
        }
        if nums.is_empty() {
            return [0.0, 0.0, 0.0, 1.0];
        }
        let at = |i: usize| nums.get(i).copied().unwrap_or(f64::NAN);
        [
            at(0),
            at(1),
            at(2),
            if nums.len() > 3 { at(3) } else { 1.0 },
        ]
    }

    /// The stylesheet's tokens, read off the crumb root, and the glow sprites.
    pub(crate) fn read_palette(&mut self) {
        let Some(cs) =
            web_sys::window().and_then(|w| w.get_computed_style(&self.root).ok().flatten())
        else {
            return;
        };
        let get = |name: &str, fb: &str| {
            self.parse_color(&cs.get_property_value(name).unwrap_or_default(), fb)
        };
        let count = self.model.as_ref().map_or(0, |m| m.categories.len());
        let dark = document()
            .ok()
            .and_then(|d| d.document_element())
            .and_then(|h| h.get_attribute("data-theme"))
            .as_deref()
            != Some("flour");
        let pal = Palette {
            bg: get("--bg", "#1a130f"),
            surface: get("--surface", "#2b2015"),
            text: get("--text-primary", "#f5ead9"),
            text2: get("--text-secondary", "#cbb79c"),
            foil: get("--accent-solid", "#dda25c"),
            focus: get("--focus-ring", "#e8b876"),
            dark,
            cats: (1..=count)
                .map(|i| get(&format!("--graph-cat-{i}"), "#dda25c"))
                .collect(),
        };
        self.sprites = pal
            .cats
            .iter()
            .filter_map(|c| glow_sprite(c, pal.dark).ok())
            .collect();
        self.foil_sprite = glow_sprite(&pal.foil, pal.dark).ok();
        self.palette = Some(pal);
    }

    pub(crate) fn draw(&mut self, now: f64) -> R {
        let Some(pal) = self.palette.clone() else {
            return Ok(());
        };
        let ctx = self.ctx.clone();
        let t = (now - self.t0) / 1000.0;
        ctx.set_transform(self.dpr, 0.0, 0.0, self.dpr, 0.0, 0.0)?;
        ctx.clear_rect(0.0, 0.0, self.w, self.h);
        self.draw_dust(&ctx, &pal, t)?;
        let bob = if self.reduced { 0.0 } else { 1.0 };
        let anatomy = self.view == "anatomy";
        let zoom_scale = if anatomy {
            self.cam.k
        } else {
            self.cam.k.powf(0.55)
        };
        let focus = self.focus;
        let cam = self.cam;
        let (w, h) = (self.w, self.h);
        if let Some(model) = self.model.as_mut() {
            for (i, n) in model.nodes.iter_mut().enumerate() {
                let sx = (n.px - cam.x) * cam.k + w / 2.0;
                let sy = (n.py - cam.y) * cam.k + h / 2.0;
                let wobble = if anatomy && Some(i) == focus {
                    0.0
                } else {
                    bob * 1.6
                };
                n.sx = sx + (t * 0.7 + n.phase).cos() * wobble;
                n.sy = sy + (t * 0.9 + n.phase * 1.3).sin() * wobble;
                n.sr = n.pr * zoom_scale;
            }
        }
        self.overlays = Some(self.overlay_boxes());
        if anatomy || self.ring > 0.01 {
            self.draw_anatomy_under(&ctx, &pal, t)?;
        }
        if !anatomy {
            self.family_boxes = None;
            self.draw_family_labels(&ctx, &pal)?;
            if self.mix > 0.01 {
                self.draw_wheel_bands(&ctx, &pal)?;
            }
            self.draw_edges(&ctx, &pal, t)?;
        }
        self.draw_nodes(&ctx, &pal, t)?;
        if anatomy {
            self.draw_anatomy_over(&ctx, &pal)?;
        } else {
            self.draw_labels(&ctx, &pal)?;
        }
        if !anatomy && self.trace.as_ref().is_some_and(|tr| tr.path.is_some()) {
            self.draw_trace_comet(&ctx, &pal, t)?;
        }
        Ok(())
    }

    fn draw_dust(&self, ctx: &Ctx, pal: &Palette, t: f64) -> R {
        if self.reduced {
            return Ok(());
        }
        let c = &pal.text2;
        for (i, d) in self.dust.iter().enumerate() {
            let y = ((d.y - t * d.s * 0.012) % 1.0 + 1.0) % 1.0;
            let x = d.x + (t * 0.2 + i as f64).sin() * 0.006;
            ctx.set_fill_style_str(&rgba(c, Some(d.a * if pal.dark { 1.0 } else { 0.7 })));
            ctx.begin_path();
            ctx.arc(x * self.w, y * self.h, d.r, 0.0, TAU)?;
            ctx.fill();
        }
        Ok(())
    }

    /// One edge's cubic, from the citing skill to the cited, and its chord.
    fn edge_curve(&self, e: usize) -> ([f64; 8], f64) {
        let Some(model) = &self.model else {
            return ([0.0; 8], 1.0);
        };
        let edge = &model.edges[e];
        let (s, t) = (&model.nodes[edge.s], &model.nodes[edge.t]);
        let (x0, y0, x3, y3) = (s.sx, s.sy, t.sx, t.sy);
        let dx = x3 - x0;
        let dy = y3 - y0;
        let mut len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 || len.is_nan() {
            len = 1.0;
        }
        let bend = 0.16;
        let qx = (x0 + x3) / 2.0 - dy * bend;
        let qy = (y0 + y3) / 2.0 + dx * bend;
        let mut c1x = x0 + (qx - x0) * 2.0 / 3.0;
        let mut c1y = y0 + (qy - y0) * 2.0 / 3.0;
        let mut c2x = x3 + (qx - x3) * 2.0 / 3.0;
        let mut c2y = y3 + (qy - y3) * 2.0 / 3.0;
        if self.mix > 0.001 {
            let o = self.to_screen(0.0, 0.0);
            let or0 = |a: f64| if a == 0.0 || a.is_nan() { 0.0 } else { a };
            let sep = angle_diff(or0(s.wa), or0(t.wa)).abs() / PI;
            let beta = 0.12 + 0.62 * (1.0 - sep);
            let w1x = o.0 + (x0 - o.0) * beta;
            let w1y = o.1 + (y0 - o.1) * beta;
            let w2x = o.0 + (x3 - o.0) * beta;
            let w2y = o.1 + (y3 - o.1) * beta;
            let m = ease_in_out_cubic(self.mix);
            c1x = lerp(c1x, w1x, m);
            c1y = lerp(c1y, w1y, m);
            c2x = lerp(c2x, w2x, m);
            c2y = lerp(c2y, w2y, m);
        }
        ([x0, y0, c1x, c1y, c2x, c2y, x3, y3], len)
    }

    fn draw_edges(&mut self, ctx: &Ctx, pal: &Palette, t: f64) -> R {
        let fade = self.edge_fade;
        if fade <= 0.0 {
            return Ok(());
        }
        let Some(model) = &self.model else {
            return Ok(());
        };
        let emph = self.emph.as_ref();
        let wheel = self.mix;
        let mut lit = Vec::new();
        let mut lens = vec![f64::NAN; model.edges.len()];
        ctx.set_line_cap("round");
        for (i, e) in model.edges.iter().enumerate() {
            let on = emph.is_none_or(|em| em.edges[i]);
            if emph.is_some() && on {
                lit.push(i);
                continue;
            }
            let (s, tn) = (&model.nodes[e.s], &model.nodes[e.t]);
            let mut alpha = (if emph.is_some() {
                0.035
            } else {
                0.09 + f64::min(0.16, e.w * 0.03)
            }) * fade;
            alpha /= f64::max(1.0, f64::min(s.out_deg as f64, 40.0) / 8.0).sqrt();
            alpha *= f64::min(s.pa, tn.pa) * 0.6 + 0.4;
            if emph.is_none() && pal.dark {
                alpha *= 1.6;
            }
            if alpha < 0.004 {
                continue;
            }
            let (c, len) = self.edge_curve(i);
            lens[i] = len;
            ctx.begin_path();
            ctx.move_to(c[0], c[1]);
            ctx.bezier_curve_to(c[2], c[3], c[4], c[5], c[6], c[7]);
            ctx.set_stroke_style_str(&rgba(
                &pal.cats[s.cat],
                Some(alpha * if wheel > 0.5 { 1.3 } else { 1.0 }),
            ));
            ctx.set_line_width(0.7 + f64::min(1.6, (1.0 + e.w).ln() * 0.55));
            ctx.stroke();
        }
        let trace = emph.is_some_and(|em| em.kind == EmphKind::Trace);
        for &i in &lit {
            let e = &model.edges[i];
            let (c, len) = self.edge_curve(i);
            lens[i] = len;
            ctx.begin_path();
            ctx.move_to(c[0], c[1]);
            ctx.bezier_curve_to(c[2], c[3], c[4], c[5], c[6], c[7]);
            ctx.set_stroke_style_str(&if trace {
                rgba(&pal.foil, Some(0.9 * fade))
            } else {
                rgba(&pal.cats[model.nodes[e.s].cat], Some(0.62 * fade))
            });
            ctx.set_line_width(if trace {
                2.6
            } else {
                1.1 + f64::min(2.2, (1.0 + e.w).ln() * 0.8)
            });
            ctx.stroke();
        }
        if self.reduced {
            self.store_lens(&lens);
            return Ok(());
        }
        // Travellers, batched by family colour (graph.js's object keys:
        // ascending integer order).
        let mut batches: std::collections::BTreeMap<usize, Vec<f64>> =
            std::collections::BTreeMap::new();
        let speed = 34.0;
        for (i, e) in model.edges.iter().enumerate() {
            let is_lit = emph.is_some_and(|em| em.edges[i]);
            if emph.is_some() && !is_lit {
                continue;
            }
            if emph.is_none() && (model.nodes[e.s].pa < 0.5 || model.nodes[e.t].pa < 0.5) {
                continue;
            }
            let (c, len) = self.edge_curve(i);
            lens[i] = len;
            let count = if is_lit { if trace { 0 } else { 3 } } else { 1 };
            for p in 0..count {
                let ph =
                    ((t * speed / f64::max(40.0, len) + e.phase + f64::from(p) / f64::from(count))
                        % 1.0
                        + 1.0)
                        % 1.0;
                let x = cubic_at(c[0], c[2], c[4], c[6], ph);
                let y = cubic_at(c[1], c[3], c[5], c[7], ph);
                let b = batches.entry(model.nodes[e.s].cat).or_default();
                b.extend([
                    x,
                    y,
                    if is_lit { 1.9 } else { 1.2 },
                    if is_lit { 1.0 } else { 0.55 * fade },
                ]);
            }
        }
        ctx.set_global_composite_operation(if pal.dark { "lighter" } else { "source-over" })?;
        for (k, b) in &batches {
            let color = &pal.cats[*k];
            for j in (0..b.len()).step_by(4) {
                ctx.set_fill_style_str(&rgba(color, Some(b[j + 3])));
                ctx.begin_path();
                ctx.arc(b[j], b[j + 1], b[j + 2], 0.0, TAU)?;
                ctx.fill();
            }
        }
        ctx.set_global_composite_operation("source-over")?;
        self.store_lens(&lens);
        Ok(())
    }

    fn store_lens(&mut self, lens: &[f64]) {
        if let Some(model) = self.model.as_mut() {
            for (e, l) in model.edges.iter_mut().zip(lens) {
                if !l.is_nan() {
                    e.len = *l;
                }
            }
        }
    }

    fn draw_nodes(&self, ctx: &Ctx, pal: &Palette, t: f64) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let anatomy = self.view == "anatomy";
        let emph = self.emph.as_ref();
        ctx.set_global_composite_operation(if pal.dark { "lighter" } else { "source-over" })?;
        for (i, n) in model.nodes.iter().enumerate() {
            if n.pa < 0.02 || n.sr < 0.3 || (anatomy && Some(i) == self.focus) {
                continue;
            }
            let lit = emph.is_some_and(|em| em.nodes[i] == 1.0) || Some(i) == self.hover;
            let g = n.sr * if lit { 5.2 } else { 4.0 };
            ctx.set_global_alpha(n.pa * if lit { 0.95 } else { 0.55 });
            if let Some(sprite) = self.sprites.get(n.cat) {
                ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                    sprite,
                    n.sx - g,
                    n.sy - g,
                    g * 2.0,
                    g * 2.0,
                )?;
            }
        }
        ctx.set_global_alpha(1.0);
        ctx.set_global_composite_operation("source-over")?;
        let bg = rgba(&pal.bg, None);
        let now_ms = js_sys::Date::now();
        for (i, n) in model.nodes.iter().enumerate() {
            if n.pa < 0.02 || n.sr < 0.3 || (anatomy && Some(i) == self.focus) {
                continue;
            }
            let color = rgba(&pal.cats[n.cat], None);
            let r = n.sr * if Some(i) == self.hover { 1.18 } else { 1.0 };
            ctx.set_global_alpha(n.pa);
            ctx.begin_path();
            ctx.arc(n.sx, n.sy, r + 1.6, 0.0, TAU)?;
            ctx.set_fill_style_str(&bg);
            ctx.fill();
            if anatomy {
                let item = self
                    .anat
                    .as_ref()
                    .and_then(|l| l.item_of(i).map(|it| &l.neighbors[it]));
                match item {
                    Some(item) if item.out.is_none() => {
                        ctx.begin_path();
                        ctx.arc(n.sx, n.sy, f64::max(0.1, r - 0.8), 0.0, TAU)?;
                        ctx.set_stroke_style_str(&color);
                        ctx.set_line_width(2.0);
                        ctx.stroke();
                    }
                    Some(item) if item.inn.is_some() => {
                        ctx.begin_path();
                        ctx.arc(n.sx, n.sy, r * 0.72, 0.0, TAU)?;
                        ctx.set_fill_style_str(&color);
                        ctx.fill();
                        ctx.begin_path();
                        ctx.arc(n.sx, n.sy, r + 0.2, 0.0, TAU)?;
                        ctx.set_stroke_style_str(&color);
                        ctx.set_line_width(1.3);
                        ctx.stroke();
                    }
                    _ => {
                        ctx.begin_path();
                        ctx.arc(n.sx, n.sy, r, 0.0, TAU)?;
                        ctx.set_fill_style_str(&color);
                        ctx.fill();
                    }
                }
            } else {
                draw_role_mark(ctx, n.role, n.sx, n.sy, r, &color, &bg, n.pa)?;
            }
            ctx.set_global_alpha(n.pa);
            if !anatomy && n.always_on && r > 2.0 {
                ctx.save();
                ctx.set_line_dash(&dash(&[2.5, 3.5]))?;
                ctx.set_line_dash_offset(if self.reduced { 0.0 } else { -t * 6.0 });
                ctx.begin_path();
                ctx.arc(n.sx, n.sy, r + 7.0, 0.0, TAU)?;
                ctx.set_stroke_style_str(&rgba(&pal.foil, Some(0.75 * n.pa)));
                ctx.set_line_width(1.1);
                ctx.stroke();
                ctx.restore();
            }
            let fresh = n.change_status.as_deref().is_some_and(|s| !s.is_empty())
                && n.change_at.as_deref().is_some_and(|s| !s.is_empty())
                && !n.change_ms.is_nan()
                && now_ms - n.change_ms <= 48.0 * 3600.0 * 1000.0;
            if fresh && r > 1.5 {
                let ph = if self.reduced {
                    0.4
                } else {
                    (t * 0.55 + n.phase) % 1.0
                };
                ctx.begin_path();
                ctx.arc(n.sx, n.sy, r + 4.0 + ph * 12.0, 0.0, TAU)?;
                ctx.set_stroke_style_str(&rgba(&pal.foil, Some((1.0 - ph) * 0.8 * n.pa)));
                ctx.set_line_width(1.5);
                ctx.stroke();
            }
            if Some(i) == self.kb_node && self.stage_focused {
                ctx.begin_path();
                ctx.arc(n.sx, n.sy, r + 5.0, 0.0, TAU)?;
                ctx.set_stroke_style_str(&rgba(&pal.focus, None));
                ctx.set_line_width(2.0);
                ctx.stroke();
            }
            ctx.set_global_alpha(1.0);
        }
        Ok(())
    }

    /// Greedy placement: the most important labels claim space first.
    fn draw_labels(&mut self, ctx: &Ctx, pal: &Palette) -> R {
        let mut boxes: Vec<[f64; 4]> = match (&self.family_boxes, self.mix < 0.5) {
            (Some(f), true) => f.clone(),
            _ => Vec::new(),
        };
        boxes.extend(self.overlays.clone().unwrap_or_default());
        let k = self.cam.k;
        self.label_boxes = Vec::new();
        let wheel = self.mix > 0.5 && self.wheel.as_ref().is_some_and(|w| w.labels_all);
        let budget = if wheel {
            999.0
        } else {
            clamp(js_round(10.0 + (k - 0.6) * 40.0), 8.0, 999.0)
        };
        ctx.set_font(&label_font(500, if wheel { 10.5 } else { 12.0 }, "body"));
        ctx.set_text_baseline("middle");
        ctx.set_line_join("round");
        let emph = self.emph.as_ref();
        let mut order = self.label_priority.clone();
        if let Some(em) = emph {
            order.sort_by_key(|&n| std::cmp::Reverse(u8::from(em.nodes[n] != 0.0)));
        }
        if let Some(h) = self.hover {
            order.insert(0, h);
        }
        if let Some(kb) = self.kb_node {
            order.insert(0, kb);
        }
        if self.mix > 0.5
            && !wheel
            && emph.is_none()
            && self.hover.is_none()
            && self.kb_node.is_none()
        {
            return Ok(());
        }
        let Some(model) = &self.model else {
            return Ok(());
        };
        let mut drawn = 0.0;
        let mut seen = vec![false; model.nodes.len()];
        let mut placed: Vec<(usize, [f64; 4])> = Vec::new();
        let mut spots_chosen: Vec<(usize, i32)> = Vec::new();
        for &i in &order {
            if seen[i] {
                continue;
            }
            seen[i] = true;
            let n = &model.nodes[i];
            if n.pa < 0.3 {
                continue;
            }
            let forced = Some(i) == self.hover
                || Some(i) == self.kb_node
                || emph.is_some_and(|em| em.kind == EmphKind::Trace && em.nodes[i] != 0.0);
            let important = forced || emph.is_some_and(|em| em.nodes[i] == 1.0);
            if !important && drawn >= budget {
                continue;
            }
            if emph.is_some_and(|em| em.nodes[i] == 0.0) {
                continue;
            }
            let w = width(ctx, &n.name)?;
            if wheel {
                if !important && self.compact {
                    continue;
                }
                self.draw_radial_label(
                    ctx,
                    pal,
                    &n.name,
                    n.sx,
                    n.sy,
                    n.wa,
                    n.sr + 6.0,
                    if important { &pal.text } else { &pal.text2 },
                    important,
                    10.5,
                )?;
                drawn += 1.0;
                continue;
            }
            let reach = n.sr
                + if self.view == "anatomy" {
                    0.0
                } else {
                    mark_reach(n.always_on, n.sr, n.role)
                };
            let spots = [
                [n.sx + reach + 6.0, n.sy],
                [n.sx - reach - 6.0 - w, n.sy],
                [n.sx - w / 2.0, n.sy - reach - 11.0],
                [n.sx - w / 2.0, n.sy + reach + 11.0],
            ];
            let tries: Vec<i32> = if n.label_spot > 0 {
                vec![n.label_spot, 0, 1, 2, 3]
            } else {
                vec![0, 1, 2, 3]
            };
            let mut chosen: i32 = -1;
            let mut bx = [0.0; 4];
            for &s in &tries {
                if chosen >= 0 {
                    break;
                }
                let sp = spots[s as usize];
                let b = [sp[0] - 3.0, sp[1] - 8.0, sp[0] + w + 3.0, sp[1] + 8.0];
                if self.label_fits(&b, i, &boxes) {
                    chosen = s;
                    bx = b;
                }
            }
            if chosen < 0 {
                if !forced {
                    continue;
                }
                chosen = if spots[0][0] + w > self.w - 8.0 { 1 } else { 0 };
                let sp = spots[chosen as usize];
                bx = [sp[0] - 3.0, sp[1] - 8.0, sp[0] + w + 3.0, sp[1] + 8.0];
            }
            spots_chosen.push((i, chosen));
            let (x, y) = (spots[chosen as usize][0], spots[chosen as usize][1]);
            boxes.push(bx);
            placed.push((i, bx));
            ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.85)));
            ctx.set_line_width(3.5);
            ctx.set_global_alpha(n.pa);
            ctx.stroke_text(&n.name, x, y)?;
            ctx.set_fill_style_str(&rgba(if important { &pal.text } else { &pal.text2 }, None));
            ctx.fill_text(&n.name, x, y)?;
            ctx.set_global_alpha(1.0);
            drawn += 1.0;
            // graph.js writes labelSpot as it goes; a later label's fit test
            // never reads it, so writing after the loop is the same.
        }
        self.label_boxes = placed;
        if let Some(model) = self.model.as_mut() {
            for (i, s) in spots_chosen {
                model.nodes[i].label_spot = s;
            }
        }
        Ok(())
    }

    fn label_fits(&self, b: &[f64; 4], owner: usize, boxes: &[[f64; 4]]) -> bool {
        if b[0] < 4.0 || b[1] < 4.0 || b[2] > self.w - 8.0 || b[3] > self.h - 4.0 {
            return false;
        }
        if boxes.iter().any(|q| boxes_meet(b, q)) {
            return false;
        }
        let Some(model) = &self.model else {
            return true;
        };
        !model
            .nodes
            .iter()
            .enumerate()
            .any(|(i, m)| i != owner && m.pa >= 0.3 && box_meets_disc(b, m.sx, m.sy, m.sr + 2.0))
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_radial_label(
        &self,
        ctx: &Ctx,
        pal: &Palette,
        text: &str,
        x: f64,
        y: f64,
        angle: f64,
        offset: f64,
        color: &[f64; 4],
        strong: bool,
        size: f64,
    ) -> R {
        ctx.save();
        ctx.set_font(&label_font(if strong { 600 } else { 500 }, size, "body"));
        ctx.translate(x, y)?;
        let flip = angle.cos() < 0.0;
        ctx.rotate(if flip { angle + PI } else { angle })?;
        ctx.set_text_align(if flip { "right" } else { "left" });
        ctx.set_text_baseline("middle");
        ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.8)));
        ctx.set_line_width(3.0);
        let dx = if flip { -offset } else { offset };
        ctx.stroke_text(text, dx, 0.0)?;
        ctx.set_fill_style_str(&rgba(color, None));
        ctx.fill_text(text, dx, 0.0)?;
        ctx.restore();
        Ok(())
    }

    fn draw_family_labels(&mut self, ctx: &Ctx, pal: &Palette) -> R {
        let a = (1.0 - self.mix) * self.edge_fade;
        if a < 0.02 {
            return Ok(());
        }
        ctx.set_font(&format!(
            "italic 500 {}px {DISPLAY}",
            if self.compact { 15 } else { 19 }
        ));
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let line_h = js_round((if self.compact { 15.0 } else { 19.0 }) * 1.2);
        let labels = self.place_family_labels(ctx, line_h)?;
        self.family_boxes = Some(labels.iter().filter(|l| l.show).map(|l| l.bx).collect());
        let Some(model) = &self.model else {
            return Ok(());
        };
        let snap = self.reduced;
        for l in &labels {
            let slug = &model.categories[l.c].slug;
            let target = if l.show { 1.0 } else { 0.0 };
            let v = match self.family_vis.get(slug) {
                Some(prev) if !snap => clamp(prev + if l.show { 0.12 } else { -0.12 }, 0.0, 1.0),
                _ => target,
            };
            self.family_vis.insert(slug.clone(), v);
            if v <= 0.0 {
                continue;
            }
            let dim = self.emph.as_ref().is_some_and(|em| {
                !model.categories[l.c]
                    .nodes
                    .iter()
                    .any(|&n| em.nodes[n] == 1.0)
            });
            ctx.set_global_alpha(a * if dim { 0.25 } else { 0.85 } * v);
            ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.7)));
            ctx.set_line_width(4.0);
            ctx.set_line_join("round");
            ctx.set_fill_style_str(&rgba(&pal.text2, None));
            let lay = &l.layouts[l.layout];
            let rows = lay.lines.len() as f64;
            let top = l.y - (rows - 1.0) * line_h / 2.0;
            for (i, line) in lay.lines.iter().enumerate() {
                ctx.stroke_text(line, l.x, top + i as f64 * line_h)?;
                ctx.fill_text(line, l.x, top + i as f64 * line_h)?;
            }
            let w = f64::min(lay.w, 56.0);
            ctx.set_fill_style_str(&rgba(&pal.cats[l.c], None));
            ctx.fill_rect(l.x - w / 2.0, top + (rows - 1.0) * line_h + 12.0, w, 2.0);
        }
        ctx.set_global_alpha(1.0);
        ctx.set_text_align("left");
        Ok(())
    }

    /// Each family's name where it crosses the fewest skills and no other
    /// name or overlay, near its own core; bigger families choose first.
    fn place_family_labels(&mut self, ctx: &Ctx, line_h: f64) -> Result<Vec<Fam>, JsValue> {
        let band = self.overlay_margins(10.0);
        let overlays = self.overlays.clone().unwrap_or_default();
        let frozen = self.emph.is_some();
        let (w, h) = (self.w, self.h);
        let Some(model) = &self.model else {
            return Ok(Vec::new());
        };
        let layout = |lines: Vec<String>| -> Result<FamLayout, JsValue> {
            let mut w = 0.0f64;
            for line in &lines {
                w = f64::max(w, width(ctx, line)?);
            }
            let h = 22.0 + (lines.len() as f64 - 1.0) * line_h;
            Ok(FamLayout { lines, w, h })
        };
        let mut fams: Vec<Fam> = Vec::new();
        for c in model.categories.iter().filter(|c| !c.nodes.is_empty()) {
            let (mut x, mut y, mut wsum, mut spread) = (0.0, 0.0, 0.0, 0.0);
            for &n in &c.nodes {
                let node = &model.nodes[n];
                let wt = 1.0 / (1.0 + node.degree as f64 / 6.0);
                x += node.sx * wt;
                y += node.sy * wt;
                wsum += wt;
            }
            x /= wsum;
            y /= wsum;
            for &n in &c.nodes {
                spread += js_hypot(model.nodes[n].sx - x, model.nodes[n].sy - y);
            }
            spread /= c.nodes.len() as f64;
            let words: Vec<&str> = c.title.split(' ').collect();
            let mut layouts = vec![layout(vec![c.title.clone()])?];
            let mut split: Option<FamLayout> = None;
            for i in 1..words.len() {
                let two = layout(vec![words[..i].join(" "), words[i..].join(" ")])?;
                if split.as_ref().is_none_or(|s| two.w < s.w) {
                    split = Some(two);
                }
            }
            if let Some(s) = split {
                layouts.push(s);
            }
            fams.push(Fam {
                c: c.index,
                cx: x,
                cy: y,
                layouts,
                d: f64::min(spread * 0.9, 70.0) + 14.0,
                show: false,
                x: 0.0,
                y: 0.0,
                bx: [0.0; 4],
                layout: 0,
            });
        }
        let mut order: Vec<usize> = (0..fams.len()).collect();
        order.sort_by(|&p, &q| {
            let (cp, cq) = (&model.categories[fams[p].c], &model.categories[fams[q].c]);
            cq.nodes
                .len()
                .cmp(&cp.nodes.len())
                .then(cp.index.cmp(&cq.index))
        });
        let mut placed: Vec<[f64; 4]> = Vec::new();
        for fi in order {
            let f = &fams[fi];
            let slug = model.categories[f.c].slug.clone();
            let reach = f.d * FAMILY_RINGS[FAMILY_RINGS.len() - 1] + f.layouts[0].w + 24.0;
            let near: Vec<usize> = (0..model.nodes.len())
                .filter(|&i| {
                    let n = &model.nodes[i];
                    n.ta >= 0.3 && (n.sx - f.cx).abs() < reach && (n.sy - f.cy).abs() < reach
                })
                .collect();
            let cost = |x: f64, y: f64, b: &[f64; 4]| -> (f64, bool, u32) {
                let (mut hits, mut hard, mut covered) = (0.0, false, 0u32);
                if b[0] < band.left
                    || b[2] > w - band.right
                    || b[1] < band.top
                    || b[3] > h - band.bottom
                {
                    hits += 40.0;
                    hard = true;
                }
                for o in &overlays {
                    if boxes_meet(b, o) {
                        hits += 40.0;
                        hard = true;
                    }
                }
                for p in &placed {
                    if boxes_meet(b, p) {
                        hits += 30.0;
                        hard = true;
                    }
                }
                for &i in &near {
                    let n = &model.nodes[i];
                    if box_meets_disc(b, n.sx, n.sy, n.sr + 3.0) {
                        hits += 1.0 + n.sr / 6.0;
                        if box_meets_disc(b, n.sx, n.sy, n.sr) {
                            covered += 1;
                        }
                    }
                }
                let own = js_hypot(x - f.cx, y - f.cy);
                for (gi, g) in fams.iter().enumerate() {
                    if gi != fi && js_hypot(x - g.cx, y - g.cy) < own {
                        hits += 3.0;
                        hard = true;
                        break;
                    }
                }
                (hits, hard, covered)
            };
            let mut spots: Vec<FamSpot> = Vec::new();
            for (li, l) in f.layouts.iter().enumerate() {
                for (ring, mult) in FAMILY_RINGS.iter().enumerate() {
                    let r = mult * f.d;
                    let turns = if ring == 0 { 1 } else { FAMILY_TURNS };
                    for t in 0..turns {
                        let ang = -PI / 2.0 + t as f64 * TAU / FAMILY_TURNS as f64;
                        let x = f.cx
                            + if r > 0.0 {
                                ang.cos() * (r + l.w / 2.0)
                            } else {
                                0.0
                            };
                        let y = f.cy
                            + if r > 0.0 {
                                ang.sin() * (r + l.h / 2.0)
                            } else {
                                0.0
                            };
                        let rise = (l.lines.len() as f64 - 1.0) * line_h / 2.0;
                        spots.push(FamSpot {
                            k: spots.len(),
                            x,
                            y,
                            layout: li,
                            bias: ring as f64 * 0.35 + li as f64 * 0.6 + t as f64 * 0.002,
                            bx: [
                                x - l.w / 2.0 - 4.0,
                                y - rise - 11.0,
                                x + l.w / 2.0 + 4.0,
                                y + rise + 15.0,
                            ],
                        });
                    }
                }
            }
            let prev = self
                .family_spot
                .iter()
                .find(|(s, _)| *s == slug)
                .map(|(_, k)| *k);
            let was_shown = self
                .family_shown
                .iter()
                .find(|(s, _)| *s == slug)
                .is_some_and(|(_, v)| *v);
            let allow = if was_shown { 2 } else { 1 };
            let (chosen, show) = match prev.filter(|p| frozen && *p < spots.len()) {
                Some(p) => (p, was_shown),
                None => {
                    let (mut best, mut best_score, mut spare, mut spare_score) =
                        (None, f64::INFINITY, None, f64::INFINITY);
                    for spot in &spots {
                        let (hits, hard, covered) = cost(spot.x, spot.y, &spot.bx);
                        let score = hits + spot.bias - if Some(spot.k) == prev { 1.5 } else { 0.0 };
                        if !hard && covered <= allow {
                            if score < best_score {
                                best_score = score;
                                best = Some(spot.k);
                            }
                        } else if score < spare_score {
                            spare_score = score;
                            spare = Some(spot.k);
                        }
                    }
                    (best.or(spare).unwrap_or(0), best.is_some())
                }
            };
            let spot = &spots[chosen];
            let (sx, sy, sb, sl) = (spot.x, spot.y, spot.bx, spot.layout);
            match self.family_spot.iter_mut().find(|(s, _)| *s == slug) {
                Some(e) => e.1 = chosen,
                None => self.family_spot.push((slug.clone(), chosen)),
            }
            match self.family_shown.iter_mut().find(|(s, _)| *s == slug) {
                Some(e) => e.1 = show,
                None => self.family_shown.push((slug.clone(), show)),
            }
            let f = &mut fams[fi];
            f.x = sx;
            f.y = sy;
            f.bx = sb;
            f.layout = sl;
            f.show = show;
            if show {
                placed.push(sb);
            }
        }
        Ok(fams)
    }

    /// The stage's own furniture as boxes in canvas coordinates.
    pub(crate) fn overlay_boxes(&self) -> Vec<[f64; 4]> {
        let stage = self.stage.get_bounding_client_rect();
        let mut boxes = Vec::new();
        let Ok(els) = self.root.query_selector_all("[data-crumb-hint], .crumb__tools, [data-crumb-legend], [data-crumb-trail], [data-crumb-sheet], [data-crumb-key]") else {
            return boxes;
        };
        for i in 0..els.length() {
            let Some(el) = els
                .item(i)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
            else {
                continue;
            };
            if el.hidden() || el.class_list().contains("crumb__hint--gone") {
                continue;
            }
            let r = el.get_bounding_client_rect();
            let b = [
                r.left() - stage.left(),
                r.top() - stage.top(),
                r.right() - stage.left(),
                r.bottom() - stage.top(),
            ];
            if r.width() != 0.0
                && r.height() != 0.0
                && b[2] > 0.0
                && b[0] < self.w
                && b[3] > 0.0
                && b[1] < self.h
            {
                boxes.push(b);
            }
        }
        boxes
    }

    fn draw_wheel_bands(&self, ctx: &Ctx, pal: &Palette) -> R {
        let m = ease_in_out_cubic(self.mix);
        let Some(wheel) = &self.wheel else {
            return Ok(());
        };
        let Some(model) = &self.model else {
            return Ok(());
        };
        let o = self.to_screen(0.0, 0.0);
        let r = wheel.r * self.cam.k;
        ctx.set_line_cap("butt");
        for w in &wheel.cats {
            ctx.begin_path();
            ctx.arc(
                o.0,
                o.1,
                r - 12.0,
                w.a0 + 0.004,
                w.a0 + (w.a1 - w.a0) * m - 0.004,
            )?;
            ctx.set_stroke_style_str(&rgba(&pal.cats[w.cat], Some(0.9 * m)));
            ctx.set_line_width(4.0);
            ctx.stroke();
        }
        if self.compact {
            return Ok(());
        }
        ctx.set_font(&format!("italic 500 13px {DISPLAY}"));
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        for w in &wheel.cats {
            let mid = (w.a0 + w.a1) / 2.0;
            let rr = r - 32.0;
            let cat = &model.categories[w.cat];
            if cat.nodes.len() < 3 {
                continue;
            }
            ctx.set_global_alpha(m * 0.75);
            ctx.set_fill_style_str(&rgba(&pal.text2, None));
            ctx.fill_text(
                &cat.title,
                o.0 + mid.cos() * rr * 0.9,
                o.1 + mid.sin() * rr * 0.9,
            )?;
        }
        ctx.set_global_alpha(1.0);
        ctx.set_text_align("left");
        Ok(())
    }

    pub(crate) fn hub_screen(&self) -> (f64, f64) {
        match (self.focus, &self.model) {
            (Some(f), Some(m)) => (m.nodes[f].sx, m.nodes[f].sy),
            _ => self.to_screen(0.0, 0.0),
        }
    }

    fn draw_anatomy_under(&mut self, ctx: &Ctx, pal: &Palette, t: f64) -> R {
        let (Some(f), Some(model)) = (self.focus, &self.model) else {
            return Ok(());
        };
        let Some(l) = &self.anat else { return Ok(()) };
        let k = self.cam.k;
        let o = self.hub_screen();
        let ring = ease_out_cubic(self.ring);
        let fnode = &model.nodes[f];
        let cat_color = &pal.cats[fnode.cat];
        let active = if self.pin_section >= 0 {
            self.pin_section
        } else {
            self.hover_section
        };
        let hover = self.hover;
        let link_count = l.links.len() as f64;
        let mut geo: Vec<(Option<[f64; 6]>, bool)> = Vec::with_capacity(l.links.len());
        ctx.set_line_cap("round");
        for link in &l.links {
            let item = &l.neighbors[link.item];
            let n = &model.nodes[item.node];
            if n.pa < 0.05 {
                geo.push((link.geo, link.lit));
                continue;
            }
            let (nx, ny) = (n.sx, n.sy);
            let mut lit = if active < 0 {
                hover.is_none() || hover == Some(item.node)
            } else {
                link.arc.is_some_and(|a| a as i32 == active)
            };
            if hover.is_some() && hover != Some(item.node) && hover != Some(f) && active < 0 {
                lit = false;
            }
            let calm = if active < 0 && hover.is_none() {
                clamp(0.6 - link_count * 0.0025, 0.3, 0.6)
            } else {
                0.78
            };
            let alpha = (if lit { calm } else { 0.08 }) * ring;
            let mut color = &pal.cats[n.cat];
            ctx.begin_path();

            let g: [f64; 6] = match link.kind {
                LinkKind::Out => {
                    let ang = link.arc.map_or(0.0, |a| l.arcs[a].mid);
                    let sx = o.0 + ang.cos() * l.r2 * k;
                    let sy = o.1 + ang.sin() * l.r2 * k;
                    let cr = (l.r2 + (l.big_r - l.r2) * 0.55) * k;
                    let cx = o.0 + ang.cos() * cr;
                    let cy = o.1 + ang.sin() * cr;
                    ctx.move_to(sx, sy);
                    ctx.quadratic_curve_to(cx, cy, nx, ny);
                    ctx.set_line_dash(&dash(&[]))?;
                    ctx.set_line_width(0.9 + f64::min(2.4, link.count * 0.7));
                    [sx, sy, cx, cy, nx, ny]
                }
                LinkKind::Trigger => {
                    let ta = (ny - o.1).atan2(nx - o.0);
                    let sx = o.0 + ta.cos() * l.r0 * k;
                    let sy = o.1 + ta.sin() * l.r0 * k;
                    ctx.move_to(sx, sy);
                    ctx.line_to(nx, ny);
                    ctx.set_line_dash(&dash(&[1.0, 4.0]))?;
                    color = &pal.foil;
                    ctx.set_line_width(2.0);
                    [sx, sy, (sx + nx) / 2.0, (sy + ny) / 2.0, nx, ny]
                }
                LinkKind::In => {
                    let ia = (ny - o.1).atan2(nx - o.0);
                    let sx = o.0 + ia.cos() * (l.r2 + 8.0) * k;
                    let sy = o.1 + ia.sin() * (l.r2 + 8.0) * k;
                    let mx = (sx + nx) / 2.0 + (ny - sy) * 0.12;
                    let my = (sy + ny) / 2.0 - (nx - sx) * 0.12;
                    ctx.move_to(nx, ny);
                    ctx.quadratic_curve_to(mx, my, sx, sy);
                    ctx.set_line_dash(&dash(&[3.0, 4.0]))?;
                    ctx.set_line_dash_offset(if self.reduced { 0.0 } else { t * 12.0 });
                    ctx.set_line_width(1.0 + f64::min(1.6, (1.0 + link.count).ln() * 0.6));
                    [nx, ny, mx, my, sx, sy]
                }
            };
            ctx.set_stroke_style_str(&rgba(color, Some(alpha * f64::min(1.0, n.pa + 0.2))));
            ctx.stroke();
            ctx.set_line_dash(&dash(&[]))?;
            ctx.set_line_dash_offset(0.0);
            geo.push((Some(g), lit));
        }
        if !self.reduced && ring > 0.5 {
            ctx.set_global_composite_operation(if pal.dark { "lighter" } else { "source-over" })?;
            for (link, (g, lit)) in l.links.iter().zip(&geo) {
                let (true, Some(g)) = (*lit, g) else { continue };
                let c = if link.kind == LinkKind::Trigger {
                    &pal.foil
                } else {
                    &pal.cats[model.nodes[l.neighbors[link.item].node].cat]
                };
                let n = if link.kind == LinkKind::In {
                    1.0
                } else {
                    f64::min(3.0, link.count)
                };
                let mut p = 0.0;
                while p < n {
                    let ph = ((t * 0.45 + link.phase + p / n) % 1.0 + 1.0) % 1.0;
                    let u = 1.0 - ph;
                    let x = u * u * g[0] + 2.0 * u * ph * g[2] + ph * ph * g[4];
                    let y = u * u * g[1] + 2.0 * u * ph * g[3] + ph * ph * g[5];
                    ctx.set_fill_style_str(&rgba(c, Some(0.95 * (ph * PI).sin())));
                    ctx.begin_path();
                    ctx.arc(x, y, 1.9, 0.0, TAU)?;
                    ctx.fill();
                    p += 1.0;
                }
            }
            ctx.set_global_composite_operation("source-over")?;
        }
        let (r1, r2, r3) = (l.r1 * k, l.r2 * k, (l.r2 + 7.0) * k);
        let arcs_len = l.arcs.len() as f64;
        for (i, arc) in l.arcs.iter().enumerate() {
            let stagger = clamp(
                ring * 1.6 - i as f64 / f64::max(arcs_len, 1.0) * 0.6,
                0.0,
                1.0,
            );
            if stagger <= 0.0 {
                continue;
            }
            let sec = &fnode.sections[arc.index];
            let a0 = arc.a0;
            let a1 = arc.a0 + (arc.a1 - arc.a0) * ease_out_cubic(stagger);
            let cites = sec.refs.len();
            let is_active = i as i32 == active;
            let grow = if is_active { 4.0 } else { 0.0 };
            ctx.begin_path();
            ctx.arc(o.0, o.1, r2 + grow, a0, a1)?;
            ctx.arc_with_anticlockwise(o.0, o.1, r1, a1, a0, true)?;
            ctx.close_path();
            let mut alpha = if sec.reference { 0.45 } else { 0.85 };
            if cites == 0 {
                alpha *= 0.45;
            }
            if active >= 0 && !is_active {
                alpha *= 0.5;
            }
            ctx.set_fill_style_str(&if is_active {
                rgba(&pal.foil, Some(0.95))
            } else {
                rgba(cat_color, Some(alpha))
            });
            ctx.fill();
            if sec.reference {
                ctx.save();
                ctx.clip();
                ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.35)));
                ctx.set_line_width(1.0);
                let mut hh = -r2;
                while hh < r2 * 2.0 {
                    ctx.begin_path();
                    ctx.move_to(o.0 - r2 + hh, o.1 - r2);
                    ctx.line_to(o.0 - r2 + hh - r2, o.1 + r2);
                    ctx.stroke();
                    hh += 5.0;
                }
                ctx.restore();
            }
            for s in &arc.subs {
                let b0 = s.a0;
                let b1 = f64::min(s.a1, a1);
                if b1 <= b0 {
                    continue;
                }
                ctx.begin_path();
                ctx.arc(o.0, o.1, r3 + grow, b0 + 0.01, b1 - 0.01)?;
                let has = !sec.subs[s.sub].refs.is_empty();
                ctx.set_stroke_style_str(&rgba(
                    cat_color,
                    Some((if has { 0.8 } else { 0.35 }) * stagger),
                ));
                ctx.set_line_width(2.0);
                ctx.stroke();
            }
            let span = (arc.a1 - arc.a0) * (r1 + r2) / 2.0;
            if span > 15.0 && stagger > 0.9 {
                let (mid, rr) = (arc.mid, (r1 + r2) / 2.0);
                ctx.set_font(&label_font(500, 10.0, "mono"));
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.set_fill_style_str(&if is_active {
                    rgba(&pal.bg, None)
                } else {
                    rgba(&pal.bg, Some(0.9))
                });
                ctx.fill_text(
                    &(i + 1).to_string(),
                    o.0 + mid.cos() * rr,
                    o.1 + mid.sin() * rr + 0.5,
                )?;
                ctx.set_text_align("left");
            }
        }
        if let Some(l) = self.anat.as_mut() {
            for (link, (g, lit)) in l.links.iter_mut().zip(geo) {
                link.geo = g;
                link.lit = lit;
            }
        }
        Ok(())
    }

    fn draw_anatomy_over(&mut self, ctx: &Ctx, pal: &Palette) -> R {
        let (Some(f), Some(model)) = (self.focus, &self.model) else {
            return Ok(());
        };
        let Some(l) = &self.anat else { return Ok(()) };
        let o = self.hub_screen();
        let k = self.cam.k;
        let fnode = &model.nodes[f];
        let r0 = if fnode.sr == 0.0 || fnode.sr.is_nan() {
            l.r0 * k
        } else {
            fnode.sr
        };
        let g = r0 * 2.6;
        ctx.set_global_composite_operation(if pal.dark { "lighter" } else { "source-over" })?;
        ctx.set_global_alpha(0.6);
        if let Some(sprite) = self.sprites.get(fnode.cat) {
            ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                sprite,
                o.0 - g,
                o.1 - g,
                g * 2.0,
                g * 2.0,
            )?;
        }
        ctx.set_global_alpha(1.0);
        ctx.set_global_composite_operation("source-over")?;
        ctx.begin_path();
        ctx.arc(o.0, o.1, r0, 0.0, TAU)?;
        ctx.set_fill_style_str(&rgba(&pal.surface, None));
        ctx.fill();
        ctx.set_stroke_style_str(&rgba(&pal.cats[fnode.cat], None));
        ctx.set_line_width(2.0);
        ctx.stroke();
        if self.hover == Some(f) {
            ctx.set_stroke_style_str(&rgba(&pal.focus, Some(0.8)));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.arc(o.0, o.1, r0 + 4.0, 0.0, TAU)?;
            ctx.stroke();
        }
        if r0 > 30.0 {
            self.draw_hub_text(ctx, pal, f, o.0, o.1, r0)?;
        }
        let mut labels: Vec<usize> = (0..l.neighbors.len())
            .filter(|&i| model.nodes[l.neighbors[i].node].pa >= 0.3)
            .collect();
        let active = if self.pin_section >= 0 {
            self.pin_section
        } else {
            self.hover_section
        };
        let sec = if active >= 0 {
            fnode.sections.get(active as usize)
        } else {
            None
        };
        let sec_cites =
            |n: usize| sec.is_some_and(|s| ref_count(&s.refs, &model.nodes[n].slug) != 0.0);
        ctx.set_font(&label_font(500, 12.0, "body"));
        ctx.set_text_baseline("middle");
        let (hover, kb) = (self.hover, self.kb_node);
        if l.crowded {
            labels.retain(|&i| {
                let n = l.neighbors[i].node;
                Some(n) == hover || Some(n) == kb || sec_cites(n)
            });
            self.draw_anatomy_bands(ctx, pal, o, k, if labels.is_empty() { 1.0 } else { 0.35 })?;
        } else if self.compact {
            labels.retain(|&i| {
                let item = &l.neighbors[i];
                Some(item.node) == hover
                    || Some(item.node) == kb
                    || sec_cites(item.node)
                    || item.weight >= 2.0
            });
        }
        struct Lab {
            node: usize,
            y: f64,
            right: bool,
        }
        let mut sides: [Vec<Lab>; 2] = [Vec::new(), Vec::new()];
        let mut anat_boxes = Vec::new();
        for &i in &labels {
            let n = &model.nodes[l.neighbors[i].node];
            let right = n.sx >= o.0;
            sides[usize::from(right)].push(Lab {
                node: l.neighbors[i].node,
                y: n.sy,
                right,
            });
        }
        let overlays = self.overlays.clone().unwrap_or_default();
        for arr in &mut sides {
            arr.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
            let gap = 15.0;
            for _ in 0..4 {
                for i in 1..arr.len() {
                    if arr[i].y - arr[i - 1].y < gap {
                        arr[i].y = arr[i - 1].y + gap;
                    }
                }
                for i in (0..arr.len().saturating_sub(1)).rev() {
                    if arr[i + 1].y - arr[i].y < gap {
                        arr[i].y = arr[i + 1].y - gap;
                    }
                }
            }
            for lab in arr.iter() {
                let n = &model.nodes[lab.node];
                let strong = Some(lab.node) == hover || sec_cites(lab.node);
                let x = if lab.right {
                    n.sx + n.sr + 7.0
                } else {
                    n.sx - n.sr - 7.0
                };
                ctx.set_font(&label_font(if strong { 600 } else { 500 }, 12.0, "body"));
                let room = if lab.right { self.w - 6.0 - x } else { x - 6.0 };
                let mut text: Vec<u16> = n.name.encode_utf16().collect();
                if room < 36.0 {
                    continue;
                }
                let as_str = |t: &[u16]| String::from_utf16_lossy(t);
                if width(ctx, &n.name)? > room {
                    while text.len() > 3 && width(ctx, &format!("{}…", as_str(&text)))? > room {
                        text.pop();
                    }
                    text.extend("…".encode_utf16());
                }
                let shown = as_str(&text);
                let tw = width(ctx, &shown)?;
                let bx = if lab.right {
                    [x, lab.y - 8.0, x + tw, lab.y + 8.0]
                } else {
                    [x - tw, lab.y - 8.0, x, lab.y + 8.0]
                };
                if overlays.iter().any(|q| boxes_meet(&bx, q)) {
                    continue;
                }
                anat_boxes.push((lab.node, bx));
                ctx.set_global_alpha(
                    n.pa * if sec.is_some() && !sec_cites(lab.node) {
                        0.4
                    } else {
                        1.0
                    },
                );
                if (lab.y - n.sy).abs() > 3.0 {
                    ctx.begin_path();
                    ctx.move_to(n.sx + if lab.right { n.sr } else { -n.sr }, n.sy);
                    ctx.line_to(x + if lab.right { -2.0 } else { 2.0 }, lab.y);
                    ctx.set_stroke_style_str(&rgba(&pal.text2, Some(0.35)));
                    ctx.set_line_width(1.0);
                    ctx.stroke();
                }
                ctx.set_text_align(if lab.right { "left" } else { "right" });
                ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.85)));
                ctx.set_line_width(3.5);
                ctx.set_line_join("round");
                ctx.stroke_text(&shown, x, lab.y)?;
                ctx.set_fill_style_str(&rgba(if strong { &pal.text } else { &pal.text2 }, None));
                ctx.fill_text(&shown, x, lab.y)?;
                ctx.set_global_alpha(1.0);
            }
        }
        ctx.set_text_align("left");
        self.anat_boxes = anat_boxes;
        Ok(())
    }

    fn draw_anatomy_bands(&self, ctx: &Ctx, pal: &Palette, o: (f64, f64), k: f64, alpha: f64) -> R {
        let (Some(l), Some(model)) = (&self.anat, &self.model) else {
            return Ok(());
        };
        let r = l.big_r * k;
        let ring = ease_out_cubic(self.ring);
        ctx.set_line_cap("round");
        ctx.set_font(&format!("italic 500 12.5px {DISPLAY}"));
        ctx.set_text_baseline("middle");
        for b in &l.bands {
            let a0 = b.a0 + 0.01;
            let a1 = a0 + (b.a1 - b.a0 - 0.02) * ring;
            ctx.begin_path();
            ctx.arc(o.0, o.1, r + 13.0, a0, a1)?;
            ctx.set_stroke_style_str(&rgba(&pal.cats[b.cat], Some(0.85)));
            ctx.set_line_width(3.0);
            ctx.stroke();
        }
        if ring < 0.9 {
            return Ok(());
        }
        let overlays = self.overlays.clone().unwrap_or_default();
        let mut placed: Vec<[f64; 4]> = Vec::new();
        let (w, h, line_h) = (self.w, self.h, 15.0);
        let along = [0.5, 0.32, 0.68, 0.18, 0.82];
        let mut order: Vec<usize> = (0..l.bands.len()).collect();
        order.sort_by(|&p, &q| {
            let (bp, bq) = (&l.bands[p], &l.bands[q]);
            (bq.a1 - bq.a0)
                .partial_cmp(&(bp.a1 - bp.a0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for bi in order {
            let b = &l.bands[bi];
            let sweep = b.a1 - b.a0;
            if sweep * r < 20.0 {
                continue;
            }
            let title = &model.categories[b.cat].title;
            let words: Vec<&str> = title.split(' ').collect();
            let mut layouts: Vec<Vec<String>> = vec![vec![title.clone()]];
            let mut split: Option<(Vec<String>, f64)> = None;
            for wi in 1..words.len() {
                let two = vec![words[..wi].join(" "), words[wi..].join(" ")];
                let two_w = f64::max(width(ctx, &two[0])?, width(ctx, &two[1])?);
                if split.as_ref().is_none_or(|s| two_w < s.1) {
                    split = Some((two, two_w));
                }
            }
            if let Some(s) = split {
                layouts.push(s.0);
            }
            let mut spot: Option<(f64, f64, &str, usize, [f64; 4])> = None;
            'outer: for (li, lines) in layouts.iter().enumerate() {
                let rise = (lines.len() as f64 - 1.0) * line_h / 2.0;
                let mut lw = 0.0f64;
                for line in lines {
                    lw = f64::max(lw, width(ctx, line)?);
                }
                for frac in along {
                    let ang = b.a0 + sweep * frac;
                    let (c, s) = (ang.cos(), ang.sin());
                    let x = o.0 + c * (r + 26.0);
                    let mut y = o.1 + s * (r + 26.0);
                    let align = if c > 0.25 {
                        "left"
                    } else if c < -0.25 {
                        "right"
                    } else {
                        "center"
                    };
                    if align == "center" {
                        y += s * (6.0 + rise);
                    }
                    let left = match align {
                        "left" => x,
                        "right" => x - lw,
                        _ => x - lw / 2.0,
                    };
                    let bx = [left - 6.0, y - rise - 9.0, left + lw + 6.0, y + rise + 9.0];
                    if bx[0] < 4.0 || bx[1] < 4.0 || bx[2] > w - 4.0 || bx[3] > h - 4.0 {
                        continue;
                    }
                    if overlays.iter().any(|q| boxes_meet(&bx, q))
                        || placed.iter().any(|q| boxes_meet(&bx, q))
                    {
                        continue;
                    }
                    spot = Some((x, y - rise, align, li, bx));
                    break 'outer;
                }
            }
            let Some((x, top, align, li, bx)) = spot else {
                continue;
            };
            placed.push(bx);
            ctx.set_global_alpha(alpha);
            ctx.set_text_align(align);
            ctx.set_stroke_style_str(&rgba(&pal.bg, Some(0.8)));
            ctx.set_line_width(3.5);
            ctx.set_line_join("round");
            ctx.set_fill_style_str(&rgba(&pal.text2, None));
            for (i, line) in layouts[li].iter().enumerate() {
                ctx.stroke_text(line, x, top + i as f64 * line_h)?;
                ctx.fill_text(line, x, top + i as f64 * line_h)?;
            }
            ctx.set_global_alpha(1.0);
        }
        ctx.set_text_align("left");
        Ok(())
    }

    fn draw_hub_text(&self, ctx: &Ctx, pal: &Palette, node: usize, x: f64, y: f64, r: f64) -> R {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let n = &model.nodes[node];
        let parts: Vec<&str> = n.name.split('-').collect();
        let max_w = r * 1.55;
        let mut size = clamp(r * 0.3, 11.0, 19.0);
        let mut lines: Vec<String> = Vec::new();
        for _ in 0..8 {
            ctx.set_font(&format!("600 {}px {DISPLAY}", num(size)));
            lines = Vec::new();
            let mut cur = String::new();
            for part in &parts {
                let next = if cur.is_empty() {
                    (*part).to_string()
                } else {
                    format!("{cur}-{part}")
                };
                if !cur.is_empty() && width(ctx, &next)? > max_w {
                    lines.push(format!("{cur}-"));
                    cur = (*part).to_string();
                } else {
                    cur = next;
                }
            }
            lines.push(cur);
            let mut widest = f64::NEG_INFINITY;
            for l in &lines {
                widest = f64::max(widest, width(ctx, l)?);
            }
            if widest <= max_w && lines.len() as f64 * size * 1.1 <= r * 1.3 {
                break;
            }
            size *= 0.9;
        }
        let lh = size * 1.1;
        let top = y - (lines.len() as f64 - 1.0) * lh / 2.0 - if r > 50.0 { 6.0 } else { 0.0 };
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style_str(&rgba(&pal.text, None));
        for (i, l) in lines.iter().enumerate() {
            ctx.fill_text(l, x, top + i as f64 * lh)?;
        }
        if r > 50.0 {
            let cap_y = top + lines.len() as f64 * lh + 4.0;
            let dy = cap_y - y;
            let chord = 2.0 * f64::max(0.0, r * r - dy * dy).sqrt() - 14.0;
            let cap = model.categories[n.cat].title.to_uppercase();
            for fs in [10.0, 9.0, 8.0] {
                ctx.set_font(&label_font(500, fs, "mono"));
                if width(ctx, &cap)? <= chord {
                    ctx.set_fill_style_str(&rgba(&pal.text2, None));
                    ctx.fill_text(&cap, x, cap_y)?;
                    break;
                }
            }
        }
        ctx.set_text_align("left");
        Ok(())
    }

    fn draw_trace_comet(&self, ctx: &Ctx, pal: &Palette, t: f64) -> R {
        if self.reduced {
            return Ok(());
        }
        let Some(trace) = &self.trace else {
            return Ok(());
        };
        let Some(path) = &trace.path else {
            return Ok(());
        };
        if path.edges.is_empty() {
            return Ok(());
        }
        let hop = 1.1;
        let total = path.edges.len() as f64 * hop + 0.8;
        let ph = ((t - trace.at) % total + total) % total;
        let idx = (ph / hop).floor() as usize;
        if idx >= path.edges.len() {
            return Ok(());
        }
        let local = ease_in_out_cubic((ph - idx as f64 * hop) / hop);
        let (c, _) = self.edge_curve(path.edges[idx]);
        ctx.set_global_composite_operation(if pal.dark { "lighter" } else { "source-over" })?;
        for i in 0..12 {
            let fi = f64::from(i);
            let u = clamp(local - fi * 0.018, 0.0, 1.0);
            let x = cubic_at(c[0], c[2], c[4], c[6], u);
            let y = cubic_at(c[1], c[3], c[5], c[7], u);
            ctx.set_fill_style_str(&rgba(&pal.foil, Some((1.0 - fi / 12.0) * 0.9)));
            ctx.begin_path();
            ctx.arc(x, y, 3.2 - fi * 0.2, 0.0, TAU)?;
            ctx.fill();
        }
        ctx.set_global_composite_operation("source-over")?;
        Ok(())
    }

    /// What is under a stage point: a skill, the anatomy's hub, or a section arc.
    pub(crate) fn pick(&self, sx: f64, sy: f64) -> Option<Hit> {
        let model = self.model.as_ref()?;
        let anatomy = self.view == "anatomy";
        let mut best = None;
        let mut best_d = f64::INFINITY;
        for (i, n) in model.nodes.iter().enumerate() {
            if n.ta == 0.0 && n.pa < 0.3 {
                continue;
            }
            if anatomy && Some(i) == self.focus {
                continue;
            }
            let dx = n.sx - sx;
            let dy = n.sy - sy;
            let d = (dx * dx + dy * dy).sqrt();
            let reach = f64::max(n.sr + 6.0, 11.0);
            if d < reach && d < best_d {
                best = Some(i);
                best_d = d;
            }
        }
        if let Some(n) = best {
            return Some(Hit::Node(n));
        }
        if let (true, Some(l), Some(_)) = (anatomy, &self.anat, self.focus) {
            let o = self.hub_screen();
            let k = self.cam.k;
            let dx = sx - o.0;
            let dy = sy - o.1;
            let rho = (dx * dx + dy * dy).sqrt();
            if rho <= l.r0 * k {
                return Some(Hit::Hub);
            }
            if rho >= (l.r1 - 3.0) * k && rho <= (l.r2 + 14.0) * k {
                let ang = dy.atan2(dx);
                for (j, arc) in l.arcs.iter().enumerate() {
                    let rel = angle_norm(ang - arc.a0);
                    if rel <= angle_norm(arc.a1 - arc.a0) + 0.01 {
                        return Some(Hit::Section(j));
                    }
                }
            }
        }
        None
    }
}

fn glow_sprite(color: &[f64; 4], dark: bool) -> Result<HtmlCanvasElement, JsValue> {
    let c: HtmlCanvasElement = document()?.create_element("canvas")?.unchecked_into();
    c.set_width(64);
    c.set_height(64);
    let g: Ctx = c
        .get_context("2d")?
        .ok_or("no 2d context")?
        .unchecked_into();
    let grad = g.create_radial_gradient(32.0, 32.0, 0.0, 32.0, 32.0, 32.0)?;
    grad.add_color_stop(0.0, &rgba(color, Some(if dark { 0.55 } else { 0.32 })))?;
    grad.add_color_stop(0.35, &rgba(color, Some(if dark { 0.2 } else { 0.12 })))?;
    grad.add_color_stop(1.0, &rgba(color, Some(0.0)))?;
    g.set_fill_style_canvas_gradient(&grad);
    g.fill_rect(0.0, 0.0, 64.0, 64.0);
    Ok(c)
}
