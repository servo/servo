# Servo C API compatibility matrix

This page records, honestly, **what works where** for the `servo-capi` C ABI: the
platforms that CI actually builds and tests, the runtime capability queries, the
feature flags that change behaviour, and the things that are not available yet.

It is the companion to [`ABI_STABILITY.md`](ABI_STABILITY.md) (what you may rely on
across releases) and [`C_API_GUIDE.md`](C_API_GUIDE.md) (how to use the API). The
governing rule is [ROADMAP.md](ROADMAP.md) cross-cutting rule 6: document which
functions work with which rendering contexts and features rather than silently
no-op'ing.

## 1. Supported platforms

A platform is "supported" only when CI builds `servo-capi` with `cargo cinstall`,
runs the C integration tests in `tests/capi`, and passes the symbol allowlist check
(see [`ABI_STABILITY.md`](ABI_STABILITY.md) §7.1).

The always-on check is `.github/workflows/embedding-api.yml`, which runs on every
push/PR that touches `ffi/capi/**`, `tests/capi/**`, or `docs/embedding/**`:

| OS | Architecture | How it is verified |
| --- | --- | --- |
| Linux | x86_64 | `embedding-api.yml` (always on). |
| macOS | arm64 (Apple Silicon) | `embedding-api.yml` (always on). |
| Windows | x86_64 | `embedding-api.yml` (always on). |
| Linux | arm64 | `linux-arm64.yml` `capi` job (run on dispatch/release, `capi: true`). |
| macOS | x86_64 (Intel) | `mac.yml` `capi` job (run on dispatch/release, `capi: true`). |

Sandbox availability below follows from the target, not from which workflow built it.

Other targets (`android`, `ios`, `ohos`, 32-bit ARM, RISC-V) are named by
`servo_capi_platform_name()` so a build can exist, but they are **not** built or
tested by the embedding CI and are **not** supported. In particular, **iOS is explicitly unsupported**: there is no iOS port, no Xcode project, no iOS CI, sandboxing panics by design, and JIT/UIKit/Metal embedding are not implemented (see [`IOS_PORT_SPIKE.md`](IOS_PORT_SPIKE.md) for details). Do not assume a capability is
present because the build succeeded — query it (below).

## 2. Symbol surface

Every supported platform exports the **same** set of `servo_*` functions from the
same cbindgen-generated header. There is no platform-conditional symbol set: if a
function is not available on your platform it is absent everywhere or returns an
error, it does not appear only on some platforms.

CI enforces this: `.github/workflows/embedding-api.yml` extracts the `servo_*`
symbols from the freshly generated header and `diff`s them against the checked-in
allowlist at [`ffi/capi/symbols.txt`](../../ffi/capi/symbols.txt). Adding, removing,
or renaming a public symbol fails the build until the allowlist is updated in the
same change.

The header is generated with `cpp_compat = true`, so the same header can be
`#include`d from both C and C++ (see
[`C_API_GUIDE.md`](C_API_GUIDE.md) "Using the API from C++").

## 3. Feature flags

`servo-capi` currently exposes one behaviour-changing Cargo feature:

| Feature | Default | Effect | Query |
| --- | --- | --- | --- |
| `multiprocess` | off | Runs web content (JS/DOM/layout) in separate content processes, isolated from the browser process that hosts your event loop. | `servo_capi_is_multiprocess_enabled()` |

Enabling `multiprocess` also requires `servo_options_set_multiprocess(options, true)`
at runtime and, typically, `servo_run_content_process` from spawned child processes.
See the multi-process section of [`C_API_GUIDE.md`](C_API_GUIDE.md).

## 4. Runtime capability queries

These functions are safe to call before any `Servo` instance exists and report what
the *loaded* library can do:

