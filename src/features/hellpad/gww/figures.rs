//! Diagrams for G.W.W. manuals: supply lines, sieges, liberation pace, and the live impact chart.

use crate::api::war_samples::use_war_samples;
use crate::js::num;
use leptos::prelude::*;
use std::f64::consts::PI;
use warroom_core::data::Figure as FigureKind;
use warroom_core::format::{fmt_compact, to_fixed};
use warroom_core::front_activity::{Reading, average_readings};

#[derive(Clone, Copy)]
struct Node {
    id: &'static str,
    x: f64,
    y: f64,
    se: bool,
}

fn side(n: &Node) -> &'static str {
    if n.se { "se" } else { "enemy" }
}

fn planet(p: Node, ring: bool) -> impl IntoView {
    view! {
        <g transform=format!("translate({} {})", num(p.x), num(p.y)) class=format!("gf-planet {}", side(&p))>
            {ring.then(|| view! { <circle class="gf-ring" r="17" /> })}
            <circle r="11" />
            <text y="4">{p.id}</text>
        </g>
    }
}

fn link(a: Node, b: Node, class: &'static str) -> impl IntoView {
    view! { <line class=class x1=num(a.x) y1=num(a.y) x2=num(b.x) y2=num(b.y) /> }
}

/// A line from `a` towards `b` stopping short of both planets, for attack arrows.
fn attack(a: Node, b: Node, se: bool) -> impl IntoView {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let d = dx.hypot(dy);
    let (ux, uy) = (dx / d, dy / d);
    let side = if se { "se" } else { "enemy" };
    view! {
        <line
            class=format!("gf-attack {side}")
            x1=num(a.x + ux * 16.0)
            y1=num(a.y + uy * 16.0)
            x2=num(b.x - ux * 21.0)
            y2=num(b.y - uy * 21.0)
            marker-end=format!("url(#gf-head-{side})")
        />
    }
}

fn heads() -> impl IntoView {
    view! {
        <defs>
            {["se", "enemy"]
                .map(|s| view! {
                    <marker id=format!("gf-head-{s}") viewBox="0 0 10 10" refX="6" refY="5" markerWidth="7" markerHeight="7" orient="auto">
                        <path d="M0 0L10 5L0 10z" class=format!("gf-head {s}") />
                    </marker>
                })
                .collect_view()}
        </defs>
    }
}

/// A defense on C launched from D, and Super Earth's liberation of D from B: the gambit.
fn supply_lines() -> AnyView {
    let p = |id: &'static str| -> Node {
        match id {
            "A" => Node { id, x: 70.0, y: 110.0, se: true },
            "B" => Node { id, x: 180.0, y: 50.0, se: true },
            "C" => Node { id, x: 180.0, y: 170.0, se: true },
            "D" => Node { id, x: 310.0, y: 110.0, se: false },
            "E" => Node { id, x: 430.0, y: 50.0, se: false },
            _ => Node { id: "F", x: 430.0, y: 170.0, se: false },
        }
    };
    let links = ["AB", "AC", "BC", "BD", "CD", "DE", "DF", "EF"];
    view! {
        <figure class="gww-fig">
            <svg
                viewBox="0 0 500 236"
                role="img"
                aria-label="Supply line diagram: enemy planet D attacks Super Earth planet C while Super Earth liberates D from B."
            >
                {heads()}
                {links.map(|k| link(p(&k[..1]), p(&k[1..]), "gf-link")).collect_view()}
                {attack(p("D"), p("C"), false)}
                {attack(p("B"), p("D"), true)}
                {["A", "B", "C", "D", "E", "F"].map(|id| planet(p(id), id == "C" || id == "D")).collect_view()}
                <text class="gf-label" x="180" y="210" text-anchor="middle">"Defense of C"</text>
                <text class="gf-label" x="310" y="150" text-anchor="middle">"Gambit: take D"</text>
                <g class="gf-legend" transform="translate(16 226)">
                    <circle class="gf-dot se" r="5" />
                    <text x="10" y="4">"Super Earth"</text>
                    <circle class="gf-dot enemy" r="5" cx="104" />
                    <text x="114" y="4">"Enemy"</text>
                    <line class="gf-attack enemy" x1="178" x2="206" y1="0" y2="0" />
                    <text x="214" y="4">"Attack along a supply line"</text>
                </g>
            </svg>
            <figcaption>
                "D attacks C along their supply line. Holding C needs the defense won in time, or D liberated first. Cut A, B and C, and D can't be reached at all."
            </figcaption>
        </figure>
    }
    .into_any()
}

