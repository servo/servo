# Servo Embedding Roadmap

A phased plan to take the C embedding API in `ffi/capi` from its current skeleton to a
"usable, CEF-class" embedding surface: a stable, documented, cross-platform C ABI over
libservo that a host application can link and drive without reading Servo internals.

This roadmap is grounded in the repository as it exists. Function names for planned work
are **indicative** and follow the existing `servo_<object>_<verb>` convention; the exact
set may change during implementation. The compatibility rules that any shipped function
must follow live in [`ABI_STABILITY.md`](./ABI_STABILITY.md).

Effort sizing used throughout:

- **S** — a few days of focused work.
- **M** — roughly 1–3 weeks.
- **L** — roughly 1–3 months, or spans multiple contributors/platforms.

## Definition of a usable / "minimum lovable" embeddable engine

Servo is a minimum lovable embeddable engine when a host application can, using only the
public C header and prebuilt binaries, on Linux/macOS/Windows:

1. Create a `Servo` instance with an options/preferences object and an event-loop waker.
2. Create one or more `WebView`s against an on-screen (windowed) or offscreen rendering
   context, and render them (`paint` + `present`).
3. Navigate (`load`, `reload`, back/forward), and read `url`, `title`, and `load_status`.
4. Forward keyboard, mouse, wheel, touch, and IME input and observe handled/consumed
   results.
5. React to load status, URL/title/status/cursor/favicon/history changes, new frames,
   and window/geometry requests.
6. Evaluate JavaScript and receive a JSON result or a typed error.
7. Serve custom schemes and control resource loading from the host.
8. Inject scripts/styles and exchange messages between native code and page JS.
9. Do all of the above through a **panic-free** ABI: no argument can crash the process, and
   failures are reported through documented return codes.
10. Link against a versioned header and library whose compatibility guarantees are stated
    and tested in CI.

The "minimum" cut (usable, not yet lovable) is items 1–6 plus panic-free behavior; the
rest is the path to parity.

## Backlog summary (prioritized)

- **P0 (blocks "usable")**
  - Panic-safety at all FFI boundaries (`catch_unwind`, no `assert!`/`unwrap` escaping).
  - Fix the `servo_builder_build` leak (builder not reclaimed).
  - Read/control surface: navigation, getters, zoom, resize, focus, show/hide, history.
  - JS evaluation with JSON result + error code.
  - URL/title delegate callbacks.
  - ABI version macros + documented ownership/threading contracts.
  - Cross-platform prebuilt artifacts and a dedicated CI workflow.
- **P1 (blocks "lovable" / CEF parity)**
  - Input event forwarding and richer delegate coverage.
  - Windowed + offscreen rendering contexts exposed in C.
  - Cursor, HiDPI, IME, clipboard, gamepad delegates.
  - Custom scheme handlers and resource-load interception in C.
  - Cookie/storage/network managers in C.
  - Native <-> JS message router and script injection.
  - DevTools / remote debugging exposure.
- **P2 (polish / ecosystem)**
  - Language bindings (C++, Rust wrapper, Python/other).
  - Reference C samples and a documented `embedding-api` workflow.
  - Multiprocess surfacing and sandbox documentation.
  - Performance and binary-size tracking for the cdylib.

---

## Phase 0 — Audit and foundations

**Goal.** Make the foundation safe, documented, testable, and predictable before widening
the API.

**Deliverables**

- Documentation set in `docs/embedding/`:
  - `ARCHITECTURE.md` (layers, object model, threading, rendering, messaging, error
    handling, CEF comparison).
  - `ROADMAP.md` (this file).
  - `ABI_STABILITY.md` (versioning, symbols, panic policy, contract templates).
- ABI policy: version macros, symbol prefix (`servo_`), enum/handle rules,
  `struct_size`-versioned structs (see `ABI_STABILITY.md`).
- Panic-safety audit of `ffi/capi/{lib,servo,webview,webview_delegate,rendering_context,options,preferences}.rs`:
  catalog every `assert!`, `unwrap`, `expect`, `panic!`, `unreachable!`, and every
  `extern "C"` entry point, and every place a C callback is invoked.
