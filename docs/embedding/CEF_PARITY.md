# CEF Parity — CEF concepts mapped to Servo's embedding API

Status: **Gap analysis** · Branch: `embedding/cef-parity` · Baseline commit: `3dcc4e02bc1`

This document maps Chromium Embedded Framework (CEF) concepts onto Servo's
current embedding surface and identifies the work required to reach a
CEF-like embedding experience. It complements [`AUDIT.md`](AUDIT.md) (inventory
and readiness) and [`ROADMAP.md`](ROADMAP.md) (sequencing).

## How to read this document

- **Servo Rust equivalent** names the closest API in the `servo` crate
  (`components/servo`), or `none` when Servo has no analogue.
- **Servo C API status** is about the C ABI (`servo-capi`, `ffi/capi`) only:
  - **Exposed** — reachable from C today.
  - **Partial** — partly reachable from C, or present in Rust but with little
    or no C coverage (see Notes).
  - **Missing** — no C API equivalent.

> Scope note: this analysis reflects the branch at the time of writing,
> including the C API additions that accompanied it (`ffi/capi` navigation and
> getters, zoom/resize/focus/show–hide, preference setters, JS evaluation, and
> the new `notify_url_changed` / `notify_page_title_changed` delegate callbacks).
> Rows marked "(done on this branch)" were gaps in the committed baseline
> `3dcc4e02bc1` and are implemented today. Where a capability is uncertain it is
> marked "needs verification" rather than assumed.

---

## 1. App / lifecycle

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Application object | `CefApp`, `CefApp::OnBeforeCommandLineProcessing`, `OnRegisterCustomSchemes` | `ServoDelegate` (`servo_delegate.rs`) + `WebViewDelegate` (`webview_delegate.rs`); custom schemes via `ServoBuilder::protocol_registry` | Missing | No app-level lifecycle object; responsibilities are split across two trait objects. Nothing is exposed to C. |
| Main args | `CefMainArgs` | none (server reads argv via `servo_config::opts`; `servoshell` parses CLI itself) | Missing | `servo-capi` is a library; no process entry/main-args bootstrap. |
| Settings | `CefSettings`, `CefBrowserSettings` | `ServoBuilder::{opts, preferences}`; `servo_config::opts::Opts`; `servo_config::prefs::Preferences` | Partial | Only a subset is settable via `ServoOptions` (see `options.rs`) and `ServoPreferences`. Many settings absent. |
| Init / shutdown | `CefInitialize`, `CefShutdown`, `CefShutdownCheck` | `ServoBuilder::build() -> Servo`; drop / `servo_free` | Partial | No global process init/shutdown or singleton state; lifetime is per-`Servo` handle. |
| Message loop | `CefSettings.multi_threaded_message_loop`, `external_message_pump`, `CefDoMessageLoopWork` | `EventLoopWaker` + `Servo::spin_event_loop()` (`servo.rs`) | Partial | **External-pump model only.** No multi-threaded message loop; embedder must call `spin_event_loop`. C exposes `servo_spin_event_loop` + `servo_builder_set_event_loop_waker`. |

---

