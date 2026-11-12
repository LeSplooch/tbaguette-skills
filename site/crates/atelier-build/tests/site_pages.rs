//! The revamped site, built from this repository: the Mods pages state the
//! mod's own facts, the landing page leads with the release card, every page's
//! header carries the four links, no page loads the retired site.js or graph.js,
//! and each page hands the WASM glue a fingerprinted module URL.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use atelier_build::pages::PageCtx;
use atelier_build::pages::mods::ModsSite;
use atelier_build::pages::shell::module_script;
use atelier_build::pages::unfold;
use atelier_build::repo::{self, ModSource};
use atelier_build::write::{BuildOptions, EXPECTED_SKILL_COUNT, asset_versions, build_site_with};
use atelier_core::model::categories;
use chrono::{DateTime, Utc};

const BASE: &str = "/tbaguette-skills";
/// Inside the climb release's window and the release card's, so the "New" tag
/// and the card show.
const NOW: &str = "2026-10-29T14:11:17+00:00";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(NOW)
        .unwrap()
        .with_timezone(&Utc)
}

/// One build for the whole file, into Cargo's scratch directory.
fn built() -> &'static Path {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        let parent = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("site-pages-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        let out = parent.join("docs");
        let options = BuildOptions {
            base_path: BASE.into(),
            now: now(),
            history_rev: "c1b5b199bf".into(),
        };
        build_site_with(&root(), &out, &options).expect("the build");
        out
    })
}

fn page(rel: &str) -> String {
    fs::read_to_string(built().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn html_pages(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            html_pages(&path, found);
        } else if path.extension().is_some_and(|e| e == "html") {
            found.push(path);
        }
    }
}

fn all_pages() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    html_pages(built(), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            (
                p.strip_prefix(built()).unwrap().display().to_string(),
                fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

/// Text as the pages escape it.
fn esc(text: &str) -> String {
    atelier_core::escape::html_attr(text)
}

#[test]
fn the_mods_pages_exist_and_state_the_readme_and_manifest_facts() {
    let mods = page("mods/index.html");
    let climb = page("mods/climb/index.html");
    let readme = fs::read_to_string(root().join("mods/climb/README.md")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root().join("mods/climb/.claude-plugin/plugin.json")).unwrap(),
    )
    .unwrap();

    // The manifest: version and licence on the page, the description as its
    // meta description, the version on the /mods/ card.
    let version = manifest["version"].as_str().unwrap();
    assert!(
        climb.contains(&format!("v{version}")) && climb.contains(&format!("<dd>{version}</dd>"))
    );
    assert!(climb.contains(&format!(
        "/blob/master/LICENSE\">{}</a></dd>",
        manifest["license"].as_str().unwrap()
    )));
    assert!(climb.contains(&format!(
        "<meta name=\"description\" content=\"{}\">",
        esc(manifest["description"].as_str().unwrap())
    )));
    assert!(mods.contains(&format!("v{version}")));

    // Every indented README command line, as a row of the commands table;
    // every install command, copyable.
    let mut commands = 0;
    for line in readme.lines() {
        let Some(code) = line.strip_prefix("    ") else {
            continue;
        };
        if code.starts_with("/climb") {
            let command = code.split("  ").next().unwrap().trim();
            assert!(
                climb.contains(&format!(
                    "<th scope=\"row\"><code>{}</code></th>",
                    esc(command)
                )),
                "{command}"
            );
            commands += 1;
        } else if code.contains("plugin install") {
            assert!(
                climb.contains(&format!(">{}</code>", esc(code.trim()))),
                "{code}"
            );
        }
    }
    assert_eq!(commands, 7, "the README's seven commands");
    assert!(mods.contains(&esc(
        "/plugin install climb --marketplace LeSplooch/tbaguette-skills"
    )));

    // The shell limit is stated, and the Limits section is on the page.
    assert!(climb.contains("<span class=\"climb-limit__text\">A shell command is not inspected"));
    let limits = readme
        .split("## Limits")
        .nth(1)
        .unwrap()
        .trim()
        .replace('\n', " ");
    assert!(climb.contains(&esc(&limits)), "the Limits paragraph");

    // The live log, every row as written, in the table and in the replay.
    let log = fs::read_to_string(root().join(repo::CLIMB_LIVE_LOG_PATH)).unwrap();
    for row in log.lines().skip(1) {
        let cells: String = row.split('\t').map(|c| format!("<td>{c}</td>")).collect();
        assert_eq!(
            climb.matches(&cells).count(),
            2,
            "row {row}: once in the log table, once in the replay"
        );
    }

    // The replay's finished state, and the ring's.
    assert!(climb.contains("<figure class=\"climb-replay is-met\" data-climb-replay"));
    assert!(
        climb
            .contains("climb ended: bar met: the check passed at iteration 2 ○●  exit 0  score 12")
    );
    assert!(climb.contains("data-loop-rows=\"1:1:5 2:0:12\""));

    // The links: the skill the manifest's homepage names, and the source.
    assert!(climb.contains(&format!(
        "href=\"{BASE}/skills/designing-verifiable-goals/\""
    )));
    assert!(
        climb.contains(
            "href=\"https://github.com/LeSplooch/tbaguette-skills/tree/master/mods/climb\""
        )
    );
    assert!(mods.contains(&format!("href=\"{BASE}/mods/climb/\"")));
}

/// The release card's markup, from its `<aside>` to its `</aside>`.
fn card() -> String {
    let index = page("index.html");
    let card = &index[index.find("<aside class=\"unfold\"").expect("the card")..];
    card[..card.find("</aside>").unwrap()].to_string()
}

/// An attribute's value, unescaped, from markup.
fn attribute(html: &str, name: &str) -> String {
    let start = html
        .find(&format!("{name}=\""))
        .unwrap_or_else(|| panic!("no {name}"));
    let value = &html[start + name.len() + 2..];
    value[..value.find('"').unwrap()]
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

#[test]
fn the_landing_page_leads_with_the_release_card_and_no_other_announcement() {
    let index = page("index.html");
    let main = &index[index.find("<main").expect("main")..];
    let card_at = main.find("<aside class=\"unfold\"").expect("the card");
    assert!(
        card_at < main.find("class=\"hero__grid\"").unwrap(),
        "the card leads the hero"
    );
    assert_eq!(main.matches("<aside").count(), 1, "the one announcement");
    for retired in [
        "climb-banner--release",
        "data-climb-banner",
        "century-banner",
        "release-banner",
        "graph-banner",
        "banner-slot",
        "milestone",
    ] {
        assert!(!index.contains(retired), "{retired} is gone");
    }
    let card = card();
    assert!(card.contains("aria-label=\"New: moves, a mod for the end of a run\""));
    assert!(card.contains("<span>A new mod for the end of a run.</span>"));
    assert_eq!(attribute(&card, "data-unfold-new-until"), "2026-11-26");
    assert_eq!(
        attribute(&card, "data-href"),
        format!("{BASE}/skills/{{slug}}/")
    );
    assert!(index.contains("<noscript><style>.unfold{--open:1}"));
    assert!(card.contains("data-used>100%. The report, then one question.</p>"));
}

#[test]
fn every_skill_is_one_bead_with_a_family_in_catalog_order() {
    let beads: Vec<(String, usize)> =
        serde_json::from_str(&attribute(&card(), "data-beads")).unwrap();
    let skills: Vec<&str> = categories()
        .iter()
        .flat_map(|c| c.skill_slugs.iter().copied())
        .collect();
    assert_eq!(beads.len(), EXPECTED_SKILL_COUNT);
    assert_eq!(
        beads.iter().map(|b| b.0.as_str()).collect::<Vec<_>>(),
        skills
    );
    for (slug, family) in &beads {
        assert!(categories()[*family].skill_slugs.contains(&slug.as_str()));
    }
}

#[test]
fn every_ship_leads_to_a_real_page_of_the_built_site() {
    let index = page("index.html");
    let card = card();
    let links: Vec<String> = card
        .split("class=\"unfold__ship\" href=\"")
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].to_string())
        .collect();
    assert_eq!(
        links,
        [
            format!("{BASE}/mods/moves/"),
            format!("{BASE}/skills/offering-the-next-move/"),
            format!("{BASE}/mods/"),
            format!("{BASE}/#notes-title"),
        ]
    );
    for link in &links {
        let (path, anchor) = link.split_once('#').unwrap_or((link, ""));
        let rel = path.strip_prefix(BASE).unwrap().trim_start_matches('/');
        let page = built().join(rel).join("index.html");
        assert!(page.is_file(), "{link}: {} is missing", page.display());
        if !anchor.is_empty() {
            assert!(index.contains(&format!("id=\"{anchor}\"")), "{link}");
        }
    }
}

