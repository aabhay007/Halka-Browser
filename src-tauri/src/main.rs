// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod browser_core;
mod database;

use ai::{
    create_ai_provider, AICssRule, AIExportRequest, AIPreviewRequest, AIPreviewResponse,
    AISettings, CssValidator,
};
use browser_core::bookmarks::{BookmarkEntry, BookmarksManager};
use browser_core::history::{HistoryEntry, HistoryManager};
use browser_core::navigation::NavigationManager;
use browser_core::settings::SettingsManager;
use browser_core::tab_manager::{TabData, TabManager};
use database::DatabaseManager;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State,
    WebviewUrl, WindowEvent,
};

const TOOLBAR_HEIGHT: f64 = 80.0;
const SIDEBAR_WIDTH: f64 = 340.0;

pub struct AppState {
    pub tab_manager: Arc<Mutex<TabManager>>,
    pub db: Arc<DatabaseManager>,
    pub sidebar_open: Arc<Mutex<bool>>,
    pub preview_history: Arc<Mutex<HashMap<String, Vec<Vec<AICssRule>>>>>,
}

fn get_active_content_bounds(
    window: &tauri::window::Window,
    sidebar_open: bool,
) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let inner_size = window
        .inner_size()
        .unwrap_or(tauri::PhysicalSize::new(1280, 800))
        .to_logical::<f64>(scale);

    let sidebar_offset = if sidebar_open { SIDEBAR_WIDTH } else { 0.0 };
    let width = (inner_size.width - sidebar_offset).max(200.0);
    let height = (inner_size.height - TOOLBAR_HEIGHT).max(100.0);

    let pos = LogicalPosition::new(0.0, TOOLBAR_HEIGHT);
    let size = LogicalSize::new(width, height);
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
async fn create_tab(
    app: AppHandle,
    state: State<'_, AppState>,
    url: Option<String>,
) -> Result<TabData, String> {
    let target_url = url.unwrap_or_else(|| "https://www.google.com".to_string());
    let parsed_url = NavigationManager::parse_input_to_url(&target_url);
    let initial_title = NavigationManager::extract_display_title(&parsed_url);

    let sidebar_open = {
        let is_open = state.sidebar_open.lock().unwrap();
        *is_open
    };

    let main_window = app.get_window("main").ok_or("main window not found")?;
    let (pos, size) = get_active_content_bounds(&main_window, sidebar_open);

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

    let app_handle_title = app.clone();
    let app_handle_load = app.clone();
    let tab_id_load = tab_id.clone();
    let initial_url = WebviewUrl::External(parsed_url.parse().map_err(|e| format!("{}", e))?);
    let webview_builder = WebviewBuilder::new(&tab_id, initial_url)
        .auto_resize()
        .on_document_title_changed(move |_wv, title| {
            if title.starts_with("__HALKA_AI_DATA__:") {
                let payload = title["__HALKA_AI_DATA__:".len()..].to_string();
                let _ = app_handle_title.emit("ai_element_selected", payload);
            }
        })
        .on_page_load(move |_wv, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let current_url = payload.url().as_str().to_string();
                let display_title = NavigationManager::extract_display_title(&current_url);
                let state_ref = app_handle_load.state::<AppState>();
                {
                    let mut mgr = state_ref.tab_manager.lock().unwrap();
                    mgr.update_tab(
                        &tab_id_load,
                        Some(display_title.clone()),
                        Some(current_url.clone()),
                    );
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
async fn switch_tab(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    let sidebar_open = {
        let is_open = state.sidebar_open.lock().unwrap();
        *is_open
    };

    let main_window = app.get_window("main").ok_or("main window not found")?;
    let (pos, size) = get_active_content_bounds(&main_window, sidebar_open);

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
async fn close_tab(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    if let Some(wv) = app.get_webview(&tab_id) {
        let _ = wv.close();
    }

    // Clean up preview history for closed tab
    {
        let mut history_map = state.preview_history.lock().unwrap();
        history_map.remove(&tab_id);
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
                    mgr.update_tab(
                        &tab_id,
                        Some(display_title.clone()),
                        Some(target_url.clone()),
                    );
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
async fn get_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<HistoryEntry>, String> {
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
async fn add_bookmark(
    state: State<'_, AppState>,
    url: String,
    title: String,
) -> Result<BookmarkEntry, String> {
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
async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    SettingsManager::set_setting(&state.db, &key, &value)
}

#[tauri::command]
async fn get_all_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    SettingsManager::get_all_settings(&state.db)
}

// --- AI PREVIEW MODE IPC COMMANDS ---

#[tauri::command]
async fn toggle_ai_sidebar(
    app: AppHandle,
    state: State<'_, AppState>,
    open: Option<bool>,
) -> Result<bool, String> {
    let new_state = {
        let mut is_open = state.sidebar_open.lock().unwrap();
        let target = open.unwrap_or(!*is_open);
        *is_open = target;
        target
    };

    let main_window = app.get_window("main").ok_or("main window not found")?;
    let (pos, size) = get_active_content_bounds(&main_window, new_state);

    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    };

    if let Some(tab_id) = active_id {
        if let Some(wv) = app.get_webview(&tab_id) {
            let _ = wv.set_position(pos);
            let _ = wv.set_size(size);
        }
    }

    let _ = app.emit("ai_sidebar_toggled", new_state);
    Ok(new_state)
}

#[tauri::command]
async fn start_element_picker(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    }.ok_or("No active tab found")?;

    if let Some(content_wv) = app.get_webview(&active_id) {
        let picker_js = r#"
        (function() {
          if (window.__halka_ai_picker_active) return;
          window.__halka_ai_picker_active = true;

          const existingHover = document.getElementById('__halka_ai_hover_box__');
          if (existingHover) existingHover.remove();
          const existingSelected = document.getElementById('__halka_ai_selected_box__');
          if (existingSelected) existingSelected.remove();

          const hoverBox = document.createElement('div');
          hoverBox.id = '__halka_ai_hover_box__';
          hoverBox.style.cssText = 'position: absolute; pointer-events: none !important; border: 2px solid #89b4fa; background: rgba(137, 180, 250, 0.18); z-index: 2147483647; transition: all 0.05s ease-out; box-sizing: border-box; border-radius: 3px; display: none;';
          
          const hoverBadge = document.createElement('div');
          hoverBadge.id = '__halka_ai_hover_badge__';
          hoverBadge.style.cssText = 'position: absolute; top: -22px; left: 0; background: #89b4fa; color: #11111b; font-family: monospace; font-size: 11px; font-weight: bold; padding: 2px 6px; border-radius: 3px; white-space: nowrap; pointer-events: none !important; box-shadow: 0 2px 4px rgba(0,0,0,0.3);';
          hoverBox.appendChild(hoverBadge);
          (document.body || document.documentElement).appendChild(hoverBox);

          let selectedBox = document.getElementById('__halka_ai_selected_box__');
          if (!selectedBox) {
            selectedBox = document.createElement('div');
            selectedBox.id = '__halka_ai_selected_box__';
            selectedBox.style.cssText = 'position: absolute; pointer-events: none !important; border: 2px solid #a6e3a1; background: rgba(166, 227, 161, 0.15); z-index: 2147483646; box-sizing: border-box; border-radius: 3px; display: none;';
            
            const selectedBadge = document.createElement('div');
            selectedBadge.id = '__halka_ai_selected_badge__';
            selectedBadge.style.cssText = 'position: absolute; top: -22px; right: 0; background: #a6e3a1; color: #11111b; font-family: monospace; font-size: 11px; font-weight: bold; padding: 2px 6px; border-radius: 3px; white-space: nowrap; pointer-events: none !important;';
            selectedBadge.textContent = 'SELECTED';
            selectedBox.appendChild(selectedBadge);
            (document.body || document.documentElement).appendChild(selectedBox);
          }

          function getUniqueSelector(el) {
            if (!el || el.nodeType !== Node.ELEMENT_NODE) return '';
            if (el.id && document.querySelectorAll('#' + CSS.escape(el.id)).length === 1) {
              return '#' + CSS.escape(el.id);
            }
            const tag = el.tagName.toLowerCase();
            if (tag === 'body' || tag === 'html') return tag;
            
            let path = [];
            let current = el;
            while (current && current.nodeType === Node.ELEMENT_NODE && current !== document.body && current !== document.documentElement) {
              if (current.id && document.querySelectorAll('#' + CSS.escape(current.id)).length === 1) {
                path.unshift('#' + CSS.escape(current.id));
                break;
              }
              let selector = current.tagName.toLowerCase();
              if (current.classList.length > 0) {
                const classes = Array.from(current.classList).filter(c => !c.startsWith('__halka')).slice(0, 2);
                if (classes.length > 0) {
                  selector += '.' + classes.map(c => CSS.escape(c)).join('.');
                }
              }
              
              const parent = current.parentElement;
              if (parent) {
                const siblings = Array.from(parent.children).filter(c => c.tagName === current.tagName);
                if (siblings.length > 1) {
                  const index = siblings.indexOf(current) + 1;
                  selector += ':nth-of-type(' + index + ')';
                }
              }
              path.unshift(selector);
              if (path.length >= 3) break;
              current = current.parentElement;
            }
            return path.join(' > ');
          }

          function getLayoutStyles(node) {
            if (!node || node === document.body || node === document.documentElement) return null;
            const cs = window.getComputedStyle(node);
            const layoutProps = ['display', 'flex-direction', 'align-items', 'justify-content', 'grid-template-columns', 'gap', 'width', 'padding'];
            const ls = {};
            layoutProps.forEach(p => {
              const v = cs.getPropertyValue(p);
              if (v) ls[p] = v;
            });
            return {
              tag: node.tagName.toLowerCase(),
              id: node.id || null,
              classes: Array.from(node.classList).filter(c => !c.startsWith('__halka')),
              layout_styles: ls
            };
          }

          function handleMouseMove(e) {
            const target = document.elementFromPoint(e.clientX, e.clientY);
            if (!target || target.id?.startsWith('__halka_ai_') || target === document.body || target === document.documentElement) {
              hoverBox.style.display = 'none';
              return;
            }
            const rect = target.getBoundingClientRect();
            hoverBox.style.display = 'block';
            hoverBox.style.top = (rect.top + window.scrollY) + 'px';
            hoverBox.style.left = (rect.left + window.scrollX) + 'px';
            hoverBox.style.width = rect.width + 'px';
            hoverBox.style.height = rect.height + 'px';

            const tag = target.tagName.toLowerCase();
            const id = target.id ? '#' + target.id : '';
            const classes = Array.from(target.classList).filter(c => !c.startsWith('__halka')).slice(0, 2).map(c => '.' + c).join('');
            hoverBadge.textContent = tag.toUpperCase() + id + classes;
          }

          function handleClick(e) {
            e.preventDefault();
            e.stopPropagation();

            const target = document.elementFromPoint(e.clientX, e.clientY);
            if (!target || target.id?.startsWith('__halka_ai_') || target === document.body || target === document.documentElement) {
              return;
            }

            const rect = target.getBoundingClientRect();
            selectedBox.style.display = 'block';
            selectedBox.style.top = (rect.top + window.scrollY) + 'px';
            selectedBox.style.left = (rect.left + window.scrollX) + 'px';
            selectedBox.style.width = rect.width + 'px';
            selectedBox.style.height = rect.height + 'px';

            const styleProps = [
              'display', 'position', 'width', 'height', 'margin', 'padding',
              'font-family', 'font-size', 'font-weight', 'line-height', 'color',
              'background-color', 'border', 'border-radius', 'box-shadow', 'opacity',
              'flex-direction', 'align-items', 'justify-content', 'gap', 'transform', 'z-index'
            ];
            const cs = window.getComputedStyle(target);
            const computedStyles = {};
            styleProps.forEach(p => {
              const v = cs.getPropertyValue(p);
              if (v) computedStyles[p] = v;
            });

            const attrs = {};
            ['type', 'name', 'role', 'href', 'placeholder', 'aria-label', 'title', 'src', 'alt', 'value'].forEach(a => {
              if (target.hasAttribute(a)) {
                attrs[a] = target.getAttribute(a);
              }
            });

            const selector = getUniqueSelector(target);
            const textContent = target.textContent ? target.textContent.trim().slice(0, 150) : null;

            const data = {
              tag: target.tagName.toLowerCase(),
              id: target.id || null,
              classes: Array.from(target.classList).filter(c => !c.startsWith('__halka')),
              attributes: attrs,
              text: textContent,
              selector: selector,
              computed_styles: computedStyles,
              parent_context: getLayoutStyles(target.parentElement),
              grandparent_context: getLayoutStyles(target.parentElement?.parentElement)
            };

            cleanupHover();

            const origTitle = document.title;
            document.title = '__HALKA_AI_DATA__:' + JSON.stringify(data);
            setTimeout(() => { document.title = origTitle; }, 50);
          }

          function handleKeyDown(e) {
            if (e.key === 'Escape') {
              cleanup();
            }
          }

          function cleanupHover() {
            window.removeEventListener('mousemove', handleMouseMove, true);
            window.removeEventListener('click', handleClick, true);
            if (hoverBox) hoverBox.style.display = 'none';
            window.__halka_ai_picker_active = false;
          }

          function cleanup() {
            cleanupHover();
            window.removeEventListener('keydown', handleKeyDown, true);
            if (hoverBox) hoverBox.remove();
            if (selectedBox) selectedBox.remove();
          }

          window.__halka_ai_cancel_picker = cleanup;

          window.addEventListener('mousemove', handleMouseMove, true);
          window.addEventListener('click', handleClick, true);
          window.addEventListener('keydown', handleKeyDown, true);
        })();
        "#;
        let _ = content_wv.eval(picker_js);
    }
    Ok(())
}

#[tauri::command]
async fn cancel_element_picker(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    }.ok_or("No active tab found")?;

    if let Some(content_wv) = app.get_webview(&active_id) {
        let cleanup_js = r#"
            if (window.__halka_ai_cancel_picker) {
                window.__halka_ai_cancel_picker();
            }
        "#;
        let _ = content_wv.eval(cleanup_js);
    }
    Ok(())
}

#[tauri::command]
async fn get_ai_settings(state: State<'_, AppState>) -> Result<AISettings, String> {
    let provider = SettingsManager::get_setting(&state.db, "ai_provider")?
        .unwrap_or_else(|| "groq".to_string());

    let api_key = SettingsManager::get_setting(&state.db, "ai_api_key")?
        .filter(|k| !k.trim().is_empty())
        .or_else(|| std::env::var("GROQ_API_KEY").ok())
        .unwrap_or_default();

    let model = SettingsManager::get_setting(&state.db, "ai_model")?
        .unwrap_or_else(|| "openai/gpt-oss-120b".to_string());

    Ok(AISettings {
        provider,
        api_key,
        model,
    })
}

#[tauri::command]
async fn save_ai_settings(
    state: State<'_, AppState>,
    settings: AISettings,
) -> Result<(), String> {
    SettingsManager::set_setting(&state.db, "ai_provider", &settings.provider)?;
    SettingsManager::set_setting(&state.db, "ai_api_key", &settings.api_key)?;
    SettingsManager::set_setting(&state.db, "ai_model", &settings.model)?;
    Ok(())
}

#[tauri::command]
async fn ai_generate_preview(
    state: State<'_, AppState>,
    req: AIPreviewRequest,
) -> Result<AIPreviewResponse, String> {
    let settings = get_ai_settings(state).await?;
    let provider = create_ai_provider(&settings);
    provider.generate_preview(&req).await
}

#[tauri::command]
async fn apply_preview_css(
    app: AppHandle,
    state: State<'_, AppState>,
    rules: Vec<AICssRule>,
) -> Result<String, String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    }.ok_or("No active tab found")?;

    let css_stylesheet = CssValidator::compile_to_stylesheet(&rules);

    // Push into preview history for active tab
    {
        let mut history_map = state.preview_history.lock().unwrap();
        let history = history_map.entry(active_id.clone()).or_insert_with(Vec::new);
        history.push(rules);
    }

    if let Some(content_wv) = app.get_webview(&active_id) {
        let script = format!(
            r#"(function(css) {{
                let styleEl = document.getElementById('__halka_ai_preview_style__');
                if (!styleEl) {{
                    styleEl = document.createElement('style');
                    styleEl.id = '__halka_ai_preview_style__';
                    (document.head || document.documentElement).appendChild(styleEl);
                }}
                styleEl.textContent = css;
            }})({});"#,
            serde_json::to_string(&css_stylesheet).unwrap_or_else(|_| "\"\"".to_string())
        );
        let _ = content_wv.eval(&script);
    }

    Ok(css_stylesheet)
}

