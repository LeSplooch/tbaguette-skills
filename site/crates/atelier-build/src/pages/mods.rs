//! /mods/, /mods/climb/ and a page for every other mod (`/mods/<slug>/`).
//!
//! A mod is a separate plugin in the same marketplace that changes Claude
//! Code itself. Every fact these pages state about a mod comes from its own
//! files, read at build time: `mods/<slug>/.claude-plugin/plugin.json` and
//! `mods/<slug>/README.md`. The README is parsed into its sections (the
//! install lines, the commands block, the labelled bullets, the limits) and
//! the build fails, naming the file, when a part a page shows is missing, so
//! a rewritten README can never leave a page quietly stating the old facts.
//!
//! /mods/climb/ also shows the real log of the first live climb
//! (`site/content/climb-live-run.tsv`): as a table, and replayed as a mock of
//! the Claude Code session by atelier-web's `replay` module. The replay's
//! markup is its finished state, which is what a reader without WASM or
//! with reduced motion sees.

use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use atelier_core::markdown::{self, RenderCtx};

use super::shell::{self, Page};
use super::{BRAND_ATELIER_TEXT, MODS_PATH, PageCtx, escape_html, join, strings as chrome};
use crate::repo::{CLIMB_LIVE_LOG_PATH, ClimbLogRow, ClimbRunMeta, ModManifest, ModSource};

/// The one mod whose page is its own design. Every other mod gets the plain
/// page (see [`plain_page`]).
pub const CLIMB_SLUG: &str = "climb";
const WATCH_SLUG: &str = "watch";

/// The pages' own copy (everything that is not a fact about a mod).
mod strings {
    pub const MODS_TITLE: &str = "Mods";
    pub const MODS_META: &str = "Mods change Claude Code itself. Each is its own plugin in the TBaguette marketplace, installed on its own; no skill depends on one.";
    pub const MODS_LEDE: &str = "A skill changes what your agent knows to do. A mod changes Claude Code itself: what it shows you, and what it does between your turns.";
    pub const MODS_FACT_PLUGIN: &str =
        "Each mod is its own plugin, in the same marketplace as the skills.";
    pub const MODS_FACT_INSTALL: &str = "You install each one on its own, only the ones you want.";
    pub const MODS_FACT_NO_DEPENDENCY: &str =
        "No skill depends on a mod: the atelier works the same without them.";
    pub const MODS_LIST_HEADING: &str = "The mods";
    pub const MODS_CARD_CTA_TEMPLATE: &str = "Meet {name}";
    pub const BREADCRUMB_MODS: &str = "Mods";
    pub const COPY_LABEL: &str = "Copy";
    pub const COPY_ARIA_TEMPLATE: &str = "Copy the {name} install command";
    /// The hero's copy buttons, one per install way, told apart by where the
    /// command runs.
    pub const COPY_ARIA_FIRST_TEMPLATE: &str =
        "Copy the {name} install command for a terminal session";
    pub const COPY_ARIA_SHELL_TEMPLATE: &str = "Copy the {name} install command for a shell";
    pub const READ_SKILL_TEMPLATE: &str = "Read {skill}";
    pub const SOURCE_LINK: &str = "Source on GitHub";
    pub const OTHER_WAYS: &str = "Other ways to install";
    pub const REPLAY_LABEL: &str = "A replay of the first live climb";
    pub const REPLAY_AGAIN: &str = "Replay";
    pub const REPLAY_CAPTION: &str = "Replayed from the log further down, started as that run was: the same budget, check and protected file. The exit codes, scores and timings are the run’s own.";
    pub const REPLAY_PROMPT_HINT: &str = "Your prompt stays free while it climbs";
    pub const REPLAY_LOG_FILE: &str = ".climb-log.tsv";
    pub const HOW_HEADING: &str = "How it works";
    pub const HOW_INTRO: &str =
        "Every iteration runs this loop. The check’s exit code is the only vote.";
    pub const RING_LABEL: &str = "The loop, drawn as a ring";
    pub const RING_CHANGE_NAME: &str = "The model’s turn";
    pub const RING_CHANGE_NOTE: &str = "The model works toward passing, then ends its turn.";
    pub const RING_CHECK_NAME: &str = "The check runs";
    pub const RING_CHECK_NOTE: &str = "The mod runs your check itself. Its exit code counts; the model’s account of how it went does not.";
    pub const RING_LOG_NAME: &str = "A row in the log";
    pub const RING_LOG_NOTE: &str = "Exit code, score, check time and turn time, measured by the mod and the engine, not reported by the model.";
    pub const RING_NEXT_NAME: &str = "Bar met, or again";
    pub const RING_NEXT_NOTE: &str = "Exit 0 ends the climb. Anything else queues the next iteration, until a stop condition fires.";
    pub const RING_CENTRE_TEMPLATE: &str = "iteration {iteration}";
    pub const COMMANDS_HEADING: &str = "Commands";
    pub const COMMANDS_COMMAND: &str = "Command";
    pub const COMMANDS_DOES: &str = "What it does";
    pub const STOPS_HEADING: &str = "What stops it";
    pub const PROTECTS_HEADING: &str = "What it protects";
    pub const LIMIT_LABEL: &str = "The limit";
    pub const LOG_HEADING: &str = "The log, from a real run";
    /// The log's caption. Every fact in it is read: the date, the project,
    /// what the check counted and the restart from climb-live-run.meta.tsv,
    /// the first score and the passing iteration from the log itself.
    pub const LOG_CAPTION_TEMPLATE: &str = "The first live climb, on {date}: {project}, checked by {checked_by}. {restart}It met the bar at iteration {met}. Shown as the mod wrote it.";
    pub const LOG_CAPTION_RESTART_TEMPLATE: &str = "The climb had been restarted {restarted}, so its first row starts at {first_score}: the earlier attempt had already added {carried}. ";
    pub const LOG_CAPTION_UNMET_TEMPLATE: &str =
        "The first live climb, on {date}, as the mod wrote it: {rows} rows.";
    pub const REQUIREMENTS_HEADING: &str = "Requirements and limits";
    pub const REQ_RUNS_IN: &str = "Runs in";
    pub const REQ_RUNS_IN_VALUE: &str = "Claude Code, in a build with mods";
    pub const REQ_VERSION: &str = "Version";
    pub const REQ_LICENCE: &str = "Licence";
    pub const REQ_SOURCE: &str = "Source";
    pub const WATCH_WHO_HEADING: &str = "Who can start one";
    pub const USE_HEADING: &str = "In use";
}

// ---------------------------------------------------------------------------
// The README, as blocks
// ---------------------------------------------------------------------------

/// A README read as the handful of Markdown shapes the mods' READMEs use.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Readme {
    /// The `# ` heading.
    pub title: String,
    /// The blocks before the first `## `.
    pub intro: Vec<Block>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A paragraph, its lines joined by spaces.
    Para(String),
    /// An indented code block, one string per line, the indent removed.
    Code(Vec<String>),
    /// A `- ` list, one string per item, continuation lines joined.
    List(Vec<String>),
}

