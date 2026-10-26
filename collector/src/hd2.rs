//! Rate-limited client for api.helldivers2.dev and the official game API. The collector is its only caller.

use crate::http::{Request, Transport, TransportError};
use crate::language::not_english;
use serde_json::Value;
use std::cell::Cell;
use std::fmt;

pub const DEFAULT_RETRY_MS: f64 = 30_000.0;
const REQUEST_TIMEOUT_MS: u64 = 25_000;
const MAX_BACKOFF_MS: f64 = 300_000.0;
const LANGUAGE_RETRY_MS: f64 = 60_000.0;
/// Wait once X-RateLimit-Remaining reaches 0.
const WINDOW_MS: f64 = 10_000.0;

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    RateLimited {
        retry_after_ms: f64,
    },
    /// `retry_after_ms` is set when the whole queue backs off after this failure; the job should not retry sooner.
    Status {
        status: u16,
        path: String,
        retry_after_ms: Option<f64>,
    },
    /// The answer came in another language; the cache holds it for 30 s, so the job tries again a minute later.
    NotEnglish {
        path: String,
    },
    Invalid {
        path: String,
        reason: String,
    },
}

impl ApiError {
    pub fn retry_after_ms(&self) -> Option<f64> {
        match self {
            ApiError::RateLimited { retry_after_ms } => Some(*retry_after_ms),
            ApiError::Status { retry_after_ms, .. } => *retry_after_ms,
            ApiError::NotEnglish { .. } => Some(LANGUAGE_RETRY_MS),
            ApiError::Invalid { .. } => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RateLimited { retry_after_ms } => {
                write!(f, "Rate limited, retry in {}s", (retry_after_ms / 1000.0).round())
            }
            ApiError::Status { status, path, .. } => write!(f, "API {status} for {path}"),
            ApiError::NotEnglish { path } => write!(f, "Not in English: {path}"),
            ApiError::Invalid { path, reason } => write!(f, "Unreadable answer from {path}: {reason}"),
        }
    }
}

/// Wall time, and waiting; the tests run on a clock they move themselves.
pub trait Clock {
    fn now(&self) -> f64;
    fn sleep(&self, ms: f64);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> f64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64)
    }

    fn sleep(&self, ms: f64) {
        std::thread::sleep(std::time::Duration::from_millis(ms.max(0.0) as u64));
    }
}

/// Anything the jobs can fetch JSON from by path.
pub trait Api {
    fn get(&self, path: &str) -> Result<Value, ApiError>;
}

pub struct Options {
    pub client: String,
    pub contact: String,
    pub base_url: String,
    /// Minimum gap between consecutive requests. The community API allows 5 requests per 10 s.
    pub min_gap_ms: f64,
    /// First pause after a timeout or network failure; doubles on each further one, up to MAX_BACKOFF_MS.
    pub backoff_ms: f64,
    pub timeout_ms: u64,
}

impl Options {
    pub fn new(client: &str, contact: &str, base_url: &str) -> Options {
        Options {
            client: client.into(),
            contact: contact.into(),
            base_url: base_url.into(),
            min_gap_ms: 2500.0,
            backoff_ms: DEFAULT_RETRY_MS,
            timeout_ms: REQUEST_TIMEOUT_MS,
        }
    }
}

/// Serializes every API call, spaces them out, and pauses the queue when the server answers 429 or
/// stops answering.
pub struct Hd2Client {
    opts: Options,
    transport: Box<dyn Transport>,
    clock: Box<dyn Clock>,
    next_slot: Cell<f64>,
    paused_until: Cell<f64>,
    backoff: Cell<f64>,
}

impl Hd2Client {
    pub fn new(opts: Options, transport: Box<dyn Transport>, clock: Box<dyn Clock>) -> Hd2Client {
        Hd2Client {
            opts,
            transport,
            clock,
            next_slot: Cell::new(0.0),
            paused_until: Cell::new(0.0),
            backoff: Cell::new(0.0),
        }
    }

    fn pause(&self, ms: f64) {
        self.paused_until.set(self.paused_until.get().max(self.clock.now() + ms));
    }

