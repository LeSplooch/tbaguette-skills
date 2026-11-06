//! The site's other pages: /getting-started/, /verify-install/ and /graph/
//! (templates.py `render_getting_started_page`, `render_verify_install_page`
//! and `render_graph_page`).

use atelier_core::graph::BannerSummary;

use super::index::{
    BRAND_NAME, CLAUDE_DIRECTORY_URL, GITHUB_BLOB_BASE, PLUGIN_NAME, install, install_frame,
};
use super::shell::{self, Page};
use super::{
    BRAND_ATELIER, BRAND_ATELIER_TEXT, GETTING_STARTED_PATH, GRAPH_PATH, PageCtx, contents_rail,
    escape_html, join, strings as chrome,
};

/// The graph page's strings (templates.py `Strings`).
pub(crate) mod graph_strings {
    pub const GRAPH_PAGE_TITLE: &str = "The crumb";
    pub const GRAPH_PAGE_LEDE_TEMPLATE: &str = "How {skill_count} skills lean on each other: {pair_count} cross-references, traced section by section.";
    pub const GRAPH_NOSCRIPT: &str = "The graph is drawn in the browser and needs JavaScript. Every cross-reference it shows is also a link on the skill pages themselves.";
}

/// /verify-install/'s prose (templates.py `ENGLISH_VERIFY_INSTALL_STRINGS`),
/// verbatim. `*_HTML` values are HTML and go in unescaped.
mod verify_strings {
    pub const TITLE: &str = "The install command only touches one folder";
    pub const LEDE_HTML: &str = "A <code>git clone</code> into <code>~/.claude/skills/TBaguette</code> sits right next to whatever else already lives under <code>~/.claude/skills/</code> — other skills, other plugins, things you already trust. It's a fair question whether installing this one could touch any of that. It can't. This page is why, and the actual test that proves it, not a claim restated in different words.";
    pub const WHAT_IT_DOES_HEADING: &str = "What the command does";
    pub const WHAT_IT_DOES_HTML: &str = "The published command checks for an existing clone before doing anything: if <code>~/.claude/skills/TBaguette/.git</code> already exists, it runs <code>git pull</code> in place; otherwise it runs <code>git clone</code>. The first run installs. Every run after that updates. Neither branch ever reads or writes anywhere else.";
    pub const WHY_HEADING: &str = "Why it can't reach anything else";
    pub const WHY_HTML: &str = "Both branches are scoped to the single path <code>~/.claude/skills/TBaguette</code> by construction — a <code>git clone</code> or <code>git pull</code> targeting that path has no mechanism to write outside it. The one case worth naming explicitly is a real collision: something else already sitting at that exact path, that isn't a clone of this repo. <code>git clone</code> refuses outright when its target already exists and is non-empty — it does not merge, does not overwrite, does not ask. It fails loudly and leaves whatever was there exactly as it was. That refusal is git's own behavior, not something this project added on top.";
    pub const FOUR_SCENARIOS_HEADING: &str = "Four scenarios, not one";
    pub const FOUR_SCENARIOS_INTRO_HTML_TEMPLATE: &str = "A single happy-path test would only prove the command works when nothing is in its way, which is the one case nobody actually worried about. The real test runs four scenarios, each against a throwaway <code>HOME</code> directory seeded with fake sibling skills and plugins alongside <code>{brand}</code>, checksummed before and after:";
    pub const SCENARIO_A_HTML: &str = "<strong>A — fresh install.</strong> Nothing at that path yet. Confirms the clone succeeds and the siblings are untouched.";
    pub const SCENARIO_B_HTML: &str = "<strong>B — running it again.</strong> Continues from A. Confirms the second run updates in place (<code>git pull</code>) instead of erroring the way a bare <code>git clone</code> would, and that the siblings are still untouched.";
    pub const SCENARIO_C_HTML_TEMPLATE: &str = "<strong>C — an empty directory already named <code>{brand}</code>.</strong> Confirms <code>git clone</code> is willing to use an empty directory that happens to already exist, and that siblings survive.";
    pub const SCENARIO_D_HTML: &str = "<strong>D — the real collision.</strong> A non-empty, non-git directory already sitting at that exact path, with its own unrelated content. Confirms the command refuses rather than merging into it — and that both the colliding directory's own content and every sibling skill survive the refusal.";
    pub const SCENARIO_FOOTER_HTML: &str = "All twelve checks across those four scenarios pass before this site is ever deployed — plus twelve more that run the exact command string published above, word for word, through every POSIX-ish shell this project's own build machine has installed (bash, zsh, fish, and sh), rather than trusting that this page's Python reimplementation of it stayed faithful to what actually ships. <code>cargo test</code> runs the same scenarios alongside everything else, not as a separate manual step someone has to remember.";
    pub const EVERY_PLATFORM_HEADING: &str = "Every platform, not just one";
    pub const EVERY_PLATFORM_INTRO_HTML: &str = "The install frame above shows a prompt addressed to Claude, not a command for you to type — paste it into a Claude Code conversation and Claude detects your OS and runs the matching command itself. That command, on every platform, is exactly what follows: a POSIX command for macOS, Linux, and anyone on Windows already inside WSL or Git Bash, and a PowerShell command for everyone else on Windows — which is most Windows users, since PowerShell has been the default terminal since Windows 10. All three do the exact same thing, in the same order, for the same reason: check for an existing clone, <code>git pull</code> if it's there, <code>git clone</code> if it isn't.";
    pub const POSIX_INTRO: &str =
        "The POSIX command is the one machine-verified above, across four shells:";
    pub const POWERSHELL_INTRO_HTML: &str = "The PowerShell command carries the exact same logic over to <code>Test-Path</code> and native cmdlets:";
    pub const POWERSHELL_CAVEAT_HTML: &str = "That one isn't cross-checked by an automated test the way the POSIX command is — this project's own build has no PowerShell runtime to run it against — so its correctness rests on <code>Test-Path</code>, <code>git -C</code>, and <code>git clone</code> being ordinary, well-documented behavior, not on an executed proof. If that's not good enough, that's a fair position to hold; the honest answer is \"verified by careful construction,\" not \"verified,\" for this one specifically.";
    pub const CMD_INTRO: &str = "And for Command Prompt, which nothing in the install frame targets but which still exists on every Windows machine:";
    pub const CMD_CAVEAT_HTML: &str = "Same logic again, translated to <code>IF EXIST</code> and <code>%USERPROFILE%</code> — with one honest imprecision. <code>IF EXIST</code> can't cleanly test \"is this specifically a directory\" in one line the way <code>-d</code> and <code>-PathType Container</code> can, so it matches a file or a directory at that path indiscriminately. That's never actually wrong here, since a real git checkout's <code>.git</code> is always a directory, but it's a looser guarantee than the other two, worth naming rather than glossing over.";
    pub const TEST_ITSELF_HEADING: &str = "The test itself";
    pub const TEST_ITSELF_TEMPLATE: &str = "This is <a href=\"{test_github_url}\">{test_source_path}</a>, unedited — the same file the test suite actually runs, not a representative excerpt.";
    pub const RUN_IT_YOURSELF_HEADING: &str = "Run it yourself";
    pub const RUN_IT_YOURSELF_HTML: &str = "Clone the repo and run <code>python3 verify/test_install_command.py</code> directly, or <code>cargo test --manifest-path site/Cargo.toml --workspace</code> for the full suite this page's claims are checked against. Standard-library Python only — no bash required, since it isn't guaranteed to exist on every system this might run on.";
    pub const BREADCRUMB_CURRENT: &str = "Install verification";
    pub const META_DESCRIPTION_TEMPLATE: &str = "How the install command for {brand_atelier} is verified never to alter, overwrite, or merge into any other skill or plugin you already have — with the actual test source, not just a claim.";
}

