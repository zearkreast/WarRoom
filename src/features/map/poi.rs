use crate::assets;
use crate::js::num;
use crate::ui::emblems::{EmblemArt, has_emblem};
use leptos::prelude::*;
use regex_lite::Regex;
use std::sync::LazyLock;
use warroom_core::api::Faction;
use warroom_core::effects::PlanetEffect;
use warroom_core::factions::faction_color;
use warroom_core::format::to_fixed;
use warroom_core::galaxy::Anomaly;

/// Which of the game's map logos a point of interest wears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoiKind {
    Tcs,
    Factory,
    Science,
    Industry,
    Military,
    City,
    Enemy,
    Anomaly,
    Generic,
}

impl PoiKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PoiKind::Tcs => "tcs",
            PoiKind::Factory => "factory",
            PoiKind::Science => "science",
            PoiKind::Industry => "industry",
            PoiKind::Military => "military",
            PoiKind::City => "city",
            PoiKind::Enemy => "enemy",
            PoiKind::Anomaly => "anomaly",
            PoiKind::Generic => "generic",
        }
    }
}

fn re(pattern: &str) -> Regex {
    Regex::new(&format!("(?i){pattern}")).unwrap()
}

static KINDS: LazyLock<Vec<(Regex, PoiKind)>> = LazyLock::new(|| {
    vec![
        (re("black hole|fractured"), PoiKind::Anomaly),
        (re("terminid control system"), PoiKind::Tcs),
        (re("factory hub"), PoiKind::Factory),
        (re("jet brigade factor|communications array"), PoiKind::Enemy),
        (re("science|xenoentomology|research|laborator"), PoiKind::Science),
        (re(r"factory|forge|armou?ry|mine\b|extraction|refinery|e-?711"), PoiKind::Industry),
        (re(r"training|pandora|outpost|garrison|base\b"), PoiKind::Military),
        (re("city|surveillance"), PoiKind::City),
    ]
});

pub fn poi_kind(name: &str) -> PoiKind {
    KINDS.iter().find(|(re, _)| re.is_match(name)).map_or(PoiKind::Generic, |(_, k)| *k)
}

/// Who built a point of interest: an enemy's works wear its colour, everything else is Super Earth's.
pub fn poi_origin(kind: PoiKind, enemy: Option<Faction>) -> Faction {
    match (kind, enemy) {
        (PoiKind::Enemy, Some(e)) if e != Faction::Humans => e,
        (PoiKind::Enemy, _) => Faction::Automaton,
        _ => Faction::Humans,
    }
}

/// Ready-made map marks, used instead of a drawn logo where one exists.
fn mark_of(kind: PoiKind) -> Option<String> {
    assets::url(&format!("poi-marks/{}.png", kind.as_str()))
}

static IMAGES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (re("terminid control system"), "tcs_plus.webp"),
        (re("factory hub"), "factory_hub.webp"),
        (re("center of science"), "center_of_science.webp"),
        (re("e-?711"), "e711.webp"),
        (re("training facilit"), "training.webp"),
        (re("civilian surveillance"), "surveillance.webp"),
        (re("deep mantle forge"), "forge.webp"),
    ]
});

/// The hover card's art for a planet: its first point of interest with a picture, else its anomaly's.
pub fn poi_image(pois: &[PlanetEffect], anomaly: Option<Anomaly>) -> Option<String> {
    for e in pois {
        if let Some(url) =
            IMAGES.iter().find(|(re, _)| re.is_match(&e.name)).and_then(|(_, f)| assets::url(&format!("poi/{f}")))
        {
            return Some(url);
        }
    }
    anomaly.filter(|a| *a != Anomaly::Scorched).and_then(|a| assets::url(&format!("poi/{}.webp", a.as_str())))
}

/// A band of a ring between radii r0 and r1, from angle a0 to a1 (degrees, clockwise from +x).
fn band(r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    let p = |r: f64, a: f64| {
        let t = a * std::f64::consts::PI / 180.0;
        format!("{},{}", to_fixed(12.0 + r * t.cos(), 2), to_fixed(12.0 + r * t.sin(), 2))
    };
    let large = if a1 - a0 > 180.0 { 1 } else { 0 };
    format!(
        "M{}A{},{} 0 {large} 1 {}L{}A{},{} 0 {large} 0 {}Z",
        p(r1, a0),
        num(r1),
        num(r1),
        p(r1, a1),
        p(r0, a1),
        num(r0),
        num(r0),
        p(r0, a0)
    )
}

fn dot(x: f64, y: f64, r: f64) -> String {
    format!(
        "M{},{}a{},{} 0 1 0 {},0a{},{} 0 1 0 {},0Z",
        num(x - r),
        num(y),
        num(r),
        num(r),
        num(2.0 * r),
        num(r),
        num(r),
        num(-2.0 * r)
    )
}

