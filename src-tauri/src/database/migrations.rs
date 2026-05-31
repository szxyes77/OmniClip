pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS clipboard_records (
            id TEXT PRIMARY KEY,
            record_type TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            content TEXT,
            thumbnail_path TEXT,
            title TEXT NOT NULL,
            is_starred INTEGER DEFAULT 0,
            copy_count INTEGER DEFAULT 1,
            source_app TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_used_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_hash ON clipboard_records(content_hash);
        CREATE INDEX IF NOT EXISTS idx_starred ON clipboard_records(is_starred);
        CREATE INDEX IF NOT EXISTS idx_created ON clipboard_records(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_last_used ON clipboard_records(last_used_at DESC);

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            color TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS record_tags (
            record_id TEXT REFERENCES clipboard_records(id),
            tag_id TEXT REFERENCES tags(id),
            PRIMARY KEY (record_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "
    )?;

    let _ = conn.execute(
        "ALTER TABLE clipboard_records ADD COLUMN last_used_at TEXT",
        [],
    );

    Ok(())
}
