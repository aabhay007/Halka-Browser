use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabData {
    pub id: String,
    pub title: String,
    pub url: String,
    pub active: bool,
}

pub struct TabManager {
    tabs: HashMap<String, TabData>,
    tab_order: Vec<String>,
    active_tab_id: Option<String>,
    closed_history: Vec<String>, // Stores URLs of recently closed tabs for Ctrl+Shift+T
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            tab_order: Vec::new(),
            active_tab_id: None,
            closed_history: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a new tab record and return its unique tab_id
    pub fn create_tab(&mut self, url: String, title: Option<String>) -> (String, TabData) {
        let id = format!("tab_{}", self.next_id);
        self.next_id += 1;

        let tab_title = title.unwrap_or_else(|| "New Tab".to_string());

        let tab = TabData {
            id: id.clone(),
            title: tab_title,
            url: url.clone(),
            active: true,
        };

        // Deactivate current active tab
        if let Some(ref current_active) = self.active_tab_id {
            if let Some(t) = self.tabs.get_mut(current_active) {
                t.active = false;
            }
        }

        self.tabs.insert(id.clone(), tab.clone());
        self.tab_order.push(id.clone());
        self.active_tab_id = Some(id.clone());

        (id, tab)
    }

    /// Switch active tab to target tab_id
    pub fn switch_tab(&mut self, target_id: &str) -> bool {
        if !self.tabs.contains_key(target_id) {
            return false;
        }

        for (id, tab) in self.tabs.iter_mut() {
            tab.active = id == target_id;
        }

        self.active_tab_id = Some(target_id.to_string());
        true
    }

    /// Close tab by id, returning the closed tab's URL and the new active tab_id (if any)
    pub fn close_tab(&mut self, tab_id: &str) -> (Option<String>, Option<String>) {
        let closed_url = self.tabs.get(tab_id).map(|t| t.url.clone());

        if let Some(url) = &closed_url {
            self.closed_history.push(url.clone());
        }

        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);

        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = self.tab_order.last().cloned();
            if let Some(ref new_active) = self.active_tab_id {
                if let Some(t) = self.tabs.get_mut(new_active) {
                    t.active = true;
                }
            }
        }

        (closed_url, self.active_tab_id.clone())
    }

    /// Reopen recently closed tab URL if available
    pub fn pop_closed_url(&mut self) -> Option<String> {
        self.closed_history.pop()
    }

    /// Get active tab ID
    pub fn active_tab_id(&self) -> Option<String> {
        self.active_tab_id.clone()
    }

    /// Update title or URL of a specific tab
    pub fn update_tab(&mut self, tab_id: &str, title: Option<String>, url: Option<String>) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            if let Some(t) = title {
                tab.title = t;
            }
            if let Some(u) = url {
                tab.url = u;
            }
        }
    }

    /// Get all tab records in order
    pub fn get_tabs_list(&self) -> Vec<TabData> {
        self.tab_order
            .iter()
            .filter_map(|id| self.tabs.get(id).cloned())
            .collect()
    }

    pub fn tab_count(&self) -> usize {
        self.tab_order.len()
    }
}
