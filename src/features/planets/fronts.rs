use crate::api::queries::{LiveCampaign, queries};
use crate::state::{global, settings};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use warroom_core::api::{Campaign, Faction, Planet};
use warroom_core::progress::{defense_percent, liberation_percent, projected_at, projected_completion, projected_loss};
use warroom_core::time::parse_ms;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontKind {
    Liberation,
    Defense,
}

impl FrontKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FrontKind::Liberation => "liberation",
            FrontKind::Defense => "defense",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Front {
    pub planet: Planet,
    pub campaign: Campaign,
    pub kind: FrontKind,
    pub enemy: Faction,
    pub percent: f64,
    pub rate: Option<f64>,
    /// Victory time at the current rate, if it's progressing.
    pub victory_at: Option<f64>,
    /// Liberation: when progress hits 0%. Defense: the event deadline.
    pub fail_at: Option<f64>,
    /// Defense only: projected % at the deadline.
    pub projected: Option<f64>,
    pub players: f64,
    pub in_major_order: bool,
    pub pinned: bool,
    pub station: bool,
}

pub fn build_front(
    c: &Campaign,
    rate: Option<f64>,
    mo_planets: &HashSet<i64>,
    now: f64,
    pinned: &HashSet<i64>,
    stations: &HashSet<i64>,
) -> Front {
    let p = &c.planet;
    let marks = (mo_planets.contains(&p.index), pinned.contains(&p.index), stations.contains(&p.index));
    if let (Some(def), Some(event)) = (defense_percent(p), &p.event) {
        let deadline = parse_ms(&event.end_time);
        return Front {
            planet: p.clone(),
            campaign: c.clone(),
            kind: FrontKind::Defense,
            enemy: event.faction,
            percent: def,
            rate,
            victory_at: projected_completion(def, rate, now),
            fail_at: Some(deadline),
            projected: projected_at(def, rate, now, deadline),
            players: p.statistics.player_count,
            in_major_order: marks.0,
            pinned: marks.1,
            station: marks.2,
        };
    }
    let lib = liberation_percent(p.health, p.max_health);
    Front {
        planet: p.clone(),
        campaign: c.clone(),
        kind: FrontKind::Liberation,
        enemy: p.current_owner,
        percent: lib,
        rate,
        victory_at: projected_completion(lib, rate, now),
        fail_at: projected_loss(lib, rate, now),
        projected: None,
        players: p.statistics.player_count,
        in_major_order: marks.0,
        pinned: marks.1,
        station: marks.2,
    }
}

struct Shared {
    mo_planets: Memo<Arc<HashSet<i64>>>,
    mo_deadlines: Memo<Arc<HashMap<i64, f64>>>,
    station_planets: Memo<Arc<HashSet<i64>>>,
    fronts: Memo<Arc<Vec<Front>>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

fn shared() -> &'static Shared {
    SHARED.get_or_init(|| {
        global(|| {
            let q = queries();
            let mo_planets = Memo::new(move |_| {
                let mut s = HashSet::new();
                for a in q.assignments.get().iter().flat_map(|l| l.iter()) {
                    for t in &a.tasks {
                        if let Some(i) = t.value_types.iter().position(|v| *v == 12) {
                            if t.values.get(i).is_some_and(|v| *v > 0.0) {
                                s.insert(t.values[i] as i64);
                            }
                        }
                    }
                }
                Arc::new(s)
            });
            let mo_deadlines = Memo::new(move |_| {
                let mut m: HashMap<i64, f64> = HashMap::new();
                for a in q.assignments.get().iter().flat_map(|l| l.iter()) {
                    let ends = parse_ms(&a.expiration);
                    for t in &a.tasks {
                        let planet = t
                            .value_types
                            .iter()
                            .position(|v| *v == 12)
                            .and_then(|i| t.values.get(i))
                            .copied()
                            .unwrap_or(0.0);
                        if planet > 0.0 && !(ends >= m.get(&(planet as i64)).copied().unwrap_or(f64::INFINITY)) {
                            m.insert(planet as i64, ends);
                        }
                    }
                }
                Arc::new(m)
            });
            let station_planets = Memo::new(move |_| {
                Arc::new(q.stations.get().iter().flat_map(|l| l.iter()).map(|s| s.planet_index).collect::<HashSet<_>>())
            });
            let fronts = Memo::new(move |_| {
                let data = q.campaigns.get();
                let mo = mo_planets.get();
                let stations = station_planets.get();
                let pinned: HashSet<i64> = settings().get(|s| s.pinned.iter().copied().collect());
                let updated = q.raw.updated_at.get();
                let now = if updated > 0.0 { updated } else { crate::js::now() };
                let mut list: Vec<Front> = data
                    .iter()
                    .flat_map(|l| l.iter())
                    .map(|c: &LiveCampaign| build_front(&c.campaign, c.rate, &mo, now, &pinned, &stations))
                    .collect();
                list.sort_by(|a, b| {
                    b.pinned
                        .cmp(&a.pinned)
                        .then(b.in_major_order.cmp(&a.in_major_order))
                        .then((b.kind == FrontKind::Defense).cmp(&(a.kind == FrontKind::Defense)))
                        .then(b.players.total_cmp(&a.players))
                });
                Arc::new(list)
            });
            Shared { mo_planets, mo_deadlines, station_planets, fronts }
        })
    })
}

/// Planet indices referenced by any active Major Order task.
pub fn major_order_planets() -> Memo<Arc<HashSet<i64>>> {
    shared().mo_planets
}

/// Each Major Order target planet's order deadline, epoch ms (the earliest, if two orders share one).
pub fn major_order_deadlines() -> Memo<Arc<HashMap<i64, f64>>> {
    shared().mo_deadlines
}

/// Planet indices with a Democracy Space Station in orbit.
pub fn station_planets() -> Memo<Arc<HashSet<i64>>> {
    shared().station_planets
}

/// The collector's progress rate (%/h) for a planet with an active campaign, else None.
pub fn planet_rate(planet: Option<i64>) -> Option<f64> {
    let planet = planet?;
    queries().campaigns.with(|c| c.as_ref()?.iter().find(|c| c.planet.index == planet)?.rate)
}

pub fn fronts() -> Memo<Arc<Vec<Front>>> {
    shared().fronts
}
