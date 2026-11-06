//! The landing page (templates.py `render_index` and everything it calls):
//! the release card, the hero with its install box, the "Fresh from the oven"
//! rail, the update notes, the search field and the category grid.

use std::collections::BTreeSet;

use atelier_core::graph::{BannerSummary, ChangeStamp};
use atelier_core::markdown::{self, RenderCtx};
use atelier_core::updates::UpdateEntry;
use chrono::DateTime;

use super::shell::{self, Page};
use super::{
    BRAND_ATELIER_TEXT, CLIMB_NEW_UNTIL, GETTING_STARTED_PATH, GRAPH_PATH, MODS_PATH, PageCtx,
    SkillEntry, change_badge, escape_html, join, strings as chrome, unfold,
};

/// The landing page's UI strings (templates.py `ENGLISH_STRINGS`), verbatim.
/// `{name}` placeholders are filled with `str::replace`; none of these
/// carries a literal brace.
mod strings {
    pub const HERO_HEADLINE: &str = "An atelier for the way you build.";
    pub const HERO_LEDE_TEMPLATE: &str = "{skill_count} skills for your coding agent, for the craft between the ticket and the commit, and now mods, which change Claude Code itself.";
    pub const COUNTER_LABEL: &str = "Explore the atelier";
    pub const COUNTER_SKILLS_LABEL: &str = "Skills";
    /// Under the count in the Skills preview, and before the name it cycles.
    pub const COUNTER_SKILLS_CAPTION: &str = "skills";
    pub const COUNTER_SKILLS_EXAMPLE: &str = "e.g.";
    pub const COUNTER_SKILLS_NOTE_TEMPLATE: &str =
        "{skill_count} of them, in {category_count} families, all on this page";
    pub const COUNTER_MODS_LABEL: &str = "Mods";
    /// `{name}` is the first mod's manifest name.
    pub const COUNTER_MODS_NOTE_ONE_TEMPLATE: &str =
        "Changes to Claude Code itself. The first is {name}";
    /// `{count}` mods, `{names}` their manifest names.
    pub const COUNTER_MODS_NOTE_MANY_TEMPLATE: &str =
        "Changes to Claude Code itself: {count} so far, {names}";
    pub const COUNTER_GRAPH_LABEL: &str = "Graph";
    pub const COUNTER_GRAPH_NOTE_TEMPLATE: &str =
        "{pair_count} cross-references between {skill_count} skills, traced and drawn";
    pub const COUNTER_START_LABEL: &str = "Getting started";
    pub const COUNTER_START_NOTE: &str = "The first session, in the order you’ll hit it";
    /// The three steps the Getting started preview ticks off: the page's own
    /// first three sections (install, reload, confirm), each said the way a
    /// reader types or does it.
    pub const COUNTER_START_STEPS: [&str; 3] = [
        "Paste the prompt",
        "/reload-plugins",
        "Invoke using-tbaguette",
    ];
    pub const SKILLS_TITLE: &str = "The skills";
    pub const SKILLS_INTRO: &str = "Findable by name, browsable by family, and written like something a colleague actually handed you. Grown out of my own projects over time, the collection is created and updated automatically — and often — by Claude Opus, always its latest version, as I code.";
    pub const FAMILY_INDEX_LABEL: &str = "Families";
    pub const SEARCH_LABEL: &str = "Search skills";
    pub const SEARCH_PLACEHOLDER: &str = "Search by name, summary, or category…";
    pub const SEARCH_CLEAR: &str = "Clear";
    pub const SEARCH_RESET_BUTTON: &str = "Clear search";
    pub const SEARCH_NO_MATCH_PREFIX: &str = "No skills match";
    pub const SEARCH_NO_MATCH_SUFFIX_TEMPLATE: &str = "to see all {skill_count} again.";
    pub const SEARCH_STATUS_SHOWING_ALL_TEMPLATE: &str = "Showing all {count} skills.";
    pub const SEARCH_STATUS_PARTIAL_TEMPLATE: &str = "{shown} of {total} skills match.";
    pub const SENTENCE_END: &str = ".";
    pub const CATEGORY_COUNT_SINGULAR: &str = "skill";
    pub const CATEGORY_COUNT_PLURAL: &str = "skills";
    pub const INSTALL_FRAME_LABEL_TEMPLATE: &str = "Install {brand_atelier}";
    pub const INSTALL_NOTE_VERIFIED_TEXT: &str =
        "Only ever touches this folder — verified against your other skills, not just claimed.";
    pub const INSTALL_NOTE_SEE_HOW: &str = "See how";
    pub const INSTALL_NOTE_NEW_HERE_HTML_TEMPLATE: &str = "New to the {brand_short}? <a href=\"{url}\">Getting started</a> walks through the reload, how skills fire without being asked, and what to do when nothing seems to happen.";
    pub const INSTALL_NOTE_RESTART_HTML_TEMPLATE: &str = "Restart your agent (in Claude Code, <code>/reload-plugins</code>), then invoke a skill as <code>{plugin}:skill-name</code>. On Claude that means Claude Code specifically — the Claude Desktop app and claude.ai chat load skills from your account instead of this folder, so cloning here won’t make them appear there.";
    pub const INSTALL_NOTE_RESTART_LISTED_HTML_TEMPLATE: &str = "Restart your agent (in Claude Code, <code>/reload-plugins</code>), then invoke a skill as <code>{plugin}:skill-name</code>. On Claude that means Claude Code specifically — the Claude Desktop app and claude.ai chat load skills from your account instead of this folder, so add it there from <a href=\"{url}\">Anthropic’s plugin directory</a>.";
    pub const INSTALL_NOTE_SESSION_HTML_TEMPLATE: &str = "A conversation that was already open when you installed or updated — including the one you installed from — is still running on the skill list it started with. Open a new conversation to pick up the latest, or invoke <code>{plugin}:using-tbaguette</code> in the old one.";
    pub const INSTALL_COPY_LABEL: &str = "Copy";
    pub const INSTALL_COPY_ARIA_LABEL: &str = "Copy install command";
    pub const INDEX_TITLE_SUFFIX: &str = "Claude Code skills, organized";
    pub const INDEX_META_DESCRIPTION_TEMPLATE: &str = "{skill_count} Claude Code skills for the craft between the ticket and the commit, organized into {category_count} categories and cross-linked for the moment you need one.";
}

