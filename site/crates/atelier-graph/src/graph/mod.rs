//! The skill graph ("the crumb"), ported from docs/assets/graph.js: one canvas
//! and the DOM around it, mounted into every `[data-graph-host]` on the page,
//! fed by the graph.json its `data-graph-data` names.
//!
//! Four views of one set of nodes (constellation, wheel, matrix, list) plus
//! the anatomy of one skill, all tweened; the URL hash (`#skill=`, `#view=`,
//! written with replaceState only) names what is on screen. Behaviour,
//! wording, DOM and drawing follow graph.js line by line.
//!
//! Event plumbing: every listener, timer and frame callback goes through
//! [`dispatch`], which queues the event and, unless the app is already
//! handling one, handles the queue in order. A DOM call that fires an event
//! synchronously (a `focus()` inside `openSkill`, say) therefore has its
//! handler run right after the current one finishes instead of re-entering
//! the app mid-change.

mod input;
mod layout;
mod model;
mod nav;
mod probe;
mod render;
mod text;
mod views;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::rc::{Rc, Weak};

use js_sys::{Function, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlElement, Window,
};

use layout::{Anatomy, Mulberry32, Simulation, Wheel, clamp, lerp};
use model::{Model, Path, Spot};
use render::Palette;
use text::{fmt, num};

/// Mount the graph into every host on the page; a page without one is fine.
pub fn init() -> Result<(), JsValue> {
    let doc = document()?;
    let hosts = doc.query_selector_all("[data-graph-host]")?;
    for i in 0..hosts.length() {
        if let Some(host) = hosts.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
            mount(&host)?;
        }
    }
    Ok(())
}

pub(crate) fn window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("no window"))
}
pub(crate) fn document() -> Result<Document, JsValue> {
    window()?
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))
}
pub(crate) fn perf_now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}
fn media(query: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.match_media(query).ok().flatten())
        .is_some_and(|m| m.matches())
}

/// One thing that happened to the app.
pub(crate) enum Ev {
    Click(Event),
    MouseOver(Event),
    MouseOut(Event),
    SizeChange(Event),
    Submit(Event),
    SearchInput(Event),
    PointerDown(Event),
    PointerMove(Event),
    PointerUp(Event),
    PointerCancel(Event),
    PointerLeave(Event),
    Wheel(Event),
    DblClick(Event),
    StageFocus,
    StageBlur,
    StageKey(Event),
    RootKey(Event),
    DocKey(Event),
    MatrixKey(Event),
    MatrixFocusIn(Event),
    Frame(f64),
    Loaded(Result<String, ()>),
    Retry,
    LoadingTimer,
    HintTimer,
    SayTimer,
    HoldTimer,
    HashChange,
    /// The clipboard took a walk's address.
    TraceLinkCopied,
    Resize,
    FitHint,
    Theme,
    Visible,
    Kick,
    MotionChange,
}

pub(crate) struct Shared {
    app: RefCell<App>,
    queue: RefCell<VecDeque<Ev>>,
}

pub(crate) fn dispatch(shared: &Rc<Shared>, ev: Ev) {
    shared.queue.borrow_mut().push_back(ev);
    let Ok(mut app) = shared.app.try_borrow_mut() else {
        return;
    };
    loop {
        let next = shared.queue.borrow_mut().pop_front();
        let Some(ev) = next else { break };
        if let Err(err) = app.handle(ev) {
            web_sys::console::error_1(&err);
        }
    }
}

