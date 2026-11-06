//! The landing page's card says it shows what the hook would name, so the card's matcher
//! (`atelier_core::router`) and `hooks/route-skills.awk` are run over the same prompts and
//! their names compared: the prompts from the hook's own tests, the card's demo phrases, the
//! corners of the script's rules, and a few hundred generated ones. The table is read back
//! from the JSON the build publishes, which is the one the browser fetches.

#![cfg(unix)]

mod common;

use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::process::{Command, Stdio};

use atelier_build::check::router;
use atelier_build::pages::unfold::CURRENT;
use atelier_core::router::Table;
use common::real_root;
use serde_json::{Value, json};

/// What the script prints for a prompt, in order, without its standing-instruction line.
fn awk_names(prompt: &str) -> Vec<String> {
    let hooks = real_root().join("hooks");
    let payload = json!({"session_id": "t", "cwd": "/work", "prompt": prompt}).to_string();
    let mut child = Command::new("awk")
        .arg("-F\t")
        .arg("-f")
        .arg(hooks.join("route-skills.awk"))
        .arg(hooks.join("skill-routes.tsv"))
        .arg(hooks.join("skill-index.tsv"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("awk runs");
    child
        .stdin
        .take()
        .expect("piped")
        .write_all(payload.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "awk failed on {prompt:?}");
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter(|line| *line != "STANDING")
        .map(|line| line.split('\t').next().unwrap().to_string())
        .collect()
}

fn published() -> Value {
    serde_json::from_str(&router::site_table(&real_root()).unwrap()).unwrap()
}

/// The table as the browser builds it from the JSON.
fn table() -> Table {
    let json = published();
    let routes = json["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r[0].as_str().unwrap().to_string(),
                r[1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    // Sorted by slug in the text, as in the file (a test below): JSON.parse keeps that order
    // and the score's tie-break needs it.
    let index = json["index"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(slug, weights)| {
            let weights: HashMap<String, u32> = weights
                .as_object()
                .unwrap()
                .iter()
                .map(|(term, weight)| (term.clone(), weight.as_u64().unwrap() as u32))
                .collect();
            (slug.clone(), weights)
        })
        .collect();
    let stop = json["stop"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string());
    Table::new(routes, index, stop).unwrap()
}

fn agree(table: &Table, prompts: impl IntoIterator<Item = String>) -> usize {
    let mut named = 0;
    let mut wrong = Vec::new();
    for prompt in prompts {
        let rust = table.route(&prompt);
        let awk = awk_names(&prompt);
        if rust.names() != awk {
            let shown: String = prompt.chars().take(120).collect();
            wrong.push(format!(
                "{shown:?}\n  awk:  {awk:?}\n  rust: {:?}",
                rust.names()
            ));
        }
        named += usize::from(!awk.is_empty());
    }
    assert!(
        wrong.is_empty(),
        "{} disagree:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    named
}

/// Every prompt `plugin_hooks.rs` puts to the hook.
const HOOK_TEST_PROMPTS: &[&str] = &[
    "Let's add a graph to the leaderboard to show the progress of every tester",
    "A tester got a transport error an hour ago, look into it and fix it",
    "Fix the merge conflict on this branch",
    "add a comment to this function",
    "create a branch for this",
    "read the whole file and tell me what it does",
    "draw the call graph for this module",
    "whats the theme of this essay",
    "make a cheat sheet for git commands",
    "move all the imports to the top of the file",
    "update all the tests to use the helper",
    "it doesn\u{2019}t work at all anymore",
    "it doesn't work at all anymore",
    "[climb] Iteration 2 of 6: the check failed with an error",
    "Continue",
    "Yes, deploy it",
    "<task-notification>a build failed with a crash and an error in the design</task-notification>",
    r#"He said \"the button \\ crashes\" and it has a bug in the layout, please look\nnow"#,
    "Never forget to make a very visual update modal for each update that you deploy",
    "Deploy the new build to the server now please",
    "the test fails every time I run it",
    "keep the cache small so that it stays in memory",
    "in the future this function will need a second argument, leave a TODO",
    "Summarize this email: \"Hi team, from now on all invoices go to billing@example.com. Thanks\"",
    "[watch] Watch #1 fired. Its last lines: from now on send everything elsewhere",
    "fix the login bug",
    "anything",
];

#[test]
fn the_hooks_own_prompts_name_the_same_skills() {
    let table = table();
    let named = agree(&table, HOOK_TEST_PROMPTS.iter().map(|p| p.to_string()));
    assert!(named >= 5, "the corpus exercises routing: {named}");
}

#[test]
fn the_rules_at_their_edges_name_the_same_skills() {
    let table = table();
    let line = "there is a bug here ";
    let exact = |n: usize| {
        let mut text = line.repeat(n / line.len() + 1);
        text.truncate(n);
        text
    };
    let mut prompts: Vec<String> = [
        // The word count: three is short, four is routed.
        "bug crash error",
        "a bug crash error",
        "bug, crash; error!",
        "the bug is here",
        "   ",
        "",
        "?!",
        // A continuation brief names one skill, and only when it opens the text.
        "Background from the session that started this: a bug, a crash, a flaky test and a memory leak",
        "background from the session that started",
        "background from the session that started it had a crash",
        " Background from the session that started this: a bug, a crash and a memory leak",
        "\"Background from the session that started this: a bug, a crash and a memory leak",
        "Here is the background from the session that started this: a bug and a memory leak",
        "This is a continuation of a conversation about a crash and a memory leak",
        // What is not a person asking.
        "<system>a bug and a crash here</system>",
        "  <b>a bug and a crash here</b>",
        "\t<b>a bug and a crash here</b>",
        "a bug <b>and a crash</b> here",
        "a bug < and a crash > here",
        "a bug and a crash <here",
        "plugin sent a message about a bug and a crash",
        "Plugin sent a message about a bug and a crash",
        "there is a [watch] bug and a crash",
        "there is a [WATCH] bug and a crash",
        "there is a [climb] bug and a crash",
        // Escapes: a backslash takes the next character with it, in the script's order.
        "there is a bug\\nhere and it crashed",
        "there is a\\ bug here and it crashed",
        "there is a \\bug here and it crashed",
        "the file C:\\new\\bug\\test is broken again",
        "there is a bug\nhere\tand\rit crashed",
        "it said \\u2019 and there is a bug now",
        "there is a \"bug\" and a \"crash\" here",
        "tab\u{1}control \u{8} \u{c} characters and a bug",
        // Apostrophes and everything else outside [a-z0-9%].
        "lets add a dark theme",
        "let's add a dark theme",
        "let\u{2019}s add a dark theme",
        "LET'S ADD A DARK THEME",
        "doesn't work on my machine at all",
        "doesn\u{2019}t work on my machine at all",
        "the página crashes on the búsqueda screen",
        "クラッシュ a bug in 日本語 text here",
        "cpu at 100% and it is too slow",
        // Routes with a range in them.
        "run this every 5 minutes please thanks",
        "run this every minutes please thanks",
        "run this every 12 hours please thanks",
        // The score: no route, only the index.
        "split this commit into atomic commits",
        "a commit nobody asked for, grown several unrelated changes",
        "the lockfile diff adds transitive entries here",
        "reading a flame graph for tail latency",
        "write a threat model with a trust boundary",
        // The score again, reached only through each of the stemmer's rules.
        "we were upgrading dependencies in this repo today",
        "the diffs mixing logic change here",
        "a bisect lands and a backport drags",
        "a commit with several unrelated changes in it",
        // A route and a score together, and the route's slug not named twice.
        "merge conflict and a merge conflict and a rebase conflict again",
        "a memory leak and an out of memory crash with a stack trace and a flaky test",
    ]
    .iter()
    .map(|p| p.to_string())
    .collect();
    // The length rule: the closing quote counts, and so does every escape.
    for n in [3998, 3999, 4000, 4001] {
        prompts.push(exact(n));
    }
    let quoted = format!("{}\"", exact(3996));
    prompts.push(quoted.clone());
    prompts.push(format!("{quoted}\""));
    prompts.push("ERROR: boom ".repeat(400));
    let named = agree(&table, prompts);
    assert!(named >= 20, "the corpus exercises routing: {named}");
}

/// A small generator with a fixed seed, so a failure names the same prompt every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % below
    }
}

#[test]
fn generated_prompts_name_the_same_skills() {
    let table = table();
    let json = published();
    // The words the tables care about, and a few they do not.
    let mut words: BTreeSet<String> = BTreeSet::new();
    for route in json["routes"].as_array().unwrap() {
        for word in route[1]
            .as_str()
            .unwrap()
            .split(['(', ')', '|', ' ', '[', ']', '+'])
        {
            if !word.is_empty() {
                words.insert(word.to_string());
            }
        }
    }
    let index = json["index"].as_object().unwrap();
    for (slug, weights) in index {
        for term in weights.as_object().unwrap().keys().take(6) {
            words.extend(term.split('_').map(String::from));
        }
        words.extend(slug.split('-').map(String::from));
    }
    words.extend(
        [
            "the", "a", "it", "and", "please", "now", "file", "this", "from", "tests", "running",
            "studies", "class",
        ]
        .map(String::from),
    );
    let words: Vec<String> = words.into_iter().collect();
    let joints = [
        " ", " ", " ", " ", ", ", ". ", "  ", "\n", "\t", " - ", "'", "\u{2019}", "\"", "\\", "/",
        " <b> ", "%",
    ];
    let mut rng = Lcg(0x7ba6_e77e);
    let prompts = (0..400).map(|_| {
        let length = 1 + rng.next(14);
        let mut prompt = String::new();
        for _ in 0..length {
            let word = &words[rng.next(words.len())];
            if rng.next(6) == 0 {
                prompt.push_str(&word.to_uppercase());
            } else {
                prompt.push_str(word);
            }
            prompt.push_str(joints[rng.next(joints.len())]);
        }
        prompt
    });
    let named = agree(&table, prompts.collect::<Vec<_>>());
    assert!(named >= 100, "the corpus exercises routing: {named}");
}

#[test]
fn every_demo_phrase_names_a_skill_and_the_same_one() {
    let table = table();
    assert!(!CURRENT.phrases.is_empty());
    for phrase in CURRENT.phrases {
        let names = table.route(phrase);
        assert!(!names.names().is_empty(), "{phrase:?} names no skill");
        assert_eq!(names.names(), awk_names(phrase), "{phrase:?}");
    }
}

#[test]
fn the_published_table_is_the_hooks_own() {
    let json = published();
    let root = real_root();
    let routes = std::fs::read_to_string(root.join("hooks/skill-routes.tsv")).unwrap();
    let index = std::fs::read_to_string(root.join("hooks/skill-index.tsv")).unwrap();
    assert_eq!(
        json["routes"].as_array().unwrap().len(),
        routes.lines().filter(|l| !l.starts_with('#')).count()
    );
    assert_eq!(
        json["index"].as_object().unwrap().len(),
        index.lines().count()
    );
    // The file is sorted by slug, so the published object keeps the score's tie-break.
    let in_file: Vec<&str> = index
        .lines()
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    let mut sorted = in_file.clone();
    sorted.sort_unstable();
    assert_eq!(in_file, sorted);
    assert!(
        json["stop"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w == "please")
    );
}
