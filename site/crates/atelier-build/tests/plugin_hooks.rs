//! The plugin's own hooks (scripts/test_hooks.py): the per-harness hook
//! configs, `hooks/user-prompt-submit`'s per-turn nudge in each of its output
//! modes and the skills it names for a prompt, `hooks/session-start`'s injection
//! and its update-check block, `hooks/subagent-start`, `hooks/post-tool-use`, the
//! `run-hook.cmd` wrapper, and the `.gitattributes` rule that keeps the hook
//! scripts LF on a Windows checkout.
//!
//! The update-check scenarios run against throwaway local git repositories (a
//! fake origin and a clone of it, joined by a filesystem path) so the suite
//! needs no network. The scripts locate their plugin root from `$0`, so each
//! fixture carries its own physical copy of them.

#![cfg(unix)]

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use atelier_build::check::router;
use common::real_root;
use serde_json::{Value, json};

const USING_TBAGUETTE_FIXTURE: &str = "---\nname: using-tbaguette\ndescription: fixture for plugin_hooks.rs\n---\n\n# using-tbaguette (fixture)\n";

fn hooks_dir() -> PathBuf {
    real_root().join("hooks")
}

fn json_file(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("a JSON object")
        .keys()
        .cloned()
        .collect()
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

/// Run `bash <script> <args...>` with `stdin` piped (or closed) and `envs` added.
fn bash(script: &Path, args: &[&str], stdin: Option<&str>, envs: &[(&str, &Path)]) -> Output {
    let mut cmd = Command::new("bash");
    cmd.arg(script).args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    match stdin {
        Some(_) => cmd.stdin(Stdio::piped()),
        None => cmd.stdin(Stdio::null()),
    };
    let mut child = cmd.spawn().expect("bash runs");
    if let Some(text) = stdin {
        let mut pipe = child.stdin.take().unwrap();
        // The script may exit without reading (that is the point of some
        // inputs), so a broken pipe is not an error here.
        let _ = pipe.write_all(text.as_bytes());
    }
    child.wait_with_output().expect("bash finishes")
}

fn stdout_json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}):\n{}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Copies session-start and run-hook.cmd into a fixture plugin root.
fn install_hook_scripts(root: &Path) {
    let dest_dir = root.join("hooks");
    fs::create_dir_all(&dest_dir).unwrap();
    for name in ["session-start", "run-hook.cmd"] {
        let dest = dest_dir.join(name);
        fs::copy(hooks_dir().join(name), &dest).unwrap();
        let mode = fs::metadata(&dest).unwrap().permissions().mode();
        fs::set_permissions(&dest, fs::Permissions::from_mode(mode | 0o111)).unwrap();
    }
}

/// A directory shaped like a plugin root: the one file session-start always
/// reads, plus its own copy of the hook scripts. No `.git`.
fn make_plugin_root(parent: &Path, name: &str) -> PathBuf {
    let root = parent.join(name);
    let skill_dir = root.join("skills/using-tbaguette");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), USING_TBAGUETTE_FIXTURE).unwrap();
    install_hook_scripts(&root);
    root
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} in {}: {}",
        cwd.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A throwaway repo standing in for the published one, branch `master`
/// (session-start's fetch/compare hardcodes the name), with the hook scripts
/// committed so a clone carries working copies.
fn init_origin(parent: &Path, name: &str) -> PathBuf {
    let origin = make_plugin_root(parent, name);
    git(&origin, &["init", "--quiet", "--initial-branch=master"]);
    git(&origin, &["config", "user.email", "test@example.com"]);
    git(&origin, &["config", "user.name", "Test"]);
    git(&origin, &["add", "-A"]);
    git(&origin, &["commit", "--quiet", "-m", "initial"]);
    origin
}

fn clone_of(origin: &Path, dest: &Path) {
    let parent = dest.parent().unwrap();
    git(
        parent,
        &[
            "clone",
            "--quiet",
            origin.to_str().unwrap(),
            dest.to_str().unwrap(),
        ],
    );
}