thread_local! {
    static COUNT: Cell<u32> = const { Cell::new(0) };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cam {
    pub x: f64,
    pub y: f64,
    pub k: f64,
}

pub(crate) struct Ptr {
    pub x: f64,
    pub y: f64,
    pub x0: f64,
    pub y0: f64,
}

pub(crate) struct Pinch {
    pub d: f64,
    pub k: f64,
    pub w: (f64, f64),
}

pub(crate) enum Hit {
    Node(usize),
    Hub,
    Section(usize),
}

pub(crate) struct Drag {
    pub node: Option<usize>,
    pub pan: bool,
    pub cx: f64,
    pub cy: f64,
    pub moved: bool,
}

pub(crate) struct Trace {
    pub from: Option<usize>,
    pub to: Option<usize>,
    pub path: Option<Path>,
    pub at: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmphKind {
    Trace,
    Spot,
    Family,
    Hover,
}

/// What is lit this frame: per-node weight (0 = not, 0.5 = reached by a
/// family's citations, 1 = lit) and per-edge flags.
pub(crate) struct Emphasis {
    pub nodes: Vec<f64>,
    pub edges: Vec<bool>,
    pub kind: EmphKind,
}

pub(crate) struct Dust {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub s: f64,
    pub a: f64,
}

/// Persistent callbacks, made once so a timer or frame never allocates a
/// closure that might never be freed.
struct Handles {
    frame: Function,
    say: Function,
    hold: Function,
}

pub(crate) struct App {
    me: Weak<Shared>,
    handles: Option<Handles>,
    data_url: String,
    pub(crate) root: HtmlElement,
    pub(crate) stage: HtmlElement,
    pub(crate) canvas: HtmlCanvasElement,
    pub(crate) panel: HtmlElement,
    pub(crate) ctx: CanvasRenderingContext2d,
    pub(crate) probe: CanvasRenderingContext2d,
    pub(crate) model: Option<Model>,
    pub(crate) reduced: bool,
    pub(crate) view: &'static str,
    pub(crate) overview: &'static str,
    pub(crate) focus: Option<usize>,
    pub(crate) trail: Vec<usize>,
    pub(crate) hover: Option<usize>,
    pub(crate) hover_section: i32,
    pub(crate) pin_section: i32,
    pub(crate) kb_node: Option<usize>,
    pub(crate) spot: Option<Spot>,
    /// Chosen families, in the order chosen (graph.js's object keys).
    pub(crate) families: Option<Vec<String>>,
    pub(crate) families_preview: bool,
    pub(crate) hover_family: Option<String>,
    pub(crate) trace: Option<Trace>,
    pub(crate) size_by: String,
    pub(crate) cam: Cam,
    pub(crate) cam_t: Cam,
    pub(crate) user_cam: bool,
    pub(crate) mix: f64,
    pub(crate) mix_t: f64,
    pub(crate) ring: f64,
    pub(crate) edge_fade: f64,
    pub(crate) t0: f64,
    pub(crate) last: f64,
    pub(crate) now: f64,
    running: bool,
    pub(crate) dirty: bool,
    pub(crate) pointers: BTreeMap<i32, Ptr>,
    pub(crate) pinch: Option<Pinch>,
    pub(crate) drag: Option<Drag>,
    pub(crate) touch_armed: Option<usize>,
    pub(crate) dust: Vec<Dust>,
    pub(crate) palette: Option<Palette>,
    pub(crate) sprites: Vec<HtmlCanvasElement>,
    pub(crate) foil_sprite: Option<HtmlCanvasElement>,
    pub(crate) sim: Option<Simulation>,
    pub(crate) w: f64,
    pub(crate) h: f64,
    pub(crate) dpr: f64,
    pub(crate) compact: bool,
    pub(crate) sheet: bool,
    pub(crate) wheel: Option<Wheel>,
    pub(crate) anat: Option<Anatomy>,
    pub(crate) label_priority: Vec<usize>,
    pub(crate) emph: Option<Emphasis>,
    pub(crate) intro_at: Option<f64>,
    pub(crate) base_title: Option<String>,
    frame_cost: f64,
    drew_last: bool,
    drawn_at: f64,
    stirred_at: f64,
    settle_frames: u32,
    pub(crate) draw_error: bool,
    pub(crate) overlays: Option<Vec<[f64; 4]>>,
    pub(crate) family_boxes: Option<Vec<[f64; 4]>>,
    pub(crate) label_boxes: Vec<(usize, [f64; 4])>,
    pub(crate) anat_boxes: Vec<(usize, [f64; 4])>,
    pub(crate) family_vis: BTreeMap<String, f64>,
    pub(crate) family_spot: Vec<(String, usize)>,
    pub(crate) family_shown: Vec<(String, bool)>,
    pub(crate) stage_focused: bool,
    pub(crate) panel_key: String,
    pub(crate) matrix_at: Option<(usize, usize)>,
    pub(crate) matrix_from: Option<(usize, usize)>,
    /// How many times the matrix was rendered: graph.js adds its grid's
    /// keydown listener on every render, so Escape steps back that many times.
    pub(crate) matrix_renders: u32,
    loading_timer: Option<i32>,
    say_timer: Option<i32>,
    say_pending: String,
    hold_timer: Option<i32>,
    pub(crate) hold: Option<(i32, usize)>,
    pub(crate) icon_base: (Option<String>, Option<String>),
    hash_listening: bool,
}

fn el(doc: &Document, tag: &str, cls: &str) -> Result<HtmlElement, JsValue> {
    let e: HtmlElement = doc.create_element(tag)?.unchecked_into();
    if !cls.is_empty() {
        e.set_class_name(cls);
    }
    Ok(e)
}

fn listen<T: AsRef<web_sys::EventTarget>>(
    target: &T,
    ty: &str,
    shared: &Rc<Shared>,
    make: fn(Event) -> Ev,
) -> Result<(), JsValue> {
    let target: &web_sys::EventTarget = target.as_ref();
    let s = shared.clone();
    let cb = Closure::<dyn FnMut(Event)>::new(move |e: Event| dispatch(&s, make(e)));
    target.add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())?;
    cb.forget();
    Ok(())
}

/// A numeric property as JavaScript sees it. web-sys binds `clientX` and
/// `scrollTop` as integers, but Chrome reports (and graph.js uses) fractions.
pub(crate) fn prop_f64(obj: &JsValue, name: &str) -> f64 {
    Reflect::get(obj, &JsValue::from_str(name))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
}

pub(crate) fn set_prop_f64(obj: &JsValue, name: &str, value: f64) {
    let _ = Reflect::set(obj, &JsValue::from_str(name), &JsValue::from_f64(value));
}

/// An event's `clientX`/`clientY`, fractions kept.
pub(crate) fn client_xy(e: &Event) -> (f64, f64) {
    (prop_f64(e, "clientX"), prop_f64(e, "clientY"))
}

/// `promise.then(ok, err)` with one-shot callbacks.
pub(crate) fn on_settle(
    p: &js_sys::Promise,
    ok: impl FnOnce(JsValue) + 'static,
    err: impl FnOnce(JsValue) + 'static,
) {
    let then: Result<Function, _> =
        Reflect::get(p, &JsValue::from_str("then")).map(JsCast::unchecked_into);
    if let Ok(then) = then {
        let _ = then.call2(p, &Closure::once_into_js(ok), &Closure::once_into_js(err));
    }
}

fn mount(host: &Element) -> Result<(), JsValue> {
    let doc = document()?;
    let data_url = host.get_attribute("data-graph-data").unwrap_or_default();
    let count = COUNT.with(|c| {
        c.set(c.get() + 1);
        c.get()
    });
    let stylesheet = doc
        .query_selector("link[rel=\"stylesheet\"][href*=\"/assets/styles.css\"]")?
        .and_then(|l| l.get_attribute("href"));
    let used = doc
        .query_selector("use[href*=\"/assets/icons.svg\"]")?
        .and_then(|u| u.get_attribute("href"));
    let icon = |name: &str| {
        views::icon(&views::icon_href(
            stylesheet.as_deref(),
            used.as_deref(),
            name,
        ))
    };

    let root = el(&doc, "div", "crumb crumb--loading")?;
    root.set_attribute("data-view", "constellation")?;
    let bar = el(&doc, "div", "crumb__bar")?;
    bar.set_inner_html(&views::bar_html(
        count,
        &icon("icon-search"),
        &icon("icon-route"),
    ));
    root.append_child(&bar)?;
    let body = el(&doc, "div", "crumb__body")?;
    let stage = el(&doc, "div", "crumb__stage")?;
    stage.set_attribute("tabindex", "0")?;
    stage.set_attribute("role", "application")?;
    stage.set_attribute("aria-roledescription", "graph")?;
    stage.set_attribute("aria-label", text::STAGE_LABEL)?;
    let canvas: HtmlCanvasElement = doc.create_element("canvas")?.unchecked_into();
    canvas.set_class_name("crumb__canvas");
    canvas.set_attribute("aria-hidden", "true")?;
    stage.append_child(&canvas)?;
    stage.insert_adjacent_html(
        "beforeend",
        &views::stage_furniture_html(
            media("(pointer: coarse)"),
            &icon("icon-plus"),
            &icon("icon-minus"),
            &icon("icon-fit"),
        ),
    )?;
    body.append_child(&stage)?;
    let panel = el(&doc, "aside", "crumb__panel")?;
    panel.set_attribute("aria-label", "Details")?;
    panel.set_inner_html(&views::panel_shell_html());
    body.append_child(&panel)?;
    root.append_child(&body)?;
    root.insert_adjacent_html(
        "beforeend",
        "<p class=\"visually-hidden\" aria-live=\"polite\" data-crumb-live></p>",
    )?;
    host.append_child(&root)?;
    if let Some(b) = doc.body() {
        b.class_list().add_1("graph-live")?;
    }
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .unchecked_into();
    let probe_canvas: HtmlCanvasElement = doc.create_element("canvas")?.unchecked_into();
    let probe: CanvasRenderingContext2d = probe_canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .unchecked_into();

    let t0 = perf_now();
    let app = App {
        me: Weak::new(),
        handles: None,
        data_url,
        root,
        stage,
        canvas,
        panel,
        ctx,
        probe,
        model: None,
        reduced: media("(prefers-reduced-motion: reduce)"),
        view: "constellation",
        overview: "constellation",
        focus: None,
        trail: Vec::new(),
        hover: None,
        hover_section: -1,
        pin_section: -1,
        kb_node: None,
        spot: None,
        families: None,
        families_preview: false,
        hover_family: None,
        trace: None,
        size_by: "in".into(),
        cam: Cam {
            x: 0.0,
            y: 0.0,
            k: 1.0,
        },
        cam_t: Cam {
            x: 0.0,
            y: 0.0,
            k: 1.0,
        },
        user_cam: false,
        mix: 0.0,
        mix_t: 0.0,
        ring: 0.0,
        edge_fade: 0.0,
        t0,
        last: t0,
        now: t0,
        running: false,
        dirty: true,
        pointers: BTreeMap::new(),
        pinch: None,
        drag: None,
        touch_armed: None,
        dust: Vec::new(),
        palette: None,
        sprites: Vec::new(),
        foil_sprite: None,
        sim: None,
        w: 0.0,
        h: 0.0,
        dpr: 1.0,
        compact: false,
        sheet: false,
        wheel: None,
        anat: None,
        label_priority: Vec::new(),
        emph: None,
        intro_at: None,
        base_title: None,
        frame_cost: 0.0,
        drew_last: false,
        drawn_at: f64::NAN,
        stirred_at: f64::NAN,
        settle_frames: 0,
        draw_error: false,
        overlays: None,
        family_boxes: None,
        label_boxes: Vec::new(),
        anat_boxes: Vec::new(),
        family_vis: BTreeMap::new(),
        family_spot: Vec::new(),
        family_shown: Vec::new(),
        stage_focused: false,
        panel_key: String::new(),
        matrix_at: None,
        matrix_from: None,
        matrix_renders: 0,
        loading_timer: None,
        say_timer: None,
        say_pending: String::new(),
        hold_timer: None,
        hold: None,
        icon_base: (stylesheet, used),
        hash_listening: false,
    };
    let shared = Rc::new(Shared {
        app: RefCell::new(app),
        queue: RefCell::new(VecDeque::new()),
    });
    {
        let mut app = shared.app.borrow_mut();
        app.me = Rc::downgrade(&shared);
        let s = shared.clone();
        let frame = Closure::<dyn FnMut(f64)>::new(move |t: f64| dispatch(&s, Ev::Frame(t)));
        let s = shared.clone();
        let say = Closure::<dyn FnMut()>::new(move || dispatch(&s, Ev::SayTimer));
        let s = shared.clone();
        let hold = Closure::<dyn FnMut()>::new(move || dispatch(&s, Ev::HoldTimer));
        app.handles = Some(Handles {
            frame: frame.into_js_value().unchecked_into(),
            say: say.into_js_value().unchecked_into(),
            hold: hold.into_js_value().unchecked_into(),
        });
        if let Some(status) = app.q("[data-crumb-status]") {
            status.set_inner_html("");
        }
        app.loading_timer = app.timeout_once(Ev::LoadingTimer, 300);
    }
    probe::expose(host, &shared)?;
    bind_shell(&shared)?;
    shared.app.borrow_mut().load()?;
    Ok(())
}

fn bind_shell(shared: &Rc<Shared>) -> Result<(), JsValue> {
    let app = shared.app.borrow();
    let root: &web_sys::EventTarget = app.root.as_ref();
    listen(root, "click", shared, Ev::Click)?;
    listen(root, "mouseover", shared, Ev::MouseOver)?;
    listen(root, "mouseout", shared, Ev::MouseOut)?;
    listen(root, "submit", shared, Ev::Submit)?;
    if let Some(size) = app.q("[data-crumb-size]") {
        listen(&size, "change", shared, Ev::SizeChange)?;
    }
    if let Some(input) = app.q("[data-crumb-search] input") {
        listen(&input, "input", shared, Ev::SearchInput)?;
    }
    let stage: &web_sys::EventTarget = app.stage.as_ref();
    listen(stage, "pointerdown", shared, Ev::PointerDown)?;
    listen(stage, "pointermove", shared, Ev::PointerMove)?;
    listen(stage, "pointerup", shared, Ev::PointerUp)?;
    listen(stage, "pointercancel", shared, Ev::PointerCancel)?;
    listen(stage, "pointerleave", shared, Ev::PointerLeave)?;
    {
        let s = shared.clone();
        let cb = Closure::<dyn FnMut(Event)>::new(move |e: Event| dispatch(&s, Ev::Wheel(e)));
        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        stage.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            cb.as_ref().unchecked_ref(),
            &opts,
        )?;
        cb.forget();
    }
    listen(stage, "dblclick", shared, Ev::DblClick)?;
    listen(stage, "focus", shared, |_| Ev::StageFocus)?;
    listen(stage, "blur", shared, |_| Ev::StageBlur)?;
    listen(stage, "keydown", shared, Ev::StageKey)?;
    listen(root, "keydown", shared, Ev::RootKey)?;
    listen(&document()?, "keydown", shared, Ev::DocKey)?;
    // The matrix box lives as long as the app; its keydown is counted per
    // render (see `matrix_renders`), its focusin needs adding only once.
    if let Some(matrix) = app.q("[data-crumb-matrix]") {
        listen(&matrix, "keydown", shared, Ev::MatrixKey)?;
        listen(&matrix, "focusin", shared, Ev::MatrixFocusIn)?;
    }
    Ok(())
}

