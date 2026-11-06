//! Small helpers every feature shares: finding elements, attaching
//! page-lifetime listeners, timers, media queries, and turning numbers into
//! the exact strings JavaScript would have produced (several features write
//! numbers into styles and canvas colours that the probes compare byte for
//! byte with site.js's output).

use js_sys::{Function, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use web_sys::{
    AddEventListenerOptions, Document, DomRect, Element, EventTarget, HtmlElement, MutationObserver,
    MutationObserverInit, NodeList, Window,
};

/// The distance from (x, y) to the edge of a box, 0 inside it.
pub fn distance_to_box(left: f64, top: f64, width: f64, height: f64, x: f64, y: f64) -> f64 {
    let ex = 0f64.max((x - (left + width / 2.0)).abs() - width / 2.0);
    let ey = 0f64.max((y - (top + height / 2.0)).abs() - height / 2.0);
    (ex * ex + ey * ey).sqrt()
}

/// The distance from (x, y) to a measured rectangle's edge, 0 inside it.
pub fn distance_to(r: &DomRect, x: f64, y: f64) -> f64 {
    distance_to_box(r.left(), r.top(), r.width(), r.height(), x, y)
}

/// Whether an approach effect owns the pointer. The header's Mods and Graph
/// pills sit side by side and each reacts to a mouse within reach of it, so
/// only the nearer one does: the two never move at once. `wins_ties` gives
/// one of them an exact tie.
pub fn owns_approach(own: f64, rival: Option<f64>, wins_ties: bool) -> bool {
    match rival {
        None => true,
        Some(rival) if wins_ties => own <= rival,
        Some(rival) => own < rival,
    }
}

pub fn window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("no window"))
}

pub fn document() -> Result<Document, JsValue> {
    window()?.document().ok_or_else(|| JsValue::from_str("no document"))
}

pub fn body() -> Result<HtmlElement, JsValue> {
    document()?.body().ok_or_else(|| JsValue::from_str("no body"))
}

/// A NodeList as elements, in document order.
pub fn elements(list: NodeList) -> Vec<Element> {
    (0..list.length()).filter_map(|i| list.item(i)?.dyn_into::<Element>().ok()).collect()
}

pub fn select_all(selector: &str) -> Result<Vec<Element>, JsValue> {
    Ok(elements(document()?.query_selector_all(selector)?))
}

pub fn select_within(root: &Element, selector: &str) -> Result<Vec<Element>, JsValue> {
    Ok(elements(root.query_selector_all(selector)?))
}

pub fn html(el: &Element) -> HtmlElement {
    el.clone().unchecked_into()
}

/// Attaches a listener for the page's lifetime (the closure is leaked on
/// purpose: like site.js's listeners, it is never removed).
pub fn on<E, F>(target: &EventTarget, kind: &str, f: F) -> Result<(), JsValue>
where
    E: FromWasmAbi + 'static,
    F: FnMut(E) + 'static,
{
    let cb = Closure::<dyn FnMut(E)>::new(f);
    target.add_event_listener_with_callback(kind, cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}

pub fn on_passive<E, F>(target: &EventTarget, kind: &str, f: F) -> Result<(), JsValue>
where
    E: FromWasmAbi + 'static,
    F: FnMut(E) + 'static,
{
    let cb = Closure::<dyn FnMut(E)>::new(f);
    let opts = AddEventListenerOptions::new();
    opts.set_passive(true);
    target.add_event_listener_with_callback_and_add_event_listener_options(kind, cb.as_ref().unchecked_ref(), &opts)?;
    cb.forget();
    Ok(())
}

/// Calls `f(is_intersecting)` as the element enters and leaves the viewport;
/// never, where the browser has no IntersectionObserver.
pub fn on_visible(target: &Element, f: impl FnMut(bool) + 'static) -> Result<(), JsValue> {
    if !truthy(&window()?, "IntersectionObserver") {
        return Ok(());
    }
    let mut f = f;
    let cb = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        f(Reflect::get(&entries.get(0), &"isIntersecting".into()).map(|v| v.is_truthy()).unwrap_or(false));
    });
    web_sys::IntersectionObserver::new(cb.as_ref().unchecked_ref())?.observe(target);
    cb.forget();
    Ok(())
}

