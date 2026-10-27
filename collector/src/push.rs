//! Firebase Cloud Messaging: sends Strohmann News to the Android apps subscribed to its topic.

use crate::hd2::Clock;
use crate::http::{Request, Transport};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use ring::rand::SystemRandom;
use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair};
use serde::Deserialize;
use serde_json::json;

/// Keep in sync with NEWS_CHANNEL in src-tauri/android/MainActivity.kt.template.
const CHANNEL: &str = "news";
/// FCM refuses messages over 4 KB; a Major Order briefing can come close.
const MAX_BODY: usize = 1000;
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const TIMEOUT_MS: u64 = 20_000;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ServiceAccount {
    pub project_id: String,
    pub client_email: String,
    pub private_key: String,
}

/// Which of the app's alert settings a push falls under; keep the topic names in sync with NewsTopics in
/// src-tauri/android/MainActivity.kt.template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    /// Major Orders and Galactic Campaign launches and outcomes.
    Orders,
    Dispatches,
}

impl Topic {
    /// Older APKs are on "news" alone; FCM delivers once to a device on both topics.
    fn condition(self) -> String {
        let name = match self {
            Topic::Orders => "news-orders",
            Topic::Dispatches => "news-dispatches",
        };
        format!("'news' in topics || '{name}' in topics")
    }
}

pub trait Push {
    fn push(&mut self, topic: Topic, title: &str, body: &str) -> Result<(), String>;
}

/// The service account key as base64 (`base64 -w0 key.json`), or as JSON on one line.
pub fn parse_service_account(value: &str) -> Result<ServiceAccount, String> {
    let json = if value.starts_with('{') {
        value.to_string()
    } else {
        let bytes = STANDARD.decode(value.trim()).map_err(|e| format!("FCM_SERVICE_ACCOUNT is not base64: {e}"))?;
        String::from_utf8(bytes).map_err(|e| e.to_string())?
    };
    let raw: serde_json::Value = serde_json::from_str(&json).map_err(|e| format!("FCM_SERVICE_ACCOUNT: {e}"))?;
    let field = |k: &str| raw.get(k).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from);
    match (field("project_id"), field("client_email"), field("private_key")) {
        (Some(project_id), Some(client_email), Some(private_key)) => {
            Ok(ServiceAccount { project_id, client_email, private_key })
        }
        _ => Err("FCM_SERVICE_ACCOUNT needs project_id, client_email and private_key".into()),
    }
}

/// The DER bytes inside a PEM block.
fn pem_der(pem: &str) -> Result<Vec<u8>, String> {
    let body: String = pem.lines().filter(|l| !l.starts_with("-----")).map(str::trim).collect();
    STANDARD.decode(body).map_err(|e| format!("private_key is not PEM: {e}"))
}

/// Cuts text to `max` UTF-16 units, as JavaScript counts length, ending in an ellipsis.
fn clip(text: &str, max: usize) -> String {
    if text.encode_utf16().count() <= max {
        return text.to_string();
    }
    let mut out = String::new();
    let mut units = 0;
    for c in text.chars() {
        if units + c.len_utf16() > max - 1 {
            break;
        }
        units += c.len_utf16();
        out.push(c);
    }
    out.push('…');
    out
}

pub struct Fcm {
    account: ServiceAccount,
    key: RsaKeyPair,
    transport: Box<dyn Transport>,
    clock: Box<dyn Clock>,
    token: String,
    expires: f64,
}

impl Fcm {
    pub fn new(account: ServiceAccount, transport: Box<dyn Transport>, clock: Box<dyn Clock>) -> Result<Fcm, String> {
        let key = RsaKeyPair::from_pkcs8(&pem_der(&account.private_key)?).map_err(|e| format!("private_key: {e}"))?;
        Ok(Fcm { account, key, transport, clock, token: String::new(), expires: 0.0 })
    }

