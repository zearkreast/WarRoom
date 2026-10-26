use crate::api::cache::{Query, use_query};
use crate::api::queries::queries;
use crate::api::source;
use crate::config::POLL_MS;
use leptos::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use warroom_core::time::parse_ms;

#[derive(Clone, Debug, PartialEq)]
pub struct StationJump {
    pub station: i64,
    pub at: f64,
    pub from: Option<i64>,
    pub to: i64,
}

#[derive(Deserialize)]
struct Row {
    station: i64,
    at: String,
    from_planet: Option<i64>,
    to_planet: i64,
}

/// A station's FTL jumps, newest first, as the collector logged them. Empty while the `station_jumps`
/// table doesn't exist yet (older backends).
pub fn use_station_jumps(station: impl Fn() -> i64 + 'static) -> Query<Arc<Vec<StationJump>>> {
    let last = queries().last_update;
    use_query(
        move || {
            let l = last.get()?;
            Some(format!("station_jumps:{}:{}", station(), ((l) / (POLL_MS * 5.0)).floor()))
        },
        POLL_MS * 5.0,
        |key: String| async move {
            let station = key.split(':').nth(1).unwrap_or("0").to_string();
            let rows = match source::select("station_jumps", &format!("select=*&station=eq.{station}&order=at.desc&limit=50")).await {
                Err(e) if e.status == 404 => Vec::new(),
                Err(e) => return Err(e.message),
                Ok(rows) => rows,
            };
            let rows: Vec<Row> = warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())?;
            Ok(Arc::new(
                rows.into_iter()
                    .map(|r| StationJump { station: r.station, at: parse_ms(&r.at), from: r.from_planet, to: r.to_planet })
                    .collect(),
            ))
        },
    )
}
