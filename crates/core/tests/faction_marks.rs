mod common;
use common::*;
use std::collections::HashSet;
use warroom_core::api::{Faction, Planet};
use warroom_core::faction_marks::{Overlay, faction_marks};
use warroom_core::galaxy::{GALAXY_SCALE, sector_at, sector_territory};

#[test]
fn puts_each_enemys_emblem_inside_its_own_territory_in_the_sectors_overlay() {
    let planets = fixed_planets();
    let geo = sectors_of(&planets);
    let territory = sector_territory(&planets);
    let marks = faction_marks(&planets, Overlay::Sectors, Some(&geo));
    assert!(!marks.is_empty());
    for m in &marks {
        let sector = sector_at(&geo, m.x / GALAXY_SCALE, -m.y / GALAXY_SCALE).unwrap_or("");
        let t = territory.iter().find(|(s, _)| s == sector).and_then(|(_, t)| *t);
        assert_eq!(t.map(|t| t.faction), Some(m.faction));
    }
}

#[test]
fn puts_each_enemys_emblem_on_its_largest_group_of_planets_in_the_other_overlays() {
    let planets = fixed_planets();
    for overlay in [Overlay::Regions, Overlay::Galaxy] {
        let marks = faction_marks(&planets, overlay, None);
        let mut got: Vec<Faction> = marks.iter().map(|m| m.faction).collect();
        got.sort();
        let mut held: Vec<Faction> = planets
            .iter()
            .filter(|p| p.current_owner != Faction::Humans)
            .map(|p| p.current_owner)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        held.sort();
        assert_eq!(got, held);
        for m in &marks {
            let d = |p: &Planet| (p.position.x * 500.0 - m.x).hypot(-p.position.y * 500.0 - m.y);
            let mut nearest = &planets[0];
            for p in &planets[1..] {
                if d(p) < d(nearest) {
                    nearest = p;
                }
            }
            assert_eq!(nearest.current_owner, m.faction);
        }
    }
}