/// /getting-started/'s prose (templates.py
/// `ENGLISH_GETTING_STARTED_STRINGS`), verbatim. `*_HTML` values are HTML and
/// go in unescaped.
mod getting_started_strings {
    pub const TITLE: &str = "How to actually use these skills";
    pub const LEDE_HTML: &str = "Installing is one command, and the rest of this site already covers it. The part nobody mentions is what happens after: most skills fire without you asking for them, the ones that don’t you call by name, and a job bigger than one edit wants a different opening move than a small one. This is the first session, in the order you’ll actually hit it.";
    pub const INSTALL_HEADING: &str = "Install it, if you haven’t";
    pub const INSTALL_HTML_TEMPLATE: &str = "Here it is — the same box <a href=\"{home_url}\">the landing page</a> leads with, repeated so this page does not send you elsewhere for the one thing you have to paste. It carries a prompt rather than a command: put it into whichever coding agent you use, and it works out which harness it’s in and takes that harness’s route. On Claude Code that route is a clone-or-pull into <code>~/.claude/skills/TBaguette</code>, and nothing else on your machine is touched. <a href=\"{verify_url}\">Install verification</a> is the page that proves that claim, with the actual test source rather than the claim restated in different words.";
    pub const RELOAD_HEADING: &str =
        "Reload — and know that the conversation you installed from won’t";
    pub const RELOAD_HTML: &str = "A running agent holds the skill list it started with. Restart it — in Claude Code, <code>/reload-plugins</code> does it without a restart — and every new conversation picks the library up.";
    pub const RELOAD_SESSION_HTML_TEMPLATE: &str = "The conversation you ran the install from is the exception worth naming, because it’s the one you’re most likely to test in. It started before the install, so it still has the old list, and nothing you just did changes that. Open a new conversation, or invoke <code>{plugin}:using-tbaguette</code> in the old one to load the entry point by hand.";
    pub const RELOAD_DESKTOP_HTML: &str = "One more place this bites, on Claude specifically: this is Claude <em>Code</em>. The Claude Desktop app, claude.ai and Cowork don’t read <code>~/.claude/skills/</code> at all — they install plugins on your account, from Anthropic’s plugin directory — so cloning here will never make these show up there. The Atelier isn’t listed in the directory yet; until it is, those apps also take a marketplace you add yourself from GitHub, and this repository is one: <code>LeSplooch/tbaguette-skills</code>.";
    pub const RELOAD_DESKTOP_LISTED_HTML_TEMPLATE: &str = "One more place this bites, on Claude specifically: this is Claude <em>Code</em>. The Claude Desktop app, claude.ai and Cowork don’t read <code>~/.claude/skills/</code> at all — they install plugins on your account, from Anthropic’s plugin directory. Add the Atelier there instead: open <strong>Customize › Plugins › Discover</strong> in the Desktop app or on claude.ai, or go straight to <a href=\"{url}\">its listing</a>. A plugin added on your account also turns up in Claude Code at its next session.";
    pub const CONFIRM_HEADING: &str = "Confirm it landed";
    pub const CONFIRM_HTML_TEMPLATE: &str = "Ask your agent to invoke <code>{plugin}:using-tbaguette</code>. If the install worked, it comes back with a short notice about checking skills before every response — that notice is the library’s entry point, and seeing it is proof the whole thing is loaded and callable, not just present on disk. If you get “no such skill” instead, the clone isn’t where your agent reads from, and the install prompt’s own verification steps will say which case you’re in.";
    pub const HOW_THEY_FIRE_HEADING: &str = "Most of the time you won’t invoke anything";
    pub const HOW_THEY_FIRE_HTML_TEMPLATE: &str = "Every skill carries a trigger description — the situations it is for, written out at length rather than summarized into a category. Your agent matches what you are doing against those, and <code>{plugin}:using-tbaguette</code> is the piece that makes that check habitual rather than occasional.";
    pub const HOW_THEY_FIRE_EXAMPLES_HTML_TEMPLATE: &str = "So the ordinary experience is that you never type a skill name. You say <em>fix this bug</em>, and <a class=\"skill-link\" href=\"{diagnosing_url}\"><code>diagnosing-before-fixing</code></a> arrives before a patch does — a reproduction first, then the cause, then the fix. You ask for an interface, and <a class=\"skill-link\" href=\"{formidable_url}\"><code>formidable</code></a> turns up with the questions a designer would have asked. You do not have to memorize {skill_count} names, and you should not try.";
    pub const BY_NAME_HEADING: &str = "When you do want one by name";
    pub const BY_NAME_HTML_TEMPLATE: &str = "Invoke it directly — <code>{plugin}:formidable</code>, <code>{plugin}:knowing-when-to-stop</code>, whichever one you mean. There are two honest reasons to override the automatic path. The first is that you know your own situation better than a trigger description can: the work does not look like the skill’s trigger, but the discipline still applies. The second is timing — you want it before your agent has decided this task is simple enough to skip it, which is reliably the moment it is not.";
    pub const STARTERS_HEADING: &str = "Six worth trying on purpose";
    pub const STARTERS_INTRO: &str = "Not a ranking, and not the six that matter most — the six whose effect you can watch inside a single session, which makes them the fastest way to find out whether any of this is for you.";
    pub const STARTER_FORMIDABLE_HTML: &str = "The flagship, and the one with the most visible before-and-after. Design craft for every stack there is — web, terminal, native, game HUD, print, voice — so it applies to the interface you actually have rather than the one a web tutorial assumed you had. Ask it for nothing in particular and it goes looking for the best shipped work in your domain first, then builds past what you asked for.";
    pub const STARTER_DIAGNOSING_BEFORE_FIXING_HTML: &str = "Turns <em>fix this</em> into a reproduction before a patch. The fastest way to see the difference: hand it a bug whose obvious fix you already have in mind, and watch where it goes first.";
    pub const STARTER_WRITING_THE_FAILING_TEST_FIRST_HTML: &str = "The most concrete behavior change on this list. One test that fails for the right reason before any production code exists — a different thing from a test written afterwards that has never once been red.";
    pub const STARTER_CONFIRMING_BEFORE_CLAIMING_DONE_HTML: &str = "For anyone who has been told something was finished and found out otherwise. It replaces <em>this should work</em> with a command that was actually run, and treats a tool’s own success report as a claim rather than as evidence.";
    pub const STARTER_ORCHESTRATING_WORK_END_TO_END_HTML: &str = "The entry point for anything bigger than a single edit — which is the next section, because it is the one piece of this that changes how a whole run is shaped rather than how one response is.";
    pub const STARTER_KNOWING_WHEN_TO_STOP_HTML: &str = "The counterweight to everything above. Bounded passes, diminishing returns, and naming what is being left undone instead of quietly polishing until the context runs out.";
    pub const BIGGER_WORK_HEADING: &str = "Anything bigger than one edit starts somewhere else";
    pub const BIGGER_WORK_HTML_TEMPLATE: &str = "One skill covers one stretch of work. A feature, a bug with no known cause, a migration, an audit — those take several of them, in an order, and that order is its own skill: <a class=\"skill-link\" href=\"{orchestrating_url}\"><code>orchestrating-work-end-to-end</code></a>. It routes the request to the lightest track that fits, says which phase the work is in, and names the evidence that has to exist before the next phase is allowed to open.";
    pub const BIGGER_WORK_PRACTICAL_HTML: &str = "The practical version, for you: describe what you want, and let it open with scoping rather than with an editor. What comes back first is a design to say yes or no to — which is a great deal cheaper to argue with than a finished implementation of the wrong thing.";
    pub const UPDATING_HEADING: &str = "Keeping it current";
    pub const UPDATING_HTML: &str = "Run the same install prompt again — it takes whichever route it took the first time. On the Claude Code route that route is a clone-or-pull, so a second run updates in place rather than erroring, duplicating anything, or touching whatever else lives beside it. On a harness with its own plugin command, the prompt hands back to that command, and updating is whatever that harness already does.";
    pub const UPDATING_AUTOMATIC_HTML_TEMPLATE: &str = "Mostly you will not have to remember. <a class=\"skill-link\" href=\"{keeping_current_url}\"><code>keeping-tbaguette-current</code></a> runs that check at the start of a conversation and reports what changed in readable terms rather than as a commit log. This site’s own header carries the published version and the moment it was last built, if you would rather compare by hand.";
    pub const COEXISTENCE_HEADING: &str = "Living beside your other skills";
    pub const COEXISTENCE_INSTALL_HTML_TEMPLATE: &str = "Installing {brand} adds exactly one directory, <code>~/.claude/skills/TBaguette</code>, and touches nothing else — not another library’s skills, not a plugin you already run, not your settings. If that one path is already occupied by something unrelated, the install refuses outright rather than merging into it, so a name collision fails loudly instead of quietly costing you whatever was there. <a href=\"{verify_url}\">Install verification</a> is where that is proved against the real command rather than asserted.";
    pub const COEXISTENCE_NAMES_HTML_TEMPLATE: &str = "Skill <em>names</em> can repeat across libraries, and that is fine, because {brand}’s are called with its prefix. <code>tbaguette-atelier:naming-things</code> is always this library’s; the same name without the prefix is whoever else’s. Two skills that share a name stay separately reachable and neither one shadows the other — so nothing has to be uninstalled to make room.";
    pub const COEXISTENCE_NOTICES_HTML_TEMPLATE: &str = "If you also run a library that injects its own check-the-skills-first notice — Superpowers is the one you are most likely to have — both fire, and both apply. They are not competing for the same job: <a class=\"skill-link\" href=\"{using_url}\"><code>using-tbaguette</code></a> speaks only for {brand}’s own skills and says nothing about anyone else’s. When the other library has the better answer for the job in front of you, that is the one to use. The rule here is to check what is available before answering, never to prefer this library’s skill because it is this library’s.";
    pub const COEXISTENCE_REMOVAL_HTML_TEMPLATE: &str = "And if it does get in your way, removing it is deleting that one directory. There is no uninstaller, no settings entry to unpick, and nothing else on your machine to unwind — on Windows, delete the same folder.";
    pub const COEXISTENCE_REMOVAL_COMMAND: &str = "rm -rf ~/.claude/skills/TBaguette";
    pub const OTHER_AGENTS_HEADING: &str = "Not on Claude Code?";
    pub const OTHER_AGENTS_HTML_TEMPLATE: &str = "{brand_atelier} ships an integration for Codex, Cursor, Devin, GitHub Copilot — the CLI, VS Code, and the coding agent — Gemini CLI, Hermes, Kimi Code, OpenCode, and Pi as well. Those installs differ in kind rather than just in path — a plugin command here, an extension there, a line in a config file somewhere else — which is exactly why the install box carries one prompt instead of ten recipes. It identifies its own harness and takes the matching route, and when it cannot identify one it stops and asks rather than inventing a path. <a href=\"{porting_url}\">PORTING.md</a> has the harness-by-harness breakdown of what each one loads.";
    pub const TROUBLESHOOTING_HEADING: &str = "If nothing seems to be happening";
    pub const TROUBLESHOOTING_INTRO: &str = "Roughly in the order worth checking, cheapest first:";
    pub const TROUBLESHOOTING_SESSION_HTML_TEMPLATE: &str = "<strong>The conversation predates the install</strong> — including the one you installed from. Open a new one, or invoke <code>{plugin}:using-tbaguette</code> to load the entry point by hand.";
    pub const TROUBLESHOOTING_DESKTOP_HTML: &str = "<strong>You are in Claude Desktop or claude.ai chat, not Claude Code.</strong> Those load plugins from your account rather than from <code>~/.claude/skills/</code>, so nothing you cloned is visible to them.";
    pub const TROUBLESHOOTING_DESKTOP_LISTED_HTML_TEMPLATE: &str = "<strong>You are in Claude Desktop or claude.ai chat, not Claude Code.</strong> Those load plugins from your account rather than from <code>~/.claude/skills/</code>, so nothing you cloned is visible to them. Add the Atelier from <a href=\"{url}\">its directory listing</a> instead.";
    pub const TROUBLESHOOTING_TRIGGERS_HTML: &str = "<strong>Your agent has them and simply is not reaching for them.</strong> Invoke one by name. If that works, the install is fine and what you are watching is trigger matching — say what you are doing in plainer terms, or name the skill yourself.";
    pub const TROUBLESHOOTING_PATH_HTML: &str = "<strong>The clone is not where your agent reads from.</strong> Check that <code>~/.claude/skills/TBaguette/CATALOG.md</code> exists. If that directory is there but has no <code>.git</code> inside it, what you have is a previous install that lost its own history rather than a fresh one — the install prompt recognizes that case and asks before moving anything aside.";
    pub const NEXT_HEADING: &str = "Where to go from here";
    pub const NEXT_HTML_TEMPLATE: &str = "Every skill on this site is readable in full and searchable by name, summary, or category, grouped into categories on <a href=\"{home_url}\">the landing page</a>. <a href=\"{catalog_url}\">CATALOG.md</a> is that same catalog as a single file, which is the better one to hand your agent when you want it to read rather than search.";
    pub const BREADCRUMB_CURRENT: &str = "Getting started";
    pub const META_DESCRIPTION_TEMPLATE: &str = "What to do after you install {brand_atelier}: the reload new conversations need, how skills fire without being invoked, six worth trying first, and what to check when nothing seems to happen.";
}

