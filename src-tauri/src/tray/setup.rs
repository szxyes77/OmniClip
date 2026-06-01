use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;
use tauri::Manager;
use std::sync::atomic::{AtomicBool, Ordering};

static MONITORING_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn is_monitoring() -> bool {
    MONITORING_ENABLED.load(Ordering::SeqCst)
}

pub fn toggle_monitoring() {
    let current = MONITORING_ENABLED.load(Ordering::SeqCst);
    MONITORING_ENABLED.store(!current, Ordering::SeqCst);
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    let open_main = MenuItem::with_id(app, "open_main", "显示主窗口", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let show_overlay = MenuItem::with_id(app, "show_overlay", "显示悬浮窗 (Alt+V)", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let toggle_monitor =
        MenuItem::with_id(app, "toggle_monitor", "暂停剪贴板监听", true, None::<&str>)
            .map_err(|e| e.to_string())?;

    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let quit = MenuItem::with_id(app, "quit", "退出 OmniClip", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[&open_main, &show_overlay, &toggle_monitor, &separator, &quit])
        .map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
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
                toggle_monitoring();
                let is_now_monitoring = is_monitoring();
                if let Some(menu_item) = app.tray_by_id("main").and_then(|t| t.get_item("toggle_monitor").ok()) {
                    let label = if is_now_monitoring { "暂停剪贴板监听" } else { "恢复剪贴板监听" };
                    let _ = menu_item.set_text(label);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("OmniClip - 智能剪贴板管理器")
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