#[test]
fn the_router_table_is_written_on_every_build_and_linked_by_its_content() {
    let written = page("assets/skill-router.json");
    assert_eq!(written, atelier_build::check::router::site_table(&root()).unwrap());
    assert!(written.len() < 256 * 1024, "{} bytes", written.len());
    let table: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(table["index"].as_object().unwrap().len(), EXPECTED_SKILL_COUNT);
    assert!(table["routes"][0][1].is_string());
    assert!(table["stop"].is_array());
    // The fingerprint follows this build's table, not whatever is in docs/assets.
    let url = attribute(&card(), "data-router");
    let (path, version) = url.split_once("?v=").expect("a fingerprint");
    assert_eq!(path, format!("{BASE}/assets/skill-router.json"));
    assert_eq!(version.len(), 10);
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("graph.css"), &written).unwrap();
    assert_eq!(asset_versions(dir.path()).unwrap()["graph.css"], version);
}

#[test]
fn a_build_past_the_cards_date_leaves_it_out() {
    let skills = repo::load_skills(&root()).unwrap();
    let at = |day: &str| {
        let now = DateTime::parse_from_rfc3339(&format!("{day}T23:59:59+00:00"))
            .unwrap()
            .with_timezone(&Utc);
        let ctx = PageCtx::new(
            skills.clone(),
            categories(),
            BASE,
            now,
            "3.0.0".into(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .unwrap();
        unfold::card(&ctx, &unfold::CURRENT)
    };
    assert!(at("2026-11-26").contains("data-unfold"));
    assert_eq!(at("2026-11-27"), "");
}

#[test]
fn the_card_takes_the_family_colours_in_both_themes() {
    let styles = fs::read_to_string(root().join("docs/assets/styles.css")).unwrap();
    assert!(styles.contains(".unfold,\n.counter,"), "the dark list");
    assert!(styles.contains(":is(.crumb, .unfold, .counter, .skills)"), "flour's");
}

#[test]
fn every_page_header_has_skills_mods_graph_and_getting_started_in_order() {
    let pages = all_pages();
    assert!(pages.len() > 100);
    let want = [
        format!("href=\"{BASE}/#skills\""),
        format!("href=\"{BASE}/mods/\""),
        format!("href=\"{BASE}/graph/\""),
        format!("href=\"{BASE}/getting-started/\""),
    ];
    for (name, html) in &pages {
        let nav = &html[html
            .find("<nav class=\"site-header__nav\"")
            .unwrap_or_else(|| panic!("{name}: no header nav"))..];
        let nav = &nav[..nav.find("</nav>").unwrap()];
        let at: Vec<usize> = want
            .iter()
            .map(|h| {
                nav.find(h.as_str())
                    .unwrap_or_else(|| panic!("{name}: header lacks {h}"))
            })
            .collect();
        assert!(
            at.windows(2).all(|w| w[0] < w[1]),
            "{name}: header links out of order"
        );
        assert_eq!(nav.matches("<a ").count(), 4, "{name}: four links");
        assert!(
            nav.contains("data-mods-climb"),
            "{name}: the Mods link carries its animation hook"
        );
        assert!(
            nav.contains("data-climb-new-until=\"2026-11-26\"><span class=\"visually-hidden\">, </span>New</span>"),
            "{name}: Mods is New, and its name reads \"Mods, New\" rather than \"ModsNew\""
        );
        assert!(
            nav.contains("<span class=\"visually-hidden\">Mods</span><span class=\"mods-climb__word\" aria-hidden=\"true\">"),
            "{name}: the word is read whole, not spelled out letter by letter"
        );
    }
    let climb = page("mods/climb/index.html");
    assert!(climb.contains("site-header__nav-link--mods site-header__nav-link--new\" href=\"/tbaguette-skills/mods/\" aria-current=\"true\""));
    assert!(
        page("mods/index.html").contains("href=\"/tbaguette-skills/mods/\" aria-current=\"page\"")
    );
}

#[test]
fn no_page_loads_site_js_or_graph_js() {
    let retired = regex::Regex::new(r"\b(site|graph)\.js\b").unwrap();
    for (name, html) in all_pages() {
        assert!(
            !retired.is_match(&html),
            "{name} still references {:?}",
            retired.find(&html).unwrap().as_str()
        );
    }
}

#[test]
fn every_page_loads_the_wasm_module_and_only_the_graph_page_loads_the_graph() {
    let assets = root().join("docs/assets");
    let fingerprinted = assets.join("atelier_web_bg.wasm").is_file();
    for (name, html) in all_pages() {
        assert_eq!(
            html.matches("import('/tbaguette-skills/assets/atelier_web.js")
                .count(),
            1,
            "{name}"
        );
        let wasm = format!(
            "m.default({{module_or_path:'{BASE}/assets/atelier_web_bg.wasm{}",
            if fingerprinted { "?v=" } else { "'" }
        );
        assert!(html.contains(&wasm), "{name}: {wasm}");
        let graph = html.contains("atelier_graph.js");
        assert_eq!(
            graph,
            name == "graph/index.html",
            "{name}: the graph module belongs to /graph/ alone"
        );
    }
}

/// The stylesheet links of a page, in order, with their `?v=` dropped.
fn stylesheets(html: &str) -> Vec<String> {
    let re =
        regex::Regex::new(r#"<link rel="stylesheet" href="([^"?]+)(\?v=[0-9a-f]{10})?">"#).unwrap();
    re.captures_iter(html)
        .map(|c| {
            assert!(
                c.get(2).is_some(),
                "{} is linked without a fingerprint",
                &c[1]
            );
            c[1].to_string()
        })
        .collect()
}

#[test]
fn every_page_loads_the_core_stylesheet_then_at_most_its_own() {
    let a = |name: &str| format!("{BASE}/assets/{name}");
    for (name, html) in all_pages() {
        let expected = if name == "index.html" {
            vec![a("styles.css"), a("landing.css")]
        } else if name == "graph/index.html" {
            vec![a("styles.css"), a("graph.css")]
        } else if name.starts_with("mods/") {
            vec![a("styles.css"), a("mods.css")]
        } else {
            vec![a("styles.css")]
        };
        assert_eq!(stylesheets(&html), expected, "{name}");
    }
}

#[test]
fn each_stylesheet_stays_well_under_the_plugin_directorys_file_limit() {
    // The directory holds any text file over 256 KiB; a page's stylesheet
    // reaching three quarters of that is the moment to split it again.
    for name in ["styles.css", "landing.css", "graph.css", "mods.css"] {
        let size = fs::metadata(root().join("docs/assets").join(name))
            .unwrap_or_else(|e| panic!("{name}: {e}"))
            .len();
        assert!(size < 192 * 1024, "{name} is {size} bytes, past 192 KiB");
    }
}

#[test]
fn the_module_loader_passes_the_fingerprinted_wasm_url_to_the_glue() {
    let dir = tempfile::tempdir().unwrap();
    for (name, bytes) in [
        ("atelier_web.js", "glue"),
        ("atelier_web_bg.wasm", "\0asm"),
        ("atelier_graph.js", "g"),
        ("atelier_graph_bg.wasm", "\0asm2"),
    ] {
        fs::write(dir.path().join(name), bytes).unwrap();
    }
    let versions = asset_versions(dir.path()).unwrap();
    for name in [
        "atelier_web.js",
        "atelier_web_bg.wasm",
        "atelier_graph.js",
        "atelier_graph_bg.wasm",
    ] {
        assert_eq!(versions[name].len(), 10, "{name} is fingerprinted");
    }
    let ctx = PageCtx {
        base_path: BASE.into(),
        now: now(),
        plugin_version: String::new(),
        asset_versions: versions.clone(),
        categories: &[],
        skills: BTreeMap::new(),
        fresh: BTreeMap::new(),
    };
    assert_eq!(
        module_script(&ctx, "atelier_web"),
        format!(
            "<script type=\"module\">import('{BASE}/assets/atelier_web.js?v={}').then(function(m){{return m.default({{module_or_path:'{BASE}/assets/atelier_web_bg.wasm?v={}'}});}}).catch(function(){{var r=document.documentElement;if(!r.hasAttribute('data-nowasm')){{r.setAttribute('data-nowasm','');console.info('TBaguette: WebAssembly did not load here, so the page stays static.');}}}});</script>",
            versions["atelier_web.js"], versions["atelier_web_bg.wasm"]
        )
    );
}

/// What a browser with no WASM reads: the rules that serve a page with no script, scoped to the
/// mark the loader sets, and the graph page's sentence in a form scripts do not silence.
#[test]
fn a_page_without_the_wasm_falls_back_on_what_a_page_without_script_shows() {
    let index = page("index.html");
    let quiet = "<noscript><style>.unfold{--open:1}";
    let start = index.find(quiet).expect("the no-script rules");
    let rules = &index[start + "<noscript><style>".len()..];
    let plain = &rules[..rules.find("</style>").unwrap()];
    let scope = "html[data-nowasm]:not([data-wasm]) ";
    assert!(plain.contains(".unfold__toggle") && plain.contains(".unfold__try"));
    let scoped = plain
        .replace(".unfold{", &format!("{scope}.unfold{{"))
        .replace(".unfold__more,", &format!("{scope}.unfold__more,"))
        .replace(".unfold__more-inner{", &format!("{scope}.unfold__more-inner{{"))
        .replace(".unfold__toggle,", &format!("{scope}.unfold__toggle,"))
        .replace(".unfold__try,", &format!("{scope}.unfold__try,"))
        .replace(".unfold__braid-lane{", &format!("{scope}.unfold__braid-lane{{"));
    assert!(index.contains(&format!("<style>{scoped}</style>")), "{scoped}");

    let graph = page("graph/index.html");
    let sentence = graph.split_once("<noscript><p class=\"graph-host__noscript\">").unwrap().1;
    let sentence = &sentence[..sentence.find("</p>").unwrap()];
    assert!(graph.contains(&format!(
        "<p class=\"graph-host__noscript graph-host__nowasm\">{sentence}</p>"
    )));
    let css = fs::read_to_string(root().join("docs/assets/graph.css")).unwrap();
    assert!(css.contains("html[data-nowasm] .graph-host__nowasm { display: block; }"));
    let core = fs::read_to_string(root().join("docs/assets/styles.css")).unwrap();
    assert!(core.contains("html[data-nowasm]:not([data-wasm])"), "the dead controls go");
    let landing = fs::read_to_string(root().join("docs/assets/landing.css")).unwrap();
    assert!(landing.contains(".unfold:not([data-unfold-live]) .unfold__toggle { visibility: hidden; }"));
}

/// A ModsSite read over a changed copy of the real mod.
/// One change to a mod, for the failure cases.
type Edit = Box<dyn Fn(&mut ModSource)>;

fn read_with(edit: impl Fn(&mut ModSource)) -> anyhow::Result<ModsSite> {
    let root = root();
    let skills = repo::load_skills(&root).unwrap();
    let ctx = PageCtx::new(
        skills,
        categories(),
        BASE,
        now(),
        String::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut mods = repo::mods(&root).unwrap();
    edit(mods.iter_mut().find(|m| m.slug == "climb").unwrap());
    ModsSite::read(
        &ctx,
        &mods,
        repo::climb_live_log(&root).unwrap(),
        repo::climb_live_meta(&root).unwrap(),
    )
}

#[test]
fn a_readme_missing_what_the_page_shows_fails_the_build_naming_it() {
    assert!(read_with(|_| {}).is_ok());
    let cases: [(&str, Edit); 4] = [
        (
            "`## Limits`",
            Box::new(|m| m.readme = m.readme.replace("## Limits", "## Caveats")),
        ),
        (
            "`**Protected**`",
            Box::new(|m| m.readme = m.readme.replace("**Protected**", "**Guarded**")),
        ),
        (
            "shell command",
            Box::new(|m| {
                m.readme = m
                    .readme
                    .replace("A shell command is not inspected", "Nothing escapes")
            }),
        ),
        (
            "commands block",
            Box::new(|m| m.readme = m.readme.replace("    /climb", "    climb")),
        ),
    ];
    for (what, edit) in cases {
        let err = read_with(edit).expect_err(what).to_string();
        assert!(
            err.contains("README.md") && err.contains(what),
            "{what}: {err}"
        );
    }
    let err = read_with(|m| m.manifest.homepage = "https://example.com/".into())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("plugin.json") && err.contains("homepage"),
        "{err}"
    );
}

#[test]
fn a_climb_log_that_is_not_the_mods_format_fails_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(repo::CLIMB_LIVE_LOG_PATH);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    for (text, want) in [
        ("iteration\texit\n1\t0\n", "line 1"),
        ("iteration\texit\tscore\tcheck_ms\tturn_ms\n", "no rows"),
        (
            "iteration\texit\tscore\tcheck_ms\tturn_ms\n1\t0\tx\t1\t2\n",
            "line 2",
        ),
    ] {
        fs::write(&path, text).unwrap();
        let err = repo::climb_live_log(dir.path()).unwrap_err().to_string();
        assert!(
            err.contains("climb-live-run.tsv") && err.contains(want),
            "{err}"
        );
    }
    let rows = repo::climb_live_log(&root()).unwrap();
    assert_eq!(rows.last().unwrap().exit, 0, "the live run met the bar");
}

#[test]
fn the_replay_starts_as_the_live_run_did() {
    let climb = page("mods/climb/index.html");
    let meta = repo::climb_live_meta(&root()).unwrap();
    assert_eq!(meta.budget, 6);
    assert_eq!(meta.check, "python3 check.py");
    assert_eq!(meta.protected, ["check.py"]);
    for line in [
        "<span class=\"climb-replay__text\">/climb 6 python3 check.py</span>",
        "<span class=\"climb-replay__text\">/climb protect check.py</span>",
        "data-band-start=\"climb 0/6 waiting for the first iteration\"",
        "data-band=\"climb 1/6 ○  exit 1",
    ] {
        assert!(climb.contains(line), "{line}");
    }
    let start = climb.find("/climb 6 python3 check.py").unwrap();
    assert!(
        start < climb.find("/climb protect check.py").unwrap(),
        "the mod refuses protect before a climb starts"
    );
    assert!(!climb.contains("stand-in"), "the caption no longer hedges");
    assert!(!climb.contains("./check"));
}

#[test]
fn a_climb_meta_file_that_is_not_key_tab_value_fails_naming_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(repo::CLIMB_LIVE_META_PATH);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let rest = "date\t2026-10-29\nproject\ta project\nchecked_by\tcases\n";
    let with_rest = |head: &str| format!("{head}{rest}");
    fs::write(
        &path,
        with_rest("budget\t6\ncheck\tpython3 check.py\nprotected\tcheck.py\n"),
    )
    .unwrap();
    let good = repo::climb_live_meta(dir.path()).unwrap();
    assert_eq!(good.budget, 6);
    assert_eq!(good.restart, None, "the restart pair is optional");
    for (text, want) in [
        ("budget 6\n".to_string(), "line 1"),
        (
            with_rest("budget\t0\ncheck\tx\nprotected\ty\n"),
            "budget `0`",
        ),
        (
            with_rest("budget\tsix\ncheck\tx\nprotected\ty\n"),
            "budget `six`",
        ),
        (with_rest("budget\t6\ncheck\tx\n"), "no `protected` row"),
        (
            "budget\t6\ncheck\tx\nprotected\ty\nseed\t1\n".to_string(),
            "line 4",
        ),
        ("budget\t6\nbudget\t7\n".to_string(), "line 2"),
        ("budget\t6\ncheck\t \nprotected\ty\n".to_string(), "line 2"),
        (
            "budget\t6\ncheck\tx\nprotected\ty\nproject\tp\nchecked_by\tc\n".to_string(),
            "no `date` row",
        ),
        (
            "budget\t6\ncheck\tx\nprotected\ty\ndate\tOctober\nproject\tp\nchecked_by\tc\n"
                .to_string(),
            "date `October`",
        ),
        (
            with_rest("budget\t6\ncheck\tx\nprotected\ty\nrestarted\tonce\n"),
            "go together",
        ),
    ] {
        fs::write(&path, text).unwrap();
        let err = repo::climb_live_meta(dir.path()).unwrap_err().to_string();
        assert!(
            err.contains("climb-live-run.meta.tsv") && err.contains(want),
            "{want}: {err}"
        );
    }
}

/// The log's caption on /mods/climb/ with `meta` in place of the real run's.
fn caption_with(meta: repo::ClimbRunMeta) -> String {
    let root = root();
    let ctx = PageCtx::new(
        repo::load_skills(&root).unwrap(),
        categories(),
        BASE,
        now(),
        String::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let site = ModsSite::read(
        &ctx,
        &repo::mods(&root).unwrap(),
        repo::climb_live_log(&root).unwrap(),
        meta,
    )
    .unwrap();
    let page = atelier_build::pages::mods::climb(&ctx, &site);
    let log = &page[page.find("id=\"climb-log\"").expect("the log section")..];
    let caption = &log[log
        .find("<figcaption")
        .or_else(|| log.find("<caption"))
        .expect("a caption")..];
    caption[..caption.find("</").unwrap()].to_string()
}

#[test]
fn the_logs_caption_reads_what_it_says_about_the_run() {
    let real = repo::climb_live_meta(&root()).unwrap();
    let caption = caption_with(real.clone());
    assert!(caption.contains(&real.project), "{caption}");
    assert!(caption.contains(&real.checked_by), "{caption}");
    assert!(caption.contains("29 October 2026"), "{caption}");
    let (restarted, carried) = real.restart.clone().expect("the live run restarted once");
    assert!(
        caption.contains(&restarted) && caption.contains(&carried),
        "{caption}"
    );

    // Nothing of it is typed into the template: other facts, another caption.
    let other = caption_with(repo::ClimbRunMeta {
        date: chrono::NaiveDate::from_ymd_opt(2027, 1, 2).unwrap(),
        project: "another project".into(),
        checked_by: "three probes".into(),
        restart: None,
        ..real.clone()
    });
    assert!(other.contains("2 January 2027"), "{other}");
    assert!(
        other.contains("another project") && other.contains("three probes"),
        "{other}"
    );
    assert!(
        !other.contains("restarted") && !other.contains(&carried),
        "{other}"
    );
    assert!(
        !other.contains(&real.project) && !other.contains(&real.checked_by),
        "{other}"
    );
}

#[test]
fn the_landing_hero_holds_the_install_box_and_the_counter_strip_of_four_links() {
    let index = page("index.html");
    let hero = &index[index.find("<section class=\"hero\">").unwrap()..];
    let hero = &hero[..hero.find("</section>").unwrap()];
    let main = hero.find("class=\"hero__main\"").unwrap();
    assert!(hero[main..].find("<h1 class=\"hero__headline\"").is_some());
    assert!(hero.find("class=\"install-frame\"").unwrap() > main);
    let strip = &hero[hero.find("<nav class=\"counter\"").expect("the strip")..];
    let strip = &strip[..strip.find("</nav>").unwrap()];
    let hrefs: Vec<&str> = strip
        .match_indices("<a class=\"counter__item")
        .map(|(at, _)| {
            let rest = &strip[at..];
            let h = rest.find("href=\"").unwrap() + 6;
            &rest[h..h + rest[h..].find('"').unwrap()]
        })
        .collect();
    assert_eq!(
        hrefs,
        [
            "#skills".to_string(),
            format!("{BASE}/mods/"),
            format!("{BASE}/graph/"),
            format!("{BASE}/getting-started/"),
        ]
    );
    // Each preview's static form is in the markup: the real count with a
    // name, the climb drawn to the bar, one dot per skill in the
    // constellation, three ticked steps. Each link is named by its text,
    // the figure hidden from assistive tech.
    let skills = repo::load_skills(&root()).unwrap().len();
    assert!(strip.contains(&format!("data-counter-count=\"{skills}\">{skills}</span>")));
    assert!(strip.contains("<span class=\"mini-count__name\" data-counter-names=\"formidable|"));
    assert!(strip.contains("class=\"climb-banner__dot mini-climb__dot\""));
    let constellation = &strip[strip.find("<svg class=\"mini-graph\"").unwrap()..];
    let constellation = &constellation[..constellation.find("</svg>").unwrap()];
    assert_eq!(
        constellation.matches("<circle ").count(),
        skills,
        "a dot per skill"
    );
    assert_eq!(strip.matches("class=\"mini-steps__step\"").count(), 3);
    assert_eq!(
        strip
            .matches("class=\"counter__figure\" aria-hidden=\"true\"")
            .count(),
        4
    );
    for label in ["Skills", "Mods", "Graph", "Getting started"] {
        assert!(
            strip.contains(&format!("<span class=\"counter__label\">{label}")),
            "{label}"
        );
    }
    // The graph's numbers come from the graph, not from the page.
    let graph: serde_json::Value =
        serde_json::from_str(&page("graph/graph.json")).expect("graph.json");
    assert!(graph.is_object());
    assert!(strip.contains("cross-references between"));
    // The counter does not load the graph module.
    assert!(!index.contains("atelier_graph"));
}

#[test]
fn the_skills_section_has_the_family_index_beside_twelve_families() {
    let index = page("index.html");
    let skills = &index[index
        .find("<section class=\"skills\" id=\"skills\"")
        .expect("#skills")..];
    let nav = &skills[skills.find("<nav class=\"family-index\"").unwrap()..];
    let nav = &nav[..nav.find("</nav>").unwrap()];
    let families = categories();
    assert_eq!(nav.matches("data-family-link").count(), families.len());
    for (i, c) in families.iter().enumerate() {
        assert!(
            nav.contains(&format!(
                "href=\"#{}\" data-family-link style=\"--family: var(--graph-cat-{})\"",
                c.slug,
                i + 1
            )),
            "{}",
            c.slug
        );
        assert!(skills.contains(&format!(
            "<section class=\"category-section\" id=\"{}\" data-category-section",
            c.slug
        )));
        assert!(nav.contains(&format!(
            "data-family-count=\"{n}\">{n}</span>",
            n = c.skill_slugs.len()
        )));
    }
    // Search sits on top of the families; its root no longer claims #skills.
    assert!(skills.find("data-search-root").unwrap() < skills.find("data-family-index").unwrap());
    assert_eq!(index.matches("id=\"skills\"").count(), 1);
    assert!(skills.contains("<h2 class=\"skills__title\""));
    assert!(skills.contains("<h3 class=\"category-section__title\""));
}

#[test]
fn every_footer_links_the_families_mods_graph_the_repository_and_the_licence() {
    for (name, html) in all_pages() {
        let footer = &html[html.find("<footer class=\"site-footer\">").expect(&name)..];
        for c in categories() {
            assert!(
                footer.contains(&format!("{BASE}/#{}\"", c.slug)),
                "{name}: {}",
                c.slug
            );
        }
        for href in [
            format!("{BASE}/mods/"),
            format!("{BASE}/graph/"),
            "https://github.com/LeSplooch/tbaguette-skills\"".to_string(),
            "https://github.com/LeSplooch/tbaguette-skills/blob/master/LICENSE\"".to_string(),
        ] {
            assert!(footer.contains(&format!("href=\"{href}")), "{name}: {href}");
        }
    }
}

#[test]
fn the_climb_page_says_only_what_the_readme_says_about_a_turn() {
    let climb = page("mods/climb/index.html");
    for typed in ["makes one change", "One change", "measured by the mod,"] {
        assert!(
            !climb.contains(typed),
            "{typed}: the mod sets no such limit"
        );
    }
    assert!(climb.contains("<span class=\"climb-replay__text\">takes its turn</span>"));
    assert!(climb.contains("measured by the mod and the engine"));
    // Each copy button says which install way it copies.
    assert!(climb.contains("aria-label=\"Copy the climb install command for a terminal session\""));
    assert!(climb.contains("aria-label=\"Copy the climb install command for a shell\""));
    assert!(climb.contains(">Read designing-verifiable-goals<"));
}

#[test]
fn the_mods_facts_are_marked_rather_than_numbered() {
    let mods = page("mods/index.html");
    assert_eq!(
        mods.matches("<span class=\"mods-intro__fact-mark\" aria-hidden=\"true\"></span>")
            .count(),
        3
    );
}

#[test]
fn a_skill_page_clamps_a_long_trigger_behind_a_native_disclosure_and_has_a_rail() {
    let long = page("skills/formidable/index.html");
    assert!(long.contains("<div class=\"skill-trigger skill-trigger--long\">"));
    assert!(long.contains("<details class=\"skill-trigger__more\"><summary"));
    assert!(long.contains(">Show the full trigger<"));
    assert!(long.contains("<code class=\"page-rail__invoke\">tbaguette-atelier:formidable</code>"));
    assert!(long.contains(&format!("href=\"{BASE}/#ui-and-design\"")));
    assert!(
        !long.contains("<p class=\"lede\">"),
        "not a fifteen-line lede"
    );
}

#[test]
fn the_long_guides_rails_link_headings_that_exist() {
    for page_name in ["getting-started/index.html", "verify-install/index.html"] {
        let html = page(page_name);
        let rail = &html[html.find("page-rail__contents").expect("the rail")..];
        let rail = &rail[..rail.find("</ol>").unwrap()];
        let hrefs: Vec<&str> = rail
            .split("href=\"#")
            .skip(1)
            .map(|rest| &rest[..rest.find('"').unwrap()])
            .collect();
        assert!(hrefs.len() >= 5, "{page_name}");
        for id in hrefs {
            assert!(
                html.contains(&format!("<h2 id=\"{id}\">")),
                "{page_name}: the rail links #{id}, no such heading"
            );
        }
    }
}

/// Every mod's directory name under `mods/`, sorted.
fn mod_slugs() -> Vec<String> {
    let mut slugs: Vec<String> = fs::read_dir(root().join("mods"))
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().join(".claude-plugin/plugin.json").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    slugs.sort();
    slugs
}

/// The mods that get the plain page: every one but climb.
fn plain_slugs() -> Vec<String> {
    mod_slugs().into_iter().filter(|s| s != "climb").collect()
}

#[test]
fn every_mod_has_a_page_the_build_wrote() {
    let slugs = mod_slugs();
    assert!(slugs.len() >= 6, "{slugs:?}");
    let mods = page("mods/index.html");
    for slug in slugs {
        let file = built().join(format!("mods/{slug}/index.html"));
        assert!(file.is_file(), "mods/{slug}/index.html is missing");
        assert!(mods.contains(&format!("href=\"{BASE}/mods/{slug}/\"")), "{slug}");
    }
}

#[test]
fn every_plain_mod_page_states_its_readme_and_manifest_facts() {
    let mods = page("mods/index.html");
    let marketplace: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root().join(".claude-plugin/marketplace.json")).unwrap(),
    )
    .unwrap();
    assert!(plain_slugs().iter().any(|s| s == "watch"));
    for slug in plain_slugs() {
        let slug = slug.as_str();
        let html = page(&format!("mods/{slug}/index.html"));
        let readme = fs::read_to_string(root().join(format!("mods/{slug}/README.md"))).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root().join(format!("mods/{slug}/.claude-plugin/plugin.json")))
                .unwrap(),
        )
        .unwrap();

        let version = manifest["version"].as_str().unwrap();
        assert!(
            html.contains(&format!("v{version}"))
                && html.contains(&format!("<dd>{version}</dd>")),
            "{slug}"
        );
        assert!(mods.contains(&format!("v{version}")), "{slug}");
        assert!(
            html.contains(&format!(
                "<meta name=\"description\" content=\"{}\">",
                esc(manifest["description"].as_str().unwrap())
            )),
            "{slug}"
        );
        assert!(
            html.contains(&format!(
                "/blob/master/LICENSE\">{}</a></dd>",
                manifest["license"].as_str().unwrap()
            )),
            "{slug}"
        );

        // Its install commands, exactly as the README and marketplace name them.
        let install = format!("/plugin install {slug} --marketplace LeSplooch/tbaguette-skills");
        assert!(readme.contains(&format!("    {install}\n")), "{slug}");
        assert!(html.contains(&format!(">{}</code>", esc(&install))), "{slug}");
        assert!(mods.contains(&format!(">{}</code>", esc(&install))), "{slug}");
        assert!(
            html.contains(&format!(
                ">{}</code>",
                esc(&format!(
                    "claude plugin install {slug} --marketplace LeSplooch/tbaguette-skills"
                ))
            )),
            "{slug}"
        );
        let entry = marketplace["plugins"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == slug)
            .unwrap_or_else(|| panic!("the marketplace lists {slug}"));
        assert_eq!(entry["version"], manifest["version"], "{slug}");
        assert_eq!(entry["source"], format!("./mods/{slug}"), "{slug}");

        // Every command row of the README's Use block is on the page.
        let use_block = readme
            .split("\n## ")
            .find(|sec| sec.starts_with("Use\n"))
            .unwrap_or_else(|| panic!("{slug}: the README has a `## Use`"));
        let mut commands = 0;
        for line in use_block.lines() {
            let Some(code) = line.strip_prefix("    ") else {
                continue;
            };
            if code.starts_with('/') {
                let command = code.split("  ").next().unwrap().trim();
                assert!(
                    html.contains(&format!("<code>{}</code></th>", esc(command))),
                    "{slug}: {command}"
                );
                commands += 1;
            }
        }
        assert!(commands > 0, "{slug}: a command in the README's Use block");

        // Every bullet of how it behaves is on the page, in its own item.
        let behaves = readme
            .split("\n## ")
            .find(|sec| sec.starts_with("How it behaves\n"))
            .unwrap_or_else(|| panic!("{slug}: the README has `## How it behaves`"));
        let bullets = behaves.lines().filter(|l| l.starts_with("- ")).count();
        assert!(bullets > 0, "{slug}");
        assert_eq!(
            html.matches("<li class=\"mod-behaves__item\">").count(),
            bullets,
            "{slug}"
        );
        assert!(
            html.contains("id=\"mod-use\"") && html.contains("id=\"mod-behaves\""),
            "{slug}"
        );
        assert!(
            html.contains(&format!("<svg class=\"{slug}-art\"")),
            "{slug}: its own picture"
        );
        assert!(!html.contains("data-climb-replay"), "{slug}");
        assert!(
            html.contains(&format!(
                "href=\"https://github.com/LeSplooch/tbaguette-skills/tree/master/mods/{slug}\""
            )),
            "{slug}"
        );
        assert!(mods.contains(&format!("href=\"{BASE}/mods/{slug}/\"")), "{slug}");
        assert!(
            html.contains(&format!("href=\"{BASE}/mods/\" aria-current=\"true\"")),
            "{slug}"
        );
        // The use section's heading: watch's own, "In use" for the rest.
        let heading = if slug == "watch" { "Who can start one" } else { "In use" };
        assert!(
            html.contains(&format!("<h2 id=\"mod-use\">{heading}</h2>")),
            "{slug}"
        );
    }

    // What is specific to watch.
    let watch = page("mods/watch/index.html");
    let readme = fs::read_to_string(root().join("mods/watch/README.md")).unwrap();
    let commands = readme
        .lines()
        .filter_map(|l| l.strip_prefix("    "))
        .filter(|c| c.starts_with("/watch"))
        .count();
    assert_eq!(commands, 7, "the README's seven commands");
    assert_eq!(
        watch.matches("<li class=\"mod-behaves__item\">").count(),
        4
    );
    assert!(watch.contains("A check gets two minutes."));
    assert!(watch.contains("It cannot start a command watch."));
    assert!(watch.contains("The mod API is early access"));
    assert!(watch.contains(&format!("href=\"{BASE}/skills/automating-repetition/\"")));
    assert!(watch.contains(">Read automating-repetition<"));
}