impl App {
    pub(crate) fn shared(&self) -> Option<Rc<Shared>> {
        self.me.upgrade()
    }

    /// `root.querySelector(sel)`.
    pub(crate) fn q(&self, sel: &str) -> Option<HtmlElement> {
        self.root
            .query_selector(sel)
            .ok()
            .flatten()
            .map(JsCast::unchecked_into)
    }

    pub(crate) fn icon(&self, name: &str) -> String {
        views::icon(&views::icon_href(
            self.icon_base.0.as_deref(),
            self.icon_base.1.as_deref(),
            name,
        ))
    }

    /// A one-shot timer that dispatches `ev`.
    pub(crate) fn timeout_once(&self, ev: Ev, ms: i32) -> Option<i32> {
        let shared = self.shared()?;
        let cb = Closure::once_into_js(move || dispatch(&shared, ev));
        window()
            .ok()?
            .set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), ms)
            .ok()
    }

    fn clear_timeout(id: Option<i32>) {
        if let (Some(id), Some(w)) = (id, web_sys::window()) {
            w.clear_timeout_with_handle(id);
        }
    }

    fn handle(&mut self, ev: Ev) -> Result<(), JsValue> {
        match ev {
            Ev::Click(e) => self.on_click(&e),
            Ev::MouseOver(e) => self.on_mouse_over(&e),
            Ev::MouseOut(e) => self.on_mouse_out(&e),
            Ev::SizeChange(e) => {
                if let Some(sel) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
                    self.size_by = sel.value();
                }
                self.apply_sizes(false);
                Ok(())
            }
            Ev::Submit(e) => self.on_submit(&e),
            Ev::SearchInput(e) => self.on_search_input(&e),
            Ev::PointerDown(e) => self.on_pointer_down(e.unchecked_ref()),
            Ev::PointerMove(e) => self.on_pointer_move(e.unchecked_ref()),
            Ev::PointerUp(e) => self.on_pointer_up(e.unchecked_ref(), false),
            Ev::PointerCancel(e) => self.on_pointer_up(e.unchecked_ref(), true),
            Ev::PointerLeave(e) => {
                let e: &web_sys::PointerEvent = e.unchecked_ref();
                if e.pointer_type() == "mouse" {
                    self.set_hover(None, -1);
                    self.hide_tip();
                }
                Ok(())
            }
            Ev::Wheel(e) => self.on_wheel(e.unchecked_ref()),
            Ev::DblClick(e) => self.on_dblclick(e.unchecked_ref()),
            Ev::StageFocus => {
                self.stage_focused = true;
                if self.kb_node.is_none() && self.model.is_some() {
                    self.kb_node = match (&self.anat, self.view) {
                        (Some(a), "anatomy") if !a.neighbors.is_empty() => {
                            Some(a.neighbors[0].node)
                        }
                        _ => self.label_priority.first().copied(),
                    };
                    self.announce(self.kb_node);
                }
                self.kick();
                Ok(())
            }
            Ev::StageBlur => {
                self.stage_focused = false;
                self.kick();
                Ok(())
            }
            Ev::StageKey(e) => self.on_key(e.unchecked_ref()),
            Ev::RootKey(e) => self.on_root_key(e.unchecked_ref()),
            Ev::DocKey(e) => self.on_doc_key(e.unchecked_ref()),
            Ev::MatrixKey(e) => self.on_matrix_key(e.unchecked_ref()),
            Ev::MatrixFocusIn(e) => self.on_matrix_focus_in(&e),
            Ev::Frame(now) => self.on_frame(now),
            Ev::Loaded(r) => self.on_loaded(r),
            Ev::Retry => {
                if let Some(status) = self.q("[data-crumb-status]") {
                    status.set_inner_html("");
                }
                self.load()
            }
            Ev::LoadingTimer => {
                if self.model.is_none()
                    && let Some(status) = self.q("[data-crumb-status]")
                {
                    status.set_inner_html(&views::loading_html());
                }
                Ok(())
            }
            Ev::HintTimer => {
                self.dismiss_hint();
                Ok(())
            }
            Ev::SayTimer => {
                if let Some(live) = self.q("[data-crumb-live]") {
                    live.set_text_content(Some(&self.say_pending));
                }
                Ok(())
            }
            Ev::HoldTimer => {
                if let (Some((id, node)), Some(drag)) = (self.hold, &self.drag)
                    && drag.pan
                    && !drag.moved
                    && self.pointers.contains_key(&id)
                {
                    self.hide_tip();
                    self.drag = Some(Drag {
                        node: Some(node),
                        pan: false,
                        cx: 0.0,
                        cy: 0.0,
                        moved: false,
                    });
                }
                Ok(())
            }
            Ev::HashChange => self.on_hash_change(),
            Ev::TraceLinkCopied => {
                self.say(text::COPIED_LINK);
                Ok(())
            }
            Ev::Resize => {
                self.resize();
                Ok(())
            }
            Ev::FitHint => {
                self.fit_hint();
                Ok(())
            }
            Ev::Theme => {
                self.read_palette();
                self.kick();
                Ok(())
            }
            Ev::Visible => {
                if !document()?.hidden() {
                    self.kick();
                }
                Ok(())
            }
            Ev::Kick => {
                self.kick();
                Ok(())
            }
            Ev::MotionChange => {
                self.reduced = media("(prefers-reduced-motion: reduce)");
                self.kick();
                Ok(())
            }
        }
    }

    // --- loading ----------------------------------------------------------

    fn load(&mut self) -> Result<(), JsValue> {
        let Some(shared) = self.shared() else {
            return Ok(());
        };
        let init = web_sys::RequestInit::new();
        init.set_credentials(web_sys::RequestCredentials::SameOrigin);
        let promise = window()?.fetch_with_str_and_init(&self.data_url, &init);
        let fail = {
            let s = shared.clone();
            move |_| dispatch(&s, Ev::Loaded(Err(())))
        };
        let fail2 = fail.clone();
        on_settle(
            &promise,
            move |resp: JsValue| {
                let resp: web_sys::Response = resp.unchecked_into();
                match (resp.ok(), resp.text()) {
                    (true, Ok(text)) => {
                        let s = shared.clone();
                        on_settle(
                            &text,
                            move |t| dispatch(&s, Ev::Loaded(t.as_string().ok_or(()))),
                            fail2,
                        );
                    }
                    _ => fail2(JsValue::NULL),
                }
            },
            fail,
        );
        Ok(())
    }

    fn on_loaded(&mut self, r: Result<String, ()>) -> Result<(), JsValue> {
        Self::clear_timeout(self.loading_timer.take());
        let data = r.ok().and_then(|text| model::unpack(&text).ok());
        let Some(data) = data else {
            let Some(status) = self.q("[data-crumb-status]") else {
                return Ok(());
            };
            status.set_inner_html(&views::failed_html());
            if let (Some(button), Some(shared)) = (status.query_selector("button")?, self.shared())
            {
                listen(&button, "click", &shared, |_| Ev::Retry)?;
            }
            return Ok(());
        };
        let mut random = js_sys::Math::random;
        let mut model = Model::build(&data, &mut random);
        for n in &mut model.nodes {
            if let Some(at) = &n.change_at {
                n.change_ms = js_sys::Date::parse(at);
            }
        }
        self.model = Some(model);
        self.ready()
    }

    fn ready(&mut self) -> Result<(), JsValue> {
        self.root.class_list().remove_1("crumb--loading")?;
        if let Some(status) = self.q("[data-crumb-status]") {
            status.set_inner_html("");
        }
        let (nodes, edges) = self
            .model
            .as_ref()
            .map_or((0, 0), |m| (m.nodes.len(), m.edges.len()));
        if let Some(meta) = self.q("[data-crumb-meta]") {
            meta.set_text_content(Some(&fmt(
                text::META,
                &[("skills", nodes.to_string()), ("pairs", edges.to_string())],
            )));
        }
        if let (Some(list), Some(m)) = (self.q("datalist"), &self.model) {
            list.set_inner_html(&views::datalist_html(m));
        }
        self.read_palette();
        self.apply_sizes(true);
        let rect = self.stage.get_bounding_client_rect();
        self.w = rect.width();
        self.h = rect.height();
        self.compact = self.w < 640.0;
        self.sheet = self.is_sheet();
        let m = self.overlay_margins(36.0);
        let aspect =
            (rect.width() - m.left - m.right) / f64::max(1.0, rect.height() - m.top - m.bottom);
        if let Some(model) = self.model.as_mut() {
            let mut sim = Simulation::new(&mut model.nodes, &model.categories, aspect);
            for _ in 0..160 {
                sim.tick(model);
            }
            if self.reduced {
                for _ in 0..400 {
                    sim.tick(model);
                }
                sim.alpha = 0.0;
            }
            self.sim = Some(sim);
            let mut rand = Mulberry32::new(7);
            for _ in 0..46 {
                let (x, y) = (rand.next(), rand.next());
                let r = 0.5 + rand.next() * 1.3;
                let s = 0.15 + rand.next() * 0.5;
                let a = 0.04 + rand.next() * 0.1;
                self.dust.push(Dust { x, y, r, s, a });
            }
            for n in &mut model.nodes {
                n.px = n.x * 0.05;
                n.py = n.y * 0.05;
                n.pr = 0.0;
                n.pa = 0.0;
            }
        }
        self.render_legend();
        self.render_panel()?;
        self.resize();
        self.layout_wheel();
        self.fit_camera(true, None);
        self.observe()?;
        self.intro_at = Some(perf_now());
        self.timeout_once(Ev::HintTimer, 9000);
        let hash = window()?.location().hash()?;
        let asked = hash_skill(&hash);
        let start = asked
            .as_ref()
            .and_then(|a| self.model.as_ref().and_then(|m| m.node(a)));
        self.base_title = Some(document()?.title());
        let walk = hash_trace(&hash).and_then(|(a, b)| {
            let model = self.model.as_ref()?;
            Some((model.node(&a)?, model.node(&b)?)).filter(|(a, b)| a != b)
        });
        if let Some(start) = start {
            self.open_skill(start, true)?;
        } else if let Some((a, b)) = walk {
            self.run_trace(a, b)?;
            self.set_sheet(true)?;
        } else if let Some(asked) = asked {
            self.missing_skill(&asked)?;
        } else if let Some(v) = hash_view(&hash) {
            self.set_view(v)?;
        }
        if !self.hash_listening {
            self.hash_listening = true;
            if let Some(shared) = self.shared() {
                listen(&window()?, "hashchange", &shared, |_| Ev::HashChange)?;
            }
        }
        self.kick();
        Ok(())
    }

    fn on_hash_change(&mut self) -> Result<(), JsValue> {
        let Some(model) = &self.model else {
            return Ok(());
        };
        let hash = window()?.location().hash()?;
        if let Some(asked) = hash_skill(&hash) {
            match model.node(&asked) {
                Some(n) if Some(n) != self.focus => return self.open_skill(n, false),
                Some(_) => {}
                None => return self.missing_skill(&asked),
            }
        }
        let walk = hash_trace(&hash).and_then(|(a, b)| {
            let model = self.model.as_ref()?;
            Some((model.node(&a)?, model.node(&b)?)).filter(|(a, b)| a != b)
        });
        if let Some((a, b)) = walk {
            let same = self
                .trace
                .as_ref()
                .is_some_and(|t| t.from == Some(a) && t.to == Some(b));
            if !same {
                if self.view == "anatomy" {
                    self.set_view(self.overview)?;
                }
                self.run_trace(a, b)?;
            }
            return Ok(());
        }
        if let Some(v) = hash_view(&hash)
            && v != self.view
        {
            self.set_view(v)?;
        }
        Ok(())
    }

    pub(crate) fn apply_sizes(&mut self, instant: bool) {
        let Some(model) = self.model.as_mut() else {
            return;
        };
        let max_words = model
            .nodes
            .iter()
            .map(|n| n.words)
            .fold(f64::NEG_INFINITY, f64::max);
        for n in &mut model.nodes {
            let r = match self.size_by.as_str() {
                "out" => 4.0 + 1.9 * (n.out_deg as f64).sqrt(),
                "words" => 3.5 + 13.0 * (n.words / max_words).sqrt(),
                _ => 4.0 + 2.3 * (n.in_deg as f64).sqrt(),
            };
            n.base_r = clamp(r, 3.5, 20.0);
            n.sim_r = n.base_r;
            if instant {
                n.pr = 0.0;
            }
        }
        if let Some(sim) = self.sim.as_mut() {
            sim.reheat(0.25);
        }
        let mut order: Vec<usize> = (0..model.nodes.len()).collect();
        order.sort_by(|&a, &b| {
            let (na, nb) = (&model.nodes[a], &model.nodes[b]);
            nb.base_r
                .partial_cmp(&na.base_r)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(nb.degree.cmp(&na.degree))
        });
        self.label_priority = order;
        self.kick();
    }

    // --- sizing and observers ----------------------------------------------

    pub(crate) fn resize(&mut self) {
        let rect = self.stage.get_bounding_client_rect();
        let raw = web_sys::window().map_or(1.0, |w| w.device_pixel_ratio());
        let dpr = f64::min(if raw == 0.0 || raw.is_nan() { 1.0 } else { raw }, 2.0);
        self.w = f64::max(1.0, rect.width());
        self.h = f64::max(1.0, rect.height());
        self.dpr = dpr;
        self.canvas.set_width(text::js_round(self.w * dpr) as u32);
        self.canvas.set_height(text::js_round(self.h * dpr) as u32);
        let style = self.canvas.style();
        let _ = style.set_property("width", &format!("{}px", num(self.w)));
        let _ = style.set_property("height", &format!("{}px", num(self.h)));
        self.compact = self.w < 640.0;
        self.sheet = self.is_sheet();
        if self.view == "anatomy" && self.focus.is_some() {
            self.layout_anatomy();
        }
        if self.model.is_some() {
            self.layout_wheel();
        }
        if !self.user_cam {
            self.fit_camera(false, None);
        }
        self.kick();
    }

    fn observe(&mut self) -> Result<(), JsValue> {
        let Some(shared) = self.shared() else {
            return Ok(());
        };
        let observer = |ev: fn() -> Ev| -> Result<web_sys::ResizeObserver, JsValue> {
            let s = shared.clone();
            let cb = Closure::<dyn FnMut()>::new(move || dispatch(&s, ev()));
            let ro = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref())?;
            cb.forget();
            Ok(ro)
        };
        let ro = observer(|| Ev::Resize)?;
        ro.observe(&self.stage);
        self.fit_hint();
        if let Some(tools) = self.q(".crumb__tools") {
            let ro = observer(|| Ev::FitHint)?;
            ro.observe(&tools);
            ro.observe(&self.stage);
            if let Some(legend) = self.q("[data-crumb-legend]") {
                ro.observe(&legend);
            }
        }
        {
            let s = shared.clone();
            let cb = Closure::<dyn FnMut()>::new(move || dispatch(&s, Ev::Theme));
            let mo = web_sys::MutationObserver::new(cb.as_ref().unchecked_ref())?;
            cb.forget();
            let opts = web_sys::MutationObserverInit::new();
            opts.set_attributes(true);
            opts.set_attribute_filter(&js_sys::Array::of1(&JsValue::from_str("data-theme")));
            if let Some(html) = document()?.document_element() {
                mo.observe_with_options(&html, &opts)?;
            }
        }
        listen(&document()?, "visibilitychange", &shared, |_| Ev::Visible)?;
        if let Ok(ready) = document()?.fonts().ready() {
            let s = shared.clone();
            on_settle(&ready, move |_| dispatch(&s, Ev::Kick), |_| {});
        }
        if let Some(mq) = window()?.match_media("(prefers-reduced-motion: reduce)")? {
            listen(&mq, "change", &shared, |_| Ev::MotionChange)?;
        }
        Ok(())
    }

    /// The hint shares the stage's bottom edge with the tools (and on a
    /// narrow stage the legend): centred, in the row, or above the tools.
    fn fit_hint(&self) {
        let Some(tools) = self.q(".crumb__tools") else {
            return;
        };
        let legend = self.q("[data-crumb-legend]");
        let (sw, sh) = (
            f64::from(self.stage.client_width()),
            f64::from(self.stage.client_height()),
        );
        let (tw, th) = (
            f64::from(tools.offset_width()),
            f64::from(tools.offset_height()),
        );
        let gap = f64::max(0.0, sw - f64::from(tools.offset_left()) - tw);
        let beside = tw + 2.0 * gap;
        let (mut left, mut mode, mut l, mut r, mut b) = (gap, "centre", beside, beside, None);
        if let Some(legend) = &legend
            && legend.offset_height() != 0
            && f64::from(legend.offset_top() + legend.offset_height()) > sh - th - 2.0 * gap
        {
            left = f64::from(legend.offset_left() + legend.offset_width()) + gap;
            mode = "row";
            l = left;
        }
        if sw - l - r < 240.0 {
            mode = "row";
            l = left;
        }
        if sw - l - r < 240.0 {
            mode = "above";
            r = gap;
            b = Some(gap + th + 8.0);
        }
        let _ = self.root.set_attribute("data-crumb-hint", mode);
        let style = self.root.style();
        let _ = style.set_property("--crumb-hint-l", &format!("{}px", num(l)));
        let _ = style.set_property("--crumb-hint-r", &format!("{}px", num(r)));
        match b {
            None => {
                let _ = style.remove_property("--crumb-hint-b");
            }
            Some(b) => {
                let _ = style.set_property("--crumb-hint-b", &format!("{}px", num(b)));
            }
        }
    }

    // --- camera --------------------------------------------------------------

    pub(crate) fn fit_camera(&mut self, instant: bool, subset: Option<&[usize]>) {
        let Some(model) = &self.model else { return };
        let view = self.view;
        if subset.is_none() && (view == "wheel" || view == "anatomy") {
            self.cam_t = Cam {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            };
        } else {
            let (mut min_x, mut min_y, mut max_x, mut max_y) = (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            );
            let all: Vec<usize>;
            let list = match subset {
                Some(s) => s,
                None => {
                    all = (0..model.nodes.len()).collect();
                    &all
                }
            };
            for &i in list {
                let n = &model.nodes[i];
                let (x, y) = if view == "wheel" {
                    (n.wx, n.wy)
                } else {
                    (n.x, n.y)
                };
                min_x = f64::min(min_x, x - 30.0);
                max_x = f64::max(max_x, x + 30.0);
                min_y = f64::min(min_y, y - 30.0);
                max_y = f64::max(max_y, y + 30.0);
            }
            let pad = if subset.is_some() { 90.0 } else { 36.0 };
            let m = self.overlay_margins(pad);
            let avail_w = self.w - m.left - m.right;
            let avail_h = self.h - m.top - m.bottom;
            let k = f64::min(
                avail_w / f64::max(1.0, max_x - min_x),
                avail_h / f64::max(1.0, max_y - min_y),
            );
            let k = clamp(k, 0.2, if subset.is_some() { 1.6 } else { 2.2 });
            self.cam_t = Cam {
                x: (min_x + max_x) / 2.0 + (m.right - m.left) / 2.0 / k,
                y: (min_y + max_y) / 2.0 + (m.bottom - m.top) / 2.0 / k,
                k,
            };
        }
        if instant || self.reduced {
            self.cam = self.cam_t;
        }
        self.kick();
    }

    pub(crate) fn is_sheet(&self) -> bool {
        web_sys::window()
            .and_then(|w| w.get_computed_style(&self.panel).ok().flatten())
            .and_then(|cs| cs.get_property_value("position").ok())
            .is_some_and(|p| p == "absolute")
    }

    pub(crate) fn overlay_margins(&self, pad: f64) -> Margins {
        let mut m = Margins {
            top: pad,
            bottom: pad + 12.0,
            left: pad,
            right: pad,
        };
        if self.compact || self.sheet {
            m.top = 60.0;
            m.bottom = 120.0;
            return m;
        }
        if let Some(legend) = self.q("[data-crumb-legend]")
            && self.view != "anatomy"
            && !self.is_page()
        {
            if legend.offset_height() > legend.offset_width() {
                m.left = f64::from(legend.offset_width()) + 28.0;
                m.bottom = 64.0;
            } else {
                m.bottom = f64::from(legend.offset_height()) + 24.0;
            }
        }
        m
    }

    pub(crate) fn to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let c = self.cam;
        (
            (x - c.x) * c.k + self.w / 2.0,
            (y - c.y) * c.k + self.h / 2.0,
        )
    }
    pub(crate) fn to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        let c = self.cam;
        (
            (sx - self.w / 2.0) / c.k + c.x,
            (sy - self.h / 2.0) / c.k + c.y,
        )
    }

    pub(crate) fn zoom_at(&mut self, sx: f64, sy: f64, factor: f64) {
        let before = self.to_world(sx, sy);
        let k = clamp(self.cam_t.k * factor, 0.2, 5.0);
        self.cam_t.k = k;
        self.cam_t.x = before.0 - (sx - self.w / 2.0) / k;
        self.cam_t.y = before.1 - (sy - self.h / 2.0) / k;
        self.user_cam = true;
        if self.reduced {
            self.cam = self.cam_t;
        }
        self.kick();
    }

    // --- layouts -----------------------------------------------------------------

    pub(crate) fn layout_wheel(&mut self) {
        let (w, h) = (self.w, self.h);
        if let Some(model) = self.model.as_mut() {
            self.wheel = Some(layout::layout_wheel(model, w, h));
        }
    }

    pub(crate) fn layout_anatomy(&mut self) {
        let (Some(model), Some(focus)) = (&self.model, self.focus) else {
            return;
        };
        let mut random = js_sys::Math::random;
        self.anat = Some(layout::layout_anatomy(
            model,
            focus,
            self.w,
            self.h,
            &mut random,
        ));
    }

    /// Where every node wants to be in the current view.
    fn update_targets(&mut self) {
        let view = self.view;
        let (focus, hover, mix_t) = (self.focus, self.hover, self.mix_t);
        let active_section = if self.pin_section >= 0 {
            self.pin_section
        } else {
            self.hover_section
        };
        let section_lit = self.hover_section >= 0 || self.pin_section >= 0;
        let Some(model) = self.model.as_mut() else {
            return;
        };
        let emph = &self.emph;
        let anat = &self.anat;
        let focus_refs: Option<Vec<(String, f64)>> = match (focus, section_lit) {
            (Some(f), true) if active_section >= 0 => model.nodes[f]
                .sections
                .get(active_section as usize)
                .map(|s| s.refs.clone()),
            _ => None,
        };
        for i in 0..model.nodes.len() {
            let n = &model.nodes[i];
            let (mut in_view, mut r, mut a) = (true, n.base_r, 1.0);
            let (tx, ty): (f64, f64);
            if view == "anatomy" {
                let Some(l) = anat else { continue };
                let item = l.item_of(i);
                if Some(i) == focus {
                    tx = 0.0;
                    ty = 0.0;
                    r = l.r0;
                } else if let Some(it) = item {
                    tx = l.neighbors[it].x;
                    ty = l.neighbors[it].y;
                    r = l.neighbors[it].r;
                } else {
                    let ang = n.y.atan2(n.x);
                    let ang = if ang == 0.0 || ang.is_nan() {
                        i as f64
                    } else {
                        ang
                    };
                    tx = ang.cos() * l.big_r * 2.4;
                    ty = ang.sin() * l.big_r * 2.4;
                    in_view = false;
                    r = n.base_r * 0.5;
                }
                if item.is_some()
                    && section_lit
                    && let Some(refs) = &focus_refs
                    && model::ref_count(refs, &n.slug) == 0.0
                {
                    a = 0.25;
                }
                if hover.is_some()
                    && hover != Some(i)
                    && hover != focus
                    && Some(i) != focus
                    && item.is_some()
                {
                    a = f64::min(a, 0.35);
                }
            } else {
                tx = lerp(n.x, n.wx, mix_t);
                ty = lerp(n.y, n.wy, mix_t);
                if view == "wheel" {
                    r = f64::min(n.base_r, 3.0 + n.base_r * 0.45);
                }
                if let Some(e) = emph
                    && e.nodes[i] == 0.0
                {
                    a = 0.14;
                }
            }
            let n = &mut model.nodes[i];
            n.tx = tx;
            n.ty = ty;
            n.tr = r;
            n.ta = if in_view { a } else { 0.0 };
        }
    }

    /// What is lit, overview only: a trace, else a spotlight, else chosen
    /// families, else the hovered or keyboard node.
    pub(crate) fn compute_emphasis(&self) -> Option<Emphasis> {
        if self.view == "anatomy" {
            return None;
        }
        let model = self.model.as_ref()?;
        let (nn, ne) = (model.nodes.len(), model.edges.len());
        let mut nodes = vec![0.0; nn];
        let mut edges = vec![false; ne];
        if let Some(Trace {
            path: Some(path), ..
        }) = &self.trace
        {
            for &n in &path.nodes {
                nodes[n] = 1.0;
            }
            for &e in &path.edges {
                edges[e] = true;
            }
            return Some(Emphasis {
                nodes,
                edges,
                kind: EmphKind::Trace,
            });
        }
        if let Some(spot) = &self.spot {
            match spot.kind {
                "flow" => {
                    for (i, e) in model.edges.iter().enumerate() {
                        let (sc, tc) = (
                            &model.categories[model.nodes[e.s].cat].slug,
                            &model.categories[model.nodes[e.t].cat].slug,
                        );
                        if Some(sc) == spot.from.as_ref() && Some(tc) == spot.to.as_ref() {
                            edges[i] = true;
                            nodes[e.s] = 1.0;
                            nodes[e.t] = 1.0;
                        }
                    }
                }
                "mutual" => {
                    for (i, e) in model.edges.iter().enumerate() {
                        if e.mutual {
                            edges[i] = true;
                            nodes[e.s] = 1.0;
                            nodes[e.t] = 1.0;
                        }
                    }
                }
                _ => {
                    for slug in &spot.nodes {
                        let Some(n) = model.node(slug) else { continue };
                        nodes[n] = 1.0;
                        if spot.dir == Some("pair") {
                            continue;
                        }
                        if spot.dir != Some("in") {
                            for &(t, e) in &model.nodes[n].out {
                                edges[e] = true;
                                nodes[t] = 1.0;
                            }
                        }
                        if spot.dir != Some("out") {
                            for &(s, e) in &model.nodes[n].inn {
                                edges[e] = true;
                                nodes[s] = 1.0;
                            }
                        }
                    }
                    if spot.dir == Some("pair")
                        && spot.nodes.len() == 2
                        && let (Some(a), Some(b)) =
                            (model.node(&spot.nodes[0]), model.node(&spot.nodes[1]))
                    {
                        for (s, t) in [(a, b), (b, a)] {
                            if let Some(e) = model.edge(s, t) {
                                edges[e] = true;
                            }
                        }
                    }
                }
            }
            return Some(Emphasis {
                nodes,
                edges,
                kind: EmphKind::Spot,
            });
        }
        if let Some(fams) = &self.families {
            let chosen = |c: usize| fams.iter().any(|s| *s == model.categories[c].slug);
            for (i, n) in model.nodes.iter().enumerate() {
                if chosen(n.cat) {
                    nodes[i] = 1.0;
                }
            }
            for (i, e) in model.edges.iter().enumerate() {
                if chosen(model.nodes[e.s].cat) {
                    edges[i] = true;
                    if nodes[e.t] == 0.0 {
                        nodes[e.t] = 0.5;
                    }
                }
            }
            return Some(Emphasis {
                nodes,
                edges,
                kind: EmphKind::Family,
            });
        }
        let h = self.hover.or(self.kb_node)?;
        nodes[h] = 1.0;
        for &(t, e) in &model.nodes[h].out {
            edges[e] = true;
            nodes[t] = 1.0;
        }
        for &(s, e) in &model.nodes[h].inn {
            edges[e] = true;
            nodes[s] = 1.0;
        }
        Some(Emphasis {
            nodes,
            edges,
            kind: EmphKind::Hover,
        })
    }

    // --- frame loop ------------------------------------------------------------

    pub(crate) fn kick(&mut self) {
        self.dirty = true;
        if self.running || self.model.is_none() {
            return;
        }
        self.running = true;
        self.drew_last = false;
        self.last = perf_now();
        self.request_frame();
    }

    fn request_frame(&self) {
        if let (Some(h), Some(w)) = (&self.handles, web_sys::window()) {
            let _ = w.request_animation_frame(&h.frame);
        }
    }

    /// The paced loop: full rate when stirred, a calm 30 ms (or twice the
    /// measured frame cost on a slow machine) when only the drift moves,
    /// 100 ms after twenty idle seconds, stopped while the tab is hidden.
    fn on_frame(&mut self, now: f64) -> Result<(), JsValue> {
        let hidden = document()?.hidden();
        if self.drew_last {
            self.frame_cost += (now - self.drawn_at - self.frame_cost) * 0.2;
        }
        self.drew_last = false;
        let mut calm = if self.frame_cost > 25.0 {
            2.0 * self.frame_cost
        } else {
            30.0
        };
        if now - self.stirred_at > 20000.0 {
            calm = f64::max(calm, 100.0);
        }
        if !self.dirty && now - self.stirred_at > 1500.0 && now - self.drawn_at < calm {
            if hidden {
                self.running = false;
            } else {
                self.request_frame();
            }
            return Ok(());
        }
        self.drawn_at = now;
        self.drew_last = true;
        let busy = self.frame(now);
        if busy && !document()?.hidden() {
            self.request_frame();
        } else {
            self.running = false;
        }
        Ok(())
    }

    fn frame(&mut self, now: f64) -> bool {
        let dt = clamp((now - self.last) / 1000.0, 0.0, 0.05);
        self.last = now;
        self.now = now;
        let mut busy = false;
        if !self.is_page() {
            let settling =
                self.sim.as_ref().is_some_and(Simulation::active) && self.view != "anatomy";
            if settling {
                if let (Some(sim), Some(model)) = (self.sim.as_mut(), self.model.as_mut()) {
                    for _ in 0..2 {
                        sim.tick(model);
                    }
                }
                busy = true;
                self.settle_frames += 1;
                if !self.user_cam
                    && self.drag.is_none()
                    && self.settle_frames.is_multiple_of(12)
                    && self.view == "constellation"
                    && self.spot.is_none()
                    && self.trace.is_none()
                {
                    self.fit_camera(false, None);
                }
            }
            let k = if self.reduced {
                1.0
            } else {
                1.0 - (-dt * 7.5).exp()
            };
            self.mix += (self.mix_t - self.mix) * k;
            if (self.mix_t - self.mix).abs() > 0.001 {
                busy = true;
            } else {
                self.mix = self.mix_t;
            }
            let t = self.cam_t;
            let ck = if self.reduced {
                1.0
            } else {
                1.0 - (-dt * 6.0).exp()
            };
            self.cam.x += (t.x - self.cam.x) * ck;
            self.cam.y += (t.y - self.cam.y) * ck;
            self.cam.k += (t.k - self.cam.k) * ck;
            if (t.x - self.cam.x).abs() + (t.y - self.cam.y).abs() > 0.1
                || (t.k - self.cam.k).abs() > 0.0005
            {
                busy = true;
            }
            let ring_target = if self.view == "anatomy" { 1.0 } else { 0.0 };
            self.ring += (ring_target - self.ring)
                * if self.reduced {
                    1.0
                } else {
                    1.0 - (-dt * 3.2).exp()
                };
            if (ring_target - self.ring).abs() > 0.002 {
                busy = true;
            } else {
                self.ring = ring_target;
            }
            let intro_t = (now - self.intro_at.unwrap_or(now)) / 1000.0;
            let fade_target = if self.reduced {
                1.0
            } else {
                clamp((intro_t - 0.35) / 0.9, 0.0, 1.0)
            };
            self.edge_fade = fade_target;
            if fade_target < 1.0 {
                busy = true;
            }
            self.emph = self.compute_emphasis();
            self.update_targets();
            if self.step_nodes(dt) {
                busy = true;
            }
            if let Err(err) = self.draw(now)
                && !self.draw_error
            {
                self.draw_error = true;
                web_sys::console::error_1(&err);
            }
            if busy || self.dirty {
                self.stirred_at = now;
            }
            if !self.reduced {
                busy = true;
            }
        }
        self.dirty = false;
        busy
    }

    /// Springs, slightly under-damped, toward each node's target.
    fn step_nodes(&mut self, dt: f64) -> bool {
        let reduced = self.reduced;
        let stiff = 90.0f64;
        let damp = 2.0 * stiff.sqrt() * 0.72;
        let intro_t = (self.now - self.intro_at.unwrap_or(self.now)) / 1000.0;
        let mut moving = false;
        let Some(model) = self.model.as_mut() else {
            return false;
        };
        for n in &mut model.nodes {
            if reduced {
                n.px = n.tx;
                n.py = n.ty;
                n.pr = n.tr;
                n.pa = n.ta;
                continue;
            }
            let delay = n.index as f64 * 0.006 + n.cat as f64 * 0.02;
            if intro_t < delay {
                continue;
            }
            let ax = (n.tx - n.px) * stiff - n.pvx * damp;
            let ay = (n.ty - n.py) * stiff - n.pvy * damp;
            n.pvx += ax * dt;
            n.pvy += ay * dt;
            n.px += n.pvx * dt;
            n.py += n.pvy * dt;
            let ar = (n.tr - n.pr) * 140.0 - n.prv * 2.0 * 140.0f64.sqrt() * 0.55;
            n.prv += ar * dt;
            n.pr = f64::max(0.0, n.pr + n.prv * dt);
            n.pa += (n.ta - n.pa) * (1.0 - (-dt * 6.0).exp());
            if (n.tx - n.px).abs() + (n.ty - n.py).abs() > 0.3
                || n.pvx.abs() + n.pvy.abs() > 0.5
                || (n.tr - n.pr).abs() > 0.05
                || (n.ta - n.pa).abs() > 0.01
            {
                moving = true;
            }
        }
        moving
    }

    // --- voice -------------------------------------------------------------------

    pub(crate) fn announce(&mut self, n: Option<usize>) {
        let Some(n) = n else { return };
        let Some(model) = &self.model else { return };
        let node = &model.nodes[n];
        let line = fmt(
            text::ANNOUNCE_NODE,
            &[
                ("name", node.name.clone()),
                ("family", model.categories[node.cat].title.clone()),
                ("role", text::role_copy(node.role.key()).0.to_string()),
                ("in", node.in_deg.to_string()),
                ("out", node.out_deg.to_string()),
            ],
        );
        self.say(&line);
    }

    /// Clear the live region now and set it a moment later, so a repeated
    /// sentence is announced again.
    pub(crate) fn say(&mut self, line: &str) {
        let Some(live) = self.q("[data-crumb-live]") else {
            return;
        };
        live.set_text_content(Some(""));
        Self::clear_timeout(self.say_timer.take());
        self.say_pending = line.to_string();
        if let (Some(h), Some(w)) = (&self.handles, web_sys::window()) {
            self.say_timer = w
                .set_timeout_with_callback_and_timeout_and_arguments_0(&h.say, 60)
                .ok();
        }
    }

    pub(crate) fn start_hold(&mut self, pointer: i32, node: usize) {
        Self::clear_timeout(self.hold_timer.take());
        self.hold = Some((pointer, node));
        if let (Some(h), Some(w)) = (&self.handles, web_sys::window()) {
            self.hold_timer = w
                .set_timeout_with_callback_and_timeout_and_arguments_0(&h.hold, 350)
                .ok();
        }
    }

    pub(crate) fn cancel_hold(&mut self) {
        Self::clear_timeout(self.hold_timer.take());
    }

    pub(crate) fn dismiss_hint(&self) {
        if let Some(hint) = self.q("[data-crumb-hint]") {
            let _ = hint.class_list().add_1("crumb__hint--gone");
        }
    }

    pub(crate) fn fresh_status(&self, n: usize) -> Option<&str> {
        let node = &self.model.as_ref()?.nodes[n];
        let status = node.change_status.as_deref().filter(|s| !s.is_empty())?;
        node.change_at.as_deref().filter(|s| !s.is_empty())?;
        if node.change_ms.is_nan() || js_sys::Date::now() - node.change_ms > 48.0 * 3600.0 * 1000.0
        {
            return None;
        }
        Some(status)
    }
}

