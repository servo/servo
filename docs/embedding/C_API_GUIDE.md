# Servo C API Guide

This guide is for C and C++ embedders using the `servo-capi` shared library. It
covers the runtime model, the ownership rules, the complete API surface, and a
single-file C11 example you can adapt.

The authoritative definition of the API is the generated header
`servo/servo_capi.h`. The function names and semantics below match that header.
Throughout this document, "the change set" refers to the release that introduced
the functions marked **(new)**; see [New in this release](#new-in-this-release).

- [Runtime model](#runtime-model)
- [Building and linking](#building-and-linking)
- [API at a glance](#api-at-a-glance)
- [New in this release](#new-in-this-release)
- [A complete C11 example](#a-complete-c11-example)
- [The buffer-query pattern](#the-buffer-query-pattern)
- [Evaluating JavaScript](#evaluating-javascript)
- [Taking screenshots](#taking-screenshots)
- [Threading rule](#threading-rule)
- [Ownership and free rules](#ownership-and-free-rules)
- [Error handling and panic policy](#error-handling-and-panic-policy)
- [Common pitfalls](#common-pitfalls)

## Runtime model

Servo is **single-threaded from the embedder's point of view**. Everything you do
with the C API happens on one thread — the *embedder thread*:

1. Create options and preferences.
2. Create a builder, install a `ServoEventLoopWaker`, build the `Servo`.
3. Create a rendering context and build `WebView`s.
4. Repeatedly call `servo_spin_event_loop(servo)` to let Servo make progress.

Servo does its work on internal threads, but it never calls back into your code
except through the waker and the `ServoWebViewDelegate` callbacks. The waker may
be invoked from **any** Servo thread; the delegate callbacks are invoked on the
embedder thread while it is inside `servo_spin_event_loop`.

Nothing happens on the WebView until you spin the event loop. Calls such as
`servo_webview_load`, `servo_webview_evaluate_javascript` and
`servo_webview_take_screenshot` only enqueue work; the results arrive during a
later `servo_spin_event_loop` call.

## Building and linking

`servo-capi` is a `cdylib`; there is no static library. Build and install it with
[cargo-c](https://github.com/rust-lang/cargo-c), which also runs `cbindgen` to
generate the header:

```sh
cargo install cargo-c            # CI pins cargo-c@0.10.23
cargo cinstall -p servo-capi \
    --library-type cdylib \
    --prefix /path/to/servo-sysroot
```

This installs:

- the header at `<prefix>/include/servo/servo_capi.h`
  (the `servo` subdirectory comes from `package.metadata.capi.header` in
  `ffi/capi/Cargo.toml`);
- the shared library under `<prefix>/lib`:
  `libservo_capi.so` on Linux, `libservo_capi.dylib` on macOS, and
  `servo_capi.dll` plus an import library on Windows;
- a pkg-config file (when supported by the installed cargo-c) so you can use
  `pkg-config --cflags --libs servo_capi`.

Compile and link your embedder against it:

```sh
cc -std=c11 -Wall -Wextra my_embedder.c \
   -I/path/to/servo-sysroot/include \
   -L/path/to/servo-sysroot/lib \
   -lservo_capi -o my_embedder
```

Include the header with its install path:

```c
#include <servo/servo_capi.h>
```

At runtime the loader must find the shared library. On Linux set
`LD_LIBRARY_PATH`, on macOS set `DYLD_LIBRARY_PATH`, and on Windows copy
`servo_capi.dll` next to your executable (the C tests' `build.rs` does exactly
this).

### Using the API from C++

The generated header is C++-safe: `cbindgen.toml` enables `cpp_compat`, so its
declarations are wrapped in `#ifdef __cplusplus extern "C" { ... }` and the C-only
enum shims are omitted. Include it from C++ like any other header; the calls
resolve to the same unmangled C symbols as from C:

```cpp
#include <servo/servo_capi.h>

uint32_t version = servo_capi_get_version();
```

For convenience, the source tree also ships a header-only RAII wrapper at
`ffi/capi/servo_capi.hpp` (C++17). It turns every owned handle into a move-only
`std::unique_ptr` that calls the matching `servo_*_free` on destruction, so a
handle cannot be leaked or freed twice. Pass the raw pointer with `.get()`; call
`.release()` when a function transfers ownership to Servo:

```cpp
#include <servo/servo_capi.h>
#include "servo_capi.hpp"

servo::ServoOptions options(servo_options_create());
servo::ServoBuilder builder(servo_builder_create());
if (builder && options) {
    // `servo_builder_set_options` takes ownership of `options`.
    servo_builder_set_options(builder.get(), options.release());
}
```

Compile with `-I<pfx>/include` (for `servo/servo_capi.h`) and add
`ffi/capi` to the include path (for `servo_capi.hpp`). The wrapper is compiled
and exercised on Linux, macOS, and Windows by the C++ test in
`tests/capi/cpp/test_cpp_api.cpp`; it is not yet installed by `cargo cinstall`.

**Where CI builds and tests the artifact.** The C API is built as part of the
`servo-capi-tests` crate. `tests/capi/build.rs` runs
`cargo cinstall -p servo-capi --library-type cdylib --destdir $OUT_DIR --prefix /`
and then links the C test programs against `-lservo_capi`. The dedicated
workflow `.github/workflows/embedding-api.yml` runs this on Linux, macOS and
Windows: it installs `cargo-c@0.10.23` and `cbindgen`, runs
`cargo test -p servo-capi-tests --locked`, then regenerates the header with
`cbindgen --config ffi/capi/cbindgen.toml` to confirm it still builds. The same
test is also run by `.github/workflows/linux-common.yml`,
`.github/workflows/mac.yml`, `.github/workflows/mac-arm64.yml` and
`.github/workflows/windows.yml`.

## API at a glance

All handles are opaque pointers. Unless stated otherwise, a function that takes
a plain handle pointer does **not** take ownership of it.

### Version

```c
/* (new) Static string "major.minor.patch"; never free or modify. */
const char* servo_capi_version(void);

/* (new) Packed integer (MAJOR << 16) | (MINOR << 8) | PATCH. */
uint32_t servo_capi_get_version(void);
```

The generated header also defines `SERVO_CAPI_VERSION_MAJOR`,
`SERVO_CAPI_VERSION_MINOR`, `SERVO_CAPI_VERSION_PATCH`, and the packed
`SERVO_CAPI_VERSION`. Compare `servo_capi_get_version()` against
`SERVO_CAPI_VERSION` at startup to detect a header/library version skew.

### Builder and waker

```c
ServoBuilder* servo_builder_create(void);
void servo_builder_set_preferences(ServoBuilder*, ServoPreferences*); /* takes ownership */
void servo_builder_set_options(ServoBuilder*, ServoOptions*);         /* takes ownership */
void servo_builder_set_event_loop_waker(ServoBuilder*, struct ServoEventLoopWaker);
void servo_builder_set_delegate(ServoBuilder*, struct ServoDelegate); /* takes ownership */
int servo_builder_register_protocol_handler(ServoBuilder*, const char* scheme,
                                            bool (*callback)(const struct ServoWebResourceRequest*,
                                                             struct ServoWebResourceResponse*,
                                                             void* user_data),
                                            void* user_data);                                        /* (new) */
struct Servo* servo_builder_build(ServoBuilder*);                     /* consumes builder */
void servo_builder_free(ServoBuilder*);

struct ServoEventLoopWaker {
    uint32_t struct_size;   /* set to sizeof(struct ServoEventLoopWaker) */
    uint32_t _reserved;     /* 0 */
    void (*wake_callback)(void);
};

struct ServoDelegate {
    uint32_t struct_size;   /* set to sizeof(struct ServoDelegate) */
    uint32_t _reserved;     /* 0 */
    void* user_data;
    void (*notify_devtools_server_started)(struct Servo*, uint16_t port, const char* token, void* user_data);
    bool (*request_devtools_connection)(struct Servo*, void* user_data);
};
```

> **Note on App-Level Interception:** In the internal Rust engine, `ServoDelegate::load_web_resource` allows intercepting requests before a `WebView` exists. App-level interception is not exposed in the C API; request interception in C is configured per-`WebView` through `ServoWebViewDelegate::intercept_request` or `ServoWebViewDelegate::intercept_request_streaming`.

### Custom protocol handlers (new)

Custom schemes (e.g., `myapp://...`) can be registered at build time via
`servo_builder_register_protocol_handler`. The callback receives a request
object and populates a response object. Return `true` to serve the response,
`false` to fall back to normal network loading.

```c
typedef struct ServoProtocolHandler ServoProtocolHandler;
typedef struct ServoWebResourceRequest ServoWebResourceRequest;
typedef struct ServoWebResourceResponse ServoWebResourceResponse;

bool (*ServoProtocolHandlerCallback)(const ServoWebResourceRequest* request,
                                     ServoWebResourceResponse* response,
                                     void* user_data);

int servo_builder_register_protocol_handler(ServoBuilder* builder,
                                            const char* scheme,
                                            ServoProtocolHandlerCallback callback,
                                            void* user_data); /* returns ServoError */
```

The request object provides getters for method, URL, headers, and body:

```c
const char* servo_web_resource_request_get_method(const ServoWebResourceRequest* request);
uintptr_t servo_web_resource_request_get_url(const ServoWebResourceRequest* request,
                                             char* buffer, uintptr_t buffer_len);
uintptr_t servo_web_resource_request_get_header(const ServoWebResourceRequest* request,
                                                const char* name, char* buffer, uintptr_t buffer_len);
uintptr_t servo_web_resource_request_get_body(const ServoWebResourceRequest* request,
                                              uint8_t* buffer, uintptr_t buffer_len);
```

The response object provides setters for URL, status, headers, and body:

```c
int servo_web_resource_response_set_url(ServoWebResourceResponse* response, const char* url);
int servo_web_resource_response_set_status(ServoWebResourceResponse* response,
                                           uint16_t status_code, const char* status_message);
int servo_web_resource_response_set_header(ServoWebResourceResponse* response,
                                           const char* name, const char* value);
int servo_web_resource_response_set_body(ServoWebResourceResponse* response,
                                         const uint8_t* data, uintptr_t data_len);
```

### Web resource interception and streaming (new)

Servo supports intercepting network requests per-`WebView` via `ServoWebViewDelegate`. Embedders can either:
1. Provide a whole-body response synchronously via `intercept_request`.
2. Synthesise a streaming response in chunks via `intercept_request_streaming` and `ServoWebResourceLoad`.

#### Streaming functions

```c
/* Opaque handle for an in-flight resource load. */
typedef struct ServoWebResourceLoad ServoWebResourceLoad;

/* Get the request details for this load. */
const ServoWebResourceRequest* servo_web_resource_load_get_request(
    const ServoWebResourceLoad* load);

/* Begin intercepting the load with the given response status and headers. */
int servo_web_resource_load_intercept(ServoWebResourceLoad* load,
                                      const ServoWebResourceResponse* response);

/* Send a chunk of body data to the client. Can be called multiple times. */
int servo_web_resource_load_send_body_data(ServoWebResourceLoad* load,
                                           const uint8_t* data,
                                           uintptr_t data_len);

/* Signal completion of the response. Consumes and frees `load`. */
int servo_web_resource_load_finish(ServoWebResourceLoad* load);

/* Cancel the load with a network error. Consumes and frees `load`. */
int servo_web_resource_load_cancel(ServoWebResourceLoad* load);

/* Free the handle without calling finish or cancel. Auto-finishes if intercepted. */
int servo_web_resource_load_free(ServoWebResourceLoad* load);
```

#### Ownership and lifecycle rules

- **`ServoWebResourceRequest*`**:
  - **Ownership:** Owned entirely by Servo. The embedder must **not** free this pointer.
  - **Validity:** Valid only for the duration of the callback invocation (or for the lifetime of `ServoWebResourceLoad` if accessed via `servo_web_resource_load_get_request`).
  - **Data Retention:** The embedder must copy strings or body buffers (using the buffer-query getters `servo_web_resource_request_get_*`) if needed beyond the callback scope.
  - **Thread Safety:** May be called from network or background threads during custom scheme loading; callbacks must not access thread-local data assuming the main thread.

- **`ServoWebResourceResponse*`**:
  - **Ownership:** Created and owned by Servo. The embedder must **not** free this pointer.
  - **Validity:** Valid only for the duration of the callback.
  - **Usage:** In `intercept_request`, the embedder sets the status code, response headers, and full body. In `intercept_request_streaming`, the embedder sets the status code and headers before passing the pointer to `servo_web_resource_load_intercept(load, response)`.

- **Callback return value semantics (`bool`)**:
  - **`true`:** The embedder intercepts the request. In `intercept_request`, Servo synthesises the response using the status, headers, and body populated on `ServoWebResourceResponse`. In `intercept_request_streaming`, the embedder retains or controls `ServoWebResourceLoad` to stream chunks.
  - **`false`:** The embedder declines to intercept. Any modifications to `response` are ignored, the `load` handle is automatically freed by Servo, and normal network loading proceeds.

- **Streaming load lifecycle (`ServoWebResourceLoad*`)**:
  - Delivered to `intercept_request_streaming`.
  - To stream:
    1. Populate status code and headers on `response`.
    2. Call `servo_web_resource_load_intercept(load, response)`.
    3. Call `servo_web_resource_load_send_body_data(load, bytes, len)` one or more times.
    4. Call `servo_web_resource_load_finish(load)` upon completion.
  - **Consumption & Freeing:** `servo_web_resource_load_finish` and `servo_web_resource_load_cancel` **consume and free** the `load` handle. The caller must **not** call `servo_web_resource_load_free` or access the pointer after finish or cancel has been called.
  - **Fallback `free`:** If the embedder creates or retains a `load` handle but drops it without calling `finish` or `cancel`, calling `servo_web_resource_load_free(load)` mirrors Rust's `Drop` semantics: if the load was intercepted, it auto-finishes; if it was not intercepted, the underlying network load is dropped/canceled.

### Servo

```c
ServoError servo_free(struct Servo*);
ServoError servo_spin_event_loop(struct Servo*);
ServoError servo_setup_logging(struct Servo*);
```

`servo_setup_logging` installs the process-global logger. Because logging is
global, only the first call takes effect; a later call returns
`SERVO_ERROR_INVALID_STATE` (it does not panic). `servo_free` and
`servo_spin_event_loop` return `ServoError` and follow the usual null/threading
rules.

### Options

```c
ServoOptions* servo_options_create(void);
void servo_options_free(ServoOptions*);

void servo_options_set_hard_fail(ServoOptions*, bool);
void servo_options_set_multiprocess(ServoOptions*, bool);
void servo_options_set_force_ipc(ServoOptions*, bool);
void servo_options_set_background_hang_monitor(ServoOptions*, bool);
void servo_options_set_hang_monitor_kill_script(ServoOptions*, bool);
void servo_options_set_sandbox(ServoOptions*, bool);
void servo_options_set_temporary_storage(ServoOptions*, bool);
void servo_options_set_ignore_certificate_errors(ServoOptions*, bool);
void servo_options_set_unminify_js(ServoOptions*, bool);
void servo_options_set_unminify_css(ServoOptions*, bool);
void servo_options_set_config_dir(ServoOptions*, const char*);
void servo_options_set_certificate_path(ServoOptions*, const char*);
void servo_options_set_local_script_source(ServoOptions*, const char*);
void servo_options_set_shaders_path(ServoOptions*, const char*);
void servo_options_set_random_pipeline_closure_probability(ServoOptions*, float);
void servo_options_set_random_pipeline_closure_seed(ServoOptions*, uintptr_t);
void servo_options_set_debug_option(ServoOptions*, int option, bool enabled);
void servo_options_enable_time_profiling_to_stdout(ServoOptions*, double interval_seconds);
void servo_options_enable_time_profiling_to_file(ServoOptions*, const char* file_path);
void servo_options_set_time_profiler_trace_path(ServoOptions*, const char*);
```

`servo_options_set_debug_option` takes one of the
`SERVO_DIAGNOSTICS_LOGGING_OPTION_*` constants.

### Preferences

Preferences are addressed by name and typed accessors. Any Servo preference can
be reached this way. The user agent is set through the `user_agent` string
preference.

```c
ServoPreferences* servo_preferences_create(void);
void servo_preferences_free(ServoPreferences*);

void  servo_preferences_set_bool(ServoPreferences*, const char* name, bool value);
bool  servo_preferences_get_bool(const ServoPreferences*, const char* name);
void  servo_preferences_set_i64(ServoPreferences*, const char* name, int64_t value);
int64_t servo_preferences_get_i64(const ServoPreferences*, const char* name);
void  servo_preferences_set_u64(ServoPreferences*, const char* name, uint64_t value);
uint64_t servo_preferences_get_u64(const ServoPreferences*, const char* name);
void  servo_preferences_set_string(ServoPreferences*, const char* name, const char* value);
char* servo_preferences_get_string(const ServoPreferences*, const char* name);
```

`servo_preferences_get_string` returns a newly allocated NUL-terminated string
that you must release with the C library `free()`. The `set_*` functions assert
that the preference exists and has the expected type, so use valid names.

For preferences that should change **after** `Servo` has been built, use the
runtime setters `servo_set_preference_bool` / `_i64` / `_u64` / `_string`
**(new)** — see [Runtime preference setters](#runtime-preference-setters).

### Rendering context

```c
RenderingContext* servo_rendering_context_create_software(uint32_t width, uint32_t height);
RenderingContext* servo_rendering_context_create_window(void* display_handle, void* window_handle,
                                                        uint32_t width, uint32_t height);
RenderingContext* servo_rendering_context_create_offscreen(RenderingContext* parent,
                                                           uint32_t width, uint32_t height);
void servo_rendering_context_free(RenderingContext*);
```

`servo_rendering_context_create_software` returns `NULL` on failure (including a
zero-sized request).

`servo_rendering_context_create_window` creates a GPU-accelerated rendering context
targeting a native window. The `display_handle` and `window_handle` are opaque
pointers to `raw_window_handle::RawDisplayHandle` and
`raw_window_handle::RawWindowHandle` respectively, as provided by windowing
libraries like winit, SDL, or GLFW. Returns `NULL` on failure (invalid handles,
zero size, or GPU context creation failure).

`servo_rendering_context_create_offscreen` creates a GPU-accelerated offscreen
rendering context (OSR / FBO composition) that shares the parent window context's
OpenGL context. The `parent` must be a `RenderingContext` created by
`servo_rendering_context_create_window`. The offscreen context renders to an FBO
that can be composited into the parent window. Returns `NULL` on failure (null
parent, non-window parent, zero size, or FBO creation failure).

### WebView builder, WebView and delegate

```c
ServoWebViewBuilder* servo_webview_builder_create(struct Servo*, RenderingContext*); /* takes ownership of context */
int  servo_webview_builder_set_url(ServoWebViewBuilder*, const char* url); /* 0 ok, -1 bad URL */
void servo_webview_builder_set_delegate(ServoWebViewBuilder*, ServoWebViewDelegate);
void servo_webview_builder_set_clipboard_delegate(ServoWebViewBuilder*, ServoClipboardDelegate); /* (new) */
struct WebView* servo_webview_builder_build(ServoWebViewBuilder*); /* consumes builder */
void servo_webview_builder_free(ServoWebViewBuilder*);

void servo_webview_paint(struct WebView*);
int  servo_webview_load(struct WebView*, const char* url); /* 0 ok, -1 bad URL */
void servo_webview_take_screenshot(struct WebView*, void (*callback)(const uint8_t* data,
                                      uint32_t width, uint32_t height, int32_t error,
                                      void* user_data), void* user_data);
void servo_webview_close(struct WebView*);                                              /* (new) */
void servo_webview_free(struct WebView*);

struct ServoWebViewDelegate {
    uint32_t struct_size;   /* set to sizeof(ServoWebViewDelegate) */
    uint32_t _reserved;     /* 0 */
    void* user_data;
    void (*notify_load_status_changed)(struct WebView*, int32_t load_status, void* user_data);
    void (*notify_url_changed)(struct WebView*, const char* url, void* user_data);            /* (new) */
    void (*notify_page_title_changed)(struct WebView*, const char* title, void* user_data);   /* (new) */
    void (*notify_new_frame_ready)(struct WebView*, void* user_data);
    void (*notify_closed)(struct WebView*, void* user_data);                                  /* (new) */

    // Focus category (new)
    void (*notify_focus_changed)(struct WebView*, bool focused, void* user_data);
    void (*notify_got_focus)(struct WebView*, void* user_data);

    // Load category (new)
    void (*notify_load_started)(struct WebView*, void* user_data);
    void (*notify_load_ended)(struct WebView*, void* user_data);
    void (*notify_load_error)(struct WebView*, int32_t error_code, const char* error_text, void* user_data);

    // Display category (new)
    void (*notify_cursor_changed)(struct WebView*, int32_t cursor_type, void* user_data);
    void (*notify_status_text_changed)(struct WebView*, const char* text, void* user_data);
    void (*notify_favicon_changed)(struct WebView*, const char* url, void* user_data);
    void (*show_console_message)(struct WebView*, int32_t level, const char* message, void* user_data);

    // Resource interception category (new)
    bool (*intercept_request)(struct WebView*,
                              const struct ServoWebResourceRequest*,
                              struct ServoWebResourceResponse*,
                              void* user_data);

    // Download category (new)
    void (*on_before_download)(struct WebView*, struct ServoDownloadItem*,
                               void (*continue_callback)(bool, void*), void* user_data);
    void (*on_download_updated)(struct WebView*, struct ServoDownloadItem*, void* user_data);

    // Find category (new)
    void (*on_find_result)(struct WebView*, int32_t identifier, int32_t count,
                           const struct ServoFindResult* result, void* user_data);

    // Script message category (new)
    void (*on_script_message)(struct WebView* webview,
                              const char* handler_name,
                              const char* message_json,
                              void* user_data);

    // Embedder controls category (new) - alert/confirm/prompt, select, color,
    // file, context menu and IME. Abbreviated here; see "Embedder controls"
    // below for the full list and the answer protocol.
};

/* Clipboard delegate (new). Set with
 * servo_webview_builder_set_clipboard_delegate. */
struct ServoClipboardDelegate {
    uint32_t struct_size;   /* set to sizeof(ServoClipboardDelegate) */
    uint32_t _reserved;     /* 0 */
    void* user_data;
    void (*clear)(struct WebView*, void* user_data);
    void (*get_text)(struct WebView*, struct ServoClipboardStringRequest* request, void* user_data);
    void (*set_text)(struct WebView*, const char* text, void* user_data);
};

/* Opaque handle passed to `get_text`. Exactly one of these resolves it and
 * takes ownership of the handle. */
ServoError servo_webview_clipboard_request_success(struct ServoClipboardStringRequest*, const char* text);
ServoError servo_webview_clipboard_request_failure(struct ServoClipboardStringRequest*, const char* message);
```

Any delegate field may be `NULL`. Function pointers are copied by value and must
stay valid for the lifetime of the `WebView`.

## New in this release

The following functions and delegate fields are **new** in the release that added
them. They are part of the current API, but older embedders written against a
previous header will not have declarations for them.

### Navigation and state getters

```c
/* Buffer-query getters. Return the number of bytes required, including the
 * terminating NUL. If `buffer` is NULL or `buffer_len` is too small, nothing is
 * written and the required size is returned. */
uintptr_t servo_webview_get_url(const struct WebView* webview, char* buffer, uintptr_t buffer_len);
uintptr_t servo_webview_get_title(const struct WebView* webview, char* buffer, uintptr_t buffer_len);
uintptr_t servo_webview_get_status_text(const struct WebView* webview, char* buffer, uintptr_t buffer_len);

/* One of the SERVO_LOAD_STATUS_* constants. */
int servo_webview_get_load_status(const struct WebView* webview);

/* Viewport size in device pixels. Either out pointer may be NULL. */
void servo_webview_get_size(const struct WebView* webview, uint32_t* out_width, uint32_t* out_height);

bool servo_webview_can_go_back(const struct WebView* webview);
bool servo_webview_can_go_forward(const struct WebView* webview);
void servo_webview_go_back(struct WebView* webview, uintptr_t amount);
void servo_webview_go_forward(struct WebView* webview, uintptr_t amount);
void servo_webview_reload(struct WebView* webview);
void servo_webview_clear_session_history(struct WebView* webview);

void  servo_webview_set_page_zoom(struct WebView* webview, float zoom); /* clamped to [0.1, 10.0] */
float servo_webview_page_zoom(const struct WebView* webview);
void  servo_webview_resize(struct WebView* webview, uint32_t width, uint32_t height);
void  servo_webview_set_focused(struct WebView* webview, bool focused);
bool  servo_webview_focused(const struct WebView* webview);
void  servo_webview_show(struct WebView* webview);
void  servo_webview_hide(struct WebView* webview);
void  servo_webview_notify_theme_change(struct WebView* webview, int theme); /* SERVO_THEME_* */
void  servo_webview_close(struct WebView*);                                                  /* (new) */

const char* servo_capi_version(void);
```

### Download management (new)

```c
/* Download states matching CEF */
#define SERVO_DOWNLOAD_IN_PROGRESS   0
#define SERVO_DOWNLOAD_COMPLETE      1
#define SERVO_DOWNLOAD_CANCELLED     2
#define SERVO_DOWNLOAD_INTERRUPTED   3

/* Opaque download item handle */
typedef struct ServoDownloadItem ServoDownloadItem;

/* Download item query functions (buffer-query pattern for strings) */
uintptr_t servo_download_item_get_url(const ServoDownloadItem* download,
                                      char* buffer, uintptr_t buffer_len);
uintptr_t servo_download_item_get_suggested_filename(const ServoDownloadItem* download,
                                                     char* buffer, uintptr_t buffer_len);
uint64_t servo_download_item_get_received_bytes(const ServoDownloadItem* download);
uint64_t servo_download_item_get_total_bytes(const ServoDownloadItem* download);
int servo_download_item_get_state(const ServoDownloadItem* download); /* SERVO_DOWNLOAD_* */

/* Download control functions */
int servo_download_item_cancel(ServoDownloadItem* download);  /* returns ServoError */
int servo_download_item_pause(ServoDownloadItem* download);   /* returns ServoError */
int servo_download_item_resume(ServoDownloadItem* download);  /* returns ServoError */
```

### Find-in-page (new)

```c
/* Opaque find result handle */
typedef struct ServoFindResult ServoFindResult;

/* Find result query functions */
int32_t servo_find_result_get_identifier(const ServoFindResult* result);
int32_t servo_find_result_get_active_match_ordinal(const ServoFindResult* result);
int32_t servo_find_result_get_number_of_matches(const ServoFindResult* result);
void servo_find_result_get_selection_rect(const ServoFindResult* result,
                                          float* x, float* y,
                                          float* width, float* height);

/* Find-in-page control functions */
/* Returns a positive identifier on success, or a negative ServoError code on failure */
int32_t servo_webview_find(struct WebView* webview,
                           const char* text,
                           bool forward,
                           bool match_case);
int32_t servo_webview_find_next(struct WebView* webview,
                                int32_t identifier,
                                bool forward);  /* returns ServoError */
int32_t servo_webview_find_clear(struct WebView* webview,
                                 int32_t identifier);  /* returns ServoError */

/* Find delegate callback */
typedef void (*ServoFindResultCallback)(struct WebView* webview,
                                        int32_t identifier,
                                        int32_t count,
                                        const ServoFindResult* result,
                                        void* user_data);
```

### Input event injection (new)

```c
/* Key event types: SERVO_KEY_EVENT_DOWN, SERVO_KEY_EVENT_UP */
/* Modifiers: SERVO_MODIFIER_* bitflags (ALT, CONTROL, SHIFT, META, etc.) */
/* Key locations: 0=Standard, 1=Left, 2=Right, 3=Numpad */
uint64_t servo_webview_notify_key_event(struct WebView* webview,
                                        int32_t event_type,
                                        const char* key,
                                        const char* code,
                                        int32_t location,
                                        uint32_t modifiers,
                                        bool repeat,
                                        bool is_composing);

/* Mouse button event types: SERVO_MOUSE_EVENT_DOWN, SERVO_MOUSE_EVENT_UP */
/* Mouse buttons: SERVO_MOUSE_BUTTON_* constants */
uint64_t servo_webview_notify_mouse_button_event(struct WebView* webview,
                                                 int32_t event_type,
                                                 int32_t button,
                                                 double x,
                                                 double y,
                                                 uint32_t modifiers);

/* Mouse move event */
uint64_t servo_webview_notify_mouse_move_event(struct WebView* webview,
                                               double x,
                                               double y,
                                               uint32_t modifiers);

/* Mouse leave event */
uint64_t servo_webview_notify_mouse_leave_event(struct WebView* webview,
                                                bool focus_moving_to_another_iframe);

/* Touch event types: SERVO_TOUCH_EVENT_DOWN, SERVO_TOUCH_EVENT_MOVE, SERVO_TOUCH_EVENT_UP, SERVO_TOUCH_EVENT_CANCEL */
/* Touch pointer types: SERVO_TOUCH_POINTER_TYPE_TOUCH, SERVO_TOUCH_POINTER_TYPE_PEN */
uint64_t servo_webview_notify_touch_event(struct WebView* webview,
                                          int32_t event_type,
                                          int32_t touch_id,
                                          double x,
                                          double y,
                                          int32_t pointer_type);

/* Wheel event */
/* Delta modes: SERVO_WHEEL_DELTA_PIXEL, SERVO_WHEEL_DELTA_LINE, SERVO_WHEEL_DELTA_PAGE */
uint64_t servo_webview_notify_wheel_event(struct WebView* webview,
                                          double delta_x,
                                          double delta_y,
                                          double delta_z,
                                          int32_t delta_mode,
                                          double x,
                                          double y);

/* Called when an input event has been handled by Servo.
 * The embedder should call this in response to the notify_input_event_handled
 * delegate callback (not yet exposed in this release). */
void servo_webview_notify_input_event_handled(struct WebView* webview, uint64_t event_id);
```
```

### Runtime preference setters

In addition to the `servo_preferences_*` accessors (which operate on a
`ServoPreferences` object before `Servo` is built), a live `Servo` can have
preferences changed after `servo_builder_build`:

```c
void servo_set_preference_bool(struct Servo* servo, const char* name, bool value);
void servo_set_preference_i64(struct Servo* servo, const char* name, int64_t value);
void servo_set_preference_u64(struct Servo* servo, const char* name, uint64_t value);
void servo_set_preference_string(struct Servo* servo, const char* name, const char* value);
```

If the preference does not exist or has a different type, Servo logs an error
and leaves it unchanged rather than aborting. These do not take ownership of the
`Servo` handle.

### JavaScript evaluation

```c
/* Deprecated: prefer servo_webview_evaluate_javascript_v2, which returns a
 * ServoValue instead of a serialized JSON string. The symbol remains available
 * and functional for at least one minor version (see ABI_STABILITY.md §6). */
typedef void (*ServoJavaScriptCallback)(const char* result_json,
                                        const char* error_message,
                                        void* user_data); /* (new) */

void servo_webview_evaluate_javascript(struct WebView* webview,
                                       const char* script,
                                       ServoJavaScriptCallback callback,
                                       void* user_data); /* (new, deprecated) */
```

### Value containers (new)

`servo_webview_evaluate_javascript_v2` hands back a `ServoValue`, and the same
containers are usable for anything else that crosses the boundary. A `ServoValue` is
one of null, bool, int (`int64_t`), double, string, binary, dictionary or list;
`servo_value_get_type` reports which, and the `SERVO_VALUE_TYPE_*` constants name the
answers (the Rust enum itself is not emitted into the header, so compare against the
constants).

```c
/* Build { "count": 7 } from scratch. */
ServoDictionaryValue* dict = servo_dictionary_create();
ServoValue* count = servo_value_create();
servo_value_set_int(count, 7);
servo_dictionary_set_key(dict, "count", count);  /* takes ownership of `count` */

ServoValue* value = servo_value_create();
servo_value_set_dictionary(value, dict);         /* takes ownership of `dict` */

/* Read it back: `_get_dictionary` returns a *copy*, which you free like any
 * other dictionary handle. */
int32_t type = 0;
servo_value_get_type(value, &type);              /* == SERVO_VALUE_TYPE_DICTIONARY */
ServoDictionaryValue* copy = NULL;
servo_value_get_dictionary(value, &copy);
ServoValue* read = NULL;
if (servo_dictionary_get_key(copy, "count", &read) == SERVO_ERROR_OK) {
    int64_t n = 0;
    servo_value_get_int(read, &n);               /* n == 7 */
    servo_value_free(read);
}
servo_dictionary_free(copy);
servo_value_free(value);
```

Rules:

- **Ownership moves on set.** `servo_dictionary_set_key`, `servo_list_set_value`,
  `servo_value_set_dictionary` and `servo_value_set_list` consume the value/container
  handle they are given; do not free it afterwards. They check for `NULL` arguments
  *before* taking ownership, so a rejected call leaves you holding the handle.
- **Getters copy.** `servo_dictionary_get_key`, `servo_list_get_value`,
  `servo_value_get_dictionary` and `servo_value_get_list` allocate a fresh handle that
  you own. Editing the copy does not write back into the original. This is the one
  place the C API deliberately differs from CEF, whose containers are refcounted.
- **A wrong type is `SERVO_ERROR_INVALID_STATE`** - the same answer the typed
  `servo_value_get_*` accessors give for a mismatch - not `SERVO_ERROR_NULL_ARGUMENT`.
- `servo_value_from_json` / `servo_value_to_json` convert to and from a JSON string,
  which is still the simplest way to move a whole tree across.

### Message router (native↔JS messaging)

```c
/* Message callback for JS→native messages. */
typedef void (*ServoMessageCallback)(const char* name,
                                     const char* data_json,
                                     void* user_data); /* (new) */

/* Callback for script messages providing webview handle. */
typedef void (*ServoScriptMessageCallback)(struct WebView* webview,
                                           const char* handler_name,
                                           const char* message_json,
                                           void* user_data); /* (new) */

/* Register a handler for messages from JavaScript. */
void servo_webview_add_message_handler(struct WebView* webview,
                                       const char* name,
                                       ServoMessageCallback callback,
                                       void* user_data); /* (new) */

/* Register a script message handler with WebView context. */
ServoError servo_webview_add_script_message_handler(struct WebView* webview,
                                                    const char* handler_name,
                                                    ServoScriptMessageCallback callback,
                                                    void* user_data); /* (new) */

/* Remove all handlers for a message name. */
void servo_webview_remove_message_handler(struct WebView* webview,
                                          const char* name); /* (new) */

/* Remove a script message handler. */
ServoError servo_webview_remove_script_message_handler(struct WebView* webview,
                                                       const char* handler_name); /* (new) */

/* Send a message from native to JavaScript. */
void servo_webview_post_message(struct WebView* webview,
                                const char* name,
                                const char* data_json); /* (new) */

/* Poll for messages from JavaScript. Must be called periodically. */
void servo_webview_poll_js_messages(struct WebView* webview); /* (new) */

/* Opaque handle for the user content manager. */
struct ServoUserContentManager;

/* Create a user content manager. */
ServoUserContentManager* servo_user_content_manager_create(void); /* (new) */

/* Free a user content manager. */
void servo_user_content_manager_free(ServoUserContentManager*); /* (new) */

/* Enable the message router on a user content manager. */
void servo_user_content_manager_enable_message_router(ServoUserContentManager*); /* (new) */

/* Set the user content manager on a WebView builder. */
void servo_webview_builder_set_user_content_manager(ServoWebViewBuilder*,
                                                    ServoUserContentManager*); /* (new) */
```

### Site data (cookies)

```c
/* Opaque handle for the site data manager. Obtained from a Servo instance. */
struct ServoSiteDataManager;

/* Cookie struct for C ABI. String pointers are owned by Servo and valid
 * until the next servo_site_data_* call. */
struct ServoCookie {
    const char* name;
    const char* value;
    const char* domain;
    const char* path;
    int64_t expires;      /* Unix timestamp, 0 = session */
    bool secure;
    bool http_only;
    int same_site;        /* SERVO_COOKIE_SAME_SITE_* */
};

/* SameSite constants. */
#define SERVO_COOKIE_SAME_SITE_LAX     0
#define SERVO_COOKIE_SAME_SITE_STRICT  1
#define SERVO_COOKIE_SAME_SITE_NONE    2

/* Get the site data manager for a Servo instance. The returned handle must be
 * freed with servo_site_data_manager_free. */
struct ServoSiteDataManager* servo_get_site_data_manager(struct Servo* servo);
void servo_site_data_manager_free(struct ServoSiteDataManager* manager);

/* Get cookies for a URL (buffer-query pattern for array).
 * Returns the number of cookies. Call with buffer=NULL to get required count. */
size_t servo_site_data_cookies_for_url(struct ServoSiteDataManager* manager,
                                       const char* url,
                                       struct ServoCookie* buffer,
                                       size_t buffer_len);

/* Set a cookie for a URL. Returns 0 on success, -1 on error. */
int servo_site_data_set_cookie(struct ServoSiteDataManager* manager,
                               const char* url,
                               const struct ServoCookie* cookie);

/* Clear all cookies (persistent + session). */
void servo_site_data_clear_cookies(struct ServoSiteDataManager* manager);

/* Clear only session cookies (those with no expiry). */
void servo_site_data_clear_session_cookies(struct ServoSiteDataManager* manager);

/* Storage types for site data enumeration and clearing (bitflags). */
enum ServoStorageType {
    SERVO_STORAGE_TYPE_COOKIES = (1 << 0),
    SERVO_STORAGE_TYPE_LOCAL   = (1 << 1),
    SERVO_STORAGE_TYPE_SESSION = (1 << 2),
    SERVO_STORAGE_TYPE_ALL     = (((1 << 0) | (1 << 1)) | (1 << 2)),
};

/* Site data information passed to ServoSitesVisitor. */
typedef struct ServoSiteData {
    const char* name;          /* eTLD+1 domain */
    const char* site;          /* alias to name */
    uint32_t storage_types;    /* bitmask of ServoStorageType */
} ServoSiteData;

/* Visitor callback for enumerating sites holding stored data. */
typedef bool (*ServoSitesVisitor)(const struct ServoSiteData* site,
                                  uintptr_t count,
                                  uintptr_t total,
                                  void* user_data);

/* Enumerate sites with stored data matching storage_types. */
int servo_site_data_visit_sites(struct ServoSiteDataManager* manager,
                                uint32_t storage_types,
                                ServoSitesVisitor visitor,
                                void* user_data);

/* Clear storage data for the given sites and storage_types. */
int servo_site_data_clear_site_data(struct ServoSiteDataManager* manager,
                                    const char* const* sites,
                                    uintptr_t count,
                                    uint32_t storage_types);
```

### New delegate callbacks

`notify_url_changed`, `notify_page_title_changed`, and `notify_closed` were added to
`ServoWebViewDelegate` in this release. Because the struct grew, use **designated
initializers** (`.field = value`) when constructing it, as shown in the example.
The `url` and `title` arguments are only valid for the duration of the callback.

> **Required:** set `.struct_size = sizeof(ServoWebViewDelegate)` (and likewise
> `sizeof(struct ServoDelegate)` / `sizeof(struct ServoEventLoopWaker)`) when
> constructing a delegate or waker. Servo rejects a struct whose `struct_size` is
> smaller than the size it was compiled with, returning `SERVO_ERROR_INVALID_ARGUMENT`.
> This turns header/library version skew into a clean error instead of undefined
> behavior.

**Load category** (new):
- `notify_load_started`: Fired when a load begins (corresponds to `SERVO_LOAD_STATUS_STARTED`).
- `notify_load_ended`: Fired when a load completes (corresponds to `SERVO_LOAD_STATUS_COMPLETE`).
- `notify_load_error`: Fired when a load error occurs. The `error_code` is a platform-specific
  code; `error_text` is a human-readable description (valid only during the callback).

**Display category** (new):
- `notify_cursor_changed`: Fired when the cursor type changes. `cursor_type` is a
  `SERVO_CURSOR_*` constant.
- `notify_status_text_changed`: Fired when the status text (e.g., link hover URL) changes.
  `text` may be `NULL` to indicate no status text.
- `notify_favicon_changed`: Fired when the favicon URL changes. `url` may be `NULL`
  to indicate no favicon.
- `show_console_message`: Fired when page content logs to the console (for
  example `console.log`, `console.warn`, or an uncaught script error reported to
  the console). `level` is a `SERVO_CONSOLE_LOG_LEVEL_*` constant (`DEBUG`,
  `LOG`, `INFO`, `WARN`, `ERROR`); `message` is the log text. **The C API applies
  no severity filtering** — every message logs through this callback, so filter
  in the callback if you only care about some levels. Engine-internal log output
  does not come through here (see `servo_setup_logging`).

**Lifespan category** (new):
- `notify_closed`: Fired when the `WebView` has been closed, either by the embedder
  calling `servo_webview_close` or by page content calling `window.close()`. The
  embedder should call `servo_webview_free` to release the `WebView` resources after
  this callback fires (or immediately if it does not wait for it).

All string arguments (`url`, `title`, `text`, `error_text`, `message`) are only valid
for the duration of the callback. Copy them if you need to retain the value.

### Clipboard delegate (new)

`ServoClipboardDelegate`, set with `servo_webview_builder_set_clipboard_delegate`, takes
over system clipboard access for a `WebView`. Install it when the host wants copy, cut
and paste to go through its own clipboard instead of Servo's built-in implementation.

```c
ServoClipboardDelegate clipboard = {
    .struct_size = sizeof(ServoClipboardDelegate),
    .user_data = app,
    .clear = NULL,             /* optional                       */
    .get_text = on_get_text,   /* required for paste to work     */
    .set_text = on_set_text,   /* required for copy/cut to work  */
};

void on_get_text(struct WebView* webview, struct ServoClipboardStringRequest* request,
                 void* user_data)
{
    /* Answer immediately; the page is waiting for this. */
    servo_webview_clipboard_request_success(request, "text on the clipboard");
}

void on_set_text(struct WebView* webview, const char* text, void* user_data)
{
    /* `text` is only valid for the duration of the callback. */
}
```

`get_text` hands the embedder an opaque `ServoClipboardStringRequest`. Exactly one of
`servo_webview_clipboard_request_success` (with the clipboard text) or
`servo_webview_clipboard_request_failure` (with a message) resolves it; either transfers
ownership of the handle, so it must not be used afterwards. Resolve it from the event
loop that runs `servo_spin_event_loop`: the paste editing action blocks the script
thread until the request is answered, so deferring the answer past the callback wedges
the page. A `NULL` `get_text` fails every request immediately, which keeps a
half-configured delegate from wedging the page.

`clear` and `set_text` are fire-and-forget: there is no response, and a `NULL` callback
simply means the request is ignored. When no clipboard delegate is set at all, Servo
uses its built-in system clipboard implementation.

Like the other delegates, `ServoClipboardDelegate` is ABI-versioned: set
`.struct_size = sizeof(ServoClipboardDelegate)`, leave `_reserved` zero, and prefer
designated initializers so future fields cannot shift the ones you set. A delegate whose
`struct_size` is smaller than the size it was compiled with is rejected with
`SERVO_ERROR_INVALID_ARGUMENT`.

### Gamepad delegate (new)

`ServoGamepadDelegate`, set with `servo_webview_builder_set_gamepad_delegate`, takes over
gamepad haptics for a `WebView`. Servo never rumble-effects a pad itself: the engine asks
the host to do it, and the host reports whether it did.

```c
ServoGamepadDelegate gamepad = {
    .struct_size = sizeof(ServoGamepadDelegate),
    .user_data = app,
    .handle_haptic_effect_request = on_haptic_effect,
};

void on_haptic_effect(struct ServoGamepadHapticEffectRequest* request, void* user_data)
{
    ServoGamepadHapticEffectInfo info = {
        .struct_size = sizeof(ServoGamepadHapticEffectInfo),
    };
    servo_webview_gamepad_haptic_effect_request_info(request, &info);

    /* info.gamepad_index identifies the pad. info.effect_type is
     * SERVO_GAMEPAD_HAPTIC_EFFECT_STOP or SERVO_GAMEPAD_HAPTIC_EFFECT_DUAL_RUMBLE;
     * for dual-rumble, duration/start_delay are milliseconds and the two
     * magnitudes run 0.0 to 1.0. */

    servo_webview_gamepad_haptic_effect_request_succeeded(request);
}
```

`servo_webview_gamepad_haptic_effect_request_info` only *reads* the request: it does not
resolve it and does not take ownership. Exactly one of
`servo_webview_gamepad_haptic_effect_request_succeeded` or
`servo_webview_gamepad_haptic_effect_request_failed` resolves it, and either transfers
ownership, so the handle must not be used afterwards. Resolve it from the event loop that
runs `servo_spin_event_loop`. If it is dropped unresolved - including when the callback
is `NULL` - the engine reports the effect as failed to the page rather than leaving it
waiting.

Two things distinguish this delegate from the others:

- **The callback receives no `WebView`.** The engine's own `GamepadDelegate` does not
  supply one, because a rumble request is addressed to a gamepad rather than to a page.
  The delegate is already bound to the `WebView` it was installed on.
- **It needs hardware to be consulted at all.** A request only exists when the engine was
  built with the `gamepad` feature *and* a gamepad is connected. A CI runner has neither,
  so `test_gamepad_delegate_abi` covers only the ABI contract - a well-formed struct is
  accepted, `struct_size = 0` is rejected, and a NULL builder is reported - and the
  haptic round trip itself is unverified here. Treat the round trip as untested until you
  have run it on a machine with a pad attached.

### Embedder controls (new)

Form controls a page cannot draw for itself are delivered through `ServoWebViewDelegate`:
the `alert()`/`confirm()`/`prompt()` family, `<select>`, `<input type=color>`,
`<input type=file>`, the context menu, and the IME. Each is an `Option` field, so a host
only implements the ones it actually wants to render:

| Field | Triggered by | Answer with |
|---|---|---|
| `show_alert_dialog` | `alert()` | `callback(...)` - the `bool` is ignored, the dialog just closes |
| `show_confirm_dialog` | `confirm()` | `callback(true/false, ...)` |
| `show_prompt_dialog` | `prompt()` | `callback(text, ...)` or `NULL` to cancel |
| `show_select_element` | activating a `<select>` | `callback(ids, count, ...)` - an empty list selects nothing |
| `show_color_picker` | `<input type=color>` | `callback(&color, ...)` or `NULL` to cancel |
| `show_file_picker` | `<input type=file>` | `callback(paths, count, ...)` or `NULL`/0 to cancel |
| `show_context_menu` | a right click in web content | `callback(SERVO_CONTEXT_MENU_ACTION_*, ...)` |
| `show_ime` / `hide_ime` | an editable element gains / loses focus | none - fire-and-forget |

Every one of them takes `user_data` as its last argument. **That `user_data` is the
opaque handle Rust handed you, not your own delegate's `user_data`** - pass it straight
back into the `callback` unchanged. Because your own state is not delivered to these
callbacks at all, keep it reachable another way: a file-scope pointer when your process
embeds one `WebView`, or a table keyed on the `webview` argument when it embeds several.
The `callback` is a `Once`: exactly one call consumes it, and the control is answered.

```c
void on_color_picker(struct WebView* webview, const ServoRgbColor* current_color,
                     ServoColorPickerCallback callback, void* user_data)
{
    /* `current_color` is the value the element held when the picker opened, or
     * NULL if it had none yet. */
    ServoRgbColor chosen = { 0x12, 0x34, 0x56 };
    callback(&chosen, user_data);      /* NULL here cancels the picker */
}

void on_select(struct WebView* webview, const ServoSelectElementParams* params,
               ServoSelectElementCallback callback, void* user_data)
{
    /* Answer with the option *ids* - each ServoSelectElementOption's `id` -
     * NOT an index into params->options, which counts each optgroup as one
     * entry while their options keep their own ids. */
    uintptr_t chosen[] = { params->options[0].option.id };
    callback(chosen, 1, user_data);
}
```

Answer from the event loop that runs `servo_spin_event_loop`, i.e. directly inside the
callback, unless you have a specific reason to defer. The responder is a `Once`, and it
holds the control: if you never call it, nothing is ever answered - the request stays
pending in the engine and the page waits indefinitely - and the responder allocation
leaks. A field left as `NULL` is different: Servo resolves the control itself with the
non-destructive answer, so the selection, color, and files the element already had are
submitted unchanged and the page sees no edit rather than a silently swallowed one.
Setting a field to `NULL` is the intended way to opt out of a control you do not want to
render.

Two of these are only reachable from **trusted** input, so script cannot trigger them:
`show_select_element` (`HTMLSelectElement::activation_behavior` returns early for
untrusted events) and `show_context_menu` (built only from the embedder's own mouse
handling). Reaching them from a test means injecting mouse events at a known layout
position. The color and file controls have no such gate, and the CI suite drives them
end to end in `test_embedder_controls_round_trip`: the control is triggered, the
delegate answers, and the page is read back to confirm the answer landed.

### Accessibility delegate (new)

Servo maintains an [AccessKit](https://accesskit.dev/) tree for every `WebView`, but it is
**off by default** because of the cost of keeping it up to date, and the feature is gated
on the `accessibility_enabled` preference. Both switches have to be on before any update
is delivered:

```c
/* 1. The preference is off by default. Any time after servo_builder_build: */
servo_set_preference_bool(servo, "accessibility_enabled", true);

/* 2. Ask for the tree for this WebView. */
servo_webview_set_accessibility_active(webview, true);
```

`servo_webview_set_accessibility_active` reports no error when accessibility cannot be
turned on: with the preference `false` the call is a no-op and no updates ever arrive,
which mirrors the Rust API, where `WebView::set_accessibility_active` returns `None`. Set
the preference **before** activating, or the activation is silently dropped. Deactivating
with `false` frees the tree and stops the callbacks.

Once active, every tree change is reported to the delegate:

```c
ServoWebViewDelegate delegate = {
    .struct_size = sizeof(ServoWebViewDelegate),
    .user_data = app,
    .notify_accessibility_tree_update = on_accessibility_tree_update,
};

void on_accessibility_tree_update(struct WebView* webview, struct ServoValue* tree_update,
    void* user_data)
{
    /* ...inspect tree_update... */
    servo_value_free(tree_update);   /* the callback owns the payload */
}
```

`tree_update` is a `ServoValue` dictionary rather than an opaque engine handle: it is
AccessKit's own JSON schema for a `TreeUpdate`, decoded into the value containers this API
already exposes (`nodes`, `treeId`, `focus`). Walk it with the `servo_value_*`,
`servo_dictionary_*` and `servo_list_*` functions, or hand the payload to a host-side
AccessKit binding. CEF hands the host a `CefValue` for the same event; this is the
equivalent.

Notes:

- **Ownership.** The value is allocated for the duration of the callback. Free it with
  `servo_value_free` or the payload leaks.
- **Shape.** The first update creates the tree and carries the viewport root and graft
  nodes; a later update for the document carries the page's own nodes; a focus change
  carries an empty `nodes` list. An empty `nodes` list is normal, not an error.
- **Not exposed yet:** forwarding an AccessKit *action* (a screen reader click, for
  example) back to the page. The Rust API has `Servo::forward_accessibility_action`; the C
  API does not, which is recorded in the build plan's §D.2.1.

### Close flow example

To close a `WebView` cooperatively, call `servo_webview_close` and wait for the
`notify_closed` delegate callback before freeing the `WebView`:

```c
static void on_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    app_state_t* app = (app_state_t*)user_data;
    app->webview_closed = true;
    printf("[close] WebView closed, freeing resources\n");
}

/* In the delegate setup: */
ServoWebViewDelegate delegate = {
    .struct_size = sizeof(ServoWebViewDelegate),
    .user_data = &app,
    .notify_load_status_changed = on_load_status_changed,
    .notify_url_changed = on_url_changed,
    .notify_page_title_changed = on_page_title_changed,
    .notify_new_frame_ready = on_new_frame_ready,
    .notify_closed = on_closed,  /* (new) */
};

/* To close the WebView: */
servo_webview_close(app.webview);

/* In the spin loop, wait for the close to complete: */
while (!app->webview_closed) {
    servo_spin_event_loop(servo);
    sleep_ms(15);
}

/* Now it's safe to free the WebView. */
servo_webview_free(app.webview);
app.webview = NULL;
```

The `webview` handle passed to `notify_closed` is a temporary handle valid only
for the duration of the callback. Do not retain it or pass it to
`servo_webview_free`; use the handle you already own instead.

## A complete C11 example

The following program builds Servo in single-process mode, opens a WebView,
loads a page, runs a manual spin loop, paints on new frames, queries the URL and
title with the buffer-query pattern, evaluates JavaScript, and takes a
screenshot. It is intentionally self-contained.

```c
/* servo_embed.c - minimal C11 embedder for Servo.
 *
 * Build (after `cargo cinstall -p servo-capi --prefix /path/to/sysroot`):
 *   cc -std=c11 -Wall -Wextra servo_embed.c \
 *      -I/path/to/sysroot/include -L/path/to/sysroot/lib \
 *      -lservo_capi -o servo_embed
 */
#define _POSIX_C_SOURCE 200809L /* for nanosleep on POSIX */
#include <servo/servo_capi.h>

#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <time.h>
#endif

static void sleep_ms(long period)
{
#ifdef _WIN32
    Sleep((DWORD)period);
#else
    struct timespec delay;
    delay.tv_sec = period / 1000;
    delay.tv_nsec = (period % 1000) * 1000000L;
    nanosleep(&delay, NULL);
#endif
}

/* Embedder state, passed to callbacks as user_data. It must outlive the
 * WebView. */
typedef struct {
    struct Servo* servo;
    struct WebView* webview;
    atomic_bool new_frame_pending;
} app_state_t;

/* The waker may be called from any Servo-internal thread. Keep it tiny: just
 * record that work is pending. Do not touch Servo handles here. */
static atomic_bool g_wake_pending = false;

static void on_wake(void)
{
    atomic_store_explicit(&g_wake_pending, true, memory_order_release);
}

/* --- Delegate callbacks. All run on the thread calling
 * servo_spin_event_loop. Never free a handle from inside a callback. --- */

static void on_load_status_changed(struct WebView* webview, int32_t load_status,
    void* user_data)
{
    (void)webview;
    (void)user_data;
    const char* name = "unknown";
    switch (load_status) {
    case SERVO_LOAD_STATUS_STARTED:
        name = "started";
        break;
    case SERVO_LOAD_STATUS_HEAD_PARSED:
        name = "head-parsed";
        break;
    case SERVO_LOAD_STATUS_COMPLETE:
        name = "complete";
        break;
    default:
        break;
    }
    printf("[load] %s\n", name);
}

static void on_url_changed(struct WebView* webview, const char* url, void* user_data)
{
    (void)webview;
    (void)user_data;
    /* `url` is only valid for the duration of this call. */
    printf("[url] %s\n", url);
}

static void on_page_title_changed(struct WebView* webview, const char* title,
    void* user_data)
{
    (void)webview;
    (void)user_data;
    /* `title` is only valid for the duration of this call. */
    printf("[title] %s\n", title);
}

static void on_new_frame_ready(struct WebView* webview, void* user_data)
{
    (void)webview;
    app_state_t* app = (app_state_t*)user_data;
    atomic_store_explicit(&app->new_frame_pending, true, memory_order_release);
}

static void on_screenshot(const uint8_t* data, uint32_t width, uint32_t height,
    int32_t error, void* user_data)
{
    (void)user_data;
    if (data == NULL) {
        printf("[screenshot] failed, error=%d\n", error);
        return;
    }
    printf("[screenshot] %ux%u RGBA (first pixel %u,%u,%u,%u)\n", width, height,
        data[0], data[1], data[2], data[3]);
    /* `data` is only valid for the duration of this call; copy it to keep it. */
}

static void on_javascript_result(const char* result_json, const char* error_message,
    void* user_data)
{
    (void)user_data;
    if (error_message != NULL) {
        printf("[js] error: %s\n", error_message);
    } else {
        printf("[js] result: %s\n", result_json != NULL ? result_json : "null");
    }
    /* Both strings are only valid for the duration of this call. */
}

int main(void)
{
    static app_state_t app; /* zero-initialized */

    /* 1. Options: single-process, no sandbox, temporary storage. */
    ServoOptions* options = servo_options_create();
    if (options == NULL)
        return 1;
    servo_options_set_multiprocess(options, false);
    servo_options_set_sandbox(options, false);
    servo_options_set_temporary_storage(options, false);

    /* 2. Preferences: any Servo preference can be set by name. */
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL) {
        servo_options_free(options);
        return 1;
    }
    servo_preferences_set_string(preferences, "user_agent", "ServoEmbed/1.0");

    /* 3. Builder: options/preferences transfer ownership; the waker is copied
     * by value. */
    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;
    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = on_wake,
    };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_set_options(builder, options);         /* builder owns options now */
    servo_builder_set_preferences(builder, preferences); /* builder owns preferences now */

    /* 4. Build Servo (consumes the builder) and enable logging. */
    struct Servo* servo = servo_builder_build(builder); /* builder is consumed */
    if (servo == NULL) {
        fprintf(stderr, "failed to build Servo\n");
        return 1;
    }
    servo_setup_logging(servo);
    app.servo = servo;

    /* 5. Rendering context.
     *
     * Choose ONE of the following:
     *
     * Option A: Software rendering context (headless, works everywhere).
     */
    const uint32_t width = 800;
    const uint32_t height = 600;
    RenderingContext* context =
        servo_rendering_context_create_software(width, height);

    /*
     * Option B: Windowed rendering context (GPU-accelerated, requires a native window).
     *
     * This example shows how to obtain the raw window handles from winit.
     * Adapt for SDL, GLFW, or other windowing libraries as needed.
     *
     * #include <raw_window_handle.h>  // from raw-window-handle C headers
     *
     * // ... create winit window ...
     * #ifdef _WIN32
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_HWND, .handle.hwnd = window.hwnd() as *mut _ };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_WINDOWS, .handle.windows = { .hinstance = GetModuleHandleW(NULL) } };
     * #elif defined(__linux__)
     *     // For X11
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_XLIB, .handle.xlib = { .window = window.xlib_window().unwrap() as _, .display = window.xlib_display().unwrap() as _ } };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_XLIB, .handle.xlib = { .display = window.xlib_display().unwrap() as _ } };
     *     // For Wayland
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_WAYLAND, .handle.wayland = { .surface = window.wl_surface().unwrap() as _ } };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_WAYLAND, .handle.wayland = { .display = window.wl_display().unwrap() as _ } };
     * #elif defined(__APPLE__)
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_NSVIEW, .handle.nsview = { .ns_view = window.ns_view() as _ } };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_APPKIT, .handle.appkit = { } };
     * #endif
     *
     * RenderingContext* context =
     *     servo_rendering_context_create_window(&display_handle, &window_handle, width, height);
     */

    /*
     * Option C: Offscreen rendering context (GPU-accelerated OSR / FBO composition).
     *
     * First create a window context, then create an offscreen child context.
     * The offscreen context shares the window's GL context and renders to an FBO.
     * This is useful for rendering web content to a texture for custom composition.
     *
     * #include <raw_window_handle.h>
     *
     * // ... create winit window ...
     * #ifdef _WIN32
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_HWND, .handle.hwnd = window.hwnd() as *mut _ };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_WINDOWS, .handle.windows = { .hinstance = GetModuleHandleW(NULL) } };
     * #elif defined(__linux__)
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_XLIB, .handle.xlib = { .window = window.xlib_window().unwrap() as _, .display = window.xlib_display().unwrap() as _ } };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_XLIB, .handle.xlib = { .display = window.xlib_display().unwrap() as _ } };
     * #elif defined(__APPLE__)
     *     struct RawWindowHandle window_handle = { .type = RAW_WINDOW_HANDLE_NSVIEW, .handle.nsview = { .ns_view = window.ns_view() as _ } };
     *     struct RawDisplayHandle display_handle = { .type = RAW_DISPLAY_HANDLE_APPKIT, .handle.appkit = { } };
     * #endif
     *
     * RenderingContext* window_context =
     *     servo_rendering_context_create_window(&display_handle, &window_handle, width, height);
     * if (window_context != NULL) {
     *     RenderingContext* offscreen_context =
     *         servo_rendering_context_create_offscreen(window_context, width, height);
     *     if (offscreen_context != NULL) {
     *         context = offscreen_context;  // Use offscreen context for WebView
     *         // window_context must be kept alive for the lifetime of offscreen_context
     *     }
     * }
     */

    if (context == NULL) {
        fprintf(stderr, "failed to create rendering context\n");
        servo_free(servo);
        return 1;
    }

    /* 5b. Create and configure UserContentManager with message router. */
    ServoUserContentManager* ucm = servo_user_content_manager_create();
    servo_user_content_manager_enable_message_router(ucm);

    /* 6. WebView builder and delegate. The builder takes ownership of the
     * rendering context. */
    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    servo_webview_builder_set_user_content_manager(webview_builder, ucm);
    if (webview_builder == NULL) {
        servo_rendering_context_free(context); /* we still own it here */
        servo_free(servo);
        return 1;
    }

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &app,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_new_frame_ready = on_new_frame_ready,
        // New load callbacks:
        // .notify_load_started = on_load_started,
        // .notify_load_ended = on_load_ended,
        // .notify_load_error = on_load_error,
        // New display callbacks:
        // .notify_cursor_changed = on_cursor_changed,
        // .notify_status_text_changed = on_status_text_changed,
        // .notify_favicon_changed = on_favicon_changed,
        // .show_console_message = on_console_message,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    if (servo_webview_builder_set_url(webview_builder, "data:text/html,"
            "<title>Embedded</title><h1>Hello from Servo</h1>") != 0) {
        fprintf(stderr, "invalid initial URL\n");
        servo_webview_builder_free(webview_builder);
        servo_free(servo);
        return 1;
    }

    /* 7. Build the WebView (consumes the builder). */
    struct WebView* webview = servo_webview_builder_build(webview_builder);
    if (webview == NULL) {
        servo_free(servo);
        return 1;
    }
    app.webview = webview;

    /* Navigation, zoom, focus and visibility all just enqueue work; nothing
     * happens until the event loop is spun. */
    servo_webview_load(webview, "https://example.com/");
    servo_webview_set_page_zoom(webview, 1.0f);
    servo_webview_set_focused(webview, true);
    servo_webview_show(webview);

    /* Asynchronous JavaScript evaluation. */
    servo_webview_evaluate_javascript(webview, "document.title",
        on_javascript_result, &app);

    /* 8. Manual spin loop. A real embedder blocks on the waker; this example
     * polls. All WebView calls and painting happen on this same thread. */
    int screenshot_requested = 0;
    for (int i = 0; i < 2000; i++) { /* ~30 s at 15 ms per iteration */
        servo_spin_event_loop(servo);

        /* Poll for messages from JavaScript via the message router. */
        servo_webview_poll_js_messages(webview);

        /* Wake handling: clear the flag. A blocking loop would use it to
         * decide when to wake up. */
        atomic_store_explicit(&g_wake_pending, false, memory_order_release);

        if (atomic_exchange_explicit(&app.new_frame_pending, false,
                memory_order_acq_rel)) {
            servo_webview_paint(webview); /* render into the context */
        }

        /* Buffer-query pattern for the current URL. */
        uintptr_t url_len = servo_webview_get_url(webview, NULL, 0);
        if (url_len > 0) {
            char* url = (char*)malloc((size_t)url_len);
            if (url != NULL) {
                servo_webview_get_url(webview, url, url_len);
                printf("[query] url = %s\n", url);
                free(url);
            }
        }

        /* The same pattern for the page title and status text. */
        uintptr_t title_len = servo_webview_get_title(webview, NULL, 0);
        if (title_len > 0) {
            char* title = (char*)malloc((size_t)title_len);
            if (title != NULL) {
                servo_webview_get_title(webview, title, title_len);
                printf("[query] title = %s\n", title);
                free(title);
            }
        }

        if (!screenshot_requested && i == 50) {
            servo_webview_take_screenshot(webview, on_screenshot, &app);
            screenshot_requested = 1;
        }

        sleep_ms(15);
    }

    /* 9. Tear down in reverse order. WebViews before Servo. The rendering
     * context is owned by the WebView and is not freed separately. */
    servo_webview_free(webview);
    servo_free(servo);
    return 0;
}
```

## The buffer-query pattern

`servo_webview_get_url`, `servo_webview_get_title` and
`servo_webview_get_status_text` never allocate for you. They write into a
caller-provided buffer and return the number of bytes **required, including the
terminating NUL**. If the buffer is `NULL` or too small, nothing is written and
the required size is returned:

```c
/* Ask how much space is needed. */
uintptr_t needed = servo_webview_get_title(webview, NULL, 0);
if (needed > 0) {
    char* buffer = (char*)malloc((size_t)needed);
    if (buffer != NULL) {
        uintptr_t written = servo_webview_get_title(webview, buffer, needed);
        /* If `written` is larger than `needed`, the value grew between the two
         * calls (for example, a title arrived). Retry with the new size. */
        if (written <= needed) {
            printf("title: %s\n", buffer);
        }
        free(buffer);
    }
}
```

Because the returned size can change between calls, always compare the second
return value against the buffer length before using the string.

> **Type note.** cbindgen maps the Rust `usize` type (used for buffer lengths and
> the `go_back` / `go_forward` amounts) to C's `uintptr_t`, not `size_t`. On every
> platform Servo supports these are the same underlying unsigned integer type, so
> passing a `size_t` value (for example the result of `sizeof` or `malloc`)
> compiles without warnings. The header is authoritative; match it if in doubt.

## Evaluating JavaScript

`servo_webview_evaluate_javascript` runs `script` in the document currently
loaded in the WebView and invokes the callback exactly once, on the spin thread,
when evaluation completes:

```c
static void on_js(const char* result_json, const char* error_message, void* user_data)
{
    if (error_message != NULL) {
        fprintf(stderr, "JS error: %s\n", error_message);
    } else {
        /* result_json is valid JSON. `undefined` and `null` both serialize as
         * the four characters `null`. */
        printf("JS result: %s\n", result_json);
    }
}

servo_webview_evaluate_javascript(webview, "({ title: document.title, n: 1 + 2 })",
    on_js, &app);
```

Exactly one of `result_json` / `error_message` is non-`NULL`. Both strings are
owned by Servo and are only valid for the duration of the callback; copy them if
you need to keep them. A `NULL` callback is ignored (the script is not run).

## Native↔JS Message Router

The message router provides bidirectional communication between native code and
JavaScript, similar to CEF's `CefMessageRouter`.

### Enabling the message router

Before creating a `WebView`, enable the message router on the
`UserContentManager`:

```c
ServoUserContentManager* ucm = servo_user_content_manager_create();
servo_user_content_manager_enable_message_router(ucm);
```

Then pass this `UserContentManager` to the `WebViewBuilder` (via
`servo_webview_builder_set_user_content_manager` — see [UserContentManager](#usercontentmanager)).

### Sending messages from native to JS (native→JS)

```c
servo_webview_post_message(webview, "my-message", "{\"value\": 42}");
```

This dispatches the message to registered JavaScript listeners. Listeners registered
via `window.servo.addEventListener("message", handler)` receive an event object
`{ type: "message", data: { "value": 42 }, origin }`, while named listeners registered
via `window.servoMessageRouter.addEventListener(name, handler)` (or
`window.servo.addEventListener(name, handler)`) receive the payload `data` directly.

### Receiving messages from native in JS

```javascript
// Preferred API (window.servo):
window.servo.addEventListener("message", function(event) {
    console.log("Received from native:", event.data); // { value: 42 }
});

// Or using message-router alias:
window.servoMessageRouter.addEventListener("my-message", function(data) {
    console.log("Received from native:", data); // { value: 42 }
});
```

### Sending messages from JS to native (JS→native)

```javascript
// Preferred API (window.servo):
window.servo.postMessage("js-message", { fromJS: true });

// Or using message-router alias:
window.servoMessageRouter.postMessage("js-message", { fromJS: true });
```

### Receiving messages from JS in native

```c
// Method 1: Named callback via servo_webview_add_script_message_handler:
static void on_script_message(struct WebView* webview, const char* name, const char* data_json, void* user_data) {
    printf("Received message '%s' from JS: %s\n", name, data_json);
}

servo_webview_add_script_message_handler(webview, "js-message", on_script_message, &app);

// Method 2: Via ServoWebViewDelegate:
delegate.on_script_message = on_script_message;
```

### Removing a message handler

```c
servo_webview_remove_script_message_handler(webview, "js-message");
```

### JS API reference

The injected `window.servo` object (and backward-compatible `window.servoMessageRouter` alias) provides:

- `postMessage(handler_name, data)` — Send a message to native. `data` can be any
  JSON-serializable value or primitive.
- `addEventListener("message", handler)` — Register a listener for messages from
  native host.
- `removeEventListener("message", handler)` — Unregister a message listener.

### Complete example

```c
/* Enable message router on UserContentManager */
ServoUserContentManager* ucm = servo_user_content_manager_create();
servo_user_content_manager_enable_message_router(ucm);

/* ... create WebView with this ucm ... */

/* Native→JS: send a message */
servo_webview_post_message(webview, "greeting", "{\"text\": \"Hello from native!\"}");

/* JS→native: register a handler */
static void on_ping(const char* name, const char* data_json, void* user_data) {
    (void)user_data;
    printf("JS says: %s\n", data_json);
    /* Reply from native */
    servo_webview_post_message(webview, "pong", "{\"received\": true}");
}

servo_webview_add_message_handler(webview, "ping", on_ping, NULL);

/* In the event loop, poll for JS messages */
while (running) {
    servo_spin_event_loop(servo);

    /* Poll for messages from JavaScript */
    servo_webview_poll_js_messages(webview);

    /* ... other event loop work ... */
}
```

The corresponding JavaScript:

```javascript
// Listen for native→JS
window.servoMessageRouter.addEventListener("greeting", function(data) {
    console.log("Native says:", data.text); // "Hello from native!"
    // Reply to native
    window.servoMessageRouter.postMessage("ping", { timestamp: Date.now() });
});

// Listen for native reply
window.servoMessageRouter.addEventListener("pong", function(data) {
    console.log("Native replied:", data); // { received: true }
});
```

## DevTools remote debugging

Servo can expose a remote debugging server so a DevTools frontend can inspect the
WebViews it hosts. **The server is off by default.** Enable it *before* building
the `Servo` instance, because the server is started inside
`servo_builder_build`:

```c
ServoPreferences* preferences = servo_preferences_create();
/* enabled, port (0 means "use the default port", which is 7000) */
servo_preferences_set_devtools_server(preferences, true, 9222);
servo_builder_set_preferences(builder, preferences);
```

`servo_preferences_set_devtools_server` is a convenience wrapper for the
`devtools_server_enabled` and `devtools_server_listen_address` preferences; the
same thing can be expressed with `servo_preferences_set_bool(preferences,
"devtools_server_enabled", true)` (which leaves the address at its default,
`127.0.0.1:7000`).

The server speaks the **legacy Firefox Remote Debugging Protocol** (length-prefixed
JSON packets over a plain TCP connection; see `components/devtools/protocol.rs`).
It is *not* the Chrome DevTools Protocol, not HTTP and not WebSocket, so
`chrome://inspect` and Chrome DevTools cannot attach to it — a client has to
speak that protocol.

### Authorization flow

`notify_devtools_server_started` fires once the server has started, carrying the
port and a one-time auth token that is only valid for the duration of the
callback:

```c
static void on_devtools_started(struct Servo* servo, uint16_t port,
                                const char* token, void* user_data)
{
    (void)servo;
    (void)user_data;
    /* `token` is only valid for the duration of this call. Copy it if needed. */
    printf("[devtools] Server started on port %u\n", port);
}

static bool on_devtools_connection_request(struct Servo* servo, void* user_data)
{
    (void)servo;
    (void)user_data;
    /* In a real embedder, ask the user before granting access. */
    return false;
}

ServoDelegate servo_delegate = {
    .struct_size = sizeof(ServoDelegate),
    .user_data = &app,
    .notify_devtools_server_started = on_devtools_started,
    .request_devtools_connection = on_devtools_connection_request,
};
servo_builder_set_delegate(builder, servo_delegate);
```

A client that already knows the token may present it in its first packet
(`<length>:{"auth_token":"<token>"}`) to connect without a prompt. Every other
connection is handed to `request_devtools_connection`, which returns `true` to
allow it and `false` to deny it. If no callback is set, the connection is
**denied**, so no client can attach by accident.

Both callbacks currently receive `NULL` for their `servo` argument — the Rust
`ServoDelegate` trait does not hand the delegate a `Servo` handle, so there is no
temporary handle to pass. Do not dereference it; use `user_data` to reach
embedder state (`ffi/capi/servo_delegate.rs`).

> **Security — authorizing a DevTools connection is a remote-code-execution
> grant.** A connected client can evaluate arbitrary script in *every* `WebView`
> of the process, read page content and cookie/storage data, and drive
> navigation. Treat it like handing a debugger to whoever asked:
>
> - Keep the server disabled in production embedders unless the user turned it
>   on deliberately. The API always binds `127.0.0.1`; never forward that port
>   to a network.
> - Treat the auth token as a password: it bypasses the prompt, so do not log it
>   where page content or other users could read it.
> - If the address cannot be bound, no server is started and
>   `notify_devtools_server_started` never fires. There is currently **no C
>   callback for that failure** (the Rust
>   `ServoDelegate::notify_error(ServoError::DevtoolsFailedToStart)` path has no C
>   equivalent), so treat a missing start callback as "the server did not
>   start".

The C test suite covers the start notification on all three platforms
(`tests/capi/c/test_integration.c`, `test_devtools_server_started`). The
connection gate itself is not covered by an automated test: exercising it needs a
client that completes the Remote Debugging Protocol connection.

## Taking screenshots

`servo_webview_take_screenshot` is asynchronous. The callback is invoked on the
spin thread with either:

- `data != NULL`: RGBA8 pixels, `4 * width * height` bytes, width/height set; or
- `data == NULL`: an error, reported by the `SERVO_SCREENSHOT_CAPTURE_ERROR_*`
  constant in `error`.

The pixel `data` pointer is only valid for the duration of the callback; copy it
if you need to keep the image. Servo typically calls `servo_webview_paint` first
(for example from `notify_new_frame_ready`) so the screenshot reflects the latest
frame.

## Threading rule

- **One thread owns Servo.** Create the builder, build the `Servo`, create
  `WebView`s, and call `servo_spin_event_loop` all from the same embedder thread.
  Using handles from another thread is undefined behavior.
- **The waker is the only exception.** `ServoEventLoopWaker.wake_callback` may be
  invoked from any Servo-internal thread. It must be safe to call from any
  thread, must not block, and must not call any Servo API. Its only job is to
  notify the embedder thread (for example, by writing to a pipe, an eventfd, or a
  condition variable) that it should call `servo_spin_event_loop` soon.
- **Callbacks run on the spin thread.** `ServoWebViewDelegate` callbacks are
  invoked while the embedder thread is inside `servo_spin_event_loop`. You may
  call Servo functions from a callback (for example,
  `servo_webview_paint` from `notify_new_frame_ready`), but see the pitfalls
  below.
- **Spin after every call.** WebView calls only enqueue work. If you never spin,
  nothing happens.

## Ownership and free rules

| Handle | Obtained from | Released with | Notes |
| --- | --- | --- | --- |
| `ServoOptions*` | `servo_options_create` | `servo_options_free`, or transferred by `servo_builder_set_options` | After `set_options` the builder owns it; freeing it yourself is a double free. |
| `ServoPreferences*` | `servo_preferences_create` | `servo_preferences_free`, or transferred by `servo_builder_set_preferences` | Same as options. |
| `ServoBuilder*` | `servo_builder_create` | `servo_builder_free`, or consumed by `servo_builder_build` | `build` returns a `Servo`; the builder must not be used or freed afterwards. |
| `struct Servo*` | `servo_builder_build` | `servo_free` | Free **after** all WebViews created from it. |
| `RenderingContext*` | `servo_rendering_context_create_software` | `servo_rendering_context_free`, or transferred by `servo_webview_builder_create` | Once passed to the WebView builder, it is owned by the resulting `WebView`; do not free it separately. |
| `ServoWebViewBuilder*` | `servo_webview_builder_create` | `servo_webview_builder_free`, or consumed by `servo_webview_builder_build` | Freeing the builder also drops the rendering context if it was never built. |
| `struct WebView*` | `servo_webview_builder_build` | `servo_webview_free` | Free before the `Servo`. |
| `char*` from `servo_preferences_get_string` | preference getter | C `free()` | Allocated with the system allocator. |
| Buffer passed to `servo_webview_get_*` | caller | caller | The getters only write into it; use `malloc`/`free` or a stack buffer. |
| `const char*` from `servo_capi_version` | version function | *(never)* | Static storage; do not free or modify. |
| Screenshot pixel `data` | screenshot callback | *(never)* | Valid only during the callback. |
| JS `result_json` / `error_message` | JS callback | *(never)* | Valid only during the callback. |
| `user_data` | caller | caller | Must outlive the `WebView` that references it. |

Set handles to `NULL` after freeing when you keep the variable around; the API
does not null your pointers for you.

## Error handling and panic policy

All fallible functions return a `ServoError` code (`int32_t`). Success is represented by `SERVO_ERROR_OK` (`0`); negative values indicate failure:

```c
typedef enum ServoError {
    SERVO_ERROR_OK = 0,
    SERVO_ERROR_NULL_ARGUMENT = -1,    /* Required pointer argument was NULL */
    SERVO_ERROR_INVALID_UTF8 = -2,     /* String argument was not valid UTF-8 */
    SERVO_ERROR_INVALID_STATE = -3,    /* E.g. webview shutting down, builder already consumed */
    SERVO_ERROR_NOT_FOUND = -4,        /* Requested resource not found */
    SERVO_ERROR_CAPACITY = -5,         /* Caller-provided buffer too small */
    SERVO_ERROR_UNSUPPORTED = -6,      /* Operation unsupported on this platform or build */
    SERVO_ERROR_INTERNAL = -7,         /* Internal bug in Servo */
    SERVO_ERROR_INTERNAL_PANIC = -8,   /* Panic caught at the FFI boundary */
    SERVO_ERROR_INVALID_ARGUMENT = -9, /* Bad struct_size, or a handle that is not live */
    SERVO_ERROR_WRONG_THREAD = -10,    /* Called from a thread that does not own the handle */
} ServoError;
```

### Conventions:
1. **Fallible functions**: Return `ServoError`. Embedders should check return codes for `< 0`.
2. **Object constructors**: Return a pointer (`*mut T`), returning `NULL` on failure or allocation error.
3. **Panic-free boundary guarantee**: All FFI entry points are protected by panic boundaries (`catch_unwind`). No invalid argument or internal panic can cross the C ABI to crash the host process. If a panic occurs inside Servo during an FFI call, `SERVO_ERROR_INTERNAL_PANIC` (`-8`) or `NULL` is returned.

## Hang monitoring and hung-script recovery

Servo runs a background hang monitor (BHM) and a hung-script kill switch, **both
enabled by default** (`servo_options_create()` returns defaults with both on).
What that means for an embedder:

- The script thread reports activity to the BHM. A component that stops
  servicing checkpoints for 1 second produces a transient warning (diagnostics
  only); one that stays wedged for **5 seconds** is treated as permanently
  hung.
- On a permanent hang, with the kill option enabled, Servo terminates the hung
  JavaScript: an in-flight `servo_webview_evaluate_javascript` completes with
  an **error** instead of never returning, the WebView goes inert (no further
  delegate callbacks, no frames), and nothing aborts. That error is the
  defined outcome of a kill.
- After a kill the WebView cannot be revived. Recovery: free the inert
  WebViews and create new ones — Servo spawns a fresh script thread for them —
  or exit and restart the engine. In multiprocess builds, letting the browser
  process reap the content process is the equivalent.
- Opt out per flag before `servo_builder_build`:
  - `servo_options_set_background_hang_monitor(options, false)` — no
    monitoring at all.
  - `servo_options_set_hang_monitor_kill_script(options, false)` — monitor and
    warn, but leave hung scripts running (the pre-W5.1 behaviour).
- Limitation: the kill is cooperative and interrupts **JavaScript**. A script
  thread wedged inside native code that never re-enters JavaScript is not
  interrupted; guard such work yourself or enforce app-level timeouts.

`test_hang_monitor_kills_hung_script` in `tests/capi/c/test_integration.c`
exercises this end to end with the shipped defaults.

## Common pitfalls

- **Not spinning the event loop.** `servo_webview_load`,
  `servo_webview_evaluate_javascript`, `servo_webview_take_screenshot`,
  `servo_webview_go_back` and friends only queue work. If your program never
  calls `servo_spin_event_loop`, no navigation, callback or frame will happen.
- **Wrong thread.** The C API is single-threaded: the thread that first calls
  through the FFI boundary is the *owning* thread. Any call to a boundary-wrapped
  function from another thread returns `SERVO_ERROR_WRONG_THREAD` (`-10`). The
  only cross-thread exceptions are the pure `servo_capi_*` queries and the
  protocol-handler response/request accessors called directly from a registered
  protocol handler. Keep `user_data` access on the owning thread too (or protect
  it with your own synchronization).
- **Doing too much in the waker.** The waker may run on a Servo-internal thread.
  It must not block, allocate heavily, or call Servo functions. Signal your
  event loop and return.
- **Freeing handles inside callbacks.** A delegate callback holds internal state
  of the `WebView`/`Servo`. Do not call `servo_webview_free`, `servo_free`, or
  `servo_webview_builder_free` from within a callback. Defer teardown until after
  `servo_spin_event_loop` returns.
- **Assuming callbacks run on the thread that made the request.**
  `servo_webview_take_screenshot` and `servo_webview_evaluate_javascript` invoke
  their callbacks during a later `servo_spin_event_loop`, on the spin thread.
- **Forgetting `user_data` lifetime.** The delegate's `user_data` and the
  `user_data` passed to `take_screenshot`/`evaluate_javascript` must remain valid
  until the callback fires or the `WebView` is freed. Do not point them at
  stack locals that go out of scope.
- **Not copying callback strings.** URL, title, status text, JS results and
  screenshot pixels are only valid during the callback. Copy anything you keep.
- **Freeing the rendering context twice.** After
  `servo_webview_builder_create`, the context belongs to the WebView builder (and
  then the WebView). Do not also call `servo_rendering_context_free`.
- **Freeing Servo before its WebViews.** WebViews borrow from the `Servo`; free
  every `WebView` first.
- **Reusing a consumed builder.** `servo_builder_build` and
  `servo_webview_builder_build` consume their argument. Freeing or reusing the
  builder afterwards is a use-after-free/double-free.
- **The wrong include path.** The header installs under a `servo/` subdirectory:
  `#include <servo/servo_capi.h>`, with `-I<prefix>/include` on the compile line.
- **Ignoring error returns.** All fallible functions now return a `ServoError`
  code (`int32_t`). Check the return value; a negative value indicates failure.
  Common errors include `SERVO_ERROR_NULL_ARGUMENT` (-1) for null pointers and
  `SERVO_ERROR_INVALID_UTF8` (-2) for invalid UTF-8 strings. Functions that
  previously panicked on bad input now return an error code instead.
- **Ignoring the second return of a buffer-query getter.** The required size can
  change between calls; re-check it against your buffer length before using the
  string.
- **Positional delegate initializers.** `ServoWebViewDelegate` gained leading
  `struct_size`/`_reserved` fields. Always use designated initializers and set
  `.struct_size = sizeof(...)` on every `#[repr(C)]` struct you fill in:
  `ServoWebViewDelegate`, `ServoDelegate`, `ServoEventLoopWaker`,
  `ServoClipboardDelegate`, `ServoGamepadDelegate`,
  `ServoGamepadHapticEffectInfo`, `ServoCookieFilter` and `ServoProxyConfig`.
  Servo rejects a too-small `struct_size` with `SERVO_ERROR_INVALID_ARGUMENT`,
  so a zeroed struct is rejected rather than silently read as "all defaults".
- **Reusing a handle after freeing it.** Handles are opaque pointers; using one
  after `free`, or freeing it twice, used to be a silent double free or
  use-after-free. The entry points of the handle types that have adopted the
  live-handle registry (`ServoOptions` today, see `ABI_STABILITY.md` §4.5) now
  return `SERVO_ERROR_INVALID_ARGUMENT` for a handle that is not live, including
  one that was consumed by `servo_builder_set_options`. Not every handle type
  is covered yet, so `NULL` your pointers after freeing regardless.

## Multi-process support (multiprocess feature)

Servo supports a multi-process architecture where content (JavaScript, DOM, layout)
runs in separate **content processes**, isolated from the **browser process** that
hosts the embedder's event loop and drives `servo_spin_event_loop`. This is
enabled by the `multiprocess` feature of the `servo-capi` crate (and the
underlying `servo` crate).

### Enabling multi-process

1. Build `servo-capi` with the `multiprocess` feature:
   ```sh
   cargo cinstall -p servo-capi --features multiprocess --library-type cdylib --prefix /path/to/servo-sysroot
   ```

2. In the browser process, enable multi-process mode via options:
   ```c
   ServoOptions* options = servo_options_create();
   servo_options_set_multiprocess(options, true);
   servo_options_set_sandbox(options, true);       // recommended: sandbox content processes
   servo_options_set_force_ipc(options, true);     // use IPC channels even for single-process testing
   ```

3. Make your own `main` route `--content-process <token>` to
   `servo_run_content_process` — **Servo launches your executable for you**, it
   does not ask you to launch anything (see *Launching content processes* below).

4. Build the `Servo` instance as normal. The browser process will now expect to
   communicate with content processes over IPC.

### Launching content processes

**Servo launches them.** This is the part that surprises embedders: there is no
token for you to obtain and no child for you to start. When the constellation
needs a content process — a script event loop, or the service worker manager —
and `opts.multiprocess` is on, `spawn_multiprocess`
(`components/constellation/sandboxing.rs`) creates an `IpcOneShotServer`, keeps
the token, and spawns **`current_exe()` — your own executable** — with exactly
two arguments:

```
<your executable> --content-process <token>
```

`RUST_BACKTRACE` and `RUST_LOG` are the only environment variables passed
through; there is no `SERVO_CONTENT_TOKEN`. On Linux and macOS with a sandbox
enabled the same executable is started through `gaol::sandbox::Command::me()`,
so the child arrives already inside the sandbox profile.

So the entire embedder-side protocol is: **route that argument in your own
`main`, before anything else, and hand the token to
`servo_run_content_process`.**

```c
/* Returns -1 when this process is not a Servo-spawned content child, so the
 * caller falls through to the browser path. Called first thing in main. */
static int run_as_content_process(int argc, char** argv) {
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], "--content-process") == 0) {
            /* Never returns while the content process runs; -1 on refusal. */
            int rc = servo_run_content_process(argv[i + 1]);
            return rc == 0 ? 0 : 1;
        }
    }
    return -1;
}
```

Three consequences worth knowing before the examples:

- **The child is your binary.** `current_exe()` must be the program that
  contains this routing. If the process that builds the `Servo` instance is not
  the one that can be re-executed and routed like this, content processes will
  not start, and there is no callback or query with which to redirect the spawn.
- **If you do not route the token, the browser stalls.** The parent blocks in
  `server.accept()` waiting for that child to connect, so an embedder that
  ignores `--content-process` hangs the first time an event loop is spawned.
  This is a hard failure mode, not a silent fallback to single process.
- **One process per event loop, spawned on demand.** Not a fixed pool and no
  zygote: the number of children grows with what the document does.

### What the content process does

`servo_run_content_process(token)` (`ffi/capi/multiprocess.rs`) connects to the
one-shot server, receives its `Opts` and preferences **from the browser process
over IPC**, and only then, if those options say so, enters the sandbox
(`components/servo/servo.rs::run_content_process`). Nothing has to be
initialised beforehand, so it is correct to call it as the first statement of
the child's `main`:

```c
int main(int argc, char** argv) {
    int content = run_as_content_process(argc, argv);
    if (content >= 0) return content;

    /* Normal browser process: build Servo, spin the event loop, … */
    return run_browser(argc, argv);
}
```

- It returns `-1` if `token` is NULL, if the `multiprocess` feature is off, or
  if the run fails; otherwise it does not return at all.
- The export exists in **both** builds: a binary linked against a
  non-multiprocess `servo-capi` still links, and reports the refusal at runtime.
  `tests/capi/c/test_api.c::test_multiprocess_single_process_path` asserts that
  refusal (CI builds `servo-capi` *without* the feature, so the child spawn
  itself cannot be exercised in this harness — said here rather than faked).
- An explicit `servo_create_sandbox()` before it is **redundant**: the sandbox is
  applied inside, after the options arrive, so that it matches what the browser
  process asked for. The function exists for embedders that want to place the
  step themselves (it must then be the first thing in `main`), and on Windows it
  returns `false` *after* applying the in-process mitigation policies — see
  **Sandbox** below.

### Capability queries

```c
bool servo_capi_is_multiprocess_enabled(void);  /* the build was compiled with the `multiprocess` feature */
bool servo_capi_is_sandboxing_supported(void);  /* …and this target can activate a real OS sandbox */
```

Query rather than assume. Set `servo_options_set_multiprocess(options, true)`
**only** when `servo_capi_is_multiprocess_enabled()` is true: with the feature
off the spawn sites do not exist, while the runtime flag is still read by some
browser-process paths (the hang monitor is initialised as though a content
process were coming, `components/constellation/constellation.rs`), and that
combination is exercised by no test on this branch. Likewise query
`servo_capi_is_sandboxing_supported()` instead of inferring it from the platform
string.

### Sandbox

When `servo_options_set_sandbox(options, true)` is set in the browser process,
content processes will automatically enter a restrictive sandbox on supported
platforms (Linux with `gaol`, macOS with `sandbox-exec`). The sandbox profile
allows only:
- Reading `/dev/urandom`
- Reading system fonts and framework libraries
- Reading Servo's baked-in resources (fonts, shaders, etc.)

On Linux (with `gaol`) and macOS (with `sandbox-exec`), content processes enter
the sandbox profile. On Windows, content processes apply in-process mitigation
policies (DEP, ASLR HighEntropy, strict handle check, extension-point disable) —
but mitigations are not a sandbox (no token lockdown or AppContainer). On
Android, OpenHarmony and ARM/RISC-V targets the engine itself spawns content
processes **unsandboxed** — any isolated-process containment must come from the
embedder's own process model — and on iOS there are no content processes at all
(multiprocess exits the process). On all of those `servo_create_sandbox()`
and `servo_capi_is_sandboxing_supported()` return `false`. Query
`servo_capi_is_sandboxing_supported()` at runtime rather than inferring it; the
per-platform table under **Platform notes** below is the summary.

### API summary

```c
/* Runs the content process event loop in a child that Servo spawned for your
 * executable. The export exists in every build; without the `multiprocess`
 * feature it refuses with -1. */
int servo_run_content_process(const char* token);

/* Creates and activates the sandbox. Returns true on success, false (and
 * applies mitigations only on Windows) when unsupported. Called automatically
 * by servo_run_content_process if sandbox is enabled. */
bool servo_create_sandbox(void);
```

### Complete example (browser process)

```c
/* browser.c - multi-process browser process */
#include <servo/servo_capi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Servo spawns this same executable for every content process, so the routing
 * belongs here rather than in a launcher of your own. Returns -1 when this
 * process is not a Servo-spawned content child. */
static int run_as_content_process(int argc, char** argv) {
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], "--content-process") == 0) {
            /* Never returns while the content process runs. */
            return servo_run_content_process(argv[i + 1]) == 0 ? 0 : 1;
        }
    }
    return -1;
}

int main(int argc, char** argv) {
    /* First statement: a child re-executes this binary with the token. */
    int content = run_as_content_process(argc, argv);
    if (content >= 0) return content;

    if (!servo_capi_is_multiprocess_enabled()) {
        fprintf(stderr, "servo-capi was built without the `multiprocess` feature\n");
        return 1;
    }

    ServoOptions* options = servo_options_create();
    servo_options_set_multiprocess(options, true);
    servo_options_set_sandbox(options, true);

    ServoPreferences* prefs = servo_preferences_create();
    servo_preferences_set_string(prefs, "user_agent", "MultiProcessServo/1.0");

    ServoBuilder* builder = servo_builder_create();
    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = on_wake,
    };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_set_options(builder, options);
    servo_builder_set_preferences(builder, prefs);

    struct Servo* servo = servo_builder_build(builder);
    servo_setup_logging(servo);

    /* There is deliberately no token here to collect and no child to launch:
     * whenever the constellation needs a content process it re-executes this
     * binary as `<exe> --content-process <token>`, which run_as_content_process()
     * above already handles. */

    /* Normal event loop */
    for (;;) {
        servo_spin_event_loop(servo);
        if (should_quit) break;
    }

    servo_free(servo);
    return 0;
}
```

### Complete example (content process)

There is no separate `content_process.c` to build: Servo re-executes **the same
binary**, so the routing at the top of `main` in the browser example *is* the
content-process entry point. Shown on its own for clarity:

```c
/* Same binary as the browser process. */
static int run_as_content_process(int argc, char** argv) {
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], "--content-process") == 0) {
            /* Connects to the one-shot server, receives Opts/prefs over IPC,
             * applies the sandbox if the browser process asked for one, then
             * runs the content event loop. Never returns on success. */
            return servo_run_content_process(argv[i + 1]) == 0 ? 0 : 1;
        }
    }
    return -1;
}
```

If you want to observe the sandbox step yourself (a log line, say), call
`servo_create_sandbox()` before `servo_run_content_process` — it must then be
the first thing in `main`, and its result is safe to ignore: the sandbox is
applied again inside with the options the browser process actually sent.

### Platform notes

- **Linux/macOS (x86_64)**: content processes are spawned already inside an OS
  sandbox profile — `gaol` on Linux, `sandbox-exec` on macOS — when
  `servo_options_set_sandbox(options, true)` is set. `servo_capi_is_sandboxing_supported()`
  returns `true` here.
- **Windows**: **crash isolation without OS containment.** `multiprocess` still
  gives you a separate content process (a crash there does not take down the
  browser process), but there is no broker, no token lockdown and no
  AppContainer. With `opts.sandbox` set, `servo_create_sandbox()` applies
  in-process mitigation policies (DEP, ASLR HighEntropy, strict handle check,
  extension-point disable) — a proof of concept, not a sandbox — and then
  returns `false`. The capability query returns `false` on Windows, so an
  embedder must not present Windows as sandboxed.
- **Android / OpenHarmony / ARM / RISC-V / Apple-Silicon macOS**: content
  processes are spawned unsandboxed (the engine launches `current_exe()` with no
  profile); `servo_capi_is_sandboxing_supported()` returns `false`. Any
  containment on those platforms has to come from the OS process model the
  embedder sets up — see `MULTI_PLATFORM_SANDBOX_DESIGN.md`, which is explicit
  that Servo enforces nothing there today.
- **iOS**: not supported at all: `spawn_multiprocess` logs and exits, so the
  feature must not be enabled on iOS.

### Limitations

- **No token API exists, and none is needed.** The token never reaches the
  embedder: Servo keeps it and passes it to the child it spawns. The earlier
  "wait for `servo_get_ipc_token`" placeholder is gone because the premise was
  wrong, not because the API is missing. The constraint that follows is that
  your process model must tolerate `current_exe()` being re-executed with
  `--content-process <token>`; if it cannot (a host binary that is not your
  embedder, a launcher that must not be re-run), there is no override and
  multiprocess is unusable through this API.
- Process lifecycle management (crash detection, restart, resource limits)
  is the embedder's responsibility. The browser process does wait on the
  children it spawned (`components/constellation/process_manager.rs`).
- No zygote/fork-server abstraction is provided; each content process is
  launched independently, on demand, as `current_exe()`.
- **Not exercised by this branch's CI.** `tests/capi` links a `servo-capi`
  built *without* the `multiprocess` feature, so only the refusal side of the
  contract can be asserted there
  (`test_multiprocess_single_process_path`): a NULL token, a non-NULL token, and
  `servo_create_sandbox()` all refuse, and `servo_capi_is_multiprocess_enabled()`
  reports `false`. The spawn side is documented above and can only be proven by
  an embedder running a real multiprocess build.
