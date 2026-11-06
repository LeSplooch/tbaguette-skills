//! Tab groups (site.js initTabs): click and arrow/Home/End selection with a
//! roving tabindex and automatic activation, an opt-in Windows preselect, and
//! hash deep links to a panel id, which select its tab and scroll to it.

use crate::dom::{document, html, on, prefers_reduced_motion, select_all, select_within, window};
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlElement, KeyboardEvent, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

struct Group {
    tabs: Vec<HtmlElement>,
    panels: Vec<HtmlElement>,
}

impl Group {
    fn activate(&self, index: usize, focus: bool, scroll: bool) {
        let tab = &self.tabs[index];
        for (i, candidate) in self.tabs.iter().enumerate() {
            let _ = candidate.set_attribute("aria-selected", if i == index { "true" } else { "false" });
            candidate.set_tab_index(if i == index { 0 } else { -1 });
        }
        let controls = tab.get_attribute("aria-controls");
        for panel in &self.panels {
            panel.set_hidden(Some(panel.id()) != controls);
        }
        if focus {
            let _ = tab.focus();
        }
        if scroll {
            let panel = controls.and_then(|id| document().ok()?.get_element_by_id(&id));
            if let Some(panel) = panel {
                let opts = ScrollIntoViewOptions::new();
                opts.set_behavior(if prefers_reduced_motion() { ScrollBehavior::Auto } else { ScrollBehavior::Smooth });
                opts.set_block(ScrollLogicalPosition::Start);
                panel.scroll_into_view_with_scroll_into_view_options(&opts);
            }
        }
    }
}

/// Best-effort OS sniff for the opt-in platform preselect only.
fn is_windows_platform() -> bool {
    let Ok(w) = window() else { return false };
    let nav = w.navigator();
    let ua_data = Reflect::get(&nav, &"userAgentData".into()).unwrap_or(JsValue::UNDEFINED);
    if ua_data.is_truthy()
        && let Some(p) = Reflect::get(&ua_data, &"platform".into()).ok().and_then(|p| p.as_string())
    {
        return p.to_lowercase().contains("win");
    }
    let legacy = nav.platform().ok().filter(|p| !p.is_empty()).or_else(|| nav.user_agent().ok()).unwrap_or_default();
    legacy.to_lowercase().contains("win")
}

pub fn init() -> Result<(), JsValue> {
    let groups = select_all("[data-tabs]")?;
    if groups.is_empty() {
        return Ok(());
    }
    let mut every_tab: Vec<(Rc<Group>, usize)> = Vec::new();
    for root in groups {
        let group = Rc::new(Group {
            tabs: select_within(&root, "[role=\"tab\"]")?.iter().map(html).collect(),
            panels: select_within(&root, "[role=\"tabpanel\"]")?.iter().map(html).collect(),
        });
        if root.get_attribute("data-autoselect-platform").as_deref() == Some("true")
            && is_windows_platform()
            && let Some(i) = group.tabs.iter().position(|t| t.get_attribute("data-platform").as_deref() == Some("windows"))
        {
            group.activate(i, false, false);
        }
        for (index, tab) in group.tabs.iter().enumerate() {
            let g = group.clone();
            on(tab, "click", move |_: Event| g.activate(index, false, false))?;
            let g = group.clone();
            on(tab, "keydown", move |e: KeyboardEvent| {
                let n = g.tabs.len();
                let target = match e.key().as_str() {
                    "ArrowRight" | "ArrowDown" => (index + 1) % n,
                    "ArrowLeft" | "ArrowUp" => (index + n - 1) % n,
                    "Home" => 0,
                    "End" => n - 1,
                    _ => return,
                };
                e.prevent_default();
                g.activate(target, true, false);
            })?;
            every_tab.push((group.clone(), index));
        }
    }
    let every_tab = Rc::new(every_tab);
    // The tab owning the panel the URL's hash names, if any.
    let owner = move || {
        let hash = window().and_then(|w| w.location().hash()).unwrap_or_default();
        let id = hash.strip_prefix('#').unwrap_or(&hash).to_string();
        if id.is_empty() {
            return None;
        }
        every_tab.iter().find(|(g, i)| g.tabs[*i].get_attribute("aria-controls").as_deref() == Some(id.as_str())).cloned()
    };
    let on_hash = owner.clone();
    let w = window()?;
    on(&w, "hashchange", move |_: Event| {
        if let Some((g, i)) = on_hash() {
            g.activate(i, false, true);
        }
    })?;
    // At load the panel opens at once, but the scroll waits one frame. site.js
    // ran during parsing, and the browser's own jump to the fragment, made
    // after parsing against the finished layout, superseded its scroll. This
    // module starts after that jump (made while the panel was still hidden),
    // so it makes the scroll itself, once the features after this one (the
    // header's formatted time, which can rewrap the header) have run.
    if let Some((g, i)) = owner() {
        g.activate(i, false, false);
        let scroll = Closure::once_into_js(move |_: f64| g.activate(i, false, true));
        w.request_animation_frame(scroll.unchecked_ref())?;
    }
    Ok(())
}