- Ownership audit; fix the `servo_builder_build` leak and confirm every consuming
  function reclaims its argument exactly once.
- Dedicated CI workflow `.github/workflows/embedding-api.yml` that builds the cdylib with
  `cargo cinstall -p servo-capi`, runs `cargo test -p servo-capi-tests --locked`, and
  (in later phases) checks the generated header and symbol list.
- Test harness notes and expansion of `tests/capi` coverage for the existing surface.

**Files / crates**

- `docs/embedding/*.md` (new).
- `ffi/capi/*.rs` (audit + targeted fixes only).
- `ffi/capi/cbindgen.toml`, `ffi/capi/Cargo.toml`.
- `tests/capi/{build.rs,c/*.c,tests/*.rs,lib.rs}`.
- `.github/workflows/embedding-api.yml` (new); existing `capi` jobs in
  `linux-common.yml`, `mac.yml`, `mac-arm64.yml`, `windows.yml` remain.

**Exit criteria**

- All three docs merged.
- ABI policy exists and the header/version macros are wired into cbindgen or the crate.
- Every FFI entry point is classified as "panic-safe" or tracked with an owner and a P0
  task; no untracked `assert!`/`unwrap` remains on an FFI path.
- `cargo test -p servo-capi-tests --locked` passes in the new workflow on all supported
  platforms.
- The builder leak is fixed and covered by a test or a documented reason it is deferred.

**Risks**

- The audit may surface deeper ownership bugs; keep fixes minimal and documented.
- CI cross-compilation cost for a cdylib (large dependencies, ANGLE on Windows).

**Effort.** M (mostly coordination + reading; code changes should be small).

---

## Phase 1 — Complete the C API read/control surface

**Goal.** Let a C host drive a webview end-to-end: navigate, read state, control view
attributes, and run JavaScript.

**Deliverables**

- Navigation (`ffi/capi/webview.rs`):
  - `servo_webview_can_go_back`, `servo_webview_can_go_forward`
  - `servo_webview_go_back`, `servo_webview_go_forward`
  - `servo_webview_reload`
  - `servo_webview_clear_session_history`
- Getters (`ffi/capi/webview.rs`):
  - `servo_webview_get_url` (owned C string or caller buffer + length)
  - `servo_webview_get_title`
  - `servo_webview_get_load_status` (returns a `ServoLoadStatus` value)
- View control:
  - `servo_webview_set_page_zoom` / `servo_webview_page_zoom` (Rust: `WebView::set_page_zoom` /
    `WebView::page_zoom`, `components/servo/webview.rs`)
  - `servo_webview_resize` (Rust: `WebView::resize`)
  - `servo_webview_set_focused` (Rust: `WebView::set_focused`)
  - `servo_webview_show` / `servo_webview_hide` (Rust: `WebView::show` / `hide`)
- JavaScript evaluation:
  - New module `ffi/capi/javascript.rs`.
  - `servo_webview_evaluate_javascript(webview, script, callback, user_data)` where the
    callback receives JSON text plus an error code (`JavaScriptEvaluationError` mapped to
    `SERVO_JAVASCRIPT_EVALUATION_ERROR_*`). Backing API:
    `WebView::evaluate_javascript` + `components/servo/javascript_evaluator.rs`.
- Delegate callbacks (`ffi/capi/webview_delegate.rs`):
  - `notify_url_changed` (backed by `WebViewDelegate::notify_url_changed`)
  - `notify_page_title_changed` (backed by `WebViewDelegate::notify_page_title_changed`)
- cbindgen exports for any new enums (e.g. JS evaluation error) and the corresponding
  `SERVO_*` constants.
- C tests in `tests/capi/c/test_api.c` for each new function, plus integration coverage in
  `tests/capi/c/test_integration.c` for navigation and JS round-trips.

**Files / crates**

- `ffi/capi/webview.rs`, `ffi/capi/webview_delegate.rs`, `ffi/capi/javascript.rs` (new),
  `ffi/capi/lib.rs` (module registration).
- `ffi/capi/cbindgen.toml` (`[export]` additions).
- `tests/capi/c/*.c`.
- Rust API is consumed as-is from `components/servo/*`; no engine changes expected.

**Exit criteria**

