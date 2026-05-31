use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::database::models::{ClipboardRecord, Tag};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub export_date: String,
    pub records: Vec<ExportRecord>,
    pub tags: Vec<Tag>,
    pub record_tags: Vec<RecordTagLink>,
    pub settings: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecord {
    pub id: String,
    pub record_type: String,
    pub content_hash: String,
    pub content: String,
    pub thumbnail_path: Option<String>,
    pub title: String,
    pub is_starred: bool,
    pub copy_count: i64,
    pub source_app: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTagLink {
    pub record_id: String,
    pub tag_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported_records: usize,
    pub imported_tags: usize,
    pub imported_links: usize,
    pub skipped_duplicates: usize,
}

#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    app: AppHandle,
    path: String,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let encryption = state.encryption.lock().map_err(|e| e.to_string())?;

    let records = db.get_all_records_for_index().map_err(|e| e.to_string())?;
    let tags = db.get_tags().map_err(|e| e.to_string())?;
    let record_tags = db.get_all_record_tags().map_err(|e| e.to_string())?;
    let settings = db.get_settings_raw().map_err(|e| e.to_string())?;

    let export_records: Vec<ExportRecord> = records
        .iter()
        .map(|r| ExportRecord {
            id: r.id.clone(),
            record_type: r.record_type.clone(),
            content_hash: r.content_hash.clone(),
            content: r.content.clone(),
            thumbnail_path: r.thumbnail_path.clone(),
            title: r.title.clone(),
            is_starred: r.is_starred,
            copy_count: r.copy_count,
            source_app: r.source_app.clone(),
            created_at: r.created_at.clone(),
            updated_at: r.updated_at.clone(),
            last_used_at: r.last_used_at.clone(),
        })
        .collect();

    let export_data = ExportData {
        version: "1.0".to_string(),
        export_date: Utc::now().to_rfc3339(),
        records: export_records,
        tags,
        record_tags,
        settings,
    };

    let json = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    let encrypted = encryption.encrypt_data_for_export(json.as_bytes())?;

    let export_path = PathBuf::from(path);
    if let Some(parent) = export_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let export_json = serde_json::json!({
        "encrypted_data": encrypted,
        "format_version": "1.0"
    });

    let final_json = serde_json::to_string_pretty(&export_json).map_err(|e| e.to_string())?;
    fs::write(&export_path, final_json).map_err(|e| e.to_string())?;

    Ok(format!(
        "导出成功：{} 条记录，{} 个标签",
        records.len(),
        tags.len()
    ))
}

#[tauri::command]
pub async fn import_data(
    state: State<'_, AppState>,
    _app: AppHandle,
    path: String,
) -> Result<ImportResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let encryption = state.encryption.lock().map_err(|e| e.to_string())?;

    let file_content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let wrapper: serde_json::Value =
        serde_json::from_str(&file_content).map_err(|e| e.to_string())?;

    let encrypted_data = wrapper["encrypted_data"]
        .as_str()
        .ok_or("Missing encrypted_data field")?;

    let decrypted_bytes = encryption.decrypt_data_for_import(encrypted_data)?;
    let json_str = String::from_utf8(decrypted_bytes).map_err(|e| e.to_string())?;
    let export_data: ExportData = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;

    let mut imported_records = 0;
    let mut imported_tags = 0;
    let mut imported_links = 0;
    let mut skipped_duplicates = 0;

    for record in &export_data.records {
        match db.get_record_by_id(&record.id) {
            Ok(_) => {
                skipped_duplicates += 1;
            }
            Err(_) => {
                db.import_record(record).map_err(|e| e.to_string())?;
                imported_records += 1;
            }
        }
    }

    for tag in &export_data.tags {
        match db.get_tag_by_id(&tag.id) {
            Ok(_) => {
                skipped_duplicates += 1;
            }
            Err(_) => {
                db.import_tag(tag).map_err(|e| e.to_string())?;
                imported_tags += 1;
            }
        }
    }

    for link in &export_data.record_tags {
        db.import_record_tag_link(link).map_err(|e| e.to_string())?;
        imported_links += 1;
    }

    for (key, value) in &export_data.settings {
        db.import_setting(key, value).map_err(|e| e.to_string())?;
    }

    Ok(ImportResult {
        imported_records,
        imported_tags,
        imported_links,
        skipped_duplicates,
    })
}
