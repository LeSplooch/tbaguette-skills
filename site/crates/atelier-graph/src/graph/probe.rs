//! The probe hook: `host.__crumb`, what a headless probe reads. graph.js
//! exposed its whole App object there; the port exposes the few facts a probe
//! reads: `model.nodes.length`, `view`, `ready`, and `snapshot()`, the app's
//! state as JSON in the shape graph.js gave.

use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::Element;

use super::text::num;
use super::{App, Shared, layout};

/// `host.__crumb`: what a probe reads (`model.nodes.length`, `view`, `ready` and
/// `snapshot()`).
pub(super) fn expose(host: &Element, shared: &Rc<Shared>) -> Result<(), JsValue> {
    let obj = Object::new();
    let define = |name: &str, f: Box<dyn Fn(&App) -> JsValue>| -> Result<(), JsValue> {
        let s = Rc::downgrade(shared);
        let get = Closure::<dyn Fn() -> JsValue>::new(move || {
            s.upgrade()
                .and_then(|s| s.app.try_borrow().ok().map(|a| f(&a)))
                .unwrap_or(JsValue::NULL)
        });
        let desc = Object::new();
        Reflect::set(&desc, &"get".into(), &get.into_js_value())?;
        Object::define_property(&obj, &JsValue::from_str(name), &desc);
        Ok(())
    };
    define("view", Box::new(|a| JsValue::from_str(a.view)))?;
    define(
        "ready",
        Box::new(|a| JsValue::from_bool(a.model.is_some() && a.palette.is_some())),
    )?;
    define(
        "model",
        Box::new(|a| match &a.model {
            Some(m) => {
                let nodes = Object::new();
                let _ = Reflect::set(
                    &nodes,
                    &"length".into(),
                    &JsValue::from_f64(m.nodes.len() as f64),
                );
                let model = Object::new();
                let _ = Reflect::set(&model, &"nodes".into(), &nodes);
                model.into()
            }
            None => JsValue::NULL,
        }),
    )?;
    let s = Rc::downgrade(shared);
    let snapshot = Closure::<dyn Fn() -> JsValue>::new(move || {
        s.upgrade()
            .and_then(|s| {
                s.app
                    .try_borrow()
                    .ok()
                    .map(|a| JsValue::from_str(&a.snapshot()))
            })
            .unwrap_or(JsValue::NULL)
    });
    Reflect::set(&obj, &"snapshot".into(), &snapshot.into_js_value())?;
    Reflect::set(host, &"__crumb".into(), &obj)?;
    Ok(())
}

