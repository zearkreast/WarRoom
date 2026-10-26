#![allow(dead_code)]

use warroom_core::api::EpisodeRaw;
use warroom_core::api::{Assignment, Campaign, Dispatch, Planet, SpaceStation, War, WarStatusRaw};

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
