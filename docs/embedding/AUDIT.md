# Servo Embedding API — Audit

Status: **Audit** · Branch: `embedding/cef-parity` · Baseline commit: `3dcc4e02bc1`

This document is an engineering audit of the embedding surface Servo exposes today,
the maturity of each area, the test/build infrastructure around it, and the
technical debt that stands between the current API and a CEF-like embedding
experience. For the concept-by-concept gap analysis against
Chromium Embedded Framework (CEF), see [`CEF_PARITY.md`](CEF_PARITY.md). For planned
sequencing see [`ROADMAP.md`](ROADMAP.md) (companion document).

> **Documentation only.** Nothing in this file is a code change. It describes the
> committed state of the branch at the baseline commit above. Paths are relative
> to the repository root.

---

## 1. Scope & method

**Goal.** Establish an accurate, concrete picture of what an embedder can do
with Servo *today*, via both the native Rust API and the C ABI, so that we can
plan a CEF-like surface without inventing capabilities that do not exist.

**In scope**

- The `servo` crate (`components/servo`), i.e. libservo.
- The `servo-capi` crate (`ffi/capi`), the C ABI.
- The `servo-capi-tests` crate (`tests/capi`) and its C test sources.
- Delegate traits, embedder-provided services, and rendering contexts reachable
  from those crates.
- Build/test/CI infrastructure that exercises the C ABI.

**Out of scope**

- `ports/servoshell` internals (the reference application, not an embedding API).
- Internal engine components (script, layout, style, net) except where they
  surface through the embedding API.
- Stability guarantees for the C ABI (there are none yet — see §5).

**Method.** Read the public surface of `components/servo` and every file in
`ffi/capi`; enumerated exported symbols via `#[unsafe(no_mangle)]` and the
cbindgen configuration; read the C test sources and `tests/capi/build.rs`; and
inspected the CI workflows that run the C API tests. Where a CEF concept has no
Servo equivalent, it is recorded as such rather than approximated.

**At a glance**

| Metric | Value |
| --- | --- |
| Exported C functions | **51** |
| C delegate callback fields | **3** (`user_data`, `notify_load_status_changed`, `notify_new_frame_ready`) |
| `WebViewDelegate` methods | ~30 |
| `ServoDelegate` methods | 6 |
| Rendering contexts exposed to C | 1 (`SoftwareRenderingContext`) |
| Rendering contexts in Rust | 3 (`Window`, `Offscreen`, `Software`) |
| C API unit tests (C) | 17 at baseline; **33 on this branch** |
| C API integration tests (C) | 1 |
| ABI/versioning policy | none |
| C++ API | none |

> **Branch delta (implemented on this branch since the baseline).** The inventory
> and metrics below describe the baseline commit `3dcc4e02bc1`. Since then the C
> API on this branch adds: the version constants + `servo_capi_version()`
> (`lib.rs`); runtime preference setters `servo_set_preference_bool/i64/u64/string`
> (`servo.rs`); navigation (`servo_webview_can_go_back/forward`,
> `servo_webview_go_back/forward`, `servo_webview_reload`,
> `servo_webview_clear_session_history`); getters (`servo_webview_get_url/title/
> status_text`, `servo_webview_get_load_status`, `servo_webview_get_size`); view
> control (`servo_webview_set_page_zoom/page_zoom`, `servo_webview_resize`,
> `servo_webview_set_focused/focused`, `servo_webview_show/hide`,
> `servo_webview_notify_theme_change`); JavaScript evaluation
> (`servo_webview_evaluate_javascript` + JSON serialization in a new
> `ffi/capi/javascript.rs`); and two new delegate fields (`notify_url_changed`,
> `notify_page_title_changed`). Current counts are **76 exported C functions**
> (25 added) and **5 delegate fields** (2 added). §2.2 below is annotated with
> *(added)* marks where a group grew.

---

## 2. Inventory

### 2.1 Rust embedding API (`servo` crate, `components/servo`)

The Rust API is the primary embedding surface and is organized around a few
opaque handles plus two delegate traits.

#### `Servo` / `ServoBuilder` (`components/servo/servo.rs`)

