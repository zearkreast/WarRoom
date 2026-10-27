mod common;
use common::*;
use std::collections::HashSet;
use warroom_core::api::Faction;
use warroom_core::collate::locale_cmp;
use warroom_core::planet_index::{
    PlanetFilter, SectorSort, control_shares, group_by_sector, matches_filter, matches_search,
};

fn active() -> HashSet<i64> {
    campaigns().iter().map(|c| c.planet.index).collect()
}

#[test]
fn searches_planet_sector_biome_and_hazard_names() {
    let planets = planets();
    let fury = planets.iter().find(|p| p.name == "FURY").unwrap();
    assert!(matches_search(fury, "fury"));
    assert!(matches_search(fury, &fury.sector.to_lowercase()));
    assert!(matches_search(fury, &format!("{} fury", fury.biome.name)));
    assert!(!matches_search(fury, "fury nonsense"));
}

#[test]
fn filters_by_front_and_owner() {
    let planets = planets();
    let active = active();
    let none = HashSet::new();
    assert_eq!(
        planets.iter().filter(|p| matches_filter(p, PlanetFilter::Active, &active, &none)).count(),
        active.len()
    );
    assert!(planets.iter().all(|p| !matches_filter(p, PlanetFilter::Owner(Faction::Terminids), &active, &none)
        || p.current_owner == Faction::Terminids));
    let pinned = HashSet::from([planets[3].index]);
    let hits: Vec<i64> =
        planets.iter().filter(|p| matches_filter(p, PlanetFilter::Pinned, &active, &pinned)).map(|p| p.index).collect();
    assert_eq!(hits, [planets[3].index]);
}

#[test]
fn groups_by_sector_with_exactly_one_hottest_front_and_fronts_first() {
    let planets = planets();
    let active = active();
    let groups = group_by_sector(&planets, &active, SectorSort::Players);
    assert_eq!(groups.iter().filter(|g| g.hottest).count(), 1);
    assert_eq!(groups.iter().map(|g| g.planets.len()).sum::<usize>(), planets.len());
    for g in &groups {
        let flags: Vec<bool> = g.planets.iter().map(|p| active.contains(&p.index)).collect();
        let mut sorted = flags.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(flags, sorted);
    }
    for w in groups.windows(2) {
        assert!(w[0].players >= w[1].players);
    }
    let az: Vec<String> = group_by_sector(&planets, &active, SectorSort::Az).iter().map(|g| g.name.clone()).collect();
    let mut sorted = az.clone();
    sorted.sort_by(|a, b| locale_cmp(a, b));
    assert_eq!(az, sorted);
}

#[test]
fn control_shares_add_up_to_the_whole_galaxy() {
    let planets = planets();
    let shares = control_shares(&planets);
    assert_eq!(shares.iter().map(|s| s.planets).sum::<usize>(), planets.len());
    assert!((shares.iter().map(|s| s.pct).sum::<f64>() - 100.0).abs() < 1e-6);
}