- A C sample can load a URL, wait for `Complete`, read `url`/`title`, navigate back and
  forward, reload, resize, focus, show/hide, set zoom, and evaluate JS returning JSON.
- New functions are panic-safe (error codes, no process aborts on bad input).
- Header regenerates cleanly and tests pass on Linux/macOS/Windows.

**Risks**

- String ownership conventions (who frees the returned string) must be consistent; adopt
  the ABI policy template and test for leaks.
- JS JSON serialization edge cases (`Undefined`, cycles, DOM handles such as
  `JSValue::Element`) need a documented mapping.

**Effort.** M.

---

## Phase 2 — Input and rendering parity

**Goal.** Make a C embedder feel like a real browser window: input, cursors, HiDPI, IME,
clipboard, and both on-screen and offscreen rendering.

**Deliverables**

- Input event forwarding (`ffi/capi/input.rs` planned):
  - A `#[repr(C)]` tagged-union or builder for `embedder_traits::InputEvent` variants
    (`components/shared/embedder/input_events.rs`): keyboard, mouse move/button/wheel,
    touch, IME, editing actions, mouse-left-viewport.
  - `servo_webview_notify_input_event(webview, event)`; mirror
    `WebView::notify_input_event` (`components/servo/webview.rs`) and expose the returned
    id. Possibly also `servo_webview_notify_scroll_event` for `Scroll`/`WebViewPoint`.
- Delegate coverage (`ffi/capi/webview_delegate.rs`):
  - `notify_input_event_handled` (carry `InputEventId` + `InputEventResult` bits)
  - `notify_cursor_changed` (map `embedder_traits::Cursor`)
  - `notify_status_text_changed`
  - `notify_history_changed` (entries + current index)
  - `notify_animating_changed`
  - `screen_geometry` callback (so `window.screen*` and windowed contexts work)
    *Done on this branch:* `ServoScreenGeometry` fill-struct + `screen_geometry` callback.
  - `request_move_to` / `request_resize_to` *Done on this branch:* exposed as
    `ServoWebViewDelegate` callbacks (device-pixel point/size).
  - `request_create_new` (popups) using the existing `CreateNewWebViewRequest`
- Rendering contexts (`ffi/capi/rendering_context.rs`):
  - `servo_rendering_context_create_window(display_handle, window_handle, width, height)`
    backed by `WindowRenderingContext` (`components/shared/paint/rendering_context.rs`).
  - `servo_rendering_context_create_offscreen(parent, width, height)` backed by
    `OffscreenRenderingContext`.
  - `servo_rendering_context_present(context)` and `resize` exposed for windowed paths.
- HiDPI: `servo_webview_set_hidpi_scale_factor` / getter (Rust:
  `WebView::set_hidpi_scale_factor` / `hidpi_scale_factor`).
- IME and form controls:
  - `show_embedder_control` / `hide_embedder_control` callbacks, at least for
    `InputMethodControl`, `SelectElement`, `ColorPicker`, `FilePicker`, `ContextMenu`,
    and `SimpleDialog`, with respond functions.
- Clipboard and gamepad:
  - `ClipboardDelegate` C struct + `StringRequest` responder functions
    (`servo_webview_clipboard_request_success`/`_failure`), or a simpler synchronous
    text API where the platform allows.
  - `GamepadDelegate` C struct + `GamepadHapticEffectRequest` success/failure.
- Reference host: document and test a minimal C "browser window" sample driving input.

**Files / crates**

- `ffi/capi/{input.rs (new), rendering_context.rs, webview_delegate.rs, webview.rs, lib.rs}`.
- `tests/capi/c/*.c`, `tests/capi/build.rs` (if new sources).
- `ports/servoshell` is the behavioral reference for input/cursor/IME/clipboard/refresh.

**Exit criteria**

- A C embedder can open a windowed context, forward input, observe handled results and
  cursor changes, create popups, and drive an offscreen context for compositing.
- Clipboard and gamepad delegates at least round-trip a request/response.
- Tested on Linux, macOS, and Windows (windowed contexts are inherently platform-heavy;
  document unsupported combos honestly).

**Risks**

