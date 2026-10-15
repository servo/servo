# Servo C ABI Stability and Compatibility Policy

This document defines the compatibility contract for the C API exported by the
`servo-capi` crate (`ffi/capi`, a `cdylib`) and generated into `servo/servo_capi.h` by
cbindgen. It is a policy plus copy-pasteable templates; implementation follows in the
phases described in [`ROADMAP.md`](./ROADMAP.md).

## 1. Scope: the C ABI is the compatibility boundary

- **The C ABI is the only stable interface.** Once a function, type, or constant is
  released under this policy, its behavior and signature are governed here.
- **The Rust API is not stable.** `components/servo` (`Servo`, `WebView`,
  `WebViewDelegate`, etc.) may change freely, including breaking changes, without a major
  bump of the C API. Embedders that use Rust directly opt out of this policy.
- **`ffi/capi` internals are not stable.** Module layout, private helpers, and the
  in-process ABI of the Rust types behind opaque handles may change at any time.
- **The generated header is part of the ABI.** `servo/servo_capi.h` is generated from
  `ffi/capi/cbindgen.toml` plus the `// cbindgen:*` annotations in `ffi/capi/*.rs`. Any
  change that alters the emitted declarations is an ABI change under this policy.
- **Cross-platform guarantee.** The ABI is guaranteed per (OS, architecture, build
  configuration) triple. "Supported" means: Linux x86_64/aarch64, macOS x86_64/arm64,
  Windows x86_64, with the default feature set of `servo-capi`. Other targets and custom
  feature combinations are best-effort unless stated otherwise.

## 2. Versioning

### 2.1 Version macros and runtime query

The header must define the C API version. The C API version is managed
independently of the workspace crate version (`0.7.0`); the head commit on this
branch introduces version `0.1.0` as a starting point:

```c
/* servo/version.h (generated or hand-maintained), included by servo/servo_capi.h */
#define SERVO_CAPI_VERSION_MAJOR 0
#define SERVO_CAPI_VERSION_MINOR 1
#define SERVO_CAPI_VERSION_PATCH 0
#define SERVO_CAPI_VERSION \
    ((SERVO_CAPI_VERSION_MAJOR << 16) | \
     (SERVO_CAPI_VERSION_MINOR <<  8) | \
     (SERVO_CAPI_VERSION_PATCH))
```

Plus a runtime query so hosts can detect a mismatched library at load time.
Today the query returns the release version string:

```c
/* Returns the release version string of the loaded library, e.g. "0.1.0". */
const char* servo_capi_version(void);
```

A packed integer query is also provided for cheap runtime comparisons:

```c
/* Returns the SERVO_CAPI_VERSION the loaded library was built with. */
uint32_t servo_capi_get_version(void);
```

`servo_capi_get_version()` returns `(MAJOR << 16) | (MINOR << 8) | PATCH`, matching the
`SERVO_CAPI_VERSION` constant (`ffi/capi/lib.rs`). Embedders that need the string can
still compare `servo_capi_version()` against the macros.

The Rust side obtains the values from `env!("CARGO_PKG_VERSION")` / build metadata so the
header and library cannot drift silently.

### 2.2 Semantic versioning rules

- **MAJOR** — incompatible ABI or behavioral change: removing/renaming a symbol, changing
  a signature, changing a struct field's type/order, changing ownership or thread-affinity
  semantics of an existing function, removing an enum constant, or changing default
  behavior in a way that breaks correct callers.
- **MINOR** — backward-compatible additions: new functions, new opaque handles, new enum
  constants **appended** to the end, new optional fields added via the `struct_size`
  mechanism, new callbacks added at the end of a delegate struct.
- **PATCH** — bug fixes and documentation that do not change the ABI or observable
  contract.

A frequent mistake to avoid: adding a field to a `#[repr(C)]` struct is **breaking** for
existing callers unless the struct is versioned (§3.3). Treat every layout change as
MAJOR unless it uses the extension mechanism.

### 2.3 What counts as a breaking change (checklist)

Breaking (requires MAJOR):

- `#[unsafe(no_mangle)] pub extern "C" fn` removed or renamed.
- Parameter or return type changed (including `T*` vs `T` and `int32_t` vs `size_t`).
- A function that used to accept null now rejects it, or vice versa, in a way that breaks
  correct callers.
