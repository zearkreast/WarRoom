//! Snapshot shapes stored in Supabase: written by the collector, read by the app.

use crate::api::{
    Assignment, Campaign, Dispatch, EpisodeRaw, Faction, Named, Planet, PlanetEvent, Position, Region, SpaceStation,
    Statistics, TacticalAction, War, WarStatusRaw,
};
use crate::factions::faction_by_id;
use crate::format::js_round;
use crate::time::iso_string;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    War,
    Assignments,
    Campaigns,
    Planets,
    PlanetsStatic,
    Dispatches,
    SpaceStations,
    Episodes,
    PlanetEffects,
    OrderSamples,
}

impl SnapshotKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SnapshotKind::War => "war",
            SnapshotKind::Assignments => "assignments",
            SnapshotKind::Campaigns => "campaigns",
            SnapshotKind::Planets => "planets",
            SnapshotKind::PlanetsStatic => "planets_static",
            SnapshotKind::Dispatches => "dispatches",
            SnapshotKind::SpaceStations => "space_stations",
            SnapshotKind::Episodes => "episodes",
            SnapshotKind::PlanetEffects => "planet_effects",
            SnapshotKind::OrderSamples => "order_samples",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RegionStatic {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub size: String,
    pub max_health: f64,
}

/// Planet fields that rarely change: sent once per app launch.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlanetStatic {
    pub index: i64,
    pub name: String,
    pub sector: String,
    pub biome: Named,
    pub hazards: Vec<Named>,
    pub hash: i64,
    pub position: Position,
    pub waypoints: Vec<i64>,
    pub max_health: f64,
    pub initial_owner: Faction,
    pub regions: Vec<RegionStatic>,
}

/// Planet statistics with zero counters left out (most planets have never been fought on).
/// timePlayed duplicates missionTime and accuracy is derivable, so neither is kept.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LiveStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_count: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missions_won: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missions_lost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_success_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminid_kills: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automaton_kills: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub illuminate_kills: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullets_fired: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullets_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub friendlies: Option<f64>,
}

fn nonzero(v: f64) -> Option<f64> {
    (v != 0.0 && !v.is_nan()).then_some(v)
}

fn live_stats(s: &Statistics) -> LiveStats {
    LiveStats {
        player_count: nonzero(s.player_count),
        missions_won: nonzero(s.missions_won),
        missions_lost: nonzero(s.missions_lost),
        mission_success_rate: nonzero(s.mission_success_rate),
        mission_time: nonzero(s.mission_time),
        terminid_kills: nonzero(s.terminid_kills),
        automaton_kills: nonzero(s.automaton_kills),
        illuminate_kills: nonzero(s.illuminate_kills),
        bullets_fired: nonzero(s.bullets_fired),
        bullets_hit: nonzero(s.bullets_hit),
        deaths: nonzero(s.deaths),
        friendlies: nonzero(s.friendlies),
    }
}

/// A region's changing fields. Snapshots from older collectors lack the optional ones.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RegionLive {
    pub id: i64,
    pub health: Option<f64>,
    pub is_available: bool,
    pub players: f64,
    pub regen_per_second: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_factor: Option<f64>,
    /// Progress trend in %/h; only on campaign planets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
}

/// Planet fields that change every poll.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlanetLive {
    pub index: i64,
    pub health: f64,
    pub current_owner: Faction,
    pub regen_per_second: f64,
    pub disabled: bool,
    pub event: Option<PlanetEvent>,
    pub attacking: Vec<i64>,
    pub statistics: LiveStats,
    pub regions: Vec<RegionLive>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CampaignLive {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: i64,
    pub count: i64,
    pub faction: Faction,
    pub planet: PlanetLive,
    /// Progress trend in %/h, computed by the collector from its samples; None while measuring.
    pub rate: Option<f64>,
}

/// Newest dispatches kept in the snapshot.
pub const DISPATCH_LIMIT: usize = 50;

