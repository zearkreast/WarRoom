use crate::js::num;
use leptos::html;
use leptos::prelude::*;
use std::f64::consts::PI;
use warroom_core::format::to_fixed;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
pub struct RingSegment {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub color: String,
}

const R: f64 = 92.0;
/// Radians between arcs.
const GAP: f64 = 0.035;

fn arc_path(a0: f64, a1: f64) -> String {
    // A full circle can't be one arc command; split it.
    if a1 - a0 >= PI * 2.0 - 1e-6 {
        return format!("{} {}", arc_path(a0, a0 + PI), arc_path(a0 + PI, a1));
    }
    let p = |a: f64| format!("{} {}", to_fixed(a.cos() * R, 2), to_fixed(a.sin() * R, 2));
    format!("M {} A {R} {R} 0 {} 1 {}", p(a0), u8::from(a1 - a0 > PI), p(a1))
}

/// Segmented neon ring. Each arc is its own hover target; the parent owns which one is active
/// and what the centre shows. A touch has no hover, so a tap shows an arc until the next tap, and
/// a second tap on it selects it (or lets go, without `on_select`).
#[component]
pub fn NeonRing(
    #[prop(into)] segments: Signal<Vec<RingSegment>>,
    #[prop(into)] active: Signal<Option<String>>,
    on_active: Callback<Option<String>>,
    #[prop(optional)] on_pointer_move: Option<Callback<(String, web_sys::MouseEvent)>>,
    #[prop(optional)] on_select: Option<Callback<String>>,
    center: ChildrenFn,
    #[prop(into)] label: Signal<String>,
    #[prop(default = 240.0)] size: f64,
) -> impl IntoView {
    let root = NodeRef::<html::Div>::new();
    let touch = StoredValue::new(false);
    // A tap elsewhere lets go of an arc a tap picked.
    let away = window_event_listener(leptos::ev::pointerdown, move |e| {
        if active.get_untracked().is_none() || !touch.get_value() {
            return;
        }
        let target = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        if !root.get_untracked().is_some_and(|r| r.contains(target.as_ref())) {
            on_active.run(None);
        }
    });
    on_cleanup(move || away.remove());
    let tap = move |key: String, e: web_sys::MouseEvent| {
        if !touch.get_value() {
            if let Some(f) = on_select {
                f.run(key);
            }
        } else if active.get_untracked().as_deref() != Some(key.as_str()) {
            on_active.run(Some(key.clone()));
            if let Some(f) = on_pointer_move {
                f.run((key, e));
            }
        } else if let Some(f) = on_select {
            f.run(key);
        } else {
            on_active.run(None);
        }
    };
    let arcs = move || {
        let segments = segments.get();
        let total = match segments.iter().map(|s| s.value).sum::<f64>() {
            t if t == 0.0 => 1.0,
            t => t,
        };
        let gap = if segments.iter().filter(|s| s.value > 0.0).count() > 1 { GAP } else { 0.0 };
        let mut angle = -PI / 2.0;
        segments
            .into_iter()
            .filter_map(|s| {
                let sweep = (s.value / total) * PI * 2.0;
                let (a0, a1) = (angle + gap / 2.0, angle + sweep - gap / 2.0);
                angle += sweep;
                (a1 > a0).then(|| {
                    let d = arc_path(a0, a1);
                    let key = s.key.clone();
                    let (k1, k2, k3) = (key.clone(), key.clone(), key.clone());
                    view! {
                        <path
                            d=d
                            class=move || format!("nr-arc {}", if active.get().as_deref() == Some(k1.as_str()) { "on" } else { "" })
                            style=format!("--c: {};", s.color)
                            on:pointerenter=move |e| {
                                if e.pointer_type() == "mouse" {
                                    on_active.run(Some(k2.clone()))
                                }
                            }
                            on:pointermove=move |e| {
                                if e.pointer_type() == "mouse"
                                    && let Some(f) = on_pointer_move
                                {
                                    f.run((k3.clone(), e.into()))
                                }
                            }
                            on:click=move |e| tap(key.clone(), e)
                        />
                    }
                })
            })
            .collect_view()
    };
    view! {
        <div
            node_ref=root
            class=move || format!("nr {}", if active.get().is_some() { "has-active" } else { "" })
            style=format!("width: {}px; height: {}px;", num(size), num(size))
            role="img"
            aria-label=move || label.get()
            on:pointerdown=move |e| touch.set_value(e.pointer_type() != "mouse")
            on:pointerleave=move |e| {
                if e.pointer_type() == "mouse" {
                    on_active.run(None)
                }
            }
        >
            <svg viewBox="-120 -120 240 240" aria-hidden="true">
                <circle r="110" class="nr-dots" />
                <circle r=num(R) class="nr-track" />
                {arcs}
            </svg>
            <div class="nr-center">{center()}</div>
        </div>
    }
}
