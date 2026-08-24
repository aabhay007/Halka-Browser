use crate::database::DatabaseManager;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visited_at: String,
}

pub struct HistoryManager;

impl HistoryManager {
    pub fn add_entry(db: &DatabaseManager, url: &str, title: &str) -> Result<(), String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO history (url, title, visited_at) VALUES (?1, ?2, ?3)",
            params![url, title, now],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_history(db: &DatabaseManager, limit: usize) -> Result<Vec<HistoryEntry>, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, url, title, visited_at FROM history ORDER BY id DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;

        let history_iter = stmt
            .query_map(params![limit as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2).unwrap_or_default(),
                    visited_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for entry in history_iter {
            if let Ok(item) = entry {
                entries.push(item);
            }
        }
        Ok(entries)
    }

    pub fn delete_entry(db: &DatabaseManager, id: i64) -> Result<(), String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_history(db: &DatabaseManager) -> Result<(), String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM history", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
