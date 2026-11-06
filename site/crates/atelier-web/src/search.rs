//! Landing-page search (site.js initSearch): on every input, a plain
//! lower-cased substring test against each card's pre-lowercased
//! `data-search-terms`; sections with nothing left hide, their badges recount,
//! the status line and empty state follow. No storage, no URL, no shortcut.

use crate::dom::{body, document, format_template, html, js_num, js_trim, on, select_all, select_within};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlElement, HtmlInputElement};

struct Search {
    input: HtmlInputElement,
    clear: HtmlElement,
    status: Element,
    empty: Option<HtmlElement>,
    empty_query: Option<Element>,
    cards: Vec<HtmlElement>,
    sections: Vec<HtmlElement>,
}

impl Search {
    fn apply(&self) {
        let raw = js_trim(&self.input.value());
        let query = String::from(js_sys::JsString::from(raw.as_str()).to_lower_case());
        let mut visible = 0;
        for card in &self.cards {
            let haystack = card.get_attribute("data-search-terms").unwrap_or_default();
            let matches = query.is_empty() || haystack.contains(&query);
            card.set_hidden(!matches);
            if matches {
                visible += 1;
            }
        }
        for section in &self.sections {
            let still = select_within(section, "[data-search-card]:not([hidden])").map(|v| v.len()).unwrap_or(0);
            section.set_hidden(!query.is_empty() && still == 0);
            if let Ok(Some(badge)) = section.query_selector("[data-category-count]") {
                let total = js_sys::parse_int(&badge.get_attribute("data-category-count").unwrap_or_default(), 10);
                let shown = if query.is_empty() { total } else { still as f64 };
                let noun_attr = if shown == 1.0 { "data-i18n-singular" } else { "data-i18n-plural" };
                let noun = badge.get_attribute(noun_attr).unwrap_or_else(|| "null".into());
                badge.set_text_content(Some(&format!("{} {}", js_num(shown), noun)));
            }
        }
        self.clear.set_hidden(query.is_empty());
        let show_empty = !query.is_empty() && visible == 0;
        if let Some(empty) = &self.empty {
            empty.set_hidden(!show_empty);
            if let (true, Some(eq)) = (show_empty, &self.empty_query) {
                let b = body().ok();
                let open = b.as_ref().and_then(|b| b.get_attribute("data-i18n-quote-open")).filter(|s| !s.is_empty());
                let close = b.as_ref().and_then(|b| b.get_attribute("data-i18n-quote-close")).filter(|s| !s.is_empty());
                eq.set_text_content(Some(&format!(
                    "{}{}{}",
                    open.as_deref().unwrap_or("\u{201c}"),
                    raw,
                    close.as_deref().unwrap_or("\u{201d}")
                )));
            }
        }
        let total = self.cards.len().to_string();
        let text = if query.is_empty() {
            let t = self.status.get_attribute("data-i18n-showing-all-template").unwrap_or_default();
            Some(format_template(&t, &[("count", total)]))
        } else if visible == 0 {
            body().ok().and_then(|b| b.get_attribute("data-i18n-no-match"))
        } else {
            let t = self.status.get_attribute("data-i18n-partial-template").unwrap_or_default();
            Some(format_template(&t, &[("shown", visible.to_string()), ("total", total)]))
        };
        self.status.set_text_content(text.as_deref());
    }

    fn reset(&self) {
        self.input.set_value("");
        self.apply();
        let _ = self.input.focus();
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(root) = doc.query_selector("[data-search-root]")? else { return Ok(()) };
    let need = |sel: &str| root.query_selector(sel).ok().flatten().ok_or_else(|| JsValue::from_str(&format!("search: no {sel}")));
    let input: HtmlInputElement = need("[data-search-input]")?.unchecked_into();
    let clear = html(&need("[data-search-clear]")?);
    let status = need("[data-search-status]")?;
    let empty = doc.query_selector("[data-search-empty]")?.map(|e| html(&e));
    let empty_query = match &empty { Some(e) => e.query_selector("[data-search-empty-query]")?, None => None };
    let reset_button = match &empty { Some(e) => e.query_selector("[data-search-reset]")?, None => None };
    let search = Rc::new(Search {
        input,
        clear,
        status,
        empty,
        empty_query,
        cards: select_all("[data-search-card]")?.iter().map(html).collect(),
        sections: select_all("[data-category-section]")?.iter().map(html).collect(),
    });
    let s = search.clone();
    on(&search.input, "input", move |_: Event| s.apply())?;
    let s = search.clone();
    on(&search.clear, "click", move |_: Event| s.reset())?;
    if let Some(button) = reset_button {
        let s = search.clone();
        on(&button, "click", move |_: Event| s.reset())?;
    }
    search.apply();
    Ok(())
}