/// The graph's data file, beside its page.
pub const GRAPH_DATA_FILENAME: &str = "graph.json";

/// Repository files the getting-started page sends a reader to, on GitHub
/// (appended to [`GITHUB_BLOB_BASE`]).
const PORTING_PATH: &str = "PORTING.md";
const CATALOG_PATH: &str = "CATALOG.md";

/// The test whose source /verify-install/ shows, highlighted, in full.
pub const INSTALL_TEST_SOURCE_PATH: &str = "verify/test_install_command.py";

/// The getting-started starter picks, each with its reason.
const GETTING_STARTED_STARTERS: [(&str, &str); 6] = [
    (
        "formidable",
        getting_started_strings::STARTER_FORMIDABLE_HTML,
    ),
    (
        "diagnosing-before-fixing",
        getting_started_strings::STARTER_DIAGNOSING_BEFORE_FIXING_HTML,
    ),
    (
        "writing-the-failing-test-first",
        getting_started_strings::STARTER_WRITING_THE_FAILING_TEST_FIRST_HTML,
    ),
    (
        "confirming-before-claiming-done",
        getting_started_strings::STARTER_CONFIRMING_BEFORE_CLAIMING_DONE_HTML,
    ),
    (
        "orchestrating-work-end-to-end",
        getting_started_strings::STARTER_ORCHESTRATING_WORK_END_TO_END_HTML,
    ),
    (
        "knowing-when-to-stop",
        getting_started_strings::STARTER_KNOWING_WHEN_TO_STOP_HTML,
    ),
];

