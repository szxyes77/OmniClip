mod clipboard;
mod config;
mod database;
mod encryption;
mod overlay;
mod search;
mod tray;
mod data_export;

use database::DatabaseManager;
use encryption::EncryptionManager;
use search::SearchEngine;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State, Emitter};

pub struct AppState {
    pub db: Arc<Mutex<DatabaseManager>>,
    pub encryption: Arc<Mutex<EncryptionManager>>,
    pub search: Arc<Mutex<SearchEngine>>,
}

fn decrypt_record_content(
    mut record: database::models::ClipboardRecord,
    encryption: &EncryptionManager,
) -> database::models::ClipboardRecord {
    if record.record_type == "text" {
        if let Ok(decrypted) = encryption.decrypt_string(&record.content) {
            record.content = decrypted;
        }
    }
    record
}

#[tauri::command]
async fn get_clipboard_records(
    state: State<'_, AppState>,
    limit: usize,
    offset: usize,
) -> Result<Vec<database::models::ClipboardRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let records = db.get_records(limit, offset).map_err(|e| e.to_string())?;
    let encryption = state.encryption.lock().map_err(|e| e.to_string())?;
    Ok(records.into_iter().map(|r| decrypt_record_content(r, &encryption)).collect())
}

#[tauri::command]
async fn get_history(
    state: State<'_, AppState>,
    search: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<database::models::ClipboardRecord>, String> {
    if search.trim().is_empty() {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let records = db.get_records(limit as usize, offset as usize).map_err(|e| e.to_string())?;
        let encryption = state.encryption.lock().map_err(|e| e.to_string())?;
        Ok(records.into_iter().map(|r| decrypt_record_content(r, &encryption)).collect())
    } else {
        let search_engine = state.search.lock().map_err(|e| e.to_string())?;
        let results = search_engine.search(&search, limit).await;
        let encryption = state.encryption.lock().map_err(|e| e.to_string())?;
        Ok(results.into_iter().map(|r| decrypt_record_content(r, &encryption)).collect())
    }
}

#[tauri::command]
async fn search_clips(
    state: State<'_, AppState>,
    query: String,
    limit: u32,
) -> Result<Vec<database::models::ClipboardRecord>, String> {
    let search_engine = state.search.lock().map_err(|e| e.to_string())?;
    let results = search_engine.search(&query, limit).await;
    let encryption = state.encryption.lock().map_err(|e| e.to_string())?;
    Ok(results.into_iter().map(|r| decrypt_record_content(r, &encryption)).collect())
}

#[tauri::command]
async fn search_records(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<database::models::ClipboardRecord>, String> {
    let search_engine = state.search.lock().map_err(|e| e.to_string())?;
    let results = search_engine.search(&query, 100).await;
    let encryption = state.encryption.lock().map_err(|e| e.to_string())?;
    Ok(results.into_iter().map(|r| decrypt_record_content(r, &encryption)).collect())
}

#[tauri::command]
async fn toggle_star_record(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_star(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn star_item(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_star(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_record(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_record(&id).map_err(|e| e.to_string())?;
    drop(db);
    let search = state.search.lock().map_err(|e| e.to_string())?;
    search.remove_from_index(&id).await;
    Ok(())
}

#[tauri::command]
async fn delete_item(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_record(&id).map_err(|e| e.to_string())?;
    drop(db);
    let search = state.search.lock().map_err(|e| e.to_string())?;
    search.remove_from_index(&id).await;
    Ok(())
}

#[tauri::command]
async fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<database::models::Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tags(
    state: State<'_, AppState>,
) -> Result<Vec<database::models::Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tags().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_tags_to_record(
    state: State<'_, AppState>,
    record_id: String,
    tag_ids: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_tags_to_record(&record_id, &tag_ids).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_tags(
    state: State<'_, AppState>,
    id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut tag_ids = Vec::new();
    for tag_name in &tags {
        match db.create_tag(tag_name, "#6366f1") {
            Ok(tag) => tag_ids.push(tag.id),
            Err(_) => {
                let existing = db.get_tags().map_err(|e| e.to_string())?;
                if let Some(found) = existing.iter().find(|t| t.name == *tag_name) {
                    tag_ids.push(found.id.clone());
                }
            }
        }
    }

    db.add_tags_to_record(&id, &tag_ids).map_err(|e| e.to_string())
}

#[tauri::command]
async fn rename_tag(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<database::models::Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rename_tag(&id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_tag(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings(
    state: State<'_, AppState>,
) -> Result<config::settings::AppSettings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_settings(
    state: State<'_, AppState>,
    settings: config::settings::SettingsUpdate,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_floating_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_floating_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("floating") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn paste_record(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = db.get_record_by_id(&id).map_err(|e| e.to_string())?;

    match record.record_type.as_str() {
        "text" | "files" => {
            let decryption = state.encryption.lock().map_err(|e| e.to_string())?;
            let content = decryption.decrypt_string(&record.content).map_err(|e| e.to_string())?;
            app.clipboard().write_text(&content).map_err(|e| e.to_string())?;
        }
        "image" => {
            let content = record.content;
            if let Ok(img) = image::open(&content) {
                app.clipboard().write_image(
                    tauri::image::Image::new(
                        img.to_rgba8().into_raw().into(),
                        img.width(),
                        img.height(),
                    )
                ).map_err(|e| e.to_string())?;
            }
        }
        _ => return Err("Unsupported record type".to_string()),
    }

    db.increment_copy_count(&id).map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("floating") {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn show_overlay_window(app: tauri::AppHandle) -> Result<(), String> {
    overlay::show_overlay(&app)
}

#[tauri::command]
async fn hide_overlay_window(app: tauri::AppHandle) -> Result<(), String> {
    overlay::hide_overlay(&app)
}

#[tauri::command]
async fn set_overlay_passthrough(
    app: tauri::AppHandle,
    enable: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        overlay::set_cursor_passthrough(&window, enable)?;
    }
    Ok(())
}

#[tauri::command]
async fn paste_from_overlay(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = db.get_record_by_id(&id).map_err(|e| e.to_string())?;
    let record_type = record.record_type.clone();
    let record_content = record.content.clone();

    db.increment_copy_count(&id).map_err(|e| e.to_string())?;
    drop(db);

    let decryption = state.encryption.lock().map_err(|e| e.to_string())?;

    match record_type.as_str() {
        "text" | "files" => {
            let content = decryption.decrypt_string(&record_content).map_err(|e| e.to_string())?;
            overlay::paste_with_simulated_keys(&app, &content)
        }
        "image" => {
            let image_path = record_content;
            if let Ok(img) = image::open(&image_path) {
                use tauri_plugin_clipboard_manager::ClipboardExt;
                app.clipboard().write_image(
                    tauri::image::Image::new(
                        img.to_rgba8().into_raw().into(),
                        img.width(),
                        img.height(),
                    )
                ).map_err(|e| e.to_string())?;

                overlay::hide_overlay(&app).ok();
                std::thread::sleep(std::time::Duration::from_millis(80));

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
            } else {
                Err("Failed to open image".to_string())
            }
        }
        _ => Err("Unsupported record type".to_string()),
    }
}

#[tauri::command]
async fn open_save_dialog(
    app: tauri::AppHandle,
    default_path: String,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let result = app.dialog().file()
        .set_title("Save OmniClip Backup")
        .add_filter("JSON", &["json"])
        .set_file_name(&default_path)
        .save_file();
    
    result.ok_or("No path selected".to_string())
}

#[tauri::command]
async fn open_file_dialog(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let result = app.dialog().file()
        .set_title("Import OmniClip Backup")
        .add_filter("JSON", &["json"])
        .pick_file();
    
    result.ok_or("No file selected".to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.app_handle();
            let encryption = EncryptionManager::new(&app_handle)?;
            let db = DatabaseManager::new(&app_handle, &encryption)?;
            let encryption_arc = Arc::new(Mutex::new(encryption));
            let search = SearchEngine::new(app_handle.clone(), encryption_arc.clone());

            let state = AppState {
                db: Arc::new(Mutex::new(db)),
                encryption: encryption_arc,
                search: Arc::new(Mutex::new(search)),
            };
            app.manage(state);

            let state = app.state::<AppState>();
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let records = db.get_all_records_for_index().map_err(|e| e.to_string())?;
            let search = state.search.lock().map_err(|e| e.to_string())?;
            drop(db);
            drop(search);
            let search = state.search.lock().map_err(|e| e.to_string())?;
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            rt.block_on(async {
                search.build_index(records).await;
            });
            drop(search);

            clipboard::monitor::start_monitoring(&app_handle)?;
            tray::setup::setup_system_tray(&app_handle)?;
            overlay::setup_overlay(&app_handle)?;

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
            get_history,
            search_clips,
            toggle_star_record,
            star_item,
            delete_record,
            delete_item,
            create_tag,
            get_tags,
            add_tags_to_record,
            add_tags,
            rename_tag,
            delete_tag,
            get_settings,
            update_settings,
            show_floating_window,
            hide_floating_window,
            paste_record,
            show_overlay_window,
            hide_overlay_window,
            set_overlay_passthrough,
            paste_from_overlay,
            data_export::export_data,
            data_export::import_data,
            open_save_dialog,
            open_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OmniClip");
}