fn run_session_start(plugin_root: &Path, format: Option<&str>) -> Value {
    let script = plugin_root.join("hooks/session-start");
    let args: Vec<&str> = format.into_iter().collect();
    let out = bash(&script, &args, None, &[]);
    assert!(
        out.status.success(),
        "session-start exits 0 (root={}): {}",
        plugin_root.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    stdout_json(&out)
}

fn context_of(payload: &Value) -> String {
    payload["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("hookSpecificOutput.additionalContext")
        .to_string()
}

fn str_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
}

// ---------------------------------------------------------------------------
// The hook configs
// ---------------------------------------------------------------------------

#[test]
fn hooks_json_shape() {
    let data = json_file(&hooks_dir().join("hooks.json"));
    let groups = data["hooks"]["SessionStart"]
        .as_array()
        .expect("SessionStart");
    assert_eq!(
        groups.len(),
        1,
        "declares exactly one SessionStart matcher group"
    );
    assert_eq!(
        groups[0]["matcher"], "startup|clear|compact",
        "matcher covers startup, clear, and compact"
    );
    let inner = groups[0]["hooks"].as_array().expect("hooks");
    assert_eq!(inner.len(), 1, "exactly one hook command in the group");
    let command = &inner[0];
    assert_eq!(command["type"], "command");
    assert_eq!(command["shell"], "bash", "runs through bash");
    assert_eq!(command["async"], json!(false), "not async");
    let command_str = str_at(command, "command");
    assert!(
        command_str.contains("${CLAUDE_PLUGIN_ROOT}/hooks/run-hook.cmd"),
        "invokes run-hook.cmd via CLAUDE_PLUGIN_ROOT: {command_str}"
    );
    assert!(
        command_str.contains("session-start"),
        "passes session-start as the script name"
    );

    // UserPromptSubmit: the per-turn re-assertion. SessionStart alone decays
    // over a long session, so this group existing is the fix.
    let prompt_groups = data["hooks"]["UserPromptSubmit"]
        .as_array()
        .expect("UserPromptSubmit");
    assert_eq!(
        prompt_groups.len(),
        1,
        "declares exactly one UserPromptSubmit matcher group"
    );
    let prompt_hooks = prompt_groups[0]["hooks"].as_array().expect("hooks");
    assert_eq!(
        prompt_hooks.len(),
        1,
        "exactly one UserPromptSubmit command"
    );
    let prompt = &prompt_hooks[0];
    assert_eq!(prompt["type"], "command");
    assert_eq!(prompt["shell"], "bash");
    assert_eq!(
        prompt["async"],
        json!(false),
        "UserPromptSubmit is not async (additionalContext must land before the turn)"
    );
    let prompt_str = str_at(prompt, "command");
    assert!(prompt_str.contains("${CLAUDE_PLUGIN_ROOT}/hooks/run-hook.cmd"));
    assert!(
        prompt_str.contains("user-prompt-submit"),
        "passes user-prompt-submit as the script name"
    );

    for (event, name) in [
        ("SubagentStart", "subagent-start"),
        ("PostToolUse", "post-tool-use"),
    ] {
        let hook = &data["hooks"][event][0]["hooks"][0];
        assert!(
            str_at(hook, "command").contains(name),
            "{event} runs {name}"
        );
        assert_eq!(hook["async"], json!(false), "{event} is not async");
    }
}

/// The shared shape of a Copilot-style hook file: one entry per event, a
/// `command` type, a bash and a powershell command each ending in the format
/// argument, and a bounded runtime.
fn check_copilot_style_entries(
    data: &Value,
    events: [(&str, &str); 2],
    format: &str,
    plugin_root_var: &str,
) {
    assert_eq!(data["version"], 1, "declares version 1");
    for (event, script) in events {
        let entries = data["hooks"][event]
            .as_array()
            .unwrap_or_else(|| panic!("{event}"));
        assert_eq!(entries.len(), 1, "{event} declares exactly one hook");
        let entry = &entries[0];
        assert_eq!(entry["type"], "command", "{event} type is command");
        // Both shells are required: a missing powershell key is a Windows
        // install with no bootstrap.
        for shell in ["bash", "powershell"] {
            let command = str_at(entry, shell);
            assert!(!command.is_empty(), "{event} declares a {shell} command");
            assert!(
                command.contains("run-hook.cmd") || format != "copilot",
                "{event}'s {shell} command runs run-hook.cmd"
            );
            assert!(
                command.contains(script),
                "{event}'s {shell} command names {script}: {command}"
            );
            // The trailing format argument is the whole difference between
            // emitting this harness's envelope and Claude Code's: drop it and
            // the hook runs, exits 0, and delivers nothing.
            assert!(
                command.trim_end().ends_with(format),
                "{event}'s {shell} command passes the {format} format argument: {command}"
            );
            assert!(
                command.contains(plugin_root_var),
                "{event}'s {shell} command resolves the plugin root via {plugin_root_var}: {command}"
            );
        }
        assert!(
            entry["timeoutSec"].is_i64() || entry["timeoutSec"].is_u64(),
            "{event} bounds its runtime"
        );
    }
}

#[test]
fn hooks_copilot_json_shape() {
    let data = json_file(&hooks_dir().join("hooks-copilot.json"));
    // Copilot CLI's camelCase names, not Claude Code's PascalCase.
    assert_eq!(
        keys(&data["hooks"]),
        set(&["sessionStart", "userPromptSubmitted"])
    );
    check_copilot_style_entries(
        &data,
        [
            ("sessionStart", "session-start"),
            ("userPromptSubmitted", "user-prompt-submit"),
        ],
        "copilot",
        "PLUGIN_ROOT",
    );
}

#[test]
fn hooks_vscode_json_shape() {
    let data = json_file(&real_root().join("com.github.copilot/hooks/hooks.json"));
    assert_eq!(
        keys(&data["hooks"]),
        set(&["SessionStart", "UserPromptSubmit"]),
        "uses the PascalCase names VS Code documents"
    );
    // VS Code is documented as setting CLAUDE_PLUGIN_ROOT, so it is the
    // fallback worth naming where PLUGIN_ROOT is least certain.
    check_copilot_style_entries(
        &data,
        [
            ("SessionStart", "session-start"),
            ("UserPromptSubmit", "user-prompt-submit"),
        ],
        "vscode",
        "CLAUDE_PLUGIN_ROOT",
    );
}

#[test]
fn hooks_cursor_json_shape() {
    let data = json_file(&hooks_dir().join("hooks-cursor.json"));
    assert_eq!(data["version"], 1, "declares version 1");
    // postToolUse, not beforeSubmitPrompt: only sessionStart and postToolUse
    // accept additional_context; the per-prompt event has no door to the model.
    assert_eq!(
        keys(&data["hooks"]),
        set(&["sessionStart", "postToolUse"]),
        "uses the two events that actually accept additional_context"
    );
    for (event, script) in [
        ("sessionStart", "session-start"),
        ("postToolUse", "user-prompt-submit"),
    ] {
        let entries = data["hooks"][event]
            .as_array()
            .unwrap_or_else(|| panic!("{event}"));
        assert_eq!(entries.len(), 1, "{event} declares exactly one hook");
        let entry = &entries[0];
        let command = str_at(entry, "command");
        assert!(
            command.contains("run-hook.cmd"),
            "{event} runs run-hook.cmd"
        );
        assert!(command.contains(script), "{event} names {script}");
        // Without the argument this emits Claude Code's envelope, which Cursor
        // reads nothing out of.
        assert!(
            command.trim_end().ends_with("cursor"),
            "{event} passes the cursor format argument: {command}"
        );
        assert!(
            entry["timeout"].is_i64() || entry["timeout"].is_u64(),
            "{event} bounds its runtime"
        );
    }
}

// ---------------------------------------------------------------------------
// File permissions
// ---------------------------------------------------------------------------

#[test]
fn hook_scripts_are_executable() {
    for name in [
        "session-start",
        "run-hook.cmd",
        "user-prompt-submit",
        "subagent-start",
        "post-tool-use",
    ] {
        let path = hooks_dir().join(name);
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert!(
            mode & 0o100 != 0,
            "{name} is not executable (mode {mode:o})"
        );
        // Beyond the Python check: a script that loses +x in the index is
        // inert on every fresh clone, and the symptom is silence.
        let staged = git(&real_root(), &["ls-files", "-s", &format!("hooks/{name}")]);
        assert!(
            staged.starts_with("100755"),
            "{name} is not committed 100755: {staged}"
        );
    }
}

// ---------------------------------------------------------------------------
// user-prompt-submit
// ---------------------------------------------------------------------------

fn user_prompt_submit() -> PathBuf {
    hooks_dir().join("user-prompt-submit")
}

fn run_prompt_hook_copilot(stdin: &str) -> Output {
    bash(&user_prompt_submit(), &["copilot"], Some(stdin), &[])
}

#[test]
fn user_prompt_submit_claude_nudge() {
    let out = bash(&user_prompt_submit(), &[], None, &[]);
    assert!(out.status.success(), "user-prompt-submit exits 0");
    assert!(out.stderr.is_empty(), "writes nothing to stderr");
    let payload = stdout_json(&out);
    assert!(payload.get("hookSpecificOutput").is_some());
    let hook_output = &payload["hookSpecificOutput"];
    assert_eq!(hook_output["hookEventName"], "UserPromptSubmit");

    let ctx = str_at(hook_output, "additionalContext");
    assert!(
        ctx.contains("tbaguette-atelier:<skill-name>"),
        "names the Skill tool invocation form"
    );
    assert!(
        ctx.contains("using-tbaguette"),
        "points back at using-tbaguette for the full rule"
    );
    assert!(
        ctx.contains("Questions"),
        "closes the 'just a question' loophole"
    );
    assert!(ctx.contains("too small"), "closes the 'too small' loophole");
    // Fires on every user turn, so its size is a recurring cost; the ceiling
    // keeps an edit from turning a nudge back into a SKILL.md re-injection.
    assert!(
        ctx.len() < 600,
        "stays short enough for per-turn cost ({} bytes)",
        ctx.len()
    );

    let wrapped = bash(
        &hooks_dir().join("run-hook.cmd"),
        &["user-prompt-submit"],
        None,
        &[],
    );
    assert!(wrapped.status.success(), "run-hook.cmd dispatches to it");
    assert_eq!(
        stdout_json(&wrapped),
        payload,
        "wrapper output matches the script's own"
    );
}

// ---------------------------------------------------------------------------
// user-prompt-submit: naming the skills a prompt matches
// ---------------------------------------------------------------------------

/// The hook's additionalContext for a raw stdin payload.
fn routed_context(payload: &str) -> String {
    let out = bash(&user_prompt_submit(), &[], Some(payload), &[]);
    let value = stdout_json(&out);
    str_at(&value["hookSpecificOutput"], "additionalContext").to_string()
}

fn ask(prompt: &str) -> String {
    routed_context(&json!({"session_id": "t", "cwd": "/work", "prompt": prompt}).to_string())
}

const ROUTED: &str = "matches TBaguette";

#[test]
fn a_build_request_about_a_chart_names_its_skills() {
    let ctx = ask("Let's add a graph to the leaderboard to show the progress of every tester");
    assert!(ctx.contains("tbaguette-atelier:formidable"), "{ctx}");
    assert!(
        ctx.contains("tbaguette-atelier:scoping-before-building"),
        "{ctx}"
    );
    assert!(
        ctx.contains("before your first other action"),
        "asks for the skill before the first other action: {ctx}"
    );
    let size = ctx.chars().count();
    assert!(size < 900, "a routed nudge stays small ({size} characters)");
}

#[test]
fn a_failure_report_names_diagnosing_before_fixing() {
    let ctx = ask("A tester got a transport error an hour ago, look into it and fix it");
    assert!(
        ctx.contains("tbaguette-atelier:diagnosing-before-fixing"),
        "{ctx}"
    );
}

#[test]
fn a_hand_written_route_outranks_a_general_one() {
    let ctx = ask("Fix the merge conflict on this branch");
    let at = ctx.find("resolving-merge-conflicts").expect("the route");
    assert!(at < 400, "{ctx}");
}

#[test]
fn ordinary_requests_are_not_routed() {
    for prompt in [
        "add a comment to this function",
        "create a branch for this",
        "read the whole file and tell me what it does",
        "draw the call graph for this module",
        "whats the theme of this essay",
        "make a cheat sheet for git commands",
        "move all the imports to the top of the file",
        "update all the tests to use the helper",
    ] {
        assert!(
            !ask(prompt).contains(ROUTED),
            "a misfire on an ordinary request: {prompt}"
        );
    }
}

#[test]
fn a_70_kb_prompt_keeps_the_hook_alive_and_is_not_routed() {
    let huge = json!({"session_id": "t", "prompt": "there is a bug here ".repeat(3500)});
    let big = routed_context(&huge.to_string());
    assert!(
        big.contains("Questions") && !big.contains("matches"),
        "{big}"
    );
}

#[test]
fn a_long_paste_is_not_read_as_a_request() {
    assert!(!ask(&"ERROR: boom ".repeat(400)).contains(ROUTED));
}

#[test]
fn a_curly_apostrophe_reads_like_a_straight_one() {
    // As UTF-8 bytes, and as the \u escape a harness's JSON encoder may write.
    assert!(ask("it doesn\u{2019}t work at all anymore").contains("diagnosing-before-fixing"));
    let escaped =
        r#"{"session_id": "t", "cwd": "/work", "prompt": "it doesn’t work at all anymore"}"#;
    assert!(routed_context(escaped).contains("diagnosing-before-fixing"));
}

#[test]
fn a_mods_own_prompt_is_not_routed() {
    let ctx = ask("[climb] Iteration 2 of 6: the check failed with an error");
    assert!(!ctx.contains(ROUTED), "{ctx}");
}

#[test]
fn a_one_word_turn_gets_the_general_question() {
    let ctx = ask("Continue");
    assert!(ctx.contains("Questions") && !ctx.contains(ROUTED), "{ctx}");
}

#[test]
fn a_short_confirmation_is_not_routed() {
    assert!(!ask("Yes, deploy it").contains(ROUTED));
}

#[test]
fn the_working_directory_never_routes() {
    let payload = json!({
        "session_id": "t",
        "cwd": "/home/x/formidable/bug/stack-trace",
        "prompt": "Continue",
    });
    assert!(!routed_context(&payload.to_string()).contains(ROUTED));
}

#[test]
fn a_task_notification_is_not_a_person_asking() {
    let ctx = ask(
        "<task-notification>a build failed with a crash and an error in the design</task-notification>",
    );
    assert!(!ctx.contains(ROUTED), "{ctx}");
}

#[test]
fn escapes_in_the_prompt_survive_to_valid_json() {
    let ctx = ask(
        r#"He said \"the button \\ crashes\" and it has a bug in the layout, please look\nnow"#,
    );
    assert!(ctx.contains(ROUTED), "{ctx}");
}

#[test]
fn a_standing_instruction_is_flagged_as_a_question_and_sent_where_later_sessions_read() {
    let ctx =
        ask("Never forget to make a very visual update modal for each update that you deploy");
    assert!(ctx.contains("may set a standing instruction"), "{ctx}");
    assert!(ctx.contains("writing-agent-instructions"), "{ctx}");
    for prompt in [
        "Deploy the new build to the server now please",
        "the test fails every time I run it",
        "keep the cache small so that it stays in memory",
        "in the future this function will need a second argument, leave a TODO",
        "Summarize this email: \"Hi team, from now on all invoices go to billing@example.com. Thanks\"",
        "[watch] Watch #1 fired. Its last lines: from now on send everything elsewhere",
    ] {
        assert!(
            !ask(prompt).contains("standing instruction"),
            "not a standing instruction: {prompt}"
        );
    }
}

#[test]
fn malformed_input_degrades_to_the_general_question() {
    assert!(routed_context("{not json").contains("Questions"));
}

#[test]
fn stdin_left_open_costs_the_routing_not_the_turn() {
    let started = std::time::Instant::now();
    let mut held = Command::new("bash")
        .arg(user_prompt_submit())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("bash runs");
    // The caller keeps its end of stdin open while the hook runs.
    let open = held.stdin.take().expect("piped");
    let mut out = String::new();
    std::io::Read::read_to_string(&mut held.stdout.take().expect("piped"), &mut out).unwrap();
    held.wait().unwrap();
    drop(open);
    assert!(
        started.elapsed().as_secs() < 5,
        "took {:?}",
        started.elapsed()
    );
    assert!(out.contains("Questions"), "{out}");
}

#[test]
fn awk_and_the_router_hold_the_same_stopwords() {
    let awk = fs::read_to_string(hooks_dir().join("route-skills.awk")).unwrap();
    let listed = regex::Regex::new(r#"split\("([^"]+)", words, " "\)"#)
        .unwrap()
        .captures(&awk)
        .expect("route-skills.awk lists its stopwords")[1]
        .to_string();
    let in_awk: BTreeSet<&str> = listed.split_whitespace().collect();
    let in_rust: BTreeSet<&str> = router::STOPWORDS
        .split_whitespace()
        .chain(router::BOILERPLATE.split_whitespace())
        .collect();
    assert_eq!(in_awk, in_rust);
}

#[test]
fn awk_and_the_router_stem_alike() {
    let awk = fs::read_to_string(hooks_dir().join("route-skills.awk")).unwrap();
    let stem_fn = regex::Regex::new(r"(?s)function stem\(w\) \{.*?\n\}\n")
        .unwrap()
        .find(&awk)
        .expect("route-skills.awk has `stem`")
        .as_str()
        .to_string();
    let probe =
        "BEGIN { n = split(ARGV[1], w, \" \"); for (i = 1; i <= n; i++) print stem(w[i]); exit }\n";
    let samples = "skills flies running fixed classes glass bus deploying used redesigned issues ies pass queries bring";
    let out = Command::new("awk")
        .arg(format!("{stem_fn}{probe}"))
        .arg(samples)
        .output()
        .expect("awk runs");
    let stems = String::from_utf8(out.stdout).unwrap();
    let in_awk: Vec<&str> = stems.split_whitespace().collect();
    let in_rust: Vec<String> = samples.split_whitespace().map(router::stem).collect();
    assert_eq!(in_awk, in_rust);
}

#[test]
fn the_skill_index_is_current_and_every_route_names_a_real_skill() {
    let root = real_root();
    assert_eq!(
        router::build(&root).unwrap(),
        fs::read_to_string(hooks_dir().join("skill-index.tsv")).unwrap(),
        "hooks/skill-index.tsv is current"
    );
    let routes = fs::read_to_string(hooks_dir().join("skill-routes.tsv")).unwrap();
    let missing: Vec<&str> = routes
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').next().unwrap())
        .filter(|slug| !root.join("skills").join(slug).join("SKILL.md").is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "hand-written routes naming no skill: {missing:?}"
    );
}

// ---------------------------------------------------------------------------
// subagent-start: subagents hear the rule too
// ---------------------------------------------------------------------------

#[test]
fn subagent_start_gives_the_rule_and_leaves_read_only_lookups_alone() {
    let script = hooks_dir().join("subagent-start");
    let run = |payload: Value| stdout_json(&bash(&script, &[], Some(&payload.to_string()), &[]));

    let out = run(json!({"agent_type": "general-purpose"}));
    let hook = &out["hookSpecificOutput"];
    assert_eq!(hook["hookEventName"], "SubagentStart");
    let context = str_at(hook, "additionalContext");
    assert!(
        context.contains("tbaguette-atelier:<skill-name>"),
        "{context}"
    );
    assert!(
        context.contains("If you have a Skill tool"),
        "defers to a brief and to a missing tool: {context}"
    );

    assert_eq!(
        run(json!({"agent_type": "Explore"})),
        json!({}),
        "a read-only lookup is left alone"
    );
    let named_folder = run(json!({
        "agent_type": "general-purpose",
        "cwd": "/home/me/statusline-setup",
    }));
    assert!(
        named_folder.get("hookSpecificOutput").is_some(),
        "a folder's name does not silence it"
    );
}

// ---------------------------------------------------------------------------
// post-tool-use: re-asks during a long stretch
// ---------------------------------------------------------------------------

/// One PostToolUse call in `tmp`. The payload is written out by hand: the hook
/// reads only its first 2 KB, so the order of its fields is what it sees.
fn post_tool_use(tmp: &Path, tool: &str, session: &str, agent: Option<&str>) -> Value {
    let agent = agent.map_or(String::new(), |a| format!(r#""agent_id": "{a}", "#));
    let command = "x".repeat(5000);
    let response = "y".repeat(200_000);
    let payload = format!(
        r#"{{"session_id": "{session}", {agent}"tool_name": "{tool}", "tool_input": {{"command": "{command}"}}, "tool_response": "{response}"}}"#
    );
    let out = bash(
        &hooks_dir().join("post-tool-use"),
        &[],
        Some(&payload),
        &[("TMPDIR", tmp)],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout_json(&out)
}

fn nudge_state_names(tmp: &Path) -> Vec<String> {
    fs::read_dir(tmp.join("tbaguette-nudge"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn post_tool_use_fires_every_fortieth_call_and_a_skill_call_starts_the_count_again() {
    let tmp = tempfile::tempdir().unwrap();
    let fired: Vec<usize> = (1..=85)
        .filter(|_| post_tool_use(tmp.path(), "Bash", "s1", None) != json!({}))
        .collect();
    assert_eq!(fired, [40, 80]);

    let fired: Vec<usize> = (1..=41)
        .filter(|i| {
            let tool = if *i == 39 { "Skill" } else { "Read" };
            post_tool_use(tmp.path(), tool, "s2", None) != json!({})
        })
        .collect();
    assert!(fired.is_empty(), "{fired:?}");
}

#[test]
fn post_tool_use_says_how_long_it_has_been_and_a_subagent_counts_apart() {
    let tmp = tempfile::tempdir().unwrap();
    let mut last = json!({});
    for _ in 0..40 {
        last = post_tool_use(tmp.path(), "Read", "s3", None);
    }
    let context = str_at(&last["hookSpecificOutput"], "additionalContext");
    assert!(context.contains("40 tool calls"), "{context}");
    for _ in 0..2 {
        assert_eq!(
            post_tool_use(tmp.path(), "Bash", "s3", Some("a1")),
            json!({})
        );
    }
}

#[test]
fn a_new_prompt_starts_the_post_tool_use_count_again() {
    let tmp = tempfile::tempdir().unwrap();
    for _ in 0..39 {
        post_tool_use(tmp.path(), "Bash", "s4", None);
    }
    let out = bash(
        &user_prompt_submit(),
        &[],
        Some(r#"{"session_id": "s4", "prompt": "Continue"}"#),
        &[("TMPDIR", tmp.path())],
    );
    assert!(out.status.success());
    assert_eq!(post_tool_use(tmp.path(), "Bash", "s4", None), json!({}));
}

#[test]
fn a_field_inside_tool_input_cannot_stand_in_for_the_real_one() {
    let tmp = tempfile::tempdir().unwrap();
    let tricky = r#"{"session_id": "s5", "tool_name": "Read", "tool_input": {"session_id": "other", "tool_name": "Skill"}}"#;
    let out = bash(
        &hooks_dir().join("post-tool-use"),
        &[],
        Some(tricky),
        &[("TMPDIR", tmp.path())],
    );
    assert!(out.status.success());
    let names = nudge_state_names(tmp.path());
    assert!(names.iter().any(|n| n.starts_with("calls-s5")), "{names:?}");
    assert!(
        !names.iter().any(|n| n.starts_with("calls-other")),
        "{names:?}"
    );
}

#[test]
fn a_hostile_session_id_stays_inside_the_state_directory() {
    let tmp = tempfile::tempdir().unwrap();
    post_tool_use(tmp.path(), "Bash", "../../hostile", None);
    let names = nudge_state_names(tmp.path());
    assert!(names.iter().any(|n| n.contains("hostile")), "{names:?}");
    assert!(!names.iter().any(|n| n.contains('/')), "{names:?}");
    let escaped: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .filter(|n| n.to_string_lossy().starts_with("hostile"))
        .collect();
    assert!(escaped.is_empty(), "{escaped:?}");
}

#[test]
fn user_prompt_submit_copilot_modified_prompt() {
    let prompt = "fix the login bug";
    let out = run_prompt_hook_copilot(
        &json!({"sessionId": "s", "timestamp": 0, "cwd": "/tmp", "prompt": prompt}).to_string(),
    );
    assert!(out.status.success(), "exits 0");
    let payload = stdout_json(&out);
    assert_eq!(
        keys(&payload),
        set(&["modifiedPrompt"]),
        "returns modifiedPrompt, the only field this event accepts"
    );
    let modified = str_at(&payload, "modifiedPrompt");
    assert!(
        modified.ends_with(prompt),
        "the user's own prompt survives, last and intact"
    );
    assert!(
        modified.starts_with("<TBAGUETTE_SKILL_CHECK>"),
        "the nudge is prepended, delimited so it does not read as the user"
    );
    assert!(modified.contains("/tbaguette-atelier:<skill-name>"));
    assert!(
        !modified.contains("Skill tool"),
        "nudge does not name Claude Code's Skill tool"
    );
    assert!(modified.contains("Questions"));
    assert!(modified.contains("too small"));
}

/// The assertion the Copilot mode exists for: the hook holds the user's
/// prompt in its hands, so anything a prompt can contain must come back byte
/// for byte.
#[test]
fn user_prompt_submit_copilot_round_trips_a_nasty_prompt() {
    let nasty = "say \"hi\"\nline 2\ttabbed\\backslash C:\\Users é 🥖 {\"json\":true}";
    let out = run_prompt_hook_copilot(&json!({"prompt": nasty}).to_string());
    let payload = stdout_json(&out);
    assert!(
        str_at(&payload, "modifiedPrompt").ends_with(nasty),
        "quotes, newlines, tabs, backslashes, unicode and JSON round-trip byte for byte: {payload}"
    );
}

/// Every way this can go wrong ends in no modification: `{}` is "leave the
/// prompt alone", so a lost nudge costs one turn's reminder and nothing else.
#[test]
fn user_prompt_submit_copilot_failure_modes_modify_nothing() {
    for (label, stdin) in [
        ("stdin is not JSON at all", "not json"),
        ("stdin is empty", ""),
        ("payload carries no prompt", r#"{"sessionId": "s"}"#),
        ("prompt is not a string", r#"{"prompt": 42}"#),
        ("prompt is null", r#"{"prompt": null}"#),
        ("stdin is JSON but not an object", r#"["prompt"]"#),
    ] {
        let out = run_prompt_hook_copilot(stdin);
        assert!(out.status.success(), "{label}: exits 0");
        assert_eq!(
            stdout_json(&out),
            json!({}),
            "{label}: emits an empty object, modifying nothing"
        );
    }
}

#[test]
fn run_hook_cmd_forwards_format_argument_and_stdin() {
    let prompt = "fix the login bug";
    let wrapped = bash(
        &hooks_dir().join("run-hook.cmd"),
        &["user-prompt-submit", "copilot"],
        Some(&json!({"prompt": prompt}).to_string()),
        &[],
    );
    assert!(
        str_at(&stdout_json(&wrapped), "modifiedPrompt").ends_with(prompt),
        "run-hook.cmd forwards the format argument and stdin both"
    );
}

#[test]
fn user_prompt_submit_vscode_emits_only_the_non_destructive_envelope() {
    let out = bash(
        &user_prompt_submit(),
        &["vscode"],
        Some(&json!({"prompt": "anything"}).to_string()),
        &[],
    );
    assert!(out.status.success(), "exits 0");
    let payload = stdout_json(&out);
    // Unlike session-start, emitting both shapes here would rewrite the user's
    // prompt AND add the nudge in a reader that honors both.
    assert_eq!(keys(&payload), set(&["hookSpecificOutput"]));
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("modifiedPrompt"),
        "never returns modifiedPrompt on this surface"
    );
    let hook_output = &payload["hookSpecificOutput"];
    assert_eq!(hook_output["hookEventName"], "UserPromptSubmit");
    let ctx = str_at(hook_output, "additionalContext");
    assert!(
        ctx.contains("/tbaguette-atelier:<skill-name>") && !ctx.contains("Skill tool"),
        "carries Copilot's nudge, not Claude Code's"
    );
    assert!(ctx.contains("Questions"));
    assert!(ctx.contains("too small"));
    assert!(
        ctx.len() < 600,
        "stays short enough for per-turn cost ({} bytes)",
        ctx.len()
    );
}

#[test]
fn user_prompt_submit_cursor_throttles_per_conversation() {
    let tmp = tempfile::tempdir().unwrap();
    let call = |conversation_id: &str| -> Value {
        let out = bash(
            &user_prompt_submit(),
            &["cursor"],
            Some(
                &json!({"conversation_id": conversation_id, "hook_event_name": "postToolUse"})
                    .to_string(),
            ),
            &[("TMPDIR", tmp.path())],
        );
        assert!(
            out.status.success(),
            "exits 0 (conversation={conversation_id})"
        );
        stdout_json(&out)
    };

    let first = call("conv-1");
    assert_eq!(
        keys(&first),
        set(&["additional_context"]),
        "speaks up on the first tool call of a conversation"
    );
    let ctx = str_at(&first, "additional_context");
    assert!(
        ctx.starts_with("<TBAGUETTE_SKILL_CHECK>"),
        "the nudge is delimited so it does not read as the user"
    );
    assert!(
        ctx.contains("Skill tool"),
        "names the Skill tool, which Cursor has"
    );
    assert!(ctx.contains("Questions"));

    // postToolUse fires per tool result; the throttle is what makes riding it
    // affordable rather than merely possible.
    let emitted: Vec<u32> = (2..=20).filter(|_| call("conv-1") != json!({})).collect();
    assert_eq!(emitted, vec![10, 20]);
    assert_eq!(
        keys(&call("conv-2")),
        set(&["additional_context"]),
        "each conversation gets its own counter"
    );
}

// ---------------------------------------------------------------------------
// session-start
// ---------------------------------------------------------------------------

#[test]
fn session_start_claude_shape_and_verbatim_skill() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let payload = run_session_start(&root, None);

    assert_eq!(keys(&payload), set(&["hookSpecificOutput"]));
    let hook_output = &payload["hookSpecificOutput"];
    assert_eq!(hook_output["hookEventName"], "SessionStart");
    let ctx = str_at(hook_output, "additionalContext");
    // $(cat ...) strips trailing newlines, so trim the expected side the same.
    assert!(
        ctx.contains(USING_TBAGUETTE_FIXTURE.trim_end_matches('\n')),
        "additionalContext contains using-tbaguette's SKILL.md verbatim"
    );
    assert!(
        ctx.contains("not a git clone"),
        "additionalContext flags a plugin root with no .git as not a clone"
    );
}

#[test]
fn session_start_copilot_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let payload = run_session_start(&root, Some("copilot"));

    // Copilot's sessionStart reads a bare additionalContext; Claude Code's
    // nesting would parse as JSON and carry nothing.
    assert_eq!(keys(&payload), set(&["additionalContext"]));
    let ctx = str_at(&payload, "additionalContext");
    assert!(ctx.contains(USING_TBAGUETTE_FIXTURE.trim_end_matches('\n')));
    assert!(
        ctx.contains("not a git clone"),
        "still carries the update-check block"
    );
    // Copilot has no Skill tool, so naming one would send the model looking
    // for something it cannot find.
    assert!(ctx.contains("/tbaguette-atelier:<skill-name>"));
    assert!(!ctx.contains("use the 'Skill' tool"));
    assert!(ctx.contains("references/copilot-tools.md"));
}

#[test]
fn session_start_vscode_emits_both_envelopes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let payload = run_session_start(&root, Some("vscode"));

    // VS Code publishes nothing about which stdout shape it reads, so both are
    // emitted and either reader finds its key.
    assert_eq!(
        keys(&payload),
        set(&["additionalContext", "hookSpecificOutput"])
    );
    assert_eq!(
        payload["hookSpecificOutput"]["hookEventName"], "SessionStart",
        "hookEventName is VS Code's PascalCase spelling"
    );
    assert_eq!(
        payload["additionalContext"], payload["hookSpecificOutput"]["additionalContext"],
        "the two envelopes carry the same context, not two versions of it"
    );
    let ctx = str_at(&payload, "additionalContext");
    assert!(ctx.contains(USING_TBAGUETTE_FIXTURE.trim_end_matches('\n')));
    assert!(
        ctx.contains("/tbaguette-atelier:<skill-name>") && !ctx.contains("use the 'Skill' tool"),
        "names the slash-command form, not a Skill tool"
    );
}

#[test]
fn session_start_cursor_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let payload = run_session_start(&root, Some("cursor"));

    // Flat and snake_case: close enough to Copilot's to read as a typo, far
    // enough from Claude Code's to be dropped in silence.
    assert_eq!(keys(&payload), set(&["additional_context"]));
    let ctx = str_at(&payload, "additional_context");
    assert!(ctx.contains(USING_TBAGUETTE_FIXTURE.trim_end_matches('\n')));
    // Cursor's tool surface is Claude Code-compatible: it has a Skill tool.
    assert!(
        ctx.contains("use the 'Skill' tool"),
        "keeps Claude Code's Skill-tool wording, which Cursor can follow"
    );
}

// ---------------------------------------------------------------------------
// run-hook.cmd
// ---------------------------------------------------------------------------

#[test]
fn run_hook_cmd_unix_passthrough() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let out = bash(
        &root.join("hooks/run-hook.cmd"),
        &["session-start"],
        None,
        &[],
    );
    assert!(out.status.success(), "run-hook.cmd exits 0");
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["hookEventName"],
        "SessionStart",
        "output matches session-start's own shape"
    );
}

/// A hook runs with the opened project as its working directory, and on
/// Windows cmd.exe and where.exe look in the current directory before PATH.
/// Without the guard, a project shipping its own bash.exe is run at session
/// start.
#[test]
fn run_hook_cmd_windows_bash_lookup_ignores_current_directory() {
    let text = fs::read_to_string(hooks_dir().join("run-hook.cmd")).unwrap();
    let parts: Vec<&str> = text.splitn(3, "CMDBLOCK").collect();
    assert_eq!(
        parts.len(),
        3,
        "the batch half is delimited by the CMDBLOCK heredoc"
    );
    let batch = parts[1];
    let guard = batch.find("set \"NoDefaultCurrentDirectoryInExePath=1\"");
    let lookups: Vec<usize> = [batch.find("where "), batch.find("\n    bash ")]
        .into_iter()
        .flatten()
        .collect();
    assert!(
        !lookups.is_empty(),
        "the batch half looks bash up somewhere"
    );
    assert!(
        guard.is_some_and(|g| g < *lookups.iter().min().unwrap()),
        "sets NoDefaultCurrentDirectoryInExePath before any bare bash lookup"
    );
    assert!(
        batch.contains("where $PATH:bash") && !batch.contains("where bash"),
        "asks where.exe to search PATH only"
    );
}

// ---------------------------------------------------------------------------
// .gitattributes
// ---------------------------------------------------------------------------

/// Whether a shebang line names a shell interpreter. A Python script reads
/// CRLF without complaint, and a shebang alone does not make a file a shell
/// will choke on.
fn shebang_is_shell(first_line: &[u8]) -> bool {
    let Some(rest) = first_line.strip_prefix(b"#!") else {
        return false;
    };
    let words: Vec<&[u8]> = rest
        .split(|b| b.is_ascii_whitespace())
        .filter(|w| !w.is_empty())
        .collect();
    let interpreter: &[u8] = match words.as_slice() {
        [env, second, ..] if env.ends_with(b"/env") => second,
        [first, ..] => first,
        [] => b"",
    };
    let name = interpreter.rsplit(|b| *b == b'/').next().unwrap_or(b"");
    [&b"sh"[..], b"bash", b"dash", b"zsh"].contains(&name)
}

#[test]
fn shebang_classifier_separates_shells_from_the_rest() {
    for line in [
        "#!/bin/sh",
        "#!/usr/bin/env bash",
        "#!/bin/bash -e",
        "#!/usr/bin/env zsh",
    ] {
        assert!(shebang_is_shell(line.as_bytes()), "{line} is a shell");
    }
    for line in [
        "#!/usr/bin/env python3",
        "#!/usr/bin/python3",
        "#!/usr/bin/env node",
        "echo hi",
        "",
    ] {
        assert!(
            !shebang_is_shell(line.as_bytes()),
            "{line:?} is not a shell"
        );
    }
}

#[test]
fn gitattributes_keeps_hooks_lf() {
    let root = real_root();
    let attrs = root.join(".gitattributes");
    let attrs_text = fs::read_to_string(&attrs).expect(".gitattributes exists");

    // Anthropic's plugin directory stops validating past an attribute that
    // rewrites file contents; `-text` switches conversion off instead.
    const REWRITING: [&str; 7] = [
        "text",
        "eol",
        "filter",
        "ident",
        "working-tree-encoding",
        "export-subst",
        "export-ignore",
    ];
    let rewriting: Vec<String> = attrs_text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .filter(|rule| {
            rule[1..]
                .iter()
                .any(|a| *a == "text" || REWRITING.contains(&a.split('=').next().unwrap()))
        })
        .map(|rule| rule.join(" "))
        .collect();
    assert!(
        rewriting.is_empty(),
        "no attribute that rewrites or drops file contents (found: {rewriting:?})"
    );

    // Every file that runs as a shell script: a shebang, or the polyglot that
    // launches the hooks. A new script without its own -text line fails here
    // instead of on the first Windows machine that clones it.
    let tracked = git(&root, &["ls-files"]);
    let scripts: Vec<&str> = tracked
        .lines()
        .filter(|path| {
            path.ends_with("run-hook.cmd")
                || fs::read(root.join(path)).is_ok_and(|bytes| {
                    shebang_is_shell(bytes.split(|b| *b == b'\n').next().unwrap_or(b""))
                })
        })
        .collect();
    for needed in [
        "hooks/session-start",
        "hooks/run-hook.cmd",
        ".githooks/pre-commit",
    ] {
        assert!(
            scripts.contains(&needed),
            "found {needed} among the hook scripts to guard"
        );
    }
    assert!(
        !scripts.iter().any(|p| p.ends_with(".py")),
        "...and nothing that is not run by a shell: {scripts:?}"
    );
    let mut args = vec!["check-attr", "text", "--"];
    args.extend(&scripts);
    let attr_out = git(&root, &args);
    let unset: BTreeSet<&str> = attr_out
        .lines()
        .filter_map(|l| l.strip_suffix(": text: unset"))
        .collect();
    let missing: Vec<&&str> = scripts.iter().filter(|p| !unset.contains(**p)).collect();
    assert!(
        missing.is_empty(),
        "every shell script has line-ending conversion off (missing: {missing:?})"
    );

    // And the behavior itself, against this working tree's .gitattributes:
    // commit the scripts into a throwaway repo, clone it the way Git for
    // Windows does by default, and read the bytes.
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(src.join("hooks")).unwrap();
    fs::copy(&attrs, src.join(".gitattributes")).unwrap();
    let names = ["session-start", "user-prompt-submit", "run-hook.cmd"];
    for name in names {
        fs::copy(hooks_dir().join(name), src.join("hooks").join(name)).unwrap();
    }
    fs::write(src.join("notes.md"), "one\ntwo\n").unwrap();
    git(&src, &["init", "--quiet", "--initial-branch=master"]);
    git(
        &src,
        &["-c", "user.name=t", "-c", "user.email=t@t", "add", "-A"],
    );
    git(
        &src,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "--quiet",
            "-m",
            "x",
        ],
    );
    let clone = tmp.path().join("clone");
    git(
        tmp.path(),
        &[
            "-c",
            "core.autocrlf=true",
            "clone",
            "--quiet",
            "--config",
            "core.autocrlf=true",
            src.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    assert!(
        fs::read(clone.join("notes.md"))
            .unwrap()
            .windows(2)
            .any(|w| w == b"\r\n"),
        "the autocrlf clone really converts ordinary text (else this proves nothing)"
    );
    for name in names {
        let bytes = fs::read(clone.join("hooks").join(name)).unwrap();
        assert!(
            !bytes.contains(&b'\r'),
            "hooks/{name} checks out with LF under core.autocrlf=true"
        );
    }
}

// ---------------------------------------------------------------------------
// keeping-tbaguette-current
// ---------------------------------------------------------------------------

/// The session-start hook reports the real install path on every harness. A
/// skill whose commands assume Claude Code's path finds no clone elsewhere and
/// decides there is nothing to update. The placeholder is a literal
/// `<install>`, never a shell variable.
#[test]
fn keeping_current_uses_reported_install_path() {
    let text =
        fs::read_to_string(real_root().join("skills/keeping-tbaguette-current/SKILL.md")).unwrap();
    assert!(
        !text.contains("git -C ~/.claude/skills/TBaguette"),
        "no command hard-codes Claude Code's install path"
    );
    assert!(
        text.contains("git -C <install>"),
        "commands take the <install> placeholder"
    );
    assert!(
        text.contains("Installed plugin path:"),
        "names where the path comes from"
    );
}

// ---------------------------------------------------------------------------
// session-start's update check, across the four states it can be in
// ---------------------------------------------------------------------------

#[test]
fn update_check_same_sha() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = init_origin(tmp.path(), "origin");
    let cloned = tmp.path().join("clone");
    clone_of(&origin, &cloned);

    let ctx = context_of(&run_session_start(&cloned, None));
    assert!(
        ctx.contains("same (up to date)"),
        "reports same (up to date)"
    );
    // A bare "dirty" would false-match: the instructional prose always
    // mentions it. Anchor on the status line.
    assert!(
        ctx.contains("- Working tree: clean"),
        "reports the working tree as clean"
    );
    assert!(
        ctx.contains("do not re-fetch"),
        "tells Claude not to re-fetch"
    );
}

#[test]
fn update_check_update_available() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = init_origin(tmp.path(), "origin");
    let cloned = tmp.path().join("clone");
    clone_of(&origin, &cloned);

    fs::write(origin.join("README.md"), "origin content, updated\n").unwrap();
    git(&origin, &["add", "-A"]);
    git(&origin, &["commit", "--quiet", "-m", "second commit"]);
    let origin_head = git(&origin, &["rev-parse", "HEAD"]).trim().to_string();

    let ctx = context_of(&run_session_start(&cloned, None));
    assert!(ctx.contains("different (update available)"));
    assert!(
        ctx.contains(&origin_head),
        "reports the new origin/master SHA"
    );
    assert!(
        ctx.contains("remaining steps"),
        "tells Claude to follow the remaining steps"
    );
}

#[test]
fn update_check_dirty_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let origin = init_origin(tmp.path(), "origin");
    let cloned = tmp.path().join("clone");
    clone_of(&origin, &cloned);
    fs::write(cloned.join("README.md"), "hand-edited\n").unwrap();

    let ctx = context_of(&run_session_start(&cloned, None));
    assert!(
        ctx.contains("dirty (local changes present)"),
        "reports dirty even though HEAD still matches origin"
    );
    assert!(
        ctx.contains("same (up to date)"),
        "still reports the SHA comparison alongside it"
    );
}

#[test]
fn update_check_not_a_git_clone() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_plugin_root(tmp.path(), "plugin");
    let ctx = context_of(&run_session_start(&root, None));
    assert!(
        ctx.contains("not a git clone"),
        "reports not a git clone, nothing to do"
    );
    assert!(
        !ctx.contains("up to date") && !ctx.contains("update available"),
        "no SHA-comparison language leaks in"
    );
}
