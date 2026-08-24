# Browser Engine Integration & Webview Mechanics

## 1. Engine Selection & Runtime

On Windows, the browser engine uses **Microsoft Edge WebView2**, which is powered by official, evergreen Chromium binaries updated automatically by the OS.

### Engine Capabilities Verified:
- **V8 JavaScript Engine**: Full ES2024 compliance.
- **Cookies & Local Storage**: Native session management and cookie persistence.
- **HTTP / HTTPS / TLS 1.3**: Fully enforced security certificates.
- **Media Source Extensions**: Video/Audio playback support on YouTube.
- **DevTools**: Embedded Chromium Developer Tools available via `window.open_devtools()`.

---

## 2. Dynamic Bounds & Surface Placement

Web content is mounted as a child native webview surface positioned below the Chrome header (`y = 80px`).

When the host window is resized or moved:
```rust
let (pos, size) = get_active_content_bounds(&main_window);
let _ = content_wv.set_position(pos);
let _ = content_wv.set_size(size);
```

This guarantees that web content can **never** overlap or render above the address bar or toolbar controls.

---

## 3. Multi-Tab Surface Isolation

Each open tab receives an independent native webview window handle (`tab_1`, `tab_2`, etc.). Switching tabs hides non-active webview surfaces (`webview.hide()`) and displays/focuses the target tab surface (`webview.show()`, `webview.set_focus()`).

This preserves scroll state, DOM tree, and running JS tasks across tab switches.
