//! The skill graph: which skill cites which, and from which section
//! (`scripts/skill_graph.py`, ported).
//!
//! Built from the *rendered* content rather than from the Markdown, on
//! purpose: the pipeline already decides what counts as a cross-reference
//! (a code span or a bare slug-shaped token naming a skill that exists, never
//! the page's own slug) and turns exactly those into `<a class="skill-link">`.
//! Reading the links back out of the HTML means the graph and the pages can
//! never disagree about what cites what.
//!
//! Per skill the graph holds its trigger (the frontmatter description) and
//! its sections: the SKILL.md's own h2 sections (`kind` body) and one section
//! per reference file rendered on the same page (`kind` reference), each with
//! a word count, its citations in first-mention order, and the sentence
//! around the first mention of each cited skill. A section's refs include its
//! sub-sections' refs; a sub-section keeps its own so the graph can drill in.
//! Edges are not stored: they are the sum of the refs.
//!
//! [`to_json`] writes the packed schema-2 form `docs/graph/graph.json` ships
//! in, byte for byte what the Python build writes; [`banner_summary`] is the
//! handful of numbers the landing page's banner carries inline.
//!
//! Python semantics this port keeps on purpose: lengths and offsets count
//! code points; "whitespace" is `str.isspace` (Unicode `White_Space` plus
//! U+001C..U+001F); a word character is `str.isalnum` (Unicode categories L
//! and N, verified identical over all code points on Python 3.14); and words
//! are counted per text node, so a word never joins across a tag.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::content::{self, References, RenderedSkill};
use crate::markdown::{ReferenceSection, slugify};
use crate::model::{Category, Skill};

/// The `schema` value of the packed form.
pub const PACKED_SCHEMA: u32 = 2;

/// Long enough for the sentence that carries the reason, short enough that a
/// tooltip holding one never needs to scroll.
pub const QUOTE_MAX_LENGTH: usize = 240;

/// A clause this short before a colon is a label that the quote keeps.
const LABEL_MAX_WORDS: usize = 3;

/// One skill's most recent change, as the landing page's badges see it:
/// `status` is `"new"` or `"updated"`, `at` a strict ISO timestamp. The build
/// reads it from git; a skill absent from the map has neither.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangeStamp {
    pub status: Option<String>,
    pub at: Option<String>,
}

/// Citation counts by cited slug, in first-mention order. The order is the
/// serialization order of graph.json, so it is a list rather than a map.
pub type Refs = Vec<(String, usize)>;

/// Quotes by cited slug, in first-mention order.
pub type Quotes = Vec<(String, String)>;