pub(crate) struct Margins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

/// The first match of `/<key>([chars]+)/` in `hash`: the regex scans on past
/// an occurrence with nothing after it.
fn hash_match<'a>(hash: &'a str, key: &str, allowed: fn(&u8) -> bool) -> Option<&'a str> {
    let mut from = 0;
    while let Some(at) = hash[from..].find(key) {
        let rest = &hash[from + at + key.len()..];
        let len = rest.bytes().take_while(allowed).count();
        if len > 0 {
            return Some(&rest[..len]);
        }
        from += at + 1;
    }
    None
}

/// `#skill=<slug>` (`/skill=([A-Za-z0-9_-]+)/`), lower-cased.
fn hash_skill(hash: &str) -> Option<String> {
    hash_match(hash, "skill=", |b| {
        b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-'
    })
    .map(str::to_lowercase)
}

/// `#trace=<from>..<to>`, a walk between two skills, so the longest path can
/// be sent on as a skill or a view can
/// (`/trace=([A-Za-z0-9_-]+)\.\.([A-Za-z0-9_-]+)/`), lower-cased.
pub(crate) fn hash_trace(hash: &str) -> Option<(String, String)> {
    let word = |s: &str| {
        s.bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
            .count()
    };
    let mut from = 0;
    while let Some(at) = hash[from..].find("trace=") {
        let rest = &hash[from + at + "trace=".len()..];
        let a = word(rest);
        if a > 0 && rest[a..].starts_with("..") {
            let tail = &rest[a + 2..];
            let b = word(tail);
            if b > 0 {
                return Some((rest[..a].to_lowercase(), tail[..b].to_lowercase()));
            }
        }
        from += at + 1;
    }
    None
}

