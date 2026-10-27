//! Galaxy geometry: map projection, the game's sector rings, sector territory, supply lines and anomalies.

use crate::api::{Faction, Planet, Position};
use crate::format::{js_round, to_fixed};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

/// Map units: the API's [-1, 1] galaxy coordinates scaled up, y flipped so north is up.
pub const GALAXY_SCALE: f64 = 500.0;

pub type Pt = [f64; 2];

pub fn project(p: Position) -> Pt {
    [p.x * GALAXY_SCALE, -p.y * GALAXY_SCALE]
}

pub struct Poi {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct Landmark {
    pub name: &'static str,
    pub sector: &'static str,
    pub note: &'static str,
    pub biome: &'static str,
    pub position: Position,
    pub pois: &'static [Poi],
}

/// Worlds the game's galaxy map shows but the war API leaves out; positions measured off the map.
pub const LANDMARKS: &[Landmark] = &[Landmark {
    name: "Mars",
    sector: "Sol",
    note: "Glassed by the Illuminate. No war data is published for Mars.",
    biome: "Scorched",
    position: Position { x: 0.043, y: 0.035 },
    pois: &[
        Poi {
            name: "Helldiver Training Facility",
            description: "Where recruits earned their capes until the Illuminate razed Mars in May 2185. Training has moved to Zegema Paradise.",
        },
        Poi {
            name: "Martian Armory",
            description: "Arms maker of the MA5C assault rifle, M7S SMG, M90A shotgun and M6C/SOCOM pistol. Named for Mars; the game never says its works stood there.",
        },
    ],
}];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Anomaly {
    Wormhole,
    Blackhole,
    Fractured,
    Scorched,
}

impl Anomaly {
    pub fn as_str(self) -> &'static str {
        match self {
            Anomaly::Wormhole => "wormhole",
            Anomaly::Blackhole => "blackhole",
            Anomaly::Fractured => "fractured",
            Anomaly::Scorched => "scorched",
        }
    }
}

/// Planets the game draws as space anomalies instead of worlds.
fn anomaly_by_name(name: &str) -> Option<Anomaly> {
    match name {
        "MERIDIA" => Some(Anomaly::Wormhole),
        "PENTA" => Some(Anomaly::Blackhole),
        "ANGEL'S VENTURE" | "MORADESH" | "IVIS" => Some(Anomaly::Fractured),
        _ => None,
    }
}

/// Galactic effects that change how the game draws a planet.
fn look_effect(id: i64) -> Option<Anomaly> {
    match id {
        1229 => Some(Anomaly::Wormhole),
        1177 | 1352 => Some(Anomaly::Blackhole),
        1241 | 1252 => Some(Anomaly::Fractured),
        _ => None,
    }
}

/// Planets the game draws burnt. Nothing in either API says so; the list follows the wiki's war history.
/// Re-seeding has begun on the Illuminate's targets, so drop a name once the game draws it whole again.
pub fn scorched_note(name: &str) -> Option<&'static str> {
    match name {
        "WIDOW'S HARBOR" | "PILEN V" | "NEW HAVEN" => {
            Some("Razed by the Illuminate as they fled Super Earth, May 2025.")
        }
        "MERAK" | "LESATH" | "CHORT BAY" => {
            Some("Razed by the DSS on the Helldivers' vote, on the way to Cyberstan, 2026.")
        }
        _ => None,
    }
}

/// How the game draws a planet when it isn't a plain world, from its effects, then by name.
pub fn anomaly_of(name: &str, effect_ids: &[i64]) -> Option<Anomaly> {
    if let Some(a) = effect_ids.iter().find_map(|id| look_effect(*id)) {
        return Some(a);
    }
    let name = name.to_uppercase();
    anomaly_by_name(&name).or_else(|| scorched_note(&name).map(|_| Anomaly::Scorched))
}

// ---------- Sectors ----------

/// The game lays sectors out on concentric rings around Super Earth, each sector an unbroken
/// run of angle within a ring. Ring width and offset are fitted to the planets' distances
/// from the centre; inside one ring, planets of a sector sit next to each other by angle.
pub const RING_WIDTH: f64 = 0.098;
const RING_OFFSET: f64 = 0.0035;
pub const RINGS: usize = 10;

/// Ring boundary radius `i`, galaxy units, from the core's edge out to the rim.
pub fn ring_radius(i: usize) -> f64 {
    RING_OFFSET + (i + 1) as f64 * RING_WIDTH
}