impl App {
    /// The app's state in the shape graph.js gave, so a probe written for one
    /// reads the other.
    fn snapshot(&self) -> String {
        use serde_json::{Value, json};
        let Some(model) = &self.model else {
            return "null".into();
        };
        let slug =
            |n: Option<usize>| n.map_or(Value::Null, |n| Value::from(model.nodes[n].slug.clone()));
        let f = |x: f64| serde_json::Number::from_f64(x).map_or(Value::Null, Value::Number);
        let bx = |b: &[f64; 4]| Value::Array(b.iter().map(|v| f(*v)).collect());
        let col = |c: &[f64; 4]| Value::Array(c.iter().map(|v| f(*v)).collect());
        let nodes: Vec<Value> = model
            .nodes
            .iter()
            .map(|n| {
                json!([
                    n.slug,
                    f(n.x),
                    f(n.y),
                    f(n.px),
                    f(n.py),
                    f(n.wx),
                    f(n.wy),
                    f(n.base_r),
                    f(n.tr),
                    f(n.ta),
                    f(n.pa),
                    f(n.sx),
                    f(n.sy),
                    f(n.sr),
                    n.role.key(),
                    n.in_deg,
                    n.out_deg,
                    n.degree,
                    n.fx.map_or(Value::Null, f)
                ])
            })
            .collect();
        let anat = self.anat.as_ref().map_or(Value::Null, |l| {
            json!({
                "r0": f(l.r0), "r1": f(l.r1), "r2": f(l.r2), "R": f(l.big_r), "crowded": l.crowded,
                "arcs": l.arcs.iter().map(|a| json!([f(a.a0), f(a.a1), a.subs.iter().map(|s| json!([f(s.a0), f(s.a1)])).collect::<Vec<_>>()])).collect::<Vec<_>>(),
                "neighbors": l.neighbors.iter().map(|i| json!([model.nodes[i.node].slug, f(i.angle), f(i.x), f(i.y), f(i.r), f(i.weight)])).collect::<Vec<_>>(),
                "bands": l.bands.iter().map(|b| json!([b.cat, f(b.a0), f(b.a1)])).collect::<Vec<_>>(),
                "links": l.links.iter().map(|k| format!("{}:{}:{}", match k.kind { layout::LinkKind::Out => "out", layout::LinkKind::Trigger => "trigger", layout::LinkKind::In => "in" }, model.nodes[l.neighbors[k.item].node].slug, num(k.count))).collect::<Vec<_>>(),
            })
        });
        let spot = self.spot.as_ref().map_or(Value::Null, |s| {
            json!({
                "type": s.kind,
                "nodes": if s.kind == "spot" { json!(s.nodes) } else { Value::Null },
                "dir": s.dir.map_or(Value::Null, Value::from),
                "from": s.from.clone().map_or(Value::Null, Value::from),
                "to": s.to.clone().map_or(Value::Null, Value::from),
            })
        });
        let trace = self.trace.as_ref().map_or(Value::Null, |t| {
            json!({ "from": slug(t.from), "to": slug(t.to), "path": t.path.as_ref().map_or(Value::Null, |p| json!(p.nodes.iter().map(|&n| model.nodes[n].slug.clone()).collect::<Vec<_>>())) })
        });
        let palette = self.palette.as_ref().map_or(Value::Null, |p| {
            json!({ "dark": p.dark, "bg": col(&p.bg), "text": col(&p.text), "text2": col(&p.text2), "foil": col(&p.foil), "focus": col(&p.focus), "cats": p.cats.iter().map(col).collect::<Vec<_>>() })
        });
        let edge_list: Vec<String> = model
            .edges
            .iter()
            .map(|e| {
                format!(
                    "{}:{}:{}",
                    e.key,
                    num(e.w),
                    e.sections
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect();
        let mut snap = serde_json::Map::new();
        snap.insert("view".into(), json!(self.view));
        snap.insert("overview".into(), json!(self.overview));
        snap.insert("focus".into(), json!(slug(self.focus)));
        snap.insert(
            "trail".into(),
            json!(
                self.trail
                    .iter()
                    .map(|&n| model.nodes[n].slug.clone())
                    .collect::<Vec<_>>()
            ),
        );
        snap.insert("hover".into(), json!(slug(self.hover)));
        snap.insert("kb".into(), json!(slug(self.kb_node)));
        snap.insert("pinSection".into(), json!(self.pin_section));
        snap.insert("hoverSection".into(), json!(self.hover_section));
        snap.insert("spot".into(), json!(spot));
        snap.insert(
            "families".into(),
            json!(self.families.clone().map_or(Value::Null, |f| json!(f))),
        );
        snap.insert("familiesPreview".into(), json!(self.families_preview));
        snap.insert("trace".into(), json!(trace));
        snap.insert("sizeBy".into(), json!(self.size_by));
        snap.insert("userCam".into(), json!(self.user_cam));
        snap.insert("stageFocused".into(), json!(self.stage_focused));
        snap.insert("touchArmed".into(), json!(slug(self.touch_armed)));
        snap.insert(
            "cam".into(),
            json!([f(self.cam.x), f(self.cam.y), f(self.cam.k)]),
        );
        snap.insert(
            "camT".into(),
            json!([f(self.cam_t.x), f(self.cam_t.y), f(self.cam_t.k)]),
        );
        snap.insert("mix".into(), json!(f(self.mix)));
        snap.insert("mixT".into(), json!(f(self.mix_t)));
        snap.insert("ring".into(), json!(f(self.ring)));
        snap.insert(
            "alpha".into(),
            json!(self.sim.as_ref().map_or(Value::Null, |s| f(s.alpha))),
        );
        snap.insert(
            "alphaTarget".into(),
            json!(self.sim.as_ref().map_or(Value::Null, |s| f(s.alpha_target))),
        );
        snap.insert("W".into(), json!(f(self.w)));
        snap.insert("H".into(), json!(f(self.h)));
        snap.insert("dpr".into(), json!(f(self.dpr)));
        snap.insert("compact".into(), json!(self.compact));
        snap.insert("sheet".into(), json!(self.sheet));
        snap.insert("reduced".into(), json!(self.reduced));
        snap.insert(
            "wheelR".into(),
            json!(self.wheel.as_ref().map_or(Value::Null, |w| f(w.r))),
        );
        snap.insert(
            "wheelLabelsAll".into(),
            json!(self.wheel.as_ref().is_some_and(|w| w.labels_all)),
        );
        snap.insert("nodes".into(), json!(nodes));
        snap.insert("edges".into(), json!(model.edges.len()));
        snap.insert(
            "mutual".into(),
            json!(model.edges.iter().filter(|e| e.mutual).count()),
        );
        snap.insert("edgeList".into(), json!(edge_list));
        snap.insert(
            "notes".into(),
            json!(
                model
                    .notes
                    .iter()
                    .map(|n| n.html.clone())
                    .collect::<Vec<_>>()
            ),
        );
        snap.insert("anat".into(), json!(anat));
        snap.insert(
            "labels".into(),
            json!(
                self.label_boxes
                    .iter()
                    .map(|(n, b)| {
                        let mut v = vec![Value::from(model.nodes[*n].slug.clone())];
                        v.extend(b.iter().map(|x| f(*x)));
                        Value::Array(v)
                    })
                    .collect::<Vec<_>>()
            ),
        );
        snap.insert(
            "anatLabels".into(),
            json!(
                self.anat_boxes
                    .iter()
                    .map(|(n, b)| {
                        let mut v = vec![Value::from(model.nodes[*n].slug.clone())];
                        v.extend(b.iter().map(|x| f(*x)));
                        Value::Array(v)
                    })
                    .collect::<Vec<_>>()
            ),
        );
        snap.insert(
            "familySpot".into(),
            json!(if self.family_spot.is_empty() {
                Value::Null
            } else {
                Value::Object(
                    self.family_spot
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::from(*v)))
                        .collect(),
                )
            }),
        );
        snap.insert(
            "familyShown".into(),
            json!(if self.family_shown.is_empty() {
                Value::Null
            } else {
                Value::Object(
                    self.family_shown
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::from(*v)))
                        .collect(),
                )
            }),
        );
        snap.insert("palette".into(), json!(palette));
        snap.insert(
            "matrixAt".into(),
            json!(self.matrix_at.map_or(Value::Null, |(a, b)| json!([a, b]))),
        );
        snap.insert(
            "overlays".into(),
            json!(
                self.overlays
                    .as_ref()
                    .map_or(Value::Null, |o| Value::Array(o.iter().map(bx).collect()))
            ),
        );
        let snap = Value::Object(snap);
        snap.to_string()
    }
}