/// The whole graph, unpacked (`skill_graph.build_graph`'s shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphData {
    /// A page URL with a literal `{slug}` the browser fills in.
    pub skill_url_template: String,
    pub categories: Vec<GraphCategory>,
    /// In category order, then each category's own skill order.
    pub skills: Vec<SkillNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphCategory {
    pub slug: String,
    pub title: String,
    pub skill_slugs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillNode {
    pub slug: String,
    pub name: String,
    /// The category's slug.
    pub category: String,
    /// The card teaser (`summarize_description`).
    pub summary: String,
    /// The sum of the sections' words; the trigger is not counted.
    pub words: usize,
    /// Whether the description opens "Use at the start of every conversation".
    pub always_on: bool,
    pub change_status: Option<String>,
    pub change_at: Option<String>,
    pub trigger: Citations,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Citations {
    pub refs: Refs,
    pub quotes: Quotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// One of the SKILL.md's own h2 sections.
    Body,
    /// One reference file rendered on the skill's page.
    Reference,
    /// The description; only ever an intermediate, never in `sections`.
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub kind: SectionKind,
    pub words: usize,
    pub refs: Refs,
    pub quotes: Quotes,
    pub subs: Vec<SubSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubSection {
    pub id: String,
    pub title: String,
    pub words: usize,
    pub refs: Refs,
}

/// One family on the banner: its catalog index, title and skill count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerFamily {
    pub index: usize,
    pub title: String,
    pub count: usize,
}

/// What the landing page's announcement banner draws, small enough to ride
/// in the page itself (`skill_graph.banner_summary`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerSummary {
    pub skill_count: usize,
    /// Directed (citing, cited) pairs with any citation.
    pub pair_count: usize,
    /// Pairs citing each other both ways, counted once.
    pub mutual_count: usize,
    /// The longest of the shortest walks along citations.
    pub max_steps: usize,
    /// Whether every skill can reach every other.
    pub reachable: bool,
    pub families: Vec<BannerFamily>,
    /// `[a, b, citations]` for every pair of families with any citation
    /// between them, either way, `a < b`, sorted.
    pub links: Vec<[usize; 3]>,
}

/// The whole graph for the skills on disk.
///
/// `fresh` supplies each skill's change stamp; `skill_url_template` is the
/// skill page URL with a literal `{slug}` (the base path is the caller's
/// business, so it arrives pre-built, as in the Python build).
///
/// # Panics
///
/// If a category lists a slug `skills` does not have. The build's catalog
/// gate refuses that drift before the graph is built.
pub fn build(
    skills: &BTreeMap<String, Skill>,
    categories: &[Category],
    fresh: &BTreeMap<String, ChangeStamp>,
    skill_url_template: &str,
) -> GraphData {
    let known: BTreeSet<String> = skills.keys().cloned().collect();
    // Last listing wins, as in content_pipeline's dict comprehension.
    let category_of: HashMap<&str, &str> = categories
        .iter()
        .flat_map(|c| c.skill_slugs.iter().map(move |s| (*s, c.slug)))
        .collect();
    let graph_categories: Vec<GraphCategory> = categories
        .iter()
        .map(|c| GraphCategory {
            slug: c.slug.to_string(),
            title: c.title.to_string(),
            skill_slugs: c.skill_slugs.iter().map(|s| (*s).to_string()).collect(),
        })
        .collect();
    let nodes = categories
        .iter()
        .flat_map(|c| c.skill_slugs.iter())
        .map(|slug| {
            let skill = skills
                .get(*slug)
                .unwrap_or_else(|| panic!("a category lists skill {slug:?}, which does not exist"));
            // The base path only prefixes the skill-link hrefs, and the graph
            // reads their `/skills/<slug>/` tail.
            let rendered = content::render_skill(skill, &known, "");
            let stamp = fresh.get(*slug).cloned().unwrap_or_default();
            build_skill_node(skill, &rendered, category_of[*slug], stamp)
        })
        .collect();
    GraphData {
        skill_url_template: skill_url_template.to_string(),
        categories: graph_categories,
        skills: nodes,
    }
}

/// graph.json: the graph packed to schema 2, as compact JSON with non-ASCII
/// written raw (Python's `json.dumps(ensure_ascii=False,
/// separators=(",", ":"))`).
///
/// The packing, unchanged from `skill_graph.pack_graph`: a skill, a section
/// and a sub-section are arrays in a fixed field order with trailing default
/// fields left off; a citation is `[skill index, count]` or `[skill index,
/// count, quote]`; a heading id is written only when it is not
/// `slugify(title)`; a family is its index; a skill's name only when it is
/// not its slug.
///
/// # Panics
///
/// If a section quotes a skill it does not cite (schema 2 carries a quote on
/// its citation), or cites a skill that is not in the graph.
pub fn to_json(graph: &GraphData) -> String {
    let index: HashMap<&str, usize> = graph
        .skills
        .iter()
        .enumerate()
        .map(|(i, s)| (s.slug.as_str(), i))
        .collect();
    let families: Vec<&str> = graph.categories.iter().map(|c| c.slug.as_str()).collect();
    let citations = |refs: &Refs, quotes: &Quotes, place: &str| -> Json {
        let mut stray: Vec<&str> = quotes
            .iter()
            .map(|(slug, _)| slug.as_str())
            .filter(|slug| !refs.iter().any(|(r, _)| r == slug))
            .collect();
        if !stray.is_empty() {
            stray.sort_unstable();
            panic!(
                "{place} quotes {stray:?} without citing them; schema 2 carries a quote on its citation"
            );
        }
        Json::Arr(
            refs.iter()
                .map(|(slug, count)| {
                    let at = *index.get(slug.as_str()).unwrap_or_else(|| {
                        panic!("{place} cites {slug:?}, which is not in the graph")
                    });
                    let mut entry = vec![Json::Int(at), Json::Int(*count)];
                    if let Some((_, quote)) = quotes.iter().find(|(q, _)| q == slug) {
                        entry.push(Json::Str(quote.clone()));
                    }
                    Json::Arr(entry)
                })
                .collect(),
        )
    };
    let heading_id = |id: &str, title: &str| {
        if id == slugify(title) {
            Json::Null
        } else {
            Json::Str(id.to_string())
        }
    };

    let skills = graph
        .skills
        .iter()
        .map(|skill| {
            let place = skill.slug.as_str();
            let sections = skill
                .sections
                .iter()
                .map(|section| {
                    let subs = section
                        .subs
                        .iter()
                        .map(|sub| {
                            Json::Arr(trim(
                                vec![
                                    Json::Str(sub.title.clone()),
                                    Json::Int(sub.words),
                                    citations(&sub.refs, &Vec::new(), place),
                                    heading_id(&sub.id, &sub.title),
                                ],
                                &[Json::Arr(Vec::new()), Json::Null],
                            ))
                        })
                        .collect();
                    let kind = match section.kind {
                        SectionKind::Body => 0,
                        SectionKind::Reference => 1,
                        SectionKind::Trigger => {
                            panic!("{place}: a trigger section cannot be packed")
                        }
                    };
                    Json::Arr(trim(
                        vec![
                            Json::Str(section.title.clone()),
                            Json::Int(kind),
                            Json::Int(section.words),
                            citations(&section.refs, &section.quotes, place),
                            Json::Arr(subs),
                            heading_id(&section.id, &section.title),
                        ],
                        &[Json::Arr(Vec::new()), Json::Arr(Vec::new()), Json::Null],
                    ))
                })
                .collect();
            let family = families
                .iter()
                .position(|f| *f == skill.category)
                .unwrap_or_else(|| {
                    panic!("{place}: category {:?} is not in the graph", skill.category)
                });
            let optional = |value: &Option<String>| value.clone().map_or(Json::Null, Json::Str);
            Json::Arr(trim(
                vec![
                    Json::Str(skill.slug.clone()),
                    Json::Int(family),
                    Json::Str(skill.summary.clone()),
                    Json::Int(skill.words),
                    citations(&skill.trigger.refs, &skill.trigger.quotes, place),
                    Json::Arr(sections),
                    Json::Bool(skill.always_on),
                    optional(&skill.change_status),
                    optional(&skill.change_at),
                    if skill.name == skill.slug {
                        Json::Null
                    } else {
                        Json::Str(skill.name.clone())
                    },
                ],
                &[Json::Bool(false), Json::Null, Json::Null, Json::Null],
            ))
        })
        .collect();
    let categories = graph
        .categories
        .iter()
        .map(|c| {
            Json::Arr(vec![
                Json::Str(c.slug.clone()),
                Json::Str(c.title.clone()),
                Json::Arr(
                    c.skill_slugs
                        .iter()
                        .map(|s| {
                            Json::Int(*index.get(s.as_str()).unwrap_or_else(|| {
                                panic!(
                                    "category {:?} lists {s:?}, which is not in the graph",
                                    c.slug
                                )
                            }))
                        })
                        .collect(),
                ),
            ])
        })
        .collect();

    let mut out = String::with_capacity(256 * 1024);
    out.push_str("{\"schema\":");
    out.push_str(&PACKED_SCHEMA.to_string());
    out.push_str(",\"skill_url_template\":");
    write_json(&Json::Str(graph.skill_url_template.clone()), &mut out);
    out.push_str(",\"categories\":");
    write_json(&Json::Arr(categories), &mut out);
    out.push_str(",\"skills\":");
    write_json(&Json::Arr(skills), &mut out);
    out.push('}');
    out
}

/// The banner's numbers and family-level picture.
pub fn banner_summary(graph: &GraphData) -> BannerSummary {
    let weights = edge_weights(graph);
    let category_index: HashMap<&str, usize> = graph
        .categories
        .iter()
        .enumerate()
        .map(|(i, c)| (c.slug.as_str(), i))
        .collect();
    let family_of: HashMap<&str, usize> = graph
        .skills
        .iter()
        .map(|s| (s.slug.as_str(), category_index[s.category.as_str()]))
        .collect();
    let mut links: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for &(source, target, count) in &weights {
        let (a, b) = (family_of[source], family_of[target]);
        if a != b {
            *links.entry((a.min(b), a.max(b))).or_default() += count;
        }
    }
    let pairs: HashSet<(&str, &str)> = weights.iter().map(|&(s, t, _)| (s, t)).collect();
    let mutual_count = weights
        .iter()
        .filter(|&&(a, b, _)| a < b && pairs.contains(&(b, a)))
        .count();

    let mut out: HashMap<&str, Vec<&str>> = HashMap::new();
    for &(source, target, _) in &weights {
        out.entry(source).or_default().push(target);
    }
    let slugs: Vec<&str> = graph.skills.iter().map(|s| s.slug.as_str()).collect();
    let (mut max_steps, mut reachable) = (0, true);
    for start in &slugs {
        let mut dist: HashMap<&str, usize> = HashMap::from([(*start, 0)]);
        let mut frontier = vec![*start];
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for node in frontier {
                for target in out.get(node).map(Vec::as_slice).unwrap_or_default() {
                    if !dist.contains_key(target) {
                        dist.insert(target, dist[node] + 1);
                        next.push(*target);
                    }
                }
            }
            frontier = next;
        }
        if dist.len() < slugs.len() {
            reachable = false;
        }
        max_steps = max_steps.max(dist.values().copied().max().unwrap_or(0));
    }

    BannerSummary {
        skill_count: slugs.len(),
        pair_count: weights.len(),
        mutual_count,
        max_steps,
        reachable,
        families: graph
            .categories
            .iter()
            .enumerate()
            .map(|(index, c)| BannerFamily {
                index,
                title: c.title.clone(),
                count: c.skill_slugs.len(),
            })
            .collect(),
        links: links.into_iter().map(|((a, b), w)| [a, b, w]).collect(),
    }
}