#[derive(Clone, Debug, PartialEq)]
pub struct SectorRun {
    pub sector: String,
    /// Radians, counter-clockwise from +x; a1 > a0 and a1 - a0 <= 2π.
    pub a0: f64,
    pub a1: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolarRing {
    /// Galaxy units.
    pub r0: f64,
    pub r1: f64,
    /// Covers the full circle, in angle order.
    pub runs: Vec<SectorRun>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolarEdge {
    pub d: String,
    pub a: String,
    /// None on the rim.
    pub b: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolarSectors {
    pub rings: Vec<PolarRing>,
    /// Union of each sector's runs, map units, in the order sectors first appear.
    pub fills: Vec<(String, String)>,
    /// Boundaries between neighbouring runs of different sectors, map units.
    pub edges: Vec<PolarEdge>,
    /// Disc radius, map units.
    pub rim: f64,
}

const TAU: f64 = PI * 2.0;

fn norm(a: f64) -> f64 {
    ((a % TAU) + TAU) % TAU
}

/// Point at radius r (galaxy units) and angle a (radians, counter-clockwise from +x), in map units.
pub fn polar(r: f64, a: f64) -> Pt {
    [r * a.cos() * GALAXY_SCALE, -r * a.sin() * GALAXY_SCALE]
}

fn fmt(p: Pt) -> String {
    format!("{},{}", to_fixed(p[0], 1), to_fixed(p[1], 1))
}

/// SVG arc command from the current point (at a0) to angle a1; sweep 0 runs counter-clockwise on screen.
pub fn arc_to(r: f64, a0: f64, a1: f64, sweep: u8) -> String {
    let big = to_fixed(r * GALAXY_SCALE, 1);
    format!("A{big},{big} 0 {} {sweep} {}", u8::from((a1 - a0).abs() > PI), fmt(polar(r, a1)))
}

fn wedge(r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    // A full ring is drawn as two halves: an annulus's inner circle would cut a hole in the same sector's inner runs.
    if a1 - a0 >= TAU - 1e-9 {
        return wedge(r0, r1, a0, a0 + PI) + &wedge(r0, r1, a0 + PI, a1);
    }
    let outer = format!("M{}{}", fmt(polar(r1, a0)), arc_to(r1, a0, a1, 0));
    if r0 == 0.0 {
        format!("{outer}L0,0Z")
    } else {
        format!("{outer}L{}{}Z", fmt(polar(r0, a1)), arc_to(r0, a1, a0, 1))
    }
}

fn run_at(ring: &PolarRing, a: f64) -> &str {
    let start = ring.runs[0].a0;
    let x = start + norm(a - start);
    &ring.runs.iter().find(|r| x < r.a1).unwrap_or(ring.runs.last().unwrap()).sector
}

/// Sector drawn at a galaxy point, or None outside the disc.
pub fn sector_at(geo: &PolarSectors, x: f64, y: f64) -> Option<&str> {
    let r = x.hypot(y);
    let ring = geo.rings.iter().find(|g| r < g.r1)?;
    Some(run_at(ring, y.atan2(x)))
}

struct Pin<'a> {
    s: &'a str,
    a: f64,
    ring: usize,
}

#[derive(Clone)]
struct Group<'a> {
    s: &'a str,
    /// Angles of the run's first and last planet; `first` may be below 0 for the run that wraps past 0.
    first: f64,
    last: f64,
}

/// A ring's planets in angle order, consecutive planets of one sector grouped; None for an empty ring.
fn ring_groups<'a>(pins: &[Pin<'a>], i: usize) -> Option<Vec<Group<'a>>> {
    let mut in_ring: Vec<&Pin> = pins.iter().filter(|p| p.ring == i).collect();
    if in_ring.is_empty() {
        return None;
    }
    in_ring.sort_by(|x, y| x.a.total_cmp(&y.a));
    let mut groups: Vec<Group> = Vec::new();
    for p in in_ring {
        match groups.last_mut() {
            Some(g) if g.s == p.s => g.last = p.a,
            _ => groups.push(Group { s: p.s, first: p.a, last: p.a }),
        }
    }
    if groups.len() > 1 && groups[0].s == groups[groups.len() - 1].s {
        let tail = groups.pop().unwrap();
        groups[0].first = tail.first - TAU;
    }
    Some(groups)
}

/// Ring a galaxy radius falls in, from 1 (just outside the core) to the outermost.
fn ring_index(r: f64) -> usize {
    match (0..RINGS).find(|&i| r < ring_radius(i)) {
        None => RINGS - 1,
        Some(i) => i.max(1),
    }
}

struct Pair {
    rf: f64,
    last: f64,
    first: f64,
}

