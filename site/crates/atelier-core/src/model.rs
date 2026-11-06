//! The site's data model: skills, their reference files, and the fixed
//! category grouping.

use serde::{Deserialize, Serialize};

/// One skill as the build reads it from `skills/<slug>/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    /// The directory name.
    pub slug: String,
    /// Frontmatter `name`.
    pub name: String,
    /// Frontmatter `description` (first line only; see `frontmatter`).
    pub description: String,
    /// Everything after the closing frontmatter delimiter, untouched. The
    /// leading `# Title` line is stripped at render time, not here.
    pub body: String,
    /// `reference/*.md` by name, then `reference/stacks/*.md` by name.
    pub references: Vec<Reference>,
}

/// One file under a skill's `reference/` tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    /// Path relative to the skill directory, `/`-separated:
    /// `reference/color.md`, `reference/stacks/web.md`.
    pub rel_path: String,
    /// The file stem, humanized (`cli-output` becomes `CLI Output`).
    pub title: String,
    /// The whole file.
    pub body: String,
}

/// A display category. Order here is the display order, prev/next order and
/// see-also order; `skill_slugs` order is preserved verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Category {
    pub slug: &'static str,
    pub title: &'static str,
    pub skill_slugs: &'static [&'static str],
}

/// The authoritative category grouping (content_pipeline.py `CATEGORIES`),
/// ported verbatim. It is fixed by the site's design, not derived from
/// CATALOG.md or any other source.
pub fn categories() -> &'static [Category] {
    &CATEGORIES
}

