use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS clipboard_records (
            id TEXT PRIMARY KEY,
            record_type TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            content TEXT NOT NULL,
            thumbnail_path TEXT,
            title TEXT DEFAULT '',
            is_starred INTEGER DEFAULT 0,
            copy_count INTEGER DEFAULT 0,
            source_app TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_used_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_records_created ON clipboard_records(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_records_starred ON clipboard_records(is_starred);
        CREATE INDEX IF NOT EXISTS idx_records_hash ON clipboard_records(content_hash);

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT DEFAULT '#6366f1',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS record_tags (
            record_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (record_id, tag_id),
            FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    ")?;

    Ok(())
}