/// Home / `current`, for the two prose pages.
fn breadcrumb(ctx: &PageCtx, current: &str) -> String {
    format!(
        r#"<nav class="container breadcrumb" aria-label="{aria}">
  <a href="{home}">{home_label}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">{current}</span>
</nav>"#,
        aria = escape_html(chrome::BREADCRUMB_ARIA_LABEL),
        home = ctx.url(""),
        home_label = escape_html(chrome::BREADCRUMB_HOME),
        current = escape_html(current),
    )
}

// ---------------------------------------------------------------------------
// /verify-install/
// ---------------------------------------------------------------------------

/// Highlighted source as a numbered block, one `<li>` per line (templates.py
/// `_render_code_block`); an empty line holds a space so it keeps its
/// height.
fn code_block(highlighted_lines: &[String]) -> String {
    let rows: Vec<String> = highlighted_lines
        .iter()
        .map(|line| {
            format!(
                "<li><span class=\"code-block__line-number\"></span><span class=\"code-block__line-code\">{}</span></li>",
                if line.is_empty() { " " } else { line }
            )
        })
        .collect();
    format!(
        r#"<div class="code-block" dir="ltr">
  <div class="code-block__scroll">
    <ol class="code-block__lines">
{rows}
    </ol>
  </div>
</div>"#,
        rows = join(&rows),
    )
}

