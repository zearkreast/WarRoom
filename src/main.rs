mod alerts;
mod api;
mod app;
mod assets;
mod config;
mod features;
mod history;
mod js;
mod layout;
mod nav_history;
mod state;
mod sync;
mod tauri;
mod tray;
mod ui;
mod update;

use wasm_bindgen::JsCast;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A panic stops the whole app, not one screen, so the screen gives way to the crash notice and Try again
/// reloads. The handler is inline because the app's own code can't run any more.
fn show_crash(message: &str) {
    let doc = js::document();
    let Some(main) = doc.query_selector(".main").ok().flatten().or_else(|| doc.get_element_by_id("root")) else {
        return;
    };
    let icon = |data: ui::icons::IconData, size: u32| {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="{}" aria-hidden="true">{}</svg>"#,
            data.class, data.body
        )
    };
    main.set_inner_html(&format!(
        r#"<div class="crash" role="alert">{}<div><b class="caps">This screen failed to load</b><p class="muted">{}</p></div><button class="hud-btn" onclick="location.reload()">{} Try again</button></div>"#,
        icon(ui::icons::TRIANGLE_ALERT, 22),
        escape(message),
        icon(ui::icons::ROTATE_CCW, 14),
    ));
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        console_error_panic_hook::hook(info);
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| info.to_string());
        show_crash(&message);
    }));
    // Effects made before the first render need the executor that mounting would start.
    let _ = any_spawner::Executor::init_wasm_bindgen();
    state::init();
    api::queries::init();
    history::init();
    alerts::init();
    update::init();
    // Before the first render, so a reload never flashes the default screen.
    nav_history::start();
    api::queries::start_polling();
    sync::start();
    tray::start();
    leptos::mount::mount_to(js::document().get_element_by_id("root").unwrap().unchecked_into(), app::App).forget();
}
