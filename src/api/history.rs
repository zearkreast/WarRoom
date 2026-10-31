//! Read side of the shared war history in Supabase: what the collector writes, and the war's earlier history
//! imported before it ran. Everyone sees the same history.

use crate::api::source::{self, encode};
use leptos::prelude::*;
use serde::Deserialize;
use std::sync::OnceLock;
use warroom_core::cities::CityProgress;
use warroom_core::factions::faction_by_id;
use warroom_core::owner_changes::OwnerChange;
use warroom_core::progress::Sample;
use warroom_core::time::{iso_string, parse_ms};

/// An ownership change as stored by the collector; `id` increases with every new row.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeRecord {
    pub id: i64,
    pub change: OwnerChange,
}

#[derive(Deserialize)]
struct SampleRow {
    planet: i64,
    at: String,
    owner: i64,
    percent: f64,
    players: f64,
    defending: bool,
}

#[derive(Deserialize)]
struct ChangeRow {
    id: i64,
    planet: i64,
    planet_name: String,
    from_owner: i64,
    to_owner: i64,
    at: String,
}

#[derive(Deserialize)]
struct AtRow {
    at: String,
}

fn typed<T: serde::de::DeserializeOwned>(
    r: Result<Vec<serde_json::Value>, source::SourceError>,
) -> Result<Vec<T>, String> {
    let rows = r.map_err(|e| e.message)?;
    warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())
}

/// Tables added by a later migration read as empty until it's applied.
fn unless_missing<T: serde::de::DeserializeOwned>(
    r: Result<Vec<serde_json::Value>, source::SourceError>,
) -> Result<Vec<T>, String> {
    match r {
        Err(e) if e.status == 404 => Ok(Vec::new()),
        r => typed(r),
    }
}

/// Helldivers somewhere at a moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerReading {
    pub at: f64,
    pub players: f64,
}

/// A city's reading: its Helldivers, and its liberation while it's fought over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityReading {
    pub at: f64,
    pub players: f64,
    pub percent: Option<f64>,
}

#[derive(Deserialize)]
struct ReadingRow {
    at: String,
    players: f64,
    percent: Option<f64>,
}

#[derive(Deserialize)]
struct ProgressRow {
    region: i64,
    at: String,
    percent: f64,
}

fn to_change(r: ChangeRow) -> Option<ChangeRecord> {
    Some(ChangeRecord {
        id: r.id,
        change: OwnerChange {
            planet: r.planet,
            planet_name: r.planet_name,
            from: faction_by_id(r.from_owner)?,
            to: faction_by_id(r.to_owner)?,
            at: parse_ms(&r.at),
        },
    })
}

/// PostgREST caps responses at 1000 rows.
const SAMPLE_LIMIT: usize = 1000;

/// One page of `filter` in time order: from `since` on the first page, then after `after`, the last time read.
fn page_query(filter: &str, since: Option<&str>, after: Option<&str>) -> String {
    let bound = match (after, since) {
        (Some(t), _) => format!("&at=gt.{t}"),
        (None, Some(t)) => format!("&at=gte.{t}"),
        (None, None) => String::new(),
    };
    format!("{filter}{bound}&order=at.asc&limit={SAMPLE_LIMIT}")
}

/// The time the next page starts after, or None once a short page shows there's no more.
fn next_after(page: &[serde_json::Value]) -> Option<&str> {
    if page.len() < SAMPLE_LIMIT {
        return None;
    }
    page.last()?.get("at")?.as_str()
}

/// Every row of `filter` from `since_ms` on, oldest first, a page at a time.
async fn select_all(
    table: &str,
    filter: &str,
    since_ms: Option<f64>,
) -> Result<Vec<serde_json::Value>, source::SourceError> {
    let since = since_ms.map(|ms| encode(&iso_string(ms)));
    let mut rows = Vec::new();
    let mut after = None;
    loop {
        let page = source::select(table, &page_query(filter, since.as_deref(), after.as_deref())).await?;
        after = next_after(&page).map(encode);
        rows.extend(page);
        if after.is_none() {
            return Ok(rows);
        }
    }
}

pub async fn samples(planet: i64, since_ms: f64) -> Result<Vec<Sample>, String> {
    let rows: Vec<SampleRow> =
        typed(select_all("samples", &format!("select=*&planet=eq.{planet}"), Some(since_ms)).await)?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(Sample {
                planet: r.planet,
                at: parse_ms(&r.at),
                owner: faction_by_id(r.owner)?,
                percent: r.percent,
                players: r.players,
                defending: r.defending,
            })
        })
        .collect())
}

