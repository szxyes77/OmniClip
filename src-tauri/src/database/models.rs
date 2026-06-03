use serde::{Deserialize, Serialize};
use rusqlite::{Row, Result};

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
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

impl ClipboardRecord {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            record_type: row.get(1)?,
            content_hash: row.get(2)?,
            content: row.get(3)?,
            thumbnail_path: row.get(4)?,
            title: row.get(5)?,
            is_starred: row.get::<_, i32>(6)? != 0,
            copy_count: row.get(7)?,
            source_app: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            last_used_at: row.get(11).ok(),
            tags: None,
        })
    }
}
