mod clipboard;
mod database;
mod tray;
mod sync;

use database::DatabaseManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Mutex<DatabaseManager>>,
    pub sync_server: Arc<Mutex<Option<sync::SyncServer>>>,
}

#[tauri::command]
async fn get_clipboard_records(
    state: tauri::State<'_, AppState>,
    limit: usize,
    offset: usize,
) -> Result<Vec<database::ClipboardRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let records = db.get_records(limit, offset).map_err(|e| e.to_string())?;
    Ok(records)
}

#[tauri::command]
async fn search_records(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: u32,
) -> Result<Vec<database::ClipboardRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let records = db.search_records(&query, limit).map_err(|e| e.to_string())?;
    Ok(records)
}

#[tauri::command]
async fn toggle_star_record(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_star(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_record(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_record(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn paste_record(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    use enigo::{Enigo, Key, Keyboard, Settings};

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = db.get_record_by_id(&id).map_err(|e| e.to_string())?;
    db.increment_copy_count(&id).map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    app.clipboard()
        .write_text(&record.content)
        .map_err(|e| e.to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let _ = enigo.key(Key::Control, enigo::Direction::Press);
    std::thread::sleep(std::time::Duration::from_millis(30));
    let _ = enigo.key(Key::Unicode('v'), enigo::Direction::Click);
    std::thread::sleep(std::time::Duration::from_millis(30));
    let _ = enigo.key(Key::Control, enigo::Direction::Release);

    std::thread::sleep(std::time::Duration::from_millis(100));

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }

    Ok(())
}

#[tauri::command]
async fn get_tags(state: tauri::State<'_, AppState>) -> Result<Vec<database::Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tags().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_tag(
    state: tauri::State<'_, AppState>,
    name: String,
    color: String,
) -> Result<database::Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_tag(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_all(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_monitor() -> Result<bool, String> {
    let current = crate::tray::is_monitoring();
    crate::tray::toggle_monitoring();
    Ok(!current)
}

#[tauri::command]
async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_local_ip() -> Result<String, String> {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return Ok(addr.ip().to_string());
            }
        }
    }
    Ok("127.0.0.1".to_string())
}

#[tauri::command]
async fn start_sync(
    state: tauri::State<'_, AppState>,
    port: u16,
) -> Result<bool, String> {
    let server_guard = state.sync_server.lock().map_err(|e| e.to_string())?;
    if server_guard.is_some() {
        return Ok(true);
    }
    drop(server_guard);

    let db = state.db.clone();
    let server = sync::SyncServer::start(port, db);

    let mut server_guard = state.sync_server.lock().map_err(|e| e.to_string())?;
    *server_guard = Some(server);
    Ok(true)
}

#[tauri::command]
async fn stop_sync(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let mut server_guard = state.sync_server.lock().map_err(|e| e.to_string())?;
    if let Some(_server) = server_guard.take() {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn sync_send_to_peer(
    state: tauri::State<'_, AppState>,
    peer_ip: String,
    peer_port: u16,
    text: String,
) -> Result<(), String> {
    let db = state.db.clone();
    // Get broadcaster from sync server if available
    let server_guard = state.sync_server.lock().map_err(|e| e.to_string())?;
    if let Some(server) = server_guard.as_ref() {
        let broadcaster = server.get_broadcaster();
        sync::SyncServer::send_text_to_peer(&peer_ip, peer_port, &text, &db, &broadcaster)
    } else {
        // Sync not started, just store locally
        println!("[Sync] Sync not started, storing locally");
        let content_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let title = if text.len() > 50 { &text[..50] } else { &text };
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        db_lock.insert_record("text", &content_hash, &text, title, None).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Called by clipboard monitor when new clipboard content is detected.
/// Stores to DB and broadcasts via SSE to mobile clients.
#[tauri::command]
async fn sync_clipboard_update(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let db = state.db.clone();
    let server_guard = state.sync_server.lock().map_err(|e| e.to_string())?;
    if let Some(server) = server_guard.as_ref() {
        let broadcaster = server.get_broadcaster();
        let content_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let title = if text.len() > 50 { &text[..50] } else { &text };
        
        // Store to database
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        if let Ok(record) = db_lock.insert_record("text", &content_hash, &text, title, None) {
            drop(db_lock);
            // Broadcast via SSE
            let json = serde_json::to_string(&record).unwrap_or_default();
            broadcaster.broadcast("clipboard-update", &json);
            println!("[Sync] Clipboard update broadcast via SSE");
            Ok(())
        } else {
            drop(db_lock);
            println!("[Sync] Clipboard update: duplicate content, skipping");
            Ok(())
        }
    } else {
        // Sync not started, just store locally
        let content_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let title = if text.len() > 50 { &text[..50] } else { &text };
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let _ = db_lock.insert_record("text", &content_hash, &text, title, None);
        Ok(())
    }
}

#[tauri::command]
async fn sync_request_records(
    peer_ip: String,
    peer_port: u16,
) -> Result<String, String> {
    sync::SyncServer::request_records_from_peer(&peer_ip, peer_port)
}

#[tauri::command]
async fn get_pair_code() -> Result<String, String> {
    Ok(sync::SyncServer::get_pair_code())
}

#[tauri::command]
async fn verify_peer_pair_code(code: String) -> Result<bool, String> {
    Ok(sync::SyncServer::verify_pair_code(&code))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.app_handle();
            let db = DatabaseManager::new(&app_handle)?;

            let db_arc = Arc::new(Mutex::new(db));

            let state = AppState {
                db: db_arc.clone(),
                sync_server: Arc::new(Mutex::new(None)),
            };
            app.manage(state);

            clipboard::start_monitoring(&app_handle)?;
            tray::setup_system_tray(&app_handle)?;

            let settings = db_arc.lock().unwrap();
            let sync_enabled = settings.get_setting("sync_enabled").unwrap_or_default() == "true";
            let sync_port: u16 = settings.get_setting("sync_port").ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(18910);
            drop(settings);

            if sync_enabled {
                let app_state = app.state::<AppState>();
                let mut server_guard = app_state.sync_server.lock().unwrap();
                *server_guard = Some(sync::SyncServer::start(sync_port, db_arc.clone()));
            }

            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle_clone = app_handle.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app_handle_clone.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_records,
            search_records,
            toggle_star_record,
            delete_record,
            paste_record,
            get_tags,
            create_tag,
            delete_tag,
            clear_all,
            toggle_monitor,
            get_setting,
            set_setting,
            get_local_ip,
            start_sync,
            stop_sync,
            sync_send_to_peer,
            sync_clipboard_update,
            sync_request_records,
            get_pair_code,
            verify_peer_pair_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniClip");
}
