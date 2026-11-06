//! One skill's page content, rendered: `content_pipeline.py`'s
//! `build_plain_skill_entry` and `build_formidable_skill_entry`, minus the
//! locale plumbing (the site has one locale).
//!
//! Both the skill page and the graph read this. The graph reads it back out
//! of the HTML on purpose (see `graph`), so the two must come from the same
//! rendering: an edge exists in graph.json exactly when a reader can click
//! that cross-reference on a page.

use std::collections::{BTreeMap, BTreeSet};

use crate::markdown::{self, ReferenceSection, RenderCtx, SUMMARY_MAX_LENGTH};
use crate::model::{Reference, Skill};

/// The one skill whose reference tree is split into stacks, commands and the
/// craft floor (content_pipeline.py `FORMIDABLE_SLUG`).
pub const FORMIDABLE_SLUG: &str = "formidable";

/// formidable's `reference/craft-floor.md`: a section of its own rather than
/// a command tab.
const CRAFT_FLOOR_STEM: &str = "craft-floor";

/// A skill's rendered content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedSkill {
    /// The SKILL.md body without its `# Title` line.
    pub body_html: String,
    /// The description with its cross-references linked, for the page lede.
    pub description_html: String,
    /// The card teaser and `<meta name="description">`.
    pub summary: String,
    pub references: References,
}

/// A skill's reference files, as its page lays them out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum References {
    /// Every skill but formidable: `reference/*.md` by name, ids `ref-<stem>`.
    /// Empty for a skill with no reference tree.
    Plain(Vec<ReferenceSection>),
    /// formidable: `reference/stacks/*.md` (`stack-<stem>`), then
    /// `reference/*.md` (`cmd-<stem>`) without the craft floor, which comes
    /// last on its own (`cmd-craft-floor`).
    Formidable {
        stacks: Vec<ReferenceSection>,
        commands: Vec<ReferenceSection>,
        craft_floor: Option<ReferenceSection>,
    },
}

/// Renders `skill` for its page. `known_slugs` is every skill that exists
/// (a cross-reference links only to one of those, never to `skill` itself);
/// skill links are `{base_path}/skills/<slug>/`.
pub fn render_skill(
    skill: &Skill,
    known_slugs: &BTreeSet<String>,
    base_path: &str,
) -> RenderedSkill {
    let formidable = skill.slug == FORMIDABLE_SLUG;
    let direct: Vec<(&str, &Reference)> = files_in(skill, "reference/");
    let stacks: Vec<(&str, &Reference)> = if formidable {
        files_in(skill, "reference/stacks/")
    } else {
        Vec::new()
    };

    // make_reference_link_resolver: a relative `.md` link resolves by its
    // file stem to the section that file is rendered as.
    let mut anchors = BTreeMap::new();
    if formidable {
        anchors.extend(
            stacks
                .iter()
                .map(|(s, _)| (s.to_string(), format!("stack-{s}"))),
        );
        anchors.extend(
            direct
                .iter()
                .map(|(s, _)| (s.to_string(), format!("cmd-{s}"))),
        );
    } else {
        anchors.extend(
            direct
                .iter()
                .map(|(s, _)| (s.to_string(), format!("ref-{s}"))),
        );
    }

    let ctx = RenderCtx {
        known_slugs,
        base_path,
        link_slugs: true,
        current_slug: Some(&skill.slug),
        relative_anchors: Some(&anchors),
        heading_id_prefix: "",
    };
    let section = |prefix: &str, (stem, file): &(&str, &Reference)| {
        markdown::render_reference(stem, prefix, &file.body, &ctx)
    };

    let references = if formidable {
        References::Formidable {
            stacks: stacks.iter().map(|f| section("stack", f)).collect(),
            commands: direct
                .iter()
                .filter(|(stem, _)| *stem != CRAFT_FLOOR_STEM)
                .map(|f| section("cmd", f))
                .collect(),
            craft_floor: direct
                .iter()
                .find(|(stem, _)| *stem == CRAFT_FLOOR_STEM)
                .map(|f| section("cmd", f)),
        }
    } else {
        References::Plain(direct.iter().map(|f| section("ref", f)).collect())
    };

    RenderedSkill {
        body_html: markdown::render(markdown::strip_title_heading(&skill.body), &ctx),
        description_html: markdown::render_inline(&skill.description, &ctx),
        summary: markdown::summarize_description(&skill.description, SUMMARY_MAX_LENGTH),
        references,
    }
}

/// The skill's `<dir>*.md` files (directly in `dir`, not below it), with
/// their file stems, in the order `load_skills` read them (by name).
fn files_in<'s>(skill: &'s Skill, dir: &str) -> Vec<(&'s str, &'s Reference)> {
    skill
        .references
        .iter()
        .filter_map(|r| {
            let name = r.rel_path.strip_prefix(dir)?;
            (!name.contains('/')).then(|| (file_stem(name), r))
        })
        .collect()
}

/// Python's `PurePosixPath(name).stem` for a bare file name.
fn file_stem(name: &str) -> &str {
    match name.rfind('.') {
        Some(i) if i > 0 => &name[..i],
        _ => name,
    }
}
