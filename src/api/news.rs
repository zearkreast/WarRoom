//! The dispatch archive the collector keeps, for Strohmann News past the newest 50.

use crate::api::source::{self, encode};
use warroom_core::api::Dispatch;

/// Rows of `query` as dispatches; None while the store lacks the table.
async fn read(query: &str) -> Result<Option<Vec<Dispatch>>, String> {
    let rows = match source::select("dispatches", query).await {
        Err(e) if e.status == 404 => return Ok(None),
        Err(e) => return Err(e.message),
        Ok(rows) => rows,
    };
    #[derive(serde::Deserialize)]
    struct Row {
        id: i64,
        published: String,
        kind: i64,
        message: String,
    }
    let rows: Vec<Row> = warroom_core::api::from_value(serde_json::Value::Array(rows)).map_err(|e| e.to_string())?;
    Ok(Some(
        rows.into_iter()
            .map(|r| Dispatch { id: r.id, published: r.published, kind: r.kind, message: r.message })
            .collect(),
    ))
}

/// Up to `limit` dispatches older than `before_id`, newest first.
pub async fn older_dispatches(before_id: i64, limit: usize) -> Result<Option<Vec<Dispatch>>, String> {
    read(&format!("select=*&id=lt.{before_id}&order=id.desc&limit={limit}")).await
}

/// Up to `limit` dispatches whose text holds every word, newest first.
pub async fn search_dispatches(words: &[String], limit: usize) -> Result<Option<Vec<Dispatch>>, String> {
    // Quoted, so commas and brackets in a word don't break the filter's syntax.
    let terms: Vec<String> =
        words.iter().map(|w| format!("message.ilike.\"*{}*\"", w.replace(['"', '\\'], ""))).collect();
    read(&format!("select=*&and=({})&order=id.desc&limit={limit}", encode(&terms.join(",")))).await
}
