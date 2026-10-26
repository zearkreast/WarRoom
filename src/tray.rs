use crate::state::settings;
use crate::tauri::{has_tray, invoke};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsValue;

/// Keeps the desktop app's close-to-tray behaviour in step with Settings.
pub fn start() {
    if !has_tray() {
        return;
    }
    let close_to_tray = Memo::new(move |_| settings().get(|s| s.close_to_tray));
    Effect::new(move |_| {
        let enabled = close_to_tray.get();
        spawn_local(async move {
            let _ = invoke("set_close_to_tray", crate::js::obj(&[("enabled", JsValue::from_bool(enabled))])).await;
        });
    });
}