// The game's map logos, redrawn on a 24-unit grid as filled shapes.
fn glyph(kind: PoiKind) -> String {
    match kind {
        // Two posts joined top and bottom, framing a Terminid claw.
        PoiKind::Tcs => concat!(
            "M1.5 2.5h5v2h-1.3v15h1.3v2h-5v-2h1.3v-15H1.5ZM17.5 2.5h5v2h-1.3v15h1.3v2h-5v-2h1.3v-15h-1.3Z",
            "M6.5 5.2h11v1.5h-11ZM6.5 17.3h11v1.5h-11Z",
            "M8.4 15.6c2.6-.3 5-2.2 5.6-5.4.3-1.5 0-2.7-.6-3.5 2.7 1 4 3.6 3.3 6.4-.7 2.9-3.8 4-8.3 2.5Z",
            "M9.4 12.9c1.1-.3 2-1.3 2.3-2.6-1 .2-2 1-2.3 2.6Z"
        )
        .to_string(),
        // A ring round a core, three linked nodes and an outer arc.
        PoiKind::Science => [
            dot(12.0, 12.0, 1.9),
            band(3.3, 4.9, 0.0, 359.9),
            band(6.6, 8.0, 112.0, 248.0),
            dot(3.0, 12.0, 2.0),
            "M4.8 11.3h2.5v1.4h-2.5Z".into(),
            dot(19.2, 4.8, 2.0),
            "M14.9 8.1l2.9-2.9 1 1-2.9 2.9Z".into(),
            dot(19.2, 19.2, 2.0),
            "M15.9 14.9l2.9 2.9-1 1-2.9-2.9Z".into(),
        ]
        .concat(),
        // A mallet, head to the upper right.
        PoiKind::Industry | PoiKind::Factory => {
            "M2.6 19.6 12.4 9.8l1.8 1.8-9.8 9.8ZM11.2 6.4 15.4 2.2l6.4 6.4-4.2 4.2ZM16.6 3.4l2-2 2 2-2 2Z".into()
        }
        // A banner over two chevrons.
        PoiKind::Military => {
            "M4 2.5h16v6.2l-8 3.8-8-3.8ZM4 11.2l8 3.8 8-3.8v3.3l-8 3.8-8-3.8ZM4 16.8l8 3.8 8-3.8v3.1l-8 3.8-8-3.8Z".into()
        }
        // Columns rising to a dome.
        PoiKind::City => "M2.6 21.5 3.6 11l2.6-1.2v11.7ZM7.4 21.5V8l3-1.6v15.1ZM11.4 21.5V5.6l1.2-.4 1.2.4v15.9ZM14.8 21.5V6.4l3 1.6v13.5ZM19 21.5V9.8l2.6 1.2 1 10.5Z".into(),
        // A ring struck through, for black holes and broken worlds.
        PoiKind::Anomaly => band(6.0, 7.8, 0.0, 359.9) + "M2.5 11h19v2h-19Z",
        // A plain marker for points the map has no logo for.
        PoiKind::Generic | PoiKind::Enemy => band(5.2, 7.4, 0.0, 359.9) + &dot(12.0, 12.0, 2.4),
    }
}

/// A point of interest's logo, centred on (0, 0) and `size` map units across, in its builder's colour
/// (`faction`). Drawn logos get a soft halo from a wide translucent stroke rather than a blur filter.
#[component]
pub fn PoiGlyph(kind: PoiKind, faction: Faction, size: f64) -> impl IntoView {
    if let Some(mark) = mark_of(kind) {
        // The marks carry their own outline and glow, so they're drawn a little larger.
        let s = size * 1.25;
        return view! {
            <image class=format!("poi-mark {}", kind.as_str()) href=mark x=num(-s / 2.0) y=num(-s / 2.0) width=num(s) height=num(s) />
        }
        .into_any();
    }
    let style = format!("color: {};", faction_color(Some(faction)));
    if kind == PoiKind::Enemy {
        let art = if has_emblem(faction.as_str()) { faction.as_str() } else { "Automaton" };
        return view! {
            <g class=format!("poi-glyph {}", kind.as_str()) style=style>
                <EmblemArt art=art size=size color="currentColor" class="poi-halo" />
                <EmblemArt art=art size=size color="currentColor" class="poi-shape" />
            </g>
        }
        .into_any();
    }
    let d = glyph(kind);
    let transform = format!("translate({},{}) scale({})", num(-size / 2.0), num(-size / 2.0), num(size / 24.0));
    view! {
        <g class=format!("poi-glyph {}", kind.as_str()) style=style>
            <path class="poi-halo" d=d.clone() transform=transform.clone() />
            <path class="poi-shape" d=d transform=transform />
        </g>
    }
    .into_any()
}
