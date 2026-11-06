//! The browser half of the 48-hour New/Updated window (site.js
//! initFreshness): every `[data-fresh-at]` element older than the window by
//! the visitor's clock is removed, then any "Fresh from the oven" section left
//! with no tile goes too. Must run before search and the coverflow, which
//! count what survives.

use crate::dom::{select_all, select_within};
use wasm_bindgen::prelude::*;

const FRESH_WINDOW_MS: f64 = 48.0 * 60.0 * 60.0 * 1000.0;

pub fn init() -> Result<(), JsValue> {
    let stamped = select_all("[data-fresh-at]")?;
    if stamped.is_empty() {
        return Ok(());
    }
    let now = js_sys::Date::now();
    for el in stamped {
        let stamp = el.get_attribute("data-fresh-at").unwrap_or_default();
        let when = js_sys::Date::new(&JsValue::from_str(&stamp)).get_time();
        // An unparseable stamp leaves the element as the server rendered it.
        if when.is_nan() {
            continue;
        }
        if now - when > FRESH_WINDOW_MS {
            el.remove();
        }
    }
    for section in select_all("[data-fresh-section]")? {
        if select_within(&section, ".fresh__tile")?.is_empty() {
            section.remove();
        }
    }
    Ok(())
}