/// /verify-install/: why the install command only touches one folder, the
/// three platform commands, and the test that proves it, as
/// `highlighted_lines` (atelier_core::highlight over the test's source).
pub fn verify_install(ctx: &PageCtx, highlighted_lines: &[String]) -> String {
    use verify_strings as v;
    let test_itself_html = v::TEST_ITSELF_TEMPLATE
        .replace(
            "{test_github_url}",
            &format!("{GITHUB_BLOB_BASE}{INSTALL_TEST_SOURCE_PATH}"),
        )
        .replace("{test_source_path}", INSTALL_TEST_SOURCE_PATH);
    let four_scenarios_intro_html =
        v::FOUR_SCENARIOS_INTRO_HTML_TEMPLATE.replace("{brand}", BRAND_NAME);
    let scenario_c_html = v::SCENARIO_C_HTML_TEMPLATE.replace("{brand}", BRAND_NAME);
    let meta_description =
        v::META_DESCRIPTION_TEMPLATE.replace("{brand_atelier}", BRAND_ATELIER_TEXT);
    let article = format!(
        r#"<article class="container skill-article">
<div class="page-layout">
<div class="page-layout__main">
  <div class="skill-article__head">
    <h1 class="skill-article__title">{title}</h1>
    <p class="lede">{lede_html}</p>
  </div>

  <div class="prose">
    <h2 id="what-it-does">{what_it_does_heading}</h2>
    <p>{what_it_does_html}</p>

    <h2 id="why-it-cant-reach-anything-else">{why_heading}</h2>
    <p>{why_html}</p>

    <h2 id="four-scenarios">{four_scenarios_heading}</h2>
    <p>{four_scenarios_intro_html}</p>
    <ul>
      <li>{scenario_a_html}</li>
      <li>{scenario_b_html}</li>
      <li>{scenario_c_html}</li>
      <li>{scenario_d_html}</li>
    </ul>
    <p>{scenario_footer_html}</p>

    <h2 id="every-platform">{every_platform_heading}</h2>
    <p>{every_platform_intro_html}</p>
    <p>{posix_intro}</p>
    <pre class="prose-code-block" dir="ltr"><code>{install_command}</code></pre>
    <p>{powershell_intro_html}</p>
    <pre class="prose-code-block" dir="ltr"><code>{install_command_powershell}</code></pre>
    <p>{powershell_caveat_html}</p>
    <p>{cmd_intro}</p>
    <pre class="prose-code-block" dir="ltr"><code>{install_command_cmd}</code></pre>
    <p>{cmd_caveat_html}</p>

    <h2 id="the-test-itself">{test_itself_heading}</h2>
    <p>{test_itself_html}</p>
  </div>

  {code_block}

  <div class="prose">
    <h2 id="run-it-yourself">{run_it_yourself_heading}</h2>
    <p>{run_it_yourself_html}</p>
  </div>
</div>
  {rail}
</div>
</article>"#,
        rail = contents_rail(&[
            ("what-it-does", v::WHAT_IT_DOES_HEADING),
            ("why-it-cant-reach-anything-else", v::WHY_HEADING),
            ("four-scenarios", v::FOUR_SCENARIOS_HEADING),
            ("every-platform", v::EVERY_PLATFORM_HEADING),
            ("the-test-itself", v::TEST_ITSELF_HEADING),
            ("run-it-yourself", v::RUN_IT_YOURSELF_HEADING),
        ]),
        title = escape_html(v::TITLE),
        lede_html = v::LEDE_HTML,
        what_it_does_heading = escape_html(v::WHAT_IT_DOES_HEADING),
        what_it_does_html = v::WHAT_IT_DOES_HTML,
        why_heading = escape_html(v::WHY_HEADING),
        why_html = v::WHY_HTML,
        four_scenarios_heading = escape_html(v::FOUR_SCENARIOS_HEADING),
        scenario_a_html = v::SCENARIO_A_HTML,
        scenario_b_html = v::SCENARIO_B_HTML,
        scenario_d_html = v::SCENARIO_D_HTML,
        scenario_footer_html = v::SCENARIO_FOOTER_HTML,
        every_platform_heading = escape_html(v::EVERY_PLATFORM_HEADING),
        every_platform_intro_html = v::EVERY_PLATFORM_INTRO_HTML,
        posix_intro = escape_html(v::POSIX_INTRO),
        install_command = escape_html(install::INSTALL_COMMAND),
        powershell_intro_html = v::POWERSHELL_INTRO_HTML,
        install_command_powershell = escape_html(install::INSTALL_COMMAND_POWERSHELL),
        powershell_caveat_html = v::POWERSHELL_CAVEAT_HTML,
        cmd_intro = escape_html(v::CMD_INTRO),
        install_command_cmd = escape_html(install::INSTALL_COMMAND_CMD),
        cmd_caveat_html = v::CMD_CAVEAT_HTML,
        test_itself_heading = escape_html(v::TEST_ITSELF_HEADING),
        code_block = code_block(highlighted_lines),
        run_it_yourself_heading = escape_html(v::RUN_IT_YOURSELF_HEADING),
        run_it_yourself_html = v::RUN_IT_YOURSELF_HTML,
    );
    let main_html = join(&[breadcrumb(ctx, v::BREADCRUMB_CURRENT), article]);
    let title = format!("{} — {BRAND_ATELIER_TEXT}", v::TITLE);
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: &meta_description,
            body_class: "page-skill",
            path_suffix: "verify-install/",
            main_html: &main_html,
        },
    )
}

