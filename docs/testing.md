# Testing Suite & Verification Guide

## 1. Automated Unit Tests

Pure browser-core logic (navigation URL parsing, tab manager order, history, bookmarks) is tested using Rust's built-in test framework:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 2. Browser Engine Validation Matrix

Every major build must satisfy the core website compatibility matrix:

1. **Google (`https://www.google.com`)**: Renders search inputs, performs searches, handles auth cookies.
2. **GitHub (`https://github.com`)**: Renders Flex/Grid layouts, shadow DOM, WebSockets, login forms.
3. **YouTube (`https://www.youtube.com`)**: Plays HTML5 video, renders layout, handles media streams.
4. **Wikipedia (`https://www.wikipedia.org`)**: Form submission, redirect handling, typography rendering.

---

## 3. Strict Architectural Verification

- **No Iframe Check**: Verify that `0` `<iframe>` elements exist in `ui/index.html`.
- **Bounds Check**: Ensure web content surface starts at `y = 80px` below the header.