/// Splits a README into its title, intro and `## ` sections.
pub fn parse_readme(text: &str) -> Readme {
    let mut readme = Readme::default();
    let mut blocks: Vec<Block> = Vec::new();
    let mut heading: Option<String> = None;
    let mut blank = false;

    fn close(readme: &mut Readme, heading: &mut Option<String>, blocks: &mut Vec<Block>) {
        let taken = std::mem::take(blocks);
        match heading.take() {
            Some(h) => readme.sections.push(Section {
                heading: h,
                blocks: taken,
            }),
            None => readme.intro.extend(taken),
        }
    }

    for line in text.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            readme.title = title.trim().to_string();
            blank = true;
            continue;
        }
        if let Some(h) = line.strip_prefix("## ") {
            close(&mut readme, &mut heading, &mut blocks);
            heading = Some(h.trim().to_string());
            blank = true;
            continue;
        }
        if line.trim().is_empty() {
            blank = true;
            continue;
        }
        let in_list_item = matches!(blocks.last(), Some(Block::List(_))) && !blank;
        if let (Some(code), false) = (line.strip_prefix("    "), in_list_item) {
            match blocks.last_mut() {
                Some(Block::Code(lines)) => {
                    if blank {
                        lines.push(String::new());
                    }
                    lines.push(code.to_string());
                }
                _ => blocks.push(Block::Code(vec![code.to_string()])),
            }
        } else if let Some(item) = line.strip_prefix("- ") {
            match blocks.last_mut() {
                Some(Block::List(items)) => items.push(item.trim().to_string()),
                _ => blocks.push(Block::List(vec![item.trim().to_string()])),
            }
        } else if in_list_item && line.starts_with(' ') {
            if let Some(Block::List(items)) = blocks.last_mut()
                && let Some(last) = items.last_mut()
            {
                last.push(' ');
                last.push_str(line.trim());
            }
        } else {
            match blocks.last_mut() {
                Some(Block::Para(text)) if !blank => {
                    text.push(' ');
                    text.push_str(line.trim());
                }
                _ => blocks.push(Block::Para(line.trim().to_string())),
            }
        }
        blank = false;
    }
    close(&mut readme, &mut heading, &mut blocks);
    readme
}

impl Readme {
    fn section(&self, heading: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.heading == heading)
    }
}

/// The commands block of a README's Use section as (command, what it does):
/// a line starting with `/` opens a row, split at its first run of two
/// spaces; an indented line continues the row above.
pub fn commands(code: &[String]) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in code {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with('/') {
            let (command, does) = match line.find("  ") {
                Some(at) => (&line[..at], line[at..].trim()),
                None => (line.as_str(), ""),
            };
            rows.push((command.trim().to_string(), does.to_string()));
        } else if let Some((_, does)) = rows.last_mut() {
            if !does.is_empty() {
                does.push(' ');
            }
            does.push_str(trimmed);
        }
    }
    rows
}

/// The bold label a bullet opens with (`**Stops** on ...` -> `Stops`).
fn bullet_label(item: &str) -> Option<&str> {
    let rest = item.strip_prefix("**")?;
    Some(&rest[..rest.find("**")?])
}

/// Sentences of a plain paragraph: split after `. ` (no abbreviation in the
/// mods' READMEs ends with one).
fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(". ") {
        out.push(&rest[..at + 1]);
        rest = rest[at + 2..].trim_start();
    }
    if !rest.trim().is_empty() {
        out.push(rest.trim());
    }
    out
}

// ---------------------------------------------------------------------------
// What the pages show, read and checked before anything is written
// ---------------------------------------------------------------------------

/// Every page's content, built from the mods' files and the live log.
#[derive(Debug, Clone)]
pub struct ModsSite {
    cards: Vec<ModCard>,
    climb: ClimbPage,
    /// Every mod but climb, in `mods/` order.
    plain: Vec<PlainPage>,
}

#[derive(Debug, Clone)]
struct ModCard {
    slug: String,
    name: String,
    version: String,
    description_html: String,
    install_command: String,
}

#[derive(Debug, Clone)]
struct InstallWay {
    lead_html: String,
    command: String,
}

/// What every mod's page opens with, read from its manifest and README.
#[derive(Debug, Clone)]
struct ModHead {
    slug: String,
    manifest: ModManifest,
    /// The manifest description up to its colon ("Run a goal as a climb.").
    headline: String,
    /// The rest of it, capitalised.
    lede: String,
    install: Vec<InstallWay>,
    install_notes_html: Vec<String>,
    commands: Vec<(String, String)>,
    /// The paragraphs of the README's `## Use`.
    use_paragraphs_html: Vec<String>,
    skill_slug: String,
    source_url: String,
}

#[derive(Debug, Clone)]
struct ClimbPage {
    head: ModHead,
    stops_html: Vec<String>,
    protected_html: String,
    limit_html: String,
    log_html: String,
    band_html: Option<String>,
    limits_html: Vec<String>,
    rows: Vec<ClimbLogRow>,
    /// What the live run was started with (its log does not record it).
    run: ClimbRunMeta,
}

#[derive(Debug, Clone)]
struct PlainPage {
    head: ModHead,
    /// The README's heading for how it behaves, and its bullets.
    behaves_heading: String,
    behaves_html: Vec<String>,
    /// The paragraph that closes that section.
    limits_html: Vec<String>,
}

/// The labels of the Use bullets "What stops it" shows, in its order.
const STOP_LABELS: [&str; 4] = [
    "Bar met",
    "Score",
    "Stops",
    "Only its own prompts advance it.",
];

impl ModsSite {
    /// Reads every mod into its card and its page. Fails naming the file
    /// when the README lacks a part the page
    /// shows, or when the manifest's homepage is not a skill page of a skill
    /// that exists.
    pub fn read(
        ctx: &PageCtx,
        mods: &[ModSource],
        log: Vec<ClimbLogRow>,
        run: ClimbRunMeta,
    ) -> Result<ModsSite> {
        let known: BTreeSet<String> = ctx.skills.keys().cloned().collect();
        let render = RenderCtx {
            known_slugs: &known,
            base_path: &ctx.base_path,
            link_slugs: true,
            current_slug: None,
            relative_anchors: None,
            heading_id_prefix: "",
        };
        let mut cards = Vec::new();
        let mut climb = None;
        let mut plain = Vec::new();
        for source in mods {
            let readme = parse_readme(&source.readme);
            let head = match source.slug.as_str() {
                CLIMB_SLUG => {
                    let page = climb_page(source, &readme, &render, ctx, log.clone(), run.clone())?;
                    let head = page.head.clone();
                    climb = Some(page);
                    head
                }
                _ => {
                    let page = plain_page(source, &readme, &render, ctx)?;
                    let head = page.head.clone();
                    plain.push(page);
                    head
                }
            };
            cards.push(ModCard {
                slug: source.slug.clone(),
                name: source.manifest.name.clone(),
                version: source.manifest.version.clone(),
                description_html: markdown::render_inline(&source.manifest.description, &render),
                install_command: head.install[0].command.clone(),
            });
        }
        let climb = climb.ok_or_else(|| anyhow!("mods/{CLIMB_SLUG}: missing"))?;
        Ok(ModsSite {
            cards,
            climb,
            plain,
        })
    }
}

impl ModsSite {
    /// Every mod's manifest name, in /mods/'s order (the landing page's
    /// counter names them).
    pub fn names(&self) -> Vec<String> {
        self.cards.iter().map(|c| c.name.clone()).collect()
    }

    /// The slugs of the mods that get the plain page, in `mods/` order.
    pub fn plain_slugs(&self) -> Vec<String> {
        self.plain.iter().map(|p| p.head.slug.clone()).collect()
    }
}

