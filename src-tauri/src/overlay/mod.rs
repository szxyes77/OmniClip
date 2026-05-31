use tauri::AppHandle;
use tauri::Manager;

pub fn setup_overlay(app: &AppHandle) -> Result<(), String> {
    setup_global_shortcut(app)?;
    Ok(())
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);

    let app_clone = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = toggle_overlay(&app_clone);
        }
    }).map_err(|e| e.to_string())?;

    Ok(())
}

fn toggle_overlay(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            _ = window.hide();
            _ = set_cursor_passthrough(&window, false);
        } else {
            position_window_at_cursor(&window).ok();
            _ = set_cursor_passthrough(&window, true);
            _ = window.show();
            _ = window.set_focus();
        }
    }
    Ok(())
}

pub fn set_cursor_passthrough(window: &tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    window.set_ignore_cursor_events(enable).map_err(|e| e.to_string())
}

pub fn show_overlay(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        position_window_at_cursor(&window).ok();
        set_cursor_passthrough(&window, true)?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn hide_overlay(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        set_cursor_passthrough(&window, false).ok();
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn paste_with_simulated_keys(app: &AppHandle, content: &str) -> Result<(), String> {
    hide_overlay(app).ok();

    std::thread::sleep(std::time::Duration::from_millis(80));

    {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        app.clipboard()
            .write_text(content)
            .map_err(|e| e.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        use enigo::Direction;
        enigo.key(enigo::Key::Control, Direction::Press).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Unicode('v'), Direction::Click).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Control, Direction::Release).ok();
    }

    #[cfg(target_os = "macos")]
    {
        use enigo::Direction;
        enigo.key(enigo::Key::Meta, Direction::Press).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Unicode('v'), Direction::Click).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Meta, Direction::Release).ok();
    }

    #[cfg(target_os = "linux")]
    {
        use enigo::Direction;
        enigo.key(enigo::Key::Control, Direction::Press).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Unicode('v'), Direction::Click).ok();
        std::thread::sleep(std::time::Duration::from_millis(30));
        enigo.key(enigo::Key::Control, Direction::Release).ok();
    }

    Ok(())
}

fn position_window_at_cursor(window: &tauri::WebviewWindow) {
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let monitor_width = (size.width as f64 * scale) as i32;
        let monitor_height = (size.height as f64 * scale) as i32;

        let win_width = 340;
        let win_height = 480;

        let x = (monitor_width / 2 - win_width / 2).max(0);
        let y = (monitor_height / 3).max(0);

        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x,
            y,
        })).ok();
    }
}
