# Embedding Servo

This directory collects the upstream documentation for embedders who want to use
Servo as a library rather than as a browser application. It covers the stable-ish
C ABI exposed by `servo-capi`, the raw Rust embedding API in `components/servo`,
the compatibility policy for both, and the process for changing what embedders
can depend on.

If you are looking for something specific, jump to the [start-here paths](#start-here)
below.

## Architecture in five lines

1. `components/servo` (**libservo**) is the Rust embedding API: `ServoBuilder`,
   `Servo`, `WebViewBuilder`, `WebView`, `RenderingContext`, preferences and the
   `WebViewDelegate` notifications.
2. `ffi/capi` (**servo-capi**) is a thin `cdylib` wrapper that re-exports a
   hand-written, `extern "C"` subset of libservo as `#[unsafe(no_mangle)]`
   functions and opaque handles.
3. `cbindgen` reads the Rust sources plus `ffi/capi/cbindgen.toml` and generates
   the C header `servo/servo_capi.h`.
4. An embedder links `-lservo_capi`, includes that header, and drives a
   single-threaded event loop by repeatedly calling `servo_spin_event_loop`; Servo
   wakes the loop through a `ServoEventLoopWaker` callback.
5. `ports/servoshell` and `components/servo/examples/winit_minimal.rs` are the
   reference embedders, and `tests/capi` exercises the C ABI from C.

## Documentation index

| Document | What it covers |
| --- | --- |
| [PRODUCTION_REPORT.md](../../PRODUCTION_REPORT.md) | Top-level report: the production-readiness verdict with measured evidence, and the complete prioritised work list across conformance, security, platforms and distribution. |
| [AUDIT.md](AUDIT.md) | Audit of the current C API surface versus what embedders need. |
| [CEF_PARITY.md](CEF_PARITY.md) | Mapping between the Servo C API and the CEF (Chromium Embedded Framework) API, and the gaps between them. |
| [ARCHITECTURE.md](ARCHITECTURE.md) | How `servo-capi`, libservo, `servoshell` and the event-loop model fit together. |
| [ROADMAP.md](ROADMAP.md) | Planned work to grow the embedding API toward feature parity. |
| [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) | Honest assessment of whether Servo is production ready / Blink-CEF-class, platform and security posture, and the realistic path forward. |
| [ABI_STABILITY.md](ABI_STABILITY.md) | Versioning and source/ABI compatibility guarantees for `servo-capi`. |
| [COMPATIBILITY.md](COMPATIBILITY.md) | Honest platform/feature/symbol matrix: what works on which platform and feature configuration. |
| [C_API_GUIDE.md](C_API_GUIDE.md) | Practical, task-oriented guide for C/C++ embedders, including a complete example. |
| [SECURITY_AUDIT.md](SECURITY_AUDIT.md) | Security & robustness audit of the engine and the embedding boundary, with file/line evidence and priorities. |
| [HARDENING_PLAN.md](HARDENING_PLAN.md) | Concrete, bounded designs for the stability work (hang monitor, reachable panics, fuzzing) and FFI-boundary hardening. |
| [IOS_PORT_SPIKE.md](IOS_PORT_SPIKE.md) | Feasibility spike and deliberate decision on an iOS port: what is missing and what the go/no-go experiment is. |
| [CONTRIBUTING_EMBEDDING.md](CONTRIBUTING_EMBEDDING.md) | How to add or change C API functions safely and get them reviewed and tested. |

## Start here

### C / C++ embedders

1. Read [C_API_GUIDE.md](C_API_GUIDE.md) for the end-to-end workflow: building and
   linking, the threading rule, the ownership rules, and a complete single-file
   C11 example.
2. Skim [ABI_STABILITY.md](ABI_STABILITY.md) to understand what you can rely on
   across releases.
3. If you need a function that does not exist yet, read
   [CONTRIBUTING_EMBEDDING.md](CONTRIBUTING_EMBEDDING.md) and
   [CEF_PARITY.md](CEF_PARITY.md), then send a patch.

The header is installed as `<prefix>/include/servo/servo_capi.h` and is the
authoritative surface. The functions below are the ones you will spend most of
your time with:

- `servo_builder_create` / `servo_builder_build` / `servo_builder_free`
- `servo_options_*` and `servo_preferences_*`
- `servo_rendering_context_create_software` / `servo_rendering_context_free`
- `servo_webview_builder_create` / `servo_webview_builder_build` /
  `servo_webview_builder_free`
- `servo_webview_load` / `servo_webview_paint` /
  `servo_webview_take_screenshot` / `servo_webview_free`
- `servo_spin_event_loop`, `servo_setup_logging`, `servo_free`

A **runnable, source-tree-independent** version of that example lives in
[`examples/c-embedder/`](../../examples/c-embedder) (CMake + `cc` + README). It
links only against an installed `servo-capi` sysroot — the way a real third-party
embedder does — and is built by the prebuilt-artifact workflow
`.github/workflows/capi-artifacts.yml`.

### Rust embedders

Use libservo directly instead of going through the C ABI:

- Start with [`components/servo/examples/winit_minimal.rs`](../../components/servo/examples/winit_minimal.rs),
  a minimal `winit`-based embedder that builds a `Servo`, creates a
  `WindowRenderingContext`, opens a `WebView` and pumps the event loop.
- Then read the API docs at <https://doc.servo.org/servo/> and
  [ARCHITECTURE.md](ARCHITECTURE.md).
- The C API is a subset of this API; if something is missing from the C API but
  present in Rust, see [CONTRIBUTING_EMBEDDING.md](CONTRIBUTING_EMBEDDING.md).

## Where these docs live

These pages are maintained **upstream in the Servo repository**, under
`docs/embedding/`. They are the source of truth for any downstream copy.

The Servo project also publishes a separate, curated book at
<https://book.servo.org>. Several older pages in `docs/` (for example
`docs/STYLE_GUIDE.md`) are now stubs that redirect to the book. These embedding
docs are intentionally different: they are versioned alongside the code so that a
given branch's documentation matches that branch's C API. If and when they are
included in the book, the repository copy remains authoritative for embedders
building from source.
