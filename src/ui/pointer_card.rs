use crate::js::{self, num};
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use warroom_core::format::fmt_int;

/// Pointer position plus the centre of the chart it hovers, so the card opens away from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerAt {
    pub x: f64,
    pub y: f64,
    pub cx: f64,
    pub cy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CardRow {
    pub key: String,
    pub name: String,
    /// Helldivers; None for rows without live data (show `note` instead).
    pub value: Option<f64>,
    pub note: Option<String>,
    /// Bar colour; defaults to the card's tone.
    pub color: Option<String>,
}

const WIDTH: f64 = 260.0;
const HEIGHT: f64 = 300.0;
const OFFSET: f64 = 18.0;

pub fn pointer_at(e: &web_sys::MouseEvent, anchor: &web_sys::Element) -> PointerAt {
    let r = anchor.get_bounding_client_rect();
    PointerAt {
        x: e.client_x() as f64,
        y: e.client_y() as f64,
        cx: r.left() + r.width() / 2.0,
        cy: r.top() + r.height() / 2.0,
    }
}

fn place(at: PointerAt) -> (f64, f64) {
    let w = js::window();
    let iw = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let ih = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let clamp_x = |v: f64| 8f64.max(v.min(iw - WIDTH - 8.0));
    let clamp_y = |v: f64| 8f64.max(v.min(ih - HEIGHT - 8.0));
    let side_x = if at.x < at.cx { at.x - WIDTH - OFFSET } else { at.x + OFFSET };
    if side_x >= 8.0 && side_x + WIDTH <= iw - 8.0 {
        return (side_x, clamp_y(at.y - HEIGHT / 2.0));
    }
    // No room beside the chart: open above or below the pointer instead.
    (clamp_x(at.x - WIDTH / 2.0), clamp_y(if at.y < at.cy { at.y - HEIGHT - OFFSET } else { at.y + OFFSET }))
}

/// Hover card that follows the pointer: a titled list of rows with relative bars.
#[component]
pub fn PointerCard(
    at: PointerAt,
    #[prop(into)] tone: String,
    #[prop(optional)] icon: Option<AnyView>,
    #[prop(into)] title: String,
    #[prop(optional)] badge: Option<AnyView>,
    rows: Vec<CardRow>,
    #[prop(optional)] footer: Option<AnyView>,
) -> impl IntoView {
    let max = rows.iter().filter_map(|r| r.value).fold(1.0, f64::max);
    let (left, top) = place(at);
    let tone2 = tone.clone();
    view! {
        <div
            class="fmap"
            style=format!("left: {}px; top: {}px; --k: {tone};", num(left), num(top))
            aria-hidden="true"
        >
            <div class="fmap-head">
                {icon} <b>{title}</b> {badge.map(|b| view! { <span class="num">{b}</span> })}
            </div>
            <ol>
                {rows
                    .into_iter()
                    .map(|r| {
                        let width = r.value.unwrap_or(0.0) / max * 100.0;
                        let k = r.color.clone().unwrap_or_else(|| tone2.clone());
                        view! {
                            <li>
                                <span class="fmap-name">{r.name}</span>
                                <span class="fmap-val">
                                    {match r.value {
                                        None => r.note.unwrap_or_default().into_any(),
                                        Some(v) => {
                                            view! {
                                                <Icon icon=icons::USERS size=10.0 />
                                                {format!(" {}", fmt_int(v))}
                                            }
                                                .into_any()
                                        }
                                    }}
                                </span>
                                <span class="fmap-bar">
                                    <span style=format!("width: {}%; --k: {k};", num(width))></span>
                                </span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
            {footer.map(|f| view! { <div class="fmap-foot">{f}</div> })}
        </div>
    }
}
