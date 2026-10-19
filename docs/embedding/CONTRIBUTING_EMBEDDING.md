# Contributing to the Embedding C API

This guide explains how to extend the Servo C API safely. It is aimed at
contributors who need to expose a new capability from libservo (`components/servo`)
through the `servo-capi` crate (`ffi/capi`) to C and C++ embedders.

The C API is a hand-written `extern "C"` surface over Rust types; it is **not**
generated from the Rust API automatically. Every function is written by hand and
documented with a complete safety contract. Because embedders are ordinary C
programs that can pass any pointer to any function, correctness here is a memory
safety question, not just an API design question.

- [Repository layout](#repository-layout)
- [Before you start](#before-you-start)
- [Step-by-step recipe](#step-by-step-recipe)
- [Copy-paste template](#copy-paste-template)
- [How cbindgen picks up your types](#how-cbindgen-picks-up-your-types)
- [Tests](#tests)
- [Validation](#validation)
- [Conventions](#conventions)
- [Review checklist](#review-checklist)

## Repository layout

| Path | Purpose |
| --- | --- |
| `ffi/capi/lib.rs` | Crate root, module list, the `ServoBuilder`/waker types, and version constants. Add `mod <name>;` here for a new module. |
| `ffi/capi/servo.rs` | `Servo` lifecycle and event loop (`servo_free`, `servo_spin_event_loop`, `servo_setup_logging`). |
| `ffi/capi/options.rs` | `ServoOptions` handle and setters. |
| `ffi/capi/preferences.rs` | `ServoPreferences` handle and typed getters/setters. |
| `ffi/capi/rendering_context.rs` | `RenderingContext` handle. |
| `ffi/capi/webview.rs` | WebView builder, `WebView` methods, themes, and the buffer helper. |
| `ffi/capi/webview_delegate.rs` | The `ServoWebViewDelegate` struct and its `WebViewDelegate` impl. |
| `ffi/capi/javascript.rs` | `ServoJavaScriptCallback` and `servo_webview_evaluate_javascript`. |
| `ffi/capi/cbindgen.toml` | cbindgen configuration: exports, enum renaming, header banner. |
| `tests/capi/c/test_api.c` | Unit tests that only need handles, not a running Servo. |
| `tests/capi/c/test_integration.c` | Integration tests that build a `Servo` and spin the event loop. |
| `tests/capi/build.rs` | Runs `cargo cinstall` to build the cdylib and compiles the C tests. |

## Before you start

1. **Check whether it already exists.** Search the header surface and
   [CEF_PARITY.md](CEF_PARITY.md). Prefer to extend an existing handle over
   adding a new one.
2. **Prefer exposing libservo, not reimplementing it.** If `components/servo`
   lacks the capability, add it there first (with tests), then wrap it. Do not
   duplicate logic in `ffi/capi`.
3. **Decide the ownership and threading story before writing code.** For every
   new pointer parameter, ask: who owns it after the call, and on which thread
   may it be used?
4. **Keep the ABI addition-only where possible.** See
   [ABI_STABILITY.md](ABI_STABILITY.md). Adding a field to a `#[repr(C)]` struct
   that is constructed by value (like `ServoWebViewDelegate`) is a breaking change
   for positional initializers; document it as such in the release notes.

## Step-by-step recipe

### 1. Put the function in the right module

Choose the module that owns the handle the function operates on, and add the
function there. If you are adding a whole new area, create a new module (for
example `ffi/capi/javascript.rs`) and register it in `ffi/capi/lib.rs`:

```rust
// ffi/capi/lib.rs
mod javascript;
```

### 2. Use the correct ABI and naming

Every exported function uses the Rust 2024 "unsafe attributes" syntax and an
`extern "C"` ABI:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn servo_webview_do_thing(webview: *mut WebView) -> bool {
    // ...
}
```

Write `pub unsafe extern "C" fn` when the function takes any raw pointer and
therefore has a safety contract callers must uphold (which is almost all of
them). The `unsafe` on the function does not change the ABI; it signals that the
caller has obligations.

### 3. Follow the opaque-handle conventions

- Handles are opaque pointers to Rust structs or type aliases. Mark an opaque
  struct with the cbindgen comment on the line immediately above it:

  ```rust
  // cbindgen:opaque
  pub struct ServoWebViewBuilder { /* private fields */ }
  ```

- Prefer a Rust type alias over a wrapper when you are exposing an existing
  libservo type; mark it `// cbindgen:no-export` so cbindgen emits only a forward
  declaration:

  ```rust
  // cbindgen:no-export
  pub type ServoOptions = servo::Opts;
  ```

- Allocate handles with `Box::into_raw(Box::new(value))` and free them with
  `Box::from_raw`:

  ```rust
  pub extern "C" fn servo_widget_create() -> *mut ServoWidget {
      Box::into_raw(Box::new(ServoWidget::default()))
  }

  pub unsafe extern "C" fn servo_widget_free(widget: *mut ServoWidget) {
      assert!(!widget.is_null(), "widget pointer must not be null");
      // SAFETY: The caller guarantees `widget` came from `servo_widget_create`
      // and has not been freed.
      unsafe { let _ = Box::from_raw(widget); }
  }
  ```

- The API does not null the caller's pointer. Document that embedders should set
  freed handles to `NULL` themselves.

### 4. Write the full safety contract in the doc comment

Every `pub unsafe extern "C" fn` must have a `# Safety` section listing exactly
what the caller must guarantee, in terms of:

- the provenance of each handle (which `_create`/`_build` returned it);
- that it has not already been freed or transferred to another ownership-taking
  call;
- that no other code reads or writes the pointed-to object during the call;
- the required thread;
- the validity/lifetime of any `const char*`, buffer or callback;
- ownership transfer, if any.

Follow the wording already used in `ffi/capi/webview.rs` and
`ffi/capi/options.rs` so the contracts stay uniform.

### 5. Validate arguments, return codes, and never panic across FFI

- Check every pointer for null and fail fast. The existing code uses
  `assert!(!ptr.is_null(), "...")` for programmer errors. New code should prefer
  a documented return code (`-1`, `false`, `NULL`) over aborting when the failure
  is recoverable.
- Document return values precisely: `0`/`-1` for parse operations, `bool` for
  predicates, `NULL` for allocation failure, and the buffer-query convention for
  string getters (see [Conventions](#conventions)).
- **Panics must not cross the FFI boundary.** Since Rust 1.81 a panic that
  unwinds out of an `extern "C"` function aborts the process. If the wrapped
  libservo code can panic, catch it:

  ```rust
  use std::panic::{AssertUnwindSafe, catch_unwind};

  let result = catch_unwind(AssertUnwindSafe(|| webview.might_panic()));
  match result {
      Ok(value) => value,
      Err(_) => {
          log::error!("panic caught at the C API boundary");
          std::ptr::null_mut() // or a documented error code
      },
  }
  ```

  Do not `unwrap()`/`expect()` on embedder-controlled data inside an exported
  function unless the failure is a documented precondition violation and you
  intend to abort.

### 6. Respect the threading rule

All handles are created on, and must only be used from, the embedder thread that
built the `Servo`. This is exactly the contract already documented on
`ServoBuilder`, `RenderingContext`, `ServoWebViewBuilder` and the WebView
functions. New functions must:

- be called from that thread;
- not be callable from the `ServoEventLoopWaker` (which may run on a Servo
  thread);
- document the thread requirement in their `# Safety` section.

If a callback gives the embedder a temporary handle (as the delegate does), keep
the existing wording: the handle is only valid for the duration of the callback
and must not be passed to an ownership-taking function such as
`servo_webview_free`.

### 7. Handle ownership transfer explicitly

If a function consumes a handle, say so in the doc comment and in the
`# Safety` section, and use `Box::from_raw`, `take()` or move semantics so a
second use is impossible. Examples in the existing code:

- `servo_builder_set_options` / `servo_builder_set_preferences` take ownership of
  the options/preferences handle;
- `servo_builder_build` consumes the builder;
- `servo_webview_builder_create` takes ownership of the rendering context;
- `servo_webview_builder_build` consumes the builder.

If a function returns a newly allocated string, document which allocator owns it
(currently the C library `malloc`, freed with `free()`).

### 8. Make cbindgen pick up the type

See [How cbindgen picks up your types](#how-cbindgen-picks-up-your-types). In
short: reference the type in a public `extern "C"` signature, mark opaque structs
with `// cbindgen:opaque`, and add any new exported enum to
`ffi/capi/cbindgen.toml`.

### 9. Add tests

- **Unit test** in `tests/capi/c/test_api.c`: add a function that exercises the
  new entry point and register it in `run_c_api_tests()` with `RUN_TEST(name)`.
  Tests must return `0` on success.
- **Integration test** in `tests/capi/c/test_integration.c` when the behavior
  needs a live `Servo`/event loop. Return from `run_c_integration_tests`; it is
  invoked from Rust via the `extern "C"` declarations in `tests/capi/lib.rs` and
  the wrappers in `tests/capi/tests/`.
- Cover the failure paths: null pointers (if documented), invalid input,
  too-small buffers, and the buffer-query growth case.
- If the change is observable to embedders, update
  [C_API_GUIDE.md](C_API_GUIDE.md), [CEF_PARITY.md](CEF_PARITY.md) and, for
  compatibility-affecting changes, [ABI_STABILITY.md](ABI_STABILITY.md).

### 10. Run the checks

```sh
cargo fmt
./mach test-tidy
cargo test -p servo-capi-tests --locked
```

`cargo test -p servo-capi-tests` requires `cargo-c` to be installed; CI pins
`cargo-c@0.10.23` (see `.github/workflows/embedding-api.yml`, which also runs
`cbindgen --config ffi/capi/cbindgen.toml` afterwards to verify the header still
generates). The test crate's `build.rs` runs `cargo cinstall -p servo-capi` and
links the C tests against `-lservo_capi`.

## Copy-paste template

Use this as a starting point. Replace names and text; keep the shape.

```rust
/// One-sentence summary of what `servo_webview_do_thing` does.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `true` if ... and `false` otherwise.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and has not yet been freed nor passed to
///   another API that takes ownership of it. No other code may read or write
///   `*webview` for the duration of this call.
/// - The call is made from the same thread that originally created the
///   `WebView`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_do_thing(webview: *mut WebView) -> bool {
    // Fail fast on invalid arguments. Prefer a documented return code over a
    // panic for recoverable errors.
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller upholds the contract documented in `# Safety`.
    let webview = unsafe { &*webview };

    // Do the work, ideally delegating to an existing libservo method. Wrap
    // panicking code in `std::panic::catch_unwind` if it can panic on
    // embedder-controlled input.
    webview.do_thing()
}
```

For a **string getter**, use the buffer-query convention:

```rust
/// Writes the widget's label into `buffer` as a NUL terminated UTF-8 string.
///
/// Returns the number of bytes required to store the label including the
/// trailing NUL terminator. If `buffer` is `NULL` or `buffer_len` is too
/// small, nothing is written and the required length is returned.
///
/// # Safety
///
/// - `widget` must be a non-null pointer to a `ServoWidget` previously
///   returned by `servo_widget_create` and not yet freed.
/// - If `buffer` is non-null it must point to at least `buffer_len` writable
///   bytes.
/// - The call must be made from the thread that created the widget.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_widget_get_label(
    widget: *const ServoWidget,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    assert!(!widget.is_null(), "widget pointer must not be null");

    // SAFETY: The caller upholds the contract documented in `# Safety`.
    let widget = unsafe { &*widget };
    let label = widget.label();

    // SAFETY: The caller guarantees `buffer` is valid for writes of
    // `buffer_len` bytes when non-null.
    unsafe { write_c_string(&label, buffer, buffer_len) }
}
```

The `write_c_string` helper in `ffi/capi/webview.rs` implements the shared
return-value and NUL-termination logic; reuse it (make it `pub(crate)`) instead
of writing a new one.

## How cbindgen picks up your types

`cargo cinstall -p servo-capi` runs [cbindgen](https://github.com/eqrion/cbindgen)
with `ffi/capi/cbindgen.toml` and writes the header to
`<prefix>/include/servo/servo_capi.h`.

- **Referenced types are emitted automatically.** Any type that appears in the
  signature of an exported function gets a declaration. You do not normally edit
  the generated header.
- **Opaque structs**: annotate with `// cbindgen:opaque` immediately above the
  struct so only a forward declaration is generated. Without it, cbindgen tries
  to generate a struct definition from the (private) fields.
- **Type aliases** that should stay opaque: annotate with
  `// cbindgen:no-export`.
- **Structs passed by value** (for example `ServoWebViewDelegate`,
  `ServoEventLoopWaker`) must be `#[repr(C)]` and have public fields.
- **Enums**: the `[export] include` list in `cbindgen.toml` names the enums to
  expose, and `[export.rename]` gives them C-friendly names such as
  `ServoLoadStatus`. `rename_variants = "ScreamingSnakeCase"` and
  `prefix_with_name = true` produce constants like
  `SERVO_LOAD_STATUS_STARTED`. To export a new enum, add it to the include list
  (and rename it if the Rust name is ambiguous).
- **Constants**: a `pub const NAME: c_int = ...;` (such as the `SERVO_THEME_*`
  values) is emitted as a define/constant. Use `SERVO_`-prefixed,
  `SCREAMING_SNAKE_CASE` names.
- **Function pointer typedefs** are generated for callback types such as
  `ServoJavaScriptCallback`; keep the Rust type a plain `Option<unsafe extern "C" fn(...)>`
  so the C typedef is a nullable function pointer.

To iterate quickly, install cbindgen yourself and regenerate without a full
`cinstall`. The exact invocation is whatever `cargo-c` uses; the config lives in
`ffi/capi/cbindgen.toml` and the crate's entry point is `ffi/capi/lib.rs`.

## Tests

The C tests are compiled and linked by `tests/capi/build.rs`, which:

1. runs `cargo cinstall -p servo-capi --library-type cdylib` into the build's
   `OUT_DIR`;
2. links the test objects against `-lservo_capi`;
3. on Windows copies `servo_capi.dll` (and the ANGLE DLLs) next to the test
   binary.

Two entry points are exposed to Rust and called from test wrappers:

- `run_c_api_tests()` from `tests/capi/c/test_api.c`, wrapped by
  `tests/capi/tests/c_api_tests.rs`;
- `run_c_integration_tests()` from `tests/capi/c/test_integration.c`, wrapped by
  `tests/capi/tests/c_integration_tests.rs`.

When adding a test:

1. Add a `static int test_something(void)` that returns `0` on success. Use the
   `RUN_TEST(test_something);` macro pattern in `run_c_api_tests()`.
2. For anything needing a live engine, extend `test_integration.c` and drive it
   from `run_c_integration_tests`.
3. `Servo` initializes global state once per process; the tests create a single
   shared instance (`create_test_servo`) and free it at the end. Do not build a
   second `Servo` in the same process.
4. Include the header as `#include "servo/servo_capi.h"` (the test include path
   is `<OUT_DIR>/include`), matching the installed layout.
5. Compile as C11; the `build.rs` passes `.std("c11")`.

### Leak testing

`test_integration.c::test_repeated_lifecycle_leaks` is the long-uptime test: it runs
three phases of 300 create/close cycles each (100 warm-up, 200 measured) and asserts
that RSS growth across the measured window stays under `LEAK_TEST_MAX_GROWTH_BYTES`.
The phases exist to localize growth rather than just to detect it:

| Phase | What one cycle does | Isolates |
|---|---|---|
| `LEAK_PHASE_RENDERING_CONTEXT` | create + free a software rendering context | surfman / GL teardown |
| `LEAK_PHASE_WEBVIEW_NO_LOAD` | build a WebView, close it, free it | WebView + painter teardown |
| `LEAK_PHASE_FULL` | as above, plus load `about:blank` and wait for load complete | the navigation path |

Read the `[leak]` lines in the test output: each phase logs the warm-up and final RSS,
the growth in KiB, whether that exceeded the bound, and the growth rate in KiB/cycle
next to the rate the bound allows. The bound is deliberately not zero — the measured
lifecycle is not leak-free, and the reasons are recorded rather than papered over:

- **Windows-only rendering-context growth** (~272 KiB/cycle, linear). Surfman's
  `SurfmanRenderingContext::drop` destroys the surfman context but nothing destroys
  the `Device`, so its GL resources are retained. This is in crates.io `surfman`,
  upstream of this tree, so it is registered as an upstream issue rather than patched
  here.
- **WebView/Painter create-close growth** on all three platforms (in-use heap grows
  ~16 KiB/cycle on Linux; RSS grows more on macOS and Windows). Page load adds little
  on top — the full phase tracks the no-load phase — so the growth is in teardown, not
  navigation.

Two measurement caveats that the code also documents:

- **RSS is reported raw, not trimmed.** WebView teardown is asynchronous, so any
  per-cycle sample catches in-flight churn as well as retention. A per-step heap
  diagnostic was tried and removed: its deltas summed to -490,317 KiB against a true
  net of +16,141 KiB, because point samples caught the allocator mid-flight. Only
  phase-level net growth is meaningful.
- **The in-use heap figure is a diagnostic only.** It comes from glibc `mallinfo2()`
  and is unavailable on macOS and Windows, where the test falls back to RSS.

`LEAK_TEST_MAX_GROWTH_BYTES` is 128 MiB over the 200-cycle measured window, about
1.55x the worst growth observed when it was set (Windows, no-load phase, 84,568 KiB).
That headroom absorbs the roughly 11,000 KiB run-to-run swing seen on Windows while
still failing on a regression of about 1.5x. If you change the cycle count, rescale
the bound from the printed KiB/cycle rate rather than keeping the absolute value.

## Validation

Before opening a pull request, run all of:

```sh
cargo fmt
./mach test-tidy
cargo test -p servo-capi-tests --locked
```

`cargo fmt` formats the Rust wrapper. `./mach test-tidy` enforces Servo's
repository-wide style and license headers (including the MPL header comment at
the top of every `ffi/capi/*.rs` and `tests/capi/c/*.c` file). The C test run
builds the real cdylib through `cargo-c` and exercises the API from C, which is
the only way to catch ABI, header and linking mistakes.

If you changed the exported surface, also make sure the header generated in CI
compiles: adding an unexported or malformed type will fail the `cinstall` step in
`tests/capi/build.rs`.

## Conventions

- **Names**: exported functions are `servo_`-prefixed `snake_case`:
  `servo_webview_get_url`. Handles are `Servo`-prefixed in Rust and appear in C
  as the generated names (`ServoBuilder`, `ServoWebViewBuilder`,
  `RenderingContext`, `WebView`, `Servo`).
- **Constants/enums**: `SERVO_`-prefixed `SCREAMING_SNAKE_CASE`
  (`SERVO_LOAD_STATUS_COMPLETE`, `SERVO_THEME_DARK`,
  `SERVO_SCREENSHOT_CAPTURE_ERROR_*`).
- **Module naming**: one module per area, named after the handle or feature
  (`webview.rs`, `javascript.rs`, `preferences.rs`).
- **ABI**: `#[unsafe(no_mangle)] pub extern "C" fn`; `#[repr(C)]` for every
  struct passed across the boundary; `Option<unsafe extern "C" fn(...)>` for
  nullable callbacks.
- **Buffer-query pattern**: getters that return a string take
  `(const T* handle, char* buffer, size_t buffer_len)` and return the required
  size **including the NUL terminator**; they write nothing when the buffer is
  `NULL` or too small. See `servo_webview_get_url`.
- **Return codes**: `0` success / `-1` failure for parse-style operations; `bool`
  for predicates; `NULL` for failed allocations; opaque handles otherwise.
- **No panics across FFI**: never let a panic unwind out of an exported
  function; use return codes or `std::panic::catch_unwind`.
- **Documentation**: every exported function has a summary, parameter ownership
  notes, return value documentation, and a `# Safety` section.
- **MPL header**: keep the two-line license comment at the top of every new
  Rust and C file.
- **Comments**: `// SAFETY:` immediately before every `unsafe` block explains
  which caller guarantees make it sound.

## Review checklist

- [ ] Function is in the correct `ffi/capi/*.rs` module and, if new, the module
      is declared in `lib.rs`.
- [ ] `#[unsafe(no_mangle)] pub extern "C"` (plus `unsafe` when it takes raw
      pointers); opaque structs marked `// cbindgen:opaque`; value structs
      `#[repr(C)]`.
- [ ] Full `# Safety` section documents handle provenance, lifetime, ownership
      transfer, and the required thread.
- [ ] All pointer arguments null-checked; recoverable errors return a documented
      value; no `unwrap`/`expect` on embedder input; no panic can cross the FFI
      boundary.
- [ ] Ownership listed: which handle is borrowed vs consumed, and who frees what.
- [ ] New types/enums referenced correctly and exported via
      `ffi/capi/cbindgen.toml` if needed.
- [ ] Unit test in `tests/capi/c/test_api.c` and/or integration test in
      `tests/capi/c/test_integration.c`, registered in the appropriate runner.
- [ ] Docs updated: `C_API_GUIDE.md`, and `CEF_PARITY.md` /
      `ABI_STABILITY.md` when relevant.
- [ ] `cargo fmt`, `./mach test-tidy`, and
      `cargo test -p servo-capi-tests --locked` all pass.
