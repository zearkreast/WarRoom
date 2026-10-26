//! One HTTP request at a time, behind a trait so the tests can answer in place of the network.

use std::time::Duration;

pub struct Request {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(&'static str, String)>,
    pub body: Option<String>,
    pub timeout_ms: u64,
}

pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug)]
pub enum TransportError {
    Timeout,
    Failed(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Timeout => f.write_str("timed out"),
            TransportError::Failed(reason) => f.write_str(reason),
        }
    }
}

pub trait Transport {
    fn send(&self, req: Request) -> Result<Response, TransportError>;
}

/// A fresh connection per request: a pooled keep-alive socket that died left every later call timing out.
pub struct Http;

impl Transport for Http {
    fn send(&self, req: Request) -> Result<Response, TransportError> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_millis(req.timeout_ms)))
            .http_status_as_error(false)
            .max_idle_connections(0)
            .build()
            .into();
        let result = match (req.method, req.body) {
            ("GET", _) => {
                let mut r = agent.get(&req.url);
                for (k, v) in &req.headers {
                    r = r.header(*k, v);
                }
                r.call()
            }
            (method, body) => {
                let mut r = agent.post(&req.url);
                debug_assert_eq!(method, "POST");
                for (k, v) in &req.headers {
                    r = r.header(*k, v);
                }
                r.send(body.unwrap_or_default())
            }
        };
        let mut res = result.map_err(|e| match e {
            ureq::Error::Timeout(_) => TransportError::Timeout,
            other => TransportError::Failed(other.to_string()),
        })?;
        let status = res.status().as_u16();
        let headers = res
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_string(), v.to_str().ok()?.to_string())))
            .collect();
        let body = res.body_mut().read_to_string().map_err(|e| match e {
            ureq::Error::Timeout(_) => TransportError::Timeout,
            other => TransportError::Failed(other.to_string()),
        })?;
        Ok(Response { status, headers, body })
    }
}
