//! The harness-manifest gates (scripts/test_harness_manifests.py): each holds
//! on the real repo, and each names its own problem after one breakage.

mod common;

use common::{breaks, passes};
use serde_json::json;

#[test]
fn manifest_json_holds() {
    passes("manifest-json");
}

#[test]
fn manifest_json_names_a_malformed_manifest() {
    breaks("manifest-json", ".kimi-plugin/plugin.json", |c| {
        c.write(".kimi-plugin/plugin.json", "{ not json");
    });
}

#[test]
fn manifest_json_names_a_missing_manifest() {
    breaks("manifest-json", "hooks/hooks-cursor.json missing", |c| {
        c.remove("hooks/hooks-cursor.json");
    });
}

#[test]
fn manifest_versions_hold() {
    passes("manifest-versions");
}

#[test]
fn manifest_versions_names_a_stale_manifest() {
    breaks(
        "manifest-versions",
        ".cursor-plugin/plugin.json: version 0.0.1",
        |c| {
            c.edit_json(".cursor-plugin/plugin.json", |v| {
                v["version"] = json!("0.0.1")
            });
        },
    );
}

#[test]
fn manifest_versions_names_the_marketplace_plugin_entry() {
    breaks(
        "manifest-versions",
        ".claude-plugin/marketplace.json (plugins[0])",
        |c| {
            c.edit_json(".claude-plugin/marketplace.json", |v| {
                v["plugins"][0]["version"] = json!("0.0.1");
            });
        },
    );
}

#[test]
fn manifest_versions_names_the_hermes_yaml() {
    breaks("manifest-versions", "plugin.yaml: version 0.0.1", |c| {
        let text = c.read("plugin.yaml");
        let stale: String = text
            .lines()
            .map(|l| {
                if l.starts_with("version:") {
                    "version: 0.0.1".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        c.write("plugin.yaml", &stale);
    });
}

#[test]
fn manifest_agents_md_symlink_holds() {
    passes("manifest-agents-md-symlink");
}

#[test]
fn manifest_agents_md_symlink_names_a_plain_file() {
    breaks(
        "manifest-agents-md-symlink",
        "AGENTS.md must be a symlink",
        |c| {
            c.remove("AGENTS.md");
            c.write("AGENTS.md", "a copy that will drift");
        },
    );
}

#[test]
fn manifest_copilot_hooks_field_holds() {
    passes("manifest-copilot-hooks-field");
}

#[test]
fn manifest_copilot_hooks_field_names_the_wrong_hook_file() {
    breaks(
        "manifest-copilot-hooks-field",
        "plugin.json: hooks is",
        |c| {
            c.edit_json("plugin.json", |v| v["hooks"] = json!("./hooks/hooks.json"));
        },
    );
}

#[test]
fn manifest_copilot_root_holds() {
    passes("manifest-copilot-root");
}

#[test]
fn manifest_copilot_root_names_a_tidier_looking_directory() {
    breaks(
        "manifest-copilot-root",
        ".github/plugin/ is not read by both",
        |c| {
            c.write(".github/plugin/plugin.json", "{}");
        },
    );
}

#[test]
fn manifest_hermes_layout_holds() {
    passes("manifest-hermes-layout");
}

#[test]
fn manifest_hermes_layout_names_the_invisible_directory() {
    breaks(
        "manifest-hermes-layout",
        ".hermes-plugin/ is invisible to Hermes",
        |c| {
            c.write(".hermes-plugin/plugin.yaml", "name: x");
        },
    );
}

#[test]
fn manifest_names_hold() {
    passes("manifest-names");
}

#[test]
fn manifest_names_names_the_odd_one_out() {
    breaks(
        "manifest-names",
        "gemini-extension.json: name \"Other\"",
        |c| {
            c.edit_json("gemini-extension.json", |v| v["name"] = json!("Other"));
        },
    );
}

#[test]
fn manifest_agent_plugins_schema_holds() {
    passes("manifest-agent-plugins-schema");
}

#[test]
fn manifest_agent_plugins_schema_names_a_dropped_schema_line() {
    breaks("manifest-agent-plugins-schema", "$schema", |c| {
        c.edit_json("plugin.json", |v| {
            v.as_object_mut().unwrap().remove("$schema");
        });
    });
}

#[test]
fn manifest_copilot_hook_casing_holds() {
    passes("manifest-copilot-hook-casing");
}

#[test]
fn manifest_copilot_hook_casing_names_a_swapped_file() {
    breaks(
        "manifest-copilot-hook-casing",
        "com.github.copilot/hooks/hooks.json",
        |c| {
            let text = c.read("com.github.copilot/hooks/hooks.json");
            c.write(
                "com.github.copilot/hooks/hooks.json",
                &text.replace("SessionStart", "sessionStart"),
            );
        },
    );
}

#[test]
fn manifest_copilot_install_spec_holds() {
    passes("manifest-copilot-install-spec");
}

#[test]
fn manifest_copilot_install_spec_names_an_unpublished_command() {
    breaks("manifest-copilot-install-spec", "PORTING.md", |c| {
        c.replace(
            "PORTING.md",
            "copilot plugin install tbaguette-atelier@tbaguette-dev",
            "copilot plugin install somewhere-else",
        );
    });
}

#[test]
fn manifest_codex_hooks_hold() {
    passes("manifest-codex-hooks");
}

#[test]
fn manifest_codex_hooks_names_the_emptied_field() {
    breaks(
        "manifest-codex-hooks",
        ".codex-plugin/plugin.json: hooks is",
        |c| {
            c.edit_json(".codex-plugin/plugin.json", |v| v["hooks"] = json!({}));
        },
    );
}

#[test]
fn manifest_hook_scripts_hold() {
    passes("manifest-hook-scripts");
}

#[test]
fn manifest_hook_scripts_expect_four_events_on_claude_code() {
    for event in ["SubagentStart", "PostToolUse"] {
        breaks(
            "manifest-hook-scripts",
            "hooks/hooks.json: events are",
            |c| {
                c.edit_json("hooks/hooks.json", |v| {
                    v["hooks"].as_object_mut().unwrap().remove(event);
                });
            },
        );
    }
}

#[test]
fn manifest_hook_scripts_names_a_missing_script() {
    breaks(
        "manifest-hook-scripts",
        "names hooks/session-start, which does not exist",
        |c| {
            c.remove("hooks/session-start");
        },
    );
}

#[test]
fn manifest_package_files_hold() {
    passes("manifest-package-files");
}

#[test]
fn manifest_package_files_names_a_missing_entry_point() {
    breaks("manifest-package-files", "does not exist", |c| {
        let main =
            serde_json::from_str::<serde_json::Value>(&c.read("package.json")).unwrap()["main"]
                .as_str()
                .unwrap()
                .to_string();
        c.remove(&main);
    });
}

#[test]
fn manifest_shared_fields_hold() {
    passes("manifest-shared-fields");
}

#[test]
fn manifest_shared_fields_names_a_drifted_keyword_list() {
    breaks(
        "manifest-shared-fields",
        ".cursor-plugin/plugin.json: keywords",
        |c| {
            c.edit_json(".cursor-plugin/plugin.json", |v| {
                v["keywords"] = json!(["skills"])
            });
        },
    );
}
