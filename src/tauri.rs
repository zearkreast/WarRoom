//! The Tauri shell's JavaScript API, as the `withGlobalTauri` script exposes it on `window.__TAURI__`.

use crate::js::{call, get, wait, window};
use wasm_bindgen::JsValue;

/// Whether the app runs inside the Tauri shell (desktop or Android) rather than a browser.
pub fn is_tauri() -> bool {
    get(&window(), "isTauri").is_truthy()
}

/// The Android build has no updater: its update is the release's APK, installed by the system.
pub fn is_android() -> bool {
    is_tauri() && crate::js::user_agent().to_lowercase().contains("android")
}

/// The desktop app has a tray icon; Android and the web don't.
pub fn has_tray() -> bool {
    is_tauri() && !is_android()
}

/// `window.__TAURI__.<module>`.
pub fn module(name: &str) -> JsValue {
    get(&get(&window(), "__TAURI__"), name)
}

/// Calls `window.__TAURI__.<module>.<function>(...args)` and awaits its promise.
pub async fn call_api(module_name: &str, function: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    wait(call(&module(module_name), function, args)?).await
}

/// Calls a command of the Rust side.
pub async fn invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue> {
    call_api("core", "invoke", &[JsValue::from_str(command), args]).await
}

/// Opens a link in the system browser.
pub async fn open_url(url: &str) -> Result<(), JsValue> {
    call_api("opener", "openUrl", &[JsValue::from_str(url)]).await.map(|_| ())
}

/// The message of a rejected JS promise or thrown value.
pub fn error_text(e: &JsValue) -> String {
    e.as_string()
        .or_else(|| get(e, "message").as_string())
        .or_else(|| js_sys::JSON::stringify(e).ok().and_then(|s| s.as_string()))
        .unwrap_or_else(|| "unknown error".into())
}
