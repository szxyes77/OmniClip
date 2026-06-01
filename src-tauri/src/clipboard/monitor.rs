use crate::clipboard::models::{ClipboardType, ClipboardContent};
use crate::database::models::ClipboardRecord;
use crate::AppState;
use arboard::Clipboard;
use image::imageops;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tauri::AppHandle;
use tokio::time;

static LAST_HASH: std::sync::OnceLock<Arc<std::sync::Mutex<Option<String>>>> = std::sync::OnceLock::new();

pub fn start_monitoring(app: &AppHandle) -> Result<(), String> {
    let app_handle = app.clone();
    let _ = LAST_HASH.get_or_init(|| Arc::new(std::sync::Mutex::new(None)));
    let hash_store = LAST_HASH.get().unwrap().clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");

        rt.block_on(async move {
            let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
            let mut interval = time::interval(Duration::from_millis(800));

            loop {
                interval.tick().await;

                if !crate::tray::setup::is_monitoring() {
                    continue;
                }

                if let Some(content) = read_clipboard(&mut clipboard) {
                    let hash = compute_hash(&content.content);

                    let should_skip = {
                        let guard = hash_store.lock().unwrap();
                        guard.as_ref() == Some(&hash)
                    };

                    if should_skip {
                        continue;
                    }

                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let db = state.db.lock().unwrap();
                        let settings = db.get_settings().unwrap_or_default();

                        let source_app = get_source_app();
                        let should_ignore = if let Some(source) = source_app {
                            settings.ignore_apps.contains(&source)
                        } else {
                            false
                        };

                        if should_ignore {
                            continue;
                        }

                        let encryption = state.encryption.lock().unwrap();
                        let encrypted_content = encryption
                            .encrypt_string(&content.content)
                            .expect("Failed to encrypt");

                        let record = db.insert_record(
                            &content.record_type.to_string(),
                            &hash,
                            &encrypted_content,
                            &content.title,
                            content.thumbnail_path.as_deref(),
                        ).ok();

                        let cleanup_days = settings.auto_cleanup_days;
                        let keep_starred = settings.keep_starred;
                        let max_count = settings.max_history_count;

                        drop(db);
                        drop(encryption);

                        if let Some(record) = record {
                            {
                                let search = state.search.lock().unwrap();
                                search.add_to_index(record.clone()).await;
                            }

                            let mut display_record = record.clone();
                            {
                                let encryption = state.encryption.lock().unwrap();
                                if display_record.record_type == "text" {
                                    if let Ok(decrypted) = encryption.decrypt_string(&display_record.content) {
                                        display_record.content = decrypted;
                                    }
                                }
                            }
                            emit_clipboard_update(&app_handle, &display_record).ok();

                            let mut guard = hash_store.lock().unwrap();
                            *guard = Some(hash);

                            let db = state.db.lock().unwrap();
                            let _ = db.cleanup_old_records(cleanup_days, keep_starred);
                            let _ = db.enforce_max_records(max_count, keep_starred);
                        }
                    }
                }
            }
        });
    });

    Ok(())
}

fn read_clipboard(clipboard: &mut Clipboard) -> Option<ClipboardContent> {
    if let Ok(text) = clipboard.get_text() {
        if !text.trim().is_empty() {
            return Some(ClipboardContent::from_text(&text));
        }
    }

    if let Ok(image) = clipboard.get_image() {
        if !image.bytes.is_empty() {
            return ClipboardContent::from_image(
                &image.bytes,
                image.width,
                image.height,
            );
        }
    }

    None
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn emit_clipboard_update(app: &AppHandle, record: &ClipboardRecord) -> Result<(), String> {
    app.emit("clipboard-update", record).map_err(|e| e.to_string())
}

fn get_source_app() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("powershell")
            .args(&[
                "-Command",
                "(Get-Process -Id (Get-Process | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } | Sort-Object StartTime -Descending | Select-Object -First 1).Id).ProcessName"
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("osascript")
            .args(&["-e", "name of first application process whose frontmost is true"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xdotool")
            .args(&["getactivewindow", "getwindowname"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}