/// Helldivers on the planet while it had no campaign, oldest first.
pub async fn idle_players(planet: i64, since_ms: f64) -> Result<Vec<PlayerReading>, String> {
    let rows: Vec<ReadingRow> = unless_missing(
        select_all("planet_players", &format!("select=at,players&planet=eq.{planet}"), Some(since_ms)).await,
    )?;
    Ok(rows.into_iter().map(|r| PlayerReading { at: parse_ms(&r.at), players: r.players }).collect())
}

/// One city's readings, oldest first.
pub async fn city_readings(planet: i64, region: i64, since_ms: f64) -> Result<Vec<CityReading>, String> {
    let since = encode(&iso_string(since_ms));
    let query = format!(
        "select=at,players,percent&planet=eq.{planet}&region=eq.{region}&at=gte.{since}&order=at.desc&limit={SAMPLE_LIMIT}"
    );
    let rows: Vec<ReadingRow> = unless_missing(source::select("region_samples", &query).await)?;
    Ok(rows
        .into_iter()
        .rev()
        .map(|r| CityReading { at: parse_ms(&r.at), players: r.players, percent: r.percent })
        .collect())
}

/// The liberation of a planet's cities while they were fought over, oldest first.
pub async fn city_progress(planet: i64, since_ms: f64) -> Result<Vec<CityProgress>, String> {
    let filter = format!("select=region,at,percent&planet=eq.{planet}&percent=not.is.null");
    let rows: Vec<ProgressRow> = unless_missing(select_all("region_samples", &filter, Some(since_ms)).await)?;
    Ok(rows.into_iter().map(|r| CityProgress { region: r.region, at: parse_ms(&r.at), percent: r.percent }).collect())
}

/// Time of the oldest stored sample for a planet (or for any planet), or None if none.
pub async fn earliest(planet: Option<i64>) -> Result<Option<f64>, String> {
    let filter = planet.map(|p| format!("&planet=eq.{p}")).unwrap_or_default();
    let rows: Vec<AtRow> = typed(source::select("samples", &format!("select=at{filter}&order=at.asc&limit=1")).await)?;
    Ok(rows.first().map(|r| parse_ms(&r.at)))
}

/// Times of samples taken while the planet was under attack, oldest first.
pub async fn defending_times(planet: i64) -> Result<Vec<f64>, String> {
    let rows: Vec<AtRow> =
        typed(select_all("samples", &format!("select=at&planet=eq.{planet}&defending=is.true"), None).await)?;
    Ok(rows.iter().map(|r| parse_ms(&r.at)).collect())
}

pub async fn changes(planet: Option<i64>, limit: usize) -> Result<Vec<ChangeRecord>, String> {
    let filter = planet.map(|p| format!("&planet=eq.{p}")).unwrap_or_default();
    let rows: Vec<ChangeRow> =
        typed(source::select("owner_changes", &format!("select=*{filter}&order=id.desc&limit={limit}")).await)?;
    Ok(rows.into_iter().filter_map(to_change).collect())
}

/// Changes newer than `after_id`, newest first.
pub async fn changes_after(after_id: i64, limit: usize) -> Result<Vec<ChangeRecord>, String> {
    let rows: Vec<ChangeRow> = typed(
        source::select("owner_changes", &format!("select=*&id=gt.{after_id}&order=id.desc&limit={limit}")).await,
    )?;
    Ok(rows.into_iter().filter_map(to_change).collect())
}

/// Changes older than `before_id`, newest first.
async fn changes_before(before_id: i64, limit: usize) -> Result<Vec<ChangeRecord>, String> {
    let rows: Vec<ChangeRow> = typed(
        source::select("owner_changes", &format!("select=*&id=lt.{before_id}&order=id.desc&limit={limit}")).await,
    )?;
    Ok(rows.into_iter().filter_map(to_change).collect())
}

const KEEP: usize = 100;

/// Puts `fresh` ahead of `held`, keeping as many as were held (at least KEEP); returns whether any fell off the end.
fn prepend(held: &mut Vec<ChangeRecord>, mut fresh: Vec<ChangeRecord>) -> bool {
    // A full page of new changes may not reach back to the held ones.
    let keep = if fresh.len() < KEEP { held.len().max(KEEP) } else { KEEP };
    fresh.append(held);
    let dropped = fresh.len() > keep;
    fresh.truncate(keep);
    *held = fresh;
    dropped
}

