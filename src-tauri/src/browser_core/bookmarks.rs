use crate::database::DatabaseManager;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub created_at: String,
}

pub struct BookmarksManager;

impl BookmarksManager {
    pub fn add_bookmark(db: &DatabaseManager, url: &str, title: &str, favicon: Option<&str>) -> Result<BookmarkEntry, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO bookmarks (url, title, favicon, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![url, title, favicon, now],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        Ok(BookmarkEntry {
            id,
            url: url.to_string(),
            title: title.to_string(),
            favicon: favicon.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn remove_bookmark(db: &DatabaseManager, url: &str) -> Result<(), String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM bookmarks WHERE url = ?1", params![url])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn is_bookmarked(db: &DatabaseManager, url: &str) -> Result<bool, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bookmarks WHERE url = ?1",
                params![url],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count > 0)
    }

    pub fn get_bookmarks(db: &DatabaseManager) -> Result<Vec<BookmarkEntry>, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, url, title, favicon, created_at FROM bookmarks ORDER BY id DESC")
            .map_err(|e| e.to_string())?;

        let bookmark_iter = stmt
            .query_map([], |row| {
                Ok(BookmarkEntry {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    favicon: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for item in bookmark_iter {
            if let Ok(bm) = item {
                entries.push(bm);
            }
        }
        Ok(entries)
    }
}