/// Calls `f` whenever the element is resized (if ResizeObserver exists).
pub fn on_resize(target: &Element, f: impl FnMut() + 'static) -> Result<(), JsValue> {
    if !truthy(&window()?, "ResizeObserver") {
        return Ok(());
    }
    let cb = Closure::<dyn FnMut()>::new(f);
    web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref())?.observe(target);
    cb.forget();
    Ok(())
}

/// Calls `f` when <html data-theme> changes.
pub fn on_theme(f: impl FnMut() + 'static) -> Result<(), JsValue> {
    let cb = Closure::<dyn FnMut()>::new(f);
    let observer = MutationObserver::new(cb.as_ref().unchecked_ref())?;
    cb.forget();
    let init = MutationObserverInit::new();
    init.set_attributes(true);
    init.set_attribute_filter(&js_sys::Array::of1(&"data-theme".into()));
    let root = document()?.document_element().ok_or_else(|| JsValue::from_str("no root"))?;
    observer.observe_with_options(&root, &init)
}

/// `document.fonts.ready.then(f)`, or `f` at once without the API.
pub fn when_fonts_ready(f: impl FnOnce() + 'static) -> Result<(), JsValue> {
    let doc = document()?;
    let fonts = Reflect::get(&doc, &"fonts".into()).unwrap_or(JsValue::UNDEFINED);
    let ready = if fonts.is_truthy() { Reflect::get(&fonts, &"ready".into()).unwrap_or(JsValue::UNDEFINED) } else { JsValue::UNDEFINED };
    match ready.dyn_into::<js_sys::Promise>() {
        Ok(p) => {
            let cb = Closure::once_into_js(move |_: JsValue| f());
            then(&p, &cb, None);
        }
        Err(_) => f(),
    }
    Ok(())
}

/// `window.matchMedia(q).matches`, false when matchMedia is missing.
pub fn media(query: &str) -> bool {
    let Ok(w) = window() else { return false };
    let has = Reflect::get(&w, &"matchMedia".into()).map(|f| f.is_function()).unwrap_or(false);
    has && matches!(w.match_media(query), Ok(Some(m)) if m.matches())
}

pub fn prefers_reduced_motion() -> bool {
    media("(prefers-reduced-motion: reduce)")
}

/// Whether `obj[name]` is truthy, as `if (window.fetch)` asks.
pub fn truthy(obj: &impl AsRef<JsValue>, name: &str) -> bool {
    Reflect::get(obj.as_ref(), &name.into()).map(|v| v.is_truthy()).unwrap_or(false)
}

/// `promise.then(ok, fail)` with plain JS functions (one-shot closures made
/// by `Closure::once_into_js`, or a long-lived closure's function).
pub fn then(promise: &JsValue, ok: &JsValue, fail: Option<&JsValue>) {
    let Ok(f) = Reflect::get(promise, &"then".into()).and_then(|f| f.dyn_into::<Function>()) else {
        return;
    };
    let _ = match fail {
        Some(fail) => f.call2(promise, ok, fail),
        None => f.call1(promise, ok),
    };
}

/// A numeric property read as a double (clientX, say, which web-sys types
/// as an integer although browsers report fractions).
pub fn num(obj: &JsValue, name: &str) -> f64 {
    Reflect::get(obj, &name.into()).ok().and_then(|v| v.as_f64()).unwrap_or(f64::NAN)
}

/// `window.setTimeout(f, ms)` with the delay passed through as a double,
/// exactly as site.js passes it.
pub fn set_timeout(f: impl FnOnce() + 'static, ms: f64) -> JsValue {
    let cb = Closure::once_into_js(f);
    call_window("setTimeout", &[cb, JsValue::from_f64(ms)])
}

