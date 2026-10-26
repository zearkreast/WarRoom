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

use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
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
