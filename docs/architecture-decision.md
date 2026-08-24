# Architecture Decision Document: Desktop Chromium Browser Engine & Shell

**Author:** Lead Software Architect & Senior Desktop Browser Engineer  
**Date:** August 24, 2026  
**Status:** Approved for Phase 1 Prototype & Engine Proof  

---

## 1. Executive Summary & Core Mandate

The primary goal of this project is to build a modern, cross-platform desktop web browser shell capable of loading complex modern web applications (Google, GitHub, YouTube, Wikipedia, login/auth flows, modern JS/CSS, cookies, local storage, HTTPS, DevTools) **without using iframes**. 

The architecture strictly separates the **Browser Shell / Chrome** (address bar, tab controls, navigation buttons, status/menus) from the **Browser Engine / Web Content Surface**. Web content must be rendered directly by a native browser engine surface managed as a native child window/overlay by the desktop core, ensuring security, site isolation, and exact bounds composition.

This document evaluates candidate Chromium embedding technologies, establishes the core architectural strategy, and defines the roadmap for engine proof (Phase 1) through full shell implementation.

---

## 2. Evaluation of Chromium Embedding Frameworks

We evaluated five candidate technical architectures against 10 critical browser requirements: Real Chromium rendering, modern web compatibility, proper cookies/storage/auth, multi-tab native surfaces, navigation control, native webview position/resize control, DevTools support, cross-platform capabilities, maintainability, and security.

### Option A: Tauri 2 + Native Multi-Webview (WRY / WebView2 Engine) [RECOMMENDED]

* **Chromium Integration Approach:**
  * **Windows:** Uses Microsoft Edge `WebView2` (Evergreen runtime, powered directly by official Chromium engine).
  * **macOS:** Uses system `WKWebView` (WebKit core).
  * **Linux:** Uses `WebKitGTK` or embedded Chromium renderer backend.
* **Architecture:**
  * Rust core (`browser-core`) manages browser state, tab lifecycle, window positioning, history, bookmarks, and SQLite persistence.
  * The main window renders the Browser Chrome UI.
  * Each active browser tab creates a **native child `Webview` surface** attached directly to the main window's native HWND/NSView/GtkWidget handle using Tauri 2's native multi-webview API.
  * **NO IFRAMES are used.** Content surfaces are native OS windows positioned dynamically to fill the content region below the chrome toolbar.
* **Advantages:**
  * 100% genuine Chromium engine on Windows (Microsoft Edge WebView2), updated automatically with security patches.
  * Zero binary bloat: binary size is ~10–15 MB instead of 150+ MB.
  * First-class Rust native integration with zero C++ wrapper friction.
  * Built-in multi-webview layout and coordinate positioning API (`WebviewBuilder::new`).
  * Built-in DevTools window support via `OpenDevToolsWindow()` / `open_devtools()`.
  * Excellent memory isolation and native process separation handled by the OS webview runtime.
* **Disadvantages:**
  * Web engine underlying surface on macOS/Linux uses system native webviews (WebKit) unless paired with a CEF binary wrapper on those platforms.
* **Verdict:** **SELECTED for Phase 1 & V1 Shell.**

---

### Option B: CEF (Chromium Embedded Framework) via Rust Bindings (`cef-rs` / `cef`)

* **Chromium Integration Approach:** Direct C++ `libcef` binary link across Windows, macOS, and Linux.
* **Advantages:**
  * Identical Chromium rendering codebase across all target platforms (Windows, macOS, Linux).
  * Complete low-level access to Chromium internals, process creation, network handlers, and CDP protocol.
* **Disadvantages:**
  * Massive binary footprint (~150MB to 250MB pre-compiled binary bundling requirement per OS).
  * Rust bindings (`cef-rs`) are complex, poorly maintained across modern Chromium releases, and require extensive C++ toolchain interop.
  * Toolchain incompatibilities (linking C++ MSVC/MinGW GCC libraries on Windows requires strict runtime matching).
  * Extremely high maintenance overhead for scaffolding and cross-compilation.
* **Verdict:** **REJECTED as primary V1 engine due to build tooling fragility and unsafe binding complexity.**

---

### Option C: Qt 6 WebEngine (`QWebEngineView`) + Rust (qmetaobject / CXX)

* **Chromium Integration Approach:** Qt wraps Chromium core into C++ widgets.
* **Advantages:**
  * Full native Chromium on Windows, macOS, and Linux.
  * Built-in native multi-tab widget architecture (`QTabWidget` + `QWebEngineView`).
* **Disadvantages:**
  * Requires heavy C++ / Qt6 SDK installations on build machines.
  * Rust FFI binding layer (`qmetaobject`, `cxx`, `qt_widgets`) introduces build friction and complex memory management across the Rust/C++ boundary.
  * Licensing constraints (LGPLv3/GPLv3).
* **Verdict:** **REJECTED due to heavy C++ toolchain dependency and complex Rust interop.**

---

### Option D: Direct Native C++ Chromium Embedding

* **Chromium Integration Approach:** Compiling and linking full Chromium content API from source.
* **Advantages:** Unfiltered access to Chromium architecture.
* **Disadvantages:** Requires 100+ GB build environment, multi-hour compile times, and dedicated build farm infrastructure. Unviable for standard desktop software architecture.
* **Verdict:** **REJECTED.**

---

