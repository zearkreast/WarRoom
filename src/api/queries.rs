//! The collector's snapshots, polled once a minute: only what changed since the last poll is downloaded,
//! and each derived list is rebuilt only when one of its own inputs changed.

use crate::api::source::{self, SnapshotRow};
use crate::config::POLL_MS;
use crate::js;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use warroom_core::api::{Assignment, Campaign, Dispatch, Planet, War, from_value};
use warroom_core::effects::{GLOOM_IDS, PlanetEffect, planet_effects};
use warroom_core::galaxy::{PolarSectors, SectorPlanet, build_polar_sectors};
use warroom_core::planets::{fix_static, is_off_map};
use warroom_core::snapshot::{
    CampaignLive, EffectsLive, EpisodeLive, OrderSample, PlanetLive, PlanetStatic, SnapshotKind, StationLive, to_planet,
};

/// A campaign with its planet rebuilt in full, plus the collector's progress rate.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveCampaign {
    pub campaign: Campaign,
    pub rate: Option<f64>,
}

impl std::ops::Deref for LiveCampaign {
    type Target = Campaign;
    fn deref(&self) -> &Campaign {
        &self.campaign
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WarEffects {
    /// Named effects per planet index; planets without any are absent.
    pub planets: HashMap<i64, Vec<PlanetEffect>>,
    /// Planets in or on the edge of the Gloom.
    pub gloom: HashSet<i64>,
    pub stations: HashMap<i64, Vec<PlanetEffect>>,
}

type Slot<T> = RwSignal<Option<Arc<T>>>;
pub type Data<T> = Memo<Option<Arc<T>>>;

#[derive(Clone, Copy)]
pub struct Snapshots {
    war: Slot<War>,
    assignments: Slot<Vec<Assignment>>,
    campaigns: Slot<Vec<CampaignLive>>,
    planets_live: Slot<Vec<PlanetLive>>,
    planets_static: Slot<Vec<PlanetStatic>>,
    dispatches: Slot<Vec<Dispatch>>,
    stations: Slot<Vec<StationLive>>,
    episodes: Slot<Vec<EpisodeLive>>,
    effects: Slot<EffectsLive>,
    order_samples: Slot<Vec<OrderSample>>,
    /// Newest `fetched_at` seen, so the next poll only downloads what changed.
    since: RwSignal<String>,
    /// No poll has succeeded yet.
    pub loading: RwSignal<bool>,
    /// The latest poll failed, after its retries; the last good data stays.
    pub error: RwSignal<bool>,
    /// When the latest poll succeeded, epoch ms.
    pub updated_at: RwSignal<f64>,
}

#[derive(Clone, Copy)]
pub struct Queries {
    pub raw: Snapshots,
    pub war: Data<War>,
    pub assignments: Data<Vec<Assignment>>,
    pub dispatches: Data<Vec<Dispatch>>,
    /// The last day of Major Order readings; None until the collector has written them once.
    pub order_samples: Data<Vec<OrderSample>>,
    /// Story campaigns, oldest first; None until the collector has written them once.
    pub episodes: Data<Vec<EpisodeLive>>,
    pub stations: Data<Vec<StationLive>>,
    pub planets: Data<Vec<Planet>>,
    pub campaigns: Data<Vec<LiveCampaign>>,
    /// All planets, with campaign planets swapped for their fresher copy (campaigns poll every minute).
    pub fresh_planets: Data<Vec<Planet>>,
    /// Sector shapes depend only on where planets are, so they're built once per static snapshot.
    pub geometry: Data<PolarSectors>,
    /// Galactic effects on planets and stations; None until the collector has written them once.
    pub effects: Data<WarEffects>,
    /// Newest collector write seen, epoch ms; used to warn when the collector has stopped.
    pub last_update: Memo<Option<f64>>,
}

static QUERIES: OnceLock<Queries> = OnceLock::new();

pub fn queries() -> Queries {
    *QUERIES.get().expect("queries initialised")
}

fn memo<T: PartialEq + Send + Sync + 'static>(f: impl Fn() -> Option<T> + Send + Sync + 'static) -> Data<T> {
    Memo::new(move |_| f().map(Arc::new))
}

fn pass<T: PartialEq + Send + Sync + 'static>(s: Slot<T>) -> Data<T> {
    Memo::new(move |_| s.get())
}

fn slot<T: Send + Sync + 'static>() -> Slot<T> {
    RwSignal::new(None)
}