`ServoBuilder` configures and produces a `Servo`:

| Builder method | Purpose |
| --- | --- |
| `build() -> Servo` | Constructs the engine. |
| `opts(Opts)` | Engine options (`servo_config::opts::Opts`). |
| `preferences(Preferences)` | Preference set (`servo_config::prefs::Preferences`). |
| `event_loop_waker(Box<dyn EventLoopWaker>)` | Wakes the embedder thread when Servo has work. |
| `protocol_registry(ProtocolRegistry)` | Registers custom URL scheme handlers. |

`Servo` methods:

| Method | Purpose |
| --- | --- |
| `spin_event_loop()` | Pump Servo; must be driven by the embedder. |
| `setup_logging()` | Initialize logging. |
| `delegate() -> Rc<dyn ServoDelegate>` / `set_delegate(Rc<dyn ServoDelegate>)` | Get/replace the app-level delegate. |
| `create_memory_report(GenericCallback<MemoryReportResult>)` | Request a memory report. |
| `execute_webdriver_command(WebDriverCommandMsg)` | Drive WebDriver. |
| `set_preference(&str, PrefValue)` | Update a preference at runtime. |
| `network_manager() -> Ref<NetworkManager>` | Network-layer state (HTTP cache). |
| `site_data_manager() -> &SiteDataManager` | Site data (cookies, storage). |
| `forward_accessibility_action(ActionRequest)` | Forward an AccessKit action to content. |
| `register_webxr_registry(Box<dyn WebXrRegistry>)` | Register WebXR device discovery. |
| `initialize_gl_accelerated_media(NativeDisplay, GlApi, GlContext)` | GL-accelerated media setup. |

#### `WebView` / `WebViewBuilder` (`components/servo/webview.rs`)

`WebViewBuilder`:

| Builder method | Purpose |
| --- | --- |
| `new(&Servo, Rc<dyn RenderingContext>)` | Create the builder for a view. |
| `delegate(Rc<dyn WebViewDelegate>)` | Attach the per-view delegate. |
| `url(Url)` | Initial URL. |
| `hidpi_scale_factor(...)` | Device scale factor. |
| `user_content_manager(Rc<UserContentManager>)` | Script/style injection. |
| `clipboard_delegate(Rc<dyn ClipboardDelegate>)` | Clipboard handling. |
| `gamepad_delegate(Rc<dyn GamepadDelegate>)` | Gamepad/haptics. |
| `build() -> WebView` | Construct the view. |

`WebView` navigation / lifecycle:

`load(Url)`, `load_request(UrlRequest)`, `reload()`, `can_go_back()`,
`go_back(usize) -> TraversalId`, `can_go_forward()`, `go_forward(usize) -> TraversalId`,
`clear_session_history()`, `load_status() -> LoadStatus`, `url() -> Option<Url>`,
`page_title() -> Option<String>`, `status_text() -> Option<String>`,
`favicon() -> Option<Ref<Image>>`, `cursor() -> Cursor`.

Geometry / visibility / theme:

`resize(PhysicalSize<u32>)`, `size() -> DeviceSize`, `hidpi_scale_factor()`,
`set_hidpi_scale_factor(...)`, `set_focused(bool)`, `focused() -> bool`, `show()`,
`hide()`, `notify_theme_change(Theme)`, `animating() -> bool`.

Input / zoom:

`notify_scroll_event(Scroll, WebViewPoint)`, `notify_input_event(InputEvent) -> InputEventId`,
`notify_media_session_action_event(MediaSessionActionType)`, `set_page_zoom(f32)`,
`page_zoom() -> f32`, `adjust_pinch_zoom(f32, DevicePoint)`, `pinch_zoom() -> f32`,
`device_pixels_per_css_pixel()`.

Rendering / content / a11y:

`paint()`, `take_screenshot(Option<WebViewRect>, impl FnOnce(Result<RgbaImage, ScreenshotCaptureError>))`,
`evaluate_javascript<T: ToString>(script, impl FnOnce(Result<JSValue, JavaScriptEvaluationError>))`,
`user_content_manager()`, `exit_fullscreen()`, `toggle_webrender_debugging(WebRenderDebugOption)`,
`capture_webrender()`, `accesskit_tree_id()`, `set_accessibility_active(bool)`, and
accessibility tree update handling.