/// Who a skill cites and who cites it, each in library order and without the
/// skill itself, for the strip under every skill page's description.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CitationList {
    pub cites: Vec<String>,
    pub cited_by: Vec<String>,
}

/// slug -> its [`CitationList`], for every skill, read from the same edges
/// the graph draws, so a skill page says what /graph/ shows.
pub fn citation_lists(graph: &GraphData) -> BTreeMap<String, CitationList> {
    let mut cites: HashMap<&str, BTreeSet<&str>> = HashMap::new();
    let mut cited_by: HashMap<&str, BTreeSet<&str>> = HashMap::new();
    let known: BTreeSet<&str> = graph.skills.iter().map(|s| s.slug.as_str()).collect();
    for (source, target, _) in edge_weights(graph) {
        if source != target && known.contains(target) {
            cites.entry(source).or_default().insert(target);
            cited_by.entry(target).or_default().insert(source);
        }
    }
    let in_library_order = |set: Option<&BTreeSet<&str>>| -> Vec<String> {
        graph
            .skills
            .iter()
            .map(|s| s.slug.as_str())
            .filter(|slug| set.is_some_and(|s| s.contains(slug)))
            .map(str::to_string)
            .collect()
    };
    graph
        .skills
        .iter()
        .map(|s| {
            let slug = s.slug.as_str();
            (
                s.slug.clone(),
                CitationList {
                    cites: in_library_order(cites.get(slug)),
                    cited_by: in_library_order(cited_by.get(slug)),
                },
            )
        })
        .collect()
}