pub fn to_static(p: &Planet) -> PlanetStatic {
    PlanetStatic {
        index: p.index,
        name: p.name.clone(),
        sector: p.sector.clone(),
        biome: p.biome.clone(),
        hazards: p.hazards.clone(),
        hash: p.hash,
        position: p.position,
        waypoints: p.waypoints.clone(),
        max_health: p.max_health,
        initial_owner: p.initial_owner,
        regions: p
            .regions
            .iter()
            .map(|r| RegionStatic {
                id: r.id,
                name: r.name.clone(),
                description: r.description.clone(),
                size: r.size.clone(),
                max_health: r.max_health,
            })
            .collect(),
    }
}

pub fn to_live(p: &Planet) -> PlanetLive {
    PlanetLive {
        index: p.index,
        health: p.health,
        current_owner: p.current_owner,
        regen_per_second: p.regen_per_second,
        disabled: p.disabled,
        event: p.event.clone(),
        attacking: p.attacking.clone(),
        statistics: live_stats(&p.statistics),
        regions: p
            .regions
            .iter()
            .map(|r| RegionLive {
                id: r.id,
                health: r.health,
                is_available: r.is_available,
                players: r.players,
                regen_per_second: r.regen_per_second,
                availability_factor: r.availability_factor,
                rate: None,
            })
            .collect(),
    }
}

pub fn to_campaign_live(c: &Campaign, rate: Option<f64>, region_rate: impl Fn(i64) -> Option<f64>) -> CampaignLive {
    let mut planet = to_live(&c.planet);
    for r in &mut planet.regions {
        r.rate = region_rate(r.id);
    }
    CampaignLive { id: c.id, kind: c.kind, count: c.count, faction: c.faction, planet, rate }
}

/// One reading of a Major Order's progress, a count per task.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderSample {
    pub id: i64,
    pub at: f64,
    pub progress: Vec<f64>,
}

/// How far back Major Order readings are kept: a day evens out the swing between busy and quiet hours.
pub const ORDER_SAMPLE_WINDOW_MS: f64 = 24.0 * 60.0 * 60_000.0;

/// Adds a reading of each running order; drops those older than the window and those of ended orders.
pub fn add_order_samples(kept: &[OrderSample], orders: &[Assignment], at: f64) -> Vec<OrderSample> {
    let running: Vec<i64> = orders.iter().map(|o| o.id).collect();
    kept.iter()
        .filter(|s| running.contains(&s.id) && s.at > at - ORDER_SAMPLE_WINDOW_MS && s.at < at)
        .cloned()
        .chain(orders.iter().map(|o| OrderSample { id: o.id, at, progress: o.progress.clone() }))
        .collect()
}

pub fn newest_dispatches(d: &[Dispatch]) -> Vec<Dispatch> {
    let mut list = d.to_vec();
    list.sort_by_key(|d| std::cmp::Reverse(d.id));
    list.truncate(DISPATCH_LIMIT);
    list
}

