//! The landing page's announcements retiring by date (site.js initGraphNews), on
//! the reader's clock: a page published before the date stays right after it.

use crate::dom::select_all;
use wasm_bindgen::prelude::*;

/// Past an announcement's `data-*-new-until` date (UTC, compared as strings, as
/// the build does), it goes, taking the row it leads with it, and the header
/// link loses its "new" class.
pub fn init_news() -> Result<(), JsValue> {
    let iso = String::from(js_sys::Date::new_0().to_iso_string());
    let today = iso.get(..10).unwrap_or(&iso).to_string();
    let attrs = ["data-graph-new-until", "data-unfold-new-until", "data-climb-new-until"];
    let selector = attrs.iter().map(|a| format!("[{a}]")).collect::<Vec<_>>().join(", ");
    for el in select_all(&selector)? {
        let until = attrs.iter().filter_map(|a| el.get_attribute(a)).find(|v| !v.is_empty()).unwrap_or_default();
        if today.as_str() <= until.as_str() {
            continue;
        }
        if let Some(link) = el.closest(".site-header__nav-link--new")? {
            link.class_list().remove_1("site-header__nav-link--new")?;
        }
        el.closest(".hero__banners")?.unwrap_or(el).remove();
    }
    Ok(())
}
