//! The gates about the skills themselves: frontmatter limits
//! (scripts/test_skill_frontmatter.py), the CATALOG.md mirror
//! (test_catalog.py) and cross-references (test_skill_references.py).

mod common;

use atelier_build::check::catalog::spell;
use common::{breaks, passes, skill_count};

const SKILL: &str = "skills/naming-things/SKILL.md";

// ---- frontmatter ----

#[test]
fn frontmatter_required_holds() {
    passes("frontmatter-required");
}

#[test]
fn frontmatter_required_names_a_skill_without_a_description() {
    breaks("frontmatter-required", "naming-things", |c| {
        c.write(SKILL, "---\nname: naming-things\n---\n\n# Naming things\n");
    });
}

#[test]
fn frontmatter_name_matches_dir_holds() {
    passes("frontmatter-name-matches-dir");
}

#[test]
fn frontmatter_name_matches_dir_names_the_mismatch() {
    breaks(
        "frontmatter-name-matches-dir",
        "naming-things: name is \"naming-thingz\"",
        |c| {
            c.replace(SKILL, "name: naming-things", "name: naming-thingz");
        },
    );
}

#[test]
fn frontmatter_name_limits_hold() {
    passes("frontmatter-name-limits");
}

#[test]
fn frontmatter_name_limits_names_a_reserved_word() {
    breaks(
        "frontmatter-name-limits",
        "naming-things: contains reserved word \"claude\"",
        |c| {
            c.replace(SKILL, "name: naming-things", "name: claude-naming");
        },
    );
}

#[test]
fn frontmatter_name_limits_names_a_bad_charset() {
    breaks(
        "frontmatter-name-limits",
        "naming-things: not lowercase-hyphen-alphanumeric",
        |c| {
            c.replace(SKILL, "name: naming-things", "name: Naming_Things");
        },
    );
}

#[test]
fn frontmatter_description_limit_holds() {
    passes("frontmatter-description-limit");
}

#[test]
fn frontmatter_description_limit_names_an_overlong_description() {
    breaks("frontmatter-description-limit", "naming-things:", |c| {
        c.replace(
            SKILL,
            "description: ",
            &format!("description: {} ", "x".repeat(1100)),
        );
    });
}

#[test]
fn frontmatter_description_limit_counts_a_wrapped_description_whole() {
    // The test's parser folds continuation lines; the build's does not. 600 + 600
    // characters across two lines is over the limit only if the lines are joined.
    breaks("frontmatter-description-limit", "naming-things: 1", |c| {
        let line = "y".repeat(600);
        c.write(
            SKILL,
            &format!(
                "---\nname: naming-things\ndescription: {line}\n  {line}\n---\n\n# Naming things\n"
            ),
        );
    });
}

#[test]
fn frontmatter_no_xml_holds() {
    passes("frontmatter-no-xml");
}

#[test]
fn frontmatter_no_xml_names_a_tag_in_a_description() {
    breaks("frontmatter-no-xml", "naming-things: description", |c| {
        c.replace(SKILL, "description: ", "description: <b>loud</b> ");
    });
}

// ---- CATALOG.md ----

#[test]
fn catalog_headings_hold() {
    passes("catalog-headings");
}

#[test]
fn catalog_headings_names_a_renamed_heading() {
    breaks("catalog-headings", "CATALOG.md headings", |c| {
        c.replace("CATALOG.md", "\n## Testing\n", "\n## Trials\n");
    });
}

#[test]
fn catalog_membership_holds() {
    passes("catalog-membership");
}

#[test]
fn catalog_membership_names_a_skill_missing_from_its_section() {
    breaks("catalog-membership", "naming-things", |c| {
        let text = c.read("CATALOG.md");
        let kept: Vec<&str> = text
            .lines()
            .filter(|l| !l.contains("| `naming-things`"))
            .collect();
        c.write("CATALOG.md", &(kept.join("\n") + "\n"));
    });
}

#[test]
fn catalog_no_duplicates_hold() {
    passes("catalog-no-duplicates");
}