/// The install commands and the agent prompt (templates.py `INSTALL_*`),
/// English-only by design: the prompt is addressed to an agent, and the
/// commands are shell syntax.
pub mod install {
    pub const INSTALL_COMMAND: &str = "[ -d ~/.claude/skills/TBaguette/.git ] && git -C ~/.claude/skills/TBaguette pull || git clone https://github.com/LeSplooch/tbaguette-skills.git ~/.claude/skills/TBaguette";
    pub const INSTALL_COMMAND_POWERSHELL: &str = "if (Test-Path \"$HOME\\.claude\\skills\\TBaguette\\.git\" -PathType Container) { git -C \"$HOME\\.claude\\skills\\TBaguette\" pull } else { git clone https://github.com/LeSplooch/tbaguette-skills.git \"$HOME\\.claude\\skills\\TBaguette\" }";
    pub const INSTALL_COMMAND_CMD: &str = "if exist \"%USERPROFILE%\\.claude\\skills\\TBaguette\\.git\" (git -C \"%USERPROFILE%\\.claude\\skills\\TBaguette\" pull) else (git clone https://github.com/LeSplooch/tbaguette-skills.git \"%USERPROFILE%\\.claude\\skills\\TBaguette\")";
    pub const INSTALL_PROMPT_HINT: &str = "Paste into whichever coding agent you use — it works out which harness it's in and installs the right way for it. It needs a shell tool, which rules out Claude Desktop; on Claude Code it's been exercised against Claude Sonnet (Max).";
    pub const INSTALL_PROMPT: &str = r#"Install (or update) TBaguette’s Atelier — a skills library for coding agents — into whichever agent you are. Use your shell tool.

Before you start: confirm git is available (git --version). If it isn't, tell me and stop — there's nothing else to try.

It is a single git repo, https://github.com/LeSplooch/tbaguette-skills.git, shipping an integration for each harness it supports (Claude Code, Codex, Cursor, Devin, GitHub Copilot CLI, Copilot in VS Code, Copilot coding agent, Gemini CLI, Hermes, Kimi Code, OpenCode, Pi). Work out which one you are running in, then take the matching route — you know your own install mechanism better than this prompt does. If you genuinely can't tell which one you are, say so and ask me, rather than falling through to the first route below.

Route A — you read Claude Code's skills directory. Target: ~/.claude/skills/TBaguette (Windows: %USERPROFILE%\.claude\skills\TBaguette). Figure out which case applies:

1. <target>/.git exists — update in place: git -C <target> pull.
2. <target> doesn't exist, or exists and is empty — install fresh:
   git clone https://github.com/LeSplooch/tbaguette-skills.git <target>.
3. <target> exists, has content, isn't a git repo, but contains CATALOG.md and skills/ — this is very likely a previous TBaguette install that lost its own git history. Say that plainly, not a "naming collision", and ask me whether to move it aside and re-clone, rather than doing that yourself.
4. Anything else already at that path — stop. Do not delete or modify it. Tell me there's a naming collision that needs a manual look.

Route B — you install plugins or extensions from a git URL with your own command. Use it, against this same repo: Hermes is `hermes plugins install LeSplooch/tbaguette-skills --enable`, where the flag is not optional — Hermes only prompts to enable a plugin when it has a terminal to prompt in, so run from a tool it would otherwise install disabled and load nothing. Hermes also security-scans what it installs, and this library trips that scan: a hundred skills about secrets, untrusted input and shell scripting read, to a pattern matcher, like the things they teach you to watch for. The findings are informational. If the scan stops the install because there is no terminal to confirm in, show me the report and the command and let me decide — do not re-run it with --force on my behalf. Kimi Code is `/plugins install https://github.com/LeSplooch/tbaguette-skills` from inside a session or `kimi plugin install https://github.com/LeSplooch/tbaguette-skills.git` from a shell, Gemini CLI is `gemini extensions install https://github.com/LeSplooch/tbaguette-skills`, Codex is `codex plugin marketplace add LeSplooch/tbaguette-skills` and then installing tbaguette-atelier from it. GitHub Copilot CLI takes two steps rather than one, because the install command needs a marketplace to name first: `copilot plugin marketplace add LeSplooch/tbaguette-skills`, then `copilot plugin install tbaguette-atelier@tbaguette-dev`. In VS Code it is not a shell command at all — tell me to run **Chat: Install Plugin From Source** from the Command Palette and paste https://github.com/LeSplooch/tbaguette-skills.git when it asks; don't clone anything yourself in place of that. If it's a command only I can type, print me the exact line instead of substituting a filesystem clone for it.

Route C — you load plugins from a config file. Show me the exact entry and ask before editing — never edit my config silently. On OpenCode that entry is "tbaguette-skills@git+https://github.com/LeSplooch/tbaguette-skills.git", added to the "plugin" array in opencode.json. If you are the Copilot coding agent, the file is .github/copilot/settings.json in the repository you are working on, and both fields are objects rather than arrays: "extraKnownMarketplaces" gets {"tbaguette-dev": {"source": {"source": "github", "repo": "LeSplooch/tbaguette-skills"}}} and "enabledPlugins" gets {"tbaguette-atelier@tbaguette-dev": true}. That edits a repository I share with other people, so show it to me and wait, the same as any other config.

Route D — none of those. Don't invent a path. Find the directory your harness actually reads: its own dotdir in my home directory, or an existing skills/, plugins/, or extensions/ folder — confirmed to be there, not assumed. Clone-or-pull into a TBaguette-named directory inside it, following Route A's four cases. If nothing you find is clearly right, clone nothing: tell me what you checked and ask.

Whichever route: never delete, move, or overwrite anything to make room. If the clone or pull command itself fails (network, permissions, auth), show me the actual error rather than retrying blindly or guessing why.

After a successful install, verify rather than assume. For a clone you placed yourself:
- <target>/.git exists and git -C <target> rev-parse HEAD succeeds.
- <target>/CATALOG.md exists and <target>/skills/ is non-empty.
- Read <target>/.claude-plugin/plugin.json's "version" field, if present, so you can tell me which version I'm now on.
For a harness-managed install, verify it the way your harness reports installed plugins — on Hermes that is `hermes plugins list`, and the row has to say enabled, not just present.

If you are Claude Code: this library is large, and by default your skill listing keeps trigger text for only part of it. Ask me whether you should raise that for me, by adding "skillListingBudgetFraction": 0.04 and "skillListingMaxDescChars": 250 to ~/.claude/settings.json, merged with what is there. Say that it costs a few thousand more tokens in every session. Change it only if I say yes.

Then tell me what happened (installed fresh, updated, or already current), which version, and the reload step for my agent specifically — restarting it, Claude Code's /reload-plugins, Kimi Code's /new, Hermes' `hermes gateway restart`. Skills then invoke as tbaguette-atelier:skill-name.

Flag one more thing, because it applies to this very conversation: this session started before the install, so it is still running on the skill list it had at startup, and nothing you just did changes that. Tell me to open a new conversation to pick up the latest — or, if I want to stay in this one, to invoke tbaguette-atelier:using-tbaguette here (if you have no skill tool, read skills/using-tbaguette/SKILL.md from the install instead)."#;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// The brand and the plugin identifier (templates.py `BRAND_NAME`,
/// `PLUGIN_NAME`, `BRAND_SHORT`).
pub(crate) const BRAND_NAME: &str = "TBaguette";
pub(crate) const PLUGIN_NAME: &str = "tbaguette-atelier";
const BRAND_SHORT: &str = "Atelier";

/// Category markers cycle through these by category index.
const CATEGORY_ICONS: [&str; 3] = ["icon-wheat", "icon-crust", "icon-grain"];

/// Most tiles the "Fresh from the oven" rail shows at once; the badges in
/// the grid stay exhaustive.
const FRESH_RAIL_LIMIT: usize = 12;

/// One tile-step of horizontal travel, matching styles.css's
/// `var(--cf-offset) * 140px` and site.js's DRAG_STEP_PX.
const FRESH_STEP_PX: i64 = 140;

/// Where a "read the whole file" link points.
pub(crate) const GITHUB_BLOB_BASE: &str =
    "https://github.com/LeSplooch/tbaguette-skills/blob/master/";

/// How many update-note entries reach the page (the newest one in-page,
/// all of them in the archive dialog).
const UPDATE_NOTES_LIMIT: usize = 20;
const UPDATE_NOTES_SOURCE_PATH: &str = "UPDATES.md";

/// The listing's address in Anthropic's plugin directory, once there is one.
/// Empty means not listed yet; setting it switches the install note and the
/// getting-started copy together.
pub(crate) const CLAUDE_DIRECTORY_URL: &str = "";

/// The climb the pictures of it show at rest, in a unit box: iteration
/// markers stepping up to the bar, one dip on the way (site.js replays the
/// same run).
const CLIMB_STEPS: [(f64, f64); 6] = [
    (0.13, 0.86),
    (0.30, 0.70),
    (0.47, 0.76),
    (0.64, 0.50),
    (0.80, 0.34),
    (0.93, 0.16),
];

// ---------------------------------------------------------------------------
// The page
// ---------------------------------------------------------------------------

/// The landing page. `notes` is UPDATES.md as parsed, newest first; `graph`
/// is the graph's banner summary; `mods` is each mod's manifest name, in
/// /mods/'s order. The fresh rail comes from `ctx.fresh`.
///
/// Top to bottom: the hero, with the release card across its top, the headline,
/// lede and install box beside the counter strip that leads to Skills, Mods,
/// Graph and Getting started, then the fresh rail and the update notes, then the
/// skills with the family index beside them.
pub fn render(
    ctx: &PageCtx,
    notes: &[UpdateEntry],
    graph: &BannerSummary,
    mods: &[String],
) -> String {
    let skill_count = ctx.skills.len();
    let category_count = ctx.categories.len();
    let main_html = join(&[
        hero(ctx, graph, mods),
        news(ctx, notes),
        skills_section(ctx),
    ]);
    let title = format!("{BRAND_ATELIER_TEXT} — {}", strings::INDEX_TITLE_SUFFIX);
    let meta_description = counts(
        strings::INDEX_META_DESCRIPTION_TEMPLATE,
        skill_count,
        category_count,
    );
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: &meta_description,
            body_class: "page-index",
            path_suffix: "",
            main_html: &main_html,
        },
    )
}