## 2. Browser

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Browser / host | `CefBrowser`, `CefBrowserHost` | `WebView` (`webview.rs`) + `WebViewBuilder` | Partial | `WebView` is the analogue; there is no browser/host split or separate process object. |
| Create browser | `CefBrowserHost::CreateBrowser` | `WebViewBuilder::new(&Servo, Rc<dyn RenderingContext>)` … `build()` | Partial | C: `servo_webview_builder_create` / `_set_url` / `_set_delegate` / `_build`. Only software-rendered views are constructible (see §4). |
| Close browser | `CefBrowserHost::CloseBrowser` | `WebViewDelegate::notify_closed`; `servo_webview_close` | **Exposed** | *(done on this branch)* C: `servo_webview_close` + `ServoWebViewDelegate::notify_closed`. Embedder calls `servo_webview_close`, then waits for `notify_closed` before `servo_webview_free`. |
| Load URL | `CefBrowserHost::LoadURL` | `WebView::load(Url)`, `WebView::load_request(UrlRequest)` | **Exposed** | *(done on this branch)* C: `servo_webview_load` and `servo_webview_load_request` (supporting custom headers). |
| Back / forward | `CefBrowserHost::GoBack`, `GoForward`, `CanGoBack`, `CanGoForward` | `WebView::go_back(usize)`, `go_forward(usize)`, `can_go_back()`, `can_go_forward()`; `notify_history_changed`, `notify_traversal_complete` | **Exposed** | *(done on this branch)* C: `servo_webview_can_go_back` / `servo_webview_can_go_forward`, `servo_webview_go_back` / `servo_webview_go_forward`, and `ServoWebViewDelegate::notify_history_changed`. |
| Reload | `CefBrowserHost::Reload` | `WebView::reload()` | **Exposed** | *(done on this branch)* C: `servo_webview_reload` (plus `servo_webview_clear_session_history`). |
| Stop load | `CefBrowserHost::StopLoad` | none | Missing | No cancellation API. |
| Main frame | `CefBrowserHost::GetMainFrame` | none (no frame abstraction) | Missing | Servo exposes `WebView` only; there is no per-frame handle. |
| Zoom level | `CefBrowserHost::SetZoomLevel`, `GetZoomLevel` | `WebView::set_page_zoom(f32)`, `page_zoom()`; `adjust_pinch_zoom`, `pinch_zoom`, `device_pixels_per_css_pixel` | **Exposed** | *(done on this branch)* C: `servo_webview_set_page_zoom` / `servo_webview_page_zoom`, pinch zoom (`servo_webview_adjust_pinch_zoom` / `servo_webview_get_pinch_zoom`), and HiDPI scale factor (`servo_webview_set_hidpi_scale_factor` / `servo_webview_get_hidpi_scale_factor`). |
| Windowless frame rate | `CefBrowserHost::SetWindowlessFrameRate` | `RenderingContext::refresh_driver()` (indirect frame pacing) | Missing | No explicit frame-rate setter. |

### Frames

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Frame URL | `CefFrame::GetURL` | `WebView::url()` (main frame only) | **Exposed** (partial) | *(done on this branch)* C: `servo_webview_get_url` (buffer-query). Main frame only; no subframe API. |
| Frame text | `CefFrame::GetText` | `WebView::evaluate_javascript` (embedder must serialize DOM) | Partial | No `GetText`; approximate via `servo_webview_evaluate_javascript` (now in C) + JSON serialization; page title/status text via `servo_webview_get_title` / `servo_webview_get_status_text`. |
| Frame load | `CefFrame::LoadURL` | `WebView::load` (main frame) | Partial | Main frame only. |
| Execute JS | `CefFrame::ExecuteJavaScript` | `WebView::evaluate_javascript(script, callback) -> Result<JSValue, JavaScriptEvaluationError>` | **Exposed** | *(done on this branch)* C: `servo_webview_evaluate_javascript(webview, script, callback, user_data)`; the callback receives JSON text plus an error string. No user-gesture flag (next row). |
| Execute JS with gesture | `CefFrame::ExecuteJavaScriptWithUserGesture` | none | Missing | No user-gesture flag. |
| Frame messaging | `CefFrame::PostMessage`, `CefProcessMessage`, `ProcessMessage` | `MessageRouter` (`message_router.rs`) + `WebView::{post_message, add_message_handler, remove_message_handler}`; JS `window.servoMessageRouter` | **Partial/Exposed** | Native→JS via `servo_webview_post_message` + `CustomEvent`; JS→native via `servo_webview_add_message_handler` + `navigator.sendBeacon` to `servo:message` scheme. JS API: `postMessage`, `addEventListener`, `removeEventListener`. |

---

