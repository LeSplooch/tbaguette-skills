//! The Hermes Agent bootstrap (the repository's root `__init__.py`;
//! scripts/test_hermes_bootstrap.py).
//!
//! This suite exists because of a failure every other suite was structurally
//! unable to see: `hermes plugins install LeSplooch/tbaguette-skills` aborted
//! outright, and the bootstrap behind it had never run once, while every
//! manifest stayed valid. These are the parts of that integration a
//! repo-local test can hold - the ones that depend on this repository's own
//! files staying in step with each other. What none of them prove is that
//! Hermes still behaves the way the module's comments say; those facts came
//! from reading hermes-agent's source on 2026-09-05 and driving a real
//! install. Re-check them against a live instance, not against this file.
//!
//! The bootstrap is Python, and Hermes loads it with Python, so these tests
//! run it with `python3` (a small driver, `HERMES_DRIVER` below, loads the
//! module the way Hermes does - by path - and reports what it builds as
//! JSON). A machine without `python3` fails these tests loudly rather than
//! skipping them: a skipped bootstrap check is how a broken install
//! masquerades as a working one.
//!
//! The install scanner check is plain Rust over the repository's files.

#![cfg(unix)]

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use common::real_root;
use fancy_regex::{Regex, RegexBuilder};
use serde_json::Value;

/// Hermes' documented default for `hooks.output_spill.max_chars`. Past it, a
/// hook's context is replaced by a 500-char head, a 500-char tail and a path.
const HERMES_DEFAULT_SPILL_MAX_CHARS: usize = 10_000;

/// Loads the root `__init__.py` by path (a hyphenated package name is not a
/// Python identifier; Hermes uses spec_from_file_location too) and prints
/// everything the checks need. `FakeContext` stands in for Hermes'
/// PluginContext with its two real constraints: `register_skill` takes a
/// pathlib.Path (a str raises AttributeError there and hermes silently
/// disables the whole plugin), and it rejects a name carrying the namespace
/// separator, because it applies the namespace itself.
const HERMES_DRIVER: &str = r#"
import importlib.util, json, re, sys
from pathlib import Path

repo = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("tbaguette_hermes_plugin", repo / "__init__.py")
plugin = importlib.util.module_from_spec(spec)
spec.loader.exec_module(plugin)


class FakeContext:
    def __init__(self):
        self.skills = {}
        self.hooks = {}

    def register_skill(self, name, path, description="", frontmatter=None):
        assert isinstance(path, Path), f"register_skill needs a Path, got {type(path)}"
        assert ":" not in name, f"Hermes rejects a namespaced skill name: {name!r}"
        assert path.is_file(), f"no SKILL.md at {path}"
        self.skills[f"tbaguette-atelier:{name}"] = str(path)

    def register_hook(self, event, callback):
        self.hooks[event] = callback


body = re.sub(r"^---\n[\s\S]*?\n---\n", "", (repo / "skills/using-tbaguette/SKILL.md").read_text(encoding="utf-8"))
headings = sorted({part.split("\n", 1)[0][3:].strip() for part in re.split(r"(?m)^(?=## )", body)[1:]})

skills_dir = plugin._skills_dir()
bootstrap = plugin._build_bootstrap(skills_dir)
within_margin = len(bootstrap) <= plugin._spill_budget()

# A tightened hooks.output_spill.max_chars must shed content, never spill.
original = plugin._spill_budget
trimmed = {}
try:
    for limit in (8000, 6000, 4000, 3000):
        plugin._spill_budget = lambda lim=limit: max(lim - plugin.SPILL_MARGIN, 1500)
        trimmed[str(limit)] = len(plugin._build_bootstrap(skills_dir))
finally:
    plugin._spill_budget = original

ctx = FakeContext()
plugin.register(ctx)
on_disk = sorted(p.name for p in (repo / "skills").iterdir() if (p / "SKILL.md").is_file())
hook = ctx.hooks.get("pre_llm_call")
first = hook(is_first_turn=True, user_message="hi", conversation_history=None) if hook else None
later = hook(is_first_turn=False, user_message="hi", conversation_history=[1, 2]) if hook else None

