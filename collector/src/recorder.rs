//! Planet progress samples and ownership changes: loaded from Supabase at start, kept for rates, written each poll.

use crate::supabase::{Supabase, unless_missing};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use warroom_core::api::{Faction, Planet, Region};
use warroom_core::factions::{faction_by_id, faction_id};
use warroom_core::owner_changes::{OwnerChange, detect_owner_changes, is_active};
use warroom_core::progress::{Sample, rate_per_hour, to_sample};
use warroom_core::time::{iso_string, parse_ms};

/// Samples kept in memory for rates. Matches the window rate_per_hour looks at.
const RATE_WINDOW_MS: f64 = 60.0 * 60_000.0;
/// PostgREST returns at most 1000 rows per request by default.
const PAGE: usize = 1000;
/// A city is stored at most this often; the campaigns task reads its planet every minute.
const REGION_EVERY_MS: f64 = 4.5 * 60_000.0;

#[derive(Deserialize)]
struct SampleRow {
    planet: i64,
    at: String,
    owner: i64,
    percent: f64,
    players: f64,
    defending: bool,
}

fn to_row(s: &Sample) -> Value {
    json!({
        "planet": s.planet,
        "at": iso_string(s.at),
        "owner": faction_id(s.owner),
        "percent": s.percent,
        "players": s.players,
        "defending": s.defending,
    })
}

pub struct IngestResult {
    pub samples: usize,
    pub changes: Vec<OwnerChange>,
}

/// A city worth a reading: Helldivers in it, or its liberation counting down.
fn region_row(p: &Planet, r: &Region, at: f64) -> Option<Value> {
    let percent = r
        .health
        .filter(|_| r.is_available && r.max_health != 0.0 && !r.max_health.is_nan())
        .map(|h| 100.0 - (h / r.max_health) * 100.0);
    if r.players <= 0.0 && percent.is_none() {
        return None;
    }
    Some(json!({ "planet": p.index, "region": r.id, "at": iso_string(at), "players": r.players, "percent": percent }))
}

/// A region's liberation as a sample of its planet, so the planet's rate maths applies to it.
fn region_sample(p: &Planet, r: &Region, at: f64) -> Option<Sample> {
    let health = r.health?;
    if r.max_health == 0.0 || r.max_health.is_nan() {
        return None;
    }
    Some(Sample {
        planet: p.index,
        at,
        owner: p.current_owner,
        percent: 100.0 - (health / r.max_health) * 100.0,
        players: r.players,
        defending: p.event.is_some(),
    })
}

fn keep<K: std::hash::Hash + Eq>(map: &mut HashMap<K, Vec<Sample>>, key: K, s: Sample) {
    let cutoff = s.at - RATE_WINDOW_MS;
    let list = map.entry(key).or_default();
    list.push(s);
    let stale = list.iter().take_while(|x| x.at < cutoff).count();
    list.drain(..stale);
}

/// Turns each API snapshot into stored samples and owner-change events, and keeps the
/// last hour of samples in memory so campaign rates survive between polls.
/// Region readings are kept in memory only, so their rates start over after a restart.
#[derive(Default)]
pub struct Recorder {
    owners: HashMap<i64, Faction>,
    recent: HashMap<i64, Vec<Sample>>,
    regions: HashMap<(i64, i64), Vec<Sample>>,
    region_written: HashMap<(i64, i64), f64>,
}

impl Recorder {
    /// Restores state after a restart, so a restart neither forgets rates nor invents captures.
    pub fn load(&mut self, db: &Supabase, now: f64) -> Result<(), String> {
        #[derive(Deserialize)]
        struct OwnerRow {
            planet: i64,
            owner: i64,
        }
        let owners: Vec<OwnerRow> = rows(db.select("planet_owners", "select=planet,owner")?)?;
        self.owners = owners.into_iter().filter_map(|o| Some((o.planet, faction_by_id(o.owner)?))).collect();
        let since = iso_string(now - RATE_WINDOW_MS);
        let mut offset = 0;
        loop {
            let page: Vec<SampleRow> = rows(
                db.select("samples", &format!("select=*&at=gte.{since}&order=at.asc&limit={PAGE}&offset={offset}"))?,
            )?;
            let n = page.len();
            for r in page {
                if let Some(owner) = faction_by_id(r.owner) {
                    let s = Sample {
                        planet: r.planet,
                        at: parse_ms(&r.at),
                        owner,
                        percent: r.percent,
                        players: r.players,
                        defending: r.defending,
                    };
                    keep(&mut self.recent, s.planet, s);
                }
            }
            if n < PAGE {
                break;
            }
            offset += PAGE;
        }
        Ok(())
    }