## 3. Handlers

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Client + lifespan | `CefClient`, `CefLifeSpanHandler` (`OnAfterCreated`, `OnBeforeClose`, `OnBeforePopup`) | `WebViewDelegate::{notify_closed, notify_crashed, request_create_new}` | **Exposed** (partial) | *(done on this branch)* C delegate includes `notify_closed` and `notify_crashed`. Embedder can observe crash reason and backtrace. |
| Load | `CefLoadHandler` (`OnLoadingStateChange`, `OnLoadStart`, `OnLoadEnd`, `OnLoadError`) | `WebViewDelegate::{notify_load_status_changed, notify_url_changed, notify_page_title_changed, request_navigation}`; `WebView::load_status()` | **Exposed** | C: `notify_load_status_changed` (`SERVO_LOAD_STATUS_*`), `servo_webview_get_load_status`, delegate callbacks `notify_url_changed` / `notify_page_title_changed`, `notify_load_started`, `notify_load_ended`, `notify_load_error`, plus `servo_webview_get_url` / `get_title` / `get_status_text`. |
| Display | `CefDisplayHandler` (`OnTitleChange`, `OnAddressChange`, `OnCursorChange`, `OnStatusMessage`, `OnConsoleMessage`, `OnFaviconURLChange`) | `WebViewDelegate::{notify_page_title_changed, notify_url_changed, notify_cursor_changed, notify_status_text_changed, notify_favicon_changed, show_console_message}`; `WebView::{page_title, url, status_text, favicon, cursor}` | **Exposed** | *(done on this branch)* C reads title/url/status text and receives `notify_url_changed`, `notify_page_title_changed`, `notify_cursor_changed` (`SERVO_CURSOR_*`), `notify_status_text_changed`, `notify_favicon_changed`, and `show_console_message` (`SERVO_CONSOLE_LOG_LEVEL_*`). |
| Request | `CefRequestHandler` (`OnBeforeBrowse`, `OnCertificateError`, `OnAuthCredentials`, `GetResourceRequestHandler`) | `WebViewDelegate::{request_navigation, request_authentication, request_permission, load_web_resource}` | **Exposed** | *(done on this branch)* C: `request_authentication` with `ServoAuthenticationCallback`, `request_permission` (`ServoPermissionType`), and `intercept_request` (`ServoWebResourceRequest` / `ServoWebResourceResponse`). |
| Resource request | `CefResourceRequestHandler` (`OnBeforeResourceLoad`, `GetResourceHandler`) | `WebViewDelegate::load_web_resource` + `WebResourceLoad::intercept` | **Exposed** | *(done on this branch)* C: `intercept_request` delegate callback with request inspection and custom response body/headers/status. |
| Context menu | `CefContextMenuHandler` (`OnBeforeContextMenu`, `OnContextMenuCommand`) | `WebViewDelegate::show_embedder_control(EmbedderControl::ContextMenu)`; `ContextMenu`/`ContextMenuAction` | Partial | Rust-only; not in C. |
| Dialog | `CefDialogHandler::OnFileDialog` | `WebViewDelegate::{show_embedder_control(EmbedderControl::{FilePicker, SelectElement}), request_permission, show_bluetooth_device_dialog}` | **Exposed** (partial) | *(done on this branch)* C: `show_file_picker` delegate callback with `ServoFilePickerCallback` supporting single and multiple file selection. |
| JS dialog | `CefJSDialogHandler` (`OnJSDialog`, `OnBeforeUnloadDialog`) | `WebViewDelegate::{show_embedder_control(EmbedderControl::{AlertDialog, ConfirmDialog, PromptDialog}), request_unload}` | **Exposed** | *(done on this branch)* C: `show_alert_dialog`, `show_confirm_dialog`, `show_prompt_dialog`, and `request_unload` with safe trampolines. |
| Focus | `CefFocusHandler` (`OnSetFocus`, `OnGotFocus`) | `WebView::{set_focused, focused}`; `WebViewDelegate::{notify_focus_changed, notify_got_focus}` | **Exposed** | *(done on this branch)* C: `servo_webview_set_focused` / `servo_webview_focused`, and delegate callbacks `notify_focus_changed` / `notify_got_focus`. |
| Keyboard | `CefKeyboardHandler` (`OnPreKeyEvent`, `OnKeyEvent`) | `WebView::notify_input_event(InputEvent) -> InputEventId`; `notify_input_event_handled` | **Exposed** | *(done on this branch)* C: `servo_webview_notify_key_event`, `servo_webview_notify_mouse_button_event`, `servo_webview_notify_mouse_move_event`, `servo_webview_notify_mouse_leave_event`, `servo_webview_notify_touch_event`, `servo_webview_notify_wheel_event`, `servo_webview_notify_input_event_handled`. All `InputEvent` variants mapped. |
| IME | `CefBrowserHost::ImeSetComposition`, `ImeCommitText`, `ImeCancelComposition` | `InputEvent::Ime(ImeEvent)`; `EmbedderControl::InputMethod` | **Exposed** | *(done on this branch)* C: `servo_webview_notify_ime_composition_start`, `servo_webview_notify_ime_composition_update`, `servo_webview_notify_ime_composition_end`, `servo_webview_notify_ime_dismissed`, plus `show_ime` and `hide_ime` in `ServoWebViewDelegate`. |
| Scroll | `CefBrowserHost::SendMouseWheelEvent` | `WebView::notify_scroll_event(Scroll, WebViewPoint)` | **Exposed** | *(done on this branch)* C: `servo_webview_scroll_by`, `servo_webview_scroll_to_start`, `servo_webview_scroll_to_end`. |
| Download | `CefDownloadHandler` (`OnBeforeDownload`, `OnDownloadUpdated`) | `DownloadItem`, `DownloadDelegate` | **Exposed** | *(done on this branch)* C: `servo_download_item_get_*`, `servo_download_item_cancel`, and `ServoWebViewDelegate::{on_download_started, on_download_updated}`. |
| Drag | `CefDragHandler` (`OnDragEnter`, `OnDraggableRegionsChanged`) | none (no drag/drop API) | Missing | No drag-and-drop support surfaced. |
| Find | `CefFindHandler::OnFindResult` | `WebView::{find, find_next, clear_find}`; `WebViewDelegate::on_find_result` | **Exposed** | *(done on this branch)* C: `servo_webview_find`, `servo_webview_find_next`, `servo_webview_find_clear`, `servo_find_result_get_*`, and `ServoWebViewDelegate::on_find_result`. |
| Render (OSR) | `CefRenderHandler` (`GetViewRect`, `OnPaint`, `OnCursorChange`, `OnAcceleratedPaint`) | `WebViewDelegate::{notify_new_frame_ready, notify_cursor_changed}`; `WebView::{paint, size, resize, cursor, take_screenshot}` | **Exposed** (partial) | C exposes OnPaint: `notify_new_frame_ready` + `servo_webview_paint`, `servo_webview_get_size` / `servo_webview_resize`, `take_screenshot`, and `notify_cursor_changed`. Windowed and offscreen contexts in C. |
| Audio / Media | `CefAudioHandler`, `CefMediaSession` | `WebView::notify_media_session_action_event` | **Exposed** (partial) | *(done on this branch)* C: `servo_webview_notify_media_session_action` with `SERVO_MEDIA_SESSION_ACTION_*` (play, pause, seek, track navigation, stop). |