    /// A server that has gone quiet gets ever longer breaks instead of every job retrying into it.
    fn failed(&self, status: u16, path: String) -> ApiError {
        let b = self.backoff.get();
        let next = if b > 0.0 { (b * 2.0).min(MAX_BACKOFF_MS) } else { self.opts.backoff_ms };
        self.backoff.set(next);
        self.pause(next);
        ApiError::Status { status, path, retry_after_ms: Some(next) }
    }
}

impl Api for Hd2Client {
    fn get(&self, path: &str) -> Result<Value, ApiError> {
        let wait = self.next_slot.get().max(self.paused_until.get()) - self.clock.now();
        if wait > 0.0 {
            self.clock.sleep(wait);
        }
        self.next_slot.set(self.clock.now() + self.opts.min_gap_ms);

        // Requests are serialized, so a hung one would stall every other endpoint without a timeout.
        let sent = self.transport.send(Request {
            method: "GET",
            url: format!("{}{path}", self.opts.base_url),
            headers: vec![
                ("X-Super-Client", self.opts.client.clone()),
                ("X-Super-Contact", self.opts.contact.clone()),
                ("Accept", "application/json".into()),
                // The official game API rejects requests without it.
                ("Accept-Language", "en-US".into()),
            ],
            body: None,
            timeout_ms: self.opts.timeout_ms,
        });
        let res = match sent {
            Ok(res) => res,
            Err(TransportError::Timeout) => return Err(self.failed(0, format!("{path} (timed out)"))),
            Err(TransportError::Failed(_)) => return Err(self.failed(0, format!("{path} (unreachable)"))),
        };
        if res.status == 429 {
            let header: f64 = res.header("Retry-After").and_then(|h| h.trim().parse().ok()).unwrap_or(0.0);
            let retry_after_ms = if header.is_finite() && header > 0.0 { header * 1000.0 } else { DEFAULT_RETRY_MS };
            self.pause(retry_after_ms);
            return Err(ApiError::RateLimited { retry_after_ms });
        }
        if !res.ok() {
            if res.status >= 500 {
                return Err(self.failed(res.status, path.into()));
            }
            return Err(ApiError::Status { status: res.status, path: path.into(), retry_after_ms: None });
        }
        self.backoff.set(0.0);
        if res.header("X-RateLimit-Remaining") == Some("0") {
            self.pause(WINDOW_MS);
        }
        let body: Value = serde_json::from_str(&res.body)
            .map_err(|e| ApiError::Invalid { path: path.into(), reason: e.to_string() })?;
        if not_english(&body) {
            return Err(ApiError::NotEnglish { path: path.into() });
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Response;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Default)]
    struct FakeClock(Rc<Cell<f64>>);

    impl Clock for FakeClock {
        fn now(&self) -> f64 {
            self.0.get()
        }
        fn sleep(&self, ms: f64) {
            self.0.set(self.0.get() + ms);
        }
    }

