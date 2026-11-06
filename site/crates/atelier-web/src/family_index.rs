//! The landing page's family index: one link per family, beside the skills
//! (a sticky column at desktop widths, a strip of chips on a phone).
//!
//! Without this module the index is a plain list of anchors, and every link
//! still jumps to its family. Here it follows the page: the family whose
//! section holds the reading line (a third of the way down the viewport) is
//! marked `aria-current="true"`, and on the phone strip that chip is kept in
//! view. It follows the search too: each count becomes the family's skills
//! still showing, and a family the search has emptied is marked `is-empty`
//! (it stays in the list, so the index never jumps while the reader types).

use crate::dom::{document, html, on_passive, select_within, window};
use js_sys::Array;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlElement, MutationObserver, MutationObserverInit};

/// The reading line, as a share of the viewport's height from its top.
const READING_LINE: f64 = 0.3;

struct Family {
    link: HtmlElement,
    section: HtmlElement,
    count: Option<Element>,
}

struct Index {
    list: Option<HtmlElement>,
    families: Vec<Family>,
    active: Cell<Option<usize>>,
    frame_pending: Cell<bool>,
}

/// Which family holds the reading line: the last one (of those showing)
/// whose top has reached it, as long as the reader is not already past its
/// bottom. `spans` is each family's (top, bottom, showing) in viewport px.
pub fn family_at(spans: &[(f64, f64, bool)], line: f64) -> Option<usize> {
    let mut found = None;
    for (i, &(top, bottom, showing)) in spans.iter().enumerate() {
        if showing && top <= line {
            found = Some((i, bottom));
        }
    }
    match found {
        Some((i, bottom)) if bottom > 0.0 => Some(i),
        _ => None,
    }
}

impl Index {
    fn pick(&self) {
        let Ok(w) = window() else { return };
        let height = w
            .inner_height()
            .ok()
            .and_then(|h| h.as_f64())
            .unwrap_or(0.0);
        let spans: Vec<(f64, f64, bool)> = self
            .families
            .iter()
            .map(|f| {
                let rect = f.section.get_bounding_client_rect();
                (rect.top(), rect.bottom(), !f.section.hidden())
            })
            .collect();
        let active = family_at(&spans, height * READING_LINE);
        if active == self.active.get() {
            return;
        }
        self.active.set(active);
        for (i, f) in self.families.iter().enumerate() {
            if Some(i) == active {
                let _ = f.link.set_attribute("aria-current", "true");
            } else {
                let _ = f.link.remove_attribute("aria-current");
            }
        }
        if let Some(i) = active {
            self.reveal(i);
        }
    }

    /// On the phone strip, scrolls the strip itself (never the page) so the
    /// active chip is in view.
    fn reveal(&self, i: usize) {
        let Some(list) = &self.list else { return };
        if list.scroll_width() <= list.client_width() {
            return;
        }
        let Some(item) = self.families[i].link.parent_element() else {
            return;
        };
        let item: HtmlElement = item.unchecked_into();
        let (left, width) = (
            f64::from(item.offset_left()),
            f64::from(item.offset_width()),
        );
        let (scroll, view) = (
            f64::from(list.scroll_left()),
            f64::from(list.client_width()),
        );
        let pad = 24.0;
        if left < scroll + pad {
            list.set_scroll_left((left - pad).max(0.0) as i32);
        } else if left + width > scroll + view - pad {
            list.set_scroll_left((left + width - view + pad) as i32);
        }
    }

    /// The counts and the emptied families, after the search has run.
    fn recount(&self) {
        for f in &self.families {
            let showing = select_within(&f.section, "[data-search-card]:not([hidden])")
                .map(|v| v.len())
                .unwrap_or(0);
            if let Some(count) = &f.count {
                count.set_text_content(Some(&showing.to_string()));
            }
            let _ = f
                .link
                .class_list()
                .toggle_with_force("is-empty", f.section.hidden() || showing == 0);
        }
    }

    fn schedule(self: &Rc<Self>) {
        if self.frame_pending.replace(true) {
            return;
        }
        let me = self.clone();
        let cb = Closure::once_into_js(move || {
            me.frame_pending.set(false);
            me.pick();
        });
        if let Ok(w) = window() {
            let _ = w.request_animation_frame(cb.unchecked_ref());
        }
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(index) = doc.query_selector("[data-family-index]")? else {
        return Ok(());
    };
    let mut families = Vec::new();
    for link in select_within(&index, "[data-family-link]")? {
        let Some(id) = link
            .get_attribute("href")
            .and_then(|h| h.strip_prefix('#').map(str::to_string))
        else {
            continue;
        };
        let Some(section) = doc.get_element_by_id(&id) else {
            continue;
        };
        families.push(Family {
            count: link.query_selector("[data-family-count]")?,
            link: html(&link),
            section: html(&section),
        });
    }
    if families.is_empty() {
        return Ok(());
    }
    let list = index
        .query_selector(".family-index__list")?
        .map(|l| html(&l));
    let state = Rc::new(Index {
        list,
        families,
        active: Cell::new(None),
        frame_pending: Cell::new(false),
    });

    let w = window()?;
    let s = state.clone();
    on_passive(&w, "scroll", move |_: Event| s.schedule())?;
    let s = state.clone();
    on_passive(&w, "resize", move |_: Event| s.schedule())?;

    // The search hides cards and sections by their `hidden` attribute.
    if let Some(grid) = doc.query_selector("[data-categories]")? {
        let s = state.clone();
        let observer = Closure::<dyn FnMut()>::new(move || {
            s.recount();
            s.schedule();
        });
        let watch = MutationObserver::new(observer.as_ref().unchecked_ref())?;
        observer.forget();
        let init = MutationObserverInit::new();
        init.set_attributes(true);
        init.set_subtree(true);
        init.set_attribute_filter(&Array::of1(&"hidden".into()));
        watch.observe_with_options(&grid, &init)?;
    }
    state.pick();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::family_at;

    #[test]
    fn the_family_holding_the_reading_line_is_the_one_in_view() {
        let spans = [
            (-900.0, -100.0, true),
            (-100.0, 400.0, true),
            (400.0, 900.0, true),
        ];
        assert_eq!(family_at(&spans, 270.0), Some(1));
        assert_eq!(family_at(&spans, 500.0), Some(2));
        // Above the first family: none yet.
        assert_eq!(family_at(&[(300.0, 800.0, true)], 270.0), None);
        // Past the last family's bottom (in the footer): none.
        assert_eq!(family_at(&[(-900.0, -10.0, true)], 270.0), None);
    }

    #[test]
    fn a_family_the_search_hid_is_skipped() {
        let spans = [
            (-500.0, -100.0, true),
            (-100.0, -100.0, false),
            (-100.0, 600.0, true),
        ];
        assert_eq!(family_at(&spans, 270.0), Some(2));
        let spans = [(-500.0, 300.0, true), (300.0, 300.0, false)];
        assert_eq!(family_at(&spans, 270.0), Some(0));
    }
}
