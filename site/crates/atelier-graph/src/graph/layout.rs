//! Where the skills go: the seeded force simulation behind the constellation,
//! the wheel's circle, and a skill's anatomy (graph.js `Simulation`,
//! `layoutWheel`, `layoutAnatomy`), with the small maths they share.
//!
//! The simulation reproduces graph.js bit for bit: mulberry32 in wrapping
//! i32/u32 arithmetic (`Math.imul`), the same float operations in the same
//! order, and ticks counted per frame by the caller, never per second.

use super::model::{Cat, Model, Node, ref_count};

pub const TAU: f64 = std::f64::consts::PI * 2.0;
pub const PI: f64 = std::f64::consts::PI;

pub fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
pub fn ease_out_cubic(t: f64) -> f64 {
    let t = clamp(t, 0.0, 1.0);
    1.0 - (1.0 - t).powf(3.0)
}
pub fn ease_in_out_cubic(t: f64) -> f64 {
    let t = clamp(t, 0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(3.0) / 2.0
    }
}
/// `a % TAU` folded into [0, TAU): JavaScript's `%` is Rust's.
pub fn angle_norm(a: f64) -> f64 {
    let a = a % TAU;
    if a < 0.0 { a + TAU } else { a }
}
pub fn angle_diff(a: f64, b: f64) -> f64 {
    let d = angle_norm(b - a);
    if d > PI { d - TAU } else { d }
}
pub fn cubic_at(p0: f64, c1: f64, c2: f64, p3: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * c1 + 3.0 * u * t * t * c2 + t * t * t * p3
}

/// V8's `Math.hypot` for two values: scaled by the larger, Kahan-summed.
/// Rust's `f64::hypot` rounds differently in the last place.
pub fn js_hypot(a: f64, b: f64) -> f64 {
    let (a, b) = (a.abs(), b.abs());
    if a.is_infinite() || b.is_infinite() {
        return f64::INFINITY;
    }
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    let max = if b > a { b } else { a };
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut comp) = (0.0f64, 0.0f64);
    for v in [a, b] {
        let n = v / max;
        let summand = n * n - comp;
        let pre = sum + summand;
        comp = (pre - sum) - summand;
        sum = pre;
    }
    sum.sqrt() * max
}

/// graph.js `mulberry32`.
pub struct Mulberry32(i32);

impl Mulberry32 {
    pub fn new(seed: i32) -> Self {
        Mulberry32(seed)
    }
    pub fn next(&mut self) -> f64 {
        let seed = self.0.wrapping_add(0x6D2B_79F5);
        self.0 = seed;
        let mut t = (seed ^ ((seed as u32) >> 15) as i32).wrapping_mul(1 | seed);
        t = t.wrapping_add((t ^ ((t as u32) >> 7) as i32).wrapping_mul(61 | t)) ^ t;
        f64::from((t ^ ((t as u32) >> 14) as i32) as u32) / 4_294_967_296.0
    }
}

pub struct Simulation {
    pub alpha: f64,
    pub alpha_target: f64,
    anchors: Vec<(f64, f64)>,
}