/// (citing slug, cited slug, mentions), trigger included, in first-seen order.
fn edge_weights(graph: &GraphData) -> Vec<(&str, &str, usize)> {
    let mut weights: Vec<(&str, &str, usize)> = Vec::new();
    let mut at: HashMap<(&str, &str), usize> = HashMap::new();
    for skill in &graph.skills {
        let sources =
            std::iter::once(&skill.trigger.refs).chain(skill.sections.iter().map(|s| &s.refs));
        for refs in sources {
            for (slug, count) in refs {
                let key = (skill.slug.as_str(), slug.as_str());
                match at.get(&key) {
                    Some(&i) => weights[i].2 += count,
                    None => {
                        at.insert(key, weights.len());
                        weights.push((key.0, key.1, *count));
                    }
                }
            }
        }
    }
    weights
}

// ---------------------------------------------------------------------------
// One skill's node
// ---------------------------------------------------------------------------

static ALWAYS_ON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^Use at the start of every (?:single )?conversation").unwrap()
});

/// One reference file as a graph section: its id on the page, the section
/// title the graph shows, and its rendered HTML.
struct Document<'r> {
    id: &'r str,
    title: String,
    html: &'r str,
}

/// `skill_graph._reference_documents`: the reference files in page order,
/// titled by the group each sits in.
fn reference_documents(references: &References) -> Vec<Document<'_>> {
    fn titled<'r>(group: &str, section: &'r ReferenceSection) -> Document<'r> {
        Document {
            id: &section.id,
            title: format!("{group} · {}", section.title),
            html: &section.html,
        }
    }
    match references {
        References::Plain(sections) => sections.iter().map(|s| titled("Reference", s)).collect(),
        References::Formidable {
            stacks,
            commands,
            craft_floor,
        } => stacks
            .iter()
            .map(|s| titled("Stack", s))
            .chain(commands.iter().map(|s| titled("Command", s)))
            .chain(craft_floor.iter().map(|s| Document {
                id: &s.id,
                title: "Craft floor".to_string(),
                html: &s.html,
            }))
            .collect(),
    }
}

fn build_skill_node(
    skill: &Skill,
    rendered: &RenderedSkill,
    category: &str,
    stamp: ChangeStamp,
) -> SkillNode {
    let mut sections = extract_sections(&rendered.body_html, SectionKind::Body, "Opening", "");
    for document in reference_documents(&rendered.references) {
        // One reference file becomes one section; its own h2s become that
        // section's subs, so the graph keeps both the file and its parts.
        let parts = extract_sections(
            document.html,
            SectionKind::Reference,
            &document.title,
            document.id,
        );
        let mut section = Section {
            id: document.id.to_string(),
            title: document.title.clone(),
            kind: SectionKind::Reference,
            words: 0,
            refs: Vec::new(),
            quotes: Vec::new(),
            subs: Vec::new(),
        };
        for part in parts {
            section.words += part.words;
            for (slug, count) in &part.refs {
                add_count(&mut section.refs, slug, *count);
            }
            for (slug, quote) in &part.quotes {
                if !section.quotes.iter().any(|(s, _)| s == slug) {
                    section.quotes.push((slug.clone(), quote.clone()));
                }
            }
            if part.id != document.id {
                section.subs.push(SubSection {
                    id: part.id,
                    title: part.title,
                    words: part.words,
                    refs: part.refs,
                });
            }
        }
        sections.push(section);
    }

    let trigger = extract_sections(
        &rendered.description_html,
        SectionKind::Trigger,
        "When to use",
        "",
    )
    .into_iter()
    .next()
    .map(|s| Citations {
        refs: s.refs,
        quotes: s.quotes,
    })
    .unwrap_or_default();
    SkillNode {
        slug: skill.slug.clone(),
        name: skill.name.clone(),
        category: category.to_string(),
        summary: rendered.summary.clone(),
        words: sections.iter().map(|s| s.words).sum(),
        always_on: ALWAYS_ON_RE.is_match(&skill.description),
        change_status: stamp.status,
        change_at: stamp.at,
        trigger,
        sections,
    }
}