#[test]
fn a_plain_mod_readme_missing_what_the_page_shows_fails_the_build_naming_it() {
    let root = root();
    let ctx = PageCtx::new(
        repo::load_skills(&root).unwrap(),
        categories(),
        BASE,
        now(),
        String::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let read = |slug: &str, edit: fn(&mut ModSource)| {
        let mut mods = repo::mods(&root).unwrap();
        edit(mods.iter_mut().find(|m| m.slug == slug).unwrap());
        ModsSite::read(
            &ctx,
            &mods,
            repo::climb_live_log(&root).unwrap(),
            repo::climb_live_meta(&root).unwrap(),
        )
    };
    assert!(read("watch", |_| {}).is_ok());
    // Every plain mod: no `## How it behaves` fails the build naming its README.
    for slug in plain_slugs() {
        let err = read(&slug, |m| {
            m.readme = m.readme.replace("## How it behaves", "## Notes")
        })
        .expect_err(&slug)
        .to_string();
        assert!(
            err.contains(&format!("mods/{slug}/README.md")) && err.contains("`## How it behaves`"),
            "{slug}: {err}"
        );
        // A manifest description without `: ` fails the build naming plugin.json.
        let err = read(&slug, |m| {
            m.manifest.description = m.manifest.description.replace(": ", "; ")
        })
        .expect_err(&slug)
        .to_string();
        assert!(
            err.contains("plugin.json") && err.contains("`description` needs the form"),
            "{slug}: {err}"
        );
    }
    type ReadmeEdit = fn(&mut ModSource);
    let cases: [(&str, ReadmeEdit); 3] = [
        ("`## How it behaves`", |m| {
            m.readme = m.readme.replace("## How it behaves", "## Notes")
        }),
        ("closing paragraph", |m| {
            m.readme = m.readme.replace(
                "The mod API is early access",
                "- The mod API is early access",
            )
        }),
        ("commands block", |m| {
            m.readme = m.readme.replace("    /watch", "    watch")
        }),
    ];
    for (what, edit) in cases {
        let err = read("watch", edit).expect_err(what).to_string();
        assert!(
            err.contains("mods/watch/README.md") && err.contains(what),
            "{what}: {err}"
        );
    }
}

#[test]
fn a_skill_page_lists_who_cites_it_and_whom_it_cites() {
    let html = page("skills/karen-and-the-manager/index.html");
    let strip = &html[html.find("<dl class=\"skill-cites\">").expect("the strip")..];
    let strip = &strip[..strip.find("</dl>").unwrap()];
    let graph = format!("{BASE}/graph/#skill=karen-and-the-manager");
    assert!(strip.contains(&format!("<a href=\"{graph}\">Cited by ")));
    assert!(strip.contains(&format!("<a href=\"{graph}\">Cites ")));
    assert!(strip.contains(&format!(
        "href=\"{BASE}/skills/knowing-when-to-stop/\">knowing-when-to-stop</a>"
    )));
    assert!(
        !strip.contains(">karen-and-the-manager</a>"),
        "never itself"
    );
}

#[test]
fn the_install_prompt_offers_the_skill_listing_budget_and_asks_first() {
    let index = page("index.html");
    assert!(index.contains("skillListingBudgetFraction"));
    assert!(index.contains("skillListingMaxDescChars"));
    assert!(index.contains("Change it only if I say yes."));
}

#[test]
fn the_published_wasm_names_no_local_path() {
    let assets = root().join("docs/assets");
    for name in ["atelier_web_bg.wasm", "atelier_graph_bg.wasm"] {
        let bytes = fs::read(assets.join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        for local in [
            b"/home/".as_slice(),
            b"/Users/".as_slice(),
            b"\\Users\\".as_slice(),
        ] {
            assert!(
                !bytes.windows(local.len()).any(|w| w == local),
                "{name} carries a machine's path ({})",
                String::from_utf8_lossy(local)
            );
        }
    }
}
