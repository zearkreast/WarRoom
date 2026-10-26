//! Looks for newer releases: the desktop app installs them with Tauri's updater, the Android app hands
//! the new APK to the system.

pub mod card;

use crate::config::APP_VERSION;
use crate::js::{self, get};
use crate::tauri::{self, error_text, is_android, is_tauri};
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::sync::OnceLock;
use warroom_core::version::is_newer;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

const LATEST_RELEASE: &str = "https://api.github.com/repos/zearkreast/WarRoom/releases/latest";
const RECHECK_MS: f64 = 6.0 * 3_600_000.0;

#[derive(Clone, Debug, PartialEq)]
pub enum UpdatePhase {
    Idle,
    Checking,
    Current,
    Available { version: String, notes: String },
    Downloading { version: String, progress: Option<f64> },
    // Android: the APK is saved and Android's installer has been opened on it.
    Downloaded { version: String },
    Installing { version: String },
    HandedOff { version: String },
    Error { version: Option<String>, message: String },
}

#[derive(Clone, Copy)]
pub struct Update {
    pub state: RwSignal<UpdatePhase>,
    /// Hidden until the next launch or a manual check.
    pub dismissed: RwSignal<bool>,
}

static UPDATE: OnceLock<Update> = OnceLock::new();

pub fn init() {
    let _ = UPDATE.set(Update { state: RwSignal::new(UpdatePhase::Idle), dismissed: RwSignal::new(false) });
}

pub fn update() -> Update {
    *UPDATE.get().expect("update initialised")
}

thread_local! {
    /// The updater's Update object from the last check.
    static PENDING: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    static APK_URL: RefCell<Option<String>> = const { RefCell::new(None) };
    /// `window.__warroomApk`, kept alive here until the next download replaces it.
    static APK_EVENTS: RefCell<Option<Closure<dyn Fn(JsValue)>>> = const { RefCell::new(None) };
}

fn set(phase: UpdatePhase) {
    update().state.set(phase);
}

/// ApkUpdate in src-tauri/android/MainActivity.kt.template; APKs from before it have none.
fn apk_bridge() -> Option<JsValue> {
    Some(get(&js::window(), "WarRoomApk")).filter(|b| !b.is_undefined() && !b.is_null())
}

/// An event the bridge sends back to `window.__warroomApk`.
#[derive(Debug, PartialEq)]
enum ApkEvent {
    // `total` is -1 when the server didn't say how big the APK is.
    Progress { received: f64, total: f64 },
    Done,
    Error(String),
}

impl ApkEvent {
    fn read(e: &JsValue) -> Self {
        match get(e, "type").as_string().as_deref() {
            Some("progress") => Self::Progress {
                received: get(e, "received").as_f64().unwrap_or(0.0),
                total: get(e, "total").as_f64().unwrap_or(0.0),
            },
            Some("done") => Self::Done,
            _ => Self::Error(get(e, "message").as_string().unwrap_or_default()),
        }
    }
}

fn apk_phase(version: &str, event: ApkEvent) -> UpdatePhase {
    let version = version.to_string();
    match event {
        ApkEvent::Progress { received, total } => {
            UpdatePhase::Downloading { version, progress: (total > 0.0).then(|| received / total) }
        }
        ApkEvent::Done => UpdatePhase::Downloaded { version },
        ApkEvent::Error(message) => UpdatePhase::Error { version: Some(version), message },
    }
}

async fn check_android() -> Result<UpdatePhase, String> {
    let init = js::obj(&[("headers", js::obj(&[("Accept", "application/vnd.github+json".into())]))]);
    let res =
        js::wait(js::call(&js::window().into(), "fetch", &[LATEST_RELEASE.into(), init]).map_err(|e| error_text(&e))?)
            .await
            .map_err(|e| error_text(&e))?;
    if !get(&res, "ok").is_truthy() {
        return Err(format!("GitHub answered {}", get(&res, "status").as_f64().unwrap_or(0.0)));
    }
    let release =
        js::wait(js::call(&res, "json", &[]).map_err(|e| error_text(&e))?).await.map_err(|e| error_text(&e))?;
    let tag = get(&release, "tag_name").as_string().unwrap_or_default();
    let assets = js_sys::Array::from(&get(&release, "assets"));
    let apk = assets.iter().find(|a| get(a, "name").as_string().is_some_and(|n| n.ends_with("_android.apk")));
    let Some(apk) = apk.filter(|_| is_newer(&tag, APP_VERSION)) else { return Ok(UpdatePhase::Current) };
    APK_URL.with_borrow_mut(|u| *u = get(&apk, "browser_download_url").as_string());
    let version = tag.strip_prefix(['v', 'V']).unwrap_or(&tag).to_string();
    Ok(UpdatePhase::Available { version, notes: get(&release, "body").as_string().unwrap_or_default() })
}

