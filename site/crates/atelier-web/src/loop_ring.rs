//! The loop ring on /mods/climb/: how a climb goes round, played from the
//! live run's own log.
//!
//! The markup draws the ring finished: four stations, the exit spoke to the
//! goal, the traveller resting where the run ended and the centre naming
//! its last iteration. Here a traveller goes round once per logged
//! iteration: it pauses at each station (the matching step in the list
//! lights with it), the centre takes the iteration's exit code and score
//! when the check station is reached, and on a passing iteration the
//! traveller leaves the ring along the spoke and the goal flares. Then it
//! starts again. Frames run only while the ring is on screen and the tab is
//! visible. Off under reduced motion, which keeps the finished drawing.

use crate::dom::{
    FrameCallback, Slot, document, on_visible, prefers_reduced_motion, request_frame,
    select_within, to_fixed,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Element;

/// The ring's geometry, as the build draws it (pages/mods.rs).
const CX: f64 = 160.0;
const CY: f64 = 160.0;
const R: f64 = 104.0;
const GOAL_X: f64 = 14.0;

/// Seconds: a pause at a station, a quarter of the ring, the spoke, and the
/// hold at the goal before it starts again.
const DWELL: f64 = 0.65;
const ARC: f64 = 0.85;
const SPOKE: f64 = 0.6;
const HOLD: f64 = 2.4;

/// One iteration of the log: its number, exit code and score as written.
#[derive(Debug, Clone, PartialEq)]
struct Iteration {
    number: String,
    exit: String,
    score: String,
}

/// `1:1:5 2:0:12` -> iterations.
fn parse_rows(text: &str) -> Vec<Iteration> {
    text.split_whitespace()
        .filter_map(|row| {
            let mut parts = row.split(':');
            Some(Iteration {
                number: parts.next()?.to_string(),
                exit: parts.next()?.to_string(),
                score: parts.next().unwrap_or("").to_string(),
            })
        })
        .collect()
}

/// Where the traveller is in the timeline.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Place {
    /// Waiting at a station (0 top, clockwise).
    At(usize),
    /// Between `from` and the next station, `t` of the way.
    Arc { from: usize, t: f64 },
    /// On the spoke from the last station to the goal.
    Spoke(f64),
    /// At the goal, the climb over.
    Goal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Moment {
    /// Which iteration's lap this is.
    lap: usize,
    place: Place,
    /// Whether the check of this lap has run (its exit and score are known).
    checked: bool,
}

/// The timeline: one lap per iteration, each a pause at every station and a
/// quarter turn between them; a passing lap leaves along the spoke and holds
/// at the goal, which ends the timeline.
fn segments(rows: &[Iteration]) -> Vec<(f64, usize, Place)> {
    let mut out = Vec::new();
    for (lap, row) in rows.iter().enumerate() {
        for station in 0..4 {
            out.push((DWELL, lap, Place::At(station)));
            if station == 3 && row.exit == "0" {
                out.push((SPOKE, lap, Place::Spoke(0.0)));
                out.push((HOLD, lap, Place::Goal));
                return out;
            }
            out.push((
                ARC,
                lap,
                Place::Arc {
                    from: station,
                    t: 0.0,
                },
            ));
        }
    }
    out
}

fn moment(segments: &[(f64, usize, Place)], elapsed: f64) -> Moment {
    let total: f64 = segments.iter().map(|s| s.0).sum();
    let mut t = if total > 0.0 { elapsed % total } else { 0.0 };
    for &(dur, lap, place) in segments {
        if t < dur {
            let share = t / dur;
            let place = match place {
                Place::Arc { from, .. } => Place::Arc { from, t: share },
                Place::Spoke(_) => Place::Spoke(share),
                other => other,
            };
            let checked = match place {
                Place::At(s) => s >= 1,
                Place::Arc { from, .. } => from >= 1,
                _ => true,
            };
            return Moment {
                lap,
                place,
                checked,
            };
        }
        t -= dur;
    }
    Moment {
        lap: 0,
        place: Place::At(0),
        checked: false,
    }
}

/// Ease in and out over one move.
fn ease(t: f64) -> f64 {
    0.5 - 0.5 * (std::f64::consts::PI * t).cos()
}

fn station_point(station: usize) -> (f64, f64) {
    let angle = (-90.0 + 90.0 * station as f64).to_radians();
    (CX + R * angle.cos(), CY + R * angle.sin())
}

fn point(place: Place) -> (f64, f64) {
    match place {
        Place::At(s) => station_point(s),
        Place::Arc { from, t } => {
            let angle = (-90.0 + 90.0 * (from as f64 + ease(t))).to_radians();
            (CX + R * angle.cos(), CY + R * angle.sin())
        }
        Place::Spoke(t) => {
            let (x, y) = station_point(3);
            (x + (GOAL_X - x) * ease(t), y)
        }
        Place::Goal => (GOAL_X, CY),
    }
}

struct Ring {
    root: Element,
    traveller: Element,
    stations: Vec<Element>,
    steps: Vec<Element>,
    centre: Option<Element>,
    detail: Option<Element>,
    rows: Vec<Iteration>,
    segments: Vec<(f64, usize, Place)>,
    elapsed: Cell<f64>,
    last: Cell<f64>,
    raf: Cell<i32>,
    visible: Cell<bool>,
    shown: RefCell<Option<Moment>>,
    frame: Slot<FrameCallback>,
}

impl Ring {
    fn run(&self) {
        if self.raf.get() == 0
            && self.visible.get()
            && let Some(cb) = self.frame.borrow().as_ref()
        {
            self.raf.set(request_frame(cb));
        }
    }

    /// Moves the traveller to `m`, and updates the stations, the steps and
    /// the centre when the discrete state (lap, station, checked) changed.
    fn draw(&self, m: Moment) {
        let (x, y) = point(m.place);
        let _ = self.traveller.set_attribute("cx", &to_fixed(x, 2));
        let _ = self.traveller.set_attribute("cy", &to_fixed(y, 2));
        let state = Moment {
            place: match m.place {
                Place::Arc { from, .. } => Place::Arc { from, t: 0.0 },
                Place::Spoke(_) => Place::Spoke(0.0),
                other => other,
            },
            ..m
        };
        if *self.shown.borrow() == Some(state) {
            return;
        }
        *self.shown.borrow_mut() = Some(state);
        let active = match m.place {
            Place::At(s) => Some(s),
            _ => None,
        };
        for (i, el) in self.stations.iter().chain(&self.steps).enumerate() {
            let _ = el
                .class_list()
                .toggle_with_force("is-active", active == Some(i % 4));
        }
        let _ = self
            .root
            .class_list()
            .toggle_with_force("is-met", m.place == Place::Goal);
        let row = &self.rows[m.lap];
        if let Some(centre) = &self.centre {
            centre.set_text_content(Some(&format!("iteration {}", row.number)));
        }
        if let Some(detail) = &self.detail {
            let text = if m.checked {
                let score = if row.score.is_empty() {
                    String::new()
                } else {
                    format!(" · score {}", row.score)
                };
                format!("exit {}{score}", row.exit)
            } else {
                "one change".to_string()
            };
            detail.set_text_content(Some(&text));
        }
    }

    fn frame(&self, time: f64) {
        self.raf.set(0);
        if !self.visible.get() || document().map(|d| d.hidden()).unwrap_or(false) {
            self.last.set(0.0);
            return;
        }
        let last = if self.last.get() != 0.0 {
            self.last.get()
        } else {
            time
        };
        self.elapsed
            .set(self.elapsed.get() + 0.05f64.min((time - last) / 1000.0));
        self.last.set(time);
        self.draw(moment(&self.segments, self.elapsed.get()));
        self.run();
    }
}

pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let Some(root) = doc.query_selector("[data-loop-ring]")? else {
        return Ok(());
    };
    if prefers_reduced_motion() {
        return Ok(());
    }
    let rows = parse_rows(&root.get_attribute("data-loop-rows").unwrap_or_default());
    let Some(traveller) = root.query_selector(".loop-ring__traveller")? else {
        return Ok(());
    };
    if rows.is_empty() {
        return Ok(());
    }
    let ring = Rc::new(Ring {
        stations: select_within(&root, ".loop-ring__station")?,
        steps: select_within(&root, ".loop-ring__step")?,
        centre: root.query_selector("[data-loop-centre]")?,
        detail: root.query_selector("[data-loop-detail]")?,
        segments: segments(&rows),
        rows,
        traveller,
        root: root.clone(),
        elapsed: Cell::new(0.0),
        last: Cell::new(0.0),
        raf: Cell::new(0),
        visible: Cell::new(true),
        shown: RefCell::new(None),
        frame: RefCell::new(None),
    });
    let _ = root.class_list().add_1("is-running");
    ring.draw(moment(&ring.segments, 0.0));
    let weak = Rc::downgrade(&ring);
    *ring.frame.borrow_mut() = Some(Closure::new(move |time: f64| {
        if let Some(r) = weak.upgrade() {
            r.frame(time);
        }
    }));
    let r = ring.clone();
    on_visible(&root, move |visible| {
        r.visible.set(visible);
        if visible {
            r.last.set(0.0);
            r.run();
        }
    })?;
    let r = ring.clone();
    crate::dom::on(&doc, "visibilitychange", move |_: web_sys::Event| {
        r.last.set(0.0);
        r.run();
    })?;
    ring.run();
    std::mem::forget(ring);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Iteration> {
        parse_rows("1:1:5 2:0:12")
    }

    #[test]
    fn rows_parse_from_the_attribute() {
        assert_eq!(
            rows()[1],
            Iteration {
                number: "2".into(),
                exit: "0".into(),
                score: "12".into()
            }
        );
        assert_eq!(parse_rows("3:1:")[0].score, "");
    }

    #[test]
    fn a_failing_lap_goes_round_and_a_passing_one_leaves_by_the_spoke() {
        let segs = segments(&rows());
        // Lap 1: 4 dwells + 4 arcs; lap 2: 4 dwells + 3 arcs + spoke + hold.
        assert_eq!(segs.len(), 8 + 9);
        assert_eq!(segs.last().unwrap().2, Place::Goal);
        assert!(segs.iter().all(|s| s.1 < 2));
    }

    #[test]
    fn the_check_result_shows_from_the_check_station_on() {
        let segs = segments(&rows());
        let start = moment(&segs, 0.0);
        assert_eq!(
            start,
            Moment {
                lap: 0,
                place: Place::At(0),
                checked: false
            }
        );
        let at_check = moment(&segs, DWELL + ARC + 0.01);
        assert_eq!(at_check.place, Place::At(1));
        assert!(at_check.checked);
        let total: f64 = segs.iter().map(|s| s.0).sum();
        assert_eq!(moment(&segs, total - 0.01).place, Place::Goal);
        // And then it starts again.
        assert_eq!(moment(&segs, total + 0.01).place, Place::At(0));
    }

    #[test]
    fn stations_sit_on_the_ring_clockwise_from_the_top() {
        let close =
            |(x, y): (f64, f64), (a, b): (f64, f64)| (x - a).abs() < 1e-9 && (y - b).abs() < 1e-9;
        assert!(close(station_point(0), (160.0, 56.0)));
        assert!(close(station_point(1), (264.0, 160.0)));
        assert!(close(station_point(3), (56.0, 160.0)));
        assert!(close(point(Place::Goal), (GOAL_X, 160.0)));
    }
}
