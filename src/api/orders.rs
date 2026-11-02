//! The Major Order archive the collector keeps: every order seen, and its progress at each poll.

use crate::api::source;
use serde::Deserialize;
use warroom_core::api::{Assignment, AssignmentTask};
use warroom_core::snapshot::OrderSample;
use warroom_core::time::parse_ms;

#[derive(Deserialize)]
struct OrderRow {
    id: i64,
    title: Option<String>,
    briefing: Option<String>,
    expires: String,
    tasks: Vec<AssignmentTask>,
    progress: Vec<f64>,
    first_seen: String,
}

#[derive(Deserialize)]
struct ProgressRow {
    at: String,
    progress: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchivedOrder {
    /// As the API gave it at its last reading.
    pub order: Assignment,
    pub first_seen: f64,
}

/// PostgREST's most rows per answer.
const PAGE: usize = 1000;

/// Every row of `query`, a page at a time; None while the store lacks the table.
async fn read_all<T: serde::de::DeserializeOwned>(table: &str, query: &str) -> Result<Option<Vec<T>>, String> {
    let mut out = Vec::new();
    loop {
        let page = match source::select(table, &format!("{query}&limit={PAGE}&offset={}", out.len())).await {
            Err(e) if e.status == 404 => return Ok(None),
            Err(e) => return Err(e.message),
            Ok(rows) => rows,
        };
        let full = page.len() == PAGE;
        let rows: Vec<T> = warroom_core::api::from_value(serde_json::Value::Array(page)).map_err(|e| e.to_string())?;
        out.extend(rows);
        if !full {
            return Ok(Some(out));
        }
    }
}

/// Every order on record, latest deadline first.
pub async fn archived_orders() -> Result<Option<Vec<ArchivedOrder>>, String> {
    let rows: Option<Vec<OrderRow>> = read_all("orders", "select=*&order=expires.desc").await?;
    Ok(rows.map(|rows| {
        rows.into_iter()
            .map(|r| ArchivedOrder {
                first_seen: parse_ms(&r.first_seen),
                order: Assignment {
                    id: r.id,
                    progress: r.progress,
                    title: r.title,
                    briefing: r.briefing,
                    tasks: r.tasks,
                    expiration: r.expires,
                    ..Assignment::default()
                },
            })
            .collect()
    }))
}

/// One order's progress readings, oldest first.
pub async fn order_progress(id: i64) -> Result<Option<Vec<OrderSample>>, String> {
    let rows: Option<Vec<ProgressRow>> =
        read_all("order_progress", &format!("select=at,progress&id=eq.{id}&order=at.asc")).await?;
    Ok(rows
        .map(|rows| rows.into_iter().map(|r| OrderSample { id, at: parse_ms(&r.at), progress: r.progress }).collect()))
}