async fn check_desktop() -> Result<UpdatePhase, String> {
    let found = tauri::call_api("updater", "check", &[]).await.map_err(|e| error_text(&e))?;
    if found.is_null() || found.is_undefined() {
        return Ok(UpdatePhase::Current);
    }
    let version = get(&found, "version").as_string().unwrap_or_default();
    let notes = get(&found, "body").as_string().unwrap_or_default();
    PENDING.with_borrow_mut(|p| *p = Some(found));
    Ok(UpdatePhase::Available { version, notes })
}

/// Looks for a newer release; the web build has nothing to install, so it never looks.
pub fn check_for_update(manual: bool) {
    if !is_tauri() {
        return;
    }
    if manual {
        update().dismissed.set(false);
    }
    set(UpdatePhase::Checking);
    spawn_local(async move {
        let result = if is_android() { check_android().await } else { check_desktop().await };
        set(match result {
            Ok(phase) => phase,
            // A check that fails quietly at start-up is better than an error card on every offline launch.
            Err(message) if manual => UpdatePhase::Error { version: None, message },
            Err(_) => UpdatePhase::Idle,
        });
    });
}

/// Downloads and installs the update found by the last check, then restarts into it.
pub fn install_update() {
    let version = match update().state.get_untracked() {
        UpdatePhase::Available { version, .. } => version,
        UpdatePhase::Error { version, .. } => version.unwrap_or_default(),
        _ => return,
    };
    spawn_local(async move {
        if is_android() {
            let Some(url) = APK_URL.with_borrow(|u| u.clone()) else { return };
            let Some(bridge) = apk_bridge() else {
                match tauri::open_url(&url).await {
                    Ok(()) => set(UpdatePhase::HandedOff { version }),
                    Err(e) => set(UpdatePhase::Error { version: Some(version), message: error_text(&e) }),
                }
                return;
            };
            let v = version.clone();
            let on_event = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| set(apk_phase(&v, ApkEvent::read(&e))));
            let _ = js_sys::Reflect::set(&js::window(), &"__warroomApk".into(), on_event.as_ref());
            APK_EVENTS.with_borrow_mut(|c| *c = Some(on_event));
            set(UpdatePhase::Downloading { version: version.clone(), progress: None });
            if let Err(e) = js::call(&bridge, "download", &[url.into()]) {
                set(UpdatePhase::Error { version: Some(version), message: error_text(&e) });
            }
            return;
        }
        let Some(pending) = PENDING.with_borrow(|p| p.clone()) else { return };
        set(UpdatePhase::Downloading { version: version.clone(), progress: None });
        let total = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let received = std::rc::Rc::new(std::cell::Cell::new(0.0));
        let v = version.clone();
        let on_event = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| {
            let data = get(&e, "data");
            match get(&e, "event").as_string().as_deref() {
                Some("Started") => total.set(get(&data, "contentLength").as_f64().unwrap_or(0.0)),
                Some("Progress") => {
                    received.set(received.get() + get(&data, "chunkLength").as_f64().unwrap_or(0.0));
                    let progress = (total.get() > 0.0).then(|| received.get() / total.get());
                    set(UpdatePhase::Downloading { version: v.clone(), progress });
                }
                _ => set(UpdatePhase::Installing { version: v.clone() }),
            }
        });
        let result = match js::call(&pending, "downloadAndInstall", &[on_event.as_ref().clone()]) {
            Ok(p) => js::wait(p).await,
            Err(e) => Err(e),
        };
        drop(on_event);
        let result = match result {
            Ok(_) => tauri::call_api("process", "relaunch", &[]).await,
            Err(e) => Err(e),
        };
        if let Err(e) = result {
            set(UpdatePhase::Error { version: Some(version), message: error_text(&e) });
        }
    });
}

/// Android: opens the installer again on the downloaded APK, if it was closed without installing.
pub fn open_installer() {
    if let Some(bridge) = apk_bridge() {
        let _ = js::call(&bridge, "install", &[]);
    }
}

/// Checks shortly after start-up, then every few hours while the app stays open.
pub fn start_update_checks() {
    if !is_tauri() {
        return;
    }
    set_timeout(|| check_for_update(false), std::time::Duration::from_millis(8_000));
    let again = set_interval_with_handle(
        || {
            if matches!(update().state.get_untracked(), UpdatePhase::Idle | UpdatePhase::Current) {
                check_for_update(false);
            }
        },
        std::time::Duration::from_millis(RECHECK_MS as u64),
    );
    let _keep = SendWrapper::new(again);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_apk_download_to_the_installer() {
        let progress = |received, total| apk_phase("99.0.0", ApkEvent::Progress { received, total });
        assert_eq!(progress(25.0, 100.0), UpdatePhase::Downloading { version: "99.0.0".into(), progress: Some(0.25) });
        assert_eq!(progress(25.0, -1.0), UpdatePhase::Downloading { version: "99.0.0".into(), progress: None });
        assert_eq!(apk_phase("99.0.0", ApkEvent::Done), UpdatePhase::Downloaded { version: "99.0.0".into() });
        assert_eq!(
            apk_phase("99.0.0", ApkEvent::Error("The download is gone; download it again.".into())),
            UpdatePhase::Error {
                version: Some("99.0.0".into()),
                message: "The download is gone; download it again.".into()
            }
        );
    }
}