/// The parts of a README and manifest every mod's page shows. Fails naming
/// the file when one is missing.
fn mod_head(
    source: &ModSource,
    readme: &Readme,
    render: &RenderCtx,
    ctx: &PageCtx,
) -> Result<ModHead> {
    let file = source.readme_path.display();
    let missing = |what: &str| anyhow!("{file}: {what}, which /mods/{}/ shows", source.slug);
    let inline = |text: &str| markdown::render_inline(text, render);

    let install_section = readme
        .section("Install")
        .ok_or_else(|| missing("no `## Install` section"))?;
    let mut install = Vec::new();
    let mut install_notes_html = Vec::new();
    let mut lead: Option<&str> = None;
    for block in &install_section.blocks {
        match block {
            Block::Para(text) => {
                if let Some(earlier) = lead.replace(text) {
                    install_notes_html.push(inline(earlier));
                }
            }
            Block::Code(lines) => {
                let lead_text = lead.take().ok_or_else(|| {
                    missing("an install command with no line saying where it runs")
                })?;
                install.push(InstallWay {
                    lead_html: inline(lead_text),
                    command: lines.join("\n"),
                });
            }
            Block::List(_) => {}
        }
    }
    if let Some(last) = lead {
        install_notes_html.push(inline(last));
    }
    if install.is_empty() {
        return Err(missing("no install command under `## Install`"));
    }

    let use_section = readme
        .section("Use")
        .ok_or_else(|| missing("no `## Use` section"))?;
    let commands_rows = use_section
        .blocks
        .iter()
        .find_map(|b| match b {
            Block::Code(lines) => Some(commands(lines)),
            _ => None,
        })
        .filter(|rows| !rows.is_empty())
        .ok_or_else(|| missing("no commands block under `## Use`"))?;
    let commands_html: Vec<(String, String)> = commands_rows
        .iter()
        .map(|(command, does)| (command.clone(), inline(does)))
        .collect();
    let use_paragraphs_html: Vec<String> = use_section
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Para(text) => Some(inline(text)),
            _ => None,
        })
        .collect();

    let manifest = source.manifest.clone();
    let manifest_file = source
        .readme_path
        .with_file_name(".claude-plugin/plugin.json");
    let skill_slug = manifest
        .homepage
        .split("/skills/")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .filter(|slug| ctx.skills.contains_key(*slug))
        .ok_or_else(|| {
            anyhow!(
                "{}: `homepage` ({}) is not the page of a skill on this site",
                manifest_file.display(),
                manifest.homepage
            )
        })?
        .to_string();
    let (headline, lede) = match manifest.description.split_once(": ") {
        Some((head, rest)) => (format!("{head}."), capitalise(rest)),
        None => {
            return Err(anyhow!(
                "{}: `description` needs the form `Headline: what it does`, which /mods/{}/ splits into its title and lede",
                manifest_file.display(),
                source.slug
            ))
        }
    };
    let source_url = format!(
        "{}/tree/master/mods/{}",
        manifest.repository.trim_end_matches('/'),
        source.slug
    );
    Ok(ModHead {
        slug: source.slug.clone(),
        headline,
        lede,
        install,
        install_notes_html,
        commands: commands_html,
        use_paragraphs_html,
        skill_slug,
        source_url,
        manifest,
    })
}

fn climb_page(
    source: &ModSource,
    readme: &Readme,
    render: &RenderCtx,
    ctx: &PageCtx,
    rows: Vec<ClimbLogRow>,
    run: ClimbRunMeta,
) -> Result<ClimbPage> {
    let file = source.readme_path.display();
    let missing = |what: &str| anyhow!("{file}: {what}, which /mods/{}/ shows", source.slug);
    let inline = |text: &str| markdown::render_inline(text, render);

    let head = mod_head(source, readme, render, ctx)?;
    let use_section = readme
        .section("Use")
        .ok_or_else(|| missing("no `## Use` section"))?;
    let bullets: Vec<&String> = use_section
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::List(items) => Some(items),
            _ => None,
        })
        .flatten()
        .collect();
    let labelled = |label: &str| {
        bullets
            .iter()
            .find(|b| bullet_label(b) == Some(label))
            .map(|b| b.as_str())
            .ok_or_else(|| missing(&format!("no `**{label}**` bullet under `## Use`")))
    };
    let stops_html = STOP_LABELS
        .iter()
        .map(|label| labelled(label).map(inline))
        .collect::<Result<Vec<_>>>()?;
    let protected = labelled("Protected")?;
    let protected_sentences = sentences(protected);
    let limit_at = protected_sentences
        .iter()
        .position(|s| s.contains("shell"))
        .ok_or_else(|| {
            missing(
                "a `**Protected**` bullet that no longer states what a shell command can still do",
            )
        })?;
    let protected_html = inline(&protected_sentences[..limit_at].join(" "));
    let limit_html = inline(&protected_sentences[limit_at..].join(" "));
    let log_html = inline(labelled("Log")?);
    let band_html = bullets
        .iter()
        .find(|b| bullet_label(b).is_none() && b.contains("band"))
        .map(|b| inline(b));

    let limits_html: Vec<String> = readme
        .section("Limits")
        .ok_or_else(|| missing("no `## Limits` section"))?
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Para(text) => Some(inline(text)),
            _ => None,
        })
        .collect();
    if limits_html.is_empty() {
        return Err(missing("an empty `## Limits` section"));
    }
    Ok(ClimbPage {
        head,
        stops_html,
        protected_html,
        limit_html,
        log_html,
        band_html,
        limits_html,
        rows,
        run,
    })
}

/// The README's heading a plain mod page reads what the mod does from.
const BEHAVES_HEADING: &str = "How it behaves";

fn plain_page(
    source: &ModSource,
    readme: &Readme,
    render: &RenderCtx,
    ctx: &PageCtx,
) -> Result<PlainPage> {
    let file = source.readme_path.display();
    let missing = |what: &str| anyhow!("{file}: {what}, which /mods/{}/ shows", source.slug);
    let inline = |text: &str| markdown::render_inline(text, render);

    let head = mod_head(source, readme, render, ctx)?;
    if head.use_paragraphs_html.is_empty() {
        return Err(missing(
            "no paragraph under `## Use` saying who can start one",
        ));
    }
    let section = readme
        .section(BEHAVES_HEADING)
        .ok_or_else(|| missing("no `## How it behaves` section"))?;
    let behaves_html: Vec<String> = section
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::List(items) => Some(items),
            _ => None,
        })
        .flatten()
        .map(|item| inline(item))
        .collect();
    if behaves_html.is_empty() {
        return Err(missing("no bullets under `## How it behaves`"));
    }
    let limits_html: Vec<String> = section
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Para(text) => Some(inline(text)),
            _ => None,
        })
        .collect();
    if limits_html.is_empty() {
        return Err(missing("no closing paragraph under `## How it behaves`"));
    }
    Ok(PlainPage {
        head,
        behaves_heading: section.heading.clone(),
        behaves_html,
        limits_html,
    })
}

fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Shared pieces
// ---------------------------------------------------------------------------

fn breadcrumb(ctx: &PageCtx, trail: &[(&str, Option<String>)]) -> String {
    let mut parts = vec![format!(
        "  <a href=\"{}\">{}</a>",
        ctx.url(""),
        escape_html(chrome::BREADCRUMB_HOME)
    )];
    for (label, href) in trail {
        parts.push("  <span class=\"breadcrumb__sep\" aria-hidden=\"true\">/</span>".to_string());
        parts.push(match href {
            Some(href) => format!("  <a href=\"{href}\">{}</a>", escape_html(label)),
            None => format!(
                "  <span class=\"breadcrumb__current\" aria-current=\"page\">{}</span>",
                escape_html(label)
            ),
        });
    }
    format!(
        "<nav class=\"container breadcrumb\" aria-label=\"{}\">\n{}\n</nav>",
        escape_html(chrome::BREADCRUMB_ARIA_LABEL),
        parts.join("\n")
    )
}

/// A one-line install command with a copy button (atelier-web's install
/// feature copies `data-copy-target`'s text).
fn copy_line(ctx: &PageCtx, id: &str, command: &str, aria: &str) -> String {
    let icons = ctx.asset_url("icons.svg");
    format!(
        r#"<div class="install mod-install">
  <code class="install__command mod-install__command" id="{id}">{command}</code>
  <button class="install__copy" type="button" data-copy-target="{id}" aria-label="{aria}">
    <span class="install__copy-icons">
      <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{icons}#icon-copy"></use></svg>
      <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{icons}#icon-check"></use></svg>
    </span>
    <span data-copy-label>{label}</span>
  </button>
</div>"#,
        command = escape_html(command),
        aria = escape_html(aria),
        label = escape_html(strings::COPY_LABEL),
    )
}

/// `ms` as the climb band prints it: seconds to one decimal.
fn band_seconds(ms: u64) -> String {
    format!("{:.1}s", ms as f64 / 1000.0)
}

/// A score as written in the log (`5`, `12`, `0.5`).
fn score_text(score: f64) -> String {
    if score.fract() == 0.0 && score.abs() < 1e15 {
        format!("{}", score as i64)
    } else {
        format!("{score}")
    }
}