/// One planet alone among Super Earth's falls under siege; a pair cut off together doesn't.
fn siege() -> AnyView {
    let ring = |cx: f64, cy: f64, r: f64, count: usize, from: f64| -> Vec<Node> {
        (0..count)
            .map(|i| {
                let a = from + (i as f64 / count as f64) * PI * 2.0;
                Node { id: "", x: cx + a.cos() * r, y: cy + a.sin() * r, se: true }
            })
            .collect()
    };
    let alone = Node { id: "", x: 120.0, y: 100.0, se: false };
    let around = ring(120.0, 100.0, 62.0, 6, -PI / 2.0);
    let pair = [Node { id: "", x: 350.0, y: 100.0, se: false }, Node { id: "", x: 410.0, y: 100.0, se: false }];
    let fence = ring(380.0, 100.0, 70.0, 8, -PI / 2.0);
    let small = |p: &Node| {
        view! {
            <g transform=format!("translate({} {})", num(p.x), num(p.y)) class="gf-planet se small">
                <circle r="7" />
            </g>
        }
    };
    view! {
        <figure class="gww-fig">
            <svg
                viewBox="0 0 500 214"
                role="img"
                aria-label="Siege diagram: a lone enemy planet surrounded by Super Earth planets liberates itself; two enemy planets cut off together do not."
            >
                {around.iter().enumerate().map(|(i, p)| link(*p, around[(i + 1) % around.len()], "gf-link")).collect_view()}
                {around.iter().map(|p| link(*p, alone, "gf-link strong")).collect_view()}
                {fence.iter().enumerate().map(|(i, p)| link(*p, fence[(i + 1) % fence.len()], "gf-link")).collect_view()}
                {fence.iter().map(|p| link(*p, pair[if p.x < 380.0 { 0 } else { 1 }], "gf-link")).collect_view()}
                {link(pair[0], pair[1], "gf-link")}
                {around.iter().chain(fence.iter()).map(small).collect_view()}
                <g transform=format!("translate({} {})", num(alone.x), num(alone.y)) class="gf-planet enemy siege">
                    <circle class="gf-ring" r="17" />
                    <circle r="11" />
                </g>
                {pair
                    .iter()
                    .map(|p| view! {
                        <g transform=format!("translate({} {})", num(p.x), num(p.y)) class="gf-planet enemy">
                            <circle r="11" />
                        </g>
                    })
                    .collect_view()}
                <text class="gf-label good" x="120" y="196" text-anchor="middle">"Alone: SEAF siege, −0.5 to −1% an hour"</text>
                <text class="gf-label" x="380" y="196" text-anchor="middle">"Cut off as a pair: no siege"</text>
            </svg>
            <figcaption>"Only a planet with no neighbour of its own side left falls under a SEAF siege."</figcaption>
        </figure>
    }
    .into_any()
}

/// Liberation over a day at three turnouts, against the same resistance. Illustrative, not data.
fn decay() -> AnyView {
    const W: f64 = 500.0;
    const H: f64 = 200.0;
    let x = |h: f64| 44.0 + (h / 24.0) * (W - 150.0);
    let y = |pct: f64| 16.0 + (1.0 - pct / 100.0) * (H - 48.0);
    let line = move |net: f64| {
        (0..25)
            .map(|h| {
                let h = h as f64;
                format!(
                    "{}{} {}",
                    if h > 0.0 { "L" } else { "M" },
                    to_fixed(x(h), 1),
                    to_fixed(y((40.0 + net * h).clamp(0.0, 100.0)), 1)
                )
            })
            .collect::<String>()
    };
    let series: [(f64, &str, &str); 3] =
        [(2.2, "Above resistance", "good"), (0.0, "Matching it", "flat"), (-1.2, "Below it", "bad")];
    view! {
        <figure class="gww-fig">
            <svg
                viewBox=format!("0 0 {} {}", num(W), num(H))
                role="img"
                aria-label="Illustration: liberation rises when turnout outpaces resistance, holds when it matches, and falls when it doesn't."
            >
                {[0.0, 50.0, 100.0]
                    .map(|p| view! {
                        <g>
                            <line class="gf-grid" x1=num(x(0.0)) x2=num(x(24.0)) y1=num(y(p)) y2=num(y(p)) />
                            <text class="gf-tick" x=num(x(0.0) - 8.0) y=num(y(p) + 4.0) text-anchor="end">{format!("{}%", num(p))}</text>
                        </g>
                    })
                    .collect_view()}
                {[0.0, 12.0, 24.0]
                    .map(|h| view! { <text class="gf-tick" x=num(x(h)) y=num(H - 10.0) text-anchor="middle">{format!("{} h", num(h))}</text> })
                    .collect_view()}
                {series
                    .map(|(net, label, class)| {
                        let end = (40.0 + net * 24.0).clamp(0.0, 100.0);
                        view! {
                            <g class=format!("gf-series {class}")>
                                <path d=line(net) />
                                <text x=num(x(24.0) + 8.0) y=num(y(end) + 4.0)>{label}</text>
                            </g>
                        }
                    })
                    .collect_view()}
            </svg>
            <figcaption>
                "An illustration: the same planet from 40%, with Helldivers dealing more, as much as, or less than it wins back."
            </figcaption>
        </figure>
    }
    .into_any()
}

