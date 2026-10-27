//! In-app toasts and the briefing popup, and the OS notifications that go with them.

pub mod watch;

use crate::js::{self, get, obj};
use crate::state::{Popups, settings};
use crate::tauri;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::Cell;
use std::sync::OnceLock;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Order,
    Campaign,
    Dispatch,
    Capture,
    Loss,
    Defense,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub title: String,
    pub body: String,
    /// Full game text (dispatch or Major Order briefing) for the briefing popup.
    pub message: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Toasts {
    pub toasts: RwSignal<Vec<Toast>>,
    /// The toast whose briefing popup is open.
    pub briefing: RwSignal<Option<Toast>>,
}

const TOAST_LIMIT: usize = 5;

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
    /// None until asked; then whether OS notifications may be shown.
    static PERMISSION: Cell<Option<Option<bool>>> = const { Cell::new(None) };
}

fn next_id() -> u64 {
    NEXT_ID.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    })
}

static TOASTS: OnceLock<Toasts> = OnceLock::new();

pub fn init() {
    let _ = TOASTS.set(Toasts { toasts: RwSignal::new(Vec::new()), briefing: RwSignal::new(None) });
}

pub fn toasts() -> Toasts {
    *TOASTS.get().expect("toasts initialised")
}

/// Whether the Popups setting shows a toast of this kind.
pub fn pops_up(popups: Popups, kind: ToastKind) -> bool {
    match popups {
        Popups::All => true,
        Popups::Priority => kind != ToastKind::Dispatch,
        Popups::None => false,
    }
}

impl Toasts {
    pub fn push(&self, kind: ToastKind, title: String, body: String, message: Option<String>) {
        if !pops_up(settings().untracked(|s| s.popups), kind) {
            return;
        }
        let t = Toast { id: next_id(), kind, title, body, message };
        self.toasts.update(|list| {
            list.insert(0, t);
            list.truncate(TOAST_LIMIT);
        });
    }

    pub fn dismiss(&self, id: u64) {
        self.toasts.update(|list| list.retain(|t| t.id != id));
    }

    /// Swaps the toast for its briefing popup.
    pub fn open(&self, id: u64) {
        let found = self.toasts.with_untracked(|list| list.iter().find(|t| t.id == id).cloned());
        if let Some(t) = found {
            self.briefing.set(Some(t));
        }
        self.dismiss(id);
    }

    /// Opens a briefing popup without a toast.
    pub fn show(&self, kind: ToastKind, title: String, body: String, message: Option<String>) {
        self.briefing.set(Some(Toast { id: next_id(), kind, title, body, message }));
    }

    pub fn close_briefing(&self) {
        self.briefing.set(None);
    }
}

async fn ensure_permission() -> bool {
    if tauri::is_tauri() {
        if tauri::call_api("notification", "isPermissionGranted", &[]).await.is_ok_and(|v| v.is_truthy()) {
            return true;
        }
        return tauri::call_api("notification", "requestPermission", &[])
            .await
            .is_ok_and(|v| v.as_string().as_deref() == Some("granted"));
    }
    let n = get(&js::window(), "Notification");
    if n.is_undefined() {
        return false;
    }
    if get(&n, "permission").as_string().as_deref() == Some("granted") {
        return true;
    }
    match js::call(&n, "requestPermission", &[]) {
        Ok(p) => js::wait(p).await.is_ok_and(|v| v.as_string().as_deref() == Some("granted")),
        Err(_) => false,
    }
}

/// Asks for OS notification permission up front, so the first alert isn't the one that asks.
pub fn request_notifications() {
    if PERMISSION.get().is_some() {
        return;
    }
    PERMISSION.set(Some(None));
    spawn_local(async {
        let granted = ensure_permission().await;
        PERMISSION.set(Some(Some(granted)));
    });
}

async fn permitted() -> bool {
    request_notifications();
    loop {
        if let Some(Some(granted)) = PERMISSION.get() {
            return granted;
        }
        js::sleep(100.0).await;
    }
}

/// In-app toast unless Popups hides it; an OS notification too when `system` is on and permission is granted.
pub fn notify(kind: ToastKind, title: &str, body: &str, system: bool, message: Option<String>) {
    toasts().push(kind, title.into(), body.into(), message);
    if !system {
        return;
    }
    let (title, body) = (title.to_string(), body.to_string());
    spawn_local(async move {
        if !permitted().await {
            return;
        }
        let mut fields = vec![("title", JsValue::from_str(&title)), ("body", JsValue::from_str(&body))];
        if tauri::is_android() {
            // tauri.conf.json can't carry these: a plugins.notification entry stops the app from starting.
            fields
                .extend([("icon", JsValue::from_str("ic_notification")), ("iconColor", JsValue::from_str("#ffff5a"))]);
        }
        let options = obj(&fields);
        if tauri::is_tauri() {
            let _ = tauri::call_api("notification", "sendNotification", &[options]).await;
        } else {
            let n = get(&js::window(), "Notification");
            if let Ok(ctor) = wasm_bindgen::JsCast::dyn_into::<js_sys::Function>(n) {
                let args = js_sys::Array::of2(&JsValue::from_str(&title), &obj(&[("body", JsValue::from_str(&body))]));
                let _ = js_sys::Reflect::construct(&ctor, &args);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_hides_only_dispatches() {
        let kinds = [
            ToastKind::Order,
            ToastKind::Campaign,
            ToastKind::Dispatch,
            ToastKind::Capture,
            ToastKind::Loss,
            ToastKind::Defense,
        ];
        for k in kinds {
            assert!(pops_up(Popups::All, k));
            assert_eq!(pops_up(Popups::Priority, k), k != ToastKind::Dispatch);
            assert!(!pops_up(Popups::None, k));
        }
    }
}