#### Delegates and embedder services

- **`WebViewDelegate`** (`components/servo/webview_delegate.rs`, ~30 methods):
  notifications `notify_url_changed`, `notify_page_title_changed`,
  `notify_status_text_changed`, `notify_animating_changed`,
  `notify_load_status_changed`, `notify_cursor_changed`, `notify_favicon_changed`,
  `notify_new_frame_ready`, `notify_history_changed(Vec<Url>, usize)`,
  `notify_traversal_complete`, `notify_closed`, `notify_input_event_handled`,
  `notify_crashed`, `notify_media_session_event`, `notify_fullscreen_state_changed`,
  `notify_accessibility_tree_update`; requests `screen_geometry`,
  `request_navigation(NavigationRequest)`, `request_unload`,
  `request_move_to`, `request_resize_to`, `request_protocol_handler`,
  `request_create_new`, `request_permission(PermissionRequest)`,
  `request_authentication`, `show_bluetooth_device_dialog`,
  `show_embedder_control(EmbedderControl)`, `hide_embedder_control`,
  `load_web_resource(WebResourceLoad)`, `show_notification`,
  `show_console_message`.
- **`ServoDelegate`** (`components/servo/servo_delegate.rs`, 6 methods):
  `notify_error(ServoError)`, `notify_devtools_server_started(port, token)`,
  `request_devtools_connection(AllowOrDenyRequest)`, `load_web_resource(WebResourceLoad)`,
  `show_notification(Notification)`, `show_console_message(level, message)`.
- **`ClipboardDelegate`** (`clipboard_delegate.rs`): async string reads via `StringRequest`.
- **`GamepadDelegate`** (`gamepad_delegate.rs`, `gamepad` feature): haptic effect requests.
- **`SiteDataManager`** (`site_data_manager.rs`): `site_data(StorageType)`,
  `clear_site_data(&[&str], StorageType)`, `clear_cookies(callback)`,
  `clear_session_cookies(callback)`, `cookies_for_url`, `cookies_for_url_async`,
  `set_cookie_for_url`. `StorageType` covers cookies, local, session.
- **`NetworkManager`** (`network_manager.rs`): `cache_entries()`, `clear_cache()`
  (in-memory HTTP cache only).
- **`UserContentManager`** (`user_content_manager.rs`): `add_script`/`remove_script`
  (`UserScript`), `add_stylesheet`/`remove_stylesheet` (`UserStyleSheet`). **No
  native↔JS message router.**
- **Custom schemes** (`protocol_handler` module, re-exported from `components/servo/lib.rs`):
  `ProtocolHandler`, `ProtocolRegistry`, wired via `ServoBuilder::protocol_registry`.
- **Accessibility** (AccessKit), **WebXR** (`webxr` feature), **WebDriver**
  (`execute_webdriver_command`) are supported at the Rust level.

### 2.2 C API (`servo-capi` crate, `ffi/capi`)

`servo-capi` is a `cdylib` (`[lib] name = "servo_capi"`) that thinly wraps the
`servo` crate. Headers are generated with cbindgen from
`ffi/capi/cbindgen.toml` (`language = "C"`, `parse_deps = true`), installed by
`cargo cinstall` as `servo/servo_capi.h`. Three enum types are exported and
renamed: `DiagnosticsLoggingOption` → `ServoDiagnosticsLoggingOption`,
`LoadStatus` → `ServoLoadStatus`, `ScreenshotCaptureError` →
`ServoScreenshotCaptureError`.

**There is no C++ API and no generated C++ header.**

Exported C functions, grouped (51 at baseline `3dcc4e02bc1`; **76 on this branch**):

**Lifecycle / builder (`lib.rs`, 6 → 7)** — plus the `ServoEventLoopWaker { wake_callback }` type

- `servo_builder_create`, `servo_builder_set_preferences`, `servo_builder_set_options`,
  `servo_builder_set_event_loop_waker`, `servo_builder_build`, `servo_builder_free`