fn bound(groups: &[Group], k: usize) -> (String, f64, f64) {
    let prev = &groups[(k + groups.len() - 1) % groups.len()];
    let last = if k == 0 { prev.last - TAU } else { prev.last };
    (format!("{}|{}", prev.s, groups[k].s), last, groups[k].first)
}

fn shift_to(rf: f64, a: f64) -> f64 {
    js_round((rf - a) / TAU) * TAU
}

pub struct SectorPlanet<'a> {
    pub index: i64,
    pub sector: &'a str,
    pub position: Position,
}

/// Sector shapes from planet positions alone, as the game doesn't publish them. Where two sectors meet
/// in several rings they share one boundary angle wherever their planets allow, so sectors come out as
/// the game's stepped blocks rather than a zigzag that shifts from ring to ring.
pub fn build_polar_sectors(planets: &[SectorPlanet]) -> PolarSectors {
    let home = planets.iter().find(|p| p.index == 0).map_or("Sol", |p| p.sector);
    let mut rings = vec![PolarRing {
        r0: 0.0,
        r1: ring_radius(0),
        runs: vec![SectorRun { sector: home.into(), a0: 0.0, a1: TAU }],
    }];
    let pins: Vec<Pin> = planets
        .iter()
        .filter(|p| p.index != 0)
        .map(|p| Pin {
            s: p.sector,
            a: norm(p.position.y.atan2(p.position.x)),
            ring: ring_index(p.position.x.hypot(p.position.y)),
        })
        .collect();
    let per_ring: Vec<Option<Vec<Group>>> =
        (0..RINGS).map(|i| if i == 0 { None } else { ring_groups(&pins, i) }).collect();

    // Each pair's gap between its planets, over every ring where the two meet, in the first ring's angle frame.
    let mut pairs: HashMap<String, Pair> = HashMap::new();
    for groups in per_ring.iter().flatten() {
        if groups.len() < 2 {
            continue;
        }
        for k in 0..groups.len() {
            let (key, last, first) = bound(groups, k);
            match pairs.get_mut(&key) {
                None => {
                    pairs.insert(key, Pair { rf: (last + first) / 2.0, last, first });
                }
                Some(p) => {
                    let shift = shift_to(p.rf, (last + first) / 2.0);
                    p.last = p.last.max(last + shift);
                    p.first = p.first.min(first + shift);
                }
            }
        }
    }

    for i in 1..RINGS {
        let (r0, r1) = (ring_radius(i - 1), ring_radius(i));
        let runs = match &per_ring[i] {
            None => rings[i - 1].runs.clone(),
            Some(groups) if groups.len() == 1 => vec![SectorRun { sector: groups[0].s.into(), a0: 0.0, a1: TAU }],
            Some(groups) => {
                let cuts: Vec<f64> = (0..groups.len())
                    .map(|k| {
                        let (key, last, first) = bound(groups, k);
                        let p = &pairs[&key];
                        if p.last >= p.first {
                            (last + first) / 2.0
                        } else {
                            (p.last + p.first) / 2.0 - shift_to(p.rf, (last + first) / 2.0)
                        }
                    })
                    .collect();
                groups
                    .iter()
                    .enumerate()
                    .map(|(k, g)| SectorRun {
                        sector: g.s.into(),
                        a0: cuts[k],
                        a1: if k + 1 < groups.len() { cuts[k + 1] } else { cuts[0] + TAU },
                    })
                    .collect()
            }
        };
        rings.push(PolarRing { r0, r1, runs });
    }

    let mut fills: Vec<(String, String)> = Vec::new();
    let mut fill_at: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<PolarEdge> = Vec::new();
    for ring in &rings {
        let n = ring.runs.len();
        for (k, run) in ring.runs.iter().enumerate() {
            let w = wedge(ring.r0, ring.r1, run.a0, run.a1);
            match fill_at.get(&run.sector) {
                Some(&i) => fills[i].1.push_str(&w),
                None => {
                    fill_at.insert(run.sector.clone(), fills.len());
                    fills.push((run.sector.clone(), w));
                }
            }
            let prev = &ring.runs[(k + n - 1) % n];
            if n > 1 && prev.sector != run.sector {
                edges.push(PolarEdge {
                    d: format!("M{}L{}", fmt(polar(ring.r0, run.a0)), fmt(polar(ring.r1, run.a0))),
                    a: prev.sector.clone(),
                    b: Some(run.sector.clone()),
                });
            }
        }
    }

    // Arcs between each ring and the next (or the rim): split at both rings' run edges, keep spans whose sides differ.
    for (i, inner) in rings.iter().enumerate() {
        let outer = rings.get(i + 1);
        let r = inner.r1;
        let mut seen = HashSet::new();
        let mut cuts: Vec<f64> = inner
            .runs
            .iter()
            .chain(outer.map(|o| o.runs.iter()).into_iter().flatten())
            .map(|run| to_fixed(norm(run.a0), 9).parse::<f64>().unwrap())
            .chain([0.0])
            .filter(|c| seen.insert(c.to_bits()))
            .collect();
        cuts.sort_by(f64::total_cmp);
        cuts.push(TAU);
        struct Open<'a> {
            a: &'a str,
            b: Option<&'a str>,
            from: f64,
            to: f64,
        }
        let mut open: Option<Open> = None;
        let flush = |open: &mut Option<Open>, edges: &mut Vec<PolarEdge>| {
            if let Some(o) = open.take() {
                edges.push(PolarEdge {
                    d: format!("M{}{}", fmt(polar(r, o.from)), arc_to(r, o.from, o.to, 0)),
                    a: o.a.into(),
                    b: o.b.map(Into::into),
                });
            }
        };
        for c in 0..cuts.len() - 1 {
            let mid = (cuts[c] + cuts[c + 1]) / 2.0;
            let a = run_at(inner, mid);
            let b = outer.map(|o| run_at(o, mid));
            if Some(a) == b {
                flush(&mut open, &mut edges);
            } else if let Some(o) = open.as_mut().filter(|o| o.a == a && o.b == b) {
                o.to = cuts[c + 1];
            } else {
                flush(&mut open, &mut edges);
                open = Some(Open { a, b, from: cuts[c], to: cuts[c + 1] });
            }
        }
        flush(&mut open, &mut edges);
    }

    PolarSectors { rings, fills, edges, rim: ring_radius(RINGS - 1) * GALAXY_SCALE }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Territory {
    pub faction: Faction,
    /// Every planet of the sector is held by this enemy; otherwise the sector is contested.
    pub held: bool,
}

