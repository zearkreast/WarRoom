use crate::assets;
use crate::features::map::pawn_art::DssState;
use crate::js::{num, unique_id};
use crate::ui::emblems::StationEmblem;
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use std::f64::consts::PI;

#[component]
fn Holo(
    #[prop(into)] tone: String,
    title: AnyView,
    #[prop(optional)] stamp: Option<&'static str>,
    readout: Vec<(String, AnyView)>,
    children: Children,
) -> impl IntoView {
    let still = RwSignal::new(false);
    view! {
        <aside
            class=move || format!("st-holo {}", if still.get() { "still" } else { "" })
            style=format!("--holo: {tone};")
            aria-label="Station hologram"
        >
            <header class="st-holo-head caps">
                {title}
                <button
                    class="st-holo-still"
                    aria-pressed=move || still.get().to_string()
                    aria-label="Hold the hologram still"
                    title=move || if still.get() { "Play" } else { "Hold still" }
                    on:click=move |_| still.update(|s| *s = !*s)
                >
                    {move || view! { <Icon icon=if still.get() { icons::PLAY } else { icons::PAUSE } size=12.0 /> }}
                </button>
            </header>
            <div class="st-holo-stage">
                {children()}
                <i class="st-holo-sweep" aria-hidden="true"></i>
                {stamp.map(|s| view! { <span class="st-holo-stamp caps">{s}</span> })}
            </div>
            <i class="st-holo-base" aria-hidden="true"></i>
            <dl class="st-holo-read">
                {readout
                    .into_iter()
                    .map(|(k, v)| view! {
                        <div>
                            <dt class="caps">{k}</dt>
                            <dd class="num">{v}</dd>
                        </div>
                    })
                    .collect_view()}
            </dl>
        </aside>
    }
}

/// The Democracy Space Station side-on: the game's own summary hologram, lit by the station's state.
#[component]
pub fn DssHolo(state: DssState, readout: Vec<(String, AnyView)>) -> impl IntoView {
    let art = assets::url("stations/dss-holo.webp").unwrap_or_default();
    view! {
        <Holo
            tone=state.tone()
            title=view! {
                <StationEmblem station="dss" size=14.0 />
                " DSS · side view"
            }
            .into_any()
            readout=readout
        >
            <i class="st-holo-art" style=format!("--art: url(\"{art}\");") role="img" aria-label="The Democracy Space Station"></i>
        </Holo>
    }
}

const CX: f64 = 100.0;
const CY: f64 = 150.0;
const R: f64 = 70.0;

/// The visible half of a latitude ring, and its hidden half, `t` from -1 (south pole) to 1.
fn latitude(t: f64) -> (String, String) {
    let y = CY - t * R;
    let rx = R * (1.0 - t * t).sqrt();
    let ry = rx * 0.16;
    let (x0, x1, rx, ry, y) = (num(CX - rx), num(CX + rx), num(rx), num(ry), num(y));
    (format!("M{x0} {y}A{rx} {ry} 0 0 0 {x1} {y}"), format!("M{x0} {y}A{rx} {ry} 0 0 1 {x1} {y}"))
}

/// A meridian's visible half, at longitude `deg` from the line of sight.
fn meridian(deg: f64) -> String {
    let rx = (R * (deg * PI / 180.0).sin()).abs();
    format!(
        "M{} {}A{} {} 0 0 {} {} {}",
        num(CX),
        num(CY - R),
        num(rx),
        num(R),
        if deg > 0.0 { 1 } else { 0 },
        num(CX),
        num(CY + R)
    )
}

const SEAMS: [f64; 6] = [-62.0, -38.0, -14.0, 12.0, 36.0, 60.0];

// Red lights along the seams, as (longitude, latitude) in degrees.
fn lights() -> Vec<(f64, f64)> {
    let lats = [-48.0, -26.0, -8.0, 14.0, 32.0, 52.0];
    SEAMS
        .iter()
        .enumerate()
        .flat_map(|(i, lon)| {
            lats.iter()
                .enumerate()
                .filter(move |(j, _)| (i + j) % 3 != 0)
                .enumerate()
                .map(move |(j, (_, lat))| (*lon, lat + ((i * 7 + j * 5) % 9) as f64 - 4.0))
        })
        .collect()
}

const CRACK: &str = "175,108 152,136 136,124 114,160 94,148 72,190 52,178 24,212";

