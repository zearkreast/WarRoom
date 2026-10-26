//! Reads from Supabase's REST API with the publishable key. Inside Tauri, requests go through Rust
//! (no CORS, same code on every platform).

use crate::config::{SUPABASE_PUBLISHABLE_KEY, SUPABASE_URL};
use crate::js::{self, get, obj};
use crate::tauri;
use serde_json::Value;
use warroom_core::snapshot::SnapshotKind;
use wasm_bindgen::JsValue;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceError {
    pub status: u16,
    pub message: String,
}

impl SourceError {
    fn new(status: u16, message: impl Into<String>) -> SourceError {
        SourceError { status, message: message.into() }
    }
}

#[derive(Clone, Debug)]
pub struct SnapshotRow {
    pub kind: SnapshotKind,
    pub data: Value,
    pub fetched_at: String,
}

const REQUEST_TIMEOUT_MS: f64 = 20_000.0;

pub fn encode(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_default()
}

/// GET a URL as JSON, through Tauri's HTTP client in the app and the browser's fetch on the web.
pub async fn fetch_json(url: &str, headers: &[(&str, &str)]) -> Result<Value, SourceError> {
    let head = js_sys::Object::new();
    for (k, v) in headers {
        let _ = js_sys::Reflect::set(&head, &JsValue::from_str(k), &JsValue::from_str(v));
    }
    let ctrl = web_sys::AbortController::new().ok();
    let init = obj(&[("method", "GET".into()), ("headers", head.into())]);
    if let Some(c) = &ctrl {
        let _ = js_sys::Reflect::set(&init, &"signal".into(), &c.signal());
    }
    let timer = ctrl.clone().map(|c| {
        leptos::prelude::set_timeout_with_handle(
            move || c.abort(),
            std::time::Duration::from_millis(REQUEST_TIMEOUT_MS as u64),
        )
    });
    let sent = if tauri::is_tauri() {
        tauri::call_api("http", "fetch", &[JsValue::from_str(url), init]).await
    } else {
        js::wait(js::call(&js::window().into(), "fetch", &[JsValue::from_str(url), init]).unwrap_or(JsValue::UNDEFINED))
            .await
    };
    if let Some(Ok(t)) = timer {
        t.clear();
    }
    let res = sent.map_err(|e| SourceError::new(0, tauri::error_text(&e)))?;
    let status = get(&res, "status").as_f64().unwrap_or(0.0) as u16;
    let text = js::wait(js::call(&res, "text", &[]).map_err(|e| SourceError::new(status, tauri::error_text(&e)))?)
        .await
        .map_err(|e| SourceError::new(status, tauri::error_text(&e)))?
        .as_string()
        .unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(SourceError::new(status, format!("HTTP {status} for {url}")));
    }
    serde_json::from_str(&text).map_err(|e| SourceError::new(status, e.to_string()))
}

/// Raw PostgREST select, for history tables.
pub async fn select(table: &str, query: &str) -> Result<Vec<Value>, SourceError> {
    #[cfg(debug_assertions)]
    if fixtures_mode() {
        return Ok(Vec::new());
    }
    let url = format!("{}/rest/v1/{table}?{query}", SUPABASE_URL.trim_end_matches('/'));
    match fetch_json(&url, &[("apikey", SUPABASE_PUBLISHABLE_KEY), ("Accept", "application/json")]).await? {
        Value::Array(rows) => Ok(rows),
        other => Ok(vec![other]),
    }
}

/// A database function's rows, called with its named arguments.
pub async fn rpc(function: &str, args: &[(&str, String)]) -> Result<Vec<Value>, SourceError> {
    let query: Vec<String> = args.iter().map(|(k, v)| format!("{k}={}", encode(v))).collect();
    select(&format!("rpc/{function}"), &query.join("&")).await
}

/// Snapshots written after `since` (ISO time), or all of them when `since` is empty.
pub async fn snapshots(since: &str) -> Result<Vec<SnapshotRow>, SourceError> {
    #[cfg(debug_assertions)]
    if fixtures_mode() {
        return Ok(if since.is_empty() { fixtures::rows() } else { Vec::new() });
    }
    let filter = if since.is_empty() { String::new() } else { format!("&fetched_at=gt.{}", encode(since)) };
    let rows = select("snapshots", &format!("select=kind,data,fetched_at{filter}")).await?;
    Ok(rows
        .into_iter()
        .filter_map(|mut r| {
            let kind: SnapshotKind = serde_json::from_value(r.get("kind")?.clone()).ok()?;
            Some(SnapshotRow {
                kind,
                data: r.get_mut("data").map(Value::take).unwrap_or(Value::Null),
                fetched_at: r.get("fetched_at")?.as_str()?.to_string(),
            })
        })
        .collect())
}

/// Dev only: `?fixtures` serves the saved test fixtures, shaped like the collector's snapshots.
#[cfg(debug_assertions)]
pub fn fixtures_mode() -> bool {
    js::window().location().search().is_ok_and(|s| s.contains("fixtures"))
}

#[cfg(debug_assertions)]
mod fixtures {
    use super::SnapshotRow;
    use serde_json::Value;
    use warroom_core::api::{Campaign, Dispatch, Planet, SpaceStation, WarStatusRaw, from_str, js_value};
    use warroom_core::snapshot::*;

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!("../../crates/core/tests/fixtures/", $name))
        };
    }

    fn value<T: serde::Serialize>(v: T) -> Value {
        js_value(serde_json::to_value(v).unwrap())
    }

    pub fn rows() -> Vec<SnapshotRow> {
        let at = warroom_core::time::iso_string(crate::js::now());
        let planets: Vec<Planet> = from_str(fixture!("planets.json")).unwrap();
        let campaigns: Vec<Campaign> = from_str(fixture!("campaigns.json")).unwrap();
        let dispatches: Vec<Dispatch> = from_str(fixture!("dispatches.json")).unwrap();
        let stations: Vec<SpaceStation> = from_str(fixture!("space-stations.json")).unwrap();
        let status: WarStatusRaw = from_str(fixture!("status-effects.json")).unwrap();
        let episodes: Value = serde_json::from_str(fixture!("episodes.json")).unwrap();
        let raw: Vec<warroom_core::api::EpisodeRaw> =
            warroom_core::api::from_value(episodes["episodes"].clone()).unwrap();
        let (war_time, fetched) = (episodes["warTime"].as_f64().unwrap(), episodes["fetchedAt"].as_f64().unwrap());
        let row = |kind, data| SnapshotRow { kind, data, fetched_at: at.clone() };
        vec![
            row(SnapshotKind::War, serde_json::from_str(fixture!("war.json")).unwrap()),
            row(SnapshotKind::Assignments, serde_json::from_str(fixture!("assignments.json")).unwrap()),
            row(
                SnapshotKind::Campaigns,
                value(campaigns.iter().map(|c| to_campaign_live(c, None, |_| None)).collect::<Vec<_>>()),
            ),
            row(SnapshotKind::Planets, value(planets.iter().map(to_live).collect::<Vec<_>>())),
            row(SnapshotKind::PlanetsStatic, value(planets.iter().map(to_static).collect::<Vec<_>>())),
            row(SnapshotKind::Dispatches, value(newest_dispatches(&dispatches))),
            row(SnapshotKind::SpaceStations, value(stations.iter().map(to_station_live).collect::<Vec<_>>())),
            row(SnapshotKind::PlanetEffects, value(to_effects_live(&status))),
            row(SnapshotKind::Episodes, value(merge_episodes(&[], &to_episodes(&raw, war_time, fetched), 0.0))),
        ]
    }
}