    pub fn known_planets(&self) -> usize {
        self.owners.len()
    }

    pub fn rate(&self, planet: i64) -> Option<f64> {
        rate_per_hour(self.recent.get(&planet).map_or(&[], Vec::as_slice))
    }

    pub fn region_rate(&self, planet: i64, region: i64) -> Option<f64> {
        rate_per_hour(self.regions.get(&(planet, region)).map_or(&[], Vec::as_slice))
    }

    pub fn ingest(
        &mut self,
        db: &Supabase,
        planets: &[Planet],
        campaign: &HashSet<i64>,
        at: f64,
    ) -> Result<IngestResult, String> {
        let changes = detect_owner_changes(&self.owners, planets, at);
        let updated: Vec<&Planet> =
            planets.iter().filter(|p| self.owners.get(&p.index) != Some(&p.current_owner)).collect();
        let owner_rows: Vec<Value> =
            updated.iter().map(|p| json!({ "planet": p.index, "owner": faction_id(p.current_owner) })).collect();
        let change_rows: Vec<Value> = changes
            .iter()
            .map(|c| {
                json!({
                    "planet": c.planet,
                    "planet_name": c.planet_name,
                    "from_owner": faction_id(c.from),
                    "to_owner": faction_id(c.to),
                    "at": iso_string(c.at),
                })
            })
            .collect();
        // New owners are kept only once stored, so a failed write is retried next poll. planet_owners goes first:
        // it is safe to write twice, an owner change isn't.
        db.upsert("planet_owners", &owner_rows, None, false)?;
        db.insert("owner_changes", &change_rows)?;
        self.owners.extend(updated.iter().map(|p| (p.index, p.current_owner)));

        // A planet that just changed hands gets one sample even if it's no longer contested,
        // so its chart ends at the moment of capture. A campaign still at 0% is sampled too, or its rate
        // would never be measured.
        let changed: HashSet<i64> = changes.iter().map(|c| c.planet).collect();
        let contested = |p: &Planet| is_active(p) || changed.contains(&p.index) || campaign.contains(&p.index);
        let samples: Vec<Sample> = planets.iter().filter(|p| contested(p)).map(|p| to_sample(p, at)).collect();
        for s in &samples {
            keep(&mut self.recent, s.planet, s.clone());
        }
        let idle: Vec<Value> = planets
            .iter()
            .filter(|p| !contested(p) && p.statistics.player_count > 0.0)
            .map(|p| json!({ "planet": p.index, "at": iso_string(at), "players": p.statistics.player_count }))
            .collect();
        let mut cities = Vec::new();
        for p in planets {
            for r in &p.regions {
                let key = (p.index, r.id);
                if let Some(s) = region_sample(p, r, at) {
                    keep(&mut self.regions, key, s);
                }
                if let Some(row) = region_row(p, r, at)
                    && at - self.region_written.get(&key).copied().unwrap_or(f64::NEG_INFINITY) >= REGION_EVERY_MS
                {
                    cities.push(row);
                    self.region_written.insert(key, at);
                }
            }
        }

        db.upsert("samples", &samples.iter().map(to_row).collect::<Vec<_>>(), None, true)?;
        unless_missing(db.upsert("planet_players", &idle, None, true))?;
        unless_missing(db.upsert("region_samples", &cities, None, true))?;
        Ok(IngestResult { samples: samples.len(), changes })
    }
}

/// Typed rows from a select; null or missing fields take their defaults.
pub fn rows<T: serde::de::DeserializeOwned>(list: Vec<Value>) -> Result<Vec<T>, String> {
    warroom_core::api::from_value(Value::Array(list)).map_err(|e| e.to_string())
}
