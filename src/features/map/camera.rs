//! The counter-transforms that keep planet markers and labels upright, round and slowly growing however
//! the camera turns, tilts and zooms. They are a handful of rules rewritten each frame: an inherited
//! CSS variable would restyle every node in the map on each change (11-18 ms on desktop).

use crate::js;
use std::cell::RefCell;
use std::collections::HashMap;
use warroom_core::format::to_fixed;
use wasm_bindgen::JsCast;

const SELECTORS: [(&str, &str); 11] = [
    ("upright", ".galaxy .upright"),
    ("body", ".galaxy .pbody"),
    ("label", ".galaxy .planet-label"),
    ("tag", ".galaxy .planet-tag"),
    ("sphereLabel", ".galaxy.pl-spheres .planet-label"),
    ("sphereTag", ".galaxy.pl-spheres .planet-tag"),
    ("poiLabel", ".galaxy .planet.poi .planet-label"),
    ("poiSphereLabel", ".galaxy.pl-spheres .planet.poi .planet-label"),
    ("holoFloor", ".galaxy .holo-floor"),
    ("holoBeam", ".galaxy .holo-beam, .galaxy .holo-edge"),
    ("holoLift", ".galaxy .holo-lift"),
];

thread_local! {
    static RULES: RefCell<Option<Vec<web_sys::CssStyleRule>>> = const { RefCell::new(None) };
    static WRITTEN: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn with_rules<T>(f: impl FnOnce(&[web_sys::CssStyleRule]) -> T) -> Option<T> {
    RULES.with_borrow_mut(|rules| {
        if rules.is_none() {
            let sheet = web_sys::CssStyleSheet::new().ok()?;
            for (i, (_, sel)) in SELECTORS.iter().enumerate() {
                sheet.insert_rule_with_index(&format!("{sel} {{}}"), i as u32).ok()?;
            }
            let doc = js::document();
            let adopted = js_sys::Array::from(&js::get(&doc, "adoptedStyleSheets"));
            adopted.push(&sheet);
            js_sys::Reflect::set(&doc, &"adoptedStyleSheets".into(), &adopted).ok()?;
            let list = sheet.css_rules().ok()?;
            *rules = Some((0..SELECTORS.len() as u32).filter_map(|i| list.item(i)?.dyn_into().ok()).collect());
        }
        Some(f(rules.as_deref()?))
    })
}

fn set(name: &str, prop: &str, value: String) {
    let key = format!("{name}:{prop}");
    if WRITTEN.with_borrow(|w| w.get(&key) == Some(&value)) {
        return;
    }
    let Some(i) = SELECTORS.iter().position(|(n, _)| *n == name) else { return };
    with_rules(|rules| {
        if let Some(rule) = rules.get(i) {
            let _ = rule.style().set_property(prop, &value);
        }
    });
    WRITTEN.with_borrow_mut(|w| w.insert(key, value));
}

fn px(v: f64) -> String {
    format!("{}px", to_fixed(v, 4))
}

/// Writes the counter-transforms for zoom `k`, a turn of `rot` radians and a tilt of `tilt` radians.
pub fn write_camera(k: f64, rot: f64, tilt: f64) {
    set(
        "upright",
        "transform",
        format!(
            "rotate({}deg) scale(1, {})",
            to_fixed((-rot * 180.0) / std::f64::consts::PI, 3),
            to_fixed(1.0 / tilt.cos(), 4)
        ),
    );
    // Markers grow slower than the map when zooming in.
    let marker = 1.4 / k.powf(0.55);
    set("body", "transform", format!("scale({})", to_fixed(marker, 4)));
    let below = px(8.0 * marker + 9.0 / k);
    set("label", "font-size", px(10.0 / k));
    set("label", "stroke-width", px(3.0 / k));
    set("label", "transform", format!("translateY({})", px(-8.0 * marker - 3.0 / k)));
    set("tag", "font-size", px(8.0 / k));
    set("tag", "stroke-width", px(3.0 / k));
    set("tag", "transform", format!("translateY({below})"));
    // Spheres carry the active-planet tag on top, so their names go underneath, as on the in-game map.
    set("sphereLabel", "transform", format!("translateY({below})"));
    set("sphereTag", "transform", format!("translateY({})", px(8.0 * marker + 19.0 / k)));
    // Over a point of interest's logo, which sits where the name would.
    set("poiLabel", "transform", format!("translateY({})", px(-16.5 * marker - 3.0 / k)));
    set("poiSphereLabel", "transform", format!("translateY({below})"));
    // Hologram projectors lie flat on the table, flatter as the map tilts; units float higher as it tilts away.
    let lift = 4.0 + 6.0 * tilt.sin();
    set("holoFloor", "transform", format!("scale(1, {})", to_fixed(0.5f64.min(tilt.cos()), 4)));
    set("holoBeam", "transform", format!("scale(1, {})", to_fixed(lift + 6.0, 4)));
    set("holoLift", "transform", format!("translateY({})", px(-lift)));
}
