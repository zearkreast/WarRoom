//! Minimal Supabase REST (PostgREST) client: just the calls the collector makes.

use crate::http::{Request, Transport};
use serde_json::Value;

const TIMEOUT_MS: u64 = 30_000;

pub struct Supabase {
    base: String,
    /// The project's secret key (sb_secret_…). It bypasses RLS, so it stays on the server.
    key: String,
    transport: Box<dyn Transport>,
}

impl Supabase {
    pub fn new(url: &str, key: &str, transport: Box<dyn Transport>) -> Supabase {
        Supabase { base: format!("{}/rest/v1", url.trim_end_matches('/')), key: key.into(), transport }
    }

    fn call(
        &self,
        method: &'static str,
        path: &str,
        body: Option<&Value>,
        prefer: Option<String>,
    ) -> Result<Value, String> {
        let mut headers = vec![("apikey", self.key.clone()), ("Content-Type", "application/json".to_string())];
        if let Some(p) = prefer {
            headers.push(("Prefer", p));
        }
        let res = self
            .transport
            .send(Request {
                method,
                url: format!("{}/{path}", self.base),
                headers,
                body: body.map(|b| warroom_core::api::js_value(b.clone()).to_string()),
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("Supabase {method} {} → {e}", path.split('?').next().unwrap_or("")))?;
        if !res.ok() {
            return Err(format!(
                "Supabase {method} {} → {}: {}",
                path.split('?').next().unwrap_or(""),
                res.status,
                res.body
            ));
        }
        if res.body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&res.body).map_err(|e| format!("Supabase {method} {path}: {e}"))
    }

    pub fn select(&self, table: &str, query: &str) -> Result<Vec<Value>, String> {
        match self.call("GET", &format!("{table}?{query}"), None, None)? {
            Value::Array(rows) => Ok(rows),
            _ => Ok(Vec::new()),
        }
    }

    pub fn insert(&self, table: &str, rows: &[Value]) -> Result<(), String> {
        if rows.is_empty() {
            return Ok(());
        }
        self.call("POST", table, Some(&Value::Array(rows.to_vec())), Some("return=minimal".into())).map(|_| ())
    }

    /// Insert; on a primary-key (or `on_conflict`) clash, update the row, or skip it with `ignore_duplicates`.
    pub fn upsert(
        &self,
        table: &str,
        rows: &[Value],
        on_conflict: Option<&str>,
        ignore_duplicates: bool,
    ) -> Result<(), String> {
        if rows.is_empty() {
            return Ok(());
        }
        let q = on_conflict.map(|c| format!("?on_conflict={c}")).unwrap_or_default();
        let resolution = if ignore_duplicates { "ignore-duplicates" } else { "merge-duplicates" };
        let prefer = format!("resolution={resolution},return=minimal");
        self.call("POST", &format!("{table}{q}"), Some(&Value::Array(rows.to_vec())), Some(prefer)).map(|_| ())
    }

    pub fn rpc(&self, function: &str, args: Value) -> Result<Value, String> {
        self.call("POST", &format!("rpc/{function}"), Some(&args), None)
    }
}