- *(added)* `servo_capi_version` — returns the release version string, e.g. `"0.1.0"`;
  backed by `SERVO_CAPI_VERSION_MAJOR/MINOR/PATCH` constants.

**Servo handle (`servo.rs`, 3 → 7)**

- `servo_free`, `servo_spin_event_loop`, `servo_setup_logging`
- *(added)* `servo_set_preference_bool` / `servo_set_preference_i64` /
  `servo_set_preference_u64` / `servo_set_preference_string` — runtime preference
  setters on a built `Servo` (backed by `Servo::set_preference`), complementing the
  builder-time `ServoPreferences` API.

**Options (`options.rs`, 21)**

- `servo_options_create`, `servo_options_free`
- `servo_options_enable_time_profiling_to_file`, `servo_options_set_time_profiler_trace_path`,
  `servo_options_enable_time_profiling_to_stdout`
- `servo_options_set_hard_fail`, `servo_options_set_multiprocess`,
  `servo_options_set_force_ipc`, `servo_options_set_background_hang_monitor`,
  `servo_options_set_sandbox`, `servo_options_set_temporary_storage`,
  `servo_options_set_ignore_certificate_errors`, `servo_options_set_unminify_js`,
  `servo_options_set_unminify_css`, `servo_options_set_certificate_path`,
  `servo_options_set_local_script_source`, `servo_options_set_shaders_path`,
  `servo_options_set_config_dir`, `servo_options_set_random_pipeline_closure_probability`,
  `servo_options_set_random_pipeline_closure_seed`, `servo_options_set_debug_option`

**Preferences (`preferences.rs`, 10)**

- `servo_preferences_create`, `servo_preferences_free`
- `servo_preferences_set_bool` / `servo_preferences_get_bool`
- `servo_preferences_set_i64` / `servo_preferences_get_i64`
- `servo_preferences_set_u64` / `servo_preferences_get_u64`
- `servo_preferences_set_string` / `servo_preferences_get_string`

**Rendering context (`rendering_context.rs`, 2)**

- `servo_rendering_context_create_software(width, height)` (returns `NULL` on failure),
  `servo_rendering_context_free`

**WebView builder (`webview.rs`)**

- `servo_webview_builder_create(servo, context)`, `servo_webview_builder_set_url`,
  `servo_webview_builder_set_delegate`, `servo_webview_builder_build`,
  `servo_webview_builder_free`

**WebView (`webview.rs`, 4 → 28)**
<!-- count above includes builder + WebView functions on the branch -->

- `servo_webview_paint`, `servo_webview_load`, `servo_webview_take_screenshot`,
  `servo_webview_free`
- `servo_webview_take_screenshot` delivers raw RGBA4 pixel data via a callback and
  reports failure through `ServoScreenshotCaptureError`.
- *(added, navigation)* `servo_webview_can_go_back`, `servo_webview_can_go_forward`,
  `servo_webview_go_back(amount)`, `servo_webview_go_forward(amount)`,
  `servo_webview_reload`, `servo_webview_clear_session_history`
- *(added, getters)* `servo_webview_get_url`, `servo_webview_get_title`,
  `servo_webview_get_status_text` — buffer-query pattern: callers pass
  `(char* buffer, size_t buffer_len)` and receive the byte count required (incl. NUL);
  `servo_webview_get_load_status` (returns a `ServoLoadStatus` value);
  `servo_webview_get_size(width, height)` (either out-param may be `NULL`)
- *(added, view control)* `servo_webview_set_page_zoom(zoom)` / `servo_webview_page_zoom()`,
  `servo_webview_resize(width, height)`, `servo_webview_set_focused(bool)` /
  `servo_webview_focused()`, `servo_webview_show()`, `servo_webview_hide()`,
  `servo_webview_notify_theme_change(SERVO_THEME_LIGHT|SERVO_THEME_DARK)`

**JavaScript evaluation (`javascript.rs`, new on this branch)**

- `servo_webview_evaluate_javascript(webview, script, callback, user_data)` — the
  callback receives the result serialized to JSON plus an error string; backed by
  `WebView::evaluate_javascript` + a hand-rolled `JSValue`→JSON serializer.

