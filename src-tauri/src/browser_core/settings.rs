use crate::database::DatabaseManager;
use rusqlite::params;
use std::collections::HashMap;

pub struct SettingsManager;

impl SettingsManager {
    pub fn get_setting(db: &DatabaseManager, key: &str) -> Result<Option<String>, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let res: Result<String, _> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match res {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn set_setting(db: &DatabaseManager, key: &str, value: &str) -> Result<(), String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all_settings(db: &DatabaseManager) -> Result<HashMap<String, String>, String> {
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;

        let mut settings = HashMap::new();
        for r in rows {
            if let Ok((k, v)) = r {
                settings.insert(k, v);
            }
        }
        Ok(settings)
    }
}
