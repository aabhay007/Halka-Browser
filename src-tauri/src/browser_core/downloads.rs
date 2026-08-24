use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub path: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub state: String, // "downloading", "completed", "failed"
}

#[allow(dead_code)]
pub struct DownloadManager {
    downloads: Arc<Mutex<Vec<DownloadItem>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            downloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_download(&self, filename: String, url: String, path: String, total_bytes: u64) -> DownloadItem {
        let id = format!("dl_{}", chrono::Utc::now().timestamp_millis());
        let item = DownloadItem {
            id,
            filename,
            url,
            path,
            total_bytes,
            downloaded_bytes: 0,
            state: "completed".to_string(),
        };

        let mut list = self.downloads.lock().unwrap();
        list.push(item.clone());
        item
    }

    pub fn get_downloads(&self) -> Vec<DownloadItem> {
        self.downloads.lock().unwrap().clone()
    }
}
