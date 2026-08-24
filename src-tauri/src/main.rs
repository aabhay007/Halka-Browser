// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod browser_core;
mod database;

use browser_core::bookmarks::{BookmarkEntry, BookmarksManager};
use browser_core::history::{HistoryEntry, HistoryManager};
use browser_core::navigation::NavigationManager;
use browser_core::settings::SettingsManager;
use browser_core::tab_manager::{TabData, TabManager};
use database::DatabaseManager;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl, WindowEvent,
};

const TOOLBAR_HEIGHT: f64 = 80.0;

pub struct AppState {
    pub tab_manager: Arc<Mutex<TabManager>>,
    pub db: Arc<DatabaseManager>,
}

fn get_active_content_bounds(window: &tauri::window::Window) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let inner_size = window.inner_size().unwrap_or(tauri::PhysicalSize::new(1280, 800)).to_logical::<f64>(scale);

    let pos = LogicalPosition::new(0.0, TOOLBAR_HEIGHT);
    let size = LogicalSize::new(inner_size.width, (inner_size.height - TOOLBAR_HEIGHT).max(100.0));
    (pos, size)
}

fn emit_tab_state(app: &AppHandle, state: &AppState) {
    let tabs = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.get_tabs_list()
    };
    let _ = app.emit("tab_state_changed", tabs);
}

// --- TAB COMMANDS ---

#[tauri::command]
async fn get_tabs(state: State<'_, AppState>) -> Result<Vec<TabData>, String> {
    let mgr = state.tab_manager.lock().unwrap();
    Ok(mgr.get_tabs_list())
}

#[tauri::command]
async fn create_tab(app: AppHandle, state: State<'_, AppState>, url: Option<String>) -> Result<TabData, String> {
    let target_url = url.unwrap_or_else(|| "https://www.google.com".to_string());
    let parsed_url = NavigationManager::parse_input_to_url(&target_url);
    let initial_title = NavigationManager::extract_display_title(&parsed_url);

    let main_window = app.get_window("main").ok_or("main window not found")?;
    let (pos, size) = get_active_content_bounds(&main_window);

    let (tab_id, tab_data) = {
        let mut mgr = state.tab_manager.lock().unwrap();
        mgr.create_tab(parsed_url.clone(), Some(initial_title.clone()))
    };

    let existing_tabs = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.get_tabs_list()
    };
    for t in existing_tabs {
        if t.id != tab_id {
            if let Some(wv) = app.get_webview(&t.id) {
                let _ = wv.hide();
            }
        }
    }

    let app_handle_load = app.clone();
    let tab_id_load = tab_id.clone();
    let initial_url = WebviewUrl::External(parsed_url.parse().map_err(|e| format!("{}", e))?);
    let webview_builder = WebviewBuilder::new(&tab_id, initial_url)
        .auto_resize()
        .on_page_load(move |_wv, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let current_url = payload.url().as_str().to_string();
                let display_title = NavigationManager::extract_display_title(&current_url);
                let state_ref = app_handle_load.state::<AppState>();
                {
                    let mut mgr = state_ref.tab_manager.lock().unwrap();
                    mgr.update_tab(&tab_id_load, Some(display_title.clone()), Some(current_url.clone()));
                }
                let _ = HistoryManager::add_entry(&state_ref.db, &current_url, &display_title);
                emit_tab_state(&app_handle_load, &state_ref);

                let is_active = {
                    let mgr = state_ref.tab_manager.lock().unwrap();
                    mgr.active_tab_id() == Some(tab_id_load.clone())
                };
                if is_active {
                    let _ = app_handle_load.emit("url_changed", current_url);
                }
            }
        });
    let child_wv = main_window
        .add_child(webview_builder, pos, size)
        .map_err(|e| format!("{}", e))?;

    let _ = child_wv.show();
    let _ = child_wv.set_focus();

    // Record history
    let _ = HistoryManager::add_entry(&state.db, &parsed_url, &initial_title);

    emit_tab_state(&app, &state);
    Ok(tab_data)
}

#[tauri::command]
async fn switch_tab(app: AppHandle, state: State<'_, AppState>, tab_id: String) -> Result<(), String> {
    let main_window = app.get_window("main").ok_or("main window not found")?;
    let (pos, size) = get_active_content_bounds(&main_window);

    let updated = {
        let mut mgr = state.tab_manager.lock().unwrap();
        mgr.switch_tab(&tab_id)
    };

    if !updated {
        return Err("tab not found".into());
    }

    let all_tabs = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.get_tabs_list()
    };

    for t in all_tabs {
        if let Some(wv) = app.get_webview(&t.id) {
            if t.id == tab_id {
                let _ = wv.set_position(pos);
                let _ = wv.set_size(size);
                let _ = wv.show();
                let _ = wv.set_focus();
            } else {
                let _ = wv.hide();
            }
        }
    }

    emit_tab_state(&app, &state);
    Ok(())
}

#[tauri::command]
async fn close_tab(app: AppHandle, state: State<'_, AppState>, tab_id: String) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.close();
    }

    let (_, new_active_id) = {
        let mut mgr = state.tab_manager.lock().unwrap();
        mgr.close_tab(&tab_id)
    };

    if let Some(active_id) = new_active_id {
        let _ = switch_tab(app.clone(), state.clone(), active_id).await;
    } else {
        let _ = create_tab(app.clone(), state.clone(), Some("https://www.google.com".into())).await;
    }

    emit_tab_state(&app, &state);
    Ok(())
}