impl Simulation {
    /// Families on an ellipse in catalog order, shaped by the stage's aspect;
    /// every skill jittered around its family's anchor from one fixed seed.
    pub fn new(nodes: &mut [Node], cats: &[Cat], aspect: f64) -> Simulation {
        let n = cats.len() as f64;
        let aspect = if aspect == 0.0 || aspect.is_nan() {
            1.5
        } else {
            aspect
        };
        let stretch = clamp(aspect, 0.55, 2.2).sqrt();
        let (rx, ry) = (340.0 * stretch, 340.0 / stretch);
        let anchors: Vec<(f64, f64)> = cats
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let a = -PI / 2.0 + i as f64 * TAU / n;
                let far = if c.nodes.len() < 2 { 1.22 } else { 1.0 };
                (a.cos() * rx * far, a.sin() * ry * far)
            })
            .collect();
        let mut rand = Mulberry32::new(20_260_924);
        for node in nodes.iter_mut() {
            let anchor = anchors[node.cat];
            node.x = anchor.0 + (rand.next() - 0.5) * 120.0;
            node.y = anchor.1 + (rand.next() - 0.5) * 120.0;
            node.vx = 0.0;
            node.vy = 0.0;
        }
        Simulation {
            alpha: 1.0,
            alpha_target: 0.0,
            anchors,
        }
    }

    fn radius(node: &Node) -> f64 {
        if node.sim_r == 0.0 || node.sim_r.is_nan() {
            8.0
        } else {
            node.sim_r
        }
    }

    pub fn tick(&mut self, model: &mut Model) {
        let alpha = self.alpha;
        let nodes = &mut model.nodes;
        let len = nodes.len();
        for i in 0..len {
            for j in i + 1..len {
                let (left, right) = nodes.split_at_mut(j);
                let (a, b) = (&mut left[i], &mut right[0]);
                let mut dx = b.x - a.x;
                let mut dy = b.y - a.y;
                let mut d2 = dx * dx + dy * dy;
                if d2 < 0.01 {
                    dx = (i as f64 - j as f64) * 0.01;
                    dy = 0.01;
                    d2 = 0.0002;
                }
                if d2 > 250_000.0 {
                    continue;
                }
                let f = -520.0 * alpha / d2;
                a.vx += dx * f;
                a.vy += dy * f;
                b.vx -= dx * f;
                b.vy -= dy * f;
                let min = Self::radius(a) + Self::radius(b) + 6.0;
                if d2 < min * min {
                    let d = d2.sqrt();
                    let push = (min - d) / d * 0.5;
                    a.x -= dx * push;
                    a.y -= dy * push;
                    b.x += dx * push;
                    b.y += dy * push;
                }
            }
        }
        for e in &model.edges {
            let (sa, sb) = (e.s, e.t);
            let (ax, ay, adeg, acat) = (
                nodes[sa].x,
                nodes[sa].y,
                nodes[sa].degree as f64,
                nodes[sa].cat,
            );
            let (bx, by, bdeg, bcat) = (
                nodes[sb].x,
                nodes[sb].y,
                nodes[sb].degree as f64,
                nodes[sb].cat,
            );
            let dx = bx - ax;
            let dy = by - ay;
            let mut d = (dx * dx + dy * dy).sqrt();
            if d == 0.0 || d.is_nan() {
                d = 0.01;
            }
            let same = acat == bcat;
            let rest = if same { 48.0 } else { 200.0 };
            let k = (if same { 0.55 } else { 0.028 }) / f64::min(adeg, bdeg)
                * f64::min(2.0, 1.0 + e.w.ln() * 0.3);
            let f = (d - rest) / d * alpha * k;
            let bias = adeg / (adeg + bdeg);
            nodes[sa].vx += dx * f * (1.0 - bias);
            nodes[sa].vy += dy * f * (1.0 - bias);
            nodes[sb].vx -= dx * f * bias;
            nodes[sb].vy -= dy * f * bias;
        }
        for a in nodes.iter_mut() {
            let anchor = self.anchors[a.cat];
            let g = 0.2 / (1.0 + a.degree as f64 / 12.0);
            a.vx += (anchor.0 - a.x) * g * alpha;
            a.vy += (anchor.1 - a.y) * g * alpha;
            let centre = 0.004 + 0.06 * clamp((a.degree as f64 - 8.0) / 40.0, 0.0, 1.0);
            a.vx -= a.x * centre * alpha;
            a.vy -= a.y * centre * alpha;
        }
        for a in nodes.iter_mut() {
            if let (Some(fx), Some(fy)) = (a.fx, a.fy) {
                a.x = fx;
                a.y = fy;
                a.vx = 0.0;
                a.vy = 0.0;
                continue;
            }
            a.vx *= 0.58;
            a.vy *= 0.58;
            a.x += a.vx;
            a.y += a.vy;
        }
        self.alpha += (self.alpha_target - self.alpha) * 0.0228;
    }

    pub fn active(&self) -> bool {
        self.alpha > 0.004 || self.alpha_target > 0.0
    }
    pub fn reheat(&mut self, a: f64) {
        self.alpha = f64::max(self.alpha, a);
    }
}

pub struct WheelCat {
    pub cat: usize,
    pub a0: f64,
    pub a1: f64,
}

pub struct Wheel {
    pub r: f64,
    pub labels_all: bool,
    pub cats: Vec<WheelCat>,
}