/// Rebuilds the full `Planet` the UI works with from its static and live halves.
pub fn to_planet(s: &PlanetStatic, l: &PlanetLive) -> Planet {
    let live: HashMap<i64, &RegionLive> = l.regions.iter().map(|r| (r.id, r)).collect();
    let st = &l.statistics;
    Planet {
        index: s.index,
        name: s.name.clone(),
        sector: s.sector.clone(),
        biome: s.biome.clone(),
        // The API lists "None" as a hazard on planets without any.
        hazards: s.hazards.iter().filter(|h| h.name.to_lowercase() != "none").cloned().collect(),
        hash: s.hash,
        position: s.position,
        waypoints: s.waypoints.clone(),
        max_health: s.max_health,
        health: l.health,
        disabled: l.disabled,
        initial_owner: s.initial_owner,
        current_owner: l.current_owner,
        regen_per_second: l.regen_per_second,
        event: l.event.clone(),
        statistics: Statistics {
            missions_won: st.missions_won.unwrap_or(0.0),
            missions_lost: st.missions_lost.unwrap_or(0.0),
            mission_time: st.mission_time.unwrap_or(0.0),
            terminid_kills: st.terminid_kills.unwrap_or(0.0),
            automaton_kills: st.automaton_kills.unwrap_or(0.0),
            illuminate_kills: st.illuminate_kills.unwrap_or(0.0),
            bullets_fired: st.bullets_fired.unwrap_or(0.0),
            bullets_hit: st.bullets_hit.unwrap_or(0.0),
            time_played: 0.0,
            deaths: st.deaths.unwrap_or(0.0),
            revives: 0.0,
            friendlies: st.friendlies.unwrap_or(0.0),
            mission_success_rate: st.mission_success_rate.unwrap_or(0.0),
            accuracy: 0.0,
            player_count: st.player_count.unwrap_or(0.0),
        },
        attacking: l.attacking.clone(),
        regions: s
            .regions
            .iter()
            .map(|r| {
                let l = live.get(&r.id);
                Region {
                    id: r.id,
                    name: r.name.clone(),
                    description: r.description.clone(),
                    health: l.and_then(|l| l.health),
                    max_health: r.max_health,
                    size: r.size.clone(),
                    regen_per_second: l.and_then(|l| l.regen_per_second),
                    availability_factor: l.and_then(|l| l.availability_factor),
                    is_available: l.is_some_and(|l| l.is_available),
                    players: l.map_or(0.0, |l| l.players),
                    rate: l.and_then(|l| l.rate),
                }
            })
            .collect(),
    }
}

/// Space station as stored: the planet is reduced to its index (the app has the planet list).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StationLive {
    pub id32: i64,
    pub planet_index: i64,
    pub election_end: String,
    pub flags: i64,
    pub tactical_actions: Vec<TacticalAction>,
}

pub fn to_station_live(s: &SpaceStation) -> StationLive {
    StationLive {
        id32: s.id32,
        planet_index: s.planet.index,
        election_end: s.election_end.clone(),
        flags: s.flags,
        tactical_actions: s.tactical_actions.clone(),
    }
}

/// One row per war poll: galaxy-wide trends for the Overview and Planets charts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WarSampleRow {
    pub at: String,
    pub impact: f64,
    pub players: f64,
    pub terminids: f64,
    pub automatons: f64,
    pub illuminate: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnemyPlayers {
    pub terminids: f64,
    pub automaton: f64,
    pub illuminate: f64,
}

/// Helldivers on each enemy front, from the active campaigns (defenses count for the attacker).
pub fn players_by_enemy<'a>(planets: impl IntoIterator<Item = &'a Planet>) -> EnemyPlayers {
    let mut out = EnemyPlayers::default();
    for p in planets {
        let enemy = p.event.as_ref().map_or(p.current_owner, |e| e.faction);
        let n = p.statistics.player_count;
        match enemy {
            Faction::Terminids => out.terminids += n,
            Faction::Automaton => out.automaton += n,
            Faction::Illuminate => out.illuminate += n,
            _ => {}
        }
    }
    out
}