#[tauri::command]
async fn reopen_tab(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let closed_url = {
        let mut mgr = state.tab_manager.lock().unwrap();
        mgr.pop_closed_url()
    };

    if let Some(url) = closed_url {
        let _ = create_tab(app, state, Some(url)).await;
    }
    Ok(())
}

#[tauri::command]
async fn navigate(app: AppHandle, state: State<'_, AppState>, input: String) -> Result<(), String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    };

    if let Some(tab_id) = active_id {
        let target_url = NavigationManager::parse_input_to_url(&input);
        let display_title = NavigationManager::extract_display_title(&target_url);
        if let Some(content_wv) = app.get_webview(&tab_id) {
            if let Ok(parsed_url) = target_url.parse() {
                let _ = content_wv.navigate(parsed_url);
                {
                    let mut mgr = state.tab_manager.lock().unwrap();
                    mgr.update_tab(&tab_id, Some(display_title.clone()), Some(target_url.clone()));
                }
                let _ = HistoryManager::add_entry(&state.db, &target_url, &display_title);
                let _ = app.emit("url_changed", target_url.clone());
                emit_tab_state(&app, &state);
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn go_back(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(tab_id) = state.tab_manager.lock().unwrap().active_tab_id() {
        if let Some(content_wv) = app.get_webview(&tab_id) {
            let _ = content_wv.eval("window.history.back()");
        }
    }
    Ok(())
}

#[tauri::command]
async fn go_forward(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(tab_id) = state.tab_manager.lock().unwrap().active_tab_id() {
        if let Some(content_wv) = app.get_webview(&tab_id) {
            let _ = content_wv.eval("window.history.forward()");
        }
    }
    Ok(())
}

#[tauri::command]
async fn reload(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(tab_id) = state.tab_manager.lock().unwrap().active_tab_id() {
        if let Some(content_wv) = app.get_webview(&tab_id) {
            let _ = content_wv.reload();
        }
    }
    Ok(())
}

#[tauri::command]
async fn open_devtools(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(tab_id) = state.tab_manager.lock().unwrap().active_tab_id() {
        if let Some(content_wv) = app.get_webview(&tab_id) {
            content_wv.open_devtools();
        }
    }
    Ok(())
}

// --- HISTORY IPC COMMANDS ---

#[tauri::command]
async fn get_history(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<HistoryEntry>, String> {
    HistoryManager::get_history(&state.db, limit.unwrap_or(100))
}

#[tauri::command]
async fn delete_history_entry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    HistoryManager::delete_entry(&state.db, id)
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    HistoryManager::clear_history(&state.db)
}

// --- BOOKMARK IPC COMMANDS ---

#[tauri::command]
async fn add_bookmark(state: State<'_, AppState>, url: String, title: String) -> Result<BookmarkEntry, String> {
    BookmarksManager::add_bookmark(&state.db, &url, &title, None)
}

#[tauri::command]
async fn remove_bookmark(state: State<'_, AppState>, url: String) -> Result<(), String> {
    BookmarksManager::remove_bookmark(&state.db, &url)
}

#[tauri::command]
async fn is_bookmarked(state: State<'_, AppState>, url: String) -> Result<bool, String> {
    BookmarksManager::is_bookmarked(&state.db, &url)
}

#[tauri::command]
async fn get_bookmarks(state: State<'_, AppState>) -> Result<Vec<BookmarkEntry>, String> {
    BookmarksManager::get_bookmarks(&state.db)
}

// --- SETTINGS IPC COMMANDS ---

#[tauri::command]
async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    SettingsManager::get_setting(&state.db, &key)
}

#[tauri::command]
async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    SettingsManager::set_setting(&state.db, &key, &value)
}

#[tauri::command]
async fn get_all_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    SettingsManager::get_all_settings(&state.db)
}

fn main() {
    let app_dir = std::env::current_dir().unwrap_or_default().join(".browser_data");
    let db_manager = DatabaseManager::new(app_dir).expect("failed to initialize SQLite database");

    let app_state = AppState {
        tab_manager: Arc::new(Mutex::new(TabManager::new())),
        db: Arc::new(db_manager),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_tabs,
            create_tab,
            switch_tab,
            close_tab,
            reopen_tab,
            navigate,
            go_back,
            go_forward,
            reload,
            open_devtools,
            get_history,
            delete_history_entry,
            clear_history,
            add_bookmark,
            remove_bookmark,
            is_bookmarked,
            get_bookmarks,
            get_setting,
            set_setting,
            get_all_settings
        ])
        .setup(|app| {
            let main_window = app.get_window("main").expect("failed to get main window");
            let app_handle_init = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state_init = app_handle_init.state::<AppState>();
                let _ = create_tab(app_handle_init.clone(), state_init, Some("https://www.google.com".into())).await;
            });

            let app_handle_events = app.handle().clone();
            let main_window_events = main_window.clone();
            main_window.on_window_event(move |event| {
                let state_ref = app_handle_events.state::<AppState>();
                let active_tab_id = {
                    let mgr = state_ref.tab_manager.lock().unwrap();
                    mgr.active_tab_id()
                };

                if let Some(tab_id) = active_tab_id {
                    if let Some(content_wv) = app_handle_events.get_webview(&tab_id) {
                        let (pos, size) = get_active_content_bounds(&main_window_events);
                        match event {
                            WindowEvent::Resized(_) => {
                                let _ = content_wv.set_position(pos);
                                let _ = content_wv.set_size(size);
                            }
                            WindowEvent::CloseRequested { .. } => {
                                let _ = content_wv.close();
                            }
                            _ => {}
                        }
                    }
                }
            });

            println!("[PHASE 4] SQLite Persistence Core & Database Manager initialized.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