/// Every skill on one circle, families in catalog order with a gap between.
pub fn layout_wheel(model: &mut Model, w: f64, h: f64) -> Wheel {
    let half = f64::min(w, h) / 2.0;
    let labels_all = half - 175.0 >= 190.0;
    let r = if labels_all {
        half - 175.0
    } else {
        f64::max(110.0, half - 46.0)
    };
    let gap = 1.6;
    let slots = model.nodes.len() as f64 + gap * model.categories.len() as f64;
    let step = TAU / slots;
    let mut a = -PI / 2.0 + step * gap / 2.0;
    let mut cats = Vec::new();
    for c in &model.categories {
        let start = a;
        for &n in &c.nodes {
            let node = &mut model.nodes[n];
            node.wa = a + step / 2.0;
            node.wx = node.wa.cos() * r;
            node.wy = node.wa.sin() * r;
            a += step;
        }
        cats.push(WheelCat {
            cat: c.index,
            a0: start,
            a1: a,
        });
        a += step * gap;
    }
    Wheel {
        r,
        labels_all,
        cats,
    }
}

pub struct ArcSub {
    pub sub: usize,
    pub a0: f64,
    pub a1: f64,
}

pub struct Arc {
    pub index: usize,
    pub a0: f64,
    pub a1: f64,
    pub mid: f64,
    pub subs: Vec<ArcSub>,
}

pub struct Neighbor {
    pub node: usize,
    /// The focus's edge to it, if the focus cites it.
    pub out: Option<usize>,
    /// Its edge to the focus, if it cites the focus.
    pub inn: Option<usize>,
    pub weight: f64,
    pub pref: Option<f64>,
    pub angle: f64,
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Out,
    Trigger,
    In,
}

pub struct Link {
    pub kind: LinkKind,
    pub arc: Option<usize>,
    /// Index into `Anatomy::neighbors`.
    pub item: usize,
    pub count: f64,
    pub phase: f64,
    pub geo: Option<[f64; 6]>,
    pub lit: bool,
}

pub struct Band {
    pub cat: usize,
    pub a0: f64,
    pub a1: f64,
}

pub struct Anatomy {
    pub r0: f64,
    pub r1: f64,
    pub r2: f64,
    pub big_r: f64,
    pub arcs: Vec<Arc>,
    pub crowded: bool,
    pub bands: Vec<Band>,
    pub neighbors: Vec<Neighbor>,
    pub links: Vec<Link>,
}

impl Anatomy {
    pub fn item_of(&self, node: usize) -> Option<usize> {
        self.neighbors.iter().position(|i| i.node == node)
    }
}