/// The last day's impact modifier against Helldivers online, from this tracker's collector.
#[component]
fn ImpactLive() -> impl IntoView {
    let readings = use_war_samples().data;
    let points = Memo::new(move |_| {
        let list: Vec<Reading> = readings
            .with(|r| {
                r.as_ref().map(|r| {
                    r.iter()
                        .map(|s| Reading {
                            at: s.at,
                            impact: s.impact,
                            players: s.players,
                            terminids: s.terminids,
                            automatons: s.automatons,
                            illuminate: s.illuminate,
                        })
                        .collect()
                })
            })
            .unwrap_or_default();
        average_readings(&list, 10.0 * 60_000.0)
            .into_iter()
            .filter(|r| r.players > 0.0 && r.impact > 0.0)
            .collect::<Vec<_>>()
    });
    move || {
        let points = points.get();
        if points.len() < 6 {
            return None;
        }
        const W: f64 = 500.0;
        const H: f64 = 220.0;
        let fold = |f: fn(&Reading) -> f64| {
            points.iter().map(f).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(v), b.max(v)))
        };
        let (x0, x1) = fold(|p| p.players);
        let (y0, y1) = fold(|p| p.impact);
        let or1 = |d: f64| if d != 0.0 { d } else { 1.0 };
        let x = move |v: f64| 58.0 + ((v - x0) / or1(x1 - x0)) * (W - 78.0);
        let y = move |v: f64| 14.0 + (1.0 - (v - y0) / or1(y1 - y0)) * (H - 52.0);
        let n = points.len();
        Some(view! {
            <figure class="gww-fig">
                <svg
                    viewBox=format!("0 0 {} {}", num(W), num(H))
                    role="img"
                    aria-label="Scatter of the last day: the impact modifier is lower when more Helldivers are online."
                >
                    <line class="gf-grid" x1=num(x(x0)) x2=num(x(x1)) y1=num(y(y0)) y2=num(y(y0)) />
                    <line class="gf-grid" x1=num(x(x0)) x2=num(x(x0)) y1=num(y(y0)) y2=num(y(y1)) />
                    {points
                        .iter()
                        .enumerate()
                        .map(|(i, p)| view! {
                            <circle
                                class="gf-point"
                                cx=num(x(p.players))
                                cy=num(y(p.impact))
                                r="3"
                                style=format!("opacity: {};", num(0.25 + 0.75 * (i as f64 / (n - 1) as f64)))
                            />
                        })
                        .collect_view()}
                    <text class="gf-tick" x=num(x(x0)) y=num(H - 20.0) text-anchor="start">{fmt_compact(x0)}</text>
                    <text class="gf-tick" x=num(x(x1)) y=num(H - 20.0) text-anchor="end">{fmt_compact(x1)}</text>
                    <text class="gf-tick" x=num((x(x0) + x(x1)) / 2.0) y=num(H - 6.0) text-anchor="middle">"Helldivers online"</text>
                    <text class="gf-tick" x=num(x(x0) - 6.0) y=num(y(y1) + 4.0) text-anchor="end">{to_fixed(y1, 4)}</text>
                    <text class="gf-tick" x=num(x(x0) - 6.0) y=num(y(y0) + 4.0) text-anchor="end">{to_fixed(y0, 4)}</text>
                </svg>
                <figcaption>
                    "Live readings: each dot is ten minutes of the last day, brighter for more recent. More Helldivers online, lower modifier."
                </figcaption>
            </figure>
        })
    }
}

pub fn figure(kind: FigureKind) -> AnyView {
    match kind {
        FigureKind::SupplyLines => supply_lines(),
        FigureKind::Siege => siege(),
        FigureKind::Decay => decay(),
        FigureKind::ImpactLive => view! { <ImpactLive /> }.into_any(),
    }
}