fn add_count(refs: &mut Refs, slug: &str, count: usize) {
    match refs.iter_mut().find(|(s, _)| s == slug) {
        Some((_, n)) => *n += count,
        None => refs.push((slug.to_string(), count)),
    }
}

// ---------------------------------------------------------------------------
// Reading sections back out of rendered HTML
// ---------------------------------------------------------------------------

/// Elements whose text is one unit of prose for quoting purposes. A
/// citation's quote is the sentence around it, and a sentence never crosses
/// one of these.
const BLOCK_TAGS: [&str; 11] = [
    "p",
    "li",
    "td",
    "th",
    "h2",
    "h3",
    "h4",
    "pre",
    "blockquote",
    "dt",
    "dd",
];

/// The pipeline renders every cross-reference href as `.../skills/<slug>/`;
/// matching the tail keeps this independent of the base path.
static SKILL_HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/skills/([a-z0-9]+(?:-[a-z0-9]+)*)/$").unwrap());

/// Every h2 section of one rendered document, in reading order.
fn extract_sections(
    html: &str,
    kind: SectionKind,
    opening_title: &str,
    opening_id: &str,
) -> Vec<Section> {
    let mut walker = SectionWalker::new(kind, opening_title, opening_id);
    // A frontmatter description renders as bare inline HTML, with no block
    // tag around it, so a citation in it would have no sentence to be quoted
    // from and would reach the tooltip empty.
    if kind == SectionKind::Trigger && !html.is_empty() {
        tokenize(&format!("<p>{html}</p>"), &mut walker);
    } else {
        tokenize(html, &mut walker);
    }
    walker.close()
}

struct Block {
    text: String,
    /// `text`'s length in code points.
    length: usize,
    /// (offset, length, slug), in code points.
    mentions: Vec<(usize, usize, String)>,
    section: usize,
}

enum HeadingTarget {
    Section(usize),
    Sub(usize, usize),
}

/// `_SectionWalker`: splits one rendered document into h2 sections and h3
/// sub-sections, counting words and recording each skill-link inside them.
struct SectionWalker {
    kind: SectionKind,
    sections: Vec<Section>,
    section: usize,
    sub: Option<usize>,
    heading_level: Option<u8>,
    heading_target: Option<HeadingTarget>,
    heading_text: String,
    blocks: Vec<Block>,
    link_slug: Option<String>,
    link_start: usize,
}

impl SectionWalker {
    fn new(kind: SectionKind, opening_title: &str, opening_id: &str) -> Self {
        let mut walker = SectionWalker {
            kind,
            sections: Vec::new(),
            section: 0,
            sub: None,
            heading_level: None,
            heading_target: None,
            heading_text: String::new(),
            blocks: Vec::new(),
            link_slug: None,
            link_start: 0,
        };
        walker.section = walker.new_section(opening_id, opening_title);
        walker
    }

    fn new_section(&mut self, id: &str, title: &str) -> usize {
        self.sections.push(Section {
            id: id.to_string(),
            title: title.to_string(),
            kind: self.kind,
            words: 0,
            refs: Vec::new(),
            quotes: Vec::new(),
            subs: Vec::new(),
        });
        self.sections.len() - 1
    }

    fn start_tag(&mut self, tag: &str, attrs: &[(String, String)]) {
        let attr = |name: &str| {
            attrs
                .iter()
                .find(|(k, _)| k == name)
                .map_or("", |(_, v)| v.as_str())
        };
        if tag == "h2" || tag == "h3" {
            let heading_id = attr("id").to_string();
            if tag == "h2" {
                self.section = self.new_section(&heading_id, "");
                self.sub = None;
                self.heading_target = Some(HeadingTarget::Section(self.section));
            } else {
                let subs = &mut self.sections[self.section].subs;
                subs.push(SubSection {
                    id: heading_id,
                    title: String::new(),
                    words: 0,
                    refs: Vec::new(),
                });
                self.sub = Some(subs.len() - 1);
                self.heading_target = Some(HeadingTarget::Sub(self.section, subs.len() - 1));
            }
            self.heading_level = Some(if tag == "h2" { 2 } else { 3 });
            self.heading_text.clear();
        }
        if BLOCK_TAGS.contains(&tag) {
            self.blocks.push(Block {
                text: String::new(),
                length: 0,
                mentions: Vec::new(),
                section: self.section,
            });
        }
        if tag == "a" && attr("class").split_whitespace().any(|c| c == "skill-link") {
            let slug = SKILL_HREF_RE
                .captures(attr("href"))
                .map(|c| c[1].to_string());
            if let Some(slug) = slug {
                self.link_start = self.blocks.last().map_or(0, |b| b.length);
                self.record_ref(&slug);
                self.link_slug = Some(slug);
            }
        }
    }