---

## 4. Rendering

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Windowed rendering | `CefBrowserHost::CreateBrowser` (windowed) + `CefWindowInfo` | `WindowRenderingContext::new(display_handle, window_handle, size)` (`components/shared/paint/rendering_context.rs`) | Missing | Rust supports it via `raw-window-handle`; C exposes no windowed context. |
| Windowless / OSR | `CefRenderHandler` + `SetWindowlessFrameRate` | `OffscreenRenderingContext` (FBO composition), `SoftwareRenderingContext` | Partial | Rust has offscreen contexts; C only exposes `servo_rendering_context_create_software`. |
| Shared / accelerated textures | `OnAcceleratedPaint`, `CefSharedTexture` | `RenderingContext::{create_texture, destroy_texture, connection}` | Missing | GPU texture sharing hooks exist in Rust but are not surfaced to C. |
| View rect / size | `CefRenderHandler::GetViewRect` | `RenderingContext::{size, size2d, resize}`; `WebView::{size, resize}` | Partial | C context size is fixed at creation; *(done on this branch)* `servo_webview_get_size` / `servo_webview_resize` cover the webview-size half. No view-rect callback. |
| Cursor | `CefRenderHandler::OnCursorChange`, `CefDisplayHandler::OnCursorChange` | `WebView::cursor()`, `WebViewDelegate::notify_cursor_changed` | Partial | Rust-only; not in C. |