/// `#view=<name>` for a view the app has (`/view=([a-z]+)/`).
fn hash_view(hash: &str) -> Option<&'static str> {
    let name = hash_match(hash, "view=", u8::is_ascii_lowercase)?;
    ["constellation", "wheel", "matrix", "list"]
        .into_iter()
        .find(|v| *v == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_read_as_graph_js_reads_it() {
        assert_eq!(
            hash_skill("#skill=Clairvoyance"),
            Some("clairvoyance".into())
        );
        assert_eq!(hash_skill("#skill=a.b"), Some("a".into()));
        assert_eq!(hash_skill("#skill="), None);
        assert_eq!(hash_view("#view=wheel"), Some("wheel"));
        assert_eq!(hash_view("#view=Wheel"), None);
        assert_eq!(hash_view("#view=wheelie"), None);
        assert_eq!(hash_view("#view=1&view=list"), Some("list"));
    }

    #[test]
    fn a_walk_is_read_from_the_hash_as_graph_js_reads_it() {
        let pair = |a: &str, b: &str| Some((a.to_string(), b.to_string()));
        assert_eq!(hash_trace("#trace=atomic-commits..Karen_x"), pair("atomic-commits", "karen_x"));
        assert_eq!(hash_trace("#trace=a..b.c"), pair("a", "b"));
        assert_eq!(hash_trace("#trace=a.b"), None);
        assert_eq!(hash_trace("#trace=..b"), None);
        assert_eq!(hash_trace("#trace=a.."), None);
        assert_eq!(hash_trace("#trace=a&trace=b..c"), pair("b", "c"));
        assert_eq!(hash_trace("#view=list"), None);
    }
}