/// The band the mod draws above the prompt after `rows` (register.tsx's
/// AbovePrompt render): `climb n/budget`, or `climb ended: <reason>` once
/// the check passed, then the trail and the last row's numbers. `budget` is
/// what the climb was started with.
fn band_text(rows: &[ClimbLogRow], budget: u32) -> String {
    let Some(last) = rows.last() else {
        return format!("climb 0/{budget} waiting for the first iteration");
    };
    let trail: String = rows
        .iter()
        .map(|r| if r.exit == 0 { '●' } else { '○' })
        .collect();
    let head = if last.exit == 0 {
        format!(
            "climb ended: bar met: the check passed at iteration {}",
            last.iteration
        )
    } else {
        format!("climb {}/{budget}", rows.len())
    };
    let score = last
        .score
        .map(|s| format!("  score {}", score_text(s)))
        .unwrap_or_default();
    format!(
        "{head} {trail}  exit {}{score}  check {}  turn {}",
        last.exit,
        band_seconds(last.check_ms),
        band_seconds(last.turn_ms)
    )
}

// ---------------------------------------------------------------------------
// /mods/
// ---------------------------------------------------------------------------

/// /mods/: what a mod is, then every mod, the first one featured large; the
/// list grows a card at a time as mods arrive.
pub fn index(ctx: &PageCtx, site: &ModsSite) -> String {
    let cards: Vec<String> = site
        .cards
        .iter()
        .enumerate()
        .map(|(i, card)| mod_card(ctx, card, i == 0))
        .collect();
    let facts = [
        strings::MODS_FACT_PLUGIN,
        strings::MODS_FACT_INSTALL,
        strings::MODS_FACT_NO_DEPENDENCY,
    ]
    .iter()
    .map(|fact| {
        format!(
            "      <li class=\"mods-intro__fact\"><span class=\"mods-intro__fact-mark\" aria-hidden=\"true\"></span><span>{}</span></li>",
            escape_html(fact)
        )
    })
    .collect::<Vec<_>>();
    let article = format!(
        r#"<article class="container mods-page">
  <header class="mods-intro">
    <h1 class="mods-intro__title">{title}</h1>
    <p class="mods-intro__lede">{lede}</p>
    <ul class="mods-intro__facts">
{facts}
    </ul>
  </header>
  <section class="mods-list-section" aria-labelledby="mods-list">
    <h2 class="visually-hidden" id="mods-list">{list_heading}</h2>
    <ul class="mods-list">
{cards}
    </ul>
  </section>
</article>"#,
        title = escape_html(strings::MODS_TITLE),
        lede = escape_html(strings::MODS_LEDE),
        facts = facts.join("\n"),
        list_heading = escape_html(strings::MODS_LIST_HEADING),
        cards = join(&cards),
    );
    let main_html = join(&[
        breadcrumb(ctx, &[(strings::BREADCRUMB_MODS, None)]),
        article,
    ]);
    let title = format!("{} — {BRAND_ATELIER_TEXT}", strings::MODS_TITLE);
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: strings::MODS_META,
            body_class: "page-mods",
            path_suffix: MODS_PATH,
            main_html: &main_html,
        },
    )
}

fn mod_card(ctx: &PageCtx, card: &ModCard, featured: bool) -> String {
    let href = ctx.url(&format!("{MODS_PATH}{}/", card.slug));
    let picture = mod_svg(&card.slug);
    let art = format!("<div class=\"mod-card__art\" aria-hidden=\"true\">{picture}</div>");
    format!(
        r#"      <li class="mod-card{featured}">
        {art}
        <div class="mod-card__body">
          <h3 class="mod-card__name">{name}<span class="mod-card__version">v{version}</span></h3>
          <p class="mod-card__description">{description}</p>
        </div>
        <div class="mod-card__get">
          {install}
          <a class="mod-card__cta" href="{href}">{cta}<span class="mod-card__arrow" aria-hidden="true">&rarr;</span></a>
        </div>
      </li>"#,
        featured = if featured { " mod-card--featured" } else { "" },
        name = escape_html(&card.name),
        version = escape_html(&card.version),
        description = card.description_html,
        install = copy_line(
            ctx,
            &format!("mod-install-{}", card.slug),
            &card.install_command,
            &strings::COPY_ARIA_TEMPLATE.replace("{name}", &card.name)
        ),
        cta = escape_html(&strings::MODS_CARD_CTA_TEMPLATE.replace("{name}", &card.name)),
    )
}

// ---------------------------------------------------------------------------
// /mods/climb/
// ---------------------------------------------------------------------------

/// /mods/climb/: the hero with the replay, the loop ring, the commands, what
/// stops it and what it protects, the live log, requirements and limits.
pub fn climb(ctx: &PageCtx, site: &ModsSite) -> String {
    let c = &site.climb;
    mod_document(
        ctx,
        &c.head,
        &[
            mod_hero(ctx, &c.head, &replay(c)),
            climb_how(c),
            mod_commands(&c.head),
            climb_guards(c),
            climb_log(c),
            mod_requirements(&c.head, &c.limits_html),
        ],
    )
}

/// A plain mod's page (`/mods/<slug>/`): the hero, the commands, how it is
/// used, how it behaves, requirements and limits, every fact read from its
/// README and manifest. `None` for a slug that has no plain page.
pub fn plain(ctx: &PageCtx, site: &ModsSite, slug: &str) -> Option<String> {
    let w = site.plain.iter().find(|p| p.head.slug == slug)?;
    Some(mod_document(
        ctx,
        &w.head,
        &[
            mod_hero(ctx, &w.head, &mod_figure(slug)),
            mod_commands(&w.head),
            mod_use(w),
            mod_behaves(w),
            mod_requirements(&w.head, &w.limits_html),
        ],
    ))
}

/// A mod's page: its breadcrumb, then `sections` as the article.
fn mod_document(ctx: &PageCtx, head: &ModHead, sections: &[String]) -> String {
    let m = &head.manifest;
    let page_path = format!("{MODS_PATH}{}/", head.slug);
    let main_html = join(&[
        breadcrumb(
            ctx,
            &[
                (strings::BREADCRUMB_MODS, Some(ctx.url(MODS_PATH))),
                (&m.name, None),
            ],
        ),
        format!(
            "<article class=\"climb-page\">\n{}\n</article>",
            join(sections)
        ),
    ]);
    let title = format!(
        "{} — {} — {BRAND_ATELIER_TEXT}",
        m.name,
        strings::MODS_TITLE
    );
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: &m.description,
            body_class: "page-mod",
            path_suffix: &page_path,
            main_html: &main_html,
        },
    )
}

/// A mod's hero: what it is and how to get it, with `aside` beside it.
fn mod_hero(ctx: &PageCtx, c: &ModHead, aside: &str) -> String {
    let first = &c.install[0];
    let slug = &c.slug;
    let others: Vec<String> = c.install[1..]
        .iter()
        .enumerate()
        .map(|(i, way)| {
            format!(
                "      <li><p>{}</p>{}</li>",
                way.lead_html,
                copy_line(
                    ctx,
                    &format!("{slug}-install-{}", i + 2),
                    &way.command,
                    &copy_aria(way, &c.manifest.name)
                )
            )
        })
        .chain(
            c.install_notes_html
                .iter()
                .map(|note| format!("      <li><p>{note}</p></li>")),
        )
        .collect();
    let others_html = if others.is_empty() {
        String::new()
    } else {
        format!(
            r#"<details class="climb-hero__more">
      <summary>{label}</summary>
      <ul>
{others}
      </ul>
    </details>"#,
            label = escape_html(strings::OTHER_WAYS),
            others = others.join("\n"),
        )
    };
    format!(
        r#"<section class="container climb-hero" aria-labelledby="{slug}-title">
  <div class="climb-hero__copy">
    <p class="climb-hero__name-row"><span class="climb-hero__mark" aria-hidden="true"></span><span class="tag">mod</span><span class="climb-hero__version">v{version}</span></p>
    <h1 class="climb-hero__title" id="{slug}-title"><span class="climb-hero__name">{name}</span><span class="climb-hero__headline">{headline}</span></h1>
    <p class="climb-hero__lede">{lede}</p>
    <p class="climb-hero__where">{first_lead}</p>
    {copy}
    {others_html}
    <p class="climb-hero__links">
      <a class="climb-hero__link climb-hero__link--primary" href="{skill_href}">{route}<span>{read_skill}</span></a>
      <a class="climb-hero__link" href="{source}">{source_label}</a>
    </p>
  </div>
  {aside}
</section>"#,
        version = escape_html(&c.manifest.version),
        name = escape_html(&c.manifest.name),
        headline = escape_html(&c.headline),
        lede = escape_html(&c.lede),
        first_lead = first.lead_html,
        copy = copy_line(
            ctx,
            &format!("{slug}-install-1"),
            &first.command,
            &copy_aria(first, &c.manifest.name)
        ),
        skill_href = ctx.skill_url(&c.skill_slug),
        route = ctx.icon("icon-route", "icon climb-hero__link-icon"),
        read_skill = escape_html(&strings::READ_SKILL_TEMPLATE.replace("{skill}", &c.skill_slug)),
        source = escape_html(&c.source_url),
        source_label = escape_html(strings::SOURCE_LINK),
    )
}