    fn end_tag(&mut self, tag: &str) {
        // The link is over either way; a mention is recorded only inside a block.
        if tag == "a"
            && let Some(slug) = self.link_slug.take()
            && let Some(block) = self.blocks.last_mut()
        {
            block
                .mentions
                .push((self.link_start, block.length - self.link_start, slug));
        }
        let level = match tag {
            "h2" => Some(2),
            "h3" => Some(3),
            _ => None,
        };
        if level.is_some() && self.heading_level == level {
            let title = self
                .heading_text
                .split(is_py_space)
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            match self.heading_target.take() {
                Some(HeadingTarget::Section(i)) => self.sections[i].title = title,
                Some(HeadingTarget::Sub(i, j)) => self.sections[i].subs[j].title = title,
                None => {}
            }
            self.heading_level = None;
        }
        if BLOCK_TAGS.contains(&tag)
            && let Some(block) = self.blocks.pop()
        {
            self.flush_block(block);
        }
    }

    fn data(&mut self, data: &str) {
        if self.heading_level.is_some() {
            self.heading_text.push_str(data);
        }
        let words = count_words(data);
        let section = &mut self.sections[self.section];
        section.words += words;
        if let Some(sub) = self.sub {
            section.subs[sub].words += words;
        }
        if let Some(block) = self.blocks.last_mut() {
            block.text.push_str(data);
            block.length += data.chars().count();
        }
    }

    fn record_ref(&mut self, slug: &str) {
        let section = &mut self.sections[self.section];
        add_count(&mut section.refs, slug, 1);
        if let Some(sub) = self.sub {
            add_count(&mut section.subs[sub].refs, slug, 1);
        }
    }

    fn flush_block(&mut self, block: Block) {
        if block.mentions.is_empty() {
            return;
        }
        let text: Vec<char> = block
            .text
            .chars()
            .map(|c| if c == '\n' { ' ' } else { c })
            .collect();
        let quotes = &mut self.sections[block.section].quotes;
        for (offset, length, slug) in block.mentions {
            if !quotes.iter().any(|(s, _)| *s == slug) {
                let quote = quote_around(&text, offset, length);
                quotes.push((slug, quote));
            }
        }
    }

    fn close(mut self) -> Vec<Section> {
        while let Some(block) = self.blocks.pop() {
            self.flush_block(block);
        }
        // The stretch before the first heading only counts as a section when
        // it says something; most skills open straight into their first h2.
        if self
            .sections
            .first()
            .is_some_and(|s| s.words == 0 && s.refs.is_empty())
        {
            self.sections.remove(0);
        }
        self.sections
    }
}

/// Feeds `html` to the walker the way Python's `html.parser.HTMLParser`
/// (with `convert_charrefs=True`) reports it for the pipeline's output: one
/// data event per run of text between tags, character references decoded,
/// tag and attribute names lower-cased. It covers the markup the pipeline
/// emits (no comments, doctypes or raw-text elements); a `<` that opens no
/// tag is data, as in Python.
fn tokenize(html: &str, walker: &mut SectionWalker) {
    let mut rest = html;
    while !rest.is_empty() {
        let Some(lt) = rest.find('<') else {
            walker.data(&unescape(rest));
            return;
        };
        if lt > 0 {
            walker.data(&unescape(&rest[..lt]));
        }
        rest = &rest[lt..];
        let after = &rest[1..];
        let is_end =
            after.starts_with('/') && after[1..].starts_with(|c: char| c.is_ascii_alphabetic());
        let is_start = after.starts_with(|c: char| c.is_ascii_alphabetic());
        let close = if is_end || is_start {
            tag_end(rest)
        } else {
            None
        };
        let Some(close) = close else {
            walker.data("<");
            rest = after;
            continue;
        };
        let tag = &rest[1..close];
        if is_end {
            let name = tag[1..].trim_end_matches(is_py_space).to_ascii_lowercase();
            walker.end_tag(&name);
        } else {
            let (name, attrs) = parse_start_tag(tag);
            walker.start_tag(&name, &attrs);
        }
        rest = &rest[close + 1..];
    }
}

/// The index of the `>` closing the tag at the start of `text`, skipping
/// quoted attribute values.
fn tag_end(text: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in text.char_indices().skip(1) {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(i),
            (None, _) => {}
        }
    }
    None
}