static CATEGORIES: [Category; 12] = [
    Category {
        slug: "ui-and-design",
        title: "UI and design",
        skill_slugs: &["formidable"],
    },
    Category {
        slug: "judgment-and-meta",
        title: "Judgment and meta",
        skill_slugs: &[
            "using-tbaguette",
            "orchestrating-work-end-to-end",
            "clairvoyance",
            "calibrating-confidence",
            "estimating-effort",
            "deciding-reversibility",
            "steelmanning-alternatives",
            "managing-scope-drift",
            "revalidating-decisions",
            "reading-specifications",
            "scoping-before-building",
        ],
    },
    Category {
        // Split out of "Judgment and meta" (2026-08-25), which had grown to
        // fourteen skills spanning four unrelated activities. These five are
        // one activity: turning an approved design into tasks and getting
        // those tasks executed, whether inline, by a gated subagent, in
        // parallel, or somewhere else entirely. Each one's "When to use"
        // already bounded itself against the others while they sat in
        // different categories -- working-a-plan-task-by-task and
        // structuring-an-implementation-plan under "Communicating",
        // delegating-tasks-with-review-gates and fanning-out-independent-work
        // under "Environment and tooling" -- which is what made the split
        // obvious rather than merely tidy.
        slug: "planning-and-delegation",
        title: "Planning and delegation",
        skill_slugs: &[
            "structuring-an-implementation-plan",
            "working-a-plan-task-by-task",
            "delegating-tasks-with-review-gates",
            "fanning-out-independent-work",
            "routing-around-capability-gaps",
            "bounding-autonomous-work",
            "designing-verifiable-goals",
            "checkpointing-long-runs",
        ],
    },
    Category {
        slug: "reading-code",
        title: "Reading code",
        skill_slugs: &[
            "orienting-in-unfamiliar-code",
            "tracing-data-flow",
            "code-archaeology",
            "recovering-agent-context",
            "mapping-dependencies",
            "finding-the-seam",
            "naming-things",
        ],
    },
    Category {
        slug: "landing-changes",
        title: "Landing changes",
        skill_slugs: &[
            "atomic-commits",
            "writing-commit-messages",
            "incremental-migration",
            "refactoring-safely",
            "judging-duplication",
            "deleting-code",
            "feature-flagging",
            "resolving-merge-conflicts",
            "isolating-work-with-worktrees",
            "landing-a-finished-branch",
        ],
    },
    Category {
        slug: "testing",
        title: "Testing",
        skill_slugs: &[
            "designing-test-data",
            "auditing-new-input-categories",
            "property-based-testing",
            "testing-the-untestable",
            "flaky-test-triage",
            "regression-test-from-bug",
            "characterization-testing",
            "choosing-test-scope",
            "grounding-test-doubles",
            "writing-the-failing-test-first",
            "evaluating-llm-output",
        ],
    },
    Category {
        slug: "debugging-and-performance",
        title: "Debugging and performance",
        skill_slugs: &[
            "reproducing-bugs",
            "bisecting-failures",
            "reading-stack-traces",
            "debugging-concurrency",
            "observing-production-safely",
            "performance-profiling",
            "finding-resource-leaks",
            "diagnosing-before-fixing",
            "responding-to-incidents",
        ],
    },
    Category {
        slug: "designing-systems",
        title: "Designing systems",
        skill_slugs: &[
            "designing-apis",
            "modeling-errors",
            "designing-for-idempotency",
            "choosing-concurrency-model",
            "modeling-state-machines",
            "drawing-boundaries",
            "caching-strategy",
            "schema-evolution",
            "data-migrations",
            "configuration-management",
            "instrumenting-for-observability",
            "rate-limiting-and-backpressure",
            "tracking-data-provenance",
            "keeping-copies-in-sync",
            "building-llm-features",
        ],
    },
    Category {
        slug: "security",
        title: "Security (defensive)",
        skill_slugs: &[
            "threat-modeling",
            "handling-untrusted-input",
            "validating-numeric-input",
            "secrets-hygiene",
            "redacting-sensitive-output",
            "auditing-dependencies",
            "least-privilege-design",
        ],
    },
    Category {
        // The other half of the "Judgment and meta" split (2026-08-25). These
        // five skills spend their own text defining each other's edges --
        // finishing-what-you-started calls itself the near side of the finish
        // line and knowing-when-to-stop the far side; karen-and-the-manager
        // says it is invoked immediately after knowing-when-to-stop;
        // red-teaming-your-own-work and knowing-when-to-stop both defer the
        // evidence question to confirming-before-claiming-done. Mutual
        // boundary-setting at that density is the tell that a set of skills is
        // one activity, so they are filed as one.
        slug: "finishing-and-proving",
        title: "Finishing and proving",
        skill_slugs: &[
            "finishing-what-you-started",
            "confirming-before-claiming-done",
            "red-teaming-your-own-work",
            "karen-and-the-manager",
            "knowing-when-to-stop",
            "offering-the-next-move",
            "handing-a-step-to-a-person",
        ],
    },
    Category {
        slug: "communicating",
        title: "Communicating",
        skill_slugs: &[
            "writing-durable-docs",
            "writing-agent-instructions",
            "writing-adrs",
            "writing-release-notes",
            "writing-postmortems",
            "reviewing-code-deeply",
            "explaining-technical-work",
            "crouton",
            "handing-off-for-review",
            "verifying-review-feedback",
        ],
    },
    Category {
        slug: "environment-and-tooling",
        title: "Environment and tooling",
        skill_slugs: &[
            "portable-shell-scripting",
            "reproducible-environments",
            "designing-ci-pipelines",
            "upgrading-dependencies",
            "keeping-tbaguette-current",
            "tending-tbaguette",
            "automating-repetition",
        ],
    },
];

/// Words that keep their capitals in a title, from how the stack files' own
/// headings write them ("Stack: CLI output", "game HUD", "terminal / TUI",
/// "XR and spatial"); PDF follows by analogy.
const ACRONYM_WORDS: [(&str, &str); 5] = [
    ("cli", "CLI"),
    ("hud", "HUD"),
    ("tui", "TUI"),
    ("xr", "XR"),
    ("pdf", "PDF"),
];

/// content_pipeline.py `humanize_filename`: `web` -> `Web`,
/// `cli-output` -> `CLI Output`. Other words get Python's `.capitalize()`:
/// first character upper-cased, the rest lower-cased.
pub fn humanize_filename(stem: &str) -> String {
    stem.split('-')
        .map(
            |word| match ACRONYM_WORDS.iter().find(|(k, _)| *k == word) {
                Some((_, upper)) => (*upper).to_string(),
                None => capitalize(word),
            },
        )
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}
