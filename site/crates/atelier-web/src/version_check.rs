//! The site update check (site.js initVersionCheck, showUpdateModal) and its
//! companion, the post-reload scroll restore (initScrollRestore).
//!
//! Every 10-12 s (re-armed with a fresh jittered timeout, skipped while the
//! tab is hidden) the page fetches version.txt with no-store and a cache
//! buster, and compares its trimmed text with the header <time>'s datetime
//! ATTRIBUTE, byte for byte. A difference shows a reload-only modal and stops
//! polling; failures are silent and simply re-arm.

use crate::dom::{Slot, body, document, icon_markup, js_num, js_trim, on, set_timeout, then, truthy, window};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlButtonElement, RequestCache, RequestInit, Response};

const SCROLL_RESTORE_KEY: &str = "tbaguette-reload-scroll-y";
const POLL_MIN_MS: f64 = 10000.0;
const POLL_MAX_MS: f64 = 12000.0;

fn show_update_modal() -> Result<(), JsValue> {
    let doc = document()?;
    if doc.query_selector(".update-modal-overlay")?.is_some() {
        return Ok(());
    }
    let b = body()?;
    let title = b.get_attribute("data-i18n-modal-title");
    let text = b.get_attribute("data-i18n-modal-body");
    let reload_label = b.get_attribute("data-i18n-modal-reload");

    let overlay = doc.create_element("div")?;
    overlay.set_class_name("update-modal-overlay");
    let modal = doc.create_element("div")?;
    modal.set_class_name("update-modal");
    modal.set_attribute("role", "alertdialog")?;
    modal.set_attribute("aria-modal", "true")?;
    modal.set_attribute("aria-labelledby", "update-modal-title")?;
    modal.set_attribute("aria-describedby", "update-modal-body")?;

    // The seal is the loaf mark alone: with no sprite it is dropped whole.
    let seal_markup = icon_markup("icon-crust");
    if !seal_markup.is_empty() {
        let seal = doc.create_element("span")?;
        seal.set_class_name("update-modal__seal");
        seal.set_attribute("aria-hidden", "true")?;
        seal.set_inner_html(&seal_markup);
        modal.append_child(&seal)?;
    }
    let title_el = doc.create_element("p")?;
    title_el.set_class_name("update-modal__title");
    title_el.set_id("update-modal-title");
    title_el.set_text_content(title.as_deref());
    let body_el = doc.create_element("p")?;
    body_el.set_class_name("update-modal__body");
    body_el.set_id("update-modal-body");
    body_el.set_text_content(text.as_deref());

    let reload: HtmlButtonElement = doc.create_element("button")?.unchecked_into();
    reload.set_class_name("update-modal__reload");
    reload.set_type("button");
    let icon = icon_markup("icon-rotate");
    if !icon.is_empty() {
        let wrap = doc.create_element("span")?;
        wrap.set_inner_html(&icon);
        if let Some(svg) = wrap.first_child() {
            reload.append_child(&svg)?;
        }
    }
    let label = doc.create_element("span")?;
    label.set_text_content(reload_label.as_deref());
    reload.append_child(&label)?;

    modal.append_child(&title_el)?;
    modal.append_child(&body_el)?;
    modal.append_child(&reload)?;
    overlay.append_child(&modal)?;

    // Genuinely modal: everything already in <body> becomes inert.
    let children = b.children();
    for i in 0..children.length() {
        if let Some(el) = children.item(i) {
            el.set_attribute("inert", "")?;
        }
    }
    b.append_child(&overlay)?;

    let button = reload.clone();
    on(&reload, "click", move |_: Event| {
        // One navigation per press, however slow the reload.
        if button.get_attribute("aria-busy").as_deref() == Some("true") {
            return;
        }
        let _ = button.set_attribute("aria-busy", "true");
        if let Ok(w) = window() {
            if let Ok(Some(storage)) = w.session_storage() {
                let _ = storage.set_item(SCROLL_RESTORE_KEY, &js_num(w.scroll_y().unwrap_or(0.0)));
            }
            let _ = w.location().reload();
        }
    })?;
    reload.focus()
}

type Continuation = Closure<dyn FnMut(JsValue)>;

