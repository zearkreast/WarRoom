//! The hourly readings of the game's mission and kill counters that the collector keeps.

use crate::api::source::{self, encode};
use warroom_core::fighting::CombatReading;
use warroom_core::time::{iso_string, parse_ms};

#[derive(serde::Deserialize)]
struct Row {
    at: String,
    missions_won: f64,
    missions_lost: f64,
    terminid_kills: f64,
    automaton_kills: f64,
    illuminate_kills: f64,
    deaths: f64,
}

/// A planet's readings (or the galaxy's, for `GALAXY`) from `since_ms` on, oldest first; None while the store lacks
/// the table.
pub async fn combat_readings(planet: i64, since_ms: f64) -> Result<Option<Vec<CombatReading>>, String> {
    let since = encode(&iso_string(since_ms));
    let query = format!("select=*&planet=eq.{planet}&at=gte.{since}&order=at.asc&limit=200");
    let rows = match source::select("combat_stats", &query).await {
        Err(e) if e.status == 404 => return Ok(None),
        Err(e) => return Err(e.message),
        Ok(rows) => rows,
    };
    let rows: Vec<Row> = warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())?;
    Ok(Some(
        rows.into_iter()
            .map(|r| CombatReading {
                at: parse_ms(&r.at),
                missions_won: r.missions_won,
                missions_lost: r.missions_lost,
                terminid_kills: r.terminid_kills,
                automaton_kills: r.automaton_kills,
                illuminate_kills: r.illuminate_kills,
                deaths: r.deaths,
            })
            .collect(),
    ))
}