/// `name attr="v" attr='v' attr=v attr` (the text between `<` and `>`).
fn parse_start_tag(tag: &str) -> (String, Vec<(String, String)>) {
    let tag = tag.trim_end_matches('/');
    let name_end = tag
        .find(|c: char| is_py_space(c) || c == '/')
        .unwrap_or(tag.len());
    let name = tag[..name_end].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut rest = &tag[name_end..];
    loop {
        rest = rest.trim_start_matches(|c: char| is_py_space(c) || c == '/');
        if rest.is_empty() {
            break;
        }
        let key_end = rest
            .find(|c: char| is_py_space(c) || c == '=')
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start_matches(is_py_space);
        let mut value = String::new();
        if let Some(after_eq) = rest.strip_prefix('=') {
            let after_eq = after_eq.trim_start_matches(is_py_space);
            let (raw, remainder) = match after_eq.chars().next() {
                Some(q @ ('"' | '\'')) => {
                    let body = &after_eq[1..];
                    let end = body.find(q).unwrap_or(body.len());
                    (&body[..end], body.get(end + 1..).unwrap_or(""))
                }
                _ => {
                    let end = after_eq.find(is_py_space).unwrap_or(after_eq.len());
                    (&after_eq[..end], &after_eq[end..])
                }
            };
            value = unescape(raw);
            rest = remainder;
        }
        attrs.push((key, value));
    }
    (name, attrs)
}

/// Decodes the character references the pipeline's escaper writes (`&amp;
/// &lt; &gt; &quot;`, plus `&#x27;`, `&apos;` and numeric references).
/// Anything else is left as written; the pipeline never produces it, since
/// it escapes every `&` in the source.
fn unescape(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let decoded = rest.find(';').and_then(|semi| {
            let name = &rest[1..semi];
            let c = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => name
                    .strip_prefix("#x")
                    .or_else(|| name.strip_prefix("#X"))
                    .map(|hex| u32::from_str_radix(hex, 16))
                    .or_else(|| name.strip_prefix('#').map(str::parse::<u32>))
                    .and_then(Result::ok)
                    .and_then(char::from_u32),
            };
            c.map(|c| (c, semi))
        });
        match decoded {
            Some((c, semi)) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Words and quotes
// ---------------------------------------------------------------------------

static WORD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*").unwrap());

/// `_LABEL_RE`, anchored (Python's `re.match`).
static LABEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    let word = r"[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*";
    let more = LABEL_MAX_WORDS - 1;
    Regex::new(&format!(
        r"^((?:{word})(?:[\s\x1C-\x1F]+{word}){{0,{more}}}):[\s\x1C-\x1F]"
    ))
    .unwrap()
});

fn count_words(text: &str) -> usize {
    WORD_RE.find_iter(text).count()
}

fn count_words_in(chars: &[char]) -> usize {
    count_words(&chars.iter().collect::<String>())
}

/// Python's `str.isspace`.
fn is_py_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

fn py_strip(s: &str) -> &str {
    s.trim_matches(is_py_space)
}

/// Python `str.find(sub, start)` / `str.rfind(sub, start, end)` for one
/// character, on code points, with Python's slice-bound clamping.
fn find_char(chars: &[char], needle: char, start: isize, end: isize, from_right: bool) -> isize {
    let len = chars.len() as isize;
    let clamp = |i: isize| if i < 0 { (i + len).max(0) } else { i.min(len) };
    let (lo, hi) = (clamp(start), clamp(end));
    if lo >= hi {
        return -1;
    }
    let window = &chars[lo as usize..hi as usize];
    let found = if from_right {
        window.iter().rposition(|&c| c == needle)
    } else {
        window.iter().position(|&c| c == needle)
    };
    found.map_or(-1, |i| lo + i as isize)
}

/// Python's `s[lo:hi]` on code points.
fn py_slice(chars: &[char], lo: isize, hi: isize) -> &[char] {
    let len = chars.len() as isize;
    let clamp = |i: isize| if i < 0 { (i + len).max(0) } else { i.min(len) };
    let (lo, hi) = (clamp(lo), clamp(hi));
    if lo >= hi {
        &[]
    } else {
        &chars[lo as usize..hi as usize]
    }
}