- Ownership transfer direction changed for any pointer argument.
- A `#[repr(C)]` struct's field order/type/size changed, or a field added without
  `struct_size`.
- An enum's discriminants reassigned, or a variant removed/reordered.
- Callback struct field order changed, or a callback's documented invocation thread
  changed.
- Default behavior changed such that an existing correct caller now observes different
  results (e.g. navigation now auto-allows where it previously denied).

Not breaking (MINOR/PATCH):

- Adding a new function or new opaque type.
- Appending an enum constant and updating all `switch` statements to tolerate unknowns.
- Appending a new optional struct field via `struct_size` while keeping existing defaults.
- Appending a new callback pointer to a delegate struct, with null meaning "no callback".
- Documentation, logging, or performance changes.

## 3. Symbols, handles, structs, enums

### 3.1 Symbol naming and prefixing

- All exported symbols use the `servo_` prefix and `snake_case`:
  `servo_<object>_<verb>[_<detail>]`, e.g. `servo_webview_load`,
  `servo_builder_set_options`, `servo_rendering_context_create_software`.
- Constructors are `servo_<object>_create*`; destructors are `servo_<object>_free`.
  "Consuming" transitions keep the builder name (e.g. `servo_webview_builder_build`).
- Every exported Rust function is `#[unsafe(no_mangle)] pub extern "C" fn` and takes
  pointers, integers, or `#[repr(C)]` values only. Never expose Rust `String`, `Vec`,
  slices, `bool`-in-struct (use `uint8_t`), or `enum`-without-`repr(C)`.
- Maintain a symbol allowlist checked in CI (`nm`/`llvm-nm` on the built cdylib) so
  accidental exports, renames, or removals fail the build.
- Keep the C identifier length portable; avoid names that risk Windows/decorated-name
  limits.

### 3.2 Opaque handles

- Handles are opaque, forward-declared structs marked `// cbindgen:opaque` in Rust
  (`ServoBuilder`, `ServoOptions`, `ServoPreferences`, `RenderingContext`,
  `ServoWebViewBuilder`, `Servo`, `WebView`). The embedder must never dereference them,
  size them, or allocate them.
- Handles are single-owner `Box` allocations created by `_create`/`_build` and released by
  `_free` or consumed by exactly one `_build`/`set_*` function. The documentation for each
  function states which of the two applies.
- Handles are **not thread-safe** unless explicitly documented. Today all handles are
  thread-affine to the thread that created them; see §5.
- Do not expose the Rust struct layout. If a handle needs to be inspected, add accessor
  functions.

### 3.3 Struct layout extension: `struct_size`

Any `#[repr(C)]` struct that the embedder fills in (options-like inputs and delegate
structs) must be versioned. The first field is the struct size in bytes:

```c
typedef struct ServoWebViewDelegate {
    /* Must be set to sizeof(ServoWebViewDelegate) by the caller. */
    uint32_t struct_size;
    uint32_t _reserved;          /* 0; future flags */
    void*    user_data;
    /* callbacks appended over time; null == not interested */
    void (*notify_load_status_changed)(struct WebView*, int32_t, void*);
    void (*notify_new_frame_ready)(struct WebView*, void*);
    /* MINOR additions go here, never above existing fields */
} ServoWebViewDelegate;
```

Rust entry points read `struct_size` and reject a value smaller than the size of the
compiled-in Rust type with `SERVO_ERROR_INVALID_ARGUMENT`.

The delegate-like inputs in this API are passed **by value**, so the ABI copies exactly
`sizeof(callee's type)` bytes; a caller built against an older, smaller header cannot be
read "within `struct_size`" safely. Requiring `struct_size >= size_of::<T>()` converts
that out-of-bounds read into a clean error, while still accepting a larger `struct_size`
(a caller built against a newer header), since Servo only reads the fields it was
compiled with. This is implemented by `is_compatible_struct_size()` (`ffi/capi/lib.rs`)
and enforced in `servo_webview_builder_set_delegate`, `servo_builder_set_delegate`, and
`servo_builder_set_event_loop_waker`.

Callers must initialize `struct_size` to `sizeof(...)`; a zero-initialized delegate is
rejected. When a field is appended in a later MINOR release, existing callers must
recompile and will then pass the new size automatically.

For structs passed **by pointer** (introduced later), read only the fields within
`struct_size` using `std::ptr::read_unaligned` into a prefix/fallback so older callers
keep working without recompiling.

