# Panic-Free FFI Boundary (P0 #2) - Todo List

## 1. Core Infrastructure (lib.rs)
- [x] Define `ServoError` enum with `#[repr(i32)]` in `ffi/capi/lib.rs`
- [x] Add `catch_ffi`, `catch_ffi_i32`, `catch_ffi_ptr`, `catch_ffi_const_ptr` panic boundary wrappers in `ffi/capi/lib.rs`
- [x] Export `ServoError` from lib.rs

## 2. Update all FFI functions in capi/*.rs
- [x] `lib.rs` - ServoBuilder, Servo functions
- [x] `webview.rs` - WebView, WebViewBuilder functions
- [x] `servo.rs` - Servo runtime functions
- [x] `javascript.rs` - JavaScript evaluation
- [x] `webview_delegate.rs` - Delegate (mostly callback implementations, check for panics)
- [x] `options.rs` - Options functions
- [x] `preferences.rs` - Preferences functions
- [x] `rendering_context.rs` - Rendering context functions
- [x] `site_data.rs` - Site data & cookie management
- [x] `cookie_manager.rs` - Visitor & filter handling
- [x] `download.rs` - Download item management
- [x] `find.rs` - Find in page
- [x] `protocol_handler.rs` - Custom protocol handling
- [x] `cert_error.rs` - Certificate error continuation

For each function:
- [x] Replace `assert!(!ptr.is_null())` with null check returning `ServoError::NullArgument`
- [x] Replace `.to_str().expect(...)` with `.to_str().map_err(|_| ServoError::InvalidUtf8)?` (or safe match)
- [x] Wrap body in `catch_ffi(|| { ... })` / `catch_ffi_ptr(|| { ... })`
- [x] Fallible functions return `ServoError` (int32_t); constructors return `*mut T` with `NULL` on failure

## 3. Update cbindgen.toml
- [x] Export `ServoError` enum

## 4. Update Documentation
- [x] `docs/embedding/C_API_GUIDE.md` - document new error convention
- [x] `docs/embedding/ABI_STABILITY.md` - update error policy section

## 5. Update C Tests
- [x] `tests/capi/c/test_api.c` - add error-return tests (null args, invalid UTF-8, etc.)

## 6. Verify (in GitHub Actions)
- [ ] `cargo test -p servo-capi-tests --locked` passes in GitHub Actions
- [ ] Header generation & C compilation verify in GitHub Actions