---

## 5. Networking

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Request / response objects | `CefRequest`, `CefResponse` | `WebResourceRequest` / `WebResourceResponse` (`webview_delegate.rs`); `net_traits::request::Request`, `net_traits::response::Response` | **Exposed** (partial) | *(done on this branch)* C: `ServoWebResourceRequest` / `ServoWebResourceResponse` opaque types with getters/setters for method, URL, headers, status, body. Available in `intercept_request` delegate callback and protocol handler callback. |
| Custom scheme / resource handler | `CefSchemeHandlerFactory`, `CefResourceHandler`, `RegisterSchemeHandlerFactory` | `protocol_handler::{ProtocolHandler, ProtocolRegistry}`; `ServoBuilder::protocol_registry` | **Exposed** | *(done on this branch)* C: `servo_builder_register_protocol_handler` registers a callback for a custom scheme. The callback receives `ServoWebResourceRequest` and populates `ServoWebResourceResponse`. |
| Cookies | `CefCookieManager` | `SiteDataManager::{cookies_for_url, cookies_for_url_async, set_cookie_for_url, clear_cookies, clear_session_cookies}`; `CookieSource` | **Exposed** | *(done on this branch)* C: `servo_site_data_cookies_for_url`, `servo_site_data_set_cookie`, `servo_site_data_clear_cookies`, `servo_site_data_clear_session_cookies`, plus `servo_get_site_data_manager` / `servo_site_data_manager_free`. Async variants remain Rust-only. |
| Request context / isolation | `CefRequestContext` (isolation, cache path, proxy) | Internal public/private browsing contexts (no embedder-selectable context); `NetworkManager` for HTTP cache | Missing | No `CefRequestContext` analogue; no proxy/cache-path configuration in the embedding API. |
| Certificate errors | `CefRequestHandler::OnCertificateError` | `Opts::ignore_certificate_errors` | Partial | Global boolean only (C: `servo_options_set_ignore_certificate_errors`); no per-request decision callback. |

---

## 6. JavaScript integration

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| JS bindings / context | `CefV8Handler`, `CefV8Context`, `CefV8Value` | none (no embedder JS binding API) | Missing | Servo does not expose V8/SpiderMonkey bindings to embedders. |
| Extension registration | `CefRegisterExtension` | `UserContentManager::{add_script, remove_script}` (`UserScript`); `add_stylesheet`/`remove_stylesheet` (`UserStyleSheet`) | Partial | Rust injects scripts/styles on load; not exposed to C, and not the same as a persistent extension/context API. |
| Message router | `CefMessageRouter` (`CefMessageRouterBrowserSide`, `AddHandler`) | `MessageRouter` (`message_router.rs`) + `WebView::{post_message, add_message_handler, remove_message_handler}`; JS `window.servoMessageRouter` | **Partial/Exposed** | Native→JS via `servo_webview_post_message` + `CustomEvent`; JS→native via `servo_webview_add_message_handler` + `navigator.sendBeacon` to `servo:message` scheme. JS API: `postMessage`, `addEventListener`, `removeEventListener`. `UserContentManager::inject_message_router` enables it. |
| Value containers | `CefValue`, `CefDictionaryValue`, `CefListValue` | `JSValue` enum (result of `WebView::evaluate_javascript`) | Partial | *(done on this branch, JSValue→JSON only)* C: `servo_webview_evaluate_javascript` returns results serialized to JSON text; there are no structured C value types (`CefValue`/dictionary/list equivalents) for richer interop. |