pub fn clear_timeout(id: &JsValue) {
    call_window("clearTimeout", std::slice::from_ref(id));
}

fn call_window(name: &str, args: &[JsValue]) -> JsValue {
    let Ok(w) = window() else { return JsValue::UNDEFINED };
    let Ok(f) = Reflect::get(&w, &name.into()).and_then(|f| f.dyn_into::<Function>()) else {
        return JsValue::UNDEFINED;
    };
    let list: js_sys::Array = args.iter().collect();
    f.apply(&w, &list).unwrap_or(JsValue::UNDEFINED)
}

/// A long-lived animation-frame (or promise) callback.
pub type FrameCallback = Closure<dyn FnMut(f64)>;
pub type Slot<T> = std::cell::RefCell<Option<T>>;

/// `requestAnimationFrame` through `window`, so a page-level override sees it.
pub fn request_frame(cb: &FrameCallback) -> i32 {
    window().and_then(|w| w.request_animation_frame(cb.as_ref().unchecked_ref())).unwrap_or(0)
}

pub fn performance_now() -> f64 {
    window().ok().and_then(|w| w.performance()).map(|p| p.now()).unwrap_or(0.0)
}

/// `String(x)` for a JS number.
pub fn js_num(x: f64) -> String {
    js_sys::Number::from(x).to_string_with_radix(10).map(String::from).unwrap_or_default()
}

/// `x.toFixed(digits)`.
pub fn to_fixed(x: f64, digits: u8) -> String {
    js_sys::Number::from(x).to_fixed(digits).map(String::from).unwrap_or_default()
}

/// `Math.round`: halves go up, unlike `f64::round`.
pub fn js_round(x: f64) -> f64 {
    js_sys::Math::round(x)
}

/// `String.prototype.trim`.
pub fn js_trim(s: &str) -> String {
    String::from(js_sys::JsString::from(s).trim())
}

/// site.js's `formatTemplate`: `{key}` replaced from `values`, unknown keys
/// (and anything that is not `{` word-characters `}`) left as written.
pub fn format_template(template: &str, values: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let word = after.bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'_').count();
        if word > 0 && after.as_bytes().get(word) == Some(&b'}') {
            let key = &after[..word];
            match values.iter().find(|(k, _)| *k == key) {
                Some((_, v)) => out.push_str(v),
                None => out.push_str(&rest[open..open + word + 2]),
            }
            rest = &after[word + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// The shared sprite's URL with another fragment: every page paints the
/// wordmark from it, so the first `svg.icon use` carries base path and
/// fingerprint. Empty when no icon is on the page.
pub fn sprite_href(symbol: &str) -> String {
    let href = document()
        .ok()
        .and_then(|d| d.query_selector("svg.icon use").ok().flatten())
        .and_then(|u| u.get_attribute("href"))
        .unwrap_or_default();
    let base = href.split('#').next().unwrap_or("");
    if base.is_empty() { String::new() } else { format!("{base}#{symbol}") }
}

pub fn icon_markup(symbol: &str) -> String {
    let href = sprite_href(symbol);
    if href.is_empty() {
        return String::new();
    }
    format!("<svg class=\"icon\" aria-hidden=\"true\"><use href=\"{href}\"></use></svg>")
}

#[cfg(test)]
mod tests {
    use super::format_template;

    #[test]
    fn templates_fill_known_keys_and_keep_the_rest() {
        let v = [("count", "3".to_string()), ("total", "9".to_string())];
        assert_eq!(format_template("{count} of {total}", &v), "3 of 9");
        assert_eq!(format_template("{count} {nope} {} {a-b} {", &v), "3 {nope} {} {a-b} {");
        assert_eq!(format_template("{{count}}", &v), "{3}");
    }
}