/// Fills `{skill_count}` and `{category_count}`.
fn counts(template: &str, skill_count: usize, category_count: usize) -> String {
    template
        .replace("{skill_count}", &skill_count.to_string())
        .replace("{category_count}", &category_count.to_string())
}

/// The hero: the release card across the top, then two columns, the headline,
/// lede and install box (the page's one primary action) on the left and the
/// counter strip on the right. On a narrow screen the strip follows the install
/// box.
fn hero(ctx: &PageCtx, graph: &BannerSummary, mods: &[String]) -> String {
    let lede = counts(
        strings::HERO_LEDE_TEMPLATE,
        ctx.skills.len(),
        ctx.categories.len(),
    );
    format!(
        r##"<section class="hero">
  <div class="container">
    {card}
    <div class="hero__grid">
      <div class="hero__main">
        <h1 class="hero__headline">{headline}</h1>
        <p class="hero__lede">{lede}</p>
        {install}
      </div>
      {counter}
    </div>
  </div>
</section>"##,
        card = unfold::card(ctx, &unfold::CURRENT),
        headline = escape_html(strings::HERO_HEADLINE),
        install = install_frame(ctx, true),
        lede = escape_html(&lede),
        counter = counter_strip(ctx, graph, mods),
    )
}

/// What changed lately: the fresh rail and the update notes, in a band of
/// their own between the hero and the skills. Nothing when neither has
/// anything to say.
fn news(ctx: &PageCtx, notes: &[UpdateEntry]) -> String {
    let inner = join(&[fresh_section(ctx), update_notes(ctx, notes)]);
    if inner.is_empty() {
        return String::new();
    }
    format!("<div class=\"news\">\n<div class=\"container\">\n{inner}\n</div>\n</div>")
}

/// The skills: a head with the search field, then the family index beside
/// the twelve families. The index is a sticky column at desktop widths and a
/// strip of chips on a phone (landing.css); atelier-web's family index marks
/// the family in view and follows the search.
fn skills_section(ctx: &PageCtx) -> String {
    let sections: Vec<String> = ctx
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| category_section(ctx, category, i))
        .collect();
    format!(
        r#"<section class="skills" id="skills" aria-labelledby="skills-title">
  <div class="container">
    <div class="skills__head">
      <h2 class="skills__title" id="skills-title">{title}</h2>
      <p class="skills__intro">{intro}</p>
      {search}
    </div>
    <div class="skills__layout">
      {index}
      <div class="skills__families" data-categories>
{empty}
{sections}
      </div>
    </div>
  </div>
</section>"#,
        title = escape_html(strings::SKILLS_TITLE),
        intro = escape_html(strings::SKILLS_INTRO),
        search = search_field(ctx),
        index = family_index(ctx),
        empty = search_empty_state(ctx.skills.len()),
        sections = join(&sections),
    )
}

