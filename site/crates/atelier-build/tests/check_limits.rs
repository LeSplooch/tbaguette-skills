//! The plugin-directory rules (scripts/test_directory_readiness.py) and the
//! Codex size report (scripts/run_tests.py).

mod common;

use atelier_build::check::{self, Repo};
use common::{Copy, breaks, passes, real_root};

/// Deterministic bytes that gzip cannot shrink.
fn incompressible(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

// ---- layout that stops validation ----

#[test]
fn limits_manifest_root_holds() {
    passes("limits-manifest-root");
}

#[test]
fn limits_manifest_root_names_a_missing_manifest() {
    breaks("limits-manifest-root", ".claude-plugin/plugin.json", |c| {
        c.remove(".claude-plugin/plugin.json");
    });
}

#[test]
fn limits_five_mib_holds() {
    passes("limits-five-mib");
}

#[test]
fn limits_five_mib_names_the_big_file() {
    breaks("limits-five-mib", "huge.dat", |c| {
        c.write_bytes("huge.dat", &incompressible(5 * 1024 * 1024, 7));
    });
}

#[test]
fn limits_entry_count_holds() {
    passes("limits-entry-count");
}

#[test]
fn limits_entry_count_names_the_ten_thousand_ceiling() {
    breaks("limits-entry-count", "entries, past 10000", |c| {
        for i in 0..10_000 {
            c.write(&format!("many/f{i}.txt"), "x");
        }
    });
}

#[test]
fn limits_archive_size_holds() {
    passes("limits-archive-size");
}

#[test]
fn limits_archive_size_names_a_fifty_mib_archive() {
    breaks("limits-archive-size", "past 50 MiB", |c| {
        for i in 0..11u64 {
            c.write_bytes(&format!("bulk/{i}.dat"), &incompressible(4_900_000, i + 3));
        }
        c.commit();
    });
}

#[test]
fn limits_portable_names_hold() {
    passes("limits-portable-names");
}

#[test]
fn limits_portable_names_names_a_windows_device_name() {
    breaks("limits-portable-names", "docs/nul.txt", |c| {
        c.write("docs/nul.txt", "x");
    });
}

#[test]
fn limits_portable_names_names_a_forbidden_character() {
    breaks("limits-portable-names", "what?.md", |c| {
        c.write("what?.md", "x");
    });
}

#[test]
fn limits_case_clash_holds() {
    passes("limits-case-clash");
}

#[test]
fn limits_case_clash_names_both_spellings() {
    breaks("limits-case-clash", "Readme.md", |c| {
        c.write("Readme.md", "x");
        c.write("README.MD", "x");
        c.write("readme.md", "x");
    });
}

#[test]
fn limits_gitattributes_holds() {
    passes("limits-gitattributes");
}

#[test]
fn limits_gitattributes_names_an_export_ignore_line() {
    breaks(
        "limits-gitattributes",
        ".gitattributes: *.md export-ignore",
        |c| {
            c.write(".gitattributes", "*.md export-ignore\n");
        },
    );
}

// ---- what blocks a submission ----

#[test]
fn limits_plugin_name_holds() {
    passes("limits-plugin-name");
}

#[test]
fn limits_plugin_name_names_a_bad_name() {
    breaks(
        "limits-plugin-name",
        "\"Bad_Name\" is not lowercase ASCII kebab-case",
        |c| {
            c.edit_json(".claude-plugin/plugin.json", |v| {
                v["name"] = "Bad_Name".into()
            });
        },
    );
}

#[test]
fn limits_reserved_name_holds() {
    passes("limits-reserved-name");
}

#[test]
fn limits_reserved_name_names_a_reserved_word() {
    breaks("limits-reserved-name", "\"mcp\" is a reserved word", |c| {
        c.edit_json(".claude-plugin/plugin.json", |v| v["name"] = "mcp".into());
    });
}

#[test]
fn limits_ascii_names_hold() {
    passes("limits-ascii-names");
}

#[test]
fn limits_ascii_names_names_a_non_ascii_author() {
    breaks("limits-ascii-names", "author.name", |c| {
        c.edit_json(".claude-plugin/plugin.json", |v| {
            v["author"]["name"] = "T\u{e9}baguette".into()
        });
    });
}

#[test]
fn limits_required_fields_hold() {
    passes("limits-required-fields");
}

#[test]
fn limits_required_fields_names_the_missing_field() {
    breaks("limits-required-fields", "description is not set", |c| {
        c.edit_json(".claude-plugin/plugin.json", |v| {
            v.as_object_mut().unwrap().remove("description");
        });
    });
}

#[test]
fn limits_licence_holds() {
    passes("limits-licence");
}

#[test]
fn limits_licence_names_a_plugin_with_none() {
    breaks("limits-licence", "no licence", |c| {
        c.remove("LICENSE");
        c.edit_json(".claude-plugin/plugin.json", |v| {
            v.as_object_mut().unwrap().remove("license");
        });
    });
}

#[test]
fn limits_readme_holds() {
    passes("limits-readme");
}

#[test]
fn limits_readme_names_a_thin_readme() {
    breaks("limits-readme", "README.md has 6 words", |c| {
        c.write("README.md", "# Tiny\n\nfive words and no more\n\n```\nlots of code that does not count at all here\n```\n");
    });
}

#[test]
fn limits_litter_holds() {
    passes("limits-litter");
}

#[test]
fn limits_litter_names_the_stray_file() {
    breaks("limits-litter", "skills/.DS_Store", |c| {
        c.write("skills/.DS_Store", "x");
    });
}

#[test]
fn limits_hook_events_hold() {
    passes("limits-hook-events");
}

#[test]
fn limits_hook_events_names_an_unknown_event() {
    breaks("limits-hook-events", "InventedEvent", |c| {
        c.edit_json("hooks/hooks.json", |v| {
            v["hooks"]["InventedEvent"] = serde_json::json!([]);
        });
    });
}

#[test]
fn limits_package_launchers_hold() {
    passes("limits-package-launchers");
}

#[test]
fn limits_package_launchers_names_a_hook_that_runs_npx() {
    breaks(
        "limits-package-launchers",
        "hooks/session-start runs a package launcher (npx)",
        |c| {
            let text = c.read("hooks/session-start");
            c.write("hooks/session-start", &format!("{text}\nnpx some-tool\n"));
        },
    );
}

#[test]
fn limits_package_configs_hold() {
    passes("limits-package-configs");
}

#[test]
fn limits_package_configs_names_an_npmrc() {
    breaks("limits-package-configs", ".npmrc", |c| {
        c.write(".npmrc", "registry=https://example.invalid\n");
    });
}

// ---- what holds every update for a reviewer ----

#[test]
fn limits_text_256kib_holds() {
    passes("limits-text-256kib");
}

#[test]
fn limits_text_256kib_names_the_big_text_file() {
    breaks("limits-text-256kib", "notes.txt", |c| {
        c.write("notes.txt", &"word ".repeat(60_000));
    });
}

#[test]
fn limits_text_256kib_exempts_images() {
    let copy = Copy::new();
    copy.write_bytes("picture.png", &incompressible(300_000, 5));
    assert_eq!(copy.problems("limits-text-256kib"), Vec::<String>::new());
}

#[test]
fn limits_file_count_holds() {
    passes("limits-file-count");
}

#[test]
fn limits_file_count_names_the_ceiling() {
    breaks("limits-file-count", "files; past 512", |c| {
        for i in 0..300 {
            c.write(&format!("extra/{i}.txt"), "x");
        }
    });
}

#[test]
fn limits_binaries_hold() {
    passes("limits-binaries");
}

#[test]
fn limits_binaries_names_a_compiled_file() {
    breaks("limits-binaries", "tool.bin", |c| {
        c.write_bytes("tool.bin", b"\x7fELF\0\0\0\0");
    });
}

/// The ruling of 2026-11-06 names two files; anything else stays a failure.
#[test]
fn only_the_two_named_wasm_modules_pass_the_binary_and_size_gates() {
    let big_module = {
        let mut bytes = b"\0asm\x01\0\0\0".to_vec();
        bytes.extend(incompressible(600_000, 9));
        bytes
    };
    let copy = Copy::new();
    copy.write_bytes("docs/assets/atelier_web_bg.wasm", &big_module);
    copy.write_bytes("docs/assets/atelier_graph_bg.wasm", &big_module);
    assert_eq!(copy.problems("limits-binaries"), Vec::<String>::new());
    assert_eq!(copy.problems("limits-text-256kib"), Vec::<String>::new());

    for other in [
        "docs/assets/third_bg.wasm",
        "docs/assets/nested/atelier_web_bg.wasm",
        "atelier_web_bg.wasm",
        "docs/assets/ATELIER_WEB_BG.WASM",
    ] {
        let copy = Copy::new();
        copy.write_bytes(other, &big_module);
        for gate in ["limits-binaries", "limits-text-256kib"] {
            assert!(
                copy.problems(gate).iter().any(|p| p.contains(other)),
                "{gate} should name {other}"
            );
        }
    }
}

#[test]
fn limits_symlinks_hold() {
    passes("limits-symlinks");
}

#[test]
fn limits_symlinks_names_an_unlisted_link() {
    breaks("limits-symlinks", "shortcut", |c| {
        c.symlink("shortcut", "CLAUDE.md");
    });
}

#[test]
fn limits_lfs_holds() {
    passes("limits-lfs");
}

#[test]
fn limits_lfs_names_a_pointer_file() {
    breaks("limits-lfs", "pointer.txt", |c| {
        c.write(
            "pointer.txt",
            "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 12\n",
        );
    });
}

// ---- the Codex report ----

#[test]
fn codex_report_counts_oversized_skills_and_names_the_largest() {
    let report = check::limits::codex_report(&Repo::new(real_root())).unwrap();
    assert!(report.contains("informational, not a gate"), "{report}");
    assert!(
        report.contains(" SKILL.md files exceed 8000 bytes"),
        "{report}"
    );
}

#[test]
fn codex_report_reports_a_growing_skill_without_failing_anything() {
    let copy = Copy::new();
    let before = check::limits::codex_report(&Repo::new(copy.root())).unwrap();
    let over = |report: &str| -> usize {
        let line = report
            .lines()
            .find(|l| l.contains("SKILL.md files exceed"))
            .unwrap();
        line.split(" / ").next().unwrap().trim().parse().unwrap()
    };
    let text = copy.read("skills/karen-and-the-manager/SKILL.md");
    copy.write(
        "skills/karen-and-the-manager/SKILL.md",
        &format!("{text}\n{}", "padding words ".repeat(5000)),
    );
    let after = check::limits::codex_report(&Repo::new(copy.root())).unwrap();
    assert_eq!(over(&after), over(&before) + 1, "{after}");
    assert!(after.contains("karen-and-the-manager"), "{after}");
    assert!(
        check::run_all(copy.root())
            .iter()
            .all(|o| o.problems.is_empty()),
        "an oversized SKILL.md must not fail a gate"
    );
}