struct Poller {
    version_url: String,
    initial: String,
    on_response: Slot<Continuation>,
    on_text: Slot<Continuation>,
    on_fail: Slot<Continuation>,
}

fn schedule_next(p: &Rc<Poller>) {
    let delay = POLL_MIN_MS + js_sys::Math::random() * (POLL_MAX_MS - POLL_MIN_MS);
    let again = p.clone();
    set_timeout(move || poll(&again), delay);
}

fn poll(p: &Rc<Poller>) {
    let Ok(w) = window() else { return };
    // Backgrounded: no request, but the chain keeps running.
    if w.document().is_some_and(|d| d.hidden()) {
        schedule_next(p);
        return;
    }
    let sep = if p.version_url.contains('?') { '&' } else { '?' };
    let url = format!("{}{}t={}", p.version_url, sep, js_num(js_sys::Date::now()));
    let init = RequestInit::new();
    init.set_cache(RequestCache::NoStore);
    let request = w.fetch_with_str_and_init(&url, &init);
    let (resp, fail) = (p.on_response.borrow(), p.on_fail.borrow());
    if let (Some(resp), Some(fail)) = (resp.as_ref(), fail.as_ref()) {
        then(&request, resp.as_ref(), Some(fail.as_ref()));
    }
}

pub fn init_version_check() -> Result<(), JsValue> {
    let Some(time) = document()?.query_selector("[data-format-updated]")? else { return Ok(()) };
    let version_url = time.get_attribute("data-version-url").unwrap_or_default();
    let initial = time.get_attribute("datetime").unwrap_or_default();
    if version_url.is_empty() || initial.is_empty() || !truthy(&window()?, "fetch") {
        return Ok(());
    }
    let poller = Rc::new(Poller {
        version_url,
        initial,
        on_response: RefCell::new(None),
        on_text: RefCell::new(None),
        on_fail: RefCell::new(None),
    });
    // The three continuations are made once and reused by every poll. They
    // hold the poller weakly, so the cycle through it is not a leak.
    let weak = Rc::downgrade(&poller);
    *poller.on_fail.borrow_mut() = Some(Closure::new(move |_: JsValue| {
        if let Some(p) = weak.upgrade() {
            schedule_next(&p);
        }
    }));
    let weak = Rc::downgrade(&poller);
    *poller.on_text.borrow_mut() = Some(Closure::new(move |text: JsValue| {
        let Some(p) = weak.upgrade() else { return };
        let latest = js_trim(&text.as_string().unwrap_or_default());
        if !latest.is_empty() && latest != p.initial {
            let _ = show_update_modal();
        } else {
            schedule_next(&p);
        }
    }));
    let weak = Rc::downgrade(&poller);
    *poller.on_response.borrow_mut() = Some(Closure::new(move |response: JsValue| {
        let Some(p) = weak.upgrade() else { return };
        let response: Response = response.unchecked_into();
        let text = if response.ok() { response.text().ok() } else { None };
        match text {
            Some(promise) => {
                let (on_text, on_fail) = (p.on_text.borrow(), p.on_fail.borrow());
                if let (Some(t), Some(f)) = (on_text.as_ref(), on_fail.as_ref()) {
                    then(&promise, t.as_ref(), Some(f.as_ref()));
                }
            }
            None => schedule_next(&p),
        }
    }));
    schedule_next(&poller);
    // The page keeps the poller for its lifetime.
    std::mem::forget(poller);
    Ok(())
}

/// After the modal's reload: put the reader back where they were.
pub fn init_scroll_restore() -> Result<(), JsValue> {
    let w = window()?;
    let saved = (|| -> Result<Option<String>, JsValue> {
        let Some(storage) = w.session_storage()? else { return Ok(None) };
        let saved = storage.get_item(SCROLL_RESTORE_KEY)?;
        if saved.is_some() {
            storage.remove_item(SCROLL_RESTORE_KEY)?;
        }
        Ok(saved)
    })();
    let Ok(Some(saved)) = saved else { return Ok(()) };
    let y = js_sys::parse_int(&saved, 10);
    if !y.is_nan() {
        w.scroll_to_with_x_and_y(0.0, y);
    }
    Ok(())
}