**WebView delegate (`webview_delegate.rs`)**

- `ServoWebViewDelegate` struct with `user_data`,
  `notify_load_status_changed(webview, load_status, user_data)`,
  `notify_new_frame_ready(webview, user_data)`, and *(added)*
  `notify_url_changed(webview, url, user_data)`,
  `notify_page_title_changed(webview, title, user_data)`.

### 2.3 Embedder-provided services (Rust-only today)

The rich embedder surface — request interception (`WebResourceLoad::intercept`),
navigation control, permission/authentication prompts, dialogs, context menus,
clipboard, gamepad, cookies/site data, custom schemes, DevTools notifications —
exists **only as Rust traits**. Almost none of it is reachable from the C ABI
(the exceptions are the load-status and new-frame callbacks).

### 2.4 Rendering contexts (`components/shared/paint/rendering_context.rs`)

The `RenderingContext` trait abstracts an OpenGL/GLES context. Three
implementations exist in Rust:

| Context | Description | Reachable from C? |
| --- | --- | --- |
| `WindowRenderingContext` | Surfman context bound to a `raw-window-handle` window; windowed rendering. | No |
| `OffscreenRenderingContext` | FBO child of a `WindowRenderingContext`; OSR-style composition. | No |
| `SoftwareRenderingContext` | CPU/software rendering for headless rendering. | **Yes** (`servo_rendering_context_create_software`) |

The trait also exposes `create_texture`/`destroy_texture` and `connection()` for
GPU texture sharing, and `refresh_driver()` for frame pacing — none surfaced to C.

### 2.5 Process & threading model

- The embedder owns a **single main thread**. `Servo` and `WebView` are built on
  it and are `Rc`/`RefCell`-based (not `Send`/`Sync`); all API calls are
  thread-affine to that thread.
- Servo runs **internal threads** (script, layout, net, compositor, etc.).
  Internal work signals the embedder through `EventLoopWaker::wake`, and the
  embedder must call `Servo::spin_event_loop()` in response. This is an
  **external-message-pump** model; there is no multi-threaded message loop.
- Optional **multiprocess** mode (feature `multiprocess`): `Opts::multiprocess` /
  `force_ipc`, content-process entry point `servo::run_content_process(token)`,
  and sandboxing via `create_sandbox()`. Default builds are single-process.
- `ServoDelegate::notify_error` reports loss of the backend channel;
  `WebViewDelegate::notify_crashed` reports content crashes.

---

## 3. Readiness assessment

Maturity labels are qualitative: **prototype** (not safe to depend on),
**alpha** (usable, expected churn), **beta** (usable, stabilizing).

| Area | Maturity | Concrete assessment |
| --- | --- | --- |
| Rust `Servo`/`WebView` core | **alpha** | Coherent, well-documented handles; broad method set. `Rc`-based and thread-affine; no `Send`/`Sync`; API churns frequently. |
| Rust delegates | **alpha** | `WebViewDelegate` is broad (~30 hooks) and covers most CEF handler categories; `ServoDelegate` is small but covers app-level errors/DevTools. Signatures still evolving. |
| Rust embedder services | **alpha** | Site data, network cache, custom schemes, user content all present. No message router, no download/find/drag APIs, no per-request certificate handler, storage limited to cookies/local/session. |
| Rendering | **alpha** | Three contexts, GPU texture sharing hooks, refresh driver. Window + offscreen paths are exercised by the example, not by C tests. |
| C API surface | **prototype→alpha** | 76 functions on this branch (25 added: navigation, getters, zoom/resize/focus/show–hide, runtime preference setters, JS evaluation, theme); single rendering context (software); 5 delegate fields; still no close/stop-load, cookies, schemes, request interception, dialogs, DevTools, or process bootstrap. |
| C API ABI stability | **prototype** | Version query (`servo_capi_version`) + macros added on this branch; policy drafted in `ABI_STABILITY.md`; no deprecation mechanism yet, header regenerated unversioned. |
| C API safety at the boundary | **prototype** | Manual pointer ownership; pervasive `assert!` and `.unwrap()` on inputs; panics can unwind toward the FFI boundary (see §5). |
| Accessibility / WebXR / WebDriver | **alpha** | Accessibility tree **updates** are reachable from C as well as Rust as of W6.4 (`notify_accessibility_tree_update` plus `servo_webview_set_accessibility_active`); forwarding an a11y action, WebXR (`register_webxr_registry`) and WebDriver (`execute_webdriver_command`) remain Rust-only. |
| Process model | **alpha** | Either single- or multi-process, configured via options; no C-level process bootstrap. |
| Tooling | **alpha (Rust)** / **missing (C)** | DevTools server notifications via `ServoDelegate`; DevTools not reachable from C. |

