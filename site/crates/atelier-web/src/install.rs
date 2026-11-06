//! Copy buttons (site.js initInstallCopy): each `[data-copy-target]` copies
//! its target's text through the Clipboard API, falling back to a hidden
//! textarea and `execCommand('copy')` when the API is missing or refuses, and
//! shows "Copied!" for 1.6 s either way.

use crate::dom::{body, clear_timeout, document, on, select_all, set_timeout, then, truthy, window};
use js_sys::{Function, Promise, Reflect};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlDocument, HtmlTextAreaElement};

const COPIED_MS: f64 = 1600.0;

fn fallback_copy(text: &str) -> Result<(), JsValue> {
    let doc = document()?;
    let scratch: HtmlTextAreaElement = doc.create_element("textarea")?.unchecked_into();
    scratch.set_value(text);
    scratch.set_attribute("readonly", "")?;
    scratch.style().set_property("position", "fixed")?;
    scratch.style().set_property("opacity", "0")?;
    let b = body()?;
    b.append_child(&scratch)?;
    scratch.select();
    let _ = doc.unchecked_ref::<HtmlDocument>().exec_command("copy");
    b.remove_child(&scratch)?;
    Ok(())
}

struct Button {
    button: Element,
    label: Element,
    default_label: Option<String>,
    timer: RefCell<JsValue>,
}

fn show_copied(b: &Rc<Button>) {
    let _ = b.button.class_list().add_1("is-copied");
    let copied = body().ok().and_then(|x| x.get_attribute("data-i18n-copied"));
    b.label.set_text_content(copied.as_deref());
    clear_timeout(&b.timer.borrow());
    let again = b.clone();
    let id = set_timeout(
        move || {
            let _ = again.button.class_list().remove_1("is-copied");
            again.label.set_text_content(again.default_label.as_deref());
        },
        COPIED_MS,
    );
    *b.timer.borrow_mut() = id;
}

/// `navigator.clipboard.writeText(text)` as a promise, or None when the
/// browser has no such method.
fn clipboard_write(text: &str) -> Option<Promise> {
    let navigator = window().ok()?.navigator();
    let clipboard = Reflect::get(&navigator, &"clipboard".into()).ok()?;
    if !clipboard.is_truthy() || !truthy(&clipboard, "writeText") {
        return None;
    }
    let write: Function = Reflect::get(&clipboard, &"writeText".into()).ok()?.dyn_into().ok()?;
    write.call1(&clipboard, &JsValue::from_str(text)).ok()?.dyn_into().ok()
}

pub fn init() -> Result<(), JsValue> {
    let buttons = select_all("[data-copy-target]")?;
    let doc = document()?;
    for button in buttons {
        let target = doc.get_element_by_id(&button.get_attribute("data-copy-target").unwrap_or_else(|| "null".into()));
        let label = button.query_selector("[data-copy-label]")?;
        let (Some(target), Some(label)) = (target, label) else { continue };
        let state = Rc::new(Button {
            default_label: label.text_content(),
            button: button.clone(),
            label,
            timer: RefCell::new(JsValue::NULL),
        });
        on(&button, "click", move |_: Event| {
            let text = target.text_content().unwrap_or_default();
            match clipboard_write(&text) {
                Some(promise) => {
                    let ok = state.clone();
                    let fail = state.clone();
                    let on_ok = Closure::once_into_js(move |_: JsValue| show_copied(&ok));
                    let on_fail = Closure::once_into_js(move |_: JsValue| {
                        let _ = fallback_copy(&text);
                        show_copied(&fail);
                    });
                    then(&promise, &on_ok, Some(&on_fail));
                }
                None => {
                    let _ = fallback_copy(&text);
                    show_copied(&state);
                }
            }
        })?;
    }
    Ok(())
}