For Rust-owned output structs that grow over time, use reserved fields instead:

```c
typedef struct ServoLoadInfo {
    uint32_t struct_size;   /* filled by Servo with sizeof(struct) */
    uint32_t _reserved;
    int32_t  load_status;
    /* add fields only at the end; bump struct_size in the same release */
} ServoLoadInfo;
```

`ServoScreenGeometry` (`ffi/capi/webview_delegate.rs`) follows this pattern: Servo fills
in `struct_size` before invoking the `screen_geometry` callback, so embedders can learn
how many fields the loaded library provides. Because it is passed **by pointer**, an
embedder built against an older header safely writes only the fields it knows about; the
remaining fields keep their zeroed defaults.

### 3.4 Enum extensibility

- Exported enums use fixed-width underlying types (`#[repr(i32)]`, e.g.
  `LoadStatus`, `ScreenshotCaptureError`) and cbindgen's `ScreamingSnakeCase` +
  `prefix_with_name` (`ffi/capi/cbindgen.toml`), producing `SERVO_LOAD_STATUS_*` etc.
- **Append-only.** New variants may only be added at the end in a MINOR release. Never
  remove, reorder, or reassign existing discriminants.
- **Unknown-value handling is mandatory.** Embedders must tolerate unknown values in
  callbacks (log/ignore, do not `abort()`). Document this in the header. The C tests
  should include a deliberately-unknown enum value.
- Prefer an explicit `_UNKNOWN = 0` sentinel when zero is not otherwise meaningful.

## 4. Error and panic policy

### 4.1 No unwinding across FFI

A panic must never cross an `extern "C"` boundary. In modern Rust this aborts the process
(unwinding out of `extern "C"` is not permitted); either way it is fatal to the host and
is forbidden by this policy.

**Every** `extern "C"` function that can panic must be wrapped:

```rust
fn catch_panic<F: FnOnce() -> T + std::panic::UnwindSafe, T>(
    default: T,
    f: F,
) -> T {
    match std::panic::catch_unwind(f) {
        Ok(value) => value,
        Err(_) => default,
    }
}
```