---

## 4. Testing & build infrastructure

### Rust

The `servo` crate is built and tested with normal Cargo commands and via
`mach`/`cargo test`. Docs for `docs/*.md` are stubs redirecting to the external
Servo book.

### C API (`servo-capi-tests`, `tests/capi`)

Because Cargo cannot depend on a `cdylib` without unstable features,
`tests/capi/build.rs` builds and installs `servo-capi` itself:

1. Runs `cargo cinstall -p servo-capi` with `--library-type cdylib`, a dedicated
   `--target-dir target/cinstall`, and a `--destdir` inside `OUT_DIR`. **This
   requires `cargo-c` to be installed.**
2. Links the generated `libservo_capi` into the test crate
   (`cargo:rustc-link-lib=dylib=servo_capi`).
3. Compiles the C sources with the `cc` crate:
   - `tests/capi/c/test_api.c` — **33 unit tests** covering builder, options,
     preferences, rendering-context, webview-builder, and (on this branch) the
     navigation / getter / zoom / focus / size / theme / JS-evaluation /
     runtime-preference entry points.
   - `tests/capi/c/test_integration.c` — **1 integration test** that loads a
     `data:text/html,...` URL with a title and red background through a
     `SoftwareRenderingContext`, spins the event loop, waits for a frame, takes
     a screenshot, validates the 100×100 red image, and (on this branch) also
     asserts the `notify_url_changed` / `notify_page_title_changed` callbacks,
     `servo_webview_get_title`, and the JSON success + error paths of
     `servo_webview_evaluate_javascript` (`"1 + 1"` → `"2"`, and a throwing
     script reaching the error callback).
4. Rust entry points in `tests/capi/{c_api_tests.rs,c_integration_tests.rs}` call
   into `run_c_api_tests()` / `run_c_integration_tests()` and assert a zero exit.

Run locally with:

```sh
cargo test -p servo-capi-tests --locked
```

On Windows the build script additionally copies the produced DLL and the ANGLE
`libEGL.dll` / `libGLESv2.dll` next to the test binary.

### CI

A dedicated **`capi` job** exists in the platform workflows:

| Workflow | Runner | Notes |
| --- | --- | --- |
| `.github/workflows/linux-common.yml` | Linux | Installs LLVM/Clang 20.1 and EGL dev packages; installs `cargo-c@0.10.23`. |
| `.github/workflows/mac.yml` | `macos-15-intel` | Installs `cargo-c@0.10.23`. |
| `.github/workflows/mac-arm64.yml` | macOS ARM | Installs `cargo-c@0.10.23`. |
| `.github/workflows/windows.yml` | `windows-2022` | Uses clang-cl; copies ANGLE DLLs; installs `cargo-c@0.10.23`. |

All jobs run the same command: `cargo test -p servo-capi-tests --locked`. The
`capi` input is gated/throttled through `main.yml`, `try.yml`, and
`dispatch-workflow.yml` matrices. There is no dedicated Linux ARM job for the C
API beyond the shared inputs.

On top of the platform jobs, this branch adds
`.github/workflows/embedding-api.yml`, a self-contained **Embedding API (C API)**
workflow with a Linux/macOS/Windows matrix that runs `cargo test -p
servo-capi-tests --locked` and additionally regenerates the public header with
cbindgen (`cbindgen --config ffi/capi/cbindgen.toml`) and fails on an empty
output. It triggers on `ffi/capi/**`, `tests/capi/**`, `components/servo/**`,
`components/shared/embedder/**`, `docs/embedding/**`, and the workflow file
itself.

