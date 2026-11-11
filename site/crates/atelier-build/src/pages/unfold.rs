//! The release card at the top of the landing page. It starts small and centred, and
//! atelier-web's `unfold` feature opens it to the whole width of the page without moving
//! anything else: every skill as a bead on a braid, a field that shows what the hook would
//! name, a dial, and what shipped.
//!
//! A release is one [`Release`]. The next one changes [`CURRENT`] and nothing else here;
//! past its `new_until` date the build leaves the card out, and atelier-web's news check
//! retires a page already published.

use super::{PageCtx, escape_html, strings as chrome};

/// What one release says.
pub struct Release {
    /// The card's accessible name.
    pub label: &'static str,
    pub title: &'static str,
    pub lede: &'static str,
    /// The last day it shows, inclusive. A reader's fold is remembered per this date, so the
    /// next release opens again for someone who folded this one.
    pub new_until: &'static str,
    /// The demo types these into the field. Each must name a skill; a test holds it to that.
    pub phrases: &'static [&'static str],
    pub dial: Dial,
    pub ships: &'static [Ship],
}

/// The little demonstration beside the field: silence up to a ceiling, then one word.
pub struct Dial {
    pub command: &'static str,
    pub from: u32,
    pub at: u32,
    /// Below the ceiling, with `{n}` for the percentage.
    pub silent: &'static str,
    /// At the ceiling, with `{n}` for the percentage.
    pub told: &'static str,
}

/// One thing that shipped, and where it leads.
pub struct Ship {
    pub title: &'static str,
    pub note: &'static str,
    pub goes: Goes,
}