Then at the boundary:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn servo_webview_set_page_zoom(
    webview: *mut WebView,
    zoom: f32,
) -> ServoError { /* int32_t */ }
{
    if webview.is_null() {
        return ServoError::NullArgument;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: non-null checked above; caller upholds the handle contract.
        let webview = unsafe { &*webview };
        webview.set_page_zoom(zoom);
        ServoError::Ok
    }));
    result.unwrap_or(ServoError::InternalPanic)
}
```

Notes:

- Do not rely on `panic = "abort"`; the library must also behave if a host links it into
  an unwinding binary.
- If a panic hook is installed that itself panics or long-jumps, the boundary is
  compromised. Prefer `catch_unwind` plus a documented `SERVO_ERROR_INTERNAL`.
- Callbacks invoked *from* Rust into C (`extern "C" fn` provided by the embedder) must
  themselves not unwind; document this on every callback field (as the current docs do).

### 4.2 Error codes and result conventions

Adopt a single status enum, exported to C:

```c
typedef enum ServoError {
    SERVO_ERROR_OK = 0,
    SERVO_ERROR_NULL_ARGUMENT = -1,
    SERVO_ERROR_INVALID_UTF8 = -2,      /* bad UTF-8 */
    SERVO_ERROR_INVALID_STATE = -3,     /* e.g. webview shutting down, builder consumed */
    SERVO_ERROR_NOT_FOUND = -4,
    SERVO_ERROR_CAPACITY = -5,          /* caller buffer too small */
    SERVO_ERROR_UNSUPPORTED = -6,       /* platform/feature unavailable */
    SERVO_ERROR_INTERNAL = -7,          /* bug in Servo */
    SERVO_ERROR_INTERNAL_PANIC = -8,    /* panic caught at the boundary */
} ServoError;
```

Rules:

- Fallible functions return `ServoError` (`int32_t`). `0` is success; negative is failure;
  do not use positive values for errors.
- Object construction returns a pointer, with `NULL` for failure (optionally accompanied by
  a thread-local `servo_last_error()` accessor, to be added deliberately, not ad hoc).
- Out-parameters (strings, images, lists) are written only on success.
- Caller-provided buffers take `(buf, buf_len, out_len)` and return
  `SERVO_ERROR_CAPACITY` when too small, setting `out_len` to the required size.
- No function may leave an output uninitialized on failure; zero it or leave it untouched
  and document which.
- All `extern "C"` entry points are wrapped with `catch_unwind` to prevent unwinding
  across the FFI boundary. A caught panic results in `SERVO_ERROR_INTERNAL_PANIC`.

### 4.3 Null-safety

- Passing `NULL` for an argument that is documented non-null returns
  `SERVO_ERROR_NULL_ARGUMENT`; it must not abort. During the transition, existing
  `assert!(!ptr.is_null())` calls are P0 work in `ROADMAP.md` Phase 0/1.
- Passing `NULL` for a documented-optional pointer (e.g. an unset callback in a delegate) is
  valid and means "not interested".

### 4.4 UTF-8 and string handling

- Input strings are NUL-terminated UTF-8. Invalid UTF-8 returns
  `SERVO_ERROR_INVALID_ARGUMENT`; it must not `unwrap()`/panic.
- Returned strings are owned by Servo; provide `servo_string_free(char*)` (or document
  an ownership transfer to the caller and require `free()`). Be consistent across all
  string-returning functions and test for leaks.
- Embedded NUL is not representable in a C string; use `(ptr, len)` forms for arbitrary
  byte data and document it.

## 5. Ownership and threading contract template

Every new function's documentation must state all four items. Copy this block into the
Rust doc comment and the generated header (header text is derived from the doc comment by
cbindgen; keep both human-readable):

```text
/// <One sentence: what the function does.>
///
/// # Arguments
/// - `handle`: a non-null handle previously returned by `xxx_create`/`xxx_build`, still
///   owned by the caller and not already consumed by another function.
/// - `arg`: <meaning and valid range>.
/// - `out`: <meaning; written only on success>.
///
/// # Ownership
/// - `handle`: <borrowed | consumed>. If borrowed, ownership remains with the caller and
///   the handle must stay valid for the call. If consumed, the caller must not use or free
///   it afterwards.
/// - `out`/returned pointers: <owned by Servo | caller-provided buffer | transferred to
///   caller; free with xxx_free>.
///
/// # Threading
/// - Must be called from the same thread that created `handle` (the embedder main
///   thread). Not safe to call concurrently with any other function on the same object.
/// - <If a callback is passed: it is invoked on the main thread during
///   `servo_spin_event_loop`, at most once, and must not unwind.>
///
/// # Errors
/// - Returns `SERVO_ERROR_NULL_ARGUMENT` if `handle` is null.
/// - Returns `SERVO_ERROR_INVALID_ARGUMENT` if `arg` is out of range.
/// - ...
///
/// # Safety
/// The caller must ensure ... (the C-side preconditions for the unsafe `*mut`).
```

Templates by function shape:

- **Constructor**
  ```text
  Ownership: returns a new handle owned by the caller; free with xxx_free or consume
  with xxx_build/xxx_set_yyy.
  Threading: create on the embedder main thread; never share the handle across threads.
  ```
- **Borrowing method**
  ```text
  Ownership: `handle` borrowed, unchanged ownership; return value <...>.
  Threading: main thread only; do not call concurrently.
  ```
- **Consuming setter/builder**
  ```text
  Ownership: takes ownership of `value`; the caller must not use or free it afterwards.
  ```
- **Async operation with callback**
  ```text
  Ownership: `user_data` is owned by the caller and must remain valid until the callback
  fires or the associated object is freed.
  Threading: callback fires on the main thread during servo_spin_event_loop, at most once.
  ```
- **Result object**
  ```text
  Ownership: Servo fills the struct; the caller owns the storage. `struct_size` must be
  set by the caller before the call.
  ```

## 6. Deprecation policy and header version macros

- **Announce then wait.** A symbol is deprecated in a MINOR release by:
  1. Adding `SERVO_DEPRECATED` to its declaration (below).
  2. Documenting the replacement and the release in which removal will occur (at least
     one MINOR release later, and never in a PATCH).
  3. Emitting a one-time runtime warning (e.g. `log::warn!`) from the deprecated function.
- **Removal happens only in a MAJOR release.**
- **Never reuse a name** that has been deprecated/removed for a different purpose.

```c
#if defined(__GNUC__) || defined(__clang__)
#  define SERVO_DEPRECATED(msg) __attribute__((deprecated(msg)))
#elif defined(_MSC_VER)
#  define SERVO_DEPRECATED(msg) __declspec(deprecated(msg))
#else
#  define SERVO_DEPRECATED(msg)
#endif
```

Header must:

- Include the version macros (`servo/version.h` or inline) and a deprecation macro.
- State the exact ABI version in a comment and expose `servo_capi_get_version()`.
- Be entirely cbindgen-generated (optionally with a hand-written preamble via
  `cbindgen.toml`'s `header`/`autogen_warning`) so it cannot drift from the Rust source.
- Guard against C++ name mangling with `extern "C"` when compiled as C++.

## 7. Testing and compatibility guarantees

### 7.1 Per-platform guarantees

A release is "supported" on a platform only when:

- `cargo cinstall -p servo-capi` produces the shared library and header, and
- `cargo test -p servo-capi-tests --locked` passes, and
- the symbol allowlist check passes.

The existing `capi` jobs already run `cargo test -p servo-capi-tests --locked` after
installing `cargo-c` in `.github/workflows/linux-common.yml`, `mac.yml`, `mac-arm64.yml`,
and `windows.yml`. `ROADMAP.md` Phase 0 adds a dedicated
`.github/workflows/embedding-api.yml` to run these plus header/symbol checks.

### 7.2 What CI must verify on every change

1. **Build + tests** on Linux, macOS (x86_64/arm64), Windows via `servo-capi-tests`.
   *Implemented:* the `Embedding API (C API)` workflow (`.github/workflows/embedding-api.yml`)
   builds `servo-capi` with `cargo cinstall` and runs `servo-capi-tests` on all three hosts.
2. **Header is generated and valid**: the header is produced by cbindgen at build time and
   is **not** committed (source of truth is `ffi/capi/*.rs` + `cbindgen.toml`); CI
   regenerates it and fails if generation errors or the result is empty.
3. **Symbol surface**: compare exported symbols against the allowlist; fail on additions
   not in a reviewed list or on removals. *Implemented:* `ffi/capi/symbols.txt` is diffed
   against the cbindgen-generated header in `embedding-api.yml`.
4. **Version consistency**: header macros match `servo_capi_get_version()` and the crate
   version. *Implemented:* `test_capi_version` asserts
   `servo_capi_get_version() == SERVO_CAPI_VERSION` and
   `SERVO_CAPI_VERSION == (MAJOR << 16) | (MINOR << 8) | PATCH`.
5. **Panic-safety tests** (Phase 1+): call representative functions with null, invalid
   UTF-8, out-of-range values, and unknown enum values; assert an error code is returned
   and the process does not abort.
6. **ABI-struct tests** (Phase 1+): pass a delegate/result struct with the current
   `struct_size` and with an older, smaller `struct_size`; assert graceful behavior.
   *Implemented:* `test_delegate_struct_size_validation` passes the current, a smaller,
   and a zero `struct_size` and asserts `SERVO_ERROR_OK` vs `SERVO_ERROR_INVALID_ARGUMENT`.

### 7.3 Compatibility statement

- A binary built against header version `X` runs against library version `Y` when
  `major(X) == major(Y)` and `minor(Y) >= minor(X)` (forward-compatible additions). Do
  not promise backward compatibility across majors.
- Within a major version, a host compiled against an older header must continue to work
  with a newer library: unknown enum values are ignored, unknown symbols are simply not
  referenced, and for structs passed **by pointer** absent trailing fields are treated as
  defaults.
- Exception: the delegate-like structs passed **by value** must be recompiled against the
  newer header. Their `struct_size` is validated (§3.3), so a caller that passes a struct
  smaller than the compiled-in size gets `SERVO_ERROR_INVALID_ARGUMENT` instead of reading
  out of bounds.
- The policy applies per platform and per supported feature configuration. Functions that
  depend on an optional feature (e.g. gamepad) must return `SERVO_ERROR_UNSUPPORTED` when
  built without it, rather than being absent in a way that breaks the link, wherever that
  is practical.

## See also

- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — how the C ABI maps onto the engine.
- [`ROADMAP.md`](./ROADMAP.md) — the phases that implement this policy.
- `ffi/capi/cbindgen.toml` — enum renaming/prefixing and export list.
- `tests/capi` — the C ABI test harness (`servo-capi-tests`).