#[tauri::command]
async fn undo_preview_css(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<Vec<AICssRule>>, String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    }.ok_or("No active tab found")?;

    let previous_rules = {
        let mut history_map = state.preview_history.lock().unwrap();
        if let Some(history) = history_map.get_mut(&active_id) {
            history.pop();
            history.last().cloned()
        } else {
            None
        }
    };

    if let Some(content_wv) = app.get_webview(&active_id) {
        if let Some(ref rules) = previous_rules {
            let css_stylesheet = CssValidator::compile_to_stylesheet(rules);
            let script = format!(
                r#"(function(css) {{
                    let styleEl = document.getElementById('__halka_ai_preview_style__');
                    if (!styleEl) {{
                        styleEl = document.createElement('style');
                        styleEl.id = '__halka_ai_preview_style__';
                        (document.head || document.documentElement).appendChild(styleEl);
                    }}
                    styleEl.textContent = css;
                }})({});"#,
                serde_json::to_string(&css_stylesheet).unwrap_or_else(|_| "\"\"".to_string())
            );
            let _ = content_wv.eval(&script);
        } else {
            let script = r#"
                let styleEl = document.getElementById('__halka_ai_preview_style__');
                if (styleEl) styleEl.remove();
            "#;
            let _ = content_wv.eval(script);
        }
    }

    Ok(previous_rules)
}