/// A hero copy button's accessible name, so two buttons never share one: it
/// says where its command runs, told by the command itself (a slash command
/// is typed at a terminal session's prompt, anything else in a shell).
fn copy_aria(way: &InstallWay, name: &str) -> String {
    let template = if way.command.starts_with('/') {
        strings::COPY_ARIA_FIRST_TEMPLATE
    } else {
        strings::COPY_ARIA_SHELL_TEMPLATE
    };
    template.replace("{name}", name)
}

/// The replay's script, one line per beat, as markup: who speaks, what, and
/// for the check lines the log row they append and the band they leave.
fn replay(c: &ClimbPage) -> String {
    struct Line {
        who: &'static str,
        text: String,
        meta: String,
        attrs: String,
    }
    let bar = c
        .rows
        .iter()
        .find(|r| r.exit == 0)
        .and_then(|r| r.score)
        .or_else(|| c.rows.iter().filter_map(|r| r.score).reduce(f64::max));
    let plain = |who: &'static str, text: String| Line {
        who,
        text,
        meta: String::new(),
        attrs: String::new(),
    };
    // The mod refuses `/climb protect` before a climb has started
    // (register.tsx), so the protect command follows the start.
    let mut lines = vec![
        plain("you", format!("/climb {} {}", c.run.budget, c.run.check)),
        plain("mod", "starts the climb, queues iteration 1".to_string()),
    ];
    if !c.run.protected.is_empty() {
        let paths = c.run.protected.join(" ");
        lines.push(plain("you", format!("/climb protect {paths}")));
        lines.push(plain("mod", format!("protects {paths}")));
    }
    for (k, row) in c.rows.iter().enumerate() {
        let turn = band_seconds(row.turn_ms);
        lines.push(Line {
            who: "model",
            text: "takes its turn".to_string(),
            meta: format!("turn {turn}"),
            attrs: format!(" data-count-to=\"{}\"", &turn[..turn.len() - 1]),
        });
        let score_html = row
            .score
            .map(|s| format!(" · score: {}", score_text(s)))
            .unwrap_or_default();
        let met = row.exit == 0;
        lines.push(Line {
            who: "check",
            text: format!("exit {}{score_html}", row.exit),
            meta: format!("{} ms", row.check_ms),
            attrs: format!(
                " data-row=\"{k}\" data-band=\"{}\"{}{}",
                escape_html(&band_text(&c.rows[..=k], c.run.budget)),
                row.score
                    .map(|s| format!(" data-score=\"{}\"", score_text(s)))
                    .unwrap_or_default(),
                if met { " data-met" } else { "" }
            ),
        });
        let said = if met {
            format!("logs row {}; bar met, the climb stops", row.iteration)
        } else if k + 1 < c.rows.len() {
            format!(
                "logs row {}, queues iteration {}",
                row.iteration,
                row.iteration + 1
            )
        } else {
            format!("logs row {}", row.iteration)
        };
        lines.push(Line {
            who: "mod",
            text: said,
            meta: String::new(),
            attrs: String::new(),
        });
    }
    let lines_html: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(step, line)| {
            let meta = if line.meta.is_empty() {
                String::new()
            } else {
                format!("<span class=\"climb-replay__meta\">{}</span>", escape_html(&line.meta))
            };
            format!(
                "        <li class=\"climb-replay__line climb-replay__line--{who}\" data-step=\"{step}\"{attrs}><span class=\"climb-replay__who\">{who}</span><span class=\"climb-replay__text\">{text}</span>{meta}</li>",
                who = line.who,
                attrs = line.attrs,
                text = escape_html(&line.text),
            )
        })
        .collect();
    let log_rows: Vec<String> = c
        .rows
        .iter()
        .enumerate()
        .map(|(k, row)| {
            let cells: String = row
                .cells
                .iter()
                .map(|cell| format!("<td>{}</td>", escape_html(cell)))
                .collect();
            format!(
                "          <tr class=\"climb-replay__log-row{}\" data-log-row=\"{k}\">{cells}</tr>",
                if row.exit == 0 { " is-pass" } else { "" }
            )
        })
        .collect();
    let log_head: String = crate::repo::CLIMB_LOG_COLUMNS
        .iter()
        .map(|h| format!("<th scope=\"col\">{h}</th>"))
        .collect();
    let final_score = c.rows.last().and_then(|r| r.score);
    let met = c.rows.last().is_some_and(|r| r.exit == 0);
    let meter = match (bar, final_score) {
        (Some(bar), Some(score)) if bar > 0.0 => format!(
            r#"<div class="climb-replay__meter" data-replay-meter data-bar="{bar_text}" style="--fill:{fill:.3}">
        <span class="climb-replay__meter-label">score <span data-replay-score>{score_text}</span> / {bar_text}</span>
        <span class="climb-replay__meter-track" aria-hidden="true"><span class="climb-replay__meter-fill"></span><span class="climb-replay__meter-bar"></span><span class="climb-replay__sparks">{sparks}</span></span>
      </div>"#,
            bar_text = score_text(bar),
            fill = (score / bar).clamp(0.0, 1.0),
            score_text = score_text(score),
            sparks = (0..6)
                .map(|k| format!("<span class=\"climb-replay__spark\" style=\"--k:{k}\"></span>"))
                .collect::<String>(),
        ),
        _ => String::new(),
    };
    let caption = match &c.band_html {
        Some(band) => format!("{} {}", escape_html(strings::REPLAY_CAPTION), band),
        None => escape_html(strings::REPLAY_CAPTION),
    };
    format!(
        r#"<figure class="climb-replay{met_class}" data-climb-replay aria-label="{label}">
    <div class="climb-replay__window">
      <div class="climb-replay__bar">
        <span class="climb-replay__title">{label}</span>
        <button class="climb-replay__again" type="button" data-replay-again hidden>{icon_again}<span>{again}</span></button>
      </div>
      <ol class="climb-replay__lines">
{lines}
      </ol>
      <p class="climb-replay__band" data-replay-band data-band-start="{band_start}">{band_final}</p>
      <p class="climb-replay__prompt" aria-hidden="true"><span class="climb-replay__caret">›</span><span class="climb-replay__prompt-hint">{prompt_hint}</span></p>
      {meter}
      <div class="climb-replay__log">
        <p class="climb-replay__log-name">{log_file}</p>
        <table class="climb-replay__log-table">
          <thead><tr>{log_head}</tr></thead>
          <tbody>
{log_rows}
          </tbody>
        </table>
      </div>
    </div>
    <figcaption class="climb-replay__caption">{caption}</figcaption>
  </figure>"#,
        met_class = if met { " is-met" } else { "" },
        label = escape_html(strings::REPLAY_LABEL),
        icon_again = "<svg class=\"icon climb-replay__again-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"M4 12a8 8 0 1 0 2.6-5.9\"/><path d=\"M4 4v4.5h4.5\"/></svg>",
        again = escape_html(strings::REPLAY_AGAIN),
        lines = lines_html.join("\n"),
        band_start = escape_html(&band_text(&[], c.run.budget)),
        band_final = escape_html(&band_text(&c.rows, c.run.budget)),
        prompt_hint = escape_html(strings::REPLAY_PROMPT_HINT),
        log_file = escape_html(strings::REPLAY_LOG_FILE),
        log_rows = log_rows.join("\n"),
    )
}

