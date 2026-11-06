//! The agents tool and gates (scripts/copilot_agents.py,
//! scripts/test_copilot_agents.py).

mod common;

use std::process::Command;

use common::{Copy, breaks, passes};

const REVIEWER_SOURCE: &str = "scripts/copilot_agents/reviewer.md";
const IMPLEMENTER_PROMPT: &str =
    "skills/delegating-tasks-with-review-gates/reference/implementer-prompt.md";

fn agents(root: &std::path::Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_atelier-build"))
        .arg("agents")
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("atelier-build runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn agents_fresh_holds() {
    passes("agents-fresh");
}

#[test]
fn agents_check_passes_on_the_real_repo() {
    let (code, out) = agents(&common::real_root(), &["--check"]);
    assert_eq!((code, out.trim()), (0, "agents up to date"));
}

#[test]
fn agents_fresh_names_a_hand_edited_claude_agent() {
    breaks("agents-fresh", "stale: agents/reviewer.md", |c| {
        let text = c.read("agents/reviewer.md");
        c.write("agents/reviewer.md", &format!("{text}hand edit\n"));
    });
}

#[test]
fn agents_fresh_names_a_hand_edited_copilot_agent() {
    breaks(
        "agents-fresh",
        "stale: com.github.copilot/agents/investigator.agent.md",
        |c| {
            c.replace(
                "com.github.copilot/agents/investigator.agent.md",
                "investigator",
                "inspector",
            );
        },
    );
}

#[test]
fn agents_fresh_names_an_agent_with_no_source() {
    breaks("agents-fresh", "no source: agents/extra.md", |c| {
        c.write("agents/extra.md", "---\nname: extra\n---\n");
    });
}

#[test]
fn agents_fresh_names_a_template_that_changed_shape() {
    breaks(
        "agents-fresh",
        "'## You Do Not Dispatch Subagents' not found",
        |c| {
            c.replace_all(
                IMPLEMENTER_PROMPT,
                "## You Do Not Dispatch Subagents",
                "## Renamed",
            );
        },
    );
}

#[test]
fn agents_fresh_names_a_template_that_changed_under_an_agent() {
    // The skill's own dispatch text moved; the committed agent still carries the old.
    breaks("agents-fresh", "stale: agents/implementer.md", |c| {
        c.replace(
            IMPLEMENTER_PROMPT,
            "## Report Format",
            "## Report Format\n\n    Always include a haiku.",
        );
    });
}

#[test]
fn agents_check_exits_one_naming_the_stale_file() {
    let copy = Copy::new();
    copy.write("agents/reviewer.md", "old");
    let (code, out) = agents(copy.root(), &["--check"]);
    assert_eq!(code, 1);
    assert!(out.contains("stale: agents/reviewer.md"), "{out}");
}

#[test]
fn agents_writes_files_byte_identical_to_the_committed_ones() {
    let copy = Copy::new();
    for name in [
        "agents/reviewer.md",
        "com.github.copilot/agents/implementer.agent.md",
    ] {
        copy.write(name, "scrambled");
    }
    let (code, out) = agents(copy.root(), &[]);
    assert_eq!(code, 0, "{out}");
    for name in [
        "agents/implementer.md",
        "agents/investigator.md",
        "agents/reviewer.md",
        "com.github.copilot/agents/implementer.agent.md",
        "com.github.copilot/agents/investigator.agent.md",
        "com.github.copilot/agents/reviewer.agent.md",
    ] {
        let committed = std::fs::read(common::real_root().join(name)).unwrap();
        assert_eq!(
            std::fs::read(copy.root().join(name)).unwrap(),
            committed,
            "{name}"
        );
    }
}

#[test]
fn agents_copilot_profiles_hold() {
    passes("agents-copilot-profiles");
}

#[test]
fn agents_copilot_profiles_names_a_reviewer_that_can_edit() {
    breaks(
        "agents-copilot-profiles",
        "reviewer must not be able to edit",
        |c| {
            c.replace(
                REVIEWER_SOURCE,
                "tools: [\"execute\", \"read\", \"search\"]",
                "tools: [\"execute\", \"read\", \"search\", \"edit\"]",
            );
        },
    );
}

#[test]
fn agents_copilot_profiles_names_a_pinned_model() {
    breaks(
        "agents-copilot-profiles",
        "reviewer.agent.md: pins a model",
        |c| {
            c.replace(
                REVIEWER_SOURCE,
                "name: reviewer\n",
                "name: reviewer\nmodel: some-model\n",
            );
        },
    );
}

#[test]
fn agents_claude_profiles_hold() {
    passes("agents-claude-profiles");
}

#[test]
fn agents_claude_profiles_names_a_reviewer_that_can_edit() {
    breaks(
        "agents-claude-profiles",
        "reviewer must not have Edit",
        |c| {
            c.replace(
                REVIEWER_SOURCE,
                "tools: [\"execute\", \"read\", \"search\"]",
                "tools: [\"execute\", \"read\", \"search\", \"edit\"]",
            );
        },
    );
}

#[test]
fn agents_claude_profiles_names_an_unknown_tool_category() {
    breaks(
        "agents-claude-profiles",
        "no Claude Code tools for [\"telepathy\"]",
        |c| {
            c.replace(REVIEWER_SOURCE, "\"search\"", "\"telepathy\"");
        },
    );
}

#[test]
fn every_claude_code_agent_can_use_the_skills() {
    for role in ["implementer", "reviewer", "investigator"] {
        let text =
            std::fs::read_to_string(common::real_root().join(format!("agents/{role}.md"))).unwrap();
        let tools = text
            .lines()
            .find_map(|line| line.strip_prefix("tools: "))
            .unwrap_or_else(|| panic!("{role} lists no tools"));
        assert!(tools.split(", ").any(|t| t == "Skill"), "{role}: {tools}");
    }
}
