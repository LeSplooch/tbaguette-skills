//! The graph's model: graph.json read into `atelier_core::graph::GraphData`,
//! then the nodes, edges, roles and computed notes the app draws from
//! (graph.js `unpackGraph`, `buildModel`, `bfs`, `reachTwo`, `shortestPath`,
//! `betweenness`, `computeNotes`).
//!
//! Orders are graph.js's on purpose, since the notes' wording and the traced
//! routes depend on them: edges in first-mention order (trigger, then each
//! section), a node's `out`/`inn` in edge order, BFS neighbours stably sorted
//! by descending weight, and `top()` ties broken by catalog index.

use std::collections::{HashMap, VecDeque};

use atelier_core::graph::{
    Citations, GraphCategory, GraphData, Section, SectionKind, SkillNode, SubSection,
};
use serde_json::Value;

use super::text::{STR_NOTES, escape_html, fmt, js_round, num};

/// graph.json's packed schema.
const PACKED_SCHEMA: u64 = 2;

/// graph.js `slugifyHeading`: lower-case, quotes dropped, every other run of
/// non-`[a-z0-9]` one hyphen, trimmed; `section` when nothing is left.
pub fn slugify_heading(title: &str) -> String {
    let lower = title.to_lowercase().replace(['\'', '’'], "");
    let mut out = String::new();
    let mut gap = false;
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else {
            gap = true;
        }
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

fn as_str(v: Option<&Value>) -> String {
    v.and_then(Value::as_str).unwrap_or("").to_string()
}

fn as_count(v: Option<&Value>) -> usize {
    v.and_then(Value::as_f64)
        .map_or(0, |n| if n > 0.0 { n as usize } else { 0 })
}

fn opt_str(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::to_string)
}

/// graph.json (either schema) as the build's model.
pub fn unpack(text: &str) -> Result<GraphData, String> {
    let data: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if data.get("schema").and_then(Value::as_u64) == Some(PACKED_SCHEMA) {
        unpack_packed(&data)
    } else {
        unpack_objects(&data)
    }
}