/// Creates the snapshot store and its derived lists; called once, before anything renders.
pub fn init() {
    let raw = Snapshots {
        war: slot(),
        assignments: slot(),
        campaigns: slot(),
        planets_live: slot(),
        planets_static: slot(),
        dispatches: slot(),
        stations: slot(),
        episodes: slot(),
        effects: slot(),
        order_samples: slot(),
        since: RwSignal::new(String::new()),
        loading: RwSignal::new(true),
        error: RwSignal::new(false),
        updated_at: RwSignal::new(0.0),
    };
    let statics_by_index = Memo::new(move |_| {
        raw.planets_static.get().map(|list| Arc::new(list.iter().map(|p| (p.index, p.clone())).collect::<HashMap<_, _>>()))
    });
    let planets = memo(move || {
        let statics = statics_by_index.get()?;
        let live = raw.planets_live.get()?;
        Some(
            live.iter()
                .filter_map(|l| {
                    let st = statics.get(&l.index)?;
                    (!is_off_map(st.index, st.position)).then(|| to_planet(st, l))
                })
                .collect::<Vec<_>>(),
        )
    });
    let campaigns = memo(move || {
        let statics = statics_by_index.get()?;
        let list = raw.campaigns.get()?;
        Some(
            list.iter()
                .filter_map(|c| {
                    let st = statics.get(&c.planet.index)?;
                    Some(LiveCampaign {
                        campaign: Campaign {
                            id: c.id,
                            kind: c.kind,
                            count: c.count,
                            faction: c.faction,
                            planet: to_planet(st, &c.planet),
                        },
                        rate: c.rate,
                    })
                })
                .collect::<Vec<_>>(),
        )
    });
    let fresh_planets = memo(move || {
        let campaigns = campaigns.get();
        match planets.get() {
            None => campaigns.map(|c| c.iter().map(|c| c.planet.clone()).collect()),
            Some(planets) => {
                let Some(campaigns) = campaigns else { return Some((*planets).clone()) };
                let fresh: HashMap<i64, &Planet> = campaigns.iter().map(|c| (c.planet.index, &c.planet)).collect();
                Some(planets.iter().map(|p| fresh.get(&p.index).map_or_else(|| p.clone(), |f| (*f).clone())).collect())
            }
        }
    });
    let geometry = memo(move || {
        let statics = raw.planets_static.get()?;
        let list: Vec<SectorPlanet> = statics
            .iter()
            .filter(|p| !is_off_map(p.index, p.position))
            .map(|p| SectorPlanet { index: p.index, sector: &p.sector, position: p.position })
            .collect();
        Some(build_polar_sectors(&list))
    });
    let effects = memo(move || {
        let live = raw.effects.get()?;
        Some(WarEffects {
            planets: live.planets.iter().map(|p| (p.index, planet_effects(&p.ids))).collect(),
            gloom: live.planets.iter().filter(|p| p.ids.iter().any(|id| GLOOM_IDS.contains(id))).map(|p| p.index).collect(),
            stations: live.stations.iter().map(|s| (s.id, planet_effects(&s.ids))).collect(),
        })
    });
    let last_update = Memo::new(move |_| {
        let since = raw.since.get();
        (!since.is_empty()).then(|| warroom_core::time::parse_ms(&since))
    });
    let q = Queries {
        raw,
        war: pass(raw.war),
        assignments: pass(raw.assignments),
        dispatches: pass(raw.dispatches),
        order_samples: pass(raw.order_samples),
        episodes: pass(raw.episodes),
        stations: pass(raw.stations),
        planets,
        campaigns,
        fresh_planets,
        geometry,
        effects,
        last_update,
    };
    let _ = QUERIES.set(q);
}

fn put<T: DeserializeOwned + PartialEq + Send + Sync + 'static>(slot: Slot<T>, data: serde_json::Value) {
    match from_value::<T>(data) {
        Ok(v) => {
            if slot.with_untracked(|old| old.as_deref() != Some(&v)) {
                slot.set(Some(Arc::new(v)));
            }
        }
        Err(e) => leptos::logging::warn!("unreadable snapshot: {e}"),
    }
}

fn apply(rows: Vec<SnapshotRow>) {
    let s = queries().raw;
    let mut since = s.since.get_untracked();
    for r in rows {
        if r.fetched_at > since {
            since = r.fetched_at.clone();
        }
        match r.kind {
            SnapshotKind::War => put(s.war, r.data),
            SnapshotKind::Assignments => put(s.assignments, r.data),
            SnapshotKind::Campaigns => put(s.campaigns, r.data),
            SnapshotKind::Planets => put(s.planets_live, r.data),
            SnapshotKind::PlanetsStatic => match from_value::<Vec<PlanetStatic>>(r.data) {
                Ok(list) => {
                    let fixed: Vec<PlanetStatic> = list.into_iter().map(fix_static).collect();
                    if s.planets_static.with_untracked(|old| old.as_deref() != Some(&fixed)) {
                        s.planets_static.set(Some(Arc::new(fixed)));
                    }
                }
                Err(e) => leptos::logging::warn!("unreadable snapshot: {e}"),
            },
            SnapshotKind::Dispatches => put(s.dispatches, r.data),
            SnapshotKind::SpaceStations => put(s.stations, r.data),
            SnapshotKind::Episodes => put(s.episodes, r.data),
            SnapshotKind::PlanetEffects => put(s.effects, r.data),
            SnapshotKind::OrderSamples => put(s.order_samples, r.data),
        }
    }
    if since != s.since.get_untracked() {
        s.since.set(since);
    }
}

/// One poll, retried twice (after 1 s and 2 s) before it counts as failed.
async fn poll() {
    let s = queries().raw;
    for attempt in 0..3 {
        match source::snapshots(&s.since.get_untracked()).await {
            Ok(rows) => {
                apply(rows);
                s.loading.set(false);
                s.error.set(false);
                s.updated_at.set(js::now());
                return;
            }
            Err(_) if attempt < 2 => js::sleep(1000.0 * 2f64.powi(attempt)).await,
            Err(_) => s.error.set(true),
        }
    }
}

/// Polls now and then once a minute for as long as the app runs, in the background too.
pub fn start_polling() {
    spawn_local(async {
        loop {
            poll().await;
            js::sleep(POLL_MS).await;
        }
    });
}

/// One planet by index, fresh copy preferred; None while loading or unknown.
pub fn planet(index: Option<i64>) -> Option<Planet> {
    let index = index?;
    queries().fresh_planets.with(|list| list.as_ref()?.iter().find(|p| p.index == index).cloned())
}