---

## 7. Process model

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| Multi-process | `CefSettings.multi_process`, `CefExecuteProcess`, `CefProcessMessage` | `Opts::{multiprocess, force_ipc}`; feature-gated `servo::run_content_process(token)`; `#[cfg(feature = "multiprocess")]` | **Partial** | C can set the flags (`servo_options_set_multiprocess`, `servo_options_set_force_ipc`) and bootstrap content processes via `servo_run_content_process(token)` / `servo_create_sandbox()` (requires `multiprocess` feature). |
| Sandbox | `CefSettings.no_sandbox` | `Opts::sandbox`; `create_sandbox()` in the content process | **Partial** | C exposes `servo_options_set_sandbox` and `servo_create_sandbox()` for content-process bootstrap; guidance added in `C_API_GUIDE.md`. |
| Zygote / fork server | `CefSettings` (browser/zygote launch) | none (no zygote/fork-server abstraction) | Missing | Process creation is internal; no embedder-facing zygote control. |
| Task runner / post task | `CefTaskRunner`, `CefPostTask` | none public; internal `ConstellationProxy` message channel (`proxies.rs`) | Partial | `spin_event_loop` is the only embedder-driven loop; no public post-task/run-on-loop API. |
| Reference counting | `CefRefPtr`, `CefBaseRefCounted` | Rust `Rc`/`Box`; C handles are manual ownership | Partial | No intrusive refcount across the boundary; C uses raw pointers + `*_free`. |

---

## 8. Tooling

| CEF concept | CEF API | Servo Rust equivalent | Servo C API status | Notes / work needed |
| --- | --- | --- | --- | --- |
| DevTools remote debugging | `CefSettings.remote_debugging_port`, `CefBrowserHost::OpenDevTools` | `ServoDelegate::{notify_devtools_server_started, request_devtools_connection}` | **Exposed** | C exposes `servo_builder_set_delegate` with `notify_devtools_server_started` and `request_devtools_connection` callbacks. Enable via `devtools_server_enabled` preference. No browser-host DevTools toggle. |
| Command line | `CefCommandLine`, `GetCommandLine`, `InitCommandLine` | `servo_config::opts` + `Preferences` (CLI parsing lives in `servoshell`, not libservo) | Partial | C has per-field `ServoOptions` setters; no command-line parser/`CefCommandLine` equivalent. |

---

## 9. Prioritized gap list

### P0 — required for a credible CEF-like surface

1. **ABI/versioning policy** — no SemVer statement, no version query, no
   deprecation path. *Progress on this branch:* a version query
   (`servo_capi_version()`) and `SERVO_CAPI_VERSION_*` macros now exist, and
   `ABI_STABILITY.md` defines the policy; structured `ServoError` returns and a
   deprecation path (§7) are still outstanding.
2. **Panic-free FFI boundary** — replace `assert!`/`unwrap` with structured error
   returns across `ffi/capi` (currently dozens of `assert!` call sites and
   `.to_str().unwrap()`).
