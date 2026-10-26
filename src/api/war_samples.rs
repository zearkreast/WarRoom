use crate::api::cache::{Query, use_query};
use crate::api::queries::queries;
use crate::api::source::{self, encode};
use crate::config::POLL_MS;
use leptos::prelude::*;
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::front_activity::{FrontBucket, FrontRow, front_buckets, latest_run};
use warroom_core::galaxy::sector_territory;
use warroom_core::snapshot::WarSampleRow;
use warroom_core::time::{iso_string, parse_ms};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarSample {
    pub at: f64,
    pub impact: f64,
    pub players: f64,
    pub terminids: f64,
    pub automatons: f64,
    pub illuminate: f64,
}

const DAY_MS: f64 = 86_400_000.0;

fn rows<T: serde::de::DeserializeOwned>(
    r: Result<Vec<serde_json::Value>, source::SourceError>,
) -> Result<Vec<T>, String> {
    match r {
        // A table the backend doesn't have yet reads as empty.
        Err(e) if e.status == 404 => Ok(Vec::new()),
        Err(e) => Err(e.message),
        Ok(list) => warroom_core::api::from_value(serde_json::Value::Array(list)).map_err(|e| e.to_string()),
    }
}

/// The last 24 hours of galaxy-wide samples, ending at the newest data rather than the clock.
/// Empty while the `war_samples` table doesn't exist yet (older backends).
pub fn use_war_samples() -> Query<Arc<Vec<WarSample>>> {
    let last = queries().last_update;
    use_query(
        move || last.get().map(|end| format!("war_samples:{}:{end}", (end / POLL_MS).floor())),
        POLL_MS,
        |key: String| async move {
            let end: f64 = key.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let since = encode(&iso_string(end - DAY_MS));
            let list: Vec<WarSampleRow> =
                rows(source::select("war_samples", &format!("select=*&at=gte.{since}&order=at.asc&limit=2000")).await)?;
            Ok(Arc::new(
                list.into_iter()
                    .map(|r| WarSample {
                        at: parse_ms(&r.at),
                        impact: r.impact,
                        players: r.players,
                        terminids: r.terminids,
                        automatons: r.automatons,
                        illuminate: r.illuminate,
                    })
                    .collect(),
            ))
        },
    )
}

/// Helldivers per front over the last 24 hours, or since the latest gap in the data. Summed in the
/// database from planet samples the collector has always kept, so there is no warm-up. A defense counts
/// for its attacker, or the enemy holding most of its sector.
pub fn use_front_activity() -> (Memo<Option<Arc<Vec<FrontBucket>>>>, ReadSignal<bool>) {
    let last = queries().last_update;
    let query = use_query(
        move || last.get().map(|end| format!("front_activity:{}", (end / POLL_MS).floor())),
        POLL_MS,
        |_key: String| async move {
            let list: Vec<FrontRow> = rows(source::rpc("front_activity", &[("hours", "24".into())]).await)?;
            Ok(Arc::new(list))
        },
    );
    let planets = queries().fresh_planets;
    let data = Memo::new(move |_| {
        let rows = query.data.get()?;
        let planets = planets.get()?;
        let territory = sector_territory(&planets);
        let attacker = |i: i64| -> Option<Faction> {
            let p = planets.iter().find(|p| p.index == i)?;
            p.event
                .as_ref()
                .map(|e| e.faction)
                .or_else(|| territory.iter().find(|(s, _)| *s == p.sector).and_then(|(_, t)| t.map(|t| t.faction)))
        };
        Some(Arc::new(latest_run(&front_buckets(&rows, attacker)).to_vec()))
    });
    (data, query.loading)
}