/// The ring's geometry: four stations clockwise from the top, the exit spoke
/// leaving the last one. A 320 x 320 box.
const RING_CX: f64 = 160.0;
const RING_CY: f64 = 160.0;
const RING_R: f64 = 104.0;

fn climb_how(c: &ClimbPage) -> String {
    let stations = [
        (strings::RING_CHANGE_NAME, strings::RING_CHANGE_NOTE),
        (strings::RING_CHECK_NAME, strings::RING_CHECK_NOTE),
        (strings::RING_LOG_NAME, strings::RING_LOG_NOTE),
        (strings::RING_NEXT_NAME, strings::RING_NEXT_NOTE),
    ];
    // Top, right, bottom, left.
    let points = [
        (RING_CX, RING_CY - RING_R),
        (RING_CX + RING_R, RING_CY),
        (RING_CX, RING_CY + RING_R),
        (RING_CX - RING_R, RING_CY),
    ];
    let station_marks: String = points
        .iter()
        .enumerate()
        .map(|(i, (x, y))| {
            format!(
                "<g class=\"loop-ring__station\" data-station=\"{i}\"><circle class=\"loop-ring__station-dot\" cx=\"{x}\" cy=\"{y}\" r=\"13\"/><text class=\"loop-ring__station-num\" x=\"{x}\" y=\"{ty}\">{n}</text></g>",
                n = i + 1,
                ty = y + 4.5,
            )
        })
        .collect();
    let items: Vec<String> = stations
        .iter()
        .enumerate()
        .map(|(i, (name, note))| {
            format!(
                "      <li class=\"loop-ring__step\" data-station=\"{i}\"><span class=\"loop-ring__step-num\" aria-hidden=\"true\">{}</span><span class=\"loop-ring__step-name\">{}</span><span class=\"loop-ring__step-note\">{}</span></li>",
                i + 1,
                escape_html(name),
                escape_html(note)
            )
        })
        .collect();
    let rows_attr: Vec<String> = c
        .rows
        .iter()
        .map(|r| {
            format!(
                "{}:{}:{}",
                r.iteration,
                r.exit,
                r.score.map(score_text).unwrap_or_default()
            )
        })
        .collect();
    let last = c.rows.last();
    let centre = last
        .map(|r| strings::RING_CENTRE_TEMPLATE.replace("{iteration}", &r.iteration.to_string()))
        .unwrap_or_default();
    let centre_detail = last
        .map(|r| {
            let score = r
                .score
                .map(|s| format!(" · score {}", score_text(s)))
                .unwrap_or_default();
            format!("exit {}{score}", r.exit)
        })
        .unwrap_or_default();
    let (lx, ly) = points[3];
    format!(
        r#"<section class="climb-section climb-how" aria-labelledby="climb-how">
  <div class="container climb-how__inner">
    <div class="climb-how__copy">
      <h2 id="climb-how">{heading}</h2>
      <p>{intro}</p>
    </div>
    <div class="loop-ring{met}" data-loop-ring data-loop-rows="{rows}">
      <svg class="loop-ring__art" viewBox="0 0 320 320" role="img" aria-label="{ring_label}">
        <circle class="loop-ring__track" cx="{RING_CX}" cy="{RING_CY}" r="{RING_R}"/>
        <circle class="loop-ring__flow" cx="{RING_CX}" cy="{RING_CY}" r="{RING_R}" pathLength="100"/>
        <path class="loop-ring__exit" d="M{lx} {ly} H14"/>
        <circle class="loop-ring__goal" cx="14" cy="{ly}" r="7"/>
        {station_marks}
        <circle class="loop-ring__traveller" cx="{lx}" cy="{ly}" r="6.5"/>
        <text class="loop-ring__centre" x="{RING_CX}" y="{cy1}" data-loop-centre>{centre}</text>
        <text class="loop-ring__centre-detail" x="{RING_CX}" y="{cy2}" data-loop-detail>{centre_detail}</text>
      </svg>
      <ol class="loop-ring__steps">
{items}
      </ol>
    </div>
  </div>
</section>"#,
        heading = escape_html(strings::HOW_HEADING),
        intro = escape_html(strings::HOW_INTRO),
        met = if last.is_some_and(|r| r.exit == 0) {
            " is-met"
        } else {
            ""
        },
        rows = escape_html(&rows_attr.join(" ")),
        ring_label = escape_html(strings::RING_LABEL),
        cy1 = RING_CY - 4.0,
        cy2 = RING_CY + 18.0,
        items = items.join("\n"),
    )
}

fn mod_commands(c: &ModHead) -> String {
    let rows: Vec<String> = c
        .commands
        .iter()
        .map(|(command, does)| {
            // A README line that is only an example of the flags says nothing
            // more: it takes the row's whole width, not an empty cell.
            if does.is_empty() {
                format!(
                    "        <tr><th scope=\"row\" colspan=\"2\"><code>{}</code></th></tr>",
                    escape_html(command)
                )
            } else {
                format!(
                    "        <tr><th scope=\"row\"><code>{}</code></th><td>{does}</td></tr>",
                    escape_html(command)
                )
            }
        })
        .collect();
    format!(
        r#"<section class="container climb-section climb-commands" aria-labelledby="climb-commands">
  <h2 id="climb-commands">{heading}</h2>
  <div class="climb-table-wrap">
    <table class="climb-table">
      <thead><tr><th scope="col">{command}</th><th scope="col">{does}</th></tr></thead>
      <tbody>
{rows}
      </tbody>
    </table>
  </div>
</section>"#,
        heading = escape_html(strings::COMMANDS_HEADING),
        command = escape_html(strings::COMMANDS_COMMAND),
        does = escape_html(strings::COMMANDS_DOES),
        rows = rows.join("\n"),
    )
}

fn climb_guards(c: &ClimbPage) -> String {
    let stops: String = c
        .stops_html
        .iter()
        .map(|s| format!("\n      <li>{s}</li>"))
        .collect();
    let runs: String = c
        .head
        .use_paragraphs_html
        .iter()
        .map(|p| format!("\n    <p>{p}</p>"))
        .collect();
    format!(
        r#"<section class="container climb-section climb-guards" aria-label="{stops_heading}, {protects_heading}">
  <div class="climb-guards__col">
    <h2>{stops_heading}</h2>
    <ul class="climb-guards__list">{stops}
    </ul>
  </div>
  <div class="climb-guards__col">
    <h2>{protects_heading}</h2>
    <p>{protected}</p>
    <p class="climb-limit"><span class="climb-limit__label">{limit_label}</span><span class="climb-limit__text">{limit}</span></p>{runs}
  </div>
</section>"#,
        stops_heading = escape_html(strings::STOPS_HEADING),
        protects_heading = escape_html(strings::PROTECTS_HEADING),
        protected = c.protected_html,
        limit_label = escape_html(strings::LIMIT_LABEL),
        limit = c.limit_html,
    )
}

fn climb_log(c: &ClimbPage) -> String {
    let head: String = crate::repo::CLIMB_LOG_COLUMNS
        .iter()
        .map(|h| format!("<th scope=\"col\"><code>{h}</code></th>"))
        .collect();
    let rows: Vec<String> = c
        .rows
        .iter()
        .map(|row| {
            let cells: String = row
                .cells
                .iter()
                .map(|cell| format!("<td>{}</td>", escape_html(cell)))
                .collect();
            format!(
                "        <tr{}>{cells}</tr>",
                if row.exit == 0 {
                    " class=\"is-pass\""
                } else {
                    ""
                }
            )
        })
        .collect();
    let caption = log_caption(c);
    format!(
        r#"<section class="container climb-section climb-log" aria-labelledby="climb-log">
  <h2 id="climb-log">{heading}</h2>
  <p class="climb-log__about">{log}</p>
  <figure class="climb-log__figure">
    <div class="climb-table-wrap">
      <table class="climb-table climb-table--log" data-source="{source}">
        <thead><tr>{head}</tr></thead>
        <tbody>
{rows}
        </tbody>
      </table>
    </div>
    <figcaption class="climb-log__caption">{caption}</figcaption>
  </figure>
</section>"#,
        heading = escape_html(strings::LOG_HEADING),
        log = c.log_html,
        source = escape_html(CLIMB_LIVE_LOG_PATH),
        rows = rows.join("\n"),
        caption = escape_html(&caption),
    )
}