| Function | Returns |
| --- | --- |
| `servo_capi_version()` | The `"major.minor.patch"` version string of the loaded library. |
| `servo_capi_get_version()` | The packed integer `(MAJOR << 16) \| (MINOR << 8) \| PATCH`; compare against the `SERVO_CAPI_VERSION` macro to detect header/library skew. |
| `servo_capi_platform_name()` | `"linux"`, `"macos"`, `"windows"`, `"android"`, `"ios"`, `"ohos"`, or `"unknown"`. |
| `servo_capi_is_multiprocess_enabled()` | Whether the library was built with the `multiprocess` feature. |
| `servo_capi_is_sandboxing_supported()` | Whether the content-process sandbox can actually be created here. |

## 5. Sandboxing availability

`servo_capi_is_sandboxing_supported()` returns `true` only when the library was built
with `multiprocess` **and** the target supports sandboxing. This is a real security
property: on the platforms marked "no" below, content runs without renderer
sandboxing. Design your threat model accordingly.

| Platform / arch | `multiprocess` off | `multiprocess` on |
| --- | --- | --- |
| Linux x86_64 | no | **yes** |
| macOS x86_64 | no | **yes** |
| Linux arm64 | no | no |
| macOS arm64 | no | no |
| Windows x86_64 | no | no |
| iOS / Android / OpenHarmony / ARM / RISC-V | no | no |

## 6. Rendering contexts

A `RenderingContext` is required to build a `WebView`. The three constructors differ
in platform requirements:

| Constructor | Availability | Failure mode |
| --- | --- | --- |
| `servo_rendering_context_create_software(width, height)` | All platforms, including headless CI. | Returns `NULL` for a zero-sized request or allocation failure. |
| `servo_rendering_context_create_window(display_handle, window_handle, width, height)` | Requires a real native display and window (e.g. from winit/SDL/GLFW). | Returns `NULL` when there is no display (headless), the handles are invalid, the size is zero, or GPU context creation fails. |
| `servo_rendering_context_create_offscreen(parent, width, height)` | GPU (OSR / FBO composition); `parent` must be a **window** context on the same thread. | Returns `NULL` for a null or non-window parent, zero size, or FBO creation failure. |

`servo_webview_paint` composites the `WebView` through its rendering context;
`servo_webview_take_screenshot` captures the current frame. Both require a live
`WebView` and its context.

## 7. Threading model

The API is **single-threaded from the embedder's point of view**. Every handle is
affine to the thread that created it (normally your main thread):

- Create, use, and free each handle on that one thread.
- Servo calls your delegate callbacks and the `ServoEventLoopWaker` on that thread,
  while it is inside `servo_spin_event_loop`.
- Servo itself uses internal threads, but it never calls your code from them.

There are currently **no thread-safe / `Send` handles**. The underlying objects are
`Rc`-based, so moving a handle to another thread is unsupported. See
[`ABI_STABILITY.md`](ABI_STABILITY.md) §5 for the per-function ownership/threading
template.

## 8. Not available yet

The following are known gaps (tracked in [`ROADMAP.md`](ROADMAP.md) Phase 5), not
hidden behaviours:

- Prebuilt binaries and `pkg-config`/CMake packages per platform.
- Versioned release tarballs/installers, checksums, and a documented release process.
- Community language bindings (Python/Rust/etc.). A header-only C++ RAII wrapper is
  available in the source tree (`ffi/capi/servo_capi.hpp`, see
  [`C_API_GUIDE.md`](C_API_GUIDE.md) "Using the API from C++"), but it is not yet
  installed by `cargo cinstall`.
- Thread-safe (`Send`) handles.
- Windows and AArch64 renderer sandboxing.
- A deprecation carried through a release (the policy and the `SERVO_DEPRECATED`
  macro exist; `servo_webview_evaluate_javascript` is now deprecated in favor of
  `servo_webview_evaluate_javascript_v2`).

Until the binary/package items land, embedders build `servo-capi` from source with
`cargo cinstall` (see [`C_API_GUIDE.md`](C_API_GUIDE.md) "Building and linking").