// ---------------------------------------------------------------------------
// /getting-started/
// ---------------------------------------------------------------------------

/// One list item per starter pick, its slug linked like any cross-reference.
fn getting_started_starters(ctx: &PageCtx) -> String {
    let items: Vec<String> = GETTING_STARTED_STARTERS
        .iter()
        .map(|(slug, reason_html)| {
            format!(
                "<li><a class=\"skill-link\" href=\"{}\"><code>{}</code></a> &mdash; {reason_html}</li>",
                ctx.skill_url(slug),
                escape_html(slug),
            )
        })
        .collect();
    join(&items)
}

/// /getting-started/: the first session with the library, in the order a
/// new user hits it, with the install box repeated (without its pointer to
/// this very page).
pub fn getting_started(ctx: &PageCtx) -> String {
    use getting_started_strings as g;
    let home_url = ctx.url("");
    let verify_url = ctx.url("verify-install/");
    let plugin = |template: &str| template.replace("{plugin}", PLUGIN_NAME);
    let install_html = g::INSTALL_HTML_TEMPLATE
        .replace("{home_url}", &home_url)
        .replace("{verify_url}", &verify_url);
    let how_they_fire_examples_html = g::HOW_THEY_FIRE_EXAMPLES_HTML_TEMPLATE
        .replace("{skill_count}", &ctx.skills.len().to_string())
        .replace(
            "{diagnosing_url}",
            &ctx.skill_url("diagnosing-before-fixing"),
        )
        .replace("{formidable_url}", &ctx.skill_url("formidable"));
    let bigger_work_html = g::BIGGER_WORK_HTML_TEMPLATE.replace(
        "{orchestrating_url}",
        &ctx.skill_url("orchestrating-work-end-to-end"),
    );
    let updating_automatic_html = g::UPDATING_AUTOMATIC_HTML_TEMPLATE.replace(
        "{keeping_current_url}",
        &ctx.skill_url("keeping-tbaguette-current"),
    );
    let coexistence_install_html = g::COEXISTENCE_INSTALL_HTML_TEMPLATE
        .replace("{brand}", BRAND_NAME)
        .replace("{verify_url}", &verify_url);
    let coexistence_names_html = g::COEXISTENCE_NAMES_HTML_TEMPLATE.replace("{brand}", BRAND_NAME);
    let coexistence_notices_html = g::COEXISTENCE_NOTICES_HTML_TEMPLATE
        .replace("{brand}", BRAND_NAME)
        .replace("{using_url}", &ctx.skill_url("using-tbaguette"));
    let other_agents_html = g::OTHER_AGENTS_HTML_TEMPLATE
        .replace("{brand_atelier}", BRAND_ATELIER)
        .replace(
            "{porting_url}",
            &format!("{GITHUB_BLOB_BASE}{PORTING_PATH}"),
        );
    let (reload_desktop_html, troubleshooting_desktop_html) = if CLAUDE_DIRECTORY_URL.is_empty() {
        (
            g::RELOAD_DESKTOP_HTML.to_string(),
            g::TROUBLESHOOTING_DESKTOP_HTML.to_string(),
        )
    } else {
        let listing = escape_html(CLAUDE_DIRECTORY_URL);
        (
            g::RELOAD_DESKTOP_LISTED_HTML_TEMPLATE.replace("{url}", &listing),
            g::TROUBLESHOOTING_DESKTOP_LISTED_HTML_TEMPLATE.replace("{url}", &listing),
        )
    };
    let next_html = g::NEXT_HTML_TEMPLATE
        .replace("{home_url}", &home_url)
        .replace(
            "{catalog_url}",
            &format!("{GITHUB_BLOB_BASE}{CATALOG_PATH}"),
        );
    let meta_description =
        g::META_DESCRIPTION_TEMPLATE.replace("{brand_atelier}", BRAND_ATELIER_TEXT);

    let article = format!(
        r#"<article class="container skill-article">
<div class="page-layout">
<div class="page-layout__main">
  <div class="skill-article__head">
    <h1 class="skill-article__title">{title}</h1>
    <p class="lede">{lede_html}</p>
  </div>

  <div class="prose">
    <h2 id="install">{install_heading}</h2>
    <p>{install_html}</p>
  </div>

  {install_frame}

  <div class="prose">
    <h2 id="reload">{reload_heading}</h2>
    <p>{reload_html}</p>
    <p>{reload_session_html}</p>
    <p>{reload_desktop_html}</p>

    <h2 id="confirm">{confirm_heading}</h2>
    <p>{confirm_html}</p>

    <h2 id="how-skills-fire">{how_they_fire_heading}</h2>
    <p>{how_they_fire_html}</p>
    <p>{how_they_fire_examples_html}</p>

    <h2 id="by-name">{by_name_heading}</h2>
    <p>{by_name_html}</p>

    <h2 id="starter-set">{starters_heading}</h2>
    <p>{starters_intro}</p>
    <ul>
{starters}
    </ul>

    <h2 id="bigger-work">{bigger_work_heading}</h2>
    <p>{bigger_work_html}</p>
    <p>{bigger_work_practical_html}</p>

    <h2 id="updating">{updating_heading}</h2>
    <p>{updating_html}</p>
    <p>{updating_automatic_html}</p>

    <h2 id="alongside-others">{coexistence_heading}</h2>
    <p>{coexistence_install_html}</p>
    <p>{coexistence_names_html}</p>
    <p>{coexistence_notices_html}</p>
    <p>{coexistence_removal_html}</p>
    <pre class="prose-code-block" dir="ltr"><code>{coexistence_removal_command}</code></pre>

    <h2 id="other-agents">{other_agents_heading}</h2>
    <p>{other_agents_html}</p>

    <h2 id="nothing-happened">{troubleshooting_heading}</h2>
    <p>{troubleshooting_intro}</p>
    <ul>
      <li>{troubleshooting_session_html}</li>
      <li>{troubleshooting_desktop_html}</li>
      <li>{troubleshooting_triggers_html}</li>
      <li>{troubleshooting_path_html}</li>
    </ul>

    <h2 id="next">{next_heading}</h2>
    <p>{next_html}</p>
  </div>
</div>
  {rail}
</div>
</article>"#,
        rail = contents_rail(&[
            ("install", g::INSTALL_HEADING),
            ("reload", g::RELOAD_HEADING),
            ("confirm", g::CONFIRM_HEADING),
            ("how-skills-fire", g::HOW_THEY_FIRE_HEADING),
            ("by-name", g::BY_NAME_HEADING),
            ("starter-set", g::STARTERS_HEADING),
            ("bigger-work", g::BIGGER_WORK_HEADING),
            ("updating", g::UPDATING_HEADING),
            ("alongside-others", g::COEXISTENCE_HEADING),
            ("other-agents", g::OTHER_AGENTS_HEADING),
            ("nothing-happened", g::TROUBLESHOOTING_HEADING),
            ("next", g::NEXT_HEADING),
        ]),
        title = escape_html(g::TITLE),
        lede_html = g::LEDE_HTML,
        install_heading = escape_html(g::INSTALL_HEADING),
        install_frame = install_frame(ctx, false),
        reload_heading = escape_html(g::RELOAD_HEADING),
        reload_html = g::RELOAD_HTML,
        reload_session_html = plugin(g::RELOAD_SESSION_HTML_TEMPLATE),
        confirm_heading = escape_html(g::CONFIRM_HEADING),
        confirm_html = plugin(g::CONFIRM_HTML_TEMPLATE),
        how_they_fire_heading = escape_html(g::HOW_THEY_FIRE_HEADING),
        how_they_fire_html = plugin(g::HOW_THEY_FIRE_HTML_TEMPLATE),
        by_name_heading = escape_html(g::BY_NAME_HEADING),
        by_name_html = plugin(g::BY_NAME_HTML_TEMPLATE),
        starters_heading = escape_html(g::STARTERS_HEADING),
        starters_intro = escape_html(g::STARTERS_INTRO),
        starters = getting_started_starters(ctx),
        bigger_work_heading = escape_html(g::BIGGER_WORK_HEADING),
        bigger_work_practical_html = g::BIGGER_WORK_PRACTICAL_HTML,
        updating_heading = escape_html(g::UPDATING_HEADING),
        updating_html = g::UPDATING_HTML,
        coexistence_heading = escape_html(g::COEXISTENCE_HEADING),
        coexistence_removal_html = g::COEXISTENCE_REMOVAL_HTML_TEMPLATE,
        coexistence_removal_command = escape_html(g::COEXISTENCE_REMOVAL_COMMAND),
        other_agents_heading = escape_html(g::OTHER_AGENTS_HEADING),
        troubleshooting_heading = escape_html(g::TROUBLESHOOTING_HEADING),
        troubleshooting_intro = escape_html(g::TROUBLESHOOTING_INTRO),
        troubleshooting_session_html = plugin(g::TROUBLESHOOTING_SESSION_HTML_TEMPLATE),
        troubleshooting_triggers_html = g::TROUBLESHOOTING_TRIGGERS_HTML,
        troubleshooting_path_html = g::TROUBLESHOOTING_PATH_HTML,
        next_heading = escape_html(g::NEXT_HEADING),
    );
    let main_html = join(&[breadcrumb(ctx, g::BREADCRUMB_CURRENT), article]);
    let title = format!("{} — {BRAND_ATELIER_TEXT}", g::TITLE);
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: &meta_description,
            body_class: "page-skill",
            path_suffix: GETTING_STARTED_PATH,
            main_html: &main_html,
        },
    )
}

