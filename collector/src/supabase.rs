//! Minimal Supabase REST (PostgREST) client: just the calls the collector makes.

use crate::http::{Request, Transport};
use serde_json::{Value, json};

const TIMEOUT_MS: u64 = 30_000;

pub struct Supabase {
    base: String,
    storage: String,
    /// The project's secret key (sb_secret_…). It bypasses RLS, so it stays on the server.
    key: String,
    transport: Box<dyn Transport>,
}

impl Supabase {
    pub fn new(url: &str, key: &str, transport: Box<dyn Transport>) -> Supabase {
        let url = url.trim_end_matches('/');
        Supabase { base: format!("{url}/rest/v1"), storage: format!("{url}/storage/v1"), key: key.into(), transport }
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
                body: body.map(|b| warroom_core::api::js_value(b.clone()).to_string().into_bytes()),
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("Supabase {method} {} → {e}", path.split('?').next().unwrap_or("")))?;
        if !res.ok() {
            return Err(format!(
                "Supabase {method} {} → {}: {}",
                path.split('?').next().unwrap_or(""),
                res.status,
                res.text()
            ));
        }
        if res.body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_slice(&res.body).map_err(|e| format!("Supabase {method} {path}: {e}"))
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

    fn storage_call(&self, path: &str, content_type: &str, body: Vec<u8>, upsert: bool) -> Result<Vec<u8>, String> {
        let mut headers = vec![("apikey", self.key.clone()), ("Content-Type", content_type.to_string())];
        if !upsert {
            headers.push(("x-upsert", "false".into()));
        }
        let res = self
            .transport
            .send(Request {
                method: "POST",
                url: format!("{}/{path}", self.storage),
                headers,
                body: Some(body),
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("Storage {path} → {e}"))?;
        if !res.ok() {
            return Err(format!("Storage {path} → {}: {}", res.status, res.text()));
        }
        Ok(res.body)
    }

    /// Every file name in a bucket's root.
    pub fn storage_list(&self, bucket: &str) -> Result<Vec<String>, String> {
        const PAGE: usize = 1000;
        let mut names = Vec::new();
        loop {
            let query = json!({ "prefix": "", "limit": PAGE, "offset": names.len() });
            let body = self.storage_call(
                &format!("object/list/{bucket}"),
                "application/json",
                query.to_string().into_bytes(),
                true,
            )?;
            let page: Vec<Value> = serde_json::from_slice(&body).map_err(|e| format!("Storage list {bucket}: {e}"))?;
            let n = page.len();
            names.extend(page.iter().filter_map(|o| o.get("name")?.as_str().map(String::from)));
            if n < PAGE {
                return Ok(names);
            }
        }
    }

    /// Uploads a file; one already stored under `name` is kept and the upload fails.
    pub fn storage_upload(&self, bucket: &str, name: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), String> {
        self.storage_call(&format!("object/{bucket}/{name}"), content_type, bytes, false).map(|_| ())
    }
}

/// A table or function the database doesn't have yet: its migration hasn't been applied.
pub fn is_missing(e: &str) -> bool {
    e.contains(" → 404:")
}

/// Runs a write that needs a newer migration; until it's applied, the write is skipped instead of failing its task.
pub fn unless_missing<T>(write: Result<T, String>) -> Result<bool, String> {
    match write {
        Ok(_) => Ok(true),
        Err(e) if is_missing(&e) => Ok(false),
        Err(e) => Err(e),
    }
}