#[tauri::command]
async fn reset_preview_css(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let active_id = {
        let mgr = state.tab_manager.lock().unwrap();
        mgr.active_tab_id()
    }.ok_or("No active tab found")?;

    // Clear history stack for active tab
    {
        let mut history_map = state.preview_history.lock().unwrap();
        history_map.remove(&active_id);
    }

    if let Some(content_wv) = app.get_webview(&active_id) {
        let script = r#"
            (function() {
                const styleEl = document.getElementById('__halka_ai_preview_style__');
                if (styleEl) styleEl.remove();
                const selectedBox = document.getElementById('__halka_ai_selected_box__');
                if (selectedBox) selectedBox.remove();
                const hoverBox = document.getElementById('__halka_ai_hover_box__');
                if (hoverBox) hoverBox.remove();
                if (window.__halka_ai_cancel_picker) window.__halka_ai_cancel_picker();
            })();
        "#;
        let _ = content_wv.eval(script);
    }

    Ok(())
}

#[tauri::command]
async fn export_preview_prompt(
    state: State<'_, AppState>,
    req: AIExportRequest,
) -> Result<String, String> {
    let settings = get_ai_settings(state).await?;
    let provider = create_ai_provider(&settings);
    provider.export_prompt(&req).await
}

fn main() {
    let app_dir = std::env::current_dir().unwrap_or_default().join(".browser_data");
    let db_manager = DatabaseManager::new(app_dir).expect("failed to initialize SQLite database");

    let app_state = AppState {
        tab_manager: Arc::new(Mutex::new(TabManager::new())),
        db: Arc::new(db_manager),
        sidebar_open: Arc::new(Mutex::new(false)),
        preview_history: Arc::new(Mutex::new(HashMap::new())),
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
            get_all_settings,
            toggle_ai_sidebar,
            start_element_picker,
            cancel_element_picker,
            get_ai_settings,
            save_ai_settings,
            ai_generate_preview,
            apply_preview_css,
            undo_preview_css,
            reset_preview_css,
            export_preview_prompt
        ])
        .setup(|app| {
            let main_window = app.get_window("main").expect("failed to get main window");
            let app_handle_init = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state_init = app_handle_init.state::<AppState>();
                let _ = create_tab(
                    app_handle_init.clone(),
                    state_init,
                    Some("https://www.google.com".into()),
                )
                .await;
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
                        let sidebar_open = {
                            let is_open = state_ref.sidebar_open.lock().unwrap();
                            *is_open
                        };
                        let (pos, size) =
                            get_active_content_bounds(&main_window_events, sidebar_open);
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

            println!("[PHASE 5] AI Preview Mode Core Initialized.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
