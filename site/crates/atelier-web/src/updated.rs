//! The header's "Updated" time (site.js initUpdatedTime): the baked-in UTC
//! instant shown in the visitor's own zone and in UTC through Intl, in the
//! page's language. Anything missing or throwing leaves the server's text.

use crate::dom::{format_template, select_all, window};
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::prelude::*;

/// `Intl.DateTimeFormat`, if this browser has it.
fn date_time_format() -> Option<Function> {
    let w: JsValue = window().ok()?.into();
    let intl = Reflect::get(&w, &"Intl".into()).ok().filter(|v| v.is_truthy())?;
    Reflect::get(&intl, &"DateTimeFormat".into()).ok().filter(|v| v.is_truthy())?.dyn_into().ok()
}

/// `<html lang>` if Intl accepts it as a tag, else undefined (the browser's
/// own default), so a malformed tag cannot cost the whole timestamp.
fn page_locale(dtf: &Function) -> JsValue {
    let lang = window().ok().and_then(|w| w.document()).and_then(|d| d.document_element()).and_then(|h| h.get_attribute("lang"));
    let Some(lang) = lang.filter(|l| !l.is_empty()) else { return JsValue::UNDEFINED };
    let check = Reflect::get(dtf, &"supportedLocalesOf".into()).ok().and_then(|f| f.dyn_into::<Function>().ok());
    let list = Array::of1(&JsValue::from_str(&lang));
    match check.map(|f| f.call1(dtf, &list)) {
        Some(Ok(_)) => JsValue::from_str(&lang),
        _ => JsValue::UNDEFINED,
    }
}

fn format(dtf: &Function, locale: &JsValue, options: &[(&str, JsValue)], when: &JsValue) -> Result<String, JsValue> {
    let opts = Object::new();
    for (k, v) in options {
        Reflect::set(&opts, &JsValue::from_str(k), v)?;
    }
    let formatter = Reflect::construct(dtf, &Array::of2(locale, &opts))?;
    let f: Function = Reflect::get(&formatter, &"format".into())?.dyn_into()?;
    f.call1(&formatter, when)?.as_string().ok_or_else(|| JsValue::from_str("format"))
}

/// The formatted stamp split before its UTC part (" · 21:52 UTC"), so a
/// phone can drop that part and keep the stamp on one line. A template
/// without the separator is all local.
fn split_at_utc(text: &str) -> (String, String) {
    match text.find(" \u{b7} ") {
        Some(cut) => (text[..cut].to_string(), text[cut..].to_string()),
        None => (text.to_string(), String::new()),
    }
}

/// The two parts as spans of text (never markup), the UTC part classed for
/// the stylesheet.
fn show(el: &web_sys::Element, local: &str, utc: &str) -> Result<(), JsValue> {
    let doc = crate::dom::document()?;
    el.set_text_content(Some(local));
    if !utc.is_empty() {
        let span = doc.create_element("span")?;
        span.set_class_name("site-header__updated-utc");
        span.set_text_content(Some(utc));
        el.append_child(&span)?;
    }
    Ok(())
}

pub fn init() -> Result<(), JsValue> {
    let els = select_all("[data-format-updated]")?;
    if els.is_empty() {
        return Ok(());
    }
    let Some(dtf) = date_time_format() else { return Ok(()) };
    for el in els {
        let Some(iso) = el.get_attribute("datetime").filter(|s| !s.is_empty()) else { continue };
        let when = js_sys::Date::new(&JsValue::from_str(&iso));
        if when.get_time().is_nan() {
            continue;
        }
        let when: JsValue = when.into();
        let formatted = (|| -> Result<(String, String), JsValue> {
            let locale = page_locale(&dtf);
            let local = format(&dtf, &locale, &[("dateStyle", "medium".into()), ("timeStyle", "short".into())], &when)?;
            let utc = format(
                &dtf,
                &locale,
                &[("hour", "2-digit".into()), ("minute", "2-digit".into()), ("hour12", false.into()), ("timeZone", "UTC".into())],
                &when,
            )?;
            let template = el
                .get_attribute("data-i18n-updated-template")
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "{local} your time \u{b7} {utc} UTC".into());
            Ok(split_at_utc(&format_template(&template, &[("local", local), ("utc", utc)])))
        })();
        // Unsupported options or zone in this browser: keep the server text.
        if let Ok((local, utc)) = formatted {
            let _ = show(&el, &local, &utc);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stamp_splits_before_its_utc_part() {
        assert_eq!(
            split_at_utc("Oct 29, 2026, 10:52 PM your time \u{b7} 21:52 UTC"),
            ("Oct 29, 2026, 10:52 PM your time".to_string(), " \u{b7} 21:52 UTC".to_string())
        );
        assert_eq!(split_at_utc("no separator"), ("no separator".to_string(), String::new()));
    }
}
