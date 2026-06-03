use tauri::{AppHandle, Manager, tray::TrayIconBuilder, menu::{Menu, MenuItem, PredefinedMenuItem}, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};

static IS_MONITORING: AtomicBool = AtomicBool::new(true);

pub fn is_monitoring() -> bool {
    IS_MONITORING.load(Ordering::SeqCst)
}

pub fn toggle_monitoring() {
    let current = IS_MONITORING.load(Ordering::SeqCst);
    IS_MONITORING.store(!current, Ordering::SeqCst);
}

fn build_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let monitor_label = if is_monitoring() { "Pause Monitoring" } else { "Resume Monitoring" };
    Menu::with_items(app, &[
        &MenuItem::with_id(app, "show", "Show OmniClip", true, None::<&str>).map_err(|e| e.to_string())?,
        &PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "monitor", monitor_label, true, None::<&str>).map_err(|e| e.to_string())?,
        &PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "quit", "Quit OmniClip", true, None::<&str>).map_err(|e| e.to_string())?,
    ]).map_err(|e| e.to_string())
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app)?;

    let _ = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("OmniClip")
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "monitor" => {
                    toggle_monitoring();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("monitoring-state-changed", is_monitoring());
                    }
                    if let Some(tray) = app.tray_by_id("main") {
                        let _ = tray.set_menu(Some(build_menu(app).unwrap()));
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    Ok(())
}
