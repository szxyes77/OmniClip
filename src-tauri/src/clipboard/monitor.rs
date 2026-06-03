use crate::clipboard::models::ClipboardContent;
use crate::AppState;
use arboard::Clipboard;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri::AppHandle;
use tokio::time;

pub fn start_monitoring(app: &AppHandle) -> Result<(), String> {
    let app_handle = app.clone();
    let last_hash: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");

        rt.block_on(async move {
            let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
            let mut interval = time::interval(Duration::from_millis(500));

            loop {
                interval.tick().await;

                if let Some(text) = read_text(&mut clipboard) {
                    let hash = compute_hash(&text);

                    let should_skip = {
                        let guard = last_hash.lock().unwrap();
                        guard.as_ref() == Some(&hash)
                    };

                    if should_skip {
                        continue;
                    }

                    {
                        let mut guard = last_hash.lock().unwrap();
                        *guard = Some(hash.clone());
                    }

                    if let Some(state) = app_handle.try_state::<AppState>() {
                        if !crate::tray::is_monitoring() {
                            continue;
                        }

                        let db = state.db.lock().unwrap();

                        let content = ClipboardContent::from_text(&text);
                        let record = db.insert_record(
                            &content.record_type.to_string(),
                            &hash,
                            &content.content,
                            &content.title,
                            None,
                        ).ok();

                        // Try to broadcast via SSE if sync server is running
                        if let Some(record) = &record {
                            let server_guard = state.sync_server.lock();
                            if let Ok(guard) = server_guard {
                                if let Some(server) = guard.as_ref() {
                                    let broadcaster = server.get_broadcaster();
                                    let json = serde_json::to_string(record).unwrap_or_default();
                                    broadcaster.broadcast("clipboard-update", &json);
                                    println!("[Clipboard] New clip detected, broadcast via SSE. hash={}", &hash[..8]);
                                }
                            }
                            drop(db);

                            println!("[Clipboard] New clip detected, emitting event. hash={}", &hash[..8]);
                            let _ = app_handle.emit_to("main", "clipboard-update", record);
                        }
                    }
                }
            }
        });
    });

    Ok(())
}

fn read_text(clipboard: &mut Clipboard) -> Option<String> {
    clipboard.get_text().ok().filter(|t| !t.trim().is_empty())
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
