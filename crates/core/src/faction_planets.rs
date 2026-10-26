use crate::api::{Faction, Planet};
use crate::collate::locale_cmp;
use regex_lite::Regex;
use std::sync::LazyLock;

/// Most planets listed when hovering a faction on the control ring.
pub const LIST_SIZE: usize = 6;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommunityPlanets {
    pub fixed: Vec<String>,
    pub pool: Vec<String>,
}

/// One row of the hover list. Mars is not in the API's planet list, so it carries no planet.
#[derive(Clone, Debug, PartialEq)]
pub struct ListedPlanet {
    pub name: String,
    pub planet: Option<i64>,
    pub lore: Option<&'static str>,
}

static HEADER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\[(fixed|pool)\]$").unwrap());

/// Parses community-planets.txt: `[fixed]` and `[pool]` sections, one name per line, `#` comments.
pub fn parse_community_planets(text: &str) -> CommunityPlanets {
    let mut out = CommunityPlanets::default();
    let mut section: Option<bool> = None;
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(h) = HEADER.captures(line) {
            section = Some(h[1].to_lowercase() == "fixed");
        } else if let Some(fixed) = section {
            let list = if fixed { &mut out.fixed } else { &mut out.pool };
            list.push(line.to_uppercase());
        }
    }
    out
}

pub fn community_planets() -> CommunityPlanets {
    parse_community_planets(include_str!("../data/community-planets.txt"))
}

/// A faction's planets, busiest first.
pub fn busiest_planets(planets: &[Planet], faction: Faction, limit: usize) -> Vec<ListedPlanet> {
    let mut list: Vec<&Planet> = planets.iter().filter(|p| p.current_owner == faction).collect();
    list.sort_by(|a, b| {
        b.statistics.player_count.total_cmp(&a.statistics.player_count).then_with(|| locale_cmp(&a.name, &b.name))
    });
    list.into_iter()
        .take(limit)
        .map(|p| ListedPlanet { name: p.name.clone(), planet: Some(p.index), lore: None })
        .collect()
}

/// Super Earth's list: Super Earth, Mars, the fixed community planets, then random picks from
/// the pool. Only planets Super Earth holds are listed; the pool tops up any gaps.
pub fn super_earth_planets(
    planets: &[Planet],
    community: &CommunityPlanets,
    mut random: impl FnMut() -> f64,
) -> Vec<ListedPlanet> {
    let held: Vec<(String, &Planet)> =
        planets.iter().filter(|p| p.current_owner == Faction::Humans).map(|p| (p.name.to_uppercase(), p)).collect();
    let find = |name: &str| held.iter().rev().find(|(n, _)| n == name).map(|(_, p)| *p);
    let mut list: Vec<ListedPlanet> = Vec::new();
    let add = |list: &mut Vec<ListedPlanet>, name: &str| {
        if let Some(p) = find(name)
            && !list.iter().any(|l| l.planet == Some(p.index))
        {
            list.push(ListedPlanet { name: p.name.clone(), planet: Some(p.index), lore: None });
        }
    };
    add(&mut list, "SUPER EARTH");
    list.push(ListedPlanet { name: "MARS".into(), planet: None, lore: Some("Sol system") });
    for n in &community.fixed {
        add(&mut list, n);
    }
    let names: Vec<String> = list
        .iter()
        .filter_map(|l| l.planet)
        .filter_map(|i| planets.iter().find(|p| p.index == i).map(|p| p.name.to_uppercase()))
        .collect();
    let mut pool: Vec<&String> = community.pool.iter().filter(|n| find(n).is_some() && !names.contains(n)).collect();
    while list.len() < LIST_SIZE && !pool.is_empty() {
        let pick = ((random() * pool.len() as f64).floor() as usize).min(pool.len() - 1);
        let name = pool.remove(pick);
        add(&mut list, name);
    }
    list.truncate(LIST_SIZE);
    list
}
