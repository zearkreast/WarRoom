#![allow(dead_code)]

use warroom_core::api::{Assignment, Campaign, Dispatch, EpisodeRaw, Faction, Planet, SpaceStation, War, WarStatusRaw};
use warroom_core::galaxy::{PolarSectors, SectorPlanet, build_polar_sectors};
use warroom_core::planets::{fix_static, is_placeholder};
use warroom_core::snapshot::{
    EpisodeLive, EpisodePhase, EpisodeStatus, merge_episodes, to_episodes, to_live, to_planet, to_static,
};
use warroom_core::time::iso_string;

pub const H: f64 = 3_600_000.0;

pub fn planets() -> Vec<Planet> {
    warroom_core::api::from_str(include_str!("../fixtures/planets.json")).unwrap()
}

pub fn assignments() -> Vec<Assignment> {
    warroom_core::api::from_str(include_str!("../fixtures/assignments.json")).unwrap()
}

pub fn campaigns() -> Vec<Campaign> {
    warroom_core::api::from_str(include_str!("../fixtures/campaigns.json")).unwrap()
}

pub fn dispatches() -> Vec<Dispatch> {
    warroom_core::api::from_str(include_str!("../fixtures/dispatches.json")).unwrap()
}

pub fn war() -> War {
    warroom_core::api::from_str(include_str!("../fixtures/war.json")).unwrap()
}

pub fn stations() -> Vec<SpaceStation> {
    warroom_core::api::from_str(include_str!("../fixtures/space-stations.json")).unwrap()
}

pub fn status_effects() -> WarStatusRaw {
    warroom_core::api::from_str(include_str!("../fixtures/status-effects.json")).unwrap()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodesFixture {
    pub episodes: Vec<EpisodeRaw>,
    pub war_time: f64,
    pub fetched_at: f64,
}

pub fn episodes() -> EpisodesFixture {
    warroom_core::api::from_str(include_str!("../fixtures/episodes.json")).unwrap()
}

pub fn ms(iso: &str) -> f64 {
    warroom_core::time::parse_iso(iso).unwrap()
}

pub fn close(a: f64, b: f64, digits: i32) -> bool {
    (a - b).abs() < 10f64.powi(-digits) / 2.0
}

pub fn raw_planets() -> Vec<Planet> {
    planets().into_iter().filter(|p| !is_placeholder(p.index, p.position)).collect()
}

/// Planets with the static fixes applied, as the app builds them.
pub fn fixed_planets() -> Vec<Planet> {
    raw_planets().iter().map(|p| to_planet(&fix_static(to_static(p)), &to_live(p))).collect()
}

pub fn sectors_of(planets: &[Planet]) -> PolarSectors {
    let list: Vec<SectorPlanet> =
        planets.iter().map(|p| SectorPlanet { index: p.index, sector: &p.sector, position: p.position }).collect();
    build_polar_sectors(&list)
}

pub fn campaigns_live() -> Vec<EpisodeLive> {
    let f = episodes();
    merge_episodes(&[], &to_episodes(&f.episodes, f.war_time, f.fetched_at), 0.0)
}

pub fn dispatch(id: i64, published: f64, message: &str) -> Dispatch {
    Dispatch { id, published: iso_string(published), kind: 0, message: message.into() }
}

pub fn phase(title: &str, status: EpisodeStatus) -> EpisodePhase {
    EpisodePhase { title: title.into(), status, ..Default::default() }
}

pub fn episode(start: f64, status: EpisodeStatus, phases: Vec<EpisodePhase>) -> EpisodeLive {
    EpisodeLive {
        id: start as i64,
        title: "CAMPAIGN".into(),
        faction: Faction::Humans,
        status,
        start,
        end: Some(start + 1.0),
        phases,
        ..Default::default()
    }
}
