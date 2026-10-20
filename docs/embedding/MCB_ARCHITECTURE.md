# Minimal Complete Browser (MCB) Architecture

Status: **Architecture Specification** · Milestone: `M3`
Companion docs: [`FULL_BROWSER_ROADMAP.md`](FULL_BROWSER_ROADMAP.md), [`PARITY_MATRIX.md`](PARITY_MATRIX.md), [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## 1. Goal & Principles

The Minimal Complete Browser (MCB) bridges the gap between a pure rendering test harness (`servoshell`) and an everyday consumer browser. It delivers a lean, functional desktop browser experience across Windows, macOS, and Linux without over-engineering or speculative UI frameworks.

### Ponytail Architecture Principles
1. **Reuse existing foundations:** Build directly upon `ports/servoshell`'s window manager (`ServoShellWindow`), multi-webview support (`WebViewCollection`), and command dispatch (`UserInterfaceCommand`).
2. **Lean chrome over bloated web tech:** Implement the browser chrome natively or with minimal UI widgets rather than bundling heavy web-based UI shells.
3. **Engine embedding first:** All browser features must talk exclusively to the public Rust `servo` crate or the `servo-capi` embedding layer. No internal subsystem backdoors.
4. **Shortest path to correctness:** SQLite / JSON flat files for bookmarks/history; standard system file pickers for downloads; standard modal dialogs for permissions.

---

## 2. High-Level Architecture

```
+-------------------------------------------------------------------------+
|                        Browser Shell (MCB)                              |
|  +-------------------------------------------------------------------+  |
|  | Tab Strip: [Tab 1: Active] [Tab 2] [+]                           |  |
|  +-------------------------------------------------------------------+  |
|  | Nav Toolbar: [<-] [->] [R] [Stop] [ Address / Omnibox ] [DL] [Set] |  |
|  +-------------------------------------------------------------------+  |
|  | Notification / Permission Bar (e.g. Geolocation prompt)           |  |
|  +-------------------------------------------------------------------+  |
|  | Active WebView Viewport (Managed via `servo::WebView`)            |  |
|  |                                                                   |  |
|  +-------------------------------------------------------------------+  |
|  | Download Shelf (Active & completed file downloads)               |  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                  MCB Core Services (Single-Process UI)                  |
|  - Tab / Session Manager (`WebViewCollection` binding)                  |
|  - Omnibox Parser & Search Provider (`location_bar_input_to_url`)       |
|  - History & Bookmarks Store (SQLite / JSON)                            |
|  - Download Coordinator (threads `servo_download_handler`)              |
|  - Profile & Cookie Partition Store (Default vs Incognito)              |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                  Embedding Surface (`servo` / `ffi/capi`)                |
|  - `Servo`, `ServoBuilder`, `WebView`, `WebViewBuilder`                 |
|  - Event loops, Rendering contexts, Delegates & Handlers               |
+-------------------------------------------------------------------------+
```

---

## 3. Subsystem Breakdown

### 3.1 Tab & Window Management
- **Existing base:** `ports/servoshell/window.rs` maintains `webview_collection: RefCell<WebViewCollection>`.
- **MCB Enhancement:**
  - Introduce tab index tracking (`active_tab_id: Cell<Option<WebViewId>>`).
  - Switching tabs toggles visibility/focus: `webview.hide()` / `webview.show()`.
  - Closing tabs properly releases the `WebView` handle and triggers cleanup.
  - New tab creates a `WebViewBuilder` with the active profile.

### 3.2 Omnibox & Navigation
- **Existing base:** `ports/servoshell/parser.rs:location_bar_input_to_url` converts typed strings into either valid `url::Url` or fallback search queries.
- **MCB Enhancement:**
  - Wire address bar text input directly to `location_bar_input_to_url`.
  - Configurable default search engine (DuckDuckGo / Google / Startpage URL template).
  - Show SSL/TLS padlock state based on navigation certificate status (reported by `CertificateErrorOverrideManager` / TLS verifier).

### 3.3 Profiles & Private Browsing (Incognito)
- **Normal Profile:**
  - On-disk storage path for cookies, Cache API, and IndexedDB (`~/.config/servo/profile` or Windows `%APPDATA%\servo`).
- **Incognito Profile:**
  - In-memory `CookieStorage` and ephemeral cache.
  - Distinct `ServoOptions` instance with disk caching disabled.
  - Guaranteed wipe on window/tab close.

### 3.4 Downloads Manager
- **C API / Engine Hook:** Wire into `servo_webview_set_download_handler`.
- **Workflow:**
  1. Engine encounters download response (Content-Disposition or unhandled MIME).
  2. Embedder receives notification with suggested filename and total size.
  3. System native file picker (`rfd` or platform dialog) confirms destination.
  4. Background stream download tracks byte count and signals completion/error.
  5. UI displays download item in shelf/popover with reveal-in-folder button.

### 3.5 Permissions UI
- **Engine Hook:** Intercept `PermissionRequest` from `ports/servoshell/window.rs`.
- **Current Behavior:** Automatically denied or ignored.
- **MCB Behavior:**
  - Render top bar prompt: `"{origin} wants to access your {permission}. [Allow] [Block]"`.
  - Persist user decision per-origin in local permissions database.

### 3.6 Bookmarks & History Store
- Simple, zero-overhead storage:
  - SQLite database `places.sqlite` or JSON store `history.json`.
  - Recorded on successful document load (`NavigationState::Complete`).
  - History entry: `(url, title, timestamp, visit_count)`.
  - Bookmark entry: `(url, title, created_at, tags)`.

---

## 4. Implementation Phasing

1. **Phase 1 (Tab & Navigation Wireup):**
   - Expose tab switching and navigation buttons in `servoshell` window chrome.
   - Wire `location_bar_input_to_url` to active tab navigation.
2. **Phase 2 (Downloads & Permissions):**
   - Implement download callback handler with progress tracking.
   - Add prompt modal/banner for geolocation, clipboard, and notifications.
3. **Phase 3 (Persistence & Profiles):**
   - Add SQLite history/bookmarks store.
   - Add incognito window mode (`--private`).
4. **Phase 4 (Packaging & Ship):**
   - Package executable with default bookmarks and search settings.
