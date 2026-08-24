# Cross-Platform Support Matrix & Limitations

## Platform Target Overview

| Target OS | Primary Webview Engine | Security Model | Notes |
| :--- | :--- | :--- | :--- |
| **Windows 10/11** | Microsoft Edge WebView2 (Chromium) | Evergreen OS Sandbox | Full native Chromium engine |
| **macOS** | `WKWebView` (WebKit) / CEF Adapter | macOS App Sandbox | Full native webview surface |
| **Linux** | `WebKitGTK` / CEF Renderer | Linux WebKit Sandbox | Full native webview surface |

---

## Known Platform Limitations & Behavior

1. **Windows**: Fully supported via Microsoft Edge Chromium WebView2.
2. **DevTools Protocol**: Native DevTools accessible via `Ctrl+Shift+I` or `open_devtools` API call across debug builds.
3. **Frameless Child Windows**: Position and size bounds synchronized cleanly on window `Moved` and `Resized` events.