/// Where a ship's link leads: always a real page of the built site.
pub enum Goes {
    Skill(&'static str),
    /// A path below the base path.
    Page(&'static str),
    /// The update notes on the landing page.
    Notes,
}

pub const CURRENT: Release = Release {
    label: "New: four mods for long sessions",
    title: "Four new mods for long sessions.",
    lede: "Standing orders that survive a compaction, a guard for a tree with more than one writer, a pause for every session at once, and one quiet line for context, limits and cost.",
    new_until: "2026-11-25",
    phrases: &[
        "The checkout page crashes when the cart is empty",
        "Let's add a dark theme to the settings screen",
        "Fix the merge conflict on this branch",
        "Keep an eye on the build and tell me when it's done",
        "Migrate the users table to the new schema",
    ],
    dial: Dial {
        command: "/meter",
        from: 68,
        at: 80,
        silent: "{n}% of context. One quiet line.",
        told: "{n}%. The band speaks once.",
    },
    ships: &[
        Ship {
            title: "/order",
            note: "New mod. Say it once: a standing order goes back in front of the model after every compaction.",
            goes: Goes::Page("mods/orders/"),
        },
        Ship {
            title: "/claim",
            note: "New mod. Notices files that changed while the session was idle, and refuses a sweeping git command that would take them.",
            goes: Goes::Page("mods/claim/"),
        },
        Ship {
            title: "/rest",
            note: "New mod. Pauses every session on the machine at its next tool call, and resumes each where it stopped.",
            goes: Goes::Page("mods/rest/"),
        },
        Ship {
            title: "/meter",
            note: "New mod. Context, rate limits, cost and a cold cache in one line, each marked measured or estimate.",
            goes: Goes::Page("mods/meter/"),
        },
        Ship {
            title: "Sharper skills",
            note: "Extension tools that skip the permission check, token savers that hide a failure, and messages that only look like a yes.",
            goes: Goes::Notes,
        },
    ],
};

/// The hook's tables, as atelier-build writes them for the card (`check::router::site_table`).
pub const ROUTER_ASSET: &str = "skill-router.json";

const TOGGLE_CLOSED: &str = "Show all of it";
const TOGGLE_OPEN: &str = "Fold it back";
const TRY_LABEL: &str = "Try it. Type what you'd ask Claude.";
/// The longest request the field takes. It stays under the hook's own limit.
const TRY_MAX_LENGTH: u32 = 400;

/// What the page shows when nothing can answer it: everything that still means something. Nothing
/// can open without the module, so it is open; a toggle that does nothing and a field that answers
/// nothing are gone, and so is the lane the braid would have been drawn in. `{p}` scopes the rules:
/// the same ones serve a page with no script and a browser that could not load the WASM.
const STILL_CARD: &str = "{p}.unfold{--open:1}{p}.unfold__more,{p}.unfold__more-inner{visibility:visible;opacity:1;grid-template-rows:1fr}{p}.unfold__toggle,{p}.unfold__try,{p}.unfold__braid-lane{display:none}";

/// Both uses of [`STILL_CARD`]. `:not([data-wasm])` because /graph/ loads a second module, and
/// that one failing leaves this one's controls working.
fn still_card() -> String {
    format!(
        "<noscript><style>{}</style></noscript><style>{}</style>",
        STILL_CARD.replace("{p}", ""),
        STILL_CARD.replace("{p}", "html[data-nowasm]:not([data-wasm]) "),
    )
}

const TOGGLE_ICON: &str = r#"<svg class="unfold__toggle-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M2 8h12M6 4 2 8l4 4M10 4l4 4-4 4"/></svg>"#;

impl Goes {
    fn href(&self, ctx: &PageCtx) -> String {
        match self {
            Goes::Skill(slug) => ctx.skill_url(slug),
            Goes::Page(path) => ctx.url(path),
            Goes::Notes => format!("{}#notes-title", ctx.url("")),
        }
    }
}

/// Every skill as `[slug, family]`, in catalog order so the families read as bands.
fn beads(ctx: &PageCtx) -> Vec<(&'static str, usize)> {
    ctx.categories
        .iter()
        .enumerate()
        .flat_map(|(family, category)| category.skill_slugs.iter().map(move |slug| (*slug, family)))
        .filter(|(slug, _)| ctx.skills.contains_key(*slug))
        .collect()
}

fn json<T: serde::Serialize + ?Sized>(value: &T) -> String {
    escape_html(&serde_json::to_string(value).expect("plain data serializes"))
}

/// The card, or nothing once `release` is past its date.
pub fn card(ctx: &PageCtx, release: &Release) -> String {
    if ctx.last_updated_utc()[..10] > *release.new_until {
        return String::new();
    }
    let ships: String = release
        .ships
        .iter()
        .map(|ship| {
            format!(
                r#"<li><a class="unfold__ship" href="{href}"><b>{title}</b><span>{note}</span></a></li>"#,
                href = escape_html(&ship.goes.href(ctx)),
                title = escape_html(ship.title),
                note = escape_html(ship.note),
            )
        })
        .collect();
    let dial = &release.dial;
    format!(
        r##"<div class="hero__banners">
    <aside class="unfold" aria-label="{label}" data-unfold data-unfold-new-until="{new_until}" data-beads="{beads}" data-phrases="{phrases}" data-href="{href}" data-router="{router}" data-dial-from="{from}" data-dial-at="{at}" data-dial-silent="{silent}" data-dial-told="{told}">
      <div class="unfold__frame" aria-hidden="true"><div class="unfold__glint"></div></div>
      <div class="unfold__window">
        <canvas class="unfold__canvas" aria-hidden="true"></canvas>
        <div class="unfold__lead">
          <p class="unfold__title"><span class="change-badge change-badge--new">{badge}</span><span>{title}</span></p>
          <p class="unfold__lede">{lede}</p>
          <button class="unfold__toggle" type="button" aria-expanded="false" aria-controls="unfold-more"><span class="unfold__toggle-label"><span data-when-closed>{toggle_closed}</span><span data-when-open>{toggle_open}</span></span>{TOGGLE_ICON}</button>
        </div>
        <div class="unfold__braid-lane" aria-hidden="true"></div>
        <div class="unfold__more" id="unfold-more"><div class="unfold__more-inner"><div class="unfold__more-pad">
          <form class="unfold__try">
            <label for="unfold-input">{try_label}</label>
            <input class="unfold__input" id="unfold-input" type="text" maxlength="{TRY_MAX_LENGTH}" spellcheck="false" autocomplete="off">
            <div class="unfold__named" role="status" aria-live="off"></div>
          </form>
          <div class="unfold__watch" style="--used: {at}">
            <svg class="unfold__ring" viewBox="0 0 64 64" aria-hidden="true"><circle class="unfold__ring-track" cx="32" cy="32" r="27"/><circle class="unfold__ring-value" cx="32" cy="32" r="27" pathLength="100"/></svg>
            <div><code>{command}</code><p data-used>{resting}</p></div>
          </div>
          <ul class="unfold__ships">{ships}</ul>
        </div></div></div>
      </div>
    </aside>
    {still}
    </div>"##,
        label = escape_html(release.label),
        new_until = escape_html(release.new_until),
        beads = json(&beads(ctx)),
        phrases = json(release.phrases),
        href = escape_html(&ctx.skill_url("{slug}")),
        router = escape_html(&ctx.asset_url(ROUTER_ASSET)),
        from = dial.from,
        at = dial.at,
        silent = escape_html(dial.silent),
        told = escape_html(dial.told),
        badge = escape_html(chrome::CHANGE_BADGE_NEW),
        title = escape_html(release.title),
        lede = escape_html(release.lede),
        toggle_closed = escape_html(TOGGLE_CLOSED),
        toggle_open = escape_html(TOGGLE_OPEN),
        try_label = escape_html(TRY_LABEL),
        command = escape_html(dial.command),
        resting = escape_html(&dial.told.replace("{n}", &dial.at.to_string())),
        still = still_card(),
    )
}
