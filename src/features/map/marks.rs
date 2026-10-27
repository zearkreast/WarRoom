//! What the map draws at each planet: its marker or sphere, anomalies, tags, logos and holograms.

use crate::features::map::pawn_art::{Art, DssInfo, DssState, PAWNS, pawn_art, pawn_href};
use crate::features::map::poi::{PoiGlyph, PoiKind, poi_origin};
use crate::js::num;
use crate::state::MapPlanets;
use crate::ui::emblems::{EmblemArt, subfaction_art};
use crate::ui::icons::{self, IconData};
use leptos::prelude::*;
use warroom_core::api::Faction;
use warroom_core::effects::{EffectKind, PlanetEffect};
use warroom_core::factions::faction_color;
use warroom_core::format::{js_round, to_fixed};
use warroom_core::galaxy::Anomaly;

const FACTIONS: [Faction; 4] = [Faction::Humans, Faction::Terminids, Faction::Automaton, Faction::Illuminate];

/// The tints as linear RGB weights, close to the faction colours.
const HOLO_TINT: [(&str, [f64; 3]); 7] = [
    ("bug", [1.0, 0.58, 0.2]),
    ("bot", [1.0, 0.44, 0.44]),
    ("ill", [0.8, 0.54, 0.91]),
    ("humans", [0.27, 0.64, 1.0]),
    ("cta", [1.0, 1.0, 0.35]),
    ("bad", [1.0, 0.35, 0.35]),
    ("off", [0.55, 0.57, 0.6]),
];

/// Hologram size and spacing in marker units: drawn at HOLO_SIZE, shrunk by HOLO_SCALE, kept close to
/// the planet so it's clear whose they are.
const HOLO_SIZE: f64 = 22.0;
const HOLO_SCALE: f64 = 0.78;
const HOLO_X: f64 = 16.0;
const HOLO_STEP: f64 = 11.0;

/// Size and spacing of the DSS's active-action badges, in hologram units.
const BADGE: f64 = 9.0;
const BADGE_STEP: f64 = 10.0;

/// Logos per planet and their size in marker units; keep in step with the name lift in camera.rs.
pub const POI_MAX: usize = 2;
const POI_SIZE: f64 = 10.0;

fn stop(offset: &'static str, color: &'static str, opacity: f64) -> impl IntoView {
    view! { <stop offset=offset stop-color=color stop-opacity=num(opacity)></stop> }
}

fn tint_matrix(rgb: [f64; 3]) -> String {
    let rows: Vec<String> = rgb
        .iter()
        .map(|c| {
            let m = c * 1.6 + (1.0 - c) * 0.5;
            format!("{} {} {} 0 0", to_fixed(0.3 * m, 3), to_fixed(0.59 * m, 3), to_fixed(0.11 * m, 3))
        })
        .collect();
    format!("{} 0 0 0 1 0", rows.join(" "))
}