    type Reply = Box<dyn Fn(usize, &Request) -> Result<Response, TransportError>>;
    /// The clock and headers of each request.
    type Calls = Rc<RefCell<Vec<(f64, Vec<(&'static str, String)>)>>>;

    /// Answers each request with `reply(call number, request)` and notes the clock at each call.
    struct Fake {
        clock: FakeClock,
        calls: Calls,
        reply: Reply,
    }

    impl Transport for Fake {
        fn send(&self, req: Request) -> Result<Response, TransportError> {
            let n = self.calls.borrow().len();
            self.calls.borrow_mut().push((self.clock.now(), req.headers.clone()));
            (self.reply)(n, &req)
        }
    }

    fn json(body: &str, status: u16, headers: &[(&str, &str)]) -> Result<Response, TransportError> {
        Ok(Response {
            status,
            headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            body: body.into(),
        })
    }

    fn client(min_gap_ms: f64, backoff_ms: f64, reply: Reply) -> (Hd2Client, Calls) {
        let clock = FakeClock::default();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let fake = Fake { clock: clock.clone(), calls: calls.clone(), reply };
        let opts = Options { min_gap_ms, backoff_ms, ..Options::new("warcommand-tracker", "test", "") };
        (Hd2Client::new(opts, Box::new(fake), Box::new(clock)), calls)
    }

    fn times(calls: &Calls) -> Vec<f64> {
        calls.borrow().iter().map(|c| c.0).collect()
    }

    #[test]
    fn sends_the_required_headers() {
        let (c, calls) = client(0.0, 1.0, Box::new(|_, _| json("{\"ok\":1}", 200, &[])));
        c.get("/api/v1/war").unwrap();
        let headers = &calls.borrow()[0].1;
        assert!(headers.contains(&("X-Super-Client", "warcommand-tracker".into())));
        assert!(headers.iter().any(|(k, v)| *k == "X-Super-Contact" && !v.is_empty()));
    }

    #[test]
    fn spaces_requests_out() {
        let (c, calls) = client(1500.0, 1.0, Box::new(|_, _| json("[]", 200, &[])));
        for p in ["/a", "/b", "/c"] {
            c.get(p).unwrap();
        }
        assert_eq!(times(&calls), [0.0, 1500.0, 3000.0]);
    }

    #[test]
    fn times_out_a_hung_request_so_the_queue_keeps_moving() {
        let (c, _) = client(
            0.0,
            1.0,
            Box::new(|n, _| if n == 0 { Err(TransportError::Timeout) } else { json("{\"ok\":1}", 200, &[]) }),
        );
        assert!(c.get("/hang").unwrap_err().to_string().contains("timed out"));
        assert_eq!(c.get("/next").unwrap(), serde_json::json!({ "ok": 1 }));
    }

    #[test]
    fn rejects_an_answer_in_another_language_without_pausing_the_queue() {
        let spanish = r#"[{"message":"Los Helldivers defienden la colonia de los ataques del enemigo."}]"#;
        let (c, calls) = client(0.0, 1.0, Box::new(move |n, _| json(if n == 0 { spanish } else { "[]" }, 200, &[])));
        let err = c.get("/api/v1/dispatches").unwrap_err();
        assert!(matches!(err, ApiError::NotEnglish { .. }));
        assert_eq!(err.retry_after_ms(), Some(60_000.0));
        c.get("/api/v1/war").unwrap();
        assert_eq!(times(&calls), [0.0, 0.0]);
    }

    #[test]
    fn pauses_the_whole_queue_after_a_429_using_retry_after() {
        let (c, calls) =
            client(
                0.0,
                1.0,
                Box::new(|n, _| {
                    if n == 0 { json("null", 429, &[("Retry-After", "20")]) } else { json("{\"ok\":1}", 200, &[]) }
                }),
            );
        assert!(matches!(c.get("/a"), Err(ApiError::RateLimited { .. })));
        c.get("/b").unwrap();
        assert_eq!(times(&calls), [0.0, 20_000.0]);
    }

    #[test]
    fn backs_the_whole_queue_off_ever_longer_while_the_server_is_unreachable() {
        let (c, calls) = client(
            0.0,
            30_000.0,
            Box::new(|n, _| {
                if n < 2 { Err(TransportError::Failed("socket hang up".into())) } else { json("{\"ok\":1}", 200, &[]) }
            }),
        );
        assert_eq!(c.get("/a").unwrap_err().retry_after_ms(), Some(30_000.0));
        assert_eq!(c.get("/b").unwrap_err().retry_after_ms(), Some(60_000.0));
        assert_eq!(c.get("/c").unwrap(), serde_json::json!({ "ok": 1 }));
        assert_eq!(times(&calls), [0.0, 30_000.0, 90_000.0]);
    }

    #[test]
    fn fails_a_missing_page_without_backing_off() {
        let (c, _) = client(0.0, 1.0, Box::new(|_, _| json("", 404, &[])));
        let err = c.get("/missing").unwrap_err();
        assert!(err.to_string().contains("404"));
        assert_eq!(err.retry_after_ms(), None);
    }

    #[test]
    fn waits_out_the_window_when_the_server_says_it_is_used_up() {
        let (c, calls) = client(0.0, 1.0, Box::new(|_, _| json("{\"ok\":1}", 200, &[("X-RateLimit-Remaining", "0")])));
        c.get("/a").unwrap();
        c.get("/b").unwrap();
        assert_eq!(times(&calls), [0.0, 10_000.0]);
    }
}
