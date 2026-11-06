//! The browser half of the site. Pages are static HTML first; this module
//! enhances them through the existing data-attribute contract, and every page
//! stays readable and navigable when it never loads.
//!
//! `start` runs once when the module is instantiated. It walks `FEATURES`,
//! and each feature is isolated: an `Err` is logged once to the console and the
//! next feature still runs.

use wasm_bindgen::prelude::*;

mod banners;
mod counter;
mod coverflow;
mod dom;
mod family_index;
mod freshness;
mod install;
mod loop_ring;
mod magnet;
mod mods_button;
mod notes;
mod replay;
mod search;
mod tabs;
mod theme;
mod unfold;
mod updated;
mod version_check;

/// One feature's initialiser. It finds its own elements, attaches its own
/// listeners, and returns `Err` rather than panicking when the page lacks
/// what it needs.
type Init = fn() -> Result<(), JsValue>;

/// The registry: feature name (for the console message) and its initialiser.
/// Later tasks append here, one line per ported feature.
/// The order is site.js's (its init calls at the end of the file) and it
/// matters: freshness removes expired tiles before the coverflow and search
/// count what is left, and the version check's first jittered delay is the
/// last thing to draw on Math.random during start-up.
const FEATURES: &[(&str, Init)] = &[
    ("ready", init_ready),
    ("theme toggle", theme::init_theme_toggle),
    ("freshness", freshness::init),
    ("fresh coverflow", coverflow::init),
    ("search", search::init),
    ("install copy", install::init),
    ("tabs", tabs::init),
    ("updated time", updated::init),
    ("language switcher", theme::init_language_switcher),
    ("scroll restore", version_check::init_scroll_restore),
    ("notes archive", notes::init),
    ("news", banners::init_news),
    ("release card", unfold::init),
    ("graph magnet", magnet::init),
    ("mods climb", mods_button::init),
    ("climb replay", replay::init),
    ("loop ring", loop_ring::init),
    ("counter", counter::init),
    ("family index", family_index::init),
    ("version check", version_check::init_version_check),
];

#[wasm_bindgen(start)]
fn start() {
    run_features(FEATURES, &mut |name, err| {
        web_sys::console::error_2(
            &JsValue::from_str(&format!("atelier-web: {name} failed:")),
            err,
        );
    });
}

/// Run every feature in order, reporting each failure to `on_error` and
/// carrying on. Returns how many succeeded.
fn run_features(features: &[(&str, Init)], on_error: &mut dyn FnMut(&str, &JsValue)) -> usize {
    let mut ok = 0;
    for (name, init) in features {
        match init() {
            Ok(()) => ok += 1,
            Err(err) => on_error(name, &err),
        }
    }
    ok
}

/// Proves the module loaded and ran: marks the root element so a probe (or a
/// stylesheet) can tell the enhancement layer is alive.
fn init_ready() -> Result<(), JsValue> {
    let root = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .ok_or_else(|| JsValue::from_str("no document element"))?;
    root.set_attribute("data-wasm", "ready")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local!(static RAN: Cell<u32> = const { Cell::new(0) });

    fn counts() -> Result<(), JsValue> {
        RAN.with(|r| r.set(r.get() + 1));
        Ok(())
    }
    // JsValue::from_str panics off wasm32, so the failing feature reports
    // through a JsValue::NULL constant, which is safe on the host.
    fn fails() -> Result<(), JsValue> {
        Err(JsValue::NULL)
    }

    #[test]
    fn only_the_nearer_of_two_pills_owns_the_pointer() {
        use crate::dom::{distance_to_box, owns_approach};
        // Mods at x 100..160, Graph at 170..250, both y 0..40.
        let mods = |x: f64, y: f64| distance_to_box(100.0, 0.0, 60.0, 40.0, x, y);
        let graph = |x: f64, y: f64| distance_to_box(170.0, 0.0, 80.0, 40.0, x, y);
        for (x, y) in [(130.0, 20.0), (162.0, 60.0), (40.0, 90.0), (165.0, 20.0)] {
            let m = owns_approach(mods(x, y), Some(graph(x, y)), true);
            let g = owns_approach(graph(x, y), Some(mods(x, y)), false);
            assert!(m != g, "exactly one owns ({x}, {y})");
        }
        assert!(owns_approach(graph(200.0, 20.0), Some(mods(200.0, 20.0)), false));
        assert!(owns_approach(5.0, None, false));
        assert_eq!(distance_to_box(0.0, 0.0, 10.0, 10.0, 13.0, 14.0), 5.0);
    }

    #[test]
    fn a_failing_feature_is_reported_once_and_the_rest_still_run() {
        let mut errors = Vec::new();
        let features: &[(&str, Init)] = &[("a", counts), ("bad", fails), ("b", counts)];
        let ok = run_features(features, &mut |name, _| errors.push(name.to_string()));
        assert_eq!(ok, 2);
        assert_eq!(errors, ["bad"]);
        assert_eq!(RAN.with(Cell::get), 2);
    }
}