pub fn halo_defs() -> impl IntoView {
    let h = HOLO_SIZE;
    view! {
        <defs>
            {FACTIONS
                .map(|f| {
                    let c = faction_color(Some(f));
                    view! {
                        <radialGradient id=format!("halo-{}", f.as_str())>
                            {stop("0%", "#fff", 0.9)}
                            {stop("25%", c, 0.55)}
                            {stop("100%", c, 0.0)}
                        </radialGradient>
                    }
                })
                .collect_view()}
            <radialGradient id="sphere-shade" cx="35%" cy="30%" r="75%">
                {stop("0%", "#fff", 0.55)}
                {stop("35%", "#fff", 0.0)}
                {stop("75%", "#000", 0.3)}
                {stop("100%", "#000", 0.7)}
            </radialGradient>
            <radialGradient id="wormhole">
                {stop("0%", "#05020a", 1.0)}
                {stop("38%", "#1a0828", 1.0)}
                {stop("58%", "#b674d2", 0.95)}
                {stop("72%", "#7d3fb0", 0.45)}
                {stop("100%", "#7d3fb0", 0.0)}
            </radialGradient>
            <radialGradient id="halo-scorched">
                {stop("0%", "#ffd2a1", 0.9)}
                {stop("30%", "#ff7a2e", 0.5)}
                {stop("100%", "#ff7a2e", 0.0)}
            </radialGradient>
            <radialGradient id="fracture-glow">
                {stop("0%", "#ff7a4a", 0.55)}
                {stop("60%", "#c83a2a", 0.12)}
                {stop("100%", "#c83a2a", 0.0)}
            </radialGradient>
            // Penta's black hole, after the game's own.
            <radialGradient id="bh-glow">
                {stop("30%", "#ffb07a", 0.6)}
                {stop("60%", "#ff6a3a", 0.18)}
                {stop("100%", "#ff6a3a", 0.0)}
            </radialGradient>
            <linearGradient id="bh-disk">
                {stop("0%", "#ff5a2e", 0.0)}
                {stop("22%", "#ff7a45", 0.75)}
                {stop("50%", "#fff1e4", 1.0)}
                {stop("78%", "#ff9ac2", 0.75)}
                {stop("100%", "#ff5a8a", 0.0)}
            </linearGradient>
            // Hologram light: the column fades as it rises, the pool on the table fades from its centre.
            <linearGradient id="holo-fade-ramp" x1="0" y1="1" x2="0" y2="0">
                {stop("0%", "#fff", 0.85)}
                {stop("100%", "#fff", 0.05)}
            </linearGradient>
            <mask id="holo-fade" maskContentUnits="objectBoundingBox">
                <rect width="1" height="1" fill="url(#holo-fade-ramp)"></rect>
            </mask>
            <radialGradient id="holo-pool-ramp">
                {stop("0%", "#fff", 0.7)}
                {stop("45%", "#fff", 0.25)}
                {stop("100%", "#fff", 0.0)}
            </radialGradient>
            <mask id="holo-pool" maskContentUnits="objectBoundingBox">
                <rect width="1" height="1" fill="url(#holo-pool-ramp)"></rect>
            </mask>
            // Each hologram recoloured in its light's colour from its own brightness, so off-colour renders match.
            {HOLO_TINT
                .map(|(name, rgb)| {
                    view! {
                        <filter id=format!("holo-tint-{name}") color-interpolation-filters="sRGB">
                            <feColorMatrix type="matrix" values=tint_matrix(rgb)></feColorMatrix>
                        </filter>
                    }
                })
                .collect_view()}
            <pattern id="holo-scan" width=num(h) height="0.9" patternUnits="userSpaceOnUse">
                <rect width=num(h) height="0.35" fill="#fff" fill-opacity="0.28"></rect>
            </pattern>
            <linearGradient id="holo-sweep-band" x1="0" y1="0" x2="0" y2="1">
                {stop("0%", "#fff", 0.0)}
                {stop("50%", "#fff", 0.75)}
                {stop("100%", "#fff", 0.0)}
            </linearGradient>
            // Each hologram's own outline, for its scanlines and sweep; every pawn draws its unit at the same spot.
            {PAWNS
                .iter()
                .filter_map(|key| Some((*key, pawn_href(key)?)))
                .map(|(key, href)| {
                    view! {
                        <mask
                            id=format!("holo-mask-{key}")
                            maskUnits="userSpaceOnUse"
                            x=num(-h / 2.0)
                            y=num(-h)
                            width=num(h)
                            height=num(h)
                            style="mask-type: alpha;"
                        >
                            <image href=href x=num(-h / 2.0) y=num(-h) width=num(h) height=num(h)></image>
                        </mask>
                    }
                })
                .collect_view()}
        </defs>
    }
}

