//! The Tauri shell around the web app: plugins, the single-instance lock, the updater and the tray.

#[cfg(desktop)]
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    // First, so a second launch only brings forward the copy already running in the tray.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
        tray::show(app)
    }));
    let builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init());
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(tray::CloseToTray::default())
        .invoke_handler(tauri::generate_handler![tray::set_close_to_tray])
        .setup(|app| {
            if let Err(e) = tray::create(app.handle()) {
                eprintln!("No tray icon, so closing the window quits: {e}");
            }
            Ok(())
        })
        .on_window_event(tray::on_window_event);
    let app = builder
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|_app, _event| {
        // The dock icon brings back a window hidden to the tray.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } = _event
        {
            tray::show(_app);
        }
    });
}
