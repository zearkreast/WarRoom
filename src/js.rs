//! Small bridges to the browser: time, timers, storage, formatting and DOM queries.

use js_sys::{Function, Object, Reflect};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub fn now() -> f64 {
    js_sys::Date::now()
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("a window")
}

pub fn document() -> web_sys::Document {
    window().document().expect("a document")
}

/// Reads `obj[key]`, undefined when absent.
pub fn get(obj: &JsValue, key: &str) -> JsValue {
    Reflect::get(obj, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

/// Builds a plain object from key/value pairs.
pub fn obj(entries: &[(&str, JsValue)]) -> JsValue {
    let o = Object::new();
    for (k, v) in entries {
        let _ = Reflect::set(&o, &JsValue::from_str(k), v);
    }
    o.into()
}

/// Calls `obj.method(...args)`.
pub fn call(obj: &JsValue, method: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let f: Function = get(obj, method).dyn_into()?;
    let list = js_sys::Array::new();
    for a in args {
        list.push(a);
    }
    Reflect::apply(&f, obj, &list)
}

/// Awaits a JS promise.
pub async fn wait(promise: JsValue) -> Result<JsValue, JsValue> {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&promise)).await
}

/// Waits `ms` milliseconds.
pub async fn sleep(ms: f64) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// A signal holding the current time, refreshed every `ms` while the calling component lives.
pub fn use_now(ms: f64) -> ReadSignal<f64> {
    let (now_s, set_now) = signal(now());
    let handle = set_interval_with_handle(move || set_now.set(now()), std::time::Duration::from_millis(ms as u64)).ok();
    on_cleanup(move || {
        if let Some(h) = handle {
            h.clear();
        }
    });
    now_s
}

/// Whether a media query matches, kept up to date while the calling component lives.
pub fn use_media(query: &str) -> ReadSignal<bool> {
    let list = window().match_media(query).ok().flatten();
    let (matches, set) = signal(list.as_ref().is_some_and(|l| l.matches()));
    if let Some(list) = list {
        let on_change = Closure::<dyn Fn(web_sys::MediaQueryListEvent)>::new(move |e: web_sys::MediaQueryListEvent| {
            set.set(e.matches())
        });
        let _ = list.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
        let f: Function = on_change.as_ref().clone().unchecked_into();
        let list = send_wrapper::SendWrapper::new((list, f, on_change));
        on_cleanup(move || {
            let (list, f, _keep) = list.take();
            let _ = list.remove_event_listener_with_callback("change", &f);
        });
    }
    matches
}

pub fn local_get(key: &str) -> Option<String> {
    window().local_storage().ok().flatten()?.get_item(key).ok().flatten()
}

pub fn local_set(key: &str, value: &str) {
    if let Some(s) = window().local_storage().ok().flatten() {
        let _ = s.set_item(key, value);
    }
}

/// UI copy is English, so dates and times use English formatting whatever the system locale.
pub const LOCALE: &str = "en-GB";

fn date_string(ms: f64, method: &str, options: JsValue) -> String {
    let d = js_sys::Date::new(&JsValue::from_f64(ms));
    call(&d, method, &[JsValue::from_str(LOCALE), options]).ok().and_then(|v| v.as_string()).unwrap_or_default()
}

/// "26 Oct 2026", in local time.
pub fn fmt_date(ms: f64) -> String {
    let o = obj(&[("day", "numeric".into()), ("month", "short".into()), ("year", "numeric".into())]);
    date_string(ms, "toLocaleDateString", o)
}

/// "16:06", in local time.
pub fn fmt_time(ms: f64) -> String {
    let o = obj(&[("hour", "2-digit".into()), ("minute", "2-digit".into())]);
    date_string(ms, "toLocaleTimeString", o)
}

/// "26 Oct, 16:06", in local time.
pub fn fmt_date_time(ms: f64) -> String {
    let o = obj(&[
        ("month", "short".into()),
        ("day", "numeric".into()),
        ("hour", "2-digit".into()),
        ("minute", "2-digit".into()),
    ]);
    date_string(ms, "toLocaleString", o)
}

/// Today's local date: year, month (1-12) and day.
pub fn local_today(ms: f64) -> (i64, u32, u32) {
    let d = js_sys::Date::new(&JsValue::from_f64(ms));
    (d.get_full_year() as i64, d.get_month() + 1, d.get_date())
}

/// A page-unique id for SVG defs, like React's useId.
pub fn unique_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("{prefix}{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// The JavaScript number formatting of a value, for attributes ("16", "1.5").
pub fn num(n: f64) -> String {
    warroom_core::format::js_number(n)
}

pub fn user_agent() -> String {
    window().navigator().user_agent().unwrap_or_default()
}

pub fn prefers_reduced_motion() -> bool {
    window().match_media("(prefers-reduced-motion: reduce)").ok().flatten().is_some_and(|l| l.matches())
}

pub fn copy_text(text: &str) {
    let clipboard = window().navigator().clipboard();
    let _ = clipboard.write_text(text);
}