    /// A short-lived OAuth token from a JWT the service account signs; reused until a minute before it lapses.
    fn access_token(&mut self) -> Result<String, String> {
        let now = self.clock.now();
        if now < self.expires - 60_000.0 {
            return Ok(self.token.clone());
        }
        let iat = (now / 1000.0).floor() as i64;
        let b64 = |v: serde_json::Value| URL_SAFE_NO_PAD.encode(v.to_string());
        let unsigned = format!(
            "{}.{}",
            b64(json!({ "alg": "RS256", "typ": "JWT" })),
            b64(json!({
                "iss": self.account.client_email,
                "scope": "https://www.googleapis.com/auth/firebase.messaging",
                "aud": TOKEN_URL,
                "iat": iat,
                "exp": iat + 3600,
            }))
        );
        let mut signature = vec![0; self.key.public().modulus_len()];
        self.key
            .sign(&RSA_PKCS1_SHA256, &SystemRandom::new(), unsigned.as_bytes(), &mut signature)
            .map_err(|e| format!("FCM token signing: {e}"))?;
        let assertion = format!("{unsigned}.{}", URL_SAFE_NO_PAD.encode(signature));
        let res = self
            .transport
            .send(Request {
                method: "POST",
                url: TOKEN_URL.into(),
                headers: vec![("Content-Type", "application/x-www-form-urlencoded".into())],
                // The assertion is base64url and dots, which need no escaping in a form body.
                body: Some(format!(
                    "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={assertion}"
                )),
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("FCM token: {e}"))?;
        if !res.ok() {
            return Err(format!("FCM token {}: {}", res.status, res.body));
        }
        #[derive(Deserialize)]
        struct Token {
            access_token: String,
            expires_in: f64,
        }
        let got: Token = serde_json::from_str(&res.body).map_err(|e| format!("FCM token: {e}"))?;
        self.token = got.access_token;
        self.expires = self.clock.now() + got.expires_in * 1000.0;
        Ok(self.token.clone())
    }
}

impl Push for Fcm {
    fn push(&mut self, topic: Topic, title: &str, body: &str) -> Result<(), String> {
        let token = self.access_token()?;
        let message = json!({
            "message": {
                "condition": topic.condition(),
                "notification": { "title": title, "body": clip(body, MAX_BODY) },
                "android": {
                    "priority": "high",
                    "notification": { "channel_id": CHANNEL, "icon": "ic_notification", "color": "#ffff5a" },
                },
            },
        });
        let res = self
            .transport
            .send(Request {
                method: "POST",
                url: format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", self.account.project_id),
                headers: vec![
                    ("Authorization", format!("Bearer {token}")),
                    ("Content-Type", "application/json".into()),
                ],
                body: Some(message.to_string()),
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("FCM send: {e}"))?;
        if !res.ok() {
            return Err(format!("FCM send {}: {}", res.status, res.body));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Response, TransportError};
    use ring::signature::{RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey};
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::LazyLock;

    /// A throwaway key, made once per test run.
    static KEY: LazyLock<String> = LazyLock::new(|| {
        let key = rsa::RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).unwrap();
        key.to_pkcs8_pem(LineEnding::LF).unwrap().to_string()
    });

    fn account() -> ServiceAccount {
        ServiceAccount {
            project_id: "warroom".into(),
            client_email: "push@warroom.iam.gserviceaccount.com".into(),
            private_key: KEY.clone(),
        }
    }

    #[derive(Clone)]
    struct Clock(Rc<Cell<f64>>);
    impl crate::hd2::Clock for Clock {
        fn now(&self) -> f64 {
            self.0.get()
        }
        fn sleep(&self, _: f64) {}
    }

    #[derive(Default)]
    struct Call {
        url: String,
        body: String,
        auth: Option<String>,
    }

    /// Google's token and FCM endpoints, recording each request.
    struct Google {
        calls: Rc<RefCell<Vec<Call>>>,
        send_status: u16,
    }

    impl Transport for Google {
        fn send(&self, req: Request) -> Result<Response, TransportError> {
            let auth = req.headers.iter().find(|(k, _)| *k == "Authorization").map(|(_, v)| v.clone());
            self.calls.borrow_mut().push(Call { url: req.url.clone(), body: req.body.unwrap_or_default(), auth });
            let n = self.calls.borrow().len();
            let (status, body) = if req.url.contains("oauth2") {
                (200, format!(r#"{{"access_token":"token{n}","expires_in":3600}}"#))
            } else if self.send_status == 200 {
                (200, r#"{"name":"m1"}"#.into())
            } else {
                (self.send_status, "denied".into())
            };
            Ok(Response { status, headers: Vec::new(), body })
        }
    }

    fn fcm(send_status: u16, t: &Rc<Cell<f64>>) -> (Fcm, Rc<RefCell<Vec<Call>>>) {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let google = Google { calls: calls.clone(), send_status };
        (Fcm::new(account(), Box::new(google), Box::new(Clock(t.clone()))).unwrap(), calls)
    }

    #[test]
    fn signs_a_token_request_with_the_service_account_then_sends_to_the_topic() {
        let t = Rc::new(Cell::new(1_000_000.0));
        let (mut push, calls) = fcm(200, &t);
        push.push(Topic::Dispatches, "New dispatch", "Helldivers, stand by").unwrap();
        let calls = calls.borrow();
        let (token, send) = (&calls[0], &calls[1]);

        assert_eq!(token.url, "https://oauth2.googleapis.com/token");
        let assertion = token.body.split("assertion=").nth(1).unwrap();
        let parts: Vec<&str> = assertion.split('.').collect();
        let public = push.key.public().as_ref().to_vec();
        let signature = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
        UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public)
            .verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature)
            .unwrap();
        let claims: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();
        assert_eq!(
            claims,
            json!({
                "iss": "push@warroom.iam.gserviceaccount.com",
                "scope": "https://www.googleapis.com/auth/firebase.messaging",
                "aud": "https://oauth2.googleapis.com/token",
                "iat": 1000,
                "exp": 4600,
            })
        );

        assert_eq!(send.url, "https://fcm.googleapis.com/v1/projects/warroom/messages:send");
        assert_eq!(send.auth.as_deref(), Some("Bearer token1"));
        let body: serde_json::Value = serde_json::from_str(&send.body).unwrap();
        assert_eq!(
            body["message"],
            json!({
                "condition": "'news' in topics || 'news-dispatches' in topics",
                "notification": { "title": "New dispatch", "body": "Helldivers, stand by" },
                "android": { "priority": "high", "notification": { "channel_id": "news", "icon": "ic_notification", "color": "#ffff5a" } },
            })
        );
    }

    #[test]
    fn reuses_the_token_until_a_minute_before_it_lapses() {
        let t = Rc::new(Cell::new(0.0));
        let (mut push, calls) = fcm(200, &t);
        push.push(Topic::Orders, "a", "b").unwrap();
        t.set(3_500_000.0);
        push.push(Topic::Orders, "a", "b").unwrap();
        t.set(3_560_000.0);
        push.push(Topic::Orders, "a", "b").unwrap();
        let calls = calls.borrow();
        assert_eq!(calls.iter().filter(|c| c.url.contains("oauth2")).count(), 2);
        let auth: Vec<&str> =
            calls.iter().filter(|c| c.url.contains("fcm")).filter_map(|c| c.auth.as_deref()).collect();
        assert_eq!(auth, ["Bearer token1", "Bearer token1", "Bearer token4"]);
    }

    #[test]
    fn cuts_a_long_briefing_down_to_what_fcm_accepts() {
        let t = Rc::new(Cell::new(0.0));
        let (mut push, calls) = fcm(200, &t);
        push.push(Topic::Orders, "New Major Order", &"x".repeat(5000)).unwrap();
        let body: serde_json::Value = serde_json::from_str(&calls.borrow()[1].body).unwrap();
        let text = body["message"]["notification"]["body"].as_str().unwrap();
        assert_eq!(text.encode_utf16().count(), 1000);
        assert!(text.ends_with('…'));
    }

    #[test]
    fn fails_when_fcm_refuses_the_message() {
        let t = Rc::new(Cell::new(0.0));
        let (mut push, _) = fcm(403, &t);
        assert_eq!(push.push(Topic::Orders, "a", "b").unwrap_err(), "FCM send 403: denied");
    }

    #[test]
    fn reaches_installs_on_the_topic_or_still_on_the_old_news_one() {
        assert_eq!(Topic::Orders.condition(), "'news' in topics || 'news-orders' in topics");
        assert_eq!(Topic::Dispatches.condition(), "'news' in topics || 'news-dispatches' in topics");
    }

    #[test]
    fn reads_the_key_as_base64_or_as_json_on_one_line() {
        let a = account();
        let json = json!({ "project_id": a.project_id, "client_email": a.client_email, "private_key": a.private_key })
            .to_string();
        assert_eq!(parse_service_account(&STANDARD.encode(&json)).unwrap(), a);
        assert_eq!(parse_service_account(&json).unwrap(), a);
        assert!(parse_service_account("{}").unwrap_err().starts_with("FCM_SERVICE_ACCOUNT needs"));
    }
}