#[test]
fn catalog_no_duplicates_names_a_twice_listed_skill() {
    breaks(
        "catalog-no-duplicates",
        "listed twice: naming-things",
        |c| {
            c.replace(
                "CATALOG.md",
                "\n## Testing\n",
                "\n## Testing\n\n| `naming-things` | again |\n",
            );
        },
    );
}

#[test]
fn catalog_foreign_holds() {
    passes("catalog-foreign");
}

#[test]
fn catalog_foreign_names_a_dagger_row_that_is_a_skill() {
    breaks("catalog-foreign", "naming-things", |c| {
        c.replace(
            "CATALOG.md",
            "\n## Testing\n",
            "\n## Testing\n\n| `naming-things` † | not ours |\n",
        );
    });
}

#[test]
fn catalog_prose_counts_hold() {
    passes("catalog-prose-counts");
}

#[test]
fn catalog_prose_counts_names_a_manifest_with_the_wrong_count() {
    breaks(
        "catalog-prose-counts",
        ".codex-plugin/plugin.json does not contain",
        |c| {
            c.replace(
                ".codex-plugin/plugin.json",
                &spell(skill_count()).unwrap(),
                &spell(skill_count() - 1).unwrap(),
            );
        },
    );
}

#[test]
fn catalog_prose_counts_names_a_readme_with_the_wrong_count() {
    breaks("catalog-prose-counts", "README.md does not contain", |c| {
        let n = skill_count();
        c.replace(
            "README.md",
            &format!("{n} skills"),
            &format!("{} skills", n - 2),
        );
    });
}

#[test]
fn catalog_prose_counts_names_a_stale_spelled_count() {
    breaks(
        "catalog-prose-counts",
        "still carries the stale count",
        |c| {
            let (now, before) = (
                spell(skill_count()).unwrap(),
                spell(skill_count() - 1).unwrap(),
            );
            c.replace(
                "gemini-extension.json",
                &now,
                &format!("{before} skills, then {now}"),
            );
        },
    );
}

#[test]
fn catalog_stated_count_holds() {
    passes("catalog-stated-count");
}

#[test]
fn catalog_stated_count_names_the_wrong_number() {
    let (on_disk, stated) = (skill_count(), skill_count() - 2);
    breaks(
        "catalog-stated-count",
        &format!("CATALOG.md says {stated}"),
        |c| {
            c.replace(
                "CATALOG.md",
                &format!("{on_disk} skills,"),
                &format!("{stated} skills,"),
            );
        },
    );
}

// ---- cross-references ----

#[test]
fn references_resolve_hold() {
    passes("references-resolve");
}

#[test]
fn references_resolve_names_a_dangling_reference() {
    breaks(
        "references-resolve",
        "skills/naming-things/SKILL.md: (no-such-skill)",
        |c| {
            let text = c.read(SKILL);
            c.write(
                SKILL,
                &format!("{text}\nNot for: renaming (see no-such-skill).\n"),
            );
        },
    );
}

#[test]
fn references_reachable_hold() {
    passes("references-reachable");
}

#[test]
fn references_reachable_names_a_skill_nothing_cites() {
    breaks("references-reachable", "orphan-skill", |c| {
        c.write(
            "skills/orphan-skill/SKILL.md",
            "---\nname: orphan-skill\ndescription: x\n---\n\n# Orphan\n",
        );
    });
}

#[test]
fn references_phase_index_holds() {
    passes("references-phase-index");
}

#[test]
fn references_phase_index_names_a_skill_the_index_forgot() {
    breaks("references-phase-index", "naming-things", |c| {
        c.replace_all(
            "skills/orchestrating-work-end-to-end/reference/phase-routing.md",
            "`naming-things`",
            "`naming-thing`",
        );
    });
}

#[test]
fn references_exemptions_hold() {
    passes("references-exemptions");
}

#[test]
fn references_exemptions_names_an_exemption_that_became_a_skill() {
    breaks("references-exemptions", "e-ink", |c| {
        c.write(
            "skills/e-ink/SKILL.md",
            "---\nname: e-ink\ndescription: x\n---\n",
        );
    });
}