/// The family index: one link per family, to its section, with its skill
/// count and its colour (the graph's family colours, so a family looks the
/// same here, in the counter's constellation and on /graph/).
fn family_index(ctx: &PageCtx) -> String {
    let items: Vec<String> = ctx
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let count = category.skill_slugs.len();
            format!(
                "        <li><a class=\"family-index__link\" href=\"#{slug}\" data-family-link style=\"--family: var(--graph-cat-{n})\"><span class=\"family-index__swatch\" aria-hidden=\"true\"></span><span class=\"family-index__name\">{title}</span><span class=\"family-index__count\" data-family-count=\"{count}\">{count}</span></a></li>",
                slug = escape_html(category.slug),
                n = i + 1,
                title = escape_html(category.title),
            )
        })
        .collect();
    format!(
        r#"<nav class="family-index" aria-labelledby="family-index-title" data-family-index>
        <p class="family-index__title" id="family-index-title">{label}</p>
        <ol class="family-index__list">
{items}
        </ol>
      </nav>"#,
        label = escape_html(strings::FAMILY_INDEX_LABEL),
        items = join(&items),
    )
}

// ---------------------------------------------------------------------------
// The counter strip
// ---------------------------------------------------------------------------

/// The counter: one strip of four previews, each a link to where it shows a
/// glimpse of: Skills (the count, and their names), Mods (the climb), Graph
/// (the families as a constellation) and Getting started (its first three
/// steps). Each figure is finished drawing in the markup, which is what a
/// reader without WASM or with reduced motion sees; atelier-web's counter
/// sets it going while the strip is on screen (landing.css's loops run only
/// under `.is-live`), counts the numeral up and cycles the names.
fn counter_strip(ctx: &PageCtx, graph: &BannerSummary, mods: &[String]) -> String {
    let skill_count = ctx.skills.len();
    let category_count = ctx.categories.len();
    let names = cycling_names(ctx);
    let first_name = names.first().cloned().unwrap_or_default();
    let mods_note = match mods {
        [] => String::new(),
        [one] => strings::COUNTER_MODS_NOTE_ONE_TEMPLATE.replace("{name}", one),
        many => strings::COUNTER_MODS_NOTE_MANY_TEMPLATE
            .replace("{count}", &many.len().to_string())
            .replace("{names}", &many.join(", ")),
    };
    let mods_new = if ctx.climb_is_new() {
        format!(
            "<span class=\"change-badge change-badge--new counter__new\" data-climb-new-until=\"{CLIMB_NEW_UNTIL}\">{}</span>",
            escape_html(chrome::GRAPH_NEW_BADGE)
        )
    } else {
        String::new()
    };
    let graph_note = strings::COUNTER_GRAPH_NOTE_TEMPLATE
        .replace("{pair_count}", &graph.pair_count.to_string())
        .replace("{skill_count}", &graph.skill_count.to_string());
    let steps: String = strings::COUNTER_START_STEPS
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let text = if step.starts_with('/') {
                format!("<code>{}</code>", escape_html(step))
            } else {
                escape_html(step)
            };
            format!(
                "<li class=\"mini-steps__step\" style=\"--i:{i}\"><svg class=\"mini-steps__tick\" viewBox=\"0 0 12 12\" focusable=\"false\"><circle cx=\"6\" cy=\"6\" r=\"5.1\"/><path d=\"M3.5 6.3 5.2 7.9 8.6 4.3\" pathLength=\"10\"/></svg><span class=\"mini-steps__text\">{text}</span></li>"
            )
        })
        .collect();
    let arrow = "<span class=\"counter__arrow\" aria-hidden=\"true\">→</span>";
    let item = |kind: &str, href: &str, figure: &str, label: &str, extra: &str, note: &str| {
        format!(
            r#"      <a class="counter__item counter__item--{kind}" href="{href}">
        <span class="counter__figure" aria-hidden="true">{figure}</span>
        <span class="counter__text"><span class="counter__label">{label}{extra}</span><span class="counter__note">{note}</span></span>
        {arrow}
      </a>"#,
            href = escape_html(href),
            label = escape_html(label),
            note = escape_html(note),
        )
    };
    let skills_figure = format!(
        "<span class=\"mini-count\"><span class=\"mini-count__numeral\" data-counter-count=\"{skill_count}\">{skill_count}</span><span class=\"mini-count__caption\">{caption}</span><span class=\"mini-count__example\"><span class=\"mini-count__eg\">{example}</span> <span class=\"mini-count__name\" data-counter-names=\"{names}\">{first}</span></span></span>",
        caption = escape_html(strings::COUNTER_SKILLS_CAPTION),
        example = escape_html(strings::COUNTER_SKILLS_EXAMPLE),
        names = escape_html(&names.join("|")),
        first = escape_html(&first_name),
    );
    let items = [
        item(
            "skills",
            "#skills",
            &skills_figure,
            strings::COUNTER_SKILLS_LABEL,
            "",
            &counts(
                strings::COUNTER_SKILLS_NOTE_TEMPLATE,
                skill_count,
                category_count,
            ),
        ),
        item(
            "mods",
            &ctx.url(MODS_PATH),
            &mini_climb_svg(),
            strings::COUNTER_MODS_LABEL,
            &mods_new,
            &mods_note,
        ),
        item(
            "graph",
            &ctx.url(GRAPH_PATH),
            &mini_graph_svg(graph),
            strings::COUNTER_GRAPH_LABEL,
            "",
            &graph_note,
        ),
        item(
            "start",
            &ctx.url(GETTING_STARTED_PATH),
            &format!("<ol class=\"mini-steps\">{steps}</ol>"),
            strings::COUNTER_START_LABEL,
            "",
            strings::COUNTER_START_NOTE,
        ),
    ];
    format!(
        "<nav class=\"counter\" aria-label=\"{label}\" data-counter>\n{items}\n    </nav>",
        label = escape_html(strings::COUNTER_LABEL),
        items = items.join("\n"),
    )
}

/// Every skill's name, taken a family at a time in turn (the first of each
/// family, then the second of each...), so the names the counter cycles
/// through change family at every step.
fn cycling_names(ctx: &PageCtx) -> Vec<String> {
    let families: Vec<Vec<&SkillEntry>> = ctx.categories.iter().map(|c| ctx.skills_in(c)).collect();
    let longest = families.iter().map(Vec::len).max().unwrap_or(0);
    (0..longest)
        .flat_map(|i| families.iter().filter_map(move |f| f.get(i)))
        .map(|entry| entry.skill.name.clone())
        .collect()
}

