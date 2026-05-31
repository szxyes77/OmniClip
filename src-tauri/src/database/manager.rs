use crate::database::migrations;
use crate::database::models::{ClipboardRecord, Tag};
use crate::encryption::EncryptionManager;
use chrono::Utc;
use dirs;
use rusqlite::{params, Connection, Result, Row};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use uuid::Uuid;

pub struct DatabaseManager {
    pub conn: Connection,
    data_dir: PathBuf,
}

impl DatabaseManager {
    pub fn new(app: &AppHandle, _encryption: &EncryptionManager) -> Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("OmniClip");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let db_path = data_dir.join("omniclip.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        migrations::run_migrations(&conn).map_err(|e| e.to_string())?;

        Self::init_default_settings(&conn).map_err(|e| e.to_string())?;

        Ok(Self { conn, data_dir })
    }

    fn init_default_settings(conn: &Connection) -> Result<()> {
        let defaults = [
            ("max_history_count", "1000"),
            ("auto_cleanup_days", "30"),
            ("keep_starred", "true"),
            ("global_shortcut", "CmdOrCtrl+Shift+V"),
            ("theme", "system"),
        ];

        for (key, value) in &defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }

        Ok(())
    }

    pub fn get_records(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records 
             ORDER BY created_at DESC 
             LIMIT ?1 OFFSET ?2"
        )?;

        let records = stmt.query_map(params![limit, offset], |row| {
            Self::row_to_record(row)
        })?;

        records.collect::<Result<Vec<_>>>()
    }

    pub fn get_all_records_for_index(&self) -> Result<Vec<ClipboardRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records 
             ORDER BY created_at DESC"
        )?;

        let records = stmt.query_map([], |row| Self::row_to_record(row))?;

        records.collect::<Result<Vec<_>>>()
    }

    pub fn get_record_by_id(&self, id: &str) -> Result<ClipboardRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, content_hash, content, thumbnail_path, title, 
             is_starred, copy_count, source_app, created_at, updated_at, last_used_at 
             FROM clipboard_records 
             WHERE id = ?1"
        )?;

        stmt.query_row(params![id], |row| Self::row_to_record(row))
    }

    pub fn insert_record(
        &self,
        record_type: &str,
        content_hash: &str,
        content: &str,
        title: &str,
        thumbnail_path: Option<&str>,
    ) -> Result<ClipboardRecord> {
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

    pub fn toggle_star(&self, id: &str) -> Result<bool> {
        let current = self.get_record_by_id(id)?;
        let new_state = !current.is_starred;

        self.conn.execute(
            "UPDATE clipboard_records SET is_starred = ?1, updated_at = ?2 WHERE id = ?3",
            params![if new_state { 1 } else { 0 }, Utc::now().to_rfc3339(), id],
        )?;

        Ok(new_state)
    }

    pub fn delete_record(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM clipboard_records WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    pub fn increment_copy_count(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE clipboard_records SET copy_count = copy_count + 1, updated_at = ?1, last_used_at = ?2 WHERE id = ?3",
            params![now, now, id],
        )?;

        Ok(())
    }

    pub fn update_last_used(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE clipboard_records SET last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn create_tag(&self, name: &str, color: &str) -> Result<Tag> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, color, now],
        )?;

        Ok(Tag {
            id,
            name: name.to_string(),
            color: color.to_string(),
            created_at: now,
        })
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, 
             COUNT(rt.record_id) as record_count
             FROM tags t
             LEFT JOIN record_tags rt ON t.id = rt.tag_id
             GROUP BY t.id
             ORDER BY t.name"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                record_count: row.get(4)?,
            })
        })?;

        tags.collect::<Result<Vec<_>>>()
    }

    pub fn add_tags_to_record(&self, record_id: &str, tag_ids: &[String]) -> Result<()> {
        for tag_id in tag_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                params![record_id, tag_id],
            )?;
        }

        Ok(())
    }

    pub fn rename_tag(&self, id: &str, new_name: &str) -> Result<Tag> {
        self.conn.execute(
            "UPDATE tags SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        )?;

        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1"
        )?;

        stmt.query_row(params![id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
    }

    pub fn delete_tag(&self, id: &str) -> Result<()> {
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

    pub fn get_settings(&self) -> Result<crate::config::settings::AppSettings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut settings_map = std::collections::HashMap::new();
        for row in rows {
            let (key, value) = row?;
            settings_map.insert(key, value);
        }

        let max_history_count = settings_map
            .get("max_history_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let auto_cleanup_days = settings_map
            .get("auto_cleanup_days")
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let keep_starred = settings_map
            .get("keep_starred")
            .map(|v| v == "true")
            .unwrap_or(true);

        let global_shortcut = settings_map
            .get("global_shortcut")
            .cloned()
            .unwrap_or_else(|| "CmdOrCtrl+Shift+V".to_string());

        let theme = settings_map
            .get("theme")
            .cloned()
            .unwrap_or_else(|| "system".to_string());

        let ignore_apps = settings_map
            .get("ignore_apps")
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or_else(|| vec![]);

        let master_password_hash = settings_map
            .get("master_password_hash")
            .cloned()
            .unwrap_or_else(|| "".to_string());

        Ok(crate::config::settings::AppSettings {
            max_history_count,
            auto_cleanup_days,
            keep_starred,
            global_shortcut,
            ignore_apps,
            theme,
            master_password_hash,
        })
    }

    pub fn get_settings_raw(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<Result<Vec<_>>>()
    }

    pub fn import_record(&self, record: &crate::data_export::ExportRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO clipboard_records 
             (id, record_type, content_hash, content, thumbnail_path, title, is_starred, copy_count, source_app, created_at, updated_at, last_used_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id, record.record_type, record.content_hash, record.content,
                record.thumbnail_path, record.title, if record.is_starred { 1 } else { 0 },
                record.copy_count, record.source_app, record.created_at, record.updated_at,
                record.last_used_at
            ],
        )?;
        Ok(())
    }

    pub fn import_tag(&self, tag: &crate::database::models::Tag) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![tag.id, tag.name, tag.color, tag.created_at],
        )?;
        Ok(())
    }

    pub fn import_record_tag_link(&self, link: &crate::data_export::RecordTagLink) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
            params![link.record_id, link.tag_id],
        )?;
        Ok(())
    }

    pub fn import_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_tag_by_id(&self, id: &str) -> Result<Tag> {
        self.conn.query_row(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
            params![id],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    record_count: 0,
                })
            },
        )
    }

    pub fn get_all_record_tags(&self) -> Result<Vec<crate::data_export::RecordTagLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT record_id, tag_id FROM record_tags"
        )?;
        let links = stmt.query_map([], |row| {
            Ok(crate::data_export::RecordTagLink {
                record_id: row.get(0)?,
                tag_id: row.get(1)?,
            })
        })?;
        links.collect::<Result<Vec<_>>>()
    }

    pub fn update_settings(
        &self,
        settings: &crate::config::settings::SettingsUpdate,
    ) -> Result<()> {
        if let Some(val) = settings.max_history_count {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('max_history_count', ?1)",
                params![val.to_string()],
            )?;
        }

        if let Some(val) = settings.auto_cleanup_days {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('auto_cleanup_days', ?1)",
                params![val.to_string()],
            )?;
        }

        if let Some(val) = settings.keep_starred {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('keep_starred', ?1)",
                params![val.to_string()],
            )?;
        }

        if let Some(val) = &settings.global_shortcut {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('global_shortcut', ?1)",
                params![val],
            )?;
        }

        if let Some(val) = &settings.theme {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('theme', ?1)",
                params![val],
            )?;
        }

        if let Some(val) = &settings.master_password_hash {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('master_password_hash', ?1)",
                params![val],
            )?;
        }

        if let Some(val) = &settings.ignore_apps {
            let apps_json = serde_json::to_string(val).map_err(|e| e.to_string())?;
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('ignore_apps', ?1)",
                params![apps_json],
            )?;
        }

        Ok(())
    }

    pub fn cleanup_old_records(&self, days: usize, keep_starred: bool) -> Result<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(days as i64)).to_rfc3339();
        let sql = if keep_starred {
            "DELETE FROM clipboard_records WHERE created_at < ?1 AND is_starred = 0"
        } else {
            "DELETE FROM clipboard_records WHERE created_at < ?1"
        };
        let changes = self.conn.execute(sql, params![cutoff])?;
        self.conn.execute(
            "DELETE FROM record_tags WHERE record_id NOT IN (SELECT id FROM clipboard_records)",
            [],
        )?;
        Ok(changes)
    }

    pub fn enforce_max_records(&self, max_count: usize, keep_starred: bool) -> Result<usize> {
        let total: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM clipboard_records",
            [],
            |row| row.get(0),
        )?;

        if total <= max_count {
            return Ok(0);
        }

        let to_delete = total - max_count;
        let sql = if keep_starred {
            "DELETE FROM clipboard_records WHERE id IN (
                SELECT id FROM clipboard_records WHERE is_starred = 0 
                ORDER BY created_at ASC LIMIT ?1
            )"
        } else {
            "DELETE FROM clipboard_records WHERE id IN (
                SELECT id FROM clipboard_records 
                ORDER BY created_at ASC LIMIT ?1
            )"
        };
        let changes = self.conn.execute(sql, params![to_delete])?;
        self.conn.execute(
            "DELETE FROM record_tags WHERE record_id NOT IN (SELECT id FROM clipboard_records)",
            [],
        )?;
        Ok(changes)
    }

    fn row_to_record(row: &Row) -> Result<ClipboardRecord> {
        Ok(ClipboardRecord {
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
        })
    }
}
