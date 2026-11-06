//! The header's controls: the theme toggle (site.js initThemeToggle) and the
//! dormant language switcher's dismissal (initLanguageSwitcher).
//!
//! Dark ("crust") is the absence of `data-theme`; light is
//! `data-theme="flour"`. The stored choice is applied before first paint by
//! the inline script in every page's <head>, never here.

use crate::dom::{document, on, window};
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlDetailsElement, HtmlElement, KeyboardEvent};

const STORAGE_KEY: &str = "tbaguette-theme";

fn current_theme() -> &'static str {
    let flour = document()
        .ok()
        .and_then(|d| d.document_element())
        .and_then(|h| h.get_attribute("data-theme"))
        .is_some_and(|t| t == "flour");
    if flour { "flour" } else { "crust" }
}

/// The label names the theme a click would switch to. A missing attribute
/// becomes the string "null", as `setAttribute(name, null)` makes it.
fn describe_toggle_target(toggle: &Element, theme: &str) {
    let source = if theme == "flour" { "data-i18n-theme-dark" } else { "data-i18n-theme-light" };
    let label = toggle.get_attribute(source).unwrap_or_else(|| "null".into());
    let _ = toggle.set_attribute("aria-label", &label);
}

fn set_theme(toggle: &Element, theme: &str) {
    if let Some(root) = document().ok().and_then(|d| d.document_element()) {
        let _ = if theme == "flour" { root.set_attribute("data-theme", "flour") } else { root.remove_attribute("data-theme") };
    }
    // Storage disabled or full: the theme still applies for this load.
    if let Ok(Some(storage)) = window().map(|w| w.local_storage()).unwrap_or(Ok(None)) {
        let _ = storage.set_item(STORAGE_KEY, theme);
    }
    describe_toggle_target(toggle, theme);
}

pub fn init_theme_toggle() -> Result<(), JsValue> {
    let Some(toggle) = document()?.query_selector("[data-theme-toggle]")? else { return Ok(()) };
    describe_toggle_target(&toggle, current_theme());
    let target = toggle.clone();
    on(&toggle, "click", move |_: Event| {
        let next = if current_theme() == "flour" { "crust" } else { "flour" };
        set_theme(&target, next);
    })
}

/// A native <details> needs no script to open and close; this adds only
/// dismissal: an outside click or Escape closes it, and Escape puts focus
/// back on its summary.
pub fn init_language_switcher() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(details) = doc.query_selector(".language-switcher")? else { return Ok(()) };
    let details: HtmlDetailsElement = details.unchecked_into();
    let d = details.clone();
    on(&doc, "click", move |e: Event| {
        let inside = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()).is_some_and(|n| d.contains(Some(&n)));
        if d.open() && !inside {
            d.set_open(false);
        }
    })?;
    let d = details.clone();
    on(&doc, "keydown", move |e: KeyboardEvent| {
        if e.key() != "Escape" || !d.open() {
            return;
        }
        d.set_open(false);
        if let Ok(Some(summary)) = d.query_selector("summary") {
            let _ = summary.unchecked_into::<HtmlElement>().focus();
        }
    })
}