/// Planets the game draws as something other than a whole world: wormhole, black hole, debris, burnt rock.
fn anomaly_mark(kind: Anomaly) -> AnyView {
    match kind {
        Anomaly::Wormhole => view! {
            <g class="anomaly wormhole">
                <circle r="9" fill="url(#wormhole)"></circle>
                <circle r="5.6" class="swirl"></circle>
            </g>
        }
        .into_any(),
        // The disk is drawn twice: its far half behind the shadow, its near half in front.
        Anomaly::Blackhole => view! {
            <g class="anomaly blackhole">
                <circle r="10" fill="url(#bh-glow)"></circle>
                <ellipse rx="10.5" ry="3" fill="url(#bh-disk)"></ellipse>
                <circle r="4.9" class="lens"></circle>
                <circle r="3.9" class="photon"></circle>
                <circle r="3.4" class="horizon"></circle>
                <path d="M-10.5,0A10.5,3 0 0 0 10.5,0Z" fill="url(#bh-disk)"></path>
                <ellipse rx="8.5" ry="2.2" class="bh-flow"></ellipse>
            </g>
        }
        .into_any(),
        // What the Meridian singularity left: a hot core with the planet's crust drifting apart around it.
        _ => view! {
            <g class="anomaly fractured">
                <circle r="6.5" fill="url(#fracture-glow)"></circle>
                <path class="rock" d="M-1.9,-2.6L0.4,-3.3L2.1,-1.9L1.6,0.2L-0.6,0.9L-2.4,-0.6Z"></path>
                <path class="rock" d="M2.6,0.9L4.6,1.3L4.9,3.3L3.1,4.2L1.8,2.7Z"></path>
                <path class="rock" d="M-4.6,1.1L-2.3,1.5L-2.0,3.6L-3.9,4.4L-5.2,2.9Z"></path>
                <path class="rock" d="M3.4,-4.6L4.9,-4.2L4.6,-2.8L3.2,-3.0Z"></path>
                <path class="rock small" d="M-4.6,-3.6L-3.5,-3.9L-3.3,-2.9L-4.4,-2.7Z"></path>
                <circle class="rock small" cx="0.2" cy="4.4" r="0.6"></circle>
                <circle class="ember" r="0.9" cx="0.1" cy="1.6"></circle>
            </g>
        }
        .into_any(),
    }
}

/// Each hologram's light: its colour on the map, and the tint its render is recoloured with.
fn dss_light(state: DssState) -> (&'static str, &'static str) {
    let tint = match state {
        DssState::Active => "cta",
        DssState::Cooldown => "bad",
        DssState::Offline => "off",
        DssState::Default => "humans",
    };
    (state.tone(), tint)
}

/// A unit's light follows its own faction, whoever holds the planet it is on.
fn unit_light(prefix: &str) -> Option<(&'static str, &'static str)> {
    Some(match prefix {
        "bug" => (faction_color(Some(Faction::Terminids)), "bug"),
        "bot" => (faction_color(Some(Faction::Automaton)), "bot"),
        "ill" => (faction_color(Some(Faction::Illuminate)), "ill"),
        "seaf" => (faction_color(Some(Faction::Humans)), "humans"),
        _ => return None,
    })
}

