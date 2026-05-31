use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardRecord {
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
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    #[serde(default)]
    pub record_count: i64,
}