/// A coordinate in the counter's small drawings: one decimal, no trailing
/// `.0` noise beyond what the SVG needs.
fn coord(v: f64) -> String {
    format!("{v:.1}")
}

/// The Mods preview: the climb, small. A dashed bar, the trail
/// stepping up to it with a ring at every iteration, and the solid dot that
/// met it. Over the resting trail lies a second copy (`__run`) that
/// landing.css draws along it, step by step, while the strip is live; the
/// dot flares when it arrives. 160 x 90.
fn mini_climb_svg() -> String {
    let (x0, w, y0, h) = (10.0, 140.0, 6.0, 80.0);
    let pts: Vec<(f64, f64)> = CLIMB_STEPS
        .iter()
        .map(|(x, y)| (x0 + x * w, y0 + y * h))
        .collect();
    let mut d = format!("M{},{}", coord(pts[0].0), coord(pts[0].1));
    for (x, y) in &pts[1..] {
        d.push_str(&format!(" H{} V{}", coord(*x), coord(*y)));
    }
    let (lx, ly) = *pts.last().expect("six steps");
    let rings: String = pts[..pts.len() - 1]
        .iter()
        .map(|(x, y)| {
            format!(
                "<circle class=\"climb-banner__ring\" cx=\"{}\" cy=\"{}\" r=\"3.2\"/>",
                coord(*x),
                coord(*y)
            )
        })
        .collect();
    format!(
        "<svg class=\"mini-climb\" viewBox=\"0 0 160 90\" focusable=\"false\">\
         <line class=\"climb-banner__bar\" x1=\"4\" y1=\"{ly}\" x2=\"156\" y2=\"{ly}\"/>\
         <path class=\"climb-banner__trail\" d=\"{d}\"/>\
         <path class=\"mini-climb__run\" d=\"{d}\" pathLength=\"100\"/>\
         {rings}\
         <circle class=\"climb-banner__halo mini-climb__halo\" cx=\"{lx}\" cy=\"{ly}\" r=\"8\"/>\
         <circle class=\"climb-banner__dot mini-climb__dot\" cx=\"{lx}\" cy=\"{ly}\" r=\"4\"/>\
         </svg>",
        ly = coord(ly),
        lx = coord(lx),
    )
}

/// The Graph preview: the twelve families as a constellation, drawn from the
/// graph's banner summary (atelier-web never loads the graph for it). Each
/// family sits on a ring in catalog order, one dot per skill gathered round
/// its centre; a tie joins two families for the strongest cross-references
/// between them, fainter as they get fewer. landing.css lets each family
/// breathe, out of step with the others. 160 x 90.
fn mini_graph_svg(graph: &BannerSummary) -> String {
    let n = graph.families.len().max(1) as f64;
    let centre = |index: usize| -> (f64, f64) {
        let a = -std::f64::consts::FRAC_PI_2 + index as f64 * std::f64::consts::TAU / n;
        (80.0 + 60.0 * a.cos(), 45.0 + 30.0 * a.sin())
    };
    let mut links: Vec<[usize; 3]> = graph.links.clone();
    links.sort_by(|a, b| b[2].cmp(&a[2]).then(a.cmp(b)));
    links.truncate(MINI_GRAPH_TIES);
    let strongest = links.first().map_or(1, |l| l[2]).max(1) as f64;
    let ties: String = links
        .iter()
        .map(|[a, b, w]| {
            let ((x1, y1), (x2, y2)) = (centre(*a), centre(*b));
            // Bowed toward the middle, so ties read as a web, not spokes.
            let (mx, my) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
            let (qx, qy) = (mx + (80.0 - mx) * 0.45, my + (45.0 - my) * 0.45);
            format!(
                "<path class=\"mini-graph__tie\" d=\"M{} {} Q{} {} {} {}\" style=\"--w:{:.2}\"/>",
                coord(x1),
                coord(y1),
                coord(qx),
                coord(qy),
                coord(x2),
                coord(y2),
                0.25 + 0.75 * (*w as f64 / strongest)
            )
        })
        .collect();
    // Golden-angle spiral: dense, round and even at any count.
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    let families: String = graph
        .families
        .iter()
        .map(|family| {
            let (cx, cy) = centre(family.index);
            let dots: String = (0..family.count)
                .map(|k| {
                    let r = 2.3 * (k as f64 + 0.4).sqrt();
                    let a = k as f64 * golden;
                    format!(
                        "<circle cx=\"{}\" cy=\"{}\" r=\"1.35\"/>",
                        coord(cx + r * a.cos()),
                        coord(cy + r * a.sin())
                    )
                })
                .collect();
            format!(
                "<g class=\"mini-graph__family\" style=\"--i:{i};fill:var(--graph-cat-{n})\">{dots}</g>",
                i = family.index,
                n = family.index + 1,
            )
        })
        .collect();
    format!(
        "<svg class=\"mini-graph\" viewBox=\"0 0 160 90\" focusable=\"false\"><g class=\"mini-graph__ties\">{ties}</g>{families}</svg>"
    )
}

/// How many family-to-family ties the Graph preview draws: the strongest,
/// enough to read as a web, few enough to stay a drawing at 160 x 90.
const MINI_GRAPH_TIES: usize = 22;

// ---------------------------------------------------------------------------
// Install box
// ---------------------------------------------------------------------------

