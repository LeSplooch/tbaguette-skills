//! The live replay on /mods/climb/: a mock of the Claude Code session that
//! ran the first live climb, drawn beat by beat from its real log.
//!
//! The page's markup is the finished replay: every line shown, the band at
//! its last state, the score meter full at the bar, both log rows. That is
//! what a reader without WASM, or with reduced motion, sees. Here the
//! replay is armed (the lines and rows hidden, the band back at its first
//! state) and played once when it first scrolls into view: each line in
//! turn, a model turn counting its seconds, each check appending its log
//! row, moving the band and the meter, and the bar flaring when a check
//! passes. A Replay button appears at the end.
//!
//! The script is in the markup's data attributes (the build writes it from
//! the log), so this module knows nothing about the run itself.

use crate::dom::{
    document, on, on_visible, prefers_reduced_motion, select_within, set_timeout, to_fixed,
};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlElement};

/// The pause before a line, by the kind of line before it, ms: a model turn
/// takes the longest, as it did in the run (where it took ten seconds).
const FIRST_MS: f64 = 500.0;
const AFTER_MODEL_MS: f64 = 1500.0;
const AFTER_CHECK_MS: f64 = 800.0;
const AFTER_OTHER_MS: f64 = 650.0;
/// How long a model turn's seconds count up, and in how many ticks.
const COUNT_MS: f64 = 1300.0;
const COUNT_TICKS: u32 = 20;

struct Replay {
    figure: Element,
    lines: Vec<Element>,
    rows: Vec<Element>,
    band: Option<Element>,
    meter: Option<HtmlElement>,
    score: Option<Element>,
    again: Option<HtmlElement>,
    band_start: String,
    /// Bumped by every reset, so timers from an earlier play do nothing.
    generation: Cell<u32>,
}

fn attr(el: &Element, name: &str) -> Option<String> {
    el.get_attribute(name)
}

impl Replay {
    /// Back to before the first line.
    fn arm(&self) {
        self.generation.set(self.generation.get() + 1);
        let _ = self.figure.class_list().remove_1("is-met");
        let _ = self.figure.class_list().add_1("is-armed");
        for el in self.lines.iter().chain(&self.rows) {
            let _ = el.class_list().remove_2("is-shown", "is-working");
        }
        if let Some(band) = &self.band {
            band.set_text_content(Some(&self.band_start));
        }
        if let Some(meter) = &self.meter {
            let _ = meter.style().set_property("--fill", "0");
        }
        if let Some(score) = &self.score {
            score.set_text_content(Some("–"));
        }
        if let Some(again) = &self.again {
            again.set_hidden(true);
        }
    }

    fn show(self: &Rc<Self>, step: usize) {
        let line = &self.lines[step];
        let _ = line.class_list().add_1("is-shown");
        if let Some(target) = attr(line, "data-count-to").and_then(|v| v.parse::<f64>().ok()) {
            self.count(line, target);
        }
        if let Some(row) = attr(line, "data-row").and_then(|v| v.parse::<usize>().ok())
            && let Some(row) = self.rows.get(row)
        {
            let _ = row.class_list().add_1("is-shown");
        }
        if let (Some(band), Some(text)) = (&self.band, attr(line, "data-band")) {
            band.set_text_content(Some(&text));
        }
        if let Some(score) = attr(line, "data-score") {
            if let Some(el) = &self.score {
                el.set_text_content(Some(&score));
            }
            if let Some(meter) = &self.meter {
                let bar = attr(meter, "data-bar")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                if let (Ok(value), true) = (score.parse::<f64>(), bar > 0.0) {
                    let _ = meter
                        .style()
                        .set_property("--fill", &to_fixed((value / bar).clamp(0.0, 1.0), 3));
                }
            }
        }
        if line.has_attribute("data-met") {
            let _ = self.figure.class_list().add_1("is-met");
        }
    }

    /// A model turn: its "turn 0.0s" counts up to the logged seconds.
    fn count(self: &Rc<Self>, line: &Element, target: f64) {
        let Ok(Some(meta)) = line.query_selector(".climb-replay__meta") else {
            return;
        };
        let _ = line.class_list().add_1("is-working");
        let generation = self.generation.get();
        for tick in 0..=COUNT_TICKS {
            let (me, meta, line) = (self.clone(), meta.clone(), line.clone());
            let share = f64::from(tick) / f64::from(COUNT_TICKS);
            set_timeout(
                move || {
                    if me.generation.get() != generation {
                        return;
                    }
                    meta.set_text_content(Some(&format!("turn {}s", to_fixed(target * share, 1))));
                    if tick == COUNT_TICKS {
                        let _ = line.class_list().remove_1("is-working");
                    }
                },
                COUNT_MS * share,
            );
        }
    }

    fn play(self: &Rc<Self>) {
        self.arm();
        let generation = self.generation.get();
        let mut at = 0.0;
        for step in 0..self.lines.len() {
            at += match step.checked_sub(1).map(|p| &self.lines[p]) {
                None => FIRST_MS,
                Some(prev) if prev.class_list().contains("climb-replay__line--model") => {
                    AFTER_MODEL_MS
                }
                Some(prev) if prev.has_attribute("data-row") => AFTER_CHECK_MS,
                Some(_) => AFTER_OTHER_MS,
            };
            let me = self.clone();
            set_timeout(
                move || {
                    if me.generation.get() == generation {
                        me.show(step);
                    }
                },
                at,
            );
        }
        let me = self.clone();
        set_timeout(
            move || {
                if me.generation.get() == generation
                    && let Some(again) = &me.again
                {
                    again.set_hidden(false);
                }
            },
            at + AFTER_CHECK_MS,
        );
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(figure) = doc.query_selector("[data-climb-replay]")? else {
        return Ok(());
    };
    if prefers_reduced_motion() {
        return Ok(());
    }
    let lines = select_within(&figure, "[data-step]")?;
    if lines.is_empty() {
        return Ok(());
    }
    let band = figure.query_selector("[data-replay-band]")?;
    let replay = Rc::new(Replay {
        rows: select_within(&figure, "[data-log-row]")?,
        band_start: band
            .as_ref()
            .and_then(|b| attr(b, "data-band-start"))
            .unwrap_or_default(),
        band,
        meter: figure
            .query_selector("[data-replay-meter]")?
            .map(|m| m.unchecked_into()),
        score: figure.query_selector("[data-replay-score]")?,
        again: figure
            .query_selector("[data-replay-again]")?
            .map(|b| b.unchecked_into()),
        generation: Cell::new(0),
        lines,
        figure: figure.clone(),
    });
    replay.arm();
    if let Some(again) = &replay.again {
        let r = replay.clone();
        on(again, "click", move |_: Event| r.play())?;
    }
    // Played once, the first time it is on screen; at once where the browser
    // cannot say.
    let has_observer = crate::dom::truthy(&crate::dom::window()?, "IntersectionObserver");
    if has_observer {
        let r = replay.clone();
        let started = Cell::new(false);
        on_visible(&figure, move |visible| {
            if visible && !started.replace(true) {
                r.play();
            }
        })?;
    } else {
        replay.play();
    }
    Ok(())
}