- Platform window/OpenGL handle plumbing is the largest cross-platform risk.
- Input event struct layout is a compatibility-sensitive decision; lock it down with the
  ABI policy before shipping.
- IME and offscreen GL behavior differ per platform.

**Effort.** L.

---

## Phase 3 — Networking and content control

**Goal.** Give hosts control over how content is fetched and stored, without patching
Servo.

**Deliverables**

- Custom scheme handlers in C:
  - Expose `components/servo/lib.rs`'s `protocol_handler` module (`ProtocolHandler`,
    `ProtocolRegistry`, fetch `Request`/`Response`) through a C callback interface, and
    wire it through `ServoBuilder::protocol_registry`
    (`components/servo/servo.rs`). The Rust interface carries a "needs cleanup" TODO;
    the C binding should initially target a documented subset.
- Resource load interception in C:
  - `ServoDelegate::load_web_resource` and `WebViewDelegate::load_web_resource`
    (`components/servo/servo_delegate.rs`, `webview_delegate.rs`) surface
    `WebResourceLoad` / `InterceptedWebResourceLoad` (`intercept`, `send_body_data`,
    `finish`, `cancel`).
- Cookies, storage, and network managers in C:
  - `SiteDataManager` (`components/servo/site_data_manager.rs`): list/clear cookies and
    localStorage/sessionStorage, sync and async variants.
  - `NetworkManager` (`components/servo/network_manager.rs`): list/clear HTTP cache.
    *Done on this branch:* `servo_get_network_manager` plus
    `servo_network_manager_get_cache_entries` (JSON array of keys, buffer-query pattern)
    and `servo_network_manager_clear_cache`.
  - `servo_site_data_*` and `servo_network_*` functions plus async callback plumbing.
- Settings/preferences already exist; extend where needed (e.g. user agent, proxy).
- Tests: a C integration test that serves a custom scheme and intercepts an HTTP request,
  plus cookie set/get/clear round-trips.

**Files / crates**

- `ffi/capi/{protocol.rs, network.rs, site_data.rs, web_resource.rs}` (new),
  `ffi/capi/lib.rs`, `ffi/capi/servo.rs`.
- `components/servo/site_data_manager.rs`, `components/servo/network_manager.rs`,
  `components/servo/webview_delegate.rs`, `components/servo/servo_delegate.rs` — plus
  `components/servo/lib.rs` (protocol exports) as needed.
- `tests/capi/c/*.c`.

**Exit criteria**

- A C host can register a custom scheme, intercept and synthesize responses (including
  streamed bodies), and inspect/clear cookies and storage.
- Async operations deliver results on the main thread during `spin_event_loop`.
- The request/response ownership rules are documented and tested.

**Risks**

- The protocol-handler Rust interface is marked unstable/unclean and may churn.
- Body streaming and response lifecycle (`finish`/`cancel`/`Drop` default) are easy to get
  wrong; mirror the existing `Drop`-default responder pattern.

**Effort.** L.

---

## Phase 4 — JavaScript binding and messaging

**Goal.** Bidirectional native <-> JS communication and script injection.

**Deliverables**

- Native <-> JS message router (currently missing): a `ScriptMessageHandler`-style API.
  This requires new engine plumbing (script-side object + embedder callbacks), not just
  FFI wrapping:
  - Rust: a new delegate/handler type plus `EmbedderMsg`/`EmbedderToConstellationMessage`
    variants to carry messages, and a script-exposed object.
  - C: `servo_webview_add_script_message_handler(webview, name, callback, user_data)`,
    `servo_webview_post_message(webview, name, json)`, and a reply path.
- Script injection APIs over `UserContentManager`
  (`components/servo/user_content_manager.rs`): add/remove user scripts and stylesheets,
  with `UserScript` construction exposed in C (`embedder_traits::user_contents`).
  *Done on this branch:* `servo_user_script_create`/`_free`,
  `servo_user_style_sheet_create`/`_free`, and
  `servo_user_content_manager_add_script`/`_remove_script` /
  `_add_style_sheet`/`_remove_style_sheet`.
- JavaScript evaluation extensions: promises/await support and richer value mapping on top
  of the Phase 1 `evaluate_javascript`.