fn sphere_wire() -> impl IntoView {
    let p = |dx: f64, dy: f64| format!("{} {}", num(CX + dx), num(CY + dy));
    view! {
        <g class="ash-wire">
            <circle cx=num(CX) cy=num(CY) r=num(R) class="ash-rim" />
            {[-0.78, -0.5, -0.2, 0.12, 0.42, 0.72]
                .map(|t| {
                    let (front, back) = latitude(t);
                    view! {
                        <g>
                            <path d=back class="ash-back" />
                            <path d=front />
                        </g>
                    }
                })
                .collect_view()}
            {SEAMS.map(|d| view! { <path d=meridian(d) class="ash-seam" /> }).collect_view()}
            {lights()
                .into_iter()
                .map(|(lon, lat)| {
                    let x = CX + R * (lat * PI / 180.0).cos() * (lon * PI / 180.0).sin();
                    let y = CY - R * (lat * PI / 180.0).sin();
                    view! { <rect x=num(x - 0.8) y=num(y - 2.5) width="1.6" height="5" class="ash-light" /> }
                })
                .collect_view()}
            <path d=format!("M{}A24 6 0 0 0 {}", p(-24.0, -R + 7.0), p(24.0, -R + 7.0)) />
            <path d=format!("M{}L{}M{}L{}", p(-24.0, -R + 7.0), p(-19.0, -R - 5.0), p(24.0, -R + 7.0), p(19.0, -R - 5.0)) />
            <ellipse cx=num(CX) cy=num(CY - R - 5.0) rx="19" ry="4.5" />
            <path d=format!("M{}V{}M{}V{}", p(-8.0, -R - 9.0), num(CY - R - 24.0), p(10.0, -R - 9.0), num(CY - R - 18.0)) />
            <circle cx=num(CX - 8.0) cy=num(CY - R - 26.0) r="1.8" class="ash-light" />
            <circle cx=num(CX + 10.0) cy=num(CY - R - 20.0) r="1.5" class="ash-light" />
            <path d=format!("M{}H{}V{}H{}M{}V{}", p(67.0, -18.0), num(CX + 94.0), num(CY - 9.0), num(CX + 67.0), p(80.0, -18.0), num(CY - 9.0)) />
            <path d=format!("M{}L{}L{}", p(-50.0, 49.0), p(-66.0, 62.0), p(-58.0, 42.0)) />
        </g>
    }
}

/// The Automaton station as drawn from its in-game render, broken in two the way it ended.
#[component]
pub fn CssHolo(readout: Vec<(String, AnyView)>) -> impl IntoView {
    let id = unique_id("css-");
    let debris: [(f64, f64, f64); 6] = [
        (70.0, 262.0, 6.0),
        (118.0, 280.0, 4.0),
        (92.0, 318.0, 5.0),
        (140.0, 330.0, 3.0),
        (58.0, 350.0, 3.0),
        (106.0, 372.0, 4.0),
    ];
    view! {
        <Holo
            tone="var(--f-automaton)"
            title=view! {
                <StationEmblem station="css" size=14.0 />
                " CSS · last scan"
            }
            .into_any()
            stamp="Destroyed"
            readout=readout
        >
            <svg class="st-holo-svg glitch" viewBox="0 0 200 400" role="img" aria-label="The Automaton station, broken apart">
                <defs>
                    <clipPath id=format!("{id}-upper")>
                        <polygon points=format!("0,0 200,0 200,108 {CRACK} 0,212") />
                    </clipPath>
                    <clipPath id=format!("{id}-lower")>
                        <polygon points=format!("200,108 {CRACK} 0,212 0,400 200,400") />
                    </clipPath>
                </defs>
                <g clip-path=format!("url(#{id}-upper)") transform="translate(-6 -6) rotate(-4 100 150)">
                    {sphere_wire()}
                </g>
                <g clip-path=format!("url(#{id}-lower)") transform="translate(6 8) rotate(5 100 150)">
                    {sphere_wire()}
                </g>
                <g class="ash-debris">
                    {debris
                        .iter()
                        .enumerate()
                        .map(|(i, (x, y, s))| view! {
                            <g transform=format!("translate({} {})", num(*x), num(*y))>
                                <path
                                    d=format!("M0 {}L{} {}L{} {}Z", num(-s), num(*s), num(s * 0.6), num(-s * 0.8), num(s * 0.4))
                                    transform=format!("rotate({})", i * 47)
                                />
                                <path d=format!("M0 {}V{}", num(-s - 2.0), num(-s - 14.0 - i as f64 * 3.0)) class="ash-trail" />
                            </g>
                        })
                        .collect_view()}
                </g>
            </svg>
        </Holo>
    }
}
