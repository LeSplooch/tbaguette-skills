//! The update-notes archive (site.js initNotesArchive): where <dialog> and
//! showModal exist, the server's <details> becomes a trigger button and a
//! modal dialog holding the same panel. The page's scroll lock follows the
//! dialog's `open` attribute through a MutationObserver, not its close event.

use crate::dom::{body, document, icon_markup, js_trim, on, window};
use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlButtonElement, HtmlDialogElement, HtmlElement, MouseEvent, MutationObserver, MutationObserverInit};

const LOCK: &str = "has-notes-dialog";

fn dialog_supported() -> bool {
    let Ok(w) = window() else { return false };
    let ctor = Reflect::get(&w, &"HTMLDialogElement".into()).unwrap_or(JsValue::UNDEFINED);
    if !ctor.is_function() {
        return false;
    }
    let Ok(probe) = document().and_then(|d| d.create_element("dialog")) else { return false };
    Reflect::get(&probe, &"showModal".into()).map(|f| f.is_function()).unwrap_or(false)
}

fn set_lock(on: bool) {
    if let Some(root) = document().ok().and_then(|d| d.document_element()) {
        let _ = root.class_list().toggle_with_force(LOCK, on);
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(details) = doc.query_selector("[data-notes-archive]")? else { return Ok(()) };
    let (Some(panel), Some(summary)) = (details.query_selector("[data-notes-archive-panel]")?, details.query_selector("summary")?)
    else {
        return Ok(());
    };
    if !dialog_supported() {
        return Ok(());
    }

    let dialog: HtmlDialogElement = doc.create_element("dialog")?.unchecked_into();
    dialog.set_class_name("notes-dialog");
    dialog.set_attribute("aria-labelledby", "notes-dialog-title")?;

    let head = doc.create_element("div")?;
    head.set_class_name("notes-dialog__head");
    let icon = icon_markup("icon-wheat");
    if !icon.is_empty() {
        let wrap = doc.create_element("span")?;
        wrap.set_inner_html(&icon);
        if let Some(svg) = wrap.first_element_child() {
            svg.set_attribute("class", "icon notes-dialog__icon")?;
            head.append_child(&svg)?;
        }
    }
    let title = doc.create_element("h2")?;
    title.set_class_name("notes-dialog__title");
    title.set_id("notes-dialog-title");
    let title_text = details.get_attribute("data-archive-title").filter(|s| !s.is_empty());
    title.set_text_content(Some(title_text.as_deref().unwrap_or("Update notes")));
    head.append_child(&title)?;
    if let Some(sub) = details.get_attribute("data-archive-subtitle").filter(|s| !s.is_empty()) {
        let subtitle = doc.create_element("p")?;
        subtitle.set_class_name("notes-dialog__subtitle");
        subtitle.set_text_content(Some(&sub));
        head.append_child(&subtitle)?;
    }
    let close: HtmlButtonElement = doc.create_element("button")?.unchecked_into();
    close.set_class_name("notes-dialog__close");
    close.set_type("button");
    let close_label = details.get_attribute("data-archive-close").filter(|s| !s.is_empty());
    close.set_attribute("aria-label", close_label.as_deref().unwrap_or("Close"))?;
    head.append_child(&close)?;

    // Focusable, so twenty entries can be scrolled from the keyboard.
    let region: HtmlElement = doc.create_element("div")?.unchecked_into();
    region.set_class_name("notes-dialog__body");
    region.set_tab_index(0);
    region.append_child(&panel)?;
    dialog.append_child(&head)?;
    dialog.append_child(&region)?;

    let trigger: HtmlButtonElement = doc.create_element("button")?.unchecked_into();
    trigger.set_class_name("notes__archive-trigger");
    trigger.set_type("button");
    trigger.set_text_content(Some(&js_trim(&summary.text_content().unwrap_or_default())));

    let parent = details.parent_node().ok_or_else(|| JsValue::from_str("notes: detached"))?;
    parent.insert_before(&trigger, Some(&details))?;
    parent.remove_child(&details)?;
    body()?.append_child(&dialog)?;

    let d = dialog.clone();
    let observer = Closure::<dyn FnMut()>::new(move || set_lock(d.open()));
    let open_state = MutationObserver::new(observer.as_ref().unchecked_ref())?;
    observer.forget();
    let init = MutationObserverInit::new();
    init.set_attributes(true);
    init.set_attribute_filter(&Array::of1(&"open".into()));
    open_state.observe_with_options(&dialog, &init)?;

    let (d, r) = (dialog.clone(), region.clone());
    on(&trigger, "click", move |_: Event| {
        if d.show_modal().is_err() {
            return;
        }
        set_lock(true);
        // Arrive on the scroll region, not on "close".
        let _ = r.focus();
        r.set_scroll_top(0);
    })?;
    let d = dialog.clone();
    on(&close, "click", move |_: Event| d.close())?;
    // A click on the dialog element itself landed on the backdrop.
    let d = dialog.clone();
    on(&dialog, "click", move |e: MouseEvent| {
        if e.target().is_some_and(|t| JsValue::from(t) == JsValue::from(&d)) {
            d.close();
        }
    })?;
    on(&dialog, "close", move |_: Event| set_lock(false))
}
