use crate::api::{Faction, Planet};
use crate::galaxy::{GALAXY_SCALE, PolarSectors, Pt, polar, project, sector_at, sector_territory};
use crate::voronoi::Delaunay;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactionMark {
    pub faction: Faction,
    /// Map units.
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Regions,
    Sectors,
    Galaxy,
}

/// Groups of items joined, directly or through others, by `linked`.
fn components<T: Clone>(items: &[T], linked: impl Fn(&T, &T) -> bool) -> Vec<Vec<T>> {
    let mut seen = vec![false; items.len()];
    let mut out = Vec::new();
    for start in 0..items.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut group = vec![start];
        let mut k = 0;
        while k < group.len() {
            for j in 0..items.len() {
                if !seen[j] && linked(&items[group[k]], &items[j]) {
                    seen[j] = true;
                    group.push(j);
                }
            }
            k += 1;
        }
        out.push(group.into_iter().map(|i| items[i].clone()).collect());
    }
    out
}

fn mean(pts: &[Pt]) -> Pt {
    let n = pts.len() as f64;
    [pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n]
}

struct Cell<'a> {
    sector: &'a str,
    at: Pt,
    area: f64,
}

/// Sectors overlay: the enemy's largest run of neighbouring sectors, by area. The mark goes at its
/// area-weighted centre, or at the middle of its biggest cell where that centre falls outside.
fn sector_marks(planets: &[Planet], geo: &PolarSectors) -> Vec<FactionMark> {
    let territory = sector_territory(planets);
    let cells: Vec<Cell> = geo
        .rings
        .iter()
        .flat_map(|ring| {
            ring.runs.iter().map(move |run| {
                let r = (ring.r0 + ring.r1) / 2.0;
                let a = (run.a0 + run.a1) / 2.0;
                Cell {
                    sector: run.sector.as_str(),
                    at: polar(r, a),
                    area: (ring.r1.powi(2) - ring.r0.powi(2)) * (run.a1 - run.a0),
                }
            })
        })
        .collect();
    let touching: HashSet<(&str, &str)> = geo
        .edges
        .iter()
        .filter_map(|e| e.b.as_deref().map(|b| (e.a.as_str(), b)))
        .flat_map(|(a, b)| [(a, b), (b, a)])
        .collect();
    Faction::ENEMIES
        .into_iter()
        .filter_map(|faction| {
            let sectors: Vec<&str> = territory
                .iter()
                .filter(|(_, t)| t.is_some_and(|t| t.faction == faction))
                .map(|(s, _)| s.as_str())
                .collect();
            if sectors.is_empty() {
                return None;
            }
            let blobs: Vec<(Vec<&str>, Vec<&Cell>, f64)> = components(&sectors, |a, b| touching.contains(&(*a, *b)))
                .into_iter()
                .map(|group| {
                    let own: Vec<&Cell> = cells.iter().filter(|c| group.contains(&c.sector)).collect();
                    let area = own.iter().map(|c| c.area).sum();
                    (group, own, area)
                })
                .collect();
            let mut blob = &blobs[0];
            for b in &blobs[1..] {
                if b.2 > blob.2 {
                    blob = b;
                }
            }
            let (group, own, area) = blob;
            let x = own.iter().map(|c| c.at[0] * c.area).sum::<f64>() / area;
            let y = own.iter().map(|c| c.at[1] * c.area).sum::<f64>() / area;
            let inside = sector_at(geo, x / GALAXY_SCALE, -y / GALAXY_SCALE).is_some_and(|s| group.contains(&s));
            let (fx, fy) = if inside {
                (x, y)
            } else {
                let mut big = own[0];
                for c in &own[1..] {
                    if c.area > big.area {
                        big = c;
                    }
                }
                (big.at[0], big.at[1])
            };
            Some(FactionMark { faction, x: fx, y: fy })
        })
        .collect()
}

/// Regions and galaxy overlays: the enemy's largest group of planets, joined as that overlay joins them
/// (neighbouring territory cells, or supply lines). The mark goes at the group's centre, moved onto its
/// nearest member planet where the centre lies in someone else's territory.
fn planet_marks(planets: &[Planet], overlay: Overlay) -> Vec<FactionMark> {
    let pts: Vec<Pt> = planets.iter().map(|p| project(p.position)).collect();
    let delaunay = Delaunay::from(&pts);
    let at: HashMap<i64, usize> = planets.iter().enumerate().map(|(i, p)| (p.index, i)).collect();
    let linked = |a: &&Planet, b: &&Planet| -> bool {
        match overlay {
            Overlay::Regions => delaunay.neighbors(at[&a.index]).contains(&at[&b.index]),
            _ => a.waypoints.contains(&b.index) || b.waypoints.contains(&a.index),
        }
    };
    Faction::ENEMIES
        .into_iter()
        .filter_map(|faction| {
            let held: Vec<&Planet> = planets.iter().filter(|p| p.current_owner == faction).collect();
            if held.is_empty() {
                return None;
            }
            let groups = components(&held, linked);
            let mut blob = &groups[0];
            for g in &groups[1..] {
                if g.len() > blob.len() {
                    blob = g;
                }
            }
            let centre = mean(&blob.iter().map(|p| project(p.position)).collect::<Vec<_>>());
            let nearest = delaunay.find(centre[0], centre[1], 0).map(|i| &planets[i]);
            if nearest.is_some_and(|n| blob.iter().any(|p| std::ptr::eq(*p, n))) {
                return Some(FactionMark { faction, x: centre[0], y: centre[1] });
            }
            let dist = |p: &Planet| {
                let q = project(p.position);
                (q[0] - centre[0]).hypot(q[1] - centre[1])
            };
            let mut best = blob[0];
            for p in &blob[1..] {
                if dist(p) < dist(best) {
                    best = p;
                }
            }
            let [x, y] = project(best.position);
            Some(FactionMark { faction, x, y })
        })
        .collect()
}

/// Where each enemy's emblem goes: on its largest connected holding, as the chosen overlay draws it.
pub fn faction_marks(planets: &[Planet], overlay: Overlay, geo: Option<&PolarSectors>) -> Vec<FactionMark> {
    match (overlay, geo) {
        (Overlay::Sectors, Some(geo)) => sector_marks(planets, geo),
        _ => planet_marks(planets, overlay),
    }
}
