# WebExtension (Manifest v2 & Manifest v3) Architecture in Servo

## 1. Executive Summary

Modern web browser embedders and consumer shells require extension support for ad blocking, password management, accessibility tools, developer productivity, and custom UI enhancements.

Chromium provides extensive extension infrastructure via Chrome Extensions (Manifest v2 / v3) with `chrome.*` APIs. Servo introduces a foundational WebExtension engine implemented in `ports/servoshell/desktop/extensions.rs`, powered by Servo's `UserContentManager` script and style injection engine.

---

## 2. Supported Extension Model

### 2.1 Manifest v2 & Manifest v3 Support
Servo's extension loader parses unpackaged WebExtension directories containing `manifest.json`. Both Manifest v2 and Manifest v3 declarations are supported:

| Feature | Manifest v2 | Manifest v3 | Servo Status |
| :--- | :--- | :--- | :--- |
| **`manifest_version`** | `2` | `3` | Supported |
| **Content Scripts** | `content_scripts: [{ matches, js, css, run_at }]` | Same | Supported (`document_start`, `document_end`, `document_idle`) |
| **Content Styles** | CSS injection via `content_scripts.css` | Same | Supported via `UserStyleSheet` |
| **Background** | Background pages (`scripts: []`) | Background Service Workers (`service_worker`) | Foundation bridge; worker integration in progress |
| **Permissions** | `permissions: ["storage", ...]` | Granular host + API permissions | Supported |

---

## 3. Architecture & Execution Flow

```
+-------------------------------------------------------------+
|                      servoshell / Embedder                  |
|                                                             |
|   --load-extension=/path/to/unpacked_ext                    |
|                         |                                   |
|               Extension::load_from_dir()                    |
|                         |                                   |
|   1. Parse manifest.json (name, version, scripts, css)      |
|   2. Register CSS into UserContentManager                   |
|   3. Generate Extension Runtime JavaScript Bridge           |
|   4. Register Content Scripts into UserContentManager       |
+-------------------------+-----------------------------------+
                          |
                          v
+-------------------------------------------------------------+
|                     Servo Script Engine                     |
|                                                             |
|   Pipeline / Document Lifecycle:                            |
|     - DocumentStart: Inject bridge + document_start scripts |
|     - DocumentLoaded: Inject document_end scripts           |
|     - Idle: Inject document_idle scripts                    |
+-------------------------------------------------------------+
```

### 3.1 Content Script Execution Timings
Per the W3C WebExtensions specification:
1. `document_start`: Injected immediately after the DOM object is created, before parsing has begun or any scripts are run.
2. `document_end`: Injected immediately after the DOM tree is complete, but before subresources (images, stylesheets) are necessarily loaded.
3. `document_idle`: Injected when the browser has idle cycles after `document_end`.

Servo maps these timings directly to `UserScriptRunTime::DocumentStart`, `DocumentEnd`, and `DocumentIdle`.

---

## 4. Extension Runtime JavaScript Bridge

Content scripts interact with extension APIs via `chrome.*` and `browser.*` namespaces. Servo injects an asynchronous runtime bridge prior to executing extension content scripts:

```javascript
window.chrome = window.chrome || {};
window.browser = window.browser || window.chrome;

// chrome.runtime
window.chrome.runtime = {
    id: "<extension_id>",
    getManifest: function() { return <serialized_manifest>; },
    sendMessage: function(msg, callback) { ... },
    onMessage: {
        addListener: function(listener) { ... },
        removeListener: function(listener) { ... }
    }
};

// chrome.storage.local
window.chrome.storage = {
    local: {
        get: function(keys, callback) { ... },
        set: function(items, callback) { ... }
    }
};
```

---

## 5. Embedder Usage & CLI Flags

Embedders and developers can load extensions into `servoshell` using the command-line flag:
```bash
servoshell --load-extension=/path/to/my_extension https://example.com
```

Multiple extensions can be loaded simultaneously by passing multiple `--load-extension` flags or registering them via the `UserContentManager` API.

---

## 6. Future Roadmap

1. **SpiderMonkey Isolated Worlds:** Separate extension JavaScript execution into a distinct JS compartment with prototype shadowing, preventing untrusted page scripts from tampering with extension internals.
2. **Background Service Workers:** Support persistent or event-driven background service workers responding to `chrome.runtime.onMessage`.
3. **Native Messaging Host:** Implement Chromium-compatible stdio IPC for native desktop helper applications.
