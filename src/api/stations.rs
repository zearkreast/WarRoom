use crate::api::cache::{Query, use_query};
use crate::api::queries::queries;
use crate::api::source;
use crate::api::source::encode;
use crate::config::POLL_MS;
use leptos::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use warroom_core::station_actions::ActionChange;
use warroom_core::time::{iso_string, parse_ms};

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

/// How many of a station's newest jumps are read.
pub const JUMP_LIMIT: usize = 50;

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
            let rows = match source::select(
                "station_jumps",
                &format!("select=*&station=eq.{station}&order=at.desc&limit={JUMP_LIMIT}"),
            )
            .await
            {
                Err(e) if e.status == 404 => Vec::new(),
                Err(e) => return Err(e.message),
                Ok(rows) => rows,
            };
            let rows: Vec<Row> =
                warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())?;
            Ok(Arc::new(
                rows.into_iter()
                    .map(|r| StationJump {
                        station: r.station,
                        at: parse_ms(&r.at),
                        from: r.from_planet,
                        to: r.to_planet,
                    })
                    .collect(),
            ))
        },
    )
}

#[derive(Deserialize)]
struct ActionRow {
    action: i64,
    at: String,
    status: i64,
}

/// How far back a station's action log is read.
pub const ACTION_DAYS: f64 = 30.0;

/// A station's tactical action changes over the last 30 days, oldest first, and when its log began.
pub struct ActionLog {
    pub changes: Vec<ActionChange>,
    /// The station's first logged change ever, epoch ms; None while nothing is logged.
    pub since: Option<f64>,
}

#[derive(Deserialize)]
struct AtRow {
    at: String,
}

/// A station's action log. None while the `station_actions` table doesn't exist yet.
pub fn use_action_changes(station: impl Fn() -> i64 + 'static) -> Query<Option<Arc<ActionLog>>> {
    let last = queries().last_update;
    use_query(
        move || {
            let l = last.get()?;
            Some(format!("station_actions:{}:{}", station(), (l / (POLL_MS * 5.0)).floor()))
        },
        POLL_MS * 5.0,
        |key: String| async move {
            let mut parts = key.split(':').skip(1);
            let station = parts.next().unwrap_or("0").to_string();
            let end = parts.next().and_then(|b| b.parse::<f64>().ok()).unwrap_or(0.0) * POLL_MS * 5.0;
            let since = encode(&iso_string(end - ACTION_DAYS * 86_400_000.0));
            let rows = match source::select(
                "station_actions",
                &format!("select=action,at,status&station=eq.{station}&at=gte.{since}&order=at.asc&limit=1000"),
            )
            .await
            {
                Err(e) if e.status == 404 => return Ok(None),
                Err(e) => return Err(e.message),
                Ok(rows) => rows,
            };
            let rows: Vec<ActionRow> =
                warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())?;
            // The window can't tell a young log from a quiet month, so the first row ever is read too.
            let first =
                source::select("station_actions", &format!("select=at&station=eq.{station}&order=at.asc&limit=1"))
                    .await
                    .map_err(|e| e.message)?;
            let first: Vec<AtRow> =
                warroom_core::api::from_value(serde_json::Value::Array(first)).map_err(|e| e.to_string())?;
            Ok(Some(Arc::new(ActionLog {
                changes: rows
                    .into_iter()
                    .map(|r| ActionChange { action: r.action, at: parse_ms(&r.at), status: r.status })
                    .collect(),
                since: first.first().map(|r| parse_ms(&r.at)),
            })))
        },
    )
}
