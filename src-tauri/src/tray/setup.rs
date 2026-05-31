use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;
use tauri::Manager;

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    let open_main = MenuItem::with_id(app, "open_main", "Open Main Window", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let show_overlay = MenuItem::with_id(app, "show_overlay", "Quick Paste (Alt+V)", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let toggle_monitor =
        MenuItem::with_id(app, "toggle_monitor", "Pause Monitoring", true, None::<&str>)
            .map_err(|e| e.to_string())?;

    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let quit = MenuItem::with_id(app, "quit", "Quit OmniClip", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[&open_main, &show_overlay, &toggle_monitor, &separator, &quit])
        .map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("OmniClip - Clipboard Manager")
        .on_menu_event(|app, event| match event.id().0.as_str() {
            "open_main" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "show_overlay" => {
                let _ = crate::overlay::show_overlay(&app);
            }
            "toggle_monitor" => {
                // TODO: Toggle monitoring state
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