/// Renders that fill more of their frame than the rest, shrunk to match; SEAF's small Pelicans, grown.
fn unit_scale(key: &str) -> f64 {
    match key {
        "seaf" => 1.25,
        "bug_dragonroach" => 0.8,
        "ill_greathost" => 0.85,
        "bot_cyborgs" | "bot_incincorp" | "bot_jetbrigade" => 0.85,
        _ => 1.0,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Light {
    art: Art,
    tone: &'static str,
    tint: &'static str,
    badges: Vec<Art>,
}

/// A unit as a hologram on the war table: a projector lying on the map, a column of light, and the
/// unit floating on it. The projector is always seen at an angle, like the renders; tilting the map
/// flattens it further and lifts the unit higher.
fn holo_pawn(light: &Light, x: f64, back: bool, delay: f64) -> AnyView {
    let s = HOLO_SIZE;
    let art = light.art.clone();
    let style = format!("color: {}; --tint: url(#holo-tint-{}); --delay: {}s;", light.tone, light.tint, num(-delay));
    let n = light.badges.len() as f64;
    let badges = (!light.badges.is_empty()).then(|| {
        let at = format!("translate({},{})", num((-(n - 1.0) * BADGE_STEP) / 2.0), num(-s * 0.85 - BADGE / 2.0 - 1.0));
        // Just over the station (its render leaves the top of the frame empty), outside .holo-unit so they
        // keep their colours and don't bob.
        view! {
            <g class="holo-badges" transform=at>
                {light
                    .badges
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        view! {
                            <image
                                href=b.href.clone()
                                x=num(i as f64 * BADGE_STEP - BADGE / 2.0)
                                y=num(-BADGE / 2.0)
                                width=num(BADGE)
                                height=num(BADGE)
                            ></image>
                        }
                    })
                    .collect_view()}
            </g>
        }
    });
    view! {
        <g transform=format!("translate({},2) scale({})", num(x), num(HOLO_SCALE))>
            <g class=format!("holo {}", if back { "back" } else { "" }) style=style>
                <g class="holo-floor">
                    <circle r="8" class="holo-pool" mask="url(#holo-pool)"></circle>
                    <circle r="5.4" class="holo-ring"></circle>
                    <circle r="4.2" class="holo-ticks"></circle>
                    <circle r="1.3" class="holo-emitter"></circle>
                </g>
                <path class="holo-beam" d="M-1.6,0L-6,-1H6L1.6,0Z" mask="url(#holo-fade)"></path>
                <path class="holo-edge" d="M-1.5,0L-6,-1M1.5,0L6,-1" mask="url(#holo-fade)"></path>
                <g class="holo-lift">
                    {badges}
                    <g class="holo-unit">
                        <g transform=format!("scale({})", num(unit_scale(&art.key)))>
                            <image href=art.href.clone() x=num(-s / 2.0) y=num(-s) width=num(s) height=num(s)></image>
                            <g class="holo-light" mask=format!("url(#holo-mask-{})", art.key)>
                                <rect
                                    x=num(-s / 2.0)
                                    y=num(-s)
                                    width=num(s)
                                    height=num(s)
                                    fill="url(#holo-scan)"
                                ></rect>
                                <rect
                                    class="holo-sweep"
                                    x=num(-s / 2.0)
                                    y="-4"
                                    width=num(s)
                                    height="4"
                                    fill="url(#holo-sweep-band)"
                                ></rect>
                            </g>
                        </g>
                    </g>
                </g>
            </g>
        </g>
    }
    .into_any()
}

/// A Lucide icon placed inside the map's SVG.
fn inner_icon(icon: IconData, size: f64, x: f64, y: f64, stroke_width: f64) -> AnyView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=num(size)
            height=num(size)
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width=num(stroke_width)
            stroke-linecap="round"
            stroke-linejoin="round"
            class=icon.class
            x=num(x)
            y=num(y)
            aria-hidden="true"
            inner_html=icon.body
        ></svg>
    }
    .into_any()
}