pub fn to_war_sample<'a>(war: &War, campaign_planets: impl IntoIterator<Item = &'a Planet>, at: f64) -> WarSampleRow {
    let fronts = players_by_enemy(campaign_planets);
    WarSampleRow {
        at: iso_string(at),
        impact: war.impact_multiplier,
        players: war.statistics.player_count,
        terminids: fronts.terminids,
        automatons: fronts.automaton,
        illuminate: fronts.illuminate,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeStatus {
    #[default]
    Active,
    Won,
    Lost,
}

/// One Major Order of a campaign.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EpisodePhase {
    pub title: String,
    pub briefing: String,
    /// What the game says once the order has ended; empty while it runs.
    pub outro: String,
    pub status: EpisodeStatus,
    pub medals: Option<f64>,
    /// When the order was issued, epoch ms. The game doesn't publish it: the first order starts with its
    /// campaign, later ones are stamped by the collector when they appear, and older ones stay None.
    pub issued: Option<f64>,
}

/// The order's own name; None while the game still titles a freshly issued order just "MAJOR ORDER".
pub fn phase_name(p: &EpisodePhase) -> Option<&str> {
    let title = p.title.trim();
    (!title.is_empty() && title.to_uppercase() != "MAJOR ORDER").then_some(title)
}

/// A Galactic Campaign: Major Orders under one name, won when most of them are won; times are epoch ms.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EpisodeLive {
    pub id: i64,
    pub title: String,
    pub description: String,
    /// The enemy it is fought against; Humans when it spans several fronts.
    pub faction: Faction,
    pub status: EpisodeStatus,
    pub start: f64,
    pub end: Option<f64>,
    /// Item id of the campaign reward, if one is offered.
    pub reward: Option<i64>,
    pub phases: Vec<EpisodePhase>,
}

// The game's episode and phase status codes: 2 won, 3 failed; anything else is still running.
fn episode_status(code: i64) -> EpisodeStatus {
    match code {
        2 => EpisodeStatus::Won,
        3 => EpisodeStatus::Lost,
        _ => EpisodeStatus::Active,
    }
}

const MEDALS_MIX_ID: i64 = 897_894_480;

/// Converts the game's war-second timestamps using the war clock read at `fetched_at`.
pub fn to_episodes(list: &[EpisodeRaw], war_time: f64, fetched_at: f64) -> Vec<EpisodeLive> {
    let at = |t: f64| fetched_at - (war_time - t) * 1000.0;
    list.iter()
        .map(|e| {
            let start = at(e.start_war_time);
            EpisodeLive {
                id: e.id32,
                title: e.title.clone(),
                description: e.description.clone(),
                faction: faction_by_id(e.race).unwrap_or(Faction::Humans),
                status: episode_status(e.status),
                start,
                end: e.end_war_time.filter(|t| *t != 0.0 && !t.is_nan()).map(at),
                reward: e.rewards.first().map(|r| r.mix_id),
                phases: e
                    .phases
                    .iter()
                    .enumerate()
                    .map(|(i, p)| EpisodePhase {
                        title: p.intro_title.clone(),
                        briefing: p.intro_message.clone(),
                        outro: p.outro_message.clone().unwrap_or_default(),
                        status: episode_status(p.status),
                        medals: p.rewards.iter().find(|r| r.mix_id == MEDALS_MIX_ID).map(|r| r.amount),
                        issued: (i == 0).then_some(start),
                    })
                    .collect(),
            }
        })
        .collect()
}

/// An order's issue date, or None when it can't be right: an older collector stamped orders of
/// campaigns that had already ended with the time it first saw them.
pub fn issued_at(start: f64, end: Option<f64>, issued: Option<f64>) -> Option<f64> {
    let issued = issued.filter(|i| *i >= start)?;
    match end {
        Some(end) if issued > end => None,
        _ => Some(issued),
    }
}

/// Folds a fresh read into the stored list. Campaigns the game stops listing are kept, stored dates
/// win over re-derived ones (the war clock drifts a little between reads), and a Major Order that
/// appears on an already stored, still running campaign is stamped as issued at `now`.
pub fn merge_episodes(stored: &[EpisodeLive], fresh: &[EpisodeLive], now: f64) -> Vec<EpisodeLive> {
    let before: HashMap<i64, &EpisodeLive> = stored.iter().map(|e| (e.id, e)).collect();
    let merged: Vec<EpisodeLive> = fresh
        .iter()
        .map(|e| {
            let was = before.get(&e.id).copied();
            let start = was.map_or(e.start, |w| w.start);
            let end = was.map_or(e.end, |w| w.end.or(e.end));
            let phases = e
                .phases
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let issued = if i == 0 {
                        Some(start)
                    } else if let Some(old) = was.and_then(|w| w.phases.get(i)) {
                        issued_at(start, end, old.issued)
                    } else if was.is_some() && e.status == EpisodeStatus::Active {
                        Some(now)
                    } else {
                        None
                    };
                    EpisodePhase { issued, ..p.clone() }
                })
                .collect();
            EpisodeLive { start, end, phases, ..e.clone() }
        })
        .collect();
    let mut out: Vec<EpisodeLive> =
        stored.iter().filter(|e| !fresh.iter().any(|f| f.id == e.id)).cloned().chain(merged).collect();
    out.sort_by(|a, b| a.start.total_cmp(&b.start));
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tally {
    pub won: usize,
    pub lost: usize,
    pub running: usize,
}

