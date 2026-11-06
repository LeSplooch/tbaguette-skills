//! The Crumb, the skill graph at /graph/: its own module, loaded on that page
//! only, so the other pages never download it. The page still loads
//! atelier-web for the header, theme and update check, as it loaded site.js
//! beside graph.js.

use wasm_bindgen::prelude::*;

mod graph;

#[wasm_bindgen(start)]
pub fn start() {
    if let Err(error) = graph::init() {
        web_sys::console::error_2(&JsValue::from_str("atelier-graph: the graph failed to start:"), &error);
    }
}