// ---------------------------------------------------------------------------
// /graph/
// ---------------------------------------------------------------------------

/// /graph/: the page's head and the host atelier-graph mounts into, with
/// graph.json preloaded from beside it (the shell loads the graph's module on
/// this page). The counts come from the graph's banner summary.
pub fn graph_page(ctx: &PageCtx, summary: &BannerSummary) -> String {
    let graph_dir = ctx.url(GRAPH_PATH);
    let lede = graph_strings::GRAPH_PAGE_LEDE_TEMPLATE
        .replace("{skill_count}", &ctx.skills.len().to_string())
        .replace("{pair_count}", &summary.pair_count.to_string());
    let section = format!(
        r#"<section class="graph-page" aria-labelledby="graph-page-title">
  <div class="container graph-page__head">
    <h1 class="graph-page__title" id="graph-page-title">{title}</h1>
    <p class="graph-page__lede">{lede}</p>
  </div>
  <div class="graph-host" data-graph-host data-graph-data="{graph_dir}{GRAPH_DATA_FILENAME}">
    <noscript><p class="graph-host__noscript">{noscript}</p></noscript>
    <p class="graph-host__noscript graph-host__nowasm">{noscript}</p>
  </div>
</section>
<link rel="preload" as="fetch" href="{graph_dir}{GRAPH_DATA_FILENAME}" crossorigin="anonymous">"#,
        title = escape_html(graph_strings::GRAPH_PAGE_TITLE),
        lede = escape_html(&lede),
        noscript = escape_html(graph_strings::GRAPH_NOSCRIPT),
    );
    let title = format!("{} — {BRAND_ATELIER_TEXT}", graph_strings::GRAPH_PAGE_TITLE);
    shell::document(
        ctx,
        &Page {
            title: &title,
            meta_description: &lede,
            body_class: "page-graph",
            path_suffix: GRAPH_PATH,
            main_html: &section,
        },
    )
}
