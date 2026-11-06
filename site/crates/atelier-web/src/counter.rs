//! The landing page's counter strip: four previews, each a small working
//! copy of where it leads.
//!
//! The markup is every figure finished: the skill count in full with one
//! name under it, the climb drawn to the bar, the constellation at rest, the
//! three steps ticked. That is what a reader without WASM, or with reduced
//! motion, sees, and nothing here removes any of it.
//!
//! Here the strip is marked live (`is-live`) while it is on screen and the
//! tab is visible; landing.css runs the climb, the breathing and the ticking
//! only under that class, so they stop off screen. The first time the strip
//! goes live the numeral counts up to the count it already shows, and while
//! it is live the name under it cycles through every skill.

use crate::dom::{document, on, on_visible, prefers_reduced_motion, set_timeout};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlElement};

/// How long the numeral takes to count up, and in how many steps.
const COUNT_MS: f64 = 1100.0;
const COUNT_STEPS: u32 = 28;
/// How long each name stays before the next one comes in.
const NAME_MS: f64 = 2200.0;

struct Counter {
    strip: Element,
    numeral: Option<Element>,
    target: u32,
    name: Option<HtmlElement>,
    names: Vec<String>,
    next_name: Cell<usize>,
    on_screen: Cell<bool>,
    live: Cell<bool>,
    counted: Cell<bool>,
    /// Bumped whenever the strip stops being live, so a name timer from an
    /// earlier spell does nothing.
    generation: Cell<u32>,
}

/// The value the numeral shows `share` of the way through its count:
/// ease-out cubic, so it slows into the real number, and never past it.
pub fn count_at(target: u32, share: f64) -> u32 {
    let s = share.clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - s).powi(3);
    ((f64::from(target) * eased).round() as u32).min(target)
}

/// The names a `data-counter-names` attribute carries, in order.
pub fn parse_names(attr: &str) -> Vec<String> {
    attr.split('|')
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .collect()
}

fn page_hidden() -> bool {
    document().map(|d| d.hidden()).unwrap_or(false)
}

impl Counter {
    fn update(self: &Rc<Self>) {
        let live = self.on_screen.get() && !page_hidden();
        if live == self.live.get() {
            return;
        }
        self.live.set(live);
        let _ = self.strip.class_list().toggle_with_force("is-live", live);
        self.generation.set(self.generation.get() + 1);
        if live {
            if !self.counted.get() {
                self.counted.set(true);
                self.count_up();
            }
            self.schedule_name(self.generation.get());
        }
    }

    fn count_up(self: &Rc<Self>) {
        let Some(numeral) = self.numeral.clone() else {
            return;
        };
        for step in 0..=COUNT_STEPS {
            let (me, numeral) = (self.clone(), numeral.clone());
            let share = f64::from(step) / f64::from(COUNT_STEPS);
            set_timeout(
                move || {
                    numeral.set_text_content(Some(&count_at(me.target, share).to_string()));
                },
                share * COUNT_MS,
            );
        }
    }

    fn schedule_name(self: &Rc<Self>, generation: u32) {
        if self.name.is_none() || self.names.len() < 2 {
            return;
        }
        let me = self.clone();
        set_timeout(
            move || {
                if me.generation.get() != generation || !me.live.get() {
                    return;
                }
                me.show_next_name();
                me.schedule_name(generation);
            },
            NAME_MS,
        );
    }

    fn show_next_name(&self) {
        let Some(name) = &self.name else { return };
        let index = self.next_name.get() % self.names.len();
        self.next_name.set(index + 1);
        name.set_text_content(Some(&self.names[index]));
        // Restart the entrance: drop the class, let layout see it gone, add
        // it back.
        let _ = name.class_list().remove_1("is-in");
        let _ = name.offset_width();
        let _ = name.class_list().add_1("is-in");
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(strip) = doc.query_selector("[data-counter]")? else {
        return Ok(());
    };
    if prefers_reduced_motion() {
        return Ok(());
    }
    let numeral = strip.query_selector("[data-counter-count]")?;
    let target = numeral
        .as_ref()
        .and_then(|n| n.get_attribute("data-counter-count"))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let name_el = strip.query_selector("[data-counter-names]")?;
    let names = name_el
        .as_ref()
        .and_then(|n| n.get_attribute("data-counter-names"))
        .map(|a| parse_names(&a))
        .unwrap_or_default();
    // The name already showing is the first; the cycle starts at the second.
    let counter = Rc::new(Counter {
        strip: strip.clone(),
        numeral: if target > 0 { numeral } else { None },
        target,
        name: name_el.map(|n| n.unchecked_into()),
        names,
        next_name: Cell::new(1),
        on_screen: Cell::new(false),
        live: Cell::new(false),
        counted: Cell::new(false),
        generation: Cell::new(0),
    });
    let c = counter.clone();
    on_visible(&strip, move |visible| {
        c.on_screen.set(visible);
        c.update();
    })?;
    let c = counter.clone();
    on(&doc, "visibilitychange", move |_: Event| c.update())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_rises_from_zero_to_the_real_number_and_never_past_it() {
        assert_eq!(count_at(102, 0.0), 0);
        assert_eq!(count_at(102, 1.0), 102);
        assert_eq!(count_at(102, 2.0), 102);
        let steps: Vec<u32> = (0..=COUNT_STEPS)
            .map(|s| count_at(102, f64::from(s) / f64::from(COUNT_STEPS)))
            .collect();
        assert!(steps.windows(2).all(|w| w[0] <= w[1]), "{steps:?}");
        // Ease-out: past halfway in value well before halfway in time.
        assert!(count_at(102, 0.3) > 51);
    }

    #[test]
    fn names_are_read_in_order_without_blanks() {
        assert_eq!(parse_names("a|b| c |"), ["a", "b", "c"]);
        assert!(parse_names("").is_empty());
    }
}