/// The log's caption, every fact read: the run's own from the meta file, the
/// first score and the passing iteration from the log.
fn log_caption(c: &ClimbPage) -> String {
    let date = c.run.date.format("%-d %B %Y").to_string();
    let Some(met) = c.rows.iter().find(|r| r.exit == 0) else {
        return strings::LOG_CAPTION_UNMET_TEMPLATE
            .replace("{date}", &date)
            .replace("{rows}", &c.rows.len().to_string());
    };
    let restart = match &c.run.restart {
        Some((restarted, carried)) => strings::LOG_CAPTION_RESTART_TEMPLATE
            .replace("{restarted}", restarted)
            .replace("{carried}", carried)
            .replace(
                "{first_score}",
                &c.rows[0]
                    .score
                    .map(score_text)
                    .unwrap_or_else(|| "no score".into()),
            ),
        None => String::new(),
    };
    strings::LOG_CAPTION_TEMPLATE
        .replace("{date}", &date)
        .replace("{project}", &c.run.project)
        .replace("{checked_by}", &c.run.checked_by)
        .replace("{restart}", &restart)
        .replace("{met}", &met.iteration.to_string())
}

fn mod_requirements(c: &ModHead, limits_html: &[String]) -> String {
    let m = &c.manifest;
    let limits: String = limits_html
        .iter()
        .map(|p| format!("\n    <p>{p}</p>"))
        .collect();
    format!(
        r#"<section class="container climb-section climb-requirements" aria-labelledby="climb-requirements">
  <h2 id="climb-requirements">{heading}</h2>
  <dl class="climb-requirements__facts">
    <div><dt>{runs_in}</dt><dd>{runs_in_value}</dd></div>
    <div><dt>{version}</dt><dd>{version_value}</dd></div>
    <div><dt>{licence}</dt><dd><a href="{licence_url}">{licence_value}</a></dd></div>
    <div><dt>{source}</dt><dd><a href="{source_url}">{source_label}</a></dd></div>
  </dl>
  <div class="climb-requirements__limits">{limits}
  </div>
</section>"#,
        heading = escape_html(strings::REQUIREMENTS_HEADING),
        runs_in = escape_html(strings::REQ_RUNS_IN),
        runs_in_value = escape_html(strings::REQ_RUNS_IN_VALUE),
        version = escape_html(strings::REQ_VERSION),
        version_value = escape_html(&m.version),
        licence = escape_html(strings::REQ_LICENCE),
        licence_value = escape_html(&m.license),
        licence_url = escape_html(&format!(
            "{}/blob/master/LICENSE",
            m.repository.trim_end_matches('/')
        )),
        source = escape_html(strings::REQ_SOURCE),
        source_url = escape_html(&c.source_url),
        source_label = escape_html(
            m.repository
                .trim_end_matches('/')
                .trim_start_matches("https://github.com/"),
        ),
    )
}

// ---------------------------------------------------------------------------
// The plain mod pages and every mod's picture
// ---------------------------------------------------------------------------

/// The picture of the mod `slug`: /mods/ draws it on the card and the page
/// beside its hero.
fn mod_svg(slug: &str) -> String {
    match slug {
        CLIMB_SLUG => super::index::climb_svg(),
        WATCH_SLUG => watch_svg(),
        "orders" => orders_svg(),
        "claim" => claim_svg(),
        "rest" => rest_svg(),
        "meter" => meter_svg(),
        "moves" => moves_svg(),
        _ => plain_svg(),
    }
}

/// How many silent checks the picture draws before the one that wakes.
const WATCH_TICKS: usize = 11;

/// The mod's picture: a row of checks that say nothing, then the one that
/// does.
fn watch_svg() -> String {
    let (y, x0, step) = (60.0, 34.0, 26.0);
    let ticks: String = (0..WATCH_TICKS)
        .map(|i| {
            format!(
                "<circle class=\"watch-art__tick\" style=\"--i:{i}\" cx=\"{}\" cy=\"{y}\" r=\"5\"/>",
                x0 + step * i as f64
            )
        })
        .collect();
    let wake_x = x0 + step * WATCH_TICKS as f64 + 22.0;
    format!(
        "<svg class=\"watch-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\"><line class=\"watch-art__line\" x1=\"{x0}\" y1=\"{y}\" x2=\"{wake_x}\" y2=\"{y}\"/>{ticks}<circle class=\"watch-art__ring\" cx=\"{wake_x}\" cy=\"{y}\" r=\"22\"/><circle class=\"watch-art__wake\" cx=\"{wake_x}\" cy=\"{y}\" r=\"10\"/></svg>"
    )
}

fn mod_figure(slug: &str) -> String {
    format!(
        "<div class=\"mod-figure\" aria-hidden=\"true\">{}</div>",
        mod_svg(slug)
    )
}

/// Orders survive a compaction: the conversation (faint lines) gets shorter
/// across the dashed line, the three standing orders come through whole.
fn orders_svg() -> String {
    let mut s = String::from(
        "<svg class=\"orders-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\"><line class=\"orders-art__cut\" x1=\"200\" y1=\"14\" x2=\"200\" y2=\"106\"/>",
    );
    for i in 0..7 {
        let y = 20 + 8 * i;
        s.push_str(&format!(
            "<line class=\"orders-art__line\" x1=\"36\" y1=\"{y}\" x2=\"164\" y2=\"{y}\"/>"
        ));
    }
    for y in [20, 28] {
        s.push_str(&format!(
            "<line class=\"orders-art__line\" x1=\"236\" y1=\"{y}\" x2=\"364\" y2=\"{y}\"/>"
        ));
    }
    for shift in [0, 200] {
        for (y, to) in [(84, 124), (94, 150), (104, 108)] {
            s.push_str(&format!(
                "<line class=\"orders-art__order\" x1=\"{}\" y1=\"{y}\" x2=\"{}\" y2=\"{y}\"/>",
                36 + shift,
                to + shift
            ));
        }
    }
    s.push_str("</svg>");
    s
}

/// Files that are not this session's (dashed, with a dot) and a sweep that
/// stops short of the first of them.
fn claim_svg() -> String {
    let mut s = String::from(
        "<svg class=\"claim-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\">",
    );
    for i in 0..9 {
        let x = 28 + 38 * i;
        if [3, 5, 6].contains(&i) {
            s.push_str(&format!(
                "<rect class=\"claim-art__other\" x=\"{x}\" y=\"30\" width=\"24\" height=\"32\" rx=\"4\"/><circle class=\"claim-art__dot\" cx=\"{}\" cy=\"46\" r=\"3\"/>",
                x + 12
            ));
        } else {
            s.push_str(&format!(
                "<rect class=\"claim-art__mine\" x=\"{x}\" y=\"30\" width=\"24\" height=\"32\" rx=\"4\"/>"
            ));
        }
    }
    s.push_str("<line class=\"claim-art__sweep\" x1=\"28\" y1=\"88\" x2=\"170\" y2=\"88\"/><line class=\"claim-art__stop\" x1=\"170\" y1=\"78\" x2=\"170\" y2=\"98\"/></svg>");
    s
}

/// Every session pauses at the same moment (the two bars) and goes on from
/// where it stopped.
fn rest_svg() -> String {
    let mut s = String::from(
        "<svg class=\"rest-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\">",
    );
    for y in [30, 50, 70, 90] {
        s.push_str(&format!(
            "<line class=\"rest-art__before\" x1=\"36\" y1=\"{y}\" x2=\"176\" y2=\"{y}\"/><line class=\"rest-art__after\" x1=\"224\" y1=\"{y}\" x2=\"364\" y2=\"{y}\"/><circle class=\"rest-art__resume\" cx=\"224\" cy=\"{y}\" r=\"4\"/>"
        ));
    }
    s.push_str("<rect class=\"rest-art__bar\" x=\"188\" y=\"20\" width=\"8\" height=\"80\" rx=\"3\"/><rect class=\"rest-art__bar\" x=\"204\" y=\"20\" width=\"8\" height=\"80\" rx=\"3\"/></svg>");
    s
}