- DevTools / remote debugging exposure:
  - `ServoDelegate::notify_devtools_server_started`, `request_devtools_connection`
    (`components/servo/servo_delegate.rs`); settings via `servo_options_set_debug_option`
    and the `devtools_server_enabled` pref. Expose server start/stop and connection
    authorization in C.
- Console message and error callbacks: `show_console_message`
  (`WebViewDelegate`/`ServoDelegate`).

**Files / crates**

- `ffi/capi/{javascript.rs, user_content.rs, devtools.rs}` (new/extended).
- `components/servo/*` and `components/shared/embedder/*` for the message-router plumbing
  (this is the first phase that requires non-trivial engine work).
- `tests/capi/c/*.c` including an async message round-trip.

**Exit criteria**

- A C host can inject a script, have it call back into the host, and receive host replies.
- DevTools server startup and connection authorization are controllable from C.
- Message routing is ordered, main-thread-delivered, and leak-free under repeated
  load/unload.

**Risks**

- Highest-risk phase: the message router touches script and constellation internals and
  must interact correctly with navigation, workers, and process boundaries.
- Security: a message router is an attack surface; define trust boundaries with the
  engine team.

**Effort.** L.

---

## Phase 5 — Stability, distribution, and ecosystem

**Goal.** Ship a dependable, versioned artifact that third parties can adopt.

**Deliverables**

- ABI versioning and enforcement (per `ABI_STABILITY.md`):
  - `SERVO_CAPI_VERSION_MAJOR/MINOR/PATCH` macros and a runtime
    `servo_capi_get_version()`.
  - `struct_size`-versioned structs and append-only enums with unknown-value handling.
  - A symbol-export allowlist / link check in CI so accidental exports or renames fail
    the build.
- Reference-counted, thread-safe handles (or explicitly documented "main-thread only"
  handles) to support hosts that manage objects across threads. Today `Servo`/`WebView`
  are `Rc`-based and thread-affine; a `Send` handle requires engine changes.
- Prebuilt binaries per platform: Linux (x86_64, aarch64), macOS (x86_64, arm64), Windows
  (x86_64), published with headers and a pkg-config/CMake package.
- Language bindings: a C++ RAII wrapper over the C API, plus community bindings
  (Python/Rust/etc.) kept out of tree if they need their own release cadence.
- Packaging: versioned tarballs/MSIs, checksums, and a documented release process.
- Documentation and samples: a complete C sample app, an integration guide, and a
  compatibility matrix (which functions are present on which platforms/features).
- Observability/perf: track cdylib size, startup time, and basic rendering throughput in CI
  to catch regressions.

**Files / crates**

- `ffi/capi/*` (versioning, exports), a new `ffi/capi/include`/packaging metadata,
  `.github/workflows/embedding-api.yml` and release workflows.
- Possibly `components/servo` for `Send` handles (large change; evaluate first).
- New `examples/`/samples directory and binding repos.

**Exit criteria**

- A released header + binary pair works with a sample app on each supported platform.
- ABI compatibility rules are enforced in CI; a deliberate breaking change fails until the
  major version is bumped.
- Deprecation policy is in effect and at least one deprecation has been carried through a
  release.

**Risks**

- Distribution and code signing (especially macOS/Windows) are operationally heavy.
- `Send` handles may not be feasible without reworking `Rc` usage; if so, document the
  constraint instead of over-promising.
- Keeping bindings in sync with a moving ABI.

**Effort.** L.

---

## Cross-cutting rules for every phase

1. **Panic-free boundary.** New functions return error codes / nulls and never let a panic
   cross the FFI. See `ABI_STABILITY.md` for the exact template.
2. **Ownership and threading are documented per function** using the contract templates in
   `ABI_STABILITY.md`.
3. **Every new function has a C test** in `tests/capi` and, where relevant, an integration
   test that exercises it through `spin_event_loop`.
4. **Delegates and async operations deliver on the main thread** during
   `servo_spin_event_loop`, matching the engine's existing behavior.
5. **The header is generated by cbindgen**; hand-written ABI structs get a `struct_size`
   field and an accompanying doc block.
6. **Honest platform support**: document which functions work with which rendering
   contexts and features instead of silently no-op'ing.
