//! The graph data built from the skills on disk: its shape, its JSON, and
//! the banner summary the landing page prints.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use atelier_core::graph::{self, ChangeStamp, GraphData};
use atelier_core::model::{Skill, categories};

const SKILL_URL_TEMPLATE: &str = "/tbaguette-skills/skills/{slug}/";

fn skills() -> BTreeMap<String, Skill> {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    atelier_build::repo::load_skills(&root).expect("the repository's skills load")
}

fn graph_with(fresh: &BTreeMap<String, ChangeStamp>) -> GraphData {
    graph::build(&skills(), categories(), fresh, SKILL_URL_TEMPLATE)
}

#[test]
fn every_skill_is_a_node_and_the_json_is_stable() {
    let skills = skills();
    let graph = graph_with(&BTreeMap::new());
    assert_eq!(graph.skills.len(), skills.len());
    let json = graph::to_json(&graph);
    assert_eq!(json, graph::to_json(&graph_with(&BTreeMap::new())));
    for slug in skills.keys() {
        assert!(json.contains(slug), "{slug} missing from the graph JSON");
    }
}

#[test]
fn a_skill_missing_from_the_freshness_map_carries_no_change() {
    let graph = graph_with(&BTreeMap::new());
    for node in &graph.skills {
        assert_eq!(
            (node.change_status.as_deref(), node.change_at.as_deref()),
            (None, None)
        );
    }
}

#[test]
fn a_freshness_stamp_lands_on_its_skill_only() {
    let mut fresh = BTreeMap::new();
    fresh.insert(
        "naming-things".to_owned(),
        ChangeStamp {
            status: Some("modified".to_owned()),
            at: Some("2026-10-29T14:11:17+00:00".to_owned()),
        },
    );
    let graph = graph_with(&fresh);
    for node in &graph.skills {
        let stamped = node.slug == "naming-things";
        assert_eq!(node.change_status.is_some(), stamped, "{}", node.slug);
        assert_eq!(node.change_at.is_some(), stamped, "{}", node.slug);
    }
}

#[test]
fn the_banner_summary_describes_the_whole_graph() {
    let graph = graph_with(&BTreeMap::new());
    let summary = graph::banner_summary(&graph);
    assert_eq!(summary.skill_count, graph.skills.len());
    assert_eq!(summary.families.len(), categories().len());
    assert_eq!(
        summary.families.iter().map(|f| f.count).sum::<usize>(),
        summary.skill_count
    );
    assert!(summary.reachable, "every skill should reach every other");
    assert!(summary.pair_count >= summary.mutual_count);
    assert!(summary.max_steps >= 1);
    assert!(
        summary
            .links
            .iter()
            .all(|[a, b, citations]| a < b && *b < summary.families.len() && *citations > 0)
    );
}

#[test]
fn every_citation_in_a_description_has_a_sentence_to_quote() {
    let graph = graph_with(&BTreeMap::new());
    for node in &graph.skills {
        for (cited, _) in &node.trigger.refs {
            let quote = node
                .trigger
                .quotes
                .iter()
                .find(|(slug, _)| slug == cited)
                .map(|(_, text)| text.as_str());
            assert!(
                quote.is_some_and(|q| !q.is_empty()),
                "{} cites {cited} in its description with no quote",
                node.slug
            );
        }
    }
}