fn arr(v: Option<&Value>) -> &[Value] {
    v.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn unpack_packed(data: &Value) -> Result<GraphData, String> {
    let skills = arr(data.get("skills"));
    let cats = arr(data.get("categories"));
    let slugs: Vec<String> = skills.iter().map(|s| as_str(s.get(0))).collect();
    let families: Vec<String> = cats.iter().map(|c| as_str(c.get(0))).collect();
    let slug_at = |v: Option<&Value>| -> Result<String, String> {
        let i = v
            .and_then(Value::as_u64)
            .ok_or("a citation without a skill index")? as usize;
        slugs
            .get(i)
            .cloned()
            .ok_or_else(|| format!("skill index {i} out of range"))
    };
    let citations = |packed: Option<&Value>| -> Result<Citations, String> {
        let mut c = Citations::default();
        for entry in arr(packed) {
            let slug = slug_at(entry.get(0))?;
            c.refs.push((slug.clone(), as_count(entry.get(1))));
            if let Some(q) = entry.get(2).and_then(Value::as_str) {
                c.quotes.push((slug, q.to_string()));
            }
        }
        Ok(c)
    };
    // `field(fields, i, null)`: a present null reads as absent.
    let heading_id =
        |fields: &Value, i: usize, title: &str| match fields.get(i).and_then(Value::as_str) {
            Some(id) => id.to_string(),
            None => slugify_heading(title),
        };
    let categories = cats
        .iter()
        .map(|c| -> Result<GraphCategory, String> {
            Ok(GraphCategory {
                slug: as_str(c.get(0)),
                title: as_str(c.get(1)),
                skill_slugs: arr(c.get(2))
                    .iter()
                    .map(|i| slug_at(Some(i)))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut nodes = Vec::with_capacity(skills.len());
    for s in skills {
        let slug = as_str(s.get(0));
        let family = s.get(1).and_then(Value::as_u64).map(|i| i as usize);
        let sections = arr(s.get(5))
            .iter()
            .map(|sec| -> Result<Section, String> {
                let title = as_str(sec.get(0));
                let cited = citations(sec.get(3))?;
                let subs = arr(sec.get(4))
                    .iter()
                    .map(|sub| -> Result<SubSection, String> {
                        let sub_title = as_str(sub.get(0));
                        Ok(SubSection {
                            id: heading_id(sub, 3, &sub_title),
                            words: as_count(sub.get(1)),
                            refs: citations(sub.get(2))?.refs,
                            title: sub_title,
                        })
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Section {
                    id: heading_id(sec, 5, &title),
                    kind: if sec.get(1).and_then(Value::as_u64) == Some(1) {
                        SectionKind::Reference
                    } else {
                        SectionKind::Body
                    },
                    words: as_count(sec.get(2)),
                    refs: cited.refs,
                    quotes: cited.quotes,
                    subs,
                    title,
                })
            })
            .collect::<Result<_, _>>()?;
        nodes.push(SkillNode {
            name: opt_str(s.get(9)).unwrap_or_else(|| slug.clone()),
            category: family
                .and_then(|f| families.get(f))
                .cloned()
                .unwrap_or_default(),
            summary: as_str(s.get(2)),
            words: as_count(s.get(3)),
            always_on: s.get(6).and_then(Value::as_bool).unwrap_or(false),
            change_status: opt_str(s.get(7)),
            change_at: opt_str(s.get(8)),
            trigger: citations(s.get(4))?,
            sections,
            slug,
        });
    }
    Ok(GraphData {
        skill_url_template: as_str(data.get("skill_url_template")),
        categories,
        skills: nodes,
    })
}

/// Schema 1: the same graph as objects, which graph.js takes unchanged.
fn unpack_objects(data: &Value) -> Result<GraphData, String> {
    let refs = |v: Option<&Value>| -> Vec<(String, usize)> {
        v.and_then(Value::as_object).map_or_else(Vec::new, |m| {
            m.iter()
                .map(|(k, c)| (k.clone(), as_count(Some(c))))
                .collect()
        })
    };
    let quotes = |v: Option<&Value>| -> Vec<(String, String)> {
        v.and_then(Value::as_object).map_or_else(Vec::new, |m| {
            m.iter()
                .filter_map(|(k, q)| q.as_str().map(|q| (k.clone(), q.to_string())))
                .collect()
        })
    };
    let categories = arr(data.get("categories"))
        .iter()
        .map(|c| GraphCategory {
            slug: as_str(c.get("slug")),
            title: as_str(c.get("title")),
            skill_slugs: arr(c.get("skill_slugs"))
                .iter()
                .map(|s| as_str(Some(s)))
                .collect(),
        })
        .collect();
    let skills = arr(data.get("skills"))
        .iter()
        .map(|s| SkillNode {
            slug: as_str(s.get("slug")),
            name: as_str(s.get("name")),
            category: as_str(s.get("category")),
            summary: as_str(s.get("summary")),
            words: as_count(s.get("words")),
            always_on: s.get("always_on").and_then(Value::as_bool).unwrap_or(false),
            change_status: opt_str(s.get("change_status")),
            change_at: opt_str(s.get("change_at")),
            trigger: Citations {
                refs: refs(s.get("trigger").and_then(|t| t.get("refs"))),
                quotes: quotes(s.get("trigger").and_then(|t| t.get("quotes"))),
            },
            sections: arr(s.get("sections"))
                .iter()
                .map(|sec| Section {
                    id: as_str(sec.get("id")),
                    title: as_str(sec.get("title")),
                    kind: if sec.get("kind").and_then(Value::as_str) == Some("reference") {
                        SectionKind::Reference
                    } else {
                        SectionKind::Body
                    },
                    words: as_count(sec.get("words")),
                    refs: refs(sec.get("refs")),
                    quotes: quotes(sec.get("quotes")),
                    subs: arr(sec.get("subs"))
                        .iter()
                        .map(|sub| SubSection {
                            id: as_str(sub.get("id")),
                            title: as_str(sub.get("title")),
                            words: as_count(sub.get("words")),
                            refs: refs(sub.get("refs")),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();
    Ok(GraphData {
        skill_url_template: as_str(data.get("skill_url_template")),
        categories,
        skills,
    })
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Foundation,
    Signpost,
    Crossroads,
    Specialist,
}

impl Role {
    pub fn key(self) -> &'static str {
        match self {
            Role::Foundation => "foundation",
            Role::Signpost => "signpost",
            Role::Crossroads => "crossroads",
            Role::Specialist => "specialist",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cat {
    pub slug: String,
    pub title: String,
    pub index: usize,
    pub nodes: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Sub {
    pub words: f64,
    pub refs: Vec<(String, f64)>,
}

#[derive(Debug, Clone)]
pub struct Sec {
    pub id: String,
    pub title: String,
    pub reference: bool,
    pub words: f64,
    pub refs: Vec<(String, f64)>,
    pub quotes: Vec<(String, String)>,
    pub subs: Vec<Sub>,
}

/// `refs[slug]`, 0 when absent (graph.js tests the count's truthiness).
pub fn ref_count(refs: &[(String, f64)], slug: &str) -> f64 {
    refs.iter()
        .find(|(s, _)| s == slug)
        .map_or(0.0, |(_, c)| *c)
}

pub fn quote_of<'a>(quotes: &'a [(String, String)], slug: &str) -> Option<&'a str> {
    quotes
        .iter()
        .find(|(s, _)| s == slug)
        .map(|(_, q)| q.as_str())
}

/// One skill: its facts, and the positions every view animates.
#[derive(Debug, Clone)]
pub struct Node {
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub words: f64,
    pub always_on: bool,
    pub change_status: Option<String>,
    pub change_at: Option<String>,
    pub trigger_refs: Vec<(String, f64)>,
    pub trigger_quotes: Vec<(String, String)>,
    pub sections: Vec<Sec>,
    pub cat: usize,
    pub index: usize,
    /// (target, edge), in edge order.
    pub out: Vec<(usize, usize)>,
    /// (source, edge), in edge order.
    pub inn: Vec<(usize, usize)>,
    pub out_deg: usize,
    pub in_deg: usize,
    pub degree: usize,
    pub role: Role,
    // Simulation.
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub fx: Option<f64>,
    pub fy: Option<f64>,
    pub base_r: f64,
    pub sim_r: f64,
    // Displayed springs and their targets.
    pub px: f64,
    pub py: f64,
    pub pvx: f64,
    pub pvy: f64,
    pub pr: f64,
    pub prv: f64,
    pub pa: f64,
    pub tx: f64,
    pub ty: f64,
    pub tr: f64,
    pub ta: f64,
    pub phase: f64,
    // Wheel.
    pub wa: f64,
    pub wx: f64,
    pub wy: f64,
    // Screen, per frame (NaN until the first draw, as graph.js's undefined).
    pub sx: f64,
    pub sy: f64,
    pub sr: f64,
    pub label_spot: i32,
    /// `Date.parse(change_at)`, filled in by the app (NaN when absent).
    pub change_ms: f64,
}

impl Node {
    pub fn out_edge(&self, target: usize) -> Option<usize> {
        self.out.iter().find(|(t, _)| *t == target).map(|(_, e)| *e)
    }
    pub fn inn_edge(&self, source: usize) -> Option<usize> {
        self.inn.iter().find(|(s, _)| *s == source).map(|(_, e)| *e)
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub key: String,
    pub s: usize,
    pub t: usize,
    pub w: f64,
    pub sections: Vec<i32>,
    pub mutual: bool,
    pub phase: f64,
    pub len: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Trace { from: String, to: String },
    Spot(Spot),
    Family(Vec<String>),
}

/// A spotlight: a note's, a search's, a matrix cell's (`flow`), or every
/// mutual pair (`mutual`).
#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub kind: &'static str,
    pub nodes: Vec<String>,
    pub dir: Option<&'static str>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Spot {
    pub fn nodes(nodes: Vec<String>, dir: &'static str) -> Spot {
        Spot {
            kind: "spot",
            nodes,
            dir: Some(dir),
            from: None,
            to: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub html: String,
    pub action: Action,
}

pub struct Model {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub edge_map: HashMap<(usize, usize), usize>,
    pub by_slug: HashMap<String, usize>,
    pub categories: Vec<Cat>,
    pub notes: Vec<Note>,
    url_template: String,
}

const TAU: f64 = std::f64::consts::PI * 2.0;

fn median(values: &[f64]) -> f64 {
    let mut s = values.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if s.is_empty() {
        return 0.0;
    }
    let m = s.len() >> 1;
    if s.len() % 2 == 1 {
        s[m]
    } else {
        (s[m - 1] + s[m]) / 2.0
    }
}

/// `encodeURIComponent`.
pub fn encode_uri_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

impl Model {
    /// graph.js `buildModel`. `random` is `Math.random`, called twice per edge
    /// as graph.js does (the edge's particle phases).
    pub fn build(data: &GraphData, random: &mut dyn FnMut() -> f64) -> Model {
        let mut categories: Vec<Cat> = data
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| Cat {
                slug: c.slug.clone(),
                title: c.title.clone(),
                index: i,
                nodes: Vec::new(),
            })
            .collect();
        let cat_by_slug: HashMap<&str, usize> = data
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| (c.slug.as_str(), i))
            .collect();
        let counts = |refs: &[(String, usize)]| {
            refs.iter()
                .map(|(s, c)| (s.clone(), *c as f64))
                .collect::<Vec<_>>()
        };
        let mut nodes: Vec<Node> = data
            .skills
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let cat = cat_by_slug.get(s.category.as_str()).copied().unwrap_or(0);
                if let Some(c) = categories.get_mut(cat) {
                    c.nodes.push(i);
                }
                Node {
                    slug: s.slug.clone(),
                    name: s.name.clone(),
                    summary: s.summary.clone(),
                    words: s.words as f64,
                    always_on: s.always_on,
                    change_status: s.change_status.clone(),
                    change_at: s.change_at.clone(),
                    trigger_refs: counts(&s.trigger.refs),
                    trigger_quotes: s.trigger.quotes.clone(),
                    sections: s
                        .sections
                        .iter()
                        .map(|sec| Sec {
                            id: sec.id.clone(),
                            title: sec.title.clone(),
                            reference: sec.kind == SectionKind::Reference,
                            words: sec.words as f64,
                            refs: counts(&sec.refs),
                            quotes: sec.quotes.clone(),
                            subs: sec
                                .subs
                                .iter()
                                .map(|sub| Sub {
                                    words: sub.words as f64,
                                    refs: counts(&sub.refs),
                                })
                                .collect(),
                        })
                        .collect(),
                    cat,
                    index: i,
                    out: Vec::new(),
                    inn: Vec::new(),
                    out_deg: 0,
                    in_deg: 0,
                    degree: 0,
                    role: Role::Specialist,
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    fx: None,
                    fy: None,
                    base_r: 0.0,
                    sim_r: 0.0,
                    px: 0.0,
                    py: 0.0,
                    pvx: 0.0,
                    pvy: 0.0,
                    pr: 0.0,
                    prv: 0.0,
                    pa: 0.0,
                    tx: 0.0,
                    ty: 0.0,
                    tr: 6.0,
                    ta: 1.0,
                    phase: (i as f64 * 2.399963) % TAU,
                    wa: 0.0,
                    wx: 0.0,
                    wy: 0.0,
                    sx: f64::NAN,
                    sy: f64::NAN,
                    sr: f64::NAN,
                    label_spot: 0,
                    change_ms: f64::NAN,
                }
            })
            .collect();
        // JS keeps the first node per slug in bySlug order of assignment; the
        // last assignment wins, as `bySlug[n.slug] = n` does.
        let mut by_slug = HashMap::new();
        for n in &nodes {
            by_slug.insert(n.slug.clone(), n.index);
        }

        let mut edge_map: HashMap<(usize, usize), usize> = HashMap::new();
        let mut edges: Vec<Edge> = Vec::new();
        for n in &nodes {
            let sources = std::iter::once((&n.trigger_refs, -1)).chain(
                n.sections
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (&s.refs, i as i32)),
            );
            for (refs, index) in sources {
                for (slug, count) in refs {
                    let Some(&t) = by_slug.get(slug) else {
                        continue;
                    };
                    if t == n.index {
                        continue;
                    }
                    let e = *edge_map.entry((n.index, t)).or_insert_with(|| {
                        let phase = random();
                        let _phase2 = random();
                        edges.push(Edge {
                            key: format!("{}>{}", n.slug, slug),
                            s: n.index,
                            t,
                            w: 0.0,
                            sections: Vec::new(),
                            mutual: false,
                            phase,
                            len: 1.0,
                        });
                        edges.len() - 1
                    });
                    edges[e].w += count;
                    edges[e].sections.push(index);
                }
            }
        }
        for (i, edge) in edges.iter_mut().enumerate() {
            let (s, t) = (edge.s, edge.t);
            edge.mutual = edge_map.contains_key(&(t, s));
            // `out[slug] = e` keeps the first insertion's position.
            if nodes[s].out_edge(t).is_none() {
                nodes[s].out.push((t, i));
            }
            if nodes[t].inn_edge(s).is_none() {
                nodes[t].inn.push((s, i));
            }
        }
        for n in &mut nodes {
            n.out_deg = n.out.len();
            n.in_deg = n.inn.len();
            n.degree = n.out.len()
                + n.inn
                    .iter()
                    .filter(|(s, _)| !n.out.iter().any(|(t, _)| t == s))
                    .count();
        }
        let med_in = median(&nodes.iter().map(|n| n.in_deg as f64).collect::<Vec<_>>());
        let med_out = median(&nodes.iter().map(|n| n.out_deg as f64).collect::<Vec<_>>());
        let hi_in = f64::max(6.0, med_in * 2.0);
        let hi_out = f64::max(6.0, med_out * 2.0);
        for n in &mut nodes {
            let i = n.in_deg as f64 >= hi_in;
            let o = n.out_deg as f64 >= hi_out;
            let ratio = n.out_deg as f64 / f64::max(1.0, n.in_deg as f64);
            n.role = match (i, o) {
                (true, true) if ratio > 2.0 => Role::Signpost,
                (true, true) if ratio < 0.5 => Role::Foundation,
                (true, true) => Role::Crossroads,
                (true, false) => Role::Foundation,
                (false, true) => Role::Signpost,
                (false, false) => Role::Specialist,
            };
        }
        let mut model = Model {
            nodes,
            edges,
            edge_map,
            by_slug,
            categories: std::mem::take(&mut categories),
            notes: Vec::new(),
            url_template: data.skill_url_template.clone(),
        };
        model.notes = compute_notes(&model);
        model
    }

    pub fn node(&self, slug: &str) -> Option<usize> {
        self.by_slug.get(slug).copied()
    }

    /// `skillUrl`: the template's first `{slug}` filled in.
    pub fn skill_url(&self, slug: &str) -> String {
        self.url_template
            .replacen("{slug}", &encode_uri_component(slug), 1)
    }

    pub fn edge(&self, s: usize, t: usize) -> Option<usize> {
        self.edge_map.get(&(s, t)).copied()
    }
}

/// One breadth-first walk: distances, the edge each node was reached by, and
/// the discovery order (graph.js's `Object.keys(dist)`).
pub struct Walk {
    pub dist: Vec<Option<u32>>,
    pub prev: Vec<Option<(usize, usize)>>,
    pub order: Vec<usize>,
}

pub fn bfs(model: &Model, from: usize, directed: bool) -> Walk {
    let n = model.nodes.len();
    let mut w = Walk {
        dist: vec![None; n],
        prev: vec![None; n],
        order: vec![from],
    };
    w.dist[from] = Some(0);
    let mut queue = VecDeque::from([from]);
    while let Some(u) = queue.pop_front() {
        let node = &model.nodes[u];
        let mut next: Vec<usize> = node.out.iter().map(|(_, e)| *e).collect();
        if !directed {
            next.extend(node.inn.iter().map(|(_, e)| *e));
        }
        // Strongest ties first; a stable sort, as V8's.
        next.sort_by(|&a, &b| {
            model.edges[b]
                .w
                .partial_cmp(&model.edges[a].w)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for e in next {
            let edge = &model.edges[e];
            let v = if edge.s == u { edge.t } else { edge.s };
            if w.dist[v].is_none() {
                w.dist[v] = Some(w.dist[u].unwrap_or(0) + 1);
                w.prev[v] = Some((u, e));
                w.order.push(v);
                queue.push_back(v);
            }
        }
    }
    w
}

pub struct Reach {
    pub nodes: Vec<usize>,
    pub families: usize,
}

/// Who a skill reaches by following its own citations for two steps.
pub fn reach_two(model: &Model, node: usize) -> Reach {
    let r = bfs(model, node, true);
    let mut nodes = Vec::new();
    let mut fams: Vec<usize> = Vec::new();
    for &v in &r.order {
        let d = r.dist[v].unwrap_or(0);
        if !(1..=2).contains(&d) {
            continue;
        }
        nodes.push(v);
        let c = model.nodes[v].cat;
        if !fams.contains(&c) {
            fams.push(c);
        }
    }
    Reach {
        nodes,
        families: fams.len(),
    }
}

pub struct Path {
    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
}

pub fn shortest_path(model: &Model, a: usize, b: usize) -> Option<Path> {
    let r = bfs(model, a, true);
    r.dist[b]?;
    let mut path = vec![b];
    let mut hops = Vec::new();
    let mut cur = b;
    while cur != a {
        let (node, edge) = r.prev[cur]?;
        hops.insert(0, edge);
        path.insert(0, node);
        cur = node;
    }
    Some(Path {
        nodes: path,
        edges: hops,
    })
}

/// Brandes on the directed citation graph, accumulated in graph.js's order.
pub fn betweenness(model: &Model) -> Vec<f64> {
    let n = model.nodes.len();
    let mut cb = vec![0.0; n];
    for s in 0..n {
        let mut stack = Vec::new();
        let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut sigma = vec![0.0f64; n];
        let mut dist = vec![-1i64; n];
        sigma[s] = 1.0;
        dist[s] = 0;
        let mut queue = VecDeque::from([s]);
        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &(w, _) in &model.nodes[v].out {
                if dist[w] < 0 {
                    queue.push_back(w);
                    dist[w] = dist[v] + 1;
                }
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    preds[w].push(v);
                }
            }
        }
        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &preds[w] {
                delta[v] += sigma[v] / sigma[w] * (1.0 + delta[w]);
            }
            if w != s {
                cb[w] += delta[w];
            }
        }
    }
    cb
}

pub fn chip(model: &Model, n: usize) -> String {
    let node = &model.nodes[n];
    format!(
        "<span class=\"crumb-chip\" style=\"--chip:var(--graph-cat-{})\">{}</span>",
        node.cat + 1,
        escape_html(&node.name)
    )
}

/// The node maximising `f`, ties to the lowest index (graph.js `top`).
fn top(model: &Model, f: impl Fn(usize) -> f64) -> usize {
    let mut best = 0;
    for i in 1..model.nodes.len() {
        if f(i) > f(best) {
            best = i;
        }
    }
    best
}

fn compute_notes(model: &Model) -> Vec<Note> {
    let nodes = &model.nodes;
    let mut notes = Vec::new();
    if nodes.is_empty() {
        return notes;
    }
    let mut far: Option<(u32, usize, usize)> = None;
    let (mut reachable, mut pairs, mut at_far) = (0u32, 0u32, 0u32);
    for a in 0..nodes.len() {
        let r = bfs(model, a, true);
        for b in 0..nodes.len() {
            if a == b {
                continue;
            }
            pairs += 1;
            if let Some(d) = r.dist[b] {
                reachable += 1;
                if far.is_none_or(|f| d as i64 > f.0 as i64) {
                    far = Some((d, a, b));
                    at_far = 0;
                }
                if far.is_some_and(|f| f.0 == d) {
                    at_far += 1;
                }
            }
        }
    }
    if let Some((d, a, b)) = far {
        let ties = if at_far > 1 {
            " It is the first of {n} ordered pairs that far apart."
                .replace("{n}", &at_far.to_string())
        } else {
            String::new()
        };
        let html = if reachable == pairs {
            fmt(
                STR_NOTES.reach,
                &[
                    ("ties", ties),
                    ("d", d.to_string()),
                    ("a", chip(model, a)),
                    ("b", chip(model, b)),
                ],
            )
        } else {
            fmt(
                STR_NOTES.reach_partial,
                &[
                    ("ties", ties),
                    (
                        "pct",
                        num(js_round(100.0 * reachable as f64 / pairs as f64)),
                    ),
                    ("d", d.to_string()),
                    ("a", chip(model, a)),
                    ("b", chip(model, b)),
                ],
            )
        };
        notes.push(Note {
            html,
            action: Action::Trace {
                from: nodes[a].slug.clone(),
                to: nodes[b].slug.clone(),
            },
        });
    }

    let cited = top(model, |n| nodes[n].in_deg as f64);
    notes.push(Note {
        html: fmt(
            STR_NOTES.most_cited,
            &[
                ("a", chip(model, cited)),
                ("n", nodes[cited].in_deg.to_string()),
            ],
        ),
        action: Action::Spot(Spot::nodes(vec![nodes[cited].slug.clone()], "in")),
    });

    let wide = top(model, |n| nodes[n].out_deg as f64);
    let every_other = nodes[wide].out_deg == nodes.len() - 1;
    notes.push(Note {
        html: fmt(
            if every_other {
                STR_NOTES.widest_all
            } else {
                STR_NOTES.widest
            },
            &[
                ("a", chip(model, wide)),
                ("n", nodes[wide].out_deg.to_string()),
                ("total", (nodes.len() - 1).to_string()),
            ],
        ),
        action: Action::Spot(Spot::nodes(vec![nodes[wide].slug.clone()], "out")),
    });

    let cb = betweenness(model);
    let bridge = top(model, |n| cb[n]);
    notes.push(Note {
        html: fmt(STR_NOTES.bridge, &[("a", chip(model, bridge))]),
        action: Action::Spot(Spot::nodes(vec![nodes[bridge].slug.clone()], "both")),
    });

    let mut bond: Option<(f64, usize, usize)> = None;
    for e in &model.edges {
        if !e.mutual || nodes[e.s].index > nodes[e.t].index {
            continue;
        }
        let Some(back) = model.edge(e.t, e.s) else {
            continue;
        };
        let w = e.w + model.edges[back].w;
        if bond.is_none_or(|b| w > b.0) {
            bond = Some((w, e.s, e.t));
        }
    }
    if let Some((w, a, b)) = bond {
        notes.push(Note {
            html: fmt(
                STR_NOTES.bond,
                &[("a", chip(model, a)), ("b", chip(model, b)), ("n", num(w))],
            ),
            action: Action::Spot(Spot::nodes(
                vec![nodes[a].slug.clone(), nodes[b].slug.clone()],
                "pair",
            )),
        });
    }

    let mut fam: Vec<(usize, f64)> = model
        .categories
        .iter()
        .filter(|c| c.nodes.len() >= 3)
        .map(|c| {
            let (mut inside, mut total) = (0u32, 0u32);
            for &n in &c.nodes {
                for &(t, _) in &nodes[n].out {
                    total += 1;
                    if nodes[t].cat == c.index {
                        inside += 1;
                    }
                }
            }
            (
                c.index,
                if total > 0 {
                    inside as f64 / total as f64
                } else {
                    0.0
                },
            )
        })
        .collect();
    fam.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if fam.len() > 1 {
        let tight = fam[0];
        let loose = fam[fam.len() - 1];
        let fam_chip = |c: usize| {
            format!(
                "<span class=\"crumb-chip crumb-chip--family\" style=\"--chip:var(--graph-cat-{})\">{}</span>",
                c + 1,
                escape_html(&model.categories[c].title)
            )
        };
        notes.push(Note {
            html: fmt(
                STR_NOTES.family,
                &[
                    ("a", fam_chip(tight.0)),
                    ("p", num(js_round(tight.1 * 100.0))),
                    ("b", fam_chip(loose.0)),
                    ("q", num(js_round((1.0 - loose.1) * 100.0))),
                ],
            ),
            action: Action::Family(vec![
                model.categories[tight.0].slug.clone(),
                model.categories[loose.0].slug.clone(),
            ]),
        });
    }

    let mutual = model.edges.iter().filter(|e| e.mutual).count() as f64 / 2.0;
    notes.push(Note {
        html: fmt(STR_NOTES.mutual, &[("n", num(mutual))]),
        action: Action::Spot(Spot {
            kind: "mutual",
            nodes: Vec::new(),
            dir: None,
            from: None,
            to: None,
        }),
    });

    // Ascending by in + out, ties to the highest index.
    let mut quiet = 0;
    for i in 1..nodes.len() {
        let (a, b) = (
            nodes[i].in_deg + nodes[i].out_deg,
            nodes[quiet].in_deg + nodes[quiet].out_deg,
        );
        if a < b || (a == b && i > quiet) {
            quiet = i;
        }
    }
    let q = &nodes[quiet];
    let alike: Vec<usize> = (0..nodes.len())
        .filter(|&n| n != quiet && nodes[n].in_deg == q.in_deg && nodes[n].out_deg == q.out_deg)
        .collect();
    let copy = match alike.len() {
        0 => STR_NOTES.quiet,
        1 => STR_NOTES.quiet_pair,
        _ => STR_NOTES.quiet_many,
    };
    let mut spot_nodes = vec![q.slug.clone()];
    spot_nodes.extend(alike.iter().map(|&n| nodes[n].slug.clone()));
    notes.push(Note {
        html: fmt(
            copy,
            &[
                ("a", chip(model, quiet)),
                (
                    "b",
                    alike.first().map_or(String::new(), |&n| chip(model, n)),
                ),
                ("k", alike.len().to_string()),
                ("i", q.in_deg.to_string()),
                ("o", q.out_deg.to_string()),
            ],
        ),
        action: Action::Spot(Spot::nodes(spot_nodes, "both")),
    });

    // How findable the library is by citation: the skills few others point to.
    let thin: Vec<String> = nodes
        .iter()
        .filter(|n| n.in_deg <= THIN_CITED_BY)
        .map(|n| n.slug.clone())
        .collect();
    if !thin.is_empty() {
        notes.push(Note {
            html: fmt(
                STR_NOTES.thin,
                &[
                    ("n", thin.len().to_string()),
                    ("k", THIN_CITED_BY.to_string()),
                ],
            ),
            action: Action::Spot(Spot::nodes(thin, "in")),
        });
    }
    notes
}

/// A skill cited by this many others or fewer is one a reader rarely reaches
/// by following a citation.
const THIN_CITED_BY: usize = 2;

/// graph.js `editDistance` over UTF-16 code units.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<u16> = a.encode_utf16().collect();
    let b: Vec<u16> = b.encode_utf16().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i];
        for j in 1..=b.len() {
            let v = (prev[j] + 1)
                .min(cur[j - 1] + 1)
                .min(prev[j - 1] + usize::from(a[i - 1] != b[j - 1]));
            cur.push(v);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// `String.prototype.indexOf` in UTF-16 units, or -1.
pub fn index_of(hay: &str, needle: &str) -> i64 {
    match hay.find(needle) {
        Some(byte) => hay[..byte].encode_utf16().count() as i64,
        None => -1,
    }
}

/// JS `String.prototype.trim` (ECMAScript white space and line terminators).
pub fn js_trim(s: &str) -> &str {
    let ws = |c: char| {
        matches!(
            c,
            '\u{9}'
                | '\u{a}'
                | '\u{b}'
                | '\u{c}'
                | '\u{d}'
                | ' '
                | '\u{a0}'
                | '\u{1680}'
                | '\u{2000}'
                ..='\u{200a}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202f}'
                    | '\u{205f}'
                    | '\u{3000}'
                    | '\u{feff}'
        )
    };
    s.trim_matches(ws)
}

/// graph.js `findSkill`: an exact slug, else the name containing the query
/// earliest, ties to the most cited.
pub fn find_skill(model: &Model, text: &str) -> Option<usize> {
    let q = js_trim(text).to_lowercase();
    if q.is_empty() {
        return None;
    }
    if let Some(n) = model.node(&q) {
        return Some(n);
    }
    let mut hits: Vec<usize> = (0..model.nodes.len())
        .filter(|&n| index_of(&model.nodes[n].name, &q) >= 0)
        .collect();
    hits.sort_by(|&a, &b| {
        let (na, nb) = (&model.nodes[a], &model.nodes[b]);
        index_of(&na.name, &q)
            .cmp(&index_of(&nb.name, &q))
            .then(nb.in_deg.cmp(&na.in_deg))
    });
    hits.first().copied()
}

/// graph.js `askLibrary`'s ranking: name 4, summary 2, a section title 1;
/// best first, ties to the most cited.
pub fn ask_hits(model: &Model, text: &str) -> Vec<usize> {
    let q = js_trim(text).to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(usize, u32)> = model
        .nodes
        .iter()
        .map(|n| {
            let score = if index_of(&n.name, &q) >= 0 { 4 } else { 0 }
                + if index_of(&n.summary.to_lowercase(), &q) >= 0 {
                    2
                } else {
                    0
                }
                + u32::from(
                    n.sections
                        .iter()
                        .any(|x| index_of(&x.title.to_lowercase(), &q) >= 0),
                );
            (n.index, score)
        })
        .filter(|h| h.1 > 0)
        .collect();
    hits.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(model.nodes[b.0].in_deg.cmp(&model.nodes[a.0].in_deg))
    });
    hits.into_iter().map(|h| h.0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_json() -> String {
        std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/graph/graph.json"
        ))
        .expect("docs/graph/graph.json")
    }

    /// What the page beside it says it draws: "How N skills lean on each other: M
    /// cross-references". Both files come from one build, so these move with the skills.
    fn page_counts() -> (usize, usize) {
        let page = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/graph/index.html"
        ))
        .expect("docs/graph/index.html");
        let lede = page.split_once("How ").expect("the page's lede").1;
        let (skills, rest) = lede
            .split_once(" skills lean on each other: ")
            .expect("the lede's skill count");
        let (pairs, _) = rest
            .split_once(" cross-references")
            .expect("the lede's pair count");
        (skills.parse().unwrap(), pairs.parse().unwrap())
    }

    #[test]
    fn unpacking_the_shipped_graph_round_trips_through_the_build_writer() {
        let text = graph_json();
        let data = unpack(&text).expect("unpacks");
        assert_eq!(atelier_core::graph::to_json(&data), text);
    }

    #[test]
    fn the_model_matches_the_page_s_own_counts() {
        let data = unpack(&graph_json()).unwrap();
        let mut seed = 0.0;
        let model = Model::build(&data, &mut || {
            seed += 0.001;
            seed
        });
        let (skills, pairs) = page_counts();
        assert_eq!(model.nodes.len(), skills);
        assert_eq!(model.edges.len(), pairs);
        // Five of the notes are always there; the others depend on the graph's shape.
        assert!(model.notes.len() >= 5, "{} notes", model.notes.len());
        let thin = model.notes.last().expect("the notes end on the thinly cited");
        assert!(thin.html.contains("cited by 2 others or fewer"), "{}", thin.html);
        assert!(
            model.notes[0]
                .html
                .contains("Every skill reaches every other")
                || model.notes[0]
                    .html
                    .contains("pairs of skills are connected")
        );
    }

    #[test]
    fn mulberry_free_helpers_follow_javascript() {
        assert_eq!(slugify_heading("What’s next? (2)"), "whats-next-2");
        assert_eq!(slugify_heading("—"), "section");
        assert_eq!(edit_distance("clairvoyanc", "clairvoyance"), 1);
        assert_eq!(index_of("aé-b", "b"), 3);
        assert_eq!(encode_uri_component("a b/é"), "a%20b%2F%C3%A9");
        assert_eq!(js_trim("\u{feff} x \n"), "x");
    }
}
