mod common;
use common::*;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use warroom_core::api::Faction;
use warroom_core::galaxy::{
    Anomaly, LANDMARKS, PolarSectors, SectorPlanet, anomaly_of, build_polar_sectors, human_control, landmark,
    landmark_planet, sector_at, sector_territory, supply_lines,
};
use warroom_core::planets::is_placeholder;

fn sector_planets(planets: &[warroom_core::api::Planet]) -> Vec<SectorPlanet<'_>> {
    planets
        .iter()
        .filter(|p| !is_placeholder(p.index, p.position))
        .map(|p| SectorPlanet { index: p.index, sector: &p.sector, position: p.position })
        .collect()
}

#[test]
fn rebuilds_the_games_ring_sectors_with_every_planet_inside_its_own_sector() {
    let planets = planets();
    let geo = build_polar_sectors(&sector_planets(&planets));
    assert_eq!(geo.rings[0].runs.iter().map(|r| r.sector.as_str()).collect::<Vec<_>>(), ["Sol"]);
    for ring in &geo.rings {
        assert!(ring.runs.iter().all(|r| geo.fills.iter().any(|(s, _)| *s == r.sector)));
    }
    for p in planets.iter().filter(|q| !is_placeholder(q.index, q.position)) {
        assert_eq!(sector_at(&geo, p.position.x, p.position.y), Some(p.sector.as_str()), "{}", p.name);
    }
    for (_, d) in &geo.fills {
        assert!(d.starts_with('M') && d.ends_with('Z') && !d.contains("NaN"));
    }
    // Edges only separate different sectors; the rim counts as one side.
    assert!(geo.edges.iter().all(|e| Some(&e.a) != e.b.as_ref()));
    assert!(geo.edges.iter().any(|e| e.b.is_none()));
}

#[test]
fn gives_a_sector_to_the_enemy_holding_or_attacking_most_of_its_planets() {
    let planets = planets();
    let territory = sector_territory(&planets);
    let held = planets.iter().find(|p| p.current_owner != Faction::Humans).unwrap();
    let of = |s: &str| territory.iter().find(|(n, _)| n == s).map(|(_, t)| *t);
    assert!(of(&held.sector).unwrap().is_some());
    assert_eq!(of("Sol"), Some(None));
}

#[test]
fn dedupes_supply_lines_and_computes_control_share() {
    let planets = planets();
    let lines = supply_lines(&planets);
    let keys: std::collections::HashSet<_> = lines.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
    assert_eq!(keys.len(), lines.len());
    assert!(human_control(&planets) > 0.0);
}

#[test]
fn marks_meridia_and_penta_as_anomalies() {
    assert_eq!(anomaly_of("MERIDIA", &[]), Some(Anomaly::Wormhole));
    assert_eq!(anomaly_of("Penta", &[]), Some(Anomaly::Blackhole));
    assert_eq!(anomaly_of("FURY", &[]), None);
    assert_eq!(anomaly_of("PILEN V", &[]), Some(Anomaly::Scorched));
    assert_eq!(anomaly_of("NEW HAVEN", &[]), None);
}

#[test]
fn names_a_sector_held_only_when_the_enemy_holds_every_planet_in_it() {
    let planets = planets();
    let territory = sector_territory(&planets);
    for (sector, t) in &territory {
        let Some(t) = t else { continue };
        let all = planets.iter().filter(|p| &p.sector == sector).all(|p| p.current_owner == t.faction);
        assert_eq!(t.held, all);
    }
    assert!(territory.iter().any(|(_, t)| t.is_some_and(|t| !t.held)));
}

/// Sectors whose cells fall apart into more than one piece.
fn split_sectors(geo: &PolarSectors) -> Vec<String> {
    let tau = PI * 2.0;
    struct Cell<'a> {
        i: usize,
        k: usize,
        sector: &'a str,
        a0: f64,
        a1: f64,
    }
    let cells: Vec<Cell> = geo
        .rings
        .iter()
        .enumerate()
        .flat_map(|(i, ring)| {
            ring.runs.iter().enumerate().map(move |(k, r)| Cell { i, k, sector: &r.sector, a0: r.a0, a1: r.a1 })
        })
        .collect();
    let overlap = |a: &Cell, b: &Cell| [-tau, 0.0, tau].iter().any(|s| a.a1.min(b.a1 + s) - a.a0.max(b.a0 + s) > 1e-6);
    let mut sectors: Vec<&str> = Vec::new();
    for c in &cells {
        if !sectors.contains(&c.sector) {
            sectors.push(c.sector);
        }
    }
    sectors
        .into_iter()
        .filter(|sector| {
            let own: Vec<&Cell> = cells.iter().filter(|c| c.sector == *sector).collect();
            let mut seen: Vec<usize> = vec![0];
            let mut n = 0;
            while n < seen.len() {
                let c = own[seen[n]];
                for (j, d) in own.iter().enumerate() {
                    let touches = (d.i.abs_diff(c.i) == 1 && overlap(c, d)) || (d.i == c.i && d.k.abs_diff(c.k) == 1);
                    if touches && !seen.contains(&j) {
                        seen.push(j);
                    }
                }
                n += 1;
            }
            seen.len() < own.len()
        })
        .map(String::from)
        .collect()
}

#[test]
fn leaves_no_sector_in_pieces_once_corrected() {
    let raw = split_sectors(&sectors_of(&raw_planets()));
    for s in ["Umlaut", "Guang", "Omega"] {
        assert!(raw.iter().any(|r| r == s), "{s} in {raw:?}");
    }
    assert_eq!(split_sectors(&sectors_of(&fixed_planets())), Vec::<String>::new());
}

#[test]
fn gives_two_sectors_one_boundary_angle_in_every_ring_where_their_planets_allow_it() {
    let geo = sectors_of(&fixed_planets());
    let mut angles: HashMap<String, HashSet<String>> = HashMap::new();
    for ring in &geo.rings {
        if ring.runs.len() < 2 {
            continue;
        }
        for (k, run) in ring.runs.iter().enumerate() {
            let prev = &ring.runs[(k + ring.runs.len() - 1) % ring.runs.len()];
            let a = warroom_core::format::to_fixed(((run.a0 % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI), 4);
            angles.entry(format!("{}|{}", prev.sector, run.sector)).or_default().insert(a);
        }
    }
    let shared = angles.values().filter(|s| s.len() == 1).count();
    assert!(shared as f64 / angles.len() as f64 > 0.9);
}

#[test]
fn a_landmark_reads_as_a_planet_with_an_index_of_its_own() {
    let war_data = planets();
    for l in LANDMARKS {
        assert!(!war_data.iter().any(|p| p.index == l.index), "{} shares an index", l.name);
        assert_eq!(landmark(l.index).map(|found| found.name), Some(l.name));
    }
    assert!(landmark(0).is_none());

    let mars = landmark_planet(landmark(-1).unwrap());
    assert_eq!((mars.name.as_str(), mars.sector.as_str()), ("MARS", "Sol"));
    assert_eq!(mars.current_owner, Faction::Humans);
    assert_eq!(mars.biome.name, "Scorched");
    assert_eq!(mars.statistics.player_count, 0.0);
}
