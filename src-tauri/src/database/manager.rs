use crate::database::models::{ClipboardRecord, Tag};
use chrono::Utc;
use dirs;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use uuid::Uuid;

pub struct DatabaseManager {
    pub conn: Connection,
    data_dir: PathBuf,
}

impl DatabaseManager {
    pub fn new(_app: &AppHandle) -> std::result::Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("OmniClip");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let db_path = data_dir.join("omniclip.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        crate::database::migrations::run_migrations(&conn).map_err(|e| e.to_string())?;

        Self::init_default_settings(&conn).map_err(|e| e.to_string())?;

        Ok(Self { conn, data_dir })
    }

    fn init_default_settings(conn: &Connection) -> rusqlite::Result<()> {
        let defaults = [
            ("max_history_count", "1000"),
            ("auto_cleanup_days", "30"),
            ("keep_starred", "true"),
            ("theme", "light"),
            ("auto_start", "false"),
        ];

        for (key, value) in &defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }

        Ok(())
    }

    pub fn get_records(&self, limit: usize, offset: usize) -> rusqlite::Result<Vec<ClipboardRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records 
             ORDER BY created_at DESC 
             LIMIT ?1 OFFSET ?2"
        )?;

        let records = stmt.query_map(params![limit, offset], |row| {
            ClipboardRecord::from_row(row)
        })?;

        records.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn get_record_by_id(&self, id: &str) -> rusqlite::Result<ClipboardRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records WHERE id = ?1"
        )?;

        stmt.query_row(params![id], |row| ClipboardRecord::from_row(row))
    }

    pub fn insert_record(
        &self,
        record_type: &str,
        content_hash: &str,
        content: &str,
        title: &str,
        thumbnail_path: Option<&str>,
    ) -> rusqlite::Result<ClipboardRecord> {
        let existing = self.conn.query_row(
            "SELECT id FROM clipboard_records WHERE content_hash = ?1",
            params![content_hash],
            |row| row.get::<_, String>(0),
        );

        if let Ok(id) = existing {
            self.conn.execute(
                "UPDATE clipboard_records SET updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), id],
            )?;
            return self.get_record_by_id(&id);
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO clipboard_records 
             (id, record_type, content_hash, content, thumbnail_path, title, is_starred, copy_count, created_at, updated_at, last_used_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 1, ?7, ?8, ?9)",
            params![id, record_type, content_hash, content, thumbnail_path, title, now, now, now],
        )?;

        self.get_record_by_id(&id)
    }

    pub fn toggle_star(&self, id: &str) -> rusqlite::Result<bool> {
        let current = self.get_record_by_id(id)?;
        let new_state = !current.is_starred;

        self.conn.execute(
            "UPDATE clipboard_records SET is_starred = ?1, updated_at = ?2 WHERE id = ?3",
            params![if new_state { 1 } else { 0 }, Utc::now().to_rfc3339(), id],
        )?;

        Ok(new_state)
    }

    pub fn delete_record(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM clipboard_records WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn clear_all(&self) -> rusqlite::Result<usize> {
        let changes = self.conn.execute(
            "DELETE FROM clipboard_records",
            [],
        )?;
        Ok(changes)
    }

    pub fn increment_copy_count(&self, id: &str) -> rusqlite::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE clipboard_records SET copy_count = copy_count + 1, updated_at = ?1, last_used_at = ?2 WHERE id = ?3",
            params![now, now, id],
        )?;
        Ok(())
    }

    pub fn search_records(&self, query: &str, limit: u32) -> rusqlite::Result<Vec<ClipboardRecord>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records 
             WHERE title LIKE ?1 OR content LIKE ?1 
             ORDER BY created_at DESC 
             LIMIT ?2"
        )?;

        let records = stmt.query_map(params![pattern, limit], |row| {
            ClipboardRecord::from_row(row)
        })?;

        records.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn create_tag(&self, name: &str, color: &str) -> rusqlite::Result<Tag> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR IGNORE INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, color, now],
        )?;

        self.get_tag_by_name(name)
    }

    pub fn get_tag_by_name(&self, name: &str) -> rusqlite::Result<Tag> {
        self.conn.query_row(
            "SELECT id, name, color, created_at FROM tags WHERE name = ?1",
            params![name],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
    }

    pub fn get_tags(&self) -> rusqlite::Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, created_at FROM tags ORDER BY name"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        tags.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn add_tags_to_record(&self, record_id: &str, tag_ids: &[String]) -> rusqlite::Result<()> {
        for tag_id in tag_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                params![record_id, tag_id],
            )?;
        }
        Ok(())
    }

    pub fn get_record_tags(&self, record_id: &str) -> rusqlite::Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at 
             FROM tags t 
             JOIN record_tags rt ON t.id = rt.tag_id 
             WHERE rt.record_id = ?1"
        )?;

        let tags = stmt.query_map(params![record_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        tags.collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn delete_tag(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM record_tags WHERE tag_id = ?1",
            params![id],
        )?;

        self.conn.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> rusqlite::Result<String> {
        self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
    }

    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