### Option E: Electron Framework

* **Verdict:** **ABSOLUTELY REJECTED per prompt instruction 4.** (Disallowed under all circumstances).

---

## 3. Comparative Summary Matrix

| Criterion | Tauri 2 Multi-Webview (Selected) | CEF (Chromium Embedded) | Qt WebEngine |
| :--- | :--- | :--- | :--- |
| **Windows Engine** | Native Chromium (WebView2) | Native Chromium (libcef) | Native Chromium (QtWebEngine) |
| **No-Iframe Multi-Tab** | Native Child Webview Surfaces | Native CefBrowser Windows | Native QWebEngineView Widgets |
| **Rust Integration** | Native First-Class (Tauri/WRY) | Unstable / C++ FFI required | Complex FFI / CXX needed |
| **Binary Footprint** | Extremely Small (~10-15MB) | Heavy (~150MB+) | Heavy (~100MB+) |
| **DevTools Support** | Built-in Native / CDP | Native Chromium DevTools | Native DevTools Inspector |
| **Build Machine Requirements** | Standard Rust + OS Webview | C++ Clang/MSVC + LibCEF SDK | Qt6 SDK + C++ Compiler |

---

## 4. Multi-Tab Architecture & Native Surface Composition

To satisfy the absolute requirement that **external websites are never loaded inside iframes**, the application employs a **Native Multi-Webview Overlay Architecture**:

```
+-------------------------------------------------------------------+
|                        MAIN WINDOW (HWND)                         |
|                                                                   |
|  +-------------------------------------------------------------+  |
|  | BROWSER CHROME (Toolbar UI: Address Bar, Tabs, Buttons)    |  |
|  | Height: 80px (Native UI Region)                             |  |
|  +-------------------------------------------------------------+  |
|                                                                   |
|  +-------------------------------------------------------------+  |
|  | NATIVE BROWSER SURFACE (Tab 1 / Tab 2 / Tab 3)              |  |
|  | Bounds: x=0, y=80, width=Window.width, height=Window.height-80 |  |
|  |                                                             |  |
|  |  [REAL CHROMIUM WEB ENGINE / NATIVE WEBVIEW CORE]           |  |
|  |  Renders: https://google.com, https://github.com, etc.      |  |
|  |                                                             |  |
|  +-------------------------------------------------------------+  |
+-------------------------------------------------------------------+
```

### Key Mechanical Principles:

1. **Top Bar (Browser Chrome UI):**
   * The Browser Chrome (tabs, back/forward/reload, address bar, menu) is rendered in a clean UI window or dedicated UI region.
2. **Web Content Tabs (Native Surfaces):**
   * Each tab corresponds to an independent native `Webview` surface instantiated by Rust.
   * Tab creation creates a `Webview` with explicit initial bounds `x: 0, y: 80, width: W, height: H - 80`.
   * Switching tabs hides non-active child surfaces (`webview.hide()`) and displays/focuses the active surface (`webview.show()`, `webview.focus()`).
   * Closing a tab destroys the native webview surface handle (`webview.close()`).
3. **Resizing & Positioning:**
   * When the main window resizes, Rust intercepts the resize event and dynamically updates the native bounds of the active child `Webview`.
   * Web content is strictly bounded; it **cannot** physically render over the address bar or toolbar controls.

---

## 5. Architectural Modules & Structure

```
d:/web2/
├── docs/
│   └── architecture-decision.md
├── src-tauri/               # Rust Core Application & Engine Adapter
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Application Entry point
│   │   ├── browser_core/    # State management abstractions
│   │   │   ├── tab_manager.rs
│   │   │   ├── navigation.rs
│   │   │   ├── history.rs   # SQLite persistence
│   │   │   ├── bookmarks.rs # SQLite persistence
│   │   │   ├── downloads.rs
│   │   │   └── settings.rs
│   │   ├── browser_engine/  # Engine Abstraction Trait & Implementations
│   │   │   └── webview_adapter.rs
│   │   └── database/        # SQLite schema & helpers
│   │       └── mod.rs
└── ui/                      # Browser Chrome UI Frontend
    ├── index.html
    ├── css/
    │   └── style.css
    └── js/
        └── chrome.js
```

---

## 6. Phase Roadmap

* **Phase 0:** Architecture Research & Environment Inspection (**COMPLETED**).
* **Phase 1:** Native Engine Proof. Build minimal native application opening Google, GitHub, YouTube with navigation, cookies, redirects, HTTPS, back/forward, and reload.
* **Phase 2:** Browser Window & Chrome layout with strict native webview bounds composition (No overlay on toolbar).
* **Phase 3:** Native Multi-Tab management (Create, Close, Switch, Reopen tabs).
* **Phase 4:** Persistence (SQLite History, Bookmarks, Settings).
* **Phase 5:** Core Browser Features (Downloads, Find-in-page, Zoom, Context Menu, New Tab page).
* **Phase 6:** DevTools Integration.
* **Phase 7:** Cross-platform verification and documentation.

---

## 7. Conclusion & Recommendation

We recommend **Tauri 2 with Native Multi-Webview Child Surfaces (WRY / Chromium WebView2)** as the primary technology stack. It provides authentic native Chromium rendering on Windows, native webview surfaces without iframes, pure Rust core control, minimal binary size, and zero unsafe C++ FFI build fragility.