json.dump({
    "headings": headings,
    "bootstrap_sections": list(plugin.BOOTSTRAP_SECTIONS),
    "omitted_sections": dict(plugin.OMITTED_SECTIONS),
    "bootstrap": bootstrap,
    "within_margin": within_margin,
    "trimmed_lengths": trimmed,
    "nudge": plugin.NUDGE,
    "registered": sorted(ctx.skills),
    "on_disk": on_disk,
    "hooks": sorted(ctx.hooks),
    "first": first,
    "later": later,
}, sys.stdout)
"#;

fn facts() -> &'static Value {
    static FACTS: OnceLock<Value> = OnceLock::new();
    FACTS.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        let driver = dir.path().join("driver.py");
        fs::write(&driver, HERMES_DRIVER).unwrap();
        // -I: no user site, no PYTHON* variables, no script-directory imports.
        let out = Command::new("python3")
            .arg("-I")
            .arg(&driver)
            .arg(real_root())
            .output()
            .expect("python3 is required to run the Hermes bootstrap tests");
        assert!(
            out.status.success(),
            "the Hermes bootstrap failed to load or register:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).expect("driver prints JSON")
    })
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("an array")
        .iter()
        .map(|v| v.as_str().expect("a string").to_string())
        .collect()
}

fn bootstrap() -> &'static str {
    facts()["bootstrap"].as_str().unwrap()
}

fn nudge() -> &'static str {
    facts()["nudge"].as_str().unwrap()
}

// ---------------------------------------------------------------------------
// The section split has to stay exhaustive
// ---------------------------------------------------------------------------

/// A new `## ` section in SKILL.md is a decision: does it ride in the
/// injection, or is it left for skill_view? Neither answer is safe to reach by
/// default, so this fails until someone picks one.
#[test]
fn every_skill_section_is_carried_or_deliberately_omitted() {
    let f = facts();
    let headings = strings(&f["headings"]);
    let mut classified = strings(&f["bootstrap_sections"]);
    classified.extend(f["omitted_sections"].as_object().unwrap().keys().cloned());
    let unclassified: Vec<&String> = headings
        .iter()
        .filter(|h| !classified.contains(h))
        .collect();
    assert!(
        unclassified.is_empty(),
        "every SKILL.md section is either carried or deliberately omitted (unclassified: {unclassified:?})"
    );
}

#[test]
fn the_plugin_names_no_section_skill_md_no_longer_has() {
    let f = facts();
    let headings = strings(&f["headings"]);
    let mut classified = strings(&f["bootstrap_sections"]);
    classified.extend(f["omitted_sections"].as_object().unwrap().keys().cloned());
    let stale: Vec<&String> = classified
        .iter()
        .filter(|h| !headings.contains(h))
        .collect();
    assert!(
        stale.is_empty(),
        "the plugin names no section SKILL.md no longer has (stale: {stale:?})"
    );
}

// ---------------------------------------------------------------------------
// The size guarantee
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_fits_under_hermes_default_spill_limit() {
    let len = bootstrap().chars().count();
    assert!(
        len <= HERMES_DEFAULT_SPILL_MAX_CHARS,
        "bootstrap fits under Hermes' default spill limit ({len} <= {HERMES_DEFAULT_SPILL_MAX_CHARS})"
    );
}

/// ...with the margin the module reserves, so a small edit to SKILL.md does
/// not land right on the line.
#[test]
fn bootstrap_keeps_the_margin_the_module_reserves() {
    assert_eq!(facts()["within_margin"], true);
}

/// A tightened `hooks.output_spill.max_chars` must shed content, never spill.
#[test]
fn a_tightened_max_chars_sheds_sections_instead_of_spilling() {
    let lengths = facts()["trimmed_lengths"].as_object().unwrap();
    assert_eq!(lengths.len(), 4, "all four limits were exercised");
    for (limit, len) in lengths {
        let limit: u64 = limit.parse().unwrap();
        let len = len.as_u64().unwrap();
        assert!(
            len <= limit,
            "a max_chars of {limit} sheds sections instead of spilling ({len} chars)"
        );
    }
}

// ---------------------------------------------------------------------------
// What the injected text must and must not say
// ---------------------------------------------------------------------------

/// The bug this replaced was not the truncation. It was that the truncated
/// text still said "do not try to load using-tbaguette again" - forbidding the
/// one recovery available - while 95% of the skill was missing.
#[test]
fn the_bootstrap_never_forbids_loading_the_skill_it_summarizes() {
    assert!(!bootstrap().to_lowercase().contains("do not try to load"));
}