---

## 5. Key risks & technical debt

1. **The C API is tiny relative to the Rust API.** 51 functions, one rendering
   context (software), and a 3-field delegate. Everything an embedder needs for
   parity — navigation, JavaScript, cookies, schemes, interception, dialogs,
   DevTools — is Rust-only.
2. **No native↔JS message router.** `UserContentManager` injects scripts but
   provides no `PostMessage`/`ProcessMessage`-style channel. Any two-way
   communication must be built by the embedder on top of `evaluate_javascript`,
   which is itself not exposed to C.
3. **No request interception in the C API.** Rust has
   `WebViewDelegate::load_web_resource` + `WebResourceLoad::intercept`, but C
   callers cannot observe or replace resource loads, and custom
   `ProtocolRegistry` schemes are not wired into the C builder.
4. **No ABI or versioning policy.** The header is generated unversioned; there
   is no version query, no SemVer statement, no deprecation path, and no
   changelog for the C surface. (A `servo_capi_version`-style query is a
   prerequisite for any stable ABI work.)
5. **`assert!`/`unwrap` panics at the FFI boundary.** The C wrappers use
   `assert!(!ptr.is_null(), ...)` pervasively (dozens of call sites across
   `ffi/capi`) and `.to_str().unwrap()` for URLs/scripts. A null or non-UTF-8
   argument therefore panics; depending on panic strategy this can unwind across
   `extern "C"` (undefined behavior / abort). Errors should be returned, not
   asserted.
6. **No headed (windowed) or offscreen C rendering context.** Only
   `SoftwareRenderingContext` is exposed; `WindowRenderingContext` and
   `OffscreenRenderingContext` are Rust-only, so C embedders cannot produce an
   on-screen, GPU-accelerated, or OSR-composited browser.
7. **Single-thread affinity and non-`Send` handles.** `Servo`/`WebView` are
   `Rc`-based; every call must happen on the creating thread. This is workable
   but must be documented and enforced, and it rules out thread-hopping designs.
8. **Manual ownership with no reference counting.** C handles are raw
   `Box::from_raw` pointers with documented transfer-of-ownership rules. There
   is no `CefRefPtr`-style intrusive refcount; a mistake is a use-after-free.
9. **Delegate lifetime is manual.** `servo_webview_builder_set_delegate` copies
   function pointers that must outlive the `WebView`; there is no
   retain/release teardown hook.
10. **Cross-platform rendering fragility.** CI needs ANGLE on Windows and EGL
    packages on Linux purely to build/link the current C API; a real windowed
    context will need a documented per-platform toolchain story.
11. **`cargo-c` build coupling.** `tests/capi/build.rs` runs a nested `cargo`
    with a pinned `cargo-c` (0.10.23) and `--locked`, which is brittle and slow,
    and the build script re-runs on any `components/` or `ffi/` change.
12. **Thin documentation.** `docs/*.md` are stubs pointing at the external book;
    there are no local architecture, embedding, or ABI docs (this file and its
    companions are a first step).

---

## 6. Recommendations (summary)

1. **Stabilize the C contract first:** define a versioning/ABI policy and a
   `servo_capi_version`-style query, and replace FFI-boundary `assert!`/`unwrap`
   with structured error returns. (P0)
2. **Grow the C delegate** from 3 fields to the core handler surface (load,
   lifecycle, display, navigation, focus, keyboard, dialogs, context menu),
   with explicit string/ownership rules. (P0–P1)
3. **Expose JavaScript execution and a message channel** in C, then layer
   resource interception and custom schemes. (P0–P2)
4. **Expose windowed and offscreen rendering contexts** in C, plus resize and
   view-rect/cursor hooks. (P0–P1)
5. **Expose context/state APIs** (cookies/site data, cache, DevTools,
   permissions). (P1–P2)
6. **Define the process-model surface** (single vs multi-process bootstrap,
   sandbox) and a task-runner-style post API. (P1–P2)

Detailed, concept-by-concept mappings and a phased plan live in
[`CEF_PARITY.md`](CEF_PARITY.md); sequencing and milestones live in
[`ROADMAP.md`](ROADMAP.md).