/// Major Orders won, lost and still running. The game doesn't say how many a campaign will have.
pub fn campaign_tally(e: &EpisodeLive) -> Tally {
    let won = e.phases.iter().filter(|p| p.status == EpisodeStatus::Won).count();
    let lost = e.phases.iter().filter(|p| p.status == EpisodeStatus::Lost).count();
    Tally { won, lost, running: e.phases.len() - won - lost }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CampaignRecord {
    pub won: usize,
    pub orders: usize,
    pub orders_won: usize,
    pub streak: usize,
}

/// The archive's service record: campaigns and Major Orders won, and the victories in a row since the last failure.
pub fn campaign_record(past: &[EpisodeLive]) -> CampaignRecord {
    let mut newest_first: Vec<&EpisodeLive> = past.iter().collect();
    newest_first.sort_by(|a, b| b.start.total_cmp(&a.start));
    let streak = newest_first.iter().position(|e| e.status != EpisodeStatus::Won).unwrap_or(past.len());
    CampaignRecord {
        won: past.iter().filter(|e| e.status == EpisodeStatus::Won).count(),
        orders: past.iter().map(|e| e.phases.len()).sum(),
        orders_won: past.iter().map(|e| campaign_tally(e).won).sum(),
        streak,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlanetEffectIds {
    pub index: i64,
    pub ids: Vec<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StationEffectIds {
    pub id: i64,
    pub ids: Vec<i64>,
}

/// Galactic effect ids active on each planet and space station, from the official war status.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsLive {
    pub planets: Vec<PlanetEffectIds>,
    pub stations: Vec<StationEffectIds>,
    /// Effects a war event grants its planets until `until` (epoch ms, to the minute).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub timed: Vec<TimedEffects>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimedEffects {
    pub ids: Vec<i64>,
    pub planets: Vec<i64>,
    pub until: f64,
}

/// `at` is when `status` was fetched, which places the events' expiry in real time.
pub fn to_effects_live(status: &WarStatusRaw, at: f64) -> EffectsLive {
    let mut by_planet: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for e in &status.planet_active_effects {
        let ids = by_planet.entry(e.index).or_default();
        if !ids.contains(&e.galactic_effect_id) {
            ids.push(e.galactic_effect_id);
        }
    }
    EffectsLive {
        planets: by_planet
            .into_iter()
            .map(|(index, mut ids)| {
                ids.sort_unstable();
                PlanetEffectIds { index, ids }
            })
            .collect(),
        stations: status
            .space_stations
            .iter()
            .map(|s| {
                let mut ids = s.active_effect_ids.clone();
                ids.sort_unstable();
                StationEffectIds { id: s.id32, ids }
            })
            .collect(),
        // To the minute, so the snapshot is only rewritten when an event changes.
        timed: status
            .global_events
            .iter()
            .filter(|g| !g.effect_ids.is_empty() && !g.planet_indices.is_empty() && g.expire_time > status.time)
            .map(|g| {
                let mut ids = g.effect_ids.clone();
                ids.sort_unstable();
                let mut planets = g.planet_indices.clone();
                planets.sort_unstable();
                let until = js_round((at + (g.expire_time - status.time) * 1000.0) / 60_000.0) * 60_000.0;
                TimedEffects { ids, planets, until }
            })
            .collect(),
    }
}