/// `_quote_around`: the sentence of `text` containing the span at `offset`,
/// trimmed to [`QUOTE_MAX_LENGTH`] around that span with an ellipsis on
/// whichever side lost words. Sentence ends are ". ", "? ", "! " and "; ";
/// a colon cuts only after a clause longer than `LABEL_MAX_WORDS`.
fn quote_around(text: &[char], offset: usize, length: usize) -> String {
    let mut start = 0;
    // `[.?!;:](?=\s)` over text[:offset]: the lookahead cannot see past it.
    for i in 0..offset {
        let c = text[i];
        if matches!(c, '.' | '?' | '!' | ';' | ':') && i + 1 < offset && is_py_space(text[i + 1]) {
            // A colon after a short label ("Not for:") is part of the claim.
            if c == ':' && count_words_in(&text[start..i]) <= LABEL_MAX_WORDS {
                continue;
            }
            start = i + 1;
        }
    }
    let tail = offset + length;
    let end = (tail..text.len())
        .find(|&i| {
            matches!(text[i], '.' | '?' | '!' | ';')
                && (i + 1 == text.len() || is_py_space(text[i + 1]))
        })
        .map_or(text.len(), |i| i + 1);
    let raw = &text[start..end];
    let leading = raw.iter().take_while(|&&c| is_py_space(c)).count();
    let mention_at = offset as isize - start as isize - leading as isize;
    // A clause cut at a semicolon or colon ends on that mark, which reads as
    // a sentence left hanging; the quote stands better without it.
    let mut sentence: &[char] = &raw[leading..];
    while sentence.last().is_some_and(|&c| is_py_space(c)) {
        sentence = &sentence[..sentence.len() - 1];
    }
    while sentence
        .last()
        .is_some_and(|&c| matches!(c, ';' | ':' | ','))
    {
        sentence = &sentence[..sentence.len() - 1];
    }
    let mut max_length = QUOTE_MAX_LENGTH as isize;
    let len = sentence.len() as isize;
    if len <= max_length {
        return sentence.iter().collect();
    }
    // A kept label ("Not for:") is spent from the budget up front so a trim
    // that drops it can put it back, rather than opening on "…for:".
    let sentence_text: String = sentence.iter().collect();
    let label = LABEL_RE
        .captures(&sentence_text)
        .filter(|c| {
            (sentence_text[..c.get(0).unwrap().end()].chars().count() as isize)
                < (offset - start) as isize
        })
        .map(|c| format!("{}:", &c[1]))
        .unwrap_or_default();
    let label_len = label.chars().count() as isize;
    if !label.is_empty() {
        max_length -= label_len + 2;
    }
    // Centre the window on the mention, then snap both edges to word breaks.
    let length = length as isize;
    let half = (max_length - length).div_euclid(2);
    let mut lo = (mention_at - half).max(0);
    let mut hi = len.min(lo + max_length);
    lo = (hi - max_length).max(0);
    if lo > 0 {
        let space = find_char(sentence, ' ', lo, len, false);
        if 0 <= space && space < mention_at {
            lo = space + 1;
        }
    }
    if hi < len {
        let space = find_char(sentence, ' ', mention_at + length, hi, true);
        if space > 0 {
            hi = space;
        }
    }
    let mut quote = String::new();
    if lo > 0 {
        if !label.is_empty() && lo > label_len {
            quote.push_str(&label);
            quote.push_str(" …");
        } else {
            quote.push('…');
        }
    }
    let window: String = py_slice(sentence, lo, hi).iter().collect();
    quote.push_str(py_strip(&window));
    if hi < len {
        quote.push('…');
    }
    quote
}

// ---------------------------------------------------------------------------
// Packed JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Json {
    Null,
    Bool(bool),
    Int(usize),
    Str(String),
    Arr(Vec<Json>),
}

/// Drop trailing fields that equal their default; the defaults line up with
/// the *end* of `fields`.
fn trim(mut fields: Vec<Json>, defaults: &[Json]) -> Vec<Json> {
    let offset = fields.len() - defaults.len();
    while fields.len() > offset && fields.last() == Some(&defaults[fields.len() - 1 - offset]) {
        fields.pop();
    }
    fields
}

fn write_json(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Str(s) => write_json_string(s, out),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json(item, out);
            }
            out.push(']');
        }
    }
}

/// Python's `json.dumps(s, ensure_ascii=False)` for one string.
fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod citation_tests {
    use super::*;

    fn link(slug: &str) -> String {
        format!("<a class=\"skill-link\" href=\"/skills/{slug}/\">{slug}</a>")
    }

    #[test]
    fn a_citation_in_a_description_carries_its_sentence() {
        let html = format!("Use when naming; pairs with {}.", link("beta"));
        let sections = extract_sections(&html, SectionKind::Trigger, "When to use", "");
        assert_eq!(sections[0].refs, vec![("beta".to_string(), 1)]);
        assert_eq!(
            sections[0].quotes,
            vec![("beta".to_string(), "pairs with beta.".to_string())]
        );
    }

    fn node(slug: &str, refs: &[(&str, usize)]) -> SkillNode {
        let refs: Refs = refs.iter().map(|(s, n)| (s.to_string(), *n)).collect();
        SkillNode {
            slug: slug.to_string(),
            name: slug.to_string(),
            category: String::new(),
            summary: String::new(),
            words: 0,
            always_on: false,
            change_status: None,
            change_at: None,
            trigger: Citations::default(),
            sections: vec![Section {
                id: String::new(),
                title: String::new(),
                kind: SectionKind::Body,
                words: 0,
                refs,
                quotes: Vec::new(),
                subs: Vec::new(),
            }],
        }
    }

    #[test]
    fn citation_lists_run_both_ways_in_library_order_without_the_skill_itself() {
        let graph = GraphData {
            skill_url_template: String::new(),
            categories: Vec::new(),
            skills: vec![
                node("a", &[("b", 1), ("a", 2)]),
                node("b", &[]),
                node("c", &[("b", 1)]),
            ],
        };
        let lists = citation_lists(&graph);
        assert_eq!(
            lists["a"],
            CitationList {
                cites: vec!["b".to_string()],
                cited_by: Vec::new()
            }
        );
        assert_eq!(
            lists["b"],
            CitationList {
                cites: Vec::new(),
                cited_by: vec!["a".to_string(), "c".to_string()]
            }
        );
    }
}