/// One line of small gauges, one of them past its mark.
fn meter_svg() -> String {
    let mut s = String::from(
        "<svg class=\"meter-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\">",
    );
    for (x, fill) in [(36, 29), (124, 43), (212, 60), (300, 20)] {
        let over = fill > 56;
        s.push_str(&format!(
            "<rect class=\"meter-art__track\" x=\"{x}\" y=\"56\" width=\"70\" height=\"8\" rx=\"4\"/><rect class=\"meter-art__fill{}\" x=\"{x}\" y=\"56\" width=\"{fill}\" height=\"8\" rx=\"4\"/><line class=\"meter-art__tick\" x1=\"{tx}\" y1=\"50\" x2=\"{tx}\" y2=\"70\"/>",
            if over { " meter-art__fill--over" } else { "" },
            tx = x + 56
        ));
        if over {
            s.push_str(&format!(
                "<circle class=\"meter-art__ring\" cx=\"{tx}\" cy=\"32\" r=\"9\"/><circle class=\"meter-art__wake\" cx=\"{tx}\" cy=\"32\" r=\"4\"/>",
                tx = x + 56
            ));
        }
    }
    s.push_str("</svg>");
    s
}

/// Where along the run the picture sets a move aside.
const MOVES_SEEN: [u32; 3] = [76, 136, 196];

/// A run passes three things and sets each on the table below; where the run
/// and the table meet, they come back as options: one taken, two kept.
fn moves_svg() -> String {
    let mut s = String::from(
        "<svg class=\"moves-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\"><path class=\"moves-art__run\" d=\"M36 34H226Q248 34 248 60\"/><path class=\"moves-art__table\" d=\"M40 102H226Q248 102 248 60\"/>",
    );
    for (i, x) in MOVES_SEEN.iter().enumerate() {
        s.push_str(&format!(
            "<line class=\"moves-art__drop\" x1=\"{x}\" y1=\"42\" x2=\"{x}\" y2=\"72\"/><rect class=\"moves-art__card\" x=\"{}\" y=\"76\" width=\"44\" height=\"18\" rx=\"5\"/><circle class=\"moves-art__seen\" style=\"--i:{i}\" cx=\"{x}\" cy=\"34\" r=\"5\"/>",
            x - 22
        ));
    }
    for (i, y) in [22, 51, 80].iter().enumerate() {
        let taken = if i == 0 { " moves-art__option--taken" } else { "" };
        s.push_str(&format!(
            "<line class=\"moves-art__fan\" x1=\"248\" y1=\"60\" x2=\"284\" y2=\"{}\"/><rect class=\"moves-art__option{taken}\" x=\"284\" y=\"{y}\" width=\"80\" height=\"18\" rx=\"9\"/>",
            y + 9
        ));
    }
    s.push_str("<circle class=\"moves-art__close\" cx=\"248\" cy=\"60\" r=\"5\"/></svg>");
    s
}

/// The picture of a mod that has none of its own yet.
fn plain_svg() -> String {
    "<svg class=\"plain-art\" viewBox=\"0 0 400 120\" aria-hidden=\"true\" focusable=\"false\"><rect class=\"plain-art__box\" x=\"140\" y=\"30\" width=\"120\" height=\"60\" rx=\"8\"/><circle class=\"plain-art__dot\" cx=\"200\" cy=\"60\" r=\"5\"/></svg>".to_string()
}

fn mod_use(w: &PlainPage) -> String {
    let heading = if w.head.slug == WATCH_SLUG {
        strings::WATCH_WHO_HEADING
    } else {
        strings::USE_HEADING
    };
    let paragraphs: String = w
        .head
        .use_paragraphs_html
        .iter()
        .map(|p| format!("\n  <p>{p}</p>"))
        .collect();
    format!(
        r#"<section class="container climb-section mod-use" aria-labelledby="mod-use">
  <h2 id="mod-use">{heading}</h2>{paragraphs}
</section>"#,
        heading = escape_html(heading),
    )
}

fn mod_behaves(w: &PlainPage) -> String {
    let items: String = w
        .behaves_html
        .iter()
        .map(|item| format!("\n    <li class=\"mod-behaves__item\">{item}</li>"))
        .collect();
    format!(
        r#"<section class="container climb-section mod-behaves" aria-labelledby="mod-behaves">
  <h2 id="mod-behaves">{heading}</h2>
  <ul class="mod-behaves__list">{items}
  </ul>
</section>"#,
        heading = escape_html(&w.behaves_heading),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const README: &str = "# demo\n\nOne line\nof intro.\n\n## Install\n\nAt the prompt:\n\n    /plugin install demo\n\nIn the app: click.\n\n## Use\n\n    /demo 5 make   start: five;\n                   then more\n    /demo stop     end it\n\n- **Stops** on the bar.\n- **Protected**: edits are refused. A shell command is not inspected.\n  It is a limit.\n- A band shows.\n\nThe check runs with `sh -c`.\n\n## Limits\n\nEarly access.\n";

    #[test]
    fn a_readme_splits_into_paragraphs_code_and_lists() {
        let r = parse_readme(README);
        assert_eq!(r.title, "demo");
        assert_eq!(r.intro, vec![Block::Para("One line of intro.".into())]);
        let install = r.section("Install").unwrap();
        assert_eq!(
            install.blocks,
            vec![
                Block::Para("At the prompt:".into()),
                Block::Code(vec!["/plugin install demo".into()]),
                Block::Para("In the app: click.".into()),
            ]
        );
        let use_ = r.section("Use").unwrap();
        assert_eq!(
            use_.blocks[1],
            Block::List(vec![
                "**Stops** on the bar.".into(),
                "**Protected**: edits are refused. A shell command is not inspected. It is a limit.".into(),
                "A band shows.".into(),
            ])
        );
        assert_eq!(
            use_.blocks[2],
            Block::Para("The check runs with `sh -c`.".into())
        );
    }

    #[test]
    fn the_commands_block_reads_as_rows_with_continuations() {
        let r = parse_readme(README);
        let Block::Code(lines) = &r.section("Use").unwrap().blocks[0] else {
            panic!()
        };
        assert_eq!(
            commands(lines),
            vec![
                (
                    "/demo 5 make".to_string(),
                    "start: five; then more".to_string()
                ),
                ("/demo stop".to_string(), "end it".to_string()),
            ]
        );
    }

    #[test]
    fn the_band_reads_as_the_mod_draws_it() {
        let row = |iteration, exit, score, check_ms, turn_ms| ClimbLogRow {
            cells: vec![],
            iteration,
            exit,
            score: Some(score),
            check_ms,
            turn_ms,
        };
        let rows = [row(1, 1, 5.0, 22, 10271), row(2, 0, 12.0, 15, 10809)];
        assert_eq!(
            band_text(&[], 6),
            "climb 0/6 waiting for the first iteration"
        );
        assert_eq!(
            band_text(&[], 20),
            "climb 0/20 waiting for the first iteration"
        );
        assert_eq!(
            band_text(&rows[..1], 6),
            "climb 1/6 ○  exit 1  score 5  check 0.0s  turn 10.3s"
        );
        assert_eq!(
            band_text(&rows, 6),
            "climb ended: bar met: the check passed at iteration 2 ○●  exit 0  score 12  check 0.0s  turn 10.8s"
        );
    }

    #[test]
    fn sentences_split_after_a_full_stop_and_a_space_only() {
        assert_eq!(
            sentences("Edits to `.climb-log.tsv` are refused. A shell command is not. Done"),
            vec![
                "Edits to `.climb-log.tsv` are refused.",
                "A shell command is not.",
                "Done"
            ]
        );
    }
}