/// The install box (templates.py `_render_install`): the hero leads with it
/// and the getting-started page repeats it. `getting_started_pointer` adds
/// the last note's link to /getting-started/, which that page itself leaves
/// out.
pub(crate) fn install_frame(ctx: &PageCtx, getting_started_pointer: bool) -> String {
    let frame_label =
        strings::INSTALL_FRAME_LABEL_TEMPLATE.replace("{brand_atelier}", BRAND_ATELIER_TEXT);
    let restart_note_html = if CLAUDE_DIRECTORY_URL.is_empty() {
        strings::INSTALL_NOTE_RESTART_HTML_TEMPLATE.replace("{plugin}", PLUGIN_NAME)
    } else {
        strings::INSTALL_NOTE_RESTART_LISTED_HTML_TEMPLATE
            .replace("{plugin}", PLUGIN_NAME)
            .replace("{url}", &escape_html(CLAUDE_DIRECTORY_URL))
    };
    let session_note_html =
        strings::INSTALL_NOTE_SESSION_HTML_TEMPLATE.replace("{plugin}", PLUGIN_NAME);
    let pointer_html = if getting_started_pointer {
        let new_here_html = strings::INSTALL_NOTE_NEW_HERE_HTML_TEMPLATE
            .replace("{brand_short}", BRAND_SHORT)
            .replace("{url}", &ctx.url(GETTING_STARTED_PATH));
        format!(
            r#"<p class="install-frame__note install-frame__note--pointer">
      {grain}
      <span>{new_here_html}</span>
    </p>"#,
            grain = ctx.icon("icon-grain", "icon"),
        )
    } else {
        String::new()
    };
    let icons = ctx.asset_url("icons.svg");
    let check = ctx.icon("icon-check", "icon");
    format!(
        r#"<div class="install-frame">
  <p class="install-frame__label">
    {crust}
    {frame_label}
  </p>
  <div class="install-frame__body">
    <div class="install">
      <code class="install__command" id="install-prompt-command">{prompt}</code>
      <button class="install__copy" type="button" data-copy-target="install-prompt-command"
              aria-label="{copy_aria}">
        <span class="install__copy-icons">
          <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{icons}#icon-copy"></use></svg>
          <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{icons}#icon-check"></use></svg>
        </span>
        <span data-copy-label>{copy_label}</span>
      </button>
    </div>
    <p class="install__hint">{hint}</p>
    <p class="install-frame__note">
      {check}
      <span>{verified} <a href="{verify_url}">{see_how}</a>{sentence_end}</span>
    </p>
    <p class="install-frame__note">
      {check}
      <span>{restart_note_html}</span>
    </p>
    <p class="install-frame__note">
      {check}
      <span>{session_note_html}</span>
    </p>
    {pointer_html}
  </div>
</div>"#,
        crust = ctx.icon("icon-crust", "icon"),
        frame_label = escape_html(&frame_label),
        prompt = escape_html(install::INSTALL_PROMPT),
        copy_aria = escape_html(strings::INSTALL_COPY_ARIA_LABEL),
        copy_label = escape_html(strings::INSTALL_COPY_LABEL),
        hint = escape_html(install::INSTALL_PROMPT_HINT),
        verified = escape_html(strings::INSTALL_NOTE_VERIFIED_TEXT),
        verify_url = ctx.url("verify-install/"),
        see_how = escape_html(strings::INSTALL_NOTE_SEE_HOW),
        sentence_end = escape_html(strings::SENTENCE_END),
    )
}

/// A coordinate as Python prints `round(v, 1)`: one decimal, always shown
/// (`52.0`, `172.0`). Rust's `{:.1}` rounds the exact binary value half to
/// even, which is what Python's `round` does.
fn one_decimal(v: f64) -> String {
    format!("{v:.1}")
}

