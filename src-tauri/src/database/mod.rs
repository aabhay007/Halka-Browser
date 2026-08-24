use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct DatabaseManager {
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("browser_data.db");
        let manager = Self { db_path };
        manager.init_tables()?;
        Ok(manager)
    }

    pub fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.get_connection()?;
        
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT,
                visited_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                favicon TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT OR IGNORE INTO settings (key, value) VALUES ('search_engine', 'Google');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('home_page', 'https://www.google.com');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('default_zoom', '100');
            ",
        )?;

        Ok(())
    }
}