fn by_f64(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// One skill opened up: its sections as a ring of arcs sized by words, its
/// neighbours on an orbit beside the sections that cite them.
pub fn layout_anatomy(
    model: &Model,
    focus: usize,
    w: f64,
    h: f64,
    random: &mut dyn FnMut() -> f64,
) -> Anatomy {
    let f = &model.nodes[focus];
    let s = f64::min(w, h);
    let r0 = clamp(s * 0.12, 42.0, 84.0);
    let r1 = r0 + 9.0;
    let r2 = r1 + clamp(s * 0.04, 13.0, 24.0);
    let big_r = clamp(s * 0.37, r2 + 70.0, 340.0);
    let sections = &f.sections;
    let mut weights: Vec<f64> = sections.iter().map(|s| f64::max(s.words, 1.0)).collect();
    let mut total = 0.0;
    for w in &weights {
        total += w;
    }
    let min_w = total * 0.014;
    weights = weights.into_iter().map(|w| f64::max(w, min_w)).collect();
    total = 0.0;
    for w in &weights {
        total += w;
    }
    let n_sec = sections.len() as f64;
    let pad = if sections.len() > 1 {
        f64::min(0.03, 0.5 / n_sec)
    } else {
        0.0
    };
    let mut kind_gap = 0.0;
    for i in 1..sections.len() {
        if sections[i].reference != sections[i - 1].reference {
            kind_gap += 0.12;
        }
    }
    let avail = TAU - pad * n_sec - kind_gap;
    let mut a = -PI / 2.0;
    let mut arcs = Vec::with_capacity(sections.len());
    for (i, sec) in sections.iter().enumerate() {
        if i > 0 && sec.reference != sections[i - 1].reference {
            a += 0.12;
        }
        let span = avail * weights[i] / total;
        let a0 = a + pad / 2.0;
        let a1 = a + pad / 2.0 + span;
        let mut arc = Arc {
            index: i,
            a0,
            a1,
            mid: (a0 + a1) / 2.0,
            subs: Vec::new(),
        };
        let mut sub_total = 0.0;
        for sub in &sec.subs {
            sub_total += f64::max(sub.words, 1.0);
        }
        let rest = sec.words - sub_total;
        let lead = if sub_total != 0.0 {
            (arc.a1 - arc.a0) * f64::max(rest, 0.0) / f64::max(sec.words, 1.0)
        } else {
            0.0
        };
        let mut sa = arc.a0 + lead;
        for (si, sub) in sec.subs.iter().enumerate() {
            let sspan =
                (arc.a1 - arc.a0 - lead) * f64::max(sub.words, 1.0) / f64::max(sub_total, 1.0);
            arc.subs.push(ArcSub {
                sub: si,
                a0: sa,
                a1: sa + sspan,
            });
            sa += sspan;
        }
        a += span + pad;
        arcs.push(arc);
    }

    // Neighbours: what it cites (in its out order), then what cites it.
    let mut list: Vec<Neighbor> = Vec::new();
    let blank = |node: usize| Neighbor {
        node,
        out: None,
        inn: None,
        weight: 0.0,
        pref: None,
        angle: 0.0,
        x: 0.0,
        y: 0.0,
        r: 0.0,
    };
    for &(t, e) in &f.out {
        let mut item = blank(t);
        item.out = Some(e);
        list.push(item);
    }
    for &(src, e) in &f.inn {
        match list.iter_mut().find(|i| i.node == src) {
            Some(item) => item.inn = Some(e),
            None => {
                let mut item = blank(src);
                item.inn = Some(e);
                list.push(item);
            }
        }
    }
    for item in &mut list {
        item.weight = item.out.map_or(0.0, |e| model.edges[e].w)
            + item.inn.map_or(0.0, |e| model.edges[e].w * 0.6);
        if item.out.is_none() {
            item.pref = None;
            continue;
        }
        let (mut sx, mut sy) = (0.0, 0.0);
        let slug = &model.nodes[item.node].slug;
        for arc in &arcs {
            let c = ref_count(&sections[arc.index].refs, slug);
            if c != 0.0 {
                sx += arc.mid.cos() * c;
                sy += arc.mid.sin() * c;
            }
        }
        item.pref = Some(if sx != 0.0 || sy != 0.0 {
            sy.atan2(sx)
        } else {
            -PI / 2.0
        });
    }
    // Indices into `list`, standing in for graph.js's arrays of the same items.
    let mut placed: Vec<usize> = (0..list.len())
        .filter(|&i| list[i].pref.is_some())
        .collect();
    placed.sort_by(|&p, &q| {
        by_f64(
            angle_norm(list[p].pref.unwrap_or(0.0)),
            angle_norm(list[q].pref.unwrap_or(0.0)),
        )
    });
    for &p in &placed {
        list[p].angle = list[p].pref.unwrap_or(0.0);
    }
    let mut loose: Vec<usize> = (0..list.len())
        .filter(|&i| list[i].pref.is_none())
        .collect();
    loose.sort_by(|&p, &q| {
        let (np, nq) = (&model.nodes[list[p].node], &model.nodes[list[q].node]);
        np.cat.cmp(&nq.cat).then(np.index.cmp(&nq.index))
    });
    for item in loose {
        if placed.is_empty() {
            list[item].angle = -PI / 2.0;
            placed.push(item);
            continue;
        }
        let mut sorted: Vec<f64> = placed.iter().map(|&p| angle_norm(list[p].angle)).collect();
        sorted.sort_by(|a, b| by_f64(*a, *b));
        let (mut best, mut best_gap) = (0, -1.0);
        for i in 0..sorted.len() {
            let g = if i + 1 < sorted.len() {
                sorted[i + 1] - sorted[i]
            } else {
                sorted[0] + TAU - sorted[i]
            };
            if g > best_gap {
                best_gap = g;
                best = i;
            }
        }
        list[item].angle = sorted[best] + best_gap / 2.0;
        placed.push(item);
    }
    let crowded = list.len() > 34;
    let mut bands: Vec<Band> = Vec::new();
    if crowded {
        list.sort_by(|p, q| {
            let (cp, cq) = (model.nodes[p.node].cat, model.nodes[q.node].cat);
            cp.cmp(&cq).then(by_f64(q.weight, p.weight))
        });
        let mut fams = 0;
        let mut last: Option<usize> = None;
        for item in &list {
            let c = model.nodes[item.node].cat;
            if last != Some(c) {
                fams += 1;
                last = Some(c);
            }
        }
        let fam_gap = 0.9;
        let step = TAU / (list.len() as f64 + fam_gap * f64::from(fams));
        let mut at = -PI / 2.0 + step * fam_gap / 2.0;
        for item in &mut list {
            let c = model.nodes[item.node].cat;
            match bands.last() {
                Some(b) if b.cat == c => {}
                prev => {
                    if prev.is_some() {
                        at += step * fam_gap;
                    }
                    bands.push(Band {
                        cat: c,
                        a0: at,
                        a1: at,
                    });
                }
            }
            item.angle = at + step / 2.0;
            at += step;
            if let Some(b) = bands.last_mut() {
                b.a1 = at;
            }
        }
    }
    let min_sep = f64::min(TAU / f64::max(list.len() as f64, 1.0) * 0.92, 26.0 / big_r);
    let iters = if crowded { 0 } else { 200 };
    for _ in 0..iters {
        list.sort_by(|p, q| by_f64(angle_norm(p.angle), angle_norm(q.angle)));
        let mut moved = false;
        let n = list.len();
        for j in 0..n {
            let k = (j + 1) % n;
            if j == k {
                break;
            }
            let d = angle_norm(list[k].angle - list[j].angle);
            if d < min_sep {
                let push = (min_sep - d) / 2.0;
                list[j].angle -= push;
                list[k].angle += push;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    for item in &mut list {
        item.angle = angle_norm(item.angle);
        item.x = item.angle.cos() * big_r;
        item.y = item.angle.sin() * big_r;
        item.r = if crowded {
            clamp(3.0 + item.weight.sqrt() * 1.2, 3.5, 8.0)
        } else {
            clamp(4.0 + item.weight.sqrt() * 1.7, 4.5, 11.0)
        };
    }
    list.sort_by(|p, q| by_f64(p.angle, q.angle));
    let at_node = |node: &str| list.iter().position(|i| model.nodes[i.node].slug == node);
    let mut links = Vec::new();
    for arc in &arcs {
        for (slug, count) in &sections[arc.index].refs {
            if let Some(item) = at_node(slug) {
                links.push(Link {
                    kind: LinkKind::Out,
                    arc: Some(arc.index),
                    item,
                    count: *count,
                    phase: random(),
                    geo: None,
                    lit: false,
                });
            }
        }
    }
    for (slug, count) in &f.trigger_refs {
        if let Some(item) = at_node(slug) {
            links.push(Link {
                kind: LinkKind::Trigger,
                arc: None,
                item,
                count: *count,
                phase: random(),
                geo: None,
                lit: false,
            });
        }
    }
    for (i, item) in list.iter().enumerate() {
        if let Some(e) = item.inn {
            links.push(Link {
                kind: LinkKind::In,
                arc: None,
                item: i,
                count: model.edges[e].w,
                phase: random(),
                geo: None,
                lit: false,
            });
        }
    }
    Anatomy {
        r0,
        r1,
        r2,
        big_r,
        arcs,
        crowded,
        bands,
        neighbors: list,
        links,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulberry32_matches_javascript() {
        // First values of graph.js mulberry32(20260924) and (7), from Node.
        let mut r = Mulberry32::new(20_260_924);
        let got: Vec<f64> = (0..3).map(|_| r.next()).collect();
        assert_eq!(
            got,
            [0.5380521984770894, 0.6727424410637468, 0.8195708128623664]
        );
        let mut d = Mulberry32::new(7);
        assert_eq!(d.next(), 0.011704753153026104);
    }

    #[test]
    fn hypot_matches_v8() {
        assert_eq!(js_hypot(3.0, 4.0), 5.0);
        assert_eq!(js_hypot(0.0, 0.0), 0.0);
        assert_eq!(js_hypot(-1e300, 1e300), 1.4142135623730952e300);
        assert_eq!(js_hypot(0.1, 0.2), 0.223606797749979);
        assert_eq!(js_hypot(3.3, 1e-3), 3.3000001515151483);
    }

    #[test]
    fn angles_fold_like_javascript_remainders() {
        assert!((angle_norm(-0.5) - (TAU - 0.5)).abs() < 1e-15);
        assert!((angle_diff(0.1, TAU - 0.1) + 0.2).abs() < 1e-12);
    }
}