/// Threats and points of interest, as tags in a row.
fn pawn_tags(effects: &[PlanetEffect], faction: Faction, x: f64, class: &str) -> AnyView {
    view! {
        <g class=format!("pawns {class}") transform=format!("translate({},1)", num(x))>
            {effects
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let tone = match e.kind {
                        EffectKind::Subfaction => faction_color(Some(faction)),
                        EffectKind::Seaf => "var(--f-humans)",
                        _ => "var(--cta)",
                    };
                    let mark = match e.kind {
                        EffectKind::Subfaction => {
                            let art = subfaction_art(&e.name).unwrap_or(faction.as_str());
                            view! { <EmblemArt art=art size=4.4 /> }.into_any()
                        }
                        kind => {
                            let icon = match kind {
                                EffectKind::Seaf => icons::SHIELD,
                                EffectKind::Poi => icons::LANDMARK,
                                _ => icons::TRIANGLE_ALERT,
                            };
                            inner_icon(icon, 4.0, -2.0, -2.0, 2.6).into_any()
                        }
                    };
                    view! {
                        <g
                            class=format!("pawn {}", e.kind.as_str())
                            transform=format!("translate({},0)", num(i as f64 * 6.4))
                            style=format!("color: {tone};")
                        >
                            <rect x="-2.8" y="-2.8" width="5.6" height="5.6"></rect>
                            {mark}
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
    .into_any()
}

/// The planet's sub-factions, SEAF presence and the Democracy Space Station as holograms beside it,
/// as the game's war table shows them; threats and points of interest, which have none, follow as tags.
fn pawns(all: &[PlanetEffect], faction: Faction, dss: Option<&DssInfo>, seed: i64, enemy: bool) -> AnyView {
    // The game keeps an enemy unit's effect on a planet after it is liberated; Super Earth's planets show none.
    // Points of interest wear their own logo over the planet instead of a tag.
    let effects: Vec<PlanetEffect> = all
        .iter()
        .filter(|e| e.kind != EffectKind::Poi && (enemy || e.kind != EffectKind::Subfaction))
        .cloned()
        .collect();
    let mut holos: Vec<Light> = Vec::new();
    if let Some(d) = dss
        && let Some(art) = crate::features::map::pawn_art::dss_art(d.state)
    {
        let (tone, tint) = dss_light(d.state);
        holos.push(Light { art, tone, tint, badges: d.active.clone() });
    }
    let mut tags: Vec<PlanetEffect> = Vec::new();
    for e in &effects {
        let art = pawn_art(e);
        match art.as_ref().and_then(|a| unit_light(a.key.split('_').next().unwrap_or(""))) {
            Some((tone, tint)) => holos.push(Light { art: art.unwrap(), tone, tint, badges: Vec::new() }),
            None => tags.push(e.clone()),
        }
    }
    let placed: Vec<(Light, f64, bool, f64)> = holos
        .iter()
        .enumerate()
        .map(|(i, light)| {
            let i = i as f64;
            (light.clone(), HOLO_X + i * HOLO_STEP, i as usize % 2 == 1, (seed as f64 * 0.73 + i * 1.3) % 6.0)
        })
        .collect();
    let rest_x = if holos.is_empty() { 7.0 } else { HOLO_X + holos.len() as f64 * HOLO_STEP + 2.0 };
    // Holograms only read once zoomed in; until then every effect shows as a tag.
    view! {
        {(!effects.is_empty()).then(|| pawn_tags(&effects, faction, 7.0, "pawns-all"))}
        {(!holos.is_empty()).then(|| {
            // The back rank first, so the front one stands before it.
            let ordered = placed.iter().filter(|h| h.2).chain(placed.iter().filter(|h| !h.2));
            view! { <g class="holos">{ordered.map(|(l, x, back, delay)| holo_pawn(l, *x, *back, *delay)).collect_view()}</g> }
        })}
        {(!tags.is_empty()).then(|| pawn_tags(&tags, faction, rest_x, "pawns-rest"))}
    }
    .into_any()
}

/// Biome colour pulled towards grey, so the spheres read as worlds rather than markers.
pub fn sphere_color(hex: &str) -> String {
    let grey = "8a93a0";
    let mix = |i: usize| {
        let a = u8::from_str_radix(hex.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0) as f64;
        let b = u8::from_str_radix(&grey[i - 1..i + 1], 16).unwrap_or(0) as f64;
        format!("{:02x}", js_round(a * 0.55 + b * 0.45) as u32)
    };
    format!("#{}{}{}", mix(1), mix(3), mix(5))
}

/// Square tag over an active planet in the fighting faction's colour: crosshair to liberate, shield to defend.
fn active_tag(defense: bool, mo: bool, color: &str) -> AnyView {
    view! {
        <g class=format!("act-tag {}", if mo { "mo" } else { "" }) transform="translate(0,-10.5)" style=format!("color: {color};")>
            <path class="act-stem" d="M0,3.4V5.6"></path>
            <rect x="-3.4" y="-3.4" width="6.8" height="6.8"></rect>
            {if defense {
                view! { <path d="M0,-2.2 2,-1.4V0.2C2,1.3 1.1,2 0,2.4-1.1,2-2,1.3-2,0.2V-1.4Z"></path> }.into_any()
            } else {
                view! {
                    <circle r="1.3"></circle>
                    <path d="M-2.3,0H-1.3M1.3,0H2.3M0,-2.3V-1.3M0,1.3V2.3"></path>
                }
                .into_any()
            }}
        </g>
    }
    .into_any()
}

/// The planet's points of interest as the game's logos, side by side over the planet.
fn poi_marks(kinds: &[PoiKind], faction: Faction) -> AnyView {
    let n = kinds.len() as f64;
    view! {
        <g class="poi-marks" transform="translate(0,-10)">
            {kinds
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let x = (i as f64 - (n - 1.0) / 2.0) * (POI_SIZE + 1.5);
                    view! {
                        <g transform=format!("translate({},0)", num(x))>
                            <PoiGlyph kind=*k faction=poi_origin(*k, Some(faction)) size=POI_SIZE />
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
    .into_any()
}

fn pin_mark() -> AnyView {
    view! {
        <g class="pin-mark" transform="translate(7,-7)">
            <path d="M0,4.5C0,4.5 -3.4,1.2 -3.4,-1.1A3.4,3.4 0 0 1 3.4,-1.1C3.4,1.2 0,4.5 0,4.5Z"></path>
            <circle cy="-1.1" r="1.2"></circle>
        </g>
    }
    .into_any()
}

/// Everything a planet's marker is drawn from.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub style: MapPlanets,
    pub faction: Faction,
    pub color: String,
    pub fill: String,
    pub size: f64,
    pub anomaly: Option<Anomaly>,
    pub active: bool,
    pub defense: bool,
    pub mo: bool,
    pub enemy: bool,
    pub pinned: bool,
    pub pawns: Vec<PlanetEffect>,
    pub dss: Option<DssInfo>,
    pub seed: i64,
    pub pois: Vec<PoiKind>,
}

pub fn planet_body(b: &Body) -> AnyView {
    // Razed worlds keep a planet's size and shape; only their colour and glow turn to embers.
    let scorched = b.anomaly == Some(Anomaly::Scorched);
    let shape = match b.anomaly {
        Some(a) if a != Anomaly::Scorched => anomaly_mark(a),
        _ if b.style == MapPlanets::Spheres => {
            let r = num(4.4 * b.size);
            view! {
                <circle
                    r=r.clone()
                    class=format!("sphere {}", if scorched { "scorched" } else { "" })
                    fill=if scorched { "#8a3a16".to_string() } else { b.fill.clone() }
                ></circle>
                <circle r=r fill="url(#sphere-shade)" class="shade"></circle>
                {b.enemy.then(|| view! { <circle r="5.6" class="owner-ring" stroke=b.color.clone()></circle> })}
                {b.mo.then(|| view! { <circle r="6.8" class="mo-ring"></circle> })}
            }
            .into_any()
        }
        _ => view! {
            <circle
                r=if b.active { "9" } else { "6" }
                fill=if scorched { "url(#halo-scorched)".to_string() } else { format!("url(#halo-{})", b.faction.as_str()) }
                class="halo"
            ></circle>
            {b.active.then(|| view! { <circle r="7" class="pulse" stroke=b.color.clone()></circle> })}
            {b.mo.then(|| view! { <circle r="5" class="mo-ring"></circle> })}
            <circle
                r=num((if b.active { 2.2 } else { 1.5 }) * b.size)
                class=format!("core {}", if scorched { "scorched" } else { "" })
                fill=if scorched { "#ff7a2e".to_string() } else { b.color.clone() }
            ></circle>
        }
        .into_any(),
    };
    view! {
        <g class="pbody">
            <circle r="10" class="hit"></circle>
            {shape}
            {(b.style == MapPlanets::Spheres && b.active).then(|| active_tag(b.defense, b.mo, &b.color))}
            {b.pinned.then(pin_mark)}
            {(!b.pois.is_empty()).then(|| poi_marks(&b.pois, b.faction))}
            {(b.dss.is_some() || !b.pawns.is_empty()).then(|| pawns(&b.pawns, b.faction, b.dss.as_ref(), b.seed, b.enemy))}
        </g>
    }
    .into_any()
}