3. **JavaScript execution in C** — *done on this branch:*
   `servo_webview_evaluate_javascript` returns JSON text via a callback. Remaining:
   a user-gesture flag and the message router (P1 #8).
4. **Navigation controls in C** — *done on this branch:* `servo_webview_can_go_back`
   / `servo_webview_can_go_forward`, `servo_webview_go_back` /
   `servo_webview_go_forward`, `servo_webview_reload`,
   `servo_webview_clear_session_history`. History-change notifications stay
   Rust-only.
5. **Core C delegate surface** — *partially done:* `ServoWebViewDelegate` now
   carries `notify_url_changed`, `notify_page_title_changed`, and `notify_closed`;
   the load start/end/error, display (cursor/favicon/console), and other lifespan
   callbacks are still missing.
6. **Windowed + offscreen rendering contexts in C** — expose
   `WindowRenderingContext` and `OffscreenRenderingContext` (plus view-rect/cursor
   hooks), not just `SoftwareRenderingContext`. `servo_webview_resize` now covers
   the view-resize half.
7. **Explicit close** — *done on this branch:* `servo_webview_close` +
   `ServoWebViewDelegate::notify_closed`.

### P1 — needed for most real embedders

8. **Native↔JS message router** (`CefMessageRouter` analogue).
9. **Resource interception + custom schemes in C** —
   `WebResourceLoad::intercept` and `protocol_registry`. *(done on this branch)*
10. **Cookies and site data in C** — *done on this branch:* `servo_get_site_data_manager`
    plus the `servo_site_data_*` cookie APIs, HTTP-cache list/clear via
    `servo_get_network_manager` + `servo_network_manager_get_cache_entries`/`_clear_cache`,
    and user script/stylesheet injection via `servo_user_content_manager_add_script`/
    `_remove_script`/`_add_style_sheet`/`_remove_style_sheet`.
11. **Dialogs, context menus, permissions, authentication** in C —
    `show_embedder_control`, `request_permission`, `request_authentication`.
12. **Keyboard/focus in C** — `notify_input_event`, `set_focused`. **Done on this branch:** `servo_webview_notify_key_event`, `servo_webview_notify_mouse_button_event`, `servo_webview_notify_mouse_move_event`, `servo_webview_notify_mouse_leave_event`, `servo_webview_notify_touch_event`, `servo_webview_notify_wheel_event`, `servo_webview_notify_input_event_handled`, and focus delegate callbacks `notify_focus_changed`, `notify_got_focus`.
13. **DevTools in C** — *done on this branch:* `servo_builder_set_delegate` with `notify_devtools_server_started` and `request_devtools_connection` callbacks.
14. **Multi-process bootstrap** for C embedders — content-process entry and
    sandbox setup.

### P2 — parity polish

15. **Downloads, drag-and-drop, audio** APIs (currently absent
    even in Rust).
16. **Shared/accelerated texture OSR** and frame-rate control.
17. **`CefRequestContext`-style isolation** (cache path, profiles, proxy config).
18. **Value containers** (`CefValue`/dictionary/list) for JS interop.
19. **Reference-counted handles** / safer ownership model.
20. **Task-runner-style post API** and a `CefCommandLine` equivalent.

---

## 10. Suggested phased path

**Phase 0 — Contract & safety (P0 items 1–2).**
Land the ABI/versioning policy and a version query; convert the FFI boundary to
return errors instead of panicking; add CI checks that exercise null/invalid
inputs without aborting.

**Phase 1 — Core browser surface (P0 items 3–7).**
Expose navigation, JavaScript, the core delegate categories, windowed/offscreen
rendering contexts, resize/cursor/view-rect, and explicit close. This is the
minimum for a C application to render a real, interactive page. *(On this branch,
P0 items 3 (JS execution), 4 (navigation), and 7 (explicit close) are
implemented; item 5 is partly implemented, and resize is covered; the
windowed/offscreen contexts remain open work.)*

**Phase 2 — Content & networking (P1 items 8–12).**
Add the message router, resource interception, custom schemes *(done on this branch)*,
cookies/site data, dialogs/permissions, **and keyboard/focus (done on this branch).** This makes
the surface usable for host applications that need to control content.

**Phase 3 — Operations & processes (P1 items 13–14, P2).**
DevTools, multi-process bootstrap, texture sharing, request-context isolation,
value containers, and ownership/task-runner polish.

See [`ROADMAP.md`](ROADMAP.md) for milestones and sequencing, and
[`AUDIT.md`](AUDIT.md) for the underlying inventory and risk register.