/// The climb as a static drawing (templates.py `_climb_svg`), in a 400 x 200
/// box: the picture on /mods/'s card for it.
pub(crate) fn climb_svg() -> String {
    let (w, h) = (400.0, 200.0);
    let pts: Vec<(String, String)> = CLIMB_STEPS
        .iter()
        .map(|(x, y)| (one_decimal(x * w), one_decimal(y * h)))
        .collect();
    let mut path = format!("M{},{}", pts[0].0, pts[0].1);
    for (x1, y1) in &pts[1..] {
        path.push_str(&format!(" H{x1} V{y1}"));
    }
    let (lx, ly) = pts.last().expect("six steps");
    let rings: String = pts[..pts.len() - 1]
        .iter()
        .map(|(x, y)| {
            format!("<circle class=\"climb-banner__ring\" cx=\"{x}\" cy=\"{y}\" r=\"6.5\"/>")
        })
        .collect();
    format!(
        "<svg class=\"climb-banner__art\" viewBox=\"0 0 400 200\" preserveAspectRatio=\"xMidYMid meet\" focusable=\"false\">\
         <line class=\"climb-banner__bar\" x1=\"12\" y1=\"{ly}\" x2=\"388\" y2=\"{ly}\"/>\
         <path class=\"climb-banner__trail\" d=\"{path}\"/>\
         {rings}\
         <circle class=\"climb-banner__halo\" cx=\"{lx}\" cy=\"{ly}\" r=\"16\"/>\
         <circle class=\"climb-banner__dot\" cx=\"{lx}\" cy=\"{ly}\" r=\"7.5\"/>\
         </svg>"
    )
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

fn search_field(ctx: &PageCtx) -> String {
    format!(
        r#"<div class="search" data-search-root>
  <div class="search__field">
    <svg class="icon search__icon" aria-hidden="true"><use href="{icons}#icon-search"></use></svg>
    <label class="visually-hidden" for="skill-search">{label}</label>
    <input class="search__input" type="search" id="skill-search" data-search-input
           placeholder="{placeholder}" autocomplete="off">
    <button class="search__clear" type="button" data-search-clear hidden>{clear}</button>
  </div>
  <p class="search__status" data-search-status aria-live="polite"
     data-i18n-showing-all-template="{showing_all}"
     data-i18n-partial-template="{partial}"></p>
</div>"#,
        icons = ctx.asset_url("icons.svg"),
        label = escape_html(strings::SEARCH_LABEL),
        placeholder = escape_html(strings::SEARCH_PLACEHOLDER),
        clear = escape_html(strings::SEARCH_CLEAR),
        showing_all = escape_html(strings::SEARCH_STATUS_SHOWING_ALL_TEMPLATE),
        partial = escape_html(strings::SEARCH_STATUS_PARTIAL_TEMPLATE),
    )
}

fn search_empty_state(skill_count: usize) -> String {
    let suffix =
        strings::SEARCH_NO_MATCH_SUFFIX_TEMPLATE.replace("{skill_count}", &skill_count.to_string());
    format!(
        r#"<p class="search__empty" data-search-empty hidden>
  {prefix} <span class="search__empty-query" data-search-empty-query></span>{sentence_end}
  <button type="button" data-search-reset>{reset}</button> {suffix}
</p>"#,
        prefix = escape_html(strings::SEARCH_NO_MATCH_PREFIX),
        sentence_end = escape_html(strings::SENTENCE_END),
        reset = escape_html(strings::SEARCH_RESET_BUTTON),
        suffix = escape_html(&suffix),
    )
}

// ---------------------------------------------------------------------------
// Fresh from the oven
// ---------------------------------------------------------------------------

/// The fresh skills, newest change first, ties by slug; a stamp that does
/// not parse sorts last (generate.py `_fresh_order`).
fn fresh_order(ctx: &PageCtx) -> Vec<(&SkillEntry, &ChangeStamp)> {
    let mut entries: Vec<(&SkillEntry, &ChangeStamp)> = ctx
        .fresh
        .iter()
        .filter_map(|(slug, stamp)| ctx.skills.get(slug).map(|entry| (entry, stamp)))
        .collect();
    entries.sort_by_cached_key(|(entry, stamp)| {
        let parsed = stamp
            .at
            .as_deref()
            .and_then(|at| DateTime::parse_from_rfc3339(at).ok());
        match parsed {
            Some(at) => (
                0,
                std::cmp::Reverse(at.timestamp_micros()),
                entry.skill.slug.clone(),
            ),
            None => (1, std::cmp::Reverse(0), entry.skill.slug.clone()),
        }
    });
    entries
}

/// A tile's offset from the rail's centre, wrapping the short way round
/// (templates.py `_fresh_signed_offset`; site.js recomputes it).
fn fresh_signed_offset(index: usize, count: usize) -> i64 {
    if count <= 1 {
        return 0;
    }
    let half = count / 2;
    if index <= half {
        index as i64
    } else {
        index as i64 - count as i64
    }
}

/// How far to slide the whole fan so it sits centred in the rail, in px
/// (templates.py `_fresh_group_shift_px`). Python's `round` on a half step
/// rounds to even.
fn fresh_group_shift_px(count: usize) -> i64 {
    if count <= 1 {
        return 0;
    }
    let half = (count / 2) as i64;
    let hi = half.min(2);
    let lo = (-(count as i64 - 1 - half)).max(-2);
    // -(lo + hi) / 2 * 140 is always a multiple of 70, so exact in f64.
    (-((lo + hi) as f64) / 2.0 * FRESH_STEP_PX as f64).round_ties_even() as i64
}

/// The card's body, shared by the grid card and the rail tile.
fn card_inner(entry: &SkillEntry, badge: &str) -> String {
    format!(
        r#"  <span class="card__name-row">
    <span class="card__name">{name}</span>
    {badge}
  </span>
  <p class="card__summary">{summary}</p>
  <span class="card__foot">
    <span class="tag">{category}</span>
    <span class="card__arrow" aria-hidden="true">→</span>
  </span>
</a>"#,
        name = escape_html(&entry.skill.name),
        summary = escape_html(&entry.rendered.summary),
        category = escape_html(entry.category_title),
    )
}

fn fresh_tile(
    ctx: &PageCtx,
    entry: &SkillEntry,
    stamp: &ChangeStamp,
    index: usize,
    count: usize,
) -> String {
    let fresh_at = match stamp.at.as_deref() {
        Some(at) if !at.is_empty() => format!(" data-fresh-at=\"{}\"", escape_html(at)),
        _ => String::new(),
    };
    format!(
        "<a class=\"card fresh__tile\" href=\"{}\"{fresh_at} draggable=\"false\" style=\"--cf-offset: {}\">\n{}",
        ctx.skill_url(&entry.skill.slug),
        fresh_signed_offset(index, count),
        card_inner(entry, &change_badge(Some(stamp))),
    )
}

/// The "Fresh from the oven" rail (templates.py `_render_fresh_section`),
/// or nothing when nothing is fresh.
fn fresh_section(ctx: &PageCtx) -> String {
    let fresh = fresh_order(ctx);
    if fresh.is_empty() {
        return String::new();
    }
    let shown = &fresh[..fresh.len().min(FRESH_RAIL_LIMIT)];
    let count = shown.len();
    let tiles: Vec<String> = shown
        .iter()
        .enumerate()
        .map(|(i, (entry, stamp))| fresh_tile(ctx, entry, stamp, i, count))
        .collect();
    let nav = if count > 1 {
        r#"<div class="fresh__nav" data-fresh-nav hidden>
    <button type="button" class="fresh__nav-btn" data-fresh-prev aria-label="Show previous skill">&lsaquo;</button>
    <button type="button" class="fresh__nav-btn" data-fresh-next aria-label="Show next skill">&rsaquo;</button>
  </div>"#
    } else {
        ""
    };
    format!(
        r#"<section class="fresh" aria-labelledby="fresh-title" data-fresh-section>
  <div class="fresh__head">
    {grain}
    <h2 class="fresh__title" id="fresh-title">Fresh from the oven</h2>
    <span class="tag fresh__tag">Last 48 hours</span>
    {nav}
  </div>
  <div class="fresh__rail" data-fresh-coverflow style="--cf-shift: {shift}px">
{tiles}
  </div>
</section>"#,
        grain = ctx.icon("icon-grain", "icon fresh__icon"),
        shift = fresh_group_shift_px(count),
        tiles = join(&tiles),
    )
}

// ---------------------------------------------------------------------------
// Update notes
// ---------------------------------------------------------------------------

/// English month abbreviations, by hand rather than through a locale.
const MONTH_ABBREVIATIONS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// `2026-08-24` -> `24 Aug 2026`; anything else comes back unchanged
/// (templates.py `_format_update_date`).
fn format_update_date(iso_date: &str) -> String {
    let parts: Vec<&str> = iso_date.split('-').collect();
    let [year, month, day] = parts[..] else {
        return iso_date.to_string();
    };
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if !digits(month) || !digits(day) {
        return iso_date.to_string();
    }
    match (month.parse::<usize>(), day.parse::<u64>()) {
        (Ok(m @ 1..=12), Ok(d)) => format!("{d} {} {year}", MONTH_ABBREVIATIONS[m - 1]),
        _ => iso_date.to_string(),
    }
}

/// One dated entry: its heading (date, then the title with the skills it
/// names linked), then its bullets rendered as inline Markdown.
fn update_entry(entry: &UpdateEntry, render: &RenderCtx) -> String {
    let title = markdown::render_note_title(&entry.title, render);
    let title_html = if title.is_empty() {
        String::new()
    } else {
        format!("\n    <span class=\"notes__entry-title\">{title}</span>")
    };
    let bullets: Vec<String> = entry
        .bullets
        .iter()
        .map(|b| format!("    <li>{}</li>", markdown::render_inline(b, render)))
        .collect();
    format!(
        r#"<li class="notes__entry">
  <h3 class="notes__entry-head">
    <time class="notes__date" datetime="{date}">{shown}</time>{title_html}
  </h3>
  <ul class="notes__bullets">
{bullets}
  </ul>
</li>"#,
        date = escape_html(&entry.date),
        shown = escape_html(&format_update_date(&entry.date)),
        bullets = join(&bullets),
    )
}

/// The "Update notes" band (templates.py `_render_update_notes`): the newest
/// entry in the page, and up to [`UPDATE_NOTES_LIMIT`] in a `<details>` that
/// site.js promotes to a dialog. Nothing when there are no entries.
fn update_notes(ctx: &PageCtx, entries: &[UpdateEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let known: BTreeSet<String> = ctx.skills.keys().cloned().collect();
    // Only real skills link: UPDATES.md's code spans are mostly file names
    // and paths. No page owns these notes, so none is excluded as "self";
    // relative links have nothing to resolve to and keep only their text.
    let render = RenderCtx {
        known_slugs: &known,
        base_path: &ctx.base_path,
        link_slugs: true,
        current_slug: None,
        relative_anchors: None,
        heading_id_prefix: "",
    };
    let shown = &entries[..entries.len().min(UPDATE_NOTES_LIMIT)];
    let archive_html = if shown.len() > 1 {
        let all: Vec<String> = shown.iter().map(|e| update_entry(e, &render)).collect();
        format!(
            r#"
  <details class="notes__earlier notes__archive" data-notes-archive
           data-archive-title="Update notes"
           data-archive-subtitle="The {n} most recent entries"
           data-archive-close="Close update notes">
    <summary class="notes__earlier-summary">
      <span class="notes__earlier-label">{label}</span>
    </summary>
    <div class="notes__archive-panel" data-notes-archive-panel>
      <ol class="notes__list notes__list--earlier notes__list--archive">
{all}
      </ol>
      <p class="notes__archive-foot">
        Older than these, the whole record lives in
        <a href="{archive_url}" rel="noopener">{source}</a>.
      </p>
    </div>
  </details>"#,
            n = shown.len(),
            label = escape_html(&format!("Read all {} updates", shown.len())),
            all = join(&all),
            archive_url = escape_html(&format!("{GITHUB_BLOB_BASE}{UPDATE_NOTES_SOURCE_PATH}")),
            source = escape_html(UPDATE_NOTES_SOURCE_PATH),
        )
    } else {
        String::new()
    };
    format!(
        r#"<section class="notes" aria-labelledby="notes-title" data-update-notes>
  <div class="notes__head">
    {wheat}
    <h2 class="notes__title" id="notes-title">Update notes</h2>
  </div>
  <ol class="notes__list">
{newest}
  </ol>{archive_html}
</section>"#,
        wheat = ctx.icon("icon-wheat", "icon notes__icon"),
        newest = update_entry(&shown[0], &render),
    )
}

// ---------------------------------------------------------------------------
// The category grid
// ---------------------------------------------------------------------------

/// What site.js's filter matches a card against: name, summary and category,
/// lowercased (templates.py `_search_haystack`).
fn search_haystack(entry: &SkillEntry) -> String {
    escape_html(
        &format!(
            "{} {} {}",
            entry.skill.name, entry.rendered.summary, entry.category_title
        )
        .to_lowercase(),
    )
}

fn skill_card(ctx: &PageCtx, entry: &SkillEntry) -> String {
    format!(
        "<a class=\"card\" href=\"{}\" data-search-card data-search-terms=\"{}\">\n{}",
        ctx.skill_url(&entry.skill.slug),
        search_haystack(entry),
        card_inner(entry, &change_badge(ctx.fresh.get(&entry.skill.slug))),
    )
}

fn category_section(
    ctx: &PageCtx,
    category: &atelier_core::model::Category,
    index: usize,
) -> String {
    // The count is what the category lists, as in the Python.
    let count = category.skill_slugs.len();
    let cards: Vec<String> = ctx
        .skills_in(category)
        .into_iter()
        .map(|entry| skill_card(ctx, entry))
        .collect();
    let noun = if count == 1 {
        strings::CATEGORY_COUNT_SINGULAR
    } else {
        strings::CATEGORY_COUNT_PLURAL
    };
    format!(
        r#"<section class="category-section" id="{slug}" data-category-section style="--family: var(--graph-cat-{n})">
    <div class="category-section__head">
      {icon}
      <h3 class="category-section__title">{title}</h3>
      <span class="tag category-section__count" data-category-count="{count}"
            data-i18n-singular="{singular}"
            data-i18n-plural="{plural}">{count} {noun}</span>
    </div>
    <div class="card-grid" data-card-grid>
{cards}
    </div>
</section>"#,
        n = index + 1,
        slug = escape_html(category.slug),
        icon = ctx.icon(
            CATEGORY_ICONS[index % CATEGORY_ICONS.len()],
            "icon category-section__icon"
        ),
        title = escape_html(category.title),
        singular = escape_html(strings::CATEGORY_COUNT_SINGULAR),
        plural = escape_html(strings::CATEGORY_COUNT_PLURAL),
        noun = escape_html(noun),
        cards = join(&cards),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values are templates.py's own output for the same inputs.

    #[test]
    fn the_fan_is_centred_by_its_visible_range() {
        let shifts: Vec<i64> = (0..13).map(fresh_group_shift_px).collect();
        assert_eq!(shifts, [0, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0]);
        let offsets =
            |count| -> Vec<i64> { (0..count).map(|i| fresh_signed_offset(i, count)).collect() };
        assert_eq!(offsets(1), [0]);
        assert_eq!(offsets(2), [0, 1]);
        assert_eq!(offsets(4), [0, 1, 2, -1]);
        assert_eq!(offsets(5), [0, 1, 2, -2, -1]);
    }

    #[test]
    fn note_dates_read_day_month_year_and_odd_shapes_pass_through() {
        let shown: Vec<String> = [
            "2026-08-24",
            "2026-01-05",
            "2026-13-01",
            "2026-8",
            "x-y-z",
            "2026-12-31",
        ]
        .into_iter()
        .map(format_update_date)
        .collect();
        assert_eq!(
            shown,
            [
                "24 Aug 2026",
                "5 Jan 2026",
                "2026-13-01",
                "2026-8",
                "x-y-z",
                "31 Dec 2026"
            ]
        );
    }

    #[test]
    fn the_climb_drawing_prints_coordinates_as_python_floats() {
        let svg = climb_svg();
        assert!(svg.contains(
            "d=\"M52.0,172.0 H120.0 V140.0 H188.0 V152.0 H256.0 V100.0 H320.0 V68.0 H372.0 V32.0\""
        ));
        assert!(svg.contains(
            "<line class=\"climb-banner__bar\" x1=\"12\" y1=\"32.0\" x2=\"388\" y2=\"32.0\"/>"
        ));
    }
}
