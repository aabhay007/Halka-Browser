# System Architecture: Antigravity Desktop Web Browser

## 1. Architectural Overview

Antigravity Browser decouples the **Browser Chrome UI** from the **Web Content Rendering Surface**:

```
+-----------------------------------------------------------------------+
|                         DESKTOP OS WINDOW (HWND)                      |
|                                                                       |
|  +-----------------------------------------------------------------+  |
|  | BROWSER CHROME HEADER (Tab Strip + Toolbar)                     |  |
|  | Rendered in UI Shell (y: 0 to 80px)                            |  |
|  +-----------------------------------------------------------------+  |
|                                                                       |
|  +-----------------------------------------------------------------+  |
|  | NATIVE BROWSER CONTENT SURFACE (Active Tab Webview Window)        |  |
|  | Position: (x=0, y=80px) | Bounds: (width, height - 80px)       |  |
|  |                                                                 |  |
|  | [GENUINE CHROMIUM ENGINE / WebView2 / V8 ENGINE]                 |  |
|  | Google, GitHub, YouTube, Wikipedia, Auth, HTTPS, Cookies          |  |
|  |                                                                 |  |
|  +-----------------------------------------------------------------+  |
+-----------------------------------------------------------------------+
```

---

## 2. Component Modules

1. **Browser Core (`src-tauri/src/browser_core/`)**:
   - `tab_manager.rs`: Maintains tab ordering, active tab selection, and closed history stack.
   - `navigation.rs`: Distinguishes URLs from search queries using Google search fallback.
   - `history.rs`: Logs visited pages with timestamps to SQLite.
   - `bookmarks.rs`: Manages bookmark additions, removals, and star state checks.
   - `settings.rs`: Key-value configuration persistence.
   - `downloads.rs`: Manages download tracking.

2. **Database Layer (`src-tauri/src/database/`)**:
   - SQLite connection manager storing persistent data at `.browser_data/browser_data.db`.

3. **Browser Chrome UI (`ui/`)**:
   - `index.html`: Tab strip and toolbar structure.
   - `css/style.css`: Modern Catppuccin-themed chrome layout.
   - `js/chrome.js`: Event listeners, keyboard shortcuts, and IPC calls to Rust.
   - `newtab.html`: Home page interface.
