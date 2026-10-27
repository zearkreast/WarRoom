//! The desktop tray icon, and keeping the app running there when its window closes.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, State, Window, WindowEvent};

/// Whether closing the window only hides it, so alerts keep coming. Settings turns it on at start-up;
/// without a tray icon it stays off, since nothing would bring the window back.
#[derive(Default)]
pub struct CloseToTray {
    enabled: AtomicBool,
    tray: AtomicBool,
}

#[tauri::command]
pub fn set_close_to_tray(enabled: bool, state: State<'_, CloseToTray>) {
    state.enabled.store(enabled, Ordering::Relaxed);
}

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open WarCommand Tracker", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("WarCommand Tracker")
        .menu(&Menu::with_items(app, &[&open, &quit])?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    app.state::<CloseToTray>()
        .tray
        .store(true, Ordering::Relaxed);
    Ok(())
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let state = window.state::<CloseToTray>();
    if state.enabled.load(Ordering::Relaxed) && state.tray.load(Ordering::Relaxed) {
        api.prevent_close();
        let _ = window.hide();
    }
}