#[derive(Clone, Copy)]
pub struct History {
    pub ready: RwSignal<bool>,
    /// Most recent ownership changes, newest first.
    pub changes: RwSignal<Vec<ChangeRecord>>,
    /// Whether the store may hold changes older than the oldest in `changes`.
    pub older: RwSignal<bool>,
}

static HISTORY: OnceLock<History> = OnceLock::new();

pub fn init() {
    let _ = HISTORY.set(History {
        ready: RwSignal::new(false),
        changes: RwSignal::new(Vec::new()),
        older: RwSignal::new(false),
    });
}

pub fn history() -> History {
    *HISTORY.get().expect("history initialised")
}

impl History {
    /// Loads anything newer than what's already held. Returns only the new rows, and none
    /// on the first call, so alerts don't fire for history the user hasn't missed.
    pub async fn refresh(&self) -> Result<Vec<ChangeRecord>, String> {
        if !self.ready.get_untracked() {
            let list = changes(None, KEEP).await?;
            self.older.set(list.len() == KEEP);
            self.changes.set(list);
            self.ready.set(true);
            return Ok(Vec::new());
        }
        let newest = self.changes.with_untracked(|c| c.first().map_or(0, |c| c.id));
        let fresh = changes_after(newest, KEEP).await?;
        if !fresh.is_empty() {
            let mut dropped = false;
            self.changes.update(|c| dropped = prepend(c, fresh.clone()));
            if dropped {
                self.older.set(true);
            }
        }
        Ok(fresh)
    }

    /// Appends the page of changes before the oldest held.
    pub async fn load_older(&self) -> Result<(), String> {
        let Some(oldest) = self.changes.with_untracked(|c| c.last().map(|c| c.id)) else { return Ok(()) };
        let page = changes_before(oldest, KEEP).await?;
        let full = page.len() == KEEP;
        let mut joined = false;
        // A refresh meanwhile may have dropped the oldest; appending then would leave a gap.
        self.changes.update(|c| {
            joined = c.last().map(|c| c.id) == Some(oldest);
            if joined {
                c.extend(page);
            }
        });
        if joined {
            self.older.set(full);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use warroom_core::api::Faction;

    #[test]
    fn pages_forward_in_time_from_the_window_start() {
        let first = page_query("select=at&planet=eq.5", Some("T0"), None);
        assert_eq!(first, "select=at&planet=eq.5&at=gte.T0&order=at.asc&limit=1000");
        let next = page_query("select=at&planet=eq.5", Some("T0"), Some("T9"));
        assert_eq!(next, "select=at&planet=eq.5&at=gt.T9&order=at.asc&limit=1000");
        assert_eq!(page_query("select=at", None, None), "select=at&order=at.asc&limit=1000");
    }

    #[test]
    fn the_next_page_starts_after_a_full_page_only() {
        let page = |n: usize| (0..n).map(|i| json!({ "at": format!("t{i}") })).collect::<Vec<_>>();
        assert_eq!(next_after(&page(1000)), Some("t999"));
        assert_eq!(next_after(&page(999)), None);
        assert_eq!(next_after(&[]), None);
    }

    fn records(ids: impl DoubleEndedIterator<Item = i64>) -> Vec<ChangeRecord> {
        ids.rev()
            .map(|id| ChangeRecord {
                id,
                change: OwnerChange {
                    planet: id,
                    planet_name: format!("P{id}"),
                    from: Faction::Humans,
                    to: Faction::Automaton,
                    at: id as f64,
                },
            })
            .collect()
    }

    fn ids(c: &[ChangeRecord]) -> (i64, i64, usize) {
        (c[0].id, c[c.len() - 1].id, c.len())
    }

    #[test]
    fn new_changes_slide_the_window_but_keep_older_pages() {
        let mut held = records(1..=60);
        assert!(!prepend(&mut held, records(61..=63)));
        assert_eq!(ids(&held), (63, 1, 63));

        let mut held = records(1..=100);
        assert!(prepend(&mut held, records(101..=103)));
        assert_eq!(ids(&held), (103, 4, 100));

        let mut held = records(1..=300);
        assert!(prepend(&mut held, records(301..=302)));
        assert_eq!(ids(&held), (302, 3, 300));
    }

    #[test]
    fn a_full_page_of_new_changes_replaces_what_was_held() {
        let mut held = records(1..=300);
        assert!(prepend(&mut held, records(401..=500)));
        assert_eq!(ids(&held), (500, 401, 100));
    }
}