#[test]
fn the_bootstrap_says_how_to_read_the_rest() {
    assert!(bootstrap().contains(r#"skill_view("tbaguette-atelier:using-tbaguette")"#));
}

#[test]
fn the_bootstrap_names_what_it_left_out() {
    assert!(bootstrap().contains("left out of this injection"));
    for (omitted, reason) in facts()["omitted_sections"].as_object().unwrap() {
        let reason = reason.as_str().unwrap();
        assert!(
            bootstrap().contains(omitted.as_str()) && bootstrap().contains(reason),
            "...naming {omitted:?} and why"
        );
    }
}

#[test]
fn the_bootstrap_carries_the_namespace_rule() {
    assert!(
        bootstrap().contains("`tbaguette-atelier:` prefix is not decoration"),
        "it carries the namespace rule, which is what makes a lookup resolve"
    );
}

#[test]
fn the_bootstrap_carries_this_harness_tool_mapping() {
    assert!(bootstrap().contains("Hermes Agent Tool Mapping"));
}

#[test]
fn the_bootstrap_is_framed_for_the_model() {
    assert!(bootstrap().starts_with("<EXTREMELY_IMPORTANT>"));
    assert!(bootstrap().trim_end().ends_with("</EXTREMELY_IMPORTANT>"));
}

// ---------------------------------------------------------------------------
// The per-turn nudge
// ---------------------------------------------------------------------------

/// Session start alone is measurably not enough (~95% injected, ~42% of
/// sessions ever invoking a skill), so every harness with a per-turn seam gets
/// a nudge. Hermes' is pre_llm_call on turns after the first.
#[test]
fn the_nudge_names_hermes_invocation_form_not_the_skill_tool() {
    assert!(nudge().contains(r#"skill_view("tbaguette-atelier:<skill-name>")"#));
    assert!(!nudge().contains("Skill tool"));
}

#[test]
fn the_nudge_is_short_enough_to_ride_every_turn() {
    assert!(
        nudge().chars().count() < 600,
        "{} chars",
        nudge().chars().count()
    );
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[test]
fn every_skill_on_disk_is_registered() {
    let f = facts();
    let expected: Vec<String> = strings(&f["on_disk"])
        .into_iter()
        .map(|n| format!("tbaguette-atelier:{n}"))
        .collect();
    assert!(!expected.is_empty());
    assert_eq!(strings(&f["registered"]), expected);
}

#[test]
fn the_pre_llm_call_hook_is_registered() {
    assert!(strings(&facts()["hooks"]).contains(&"pre_llm_call".to_string()));
}

#[test]
fn first_turn_gets_the_bootstrap_and_later_turns_the_nudge() {
    let f = facts();
    assert_eq!(f["first"]["context"].as_str(), Some(bootstrap()));
    assert_eq!(f["later"]["context"].as_str(), Some(nudge()));
    // The seam fires once per user turn, so a None here is a turn with no
    // reminder at all.
    for turn in ["first", "later"] {
        assert!(
            f[turn]["context"].as_str().is_some_and(|c| !c.is_empty()),
            "the {turn} turn gets something"
        );
    }
}

// ---------------------------------------------------------------------------
// Hermes' install scanner: no critical findings
// ---------------------------------------------------------------------------
//
// Vendored from hermes-agent's tools/skills_guard.py (`_shell_write_re`,
// `_AGENT_CONFIG_FILES` and friends), read on 2026-09-05. Hermes scans a
// plugin tree before installing it, and a single `critical` finding is a
// `dangerous` verdict: a hard block that `--force` does not override.
//
// The pattern is meant to catch a real persistence attack - a shell redirect
// into an agent's config file. It reads any tag-closing `>` as that redirect,
// so an inline-code span wrapping one of these filenames looks like one. That
// is why the library writes agent-config filenames as plain prose rather than
// in backticks. It is a genuinely odd rule and it is load-bearing: five
// backticked filenames across two skills were enough to make TBaguette
// uninstallable on Hermes, silently, with every other check green.
//
// If Hermes fixes the false positive, delete this check rather than working
// around it further.

const CONFIG_FILE_GROUPS: [(&str, &str); 3] = [
    (
        "agent-config",
        r"(?:AGENTS\.md|CLAUDE\.md|\.cursorrules|\.clinerules)",
    ),
    ("Hermes-config", r"\.hermes/(?:config\.yaml|SOUL\.md)"),
    (
        "other agents' config",
        r"\.(?:claude/settings|codex/config)[\w.]*",
    ),
];

/// A literal every match of the group's patterns must contain. Files without
/// one cannot match, so skipping them is only a speed-up (the lookahead in the
/// patterns puts them on a slow engine, and `docs/` is megabytes of HTML).
const GROUP_LITERALS: [&[&str]; 3] = [
    &["AGENTS.md", "CLAUDE.md", ".cursorrules", ".clinerules"],
    &[".hermes/"],
    &[".claude/settings", ".codex/config"],
];

fn shell_write_re(file_alt: &str) -> String {
    [
        format!(r#"(?:>>|[\w"'`)\]]\s*>)\s*[~\w./-]*{file_alt}(?!\.?\w)"#),
        format!(r"\bsed\b[^\n]*\s(?:-[A-Za-z]*i[A-Za-z]*|--in-place)\b[^\n]*{file_alt}(?!\.?\w)"),
        format!(r#"\btee\s+(?:-a\s+)?[~\w./"'-]*{file_alt}(?!\.?\w)"#),
        format!(r"\b(?:cp|mv)\s+[^\s|;&]+\s+[^\n|;&]{{0,40}}?{file_alt}(?!\.?\w)"),
    ]
    .join("|")
}

const MODIFY_VERB_RE: &str = concat!(
    r"(?:\bwrit(?:e|es|ing)\b|\bwritten\b|\bedit(?:s|ed|ing)?\b",
    r"|\bmodif(?:y|ies|ied|ying|ication)s?\b|\bupdat(?:e|es|ed|ing)\b",
    r"|\bappend(?:s|ed|ing)?\b|\bprepend(?:s|ed|ing)?\b",
    r"|\binject(?:s|ed|ing)?\b|\boverwrit(?:e|es|ing)\b|\boverwritten\b",
    r"|\badd\s+to\b|\bchang(?:e|es|ed|ing)\b)"
);

/// Modification intent aimed at a config file: an imperative-position verb
/// (line start or bullet) within 80 comma-free characters of the filename, or
/// a mid-line verb behind a directive marker.
fn prose_modify_re(file_alt: &str) -> String {
    format!(
        r"(?m)^\s*(?:[-*+]\s+|\d+[.)]\s+)?{MODIFY_VERB_RE}[^\n,]{{0,80}}?{file_alt}\b|(?:\byou\s+(?:must|should|need\s+to)\s+|\bplease\s+|\bmake\s+sure\s+(?:to\s+|you\s+)|\bbe\s+sure\s+to\s+){MODIFY_VERB_RE}[^\n,]{{0,80}}?{file_alt}\b"
    )
}

/// What Hermes' own `_walk` skips. Everything else in the tree is scanned,
/// including this very file, which is why its comments watch their wording.
const EXCLUDED_DIRS: [&str; 9] = [
    ".git",
    "__pycache__",
    "node_modules",
    ".venv",
    "venv",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".tox",
];

/// What a clone of this repository would hold: tracked files plus anything
/// not yet committed or ignored, read as lossy UTF-8.
fn scannable_files() -> Vec<(PathBuf, String)> {
    let root = real_root();
    let listing = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(listing.status.success());
    String::from_utf8(listing.stdout)
        .unwrap()
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .filter(|rel| {
            !rel.components()
                .any(|c| EXCLUDED_DIRS.contains(&c.as_os_str().to_string_lossy().as_ref()))
        })
        .filter_map(|rel| {
            let bytes = fs::read(root.join(&rel)).ok()?; // tracked but deleted
            Some((rel, String::from_utf8_lossy(&bytes).into_owned()))
        })
        .collect()
}

fn compile(source: &str) -> Regex {
    RegexBuilder::new(source)
        .backtrack_limit(50_000_000)
        .build()
        .unwrap_or_else(|e| panic!("scanner pattern: {e}\n{source}"))
}

fn hits(pattern: &Regex, literals: &[&str], files: &[(PathBuf, String)]) -> Vec<String> {
    let mut found = Vec::new();
    for (rel, text) in files {
        if !literals.iter().any(|l| text.contains(l)) {
            continue;
        }
        for m in pattern.find_iter(text) {
            let m = m.unwrap_or_else(|e| panic!("{}: {e}", rel.display()));
            found.push(format!(
                "{}: {:?}",
                rel.display(),
                m.as_str().chars().take(40).collect::<String>()
            ));
        }
    }
    found
}

/// Nothing in the tree may read to Hermes as a write into a config file.
/// Run over the whole repository rather than over `skills/`, because that is
/// what Hermes scans: the built pages under `docs/` are the usual offender,
/// but an update note, a reference file, or a test comment counts the same.
#[test]
fn nothing_in_the_tree_reads_as_a_shell_write_into_a_config_file() {
    let files = scannable_files();
    assert!(files.len() > 100, "scanned {} files", files.len());
    for ((label, alt), literals) in CONFIG_FILE_GROUPS.into_iter().zip(GROUP_LITERALS) {
        let found = hits(&compile(&shell_write_re(alt)), literals, &files);
        assert!(
            found.is_empty(),
            "nothing in the tree reads as a shell write into {label} files -- found {}: {:?}",
            found.len(),
            &found[..found.len().min(3)]
        );
    }
}

/// Only the agent-config group scores prose intent as critical; Hermes' own
/// config is `high` there, because setup docs routinely say "edit config.yaml".
#[test]
fn nothing_in_the_tree_reads_as_an_instruction_to_modify_agent_config_files() {
    let files = scannable_files();
    let found = hits(
        &compile(&prose_modify_re(CONFIG_FILE_GROUPS[0].1)),
        GROUP_LITERALS[0],
        &files,
    );
    assert!(
        found.is_empty(),
        "nothing in the tree reads as an instruction to modify agent-config files -- found {}: {:?}",
        found.len(),
        &found[..found.len().min(3)]
    );
}

/// Joins pieces at run time. The tree scan reads this very file, so a sample
/// that the patterns should flag cannot be written down whole in its source.
fn sample(parts: &[&str]) -> String {
    parts.concat()
}

/// The scanner patterns themselves: each fires on the thing it exists to catch
/// (else a pattern that never matches would pass the tree scan forever) and
/// stays quiet on the descriptive prose the library really contains.
#[test]
fn the_vendored_patterns_catch_what_they_exist_to_catch() {
    let agent = CONFIG_FILE_GROUPS[0].1;
    let write = compile(&shell_write_re(agent));
    for text in [
        sample(&["echo evil > ", "CLAUDE", ".md"]),
        sample(&["echo evil >> ", "AGENTS", ".md"]),
        sample(&["cat x | tee -a ~/", "CLAUDE", ".md"]),
        sample(&["sed -i 's/a/b/' ", "AGENTS", ".md"]),
        sample(&["cp payload ", "CLAUDE", ".md"]),
        // The odd one: a tag-closing > in front of the name reads as a redirect.
        sample(&["<code>", "CLAUDE", ".md</code>"]),
    ] {
        assert!(write.is_match(&text).unwrap(), "{text:?} reads as a write");
    }
    for text in [
        sample(&["CLAUDE", ".md is read at session start"]),
        sample(&["see CLAUDE", ".md.bak for history"]),
        // A longer name that merely starts with the file's: not that file.
        sample(&["echo x > ", "CLAUDE", ".md.bak"]),
    ] {
        assert!(!write.is_match(&text).unwrap(), "{text:?} is just prose");
    }
    let prose = compile(&prose_modify_re(agent));
    for text in [
        sample(&["- edit ", "CLAUDE", ".md to add the rule"]),
        sample(&["you should update ", "AGENTS", ".md"]),
    ] {
        assert!(
            prose.is_match(&text).unwrap(),
            "{text:?} reads as an instruction"
        );
    }
    assert!(
        !prose
            .is_match(&sample(&["The loader reads ", "CLAUDE", ".md first"]))
            .unwrap()
    );

    let hermes = compile(&shell_write_re(CONFIG_FILE_GROUPS[1].1));
    assert!(
        hermes
            .is_match(&sample(&["echo x >> ~/", ".hermes/config", ".yaml"]))
            .unwrap()
    );
    let other = compile(&shell_write_re(CONFIG_FILE_GROUPS[2].1));
    assert!(
        other
            .is_match(&sample(&["echo x > ~/", ".claude/settings", ".json"]))
            .unwrap()
    );
}