/// Counts in insertion order, then the key with the highest count; ties go to the first counted.
pub fn majority<K: PartialEq + Copy>(items: impl IntoIterator<Item = K>) -> Option<K> {
    let mut counts: Vec<(K, usize)> = Vec::new();
    for k in items {
        match counts.iter_mut().find(|(c, _)| *c == k) {
            Some((_, n)) => *n += 1,
            None => counts.push((k, 1)),
        }
    }
    let best = counts.iter().map(|c| c.1).max()?;
    counts.into_iter().find(|c| c.1 == best).map(|c| c.0)
}

/// Groups items by key, keeping the order keys first appear in.
pub fn group_by<T, K: PartialEq + Clone>(items: &[T], key: impl Fn(&T) -> K) -> Vec<(K, Vec<&T>)> {
    let mut out: Vec<(K, Vec<&T>)> = Vec::new();
    for item in items {
        let k = key(item);
        match out.iter_mut().find(|(g, _)| *g == k) {
            Some((_, list)) => list.push(item),
            None => out.push((k, vec![item])),
        }
    }
    out
}

/// The enemy with most planets (held or under attack) in each sector; None where Super Earth holds them all.
pub fn sector_territory(planets: &[Planet]) -> Vec<(String, Option<Territory>)> {
    group_by(planets, |p| p.sector.clone())
        .into_iter()
        .map(|(s, list)| {
            let enemies = list.iter().filter_map(|p| {
                let enemy = if p.current_owner != Faction::Humans {
                    Some(p.current_owner)
                } else {
                    p.event.as_ref().map(|e| e.faction)
                };
                enemy.filter(|e| *e != Faction::Humans)
            });
            let territory = majority(enemies)
                .map(|faction| Territory { faction, held: list.iter().all(|p| p.current_owner == faction) });
            (s, territory)
        })
        .collect()
}

/// Unique undirected supply-line pairs from planet waypoints.
pub fn supply_lines(planets: &[Planet]) -> Vec<(i64, i64)> {
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for p in planets {
        for &w in &p.waypoints {
            let k = if p.index < w { (p.index, w) } else { (w, p.index) };
            if seen.insert(k) {
                lines.push((p.index, w));
            }
        }
    }
    lines
}

/// Share of planets (by count) currently held by Super Earth, 0-100.
pub fn human_control(planets: &[Planet]) -> f64 {
    if planets.is_empty() {
        return 0.0;
    }
    planets.iter().filter(|p| p.current_owner == Faction::Humans).count() as f64 / planets.len() as f64 * 100.0
}
