# Implementation Review — Pass 2 (repair + W5.2 + W4.1)

**Scope reviewed:** `13cad2c3a45..be9606d2fca` — **14 commits** (9 repair + 3 × W5.2 + 1 × W4.1).
Working tree clean. `PRODUCTION_REPORT.md` untouched by the agent (as instructed).

**Reviewer constraints (unchanged):** this machine has no linker (`link.exe` / `lld-link.exe`
missing) and no C toolchain for Rust besides `rustc`; `cargo check` / `cargo test` / `clippy`
cannot run; `mach test-tidy` cannot bootstrap (`ModuleNotFoundError: No module named 'toml'`).
Everything below is **source-level verification**, plus the checks that *do* run here:
`rustc --emit=metadata` reproductions, `cbindgen`, `gcc`, `g++`.

> **Update (R2 follow-up, §7):** blockers B8–B10 were fixed in the R2 pass and the branch was then
> **compiled, linked and tested end-to-end on real CI** (Linux, macOS, Windows). That surfaced two
> further defects — B11 (`E0507` in `servo-net`) and B12 (a Linux-only teardown `SIGSEGV` caused by a
> reference cycle) — both now fixed. The final gate run is green on all three platforms.

---

## 1. Verdict

**The repair ticket is genuinely done — except item 4 (fuzz) — and W5.2 is real. W4.1 is wired
up but is partly dead code and re-introduces a hard compile failure.**

Specifically:

- All 9 repair items landed and 8 of 9 are **verified correct**. Item 4's status line
  ("Completed") is **false** — see B10.
- **The branch still does not compile.** W4.1 added two `error[...]` defects (B8, B9) in the same
  new function, which is itself unreachable. The previous `E0063` (B1) is fixed.
- The W4.1 code duplicates machinery that already exists and works (`EmbedderToConstellationMessage::EvaluateJavaScript`),
  and two documents now describe behaviour that does not exist (no `CustomEvent`, a message variant
  that is never constructed).

| Ticket | Status |
|---|---|
| Repair items 1–3, 5–9 | ✅ verified correct |
| Repair item 4 (fuzz) | 🔴 **not fixed** (B10) — bad dep path + bad import |
| W5.2 (CMake/pkg-config/CI verify) | ✅ structurally correct; still never dispatched |
| W4.1 (message router) | 🟡 wired & real, but contains **2 compile blockers (B8, B9)**, dead/redundant path, and doc overclaims |
| Build | 🔴 **never run anywhere; would fail on B8/E0433** |

---

## 2. New blockers (introduced by W4.1 — must fix before anything else)

All three live in the *new* native→JS path. Note that this whole path is **unreachable** (see §4),
so the cleanest fix is to delete it; the blockers disappear with it.

### B8 — `serde_json` used in `components/constellation` but not declared → `E0433`

`components/constellation/constellation.rs:3288`:

```rust
name = serde_json::to_string(&name).unwrap_or_else(|_| "\"\"".into()),
```

`components/constellation/Cargo.toml` `[dependencies]` lists `serde` but **not `serde_json`**
(and there is no `pub use`/`extern crate serde_json` in the crate). Rust only resolves crates that
are *direct* dependencies, so this is `error[E0433]: failed to resolve: use of undeclared crate or
module `serde_json``. `serde_json` is in the workspace table (`Cargo.toml:228`), so the fix is one
line in the crate manifest — or delete the function.

### B9 — `JavaScriptEvaluationId::new()` does not exist → `E0599`

`components/constellation/constellation.rs:3295`:

```rust
let _ = pipeline.event_loop.send(ScriptThreadMessage::EvaluateJavaScript(
    webview_id,
    pipeline.id,
    JavaScriptEvaluationId::new(),   // <-- no such associated function
    script,
));
```

The type is a plain tuple struct, `pub struct JavaScriptEvaluationId(pub usize)`
(`components/shared/embedder/lib.rs:1037`), and there is **no `impl JavaScriptEvaluationId`
anywhere in the repo** (`git grep "impl JavaScriptEvaluationId"` → nothing). It is constructed
directly elsewhere: `JavaScriptEvaluationId(0)` / `JavaScriptEvaluationId(self.current_id.0 + 1)`
(`components/servo/javascript_evaluator.rs:25,32`). → `error[E0599]: no function or associated
item named `new` found for struct `JavaScriptEvaluationId``.

### B10 — repair item 4 did not fix the fuzz crate (status says "Completed")

Two independent defects remain in `fuzz/`:

1. **Wrong dependency path.** `fuzz/Cargo.toml` adds
   `[dependencies.servo_url] path = "../components/servo_url"` — that directory does not exist.
   The `servo-url` package (lib name `servo_url`) is at **`components/url`**. Cargo cannot load the
   manifest.
2. **Unresolved import.** `fuzz/fuzz_targets/fuzz_cookie_parser.rs` has
   `use net::cookie::{CookieSource, ServoCookie};`. `CookieSource` is defined in **`net_traits`**
   (`components/shared/net/lib.rs:1378`) and `components/net/cookie.rs` imports it **privately**
   (`use net_traits::{CookieSource, ...}`) with no `pub use` re-export. → `error[E0432]: unresolved
   import `net::cookie::CookieSource``.

(For the record, the parts that *are* right: `ServoUrl::parse` exists at `components/url/lib.rs:70`,
and the call now matches the real signature `from_cookie_string(&str, &ServoUrl, CookieSource)`.
The `[workspace]` marker and the root `exclude = [..., "fuzz"]` are also correct.)

---

## 3. Verified correct (repair items 1–3, 5–9 and the ABI)

| # | Item | Evidence |
|---|---|---|
| 1 | `E0063` (B1) | All four `SameSiteContext` literals now carry `partition_key`; `http_loader.rs:446`, `tests/cookie.rs:637,644,694`. |
| 2 | Sandbox overclaim (B4) | `servo_capi_is_sandboxing_supported()` now excludes windows / ios / android / arm / aarch64 / riscv32 / riscv64 / ohos (`ffi/capi/lib.rs:450-462`). `servo_create_sandbox()` applies the Windows mitigations then **returns `false`** on Windows, and consults the predicate elsewhere (`ffi/capi/multiprocess.rs`). `COMPATIBILITY.md` §5, `ARCHITECTURE.md`, `SECURITY.md`, `C_API_GUIDE.md`, `MULTI_PLATFORM_SANDBOX_DESIGN.md` corrected to "mitigations / OS process model — not a sandbox". Reproduced the cfg-gated-tail shape with `rustc --cfg 'feature="multiprocess"'` on Windows → **exit 0** (no `E0308`). |
| 3 | CHIPS (B4) | `cookie.rs`: partitioned cookies are rejected unless `Secure` **and** the request is a secure scheme; storage documented as *stored but never sent* (fail-closed), and `SECURITY.md` restored that sentence. |
| 5 | ohos cfg collision (B3) | `not(target_env = "ohos")` added to the `arm`/`aarch64` `create_sandbox` arm → no `E0428`. |
| 6 | FFI thread check + dead handles (B5/B6) | `assert_main_thread()` moved **inside** all four `catch_ffi*` closures (`lib.rs:263,275,286,297`); the lock is now poison-safe (`let Ok(mut lock) = … else { return }`). `register_handle`/`validate_handle`/`unregister_handle` are **deleted** (`git grep` → empty), and the `SECURITY.md` sentence corrected. `tests/capi` coverage added (Windows `sandboxing == false` assertion, `test_debug_thread_checks`, `test_script_message_router`). |
| 7 | SECURITY.md | Weekly-scan claim and the invented 48h/7/14/30/90-day SLAs removed; backport promise and threat model softened. |
| 8 | WPT baseline | Retitled; explicitly states it is a meta-expectation recount, not a pass rate; regeneration instructions added. |
| 9 | Helper scripts | `etc/ci/wpt_pass_rate.py` and `site_smoketest.py` documented as manual commands in `WPT_CONFORMANCE_BASELINE.md` / `SITE_SMOKETEST_REPORT.md`. |

**ABI/symbol checks (run here):**

| Command | Result |
|---|---|
| `cbindgen --config ffi/capi/cbindgen.toml --output <h> ffi/capi` | exit 0 |
| `gcc -std=c11 -Wall -fsyntax-only <h>` | exit 0 |
| `g++ -std=c++17 -Wall -Wextra -Werror -pedantic -fsyntax-only <h>` | exit 0 |
| `gcc -std=c11 -Wall -Wextra -fsyntax-only -I<gen> tests/capi/c/test_api.c` | exit 0 (only the intentional `servo_webview_evaluate_javascript` deprecation warnings) |
| `git diff … -- ffi/capi/symbols.txt` | **additions only** (`servo_webview_add_script_message_handler`, `servo_webview_remove_script_message_handler`); no removals |
| new `ServoWebViewDelegate.on_script_message` | appended **last** in the struct → ABI-safe; `struct_size` present; the new C test initialises it by designation and compiles |

The three documented `SECURITY.md` claims now match the code (predicate false on the six
unsupported targets, CHIPS stored-not-sent, no SLA).

---

## 4. W4.1 — dead / redundant path and doc overclaims

### 4.1 The new native→JS path is redundant, broken, and unreachable

`WebView::post_message_to_script` (`components/servo/webview.rs:906`) has **no callers**
(`git grep post_message_to_script` → only its own definition). It sends
`EmbedderToConstellationMessage::PostMessageToScript`, which the constellation handles in the new
`handle_post_message_to_script` (`constellation.rs:3265-3300`) — the function that contains B8 and
B9.

But Servo **already** has this exact route:
`EmbedderToConstellationMessage::EvaluateJavaScript` (`components/shared/constellation/lib.rs:139`,
handled at `constellation.rs:1343`, forwarded to `ScriptThreadMessage::EvaluateJavaScript` at
`constellation.rs:1543`), driven by `WebView::evaluate_javascript` → `JavaScriptEvaluator`
(`components/servo/javascript_evaluator.rs`). That is what the C API's `servo_webview_post_message`
actually uses today (`ffi/capi/message_router.rs:253` → `webview.post_message` → `webview.rs:900`),
and it works without any of the new code.

So the new path is a second, worse copy of existing machinery. **Recommended fix: delete it**
(and with it B8/B9):
`post_message_to_script` (webview.rs:906), `EmbedderToConstellationMessage::PostMessageToScript`,
its match arm (`constellation.rs:1425`) and `handle_post_message_to_script` (`:3265`), the tracing
arm (`tracing.rs:89`).

### 4.2 `EmbedderMsg::OnScriptMessage` is never constructed

`components/shared/embedder/lib.rs:551` adds the variant and `components/servo/servo.rs:818` handles
it, but **nothing constructs it** (`git grep OnScriptMessage` → only those two sites). The live
JS→native path is the polling one: `WebView::poll_js_messages` → `evaluate_javascript(poll_script)`
→ `delegate.notify_script_message(...)` + `router.handle_js_message(...)`. `ARCHITECTURE.md` §5.4
claims `OnScriptMessage` "routes messages from the engine to the embedder" — it does not. Either
wire it (script-thread push) or delete the variant and its arm.

### 4.3 `CustomEvent named servo-message` is not what the code does

`docs/embedding/ARCHITECTURE.md` §5.4 and `docs/embedding/C_API_GUIDE.md:1236` (and the doc comment
on `components/servo/message_router.rs`) say native→JS "dispatches a `CustomEvent` named
`servo-message` on `window` with `detail: { name, data }`". The implementation
(`MessageRouter::bootstrap_script`, `_dispatch`) instead calls the registered listener functions
with a plain object `{ type: "message", data, origin }`. There is no `CustomEvent` and no
`servo-message` event. Fix the wording, or implement a real `CustomEvent`. (The wording predates
this pass in `message_router.rs`, but W4.1 propagated it into two more documents.)

### 4.4 Duplicated script-building

`MessageRouter::message_to_javascript` (`components/servo/message_router.rs`) already builds the
native→JS dispatch snippet; `handle_post_message_to_script` re-implements it inline.

---

## 5. W5.2 — assessed (structurally correct, never executed)

- `.github/workflows/capi-artifacts.yml`: new steps install the CMake + pkg-config files into the
  sysroot and then **really** compile and link `examples/c-embedder`, both via `find_package(Servo)`
  and a direct `gcc` link, under `set -euo pipefail` and with **no `|| true`**. The install staging
  is consistent with the earlier step (`cargo cinstall … --destdir target/artifacts --prefix /`).
- `support/cmake/ServoConfig.cmake` + `ServoConfigVersion.cmake`: `_SERVO_PREFIX` resolves to the
  sysroot; `INTERFACE_INCLUDE_DIRECTORIES = <prefix>/include` matches `main.c`'s
  `#include <servo/servo_capi.h>` and `[package.metadata.capi.header] subdirectory = "servo"`.
- `support/pkgconfig/servo_capi.pc`: `prefix=${pcfiledir}/../..` → sysroot; `-lservo_capi` /
  `-I${includedir}` correct.
- `examples/c-embedder/CMakeLists.txt`: find_package → pkg-config → direct fallback; DLL copy on
  Windows.
- **Caveat:** the workflow has still **never been dispatched**, and the Windows branch assumes a
  MinGW toolchain / `mingw32-make` on the runner. Structural review only — no execution.

---

## 6. Ordered fix list (paste this to the agent as the next ticket)

1. **Delete the redundant native→JS path** (this removes B8 and B9):
   `WebView::post_message_to_script` (`components/servo/webview.rs:906`);
   `EmbedderToConstellationMessage::PostMessageToScript` (`components/shared/constellation/lib.rs`);
   its match arm (`components/constellation/constellation.rs:1425`) and the whole
   `handle_post_message_to_script` fn (`:3265-3300`); the `tracing.rs` arm.
   Native→JS already works through `EmbedderToConstellationMessage::EvaluateJavaScript` /
   `WebView::evaluate_javascript` / `servo_webview_post_message` — do **not** re-add a second route.
   - If instead you decide to keep it: add `serde_json` to `components/constellation/Cargo.toml`
     (fixes B8) **and** replace `JavaScriptEvaluationId::new()` with a real id (fixes B9) — but then
     also route `WebView::post_message` through it, or it stays dead.
2. **Resolve `EmbedderMsg::OnScriptMessage`** (`components/shared/embedder/lib.rs:551`,
   `components/servo/servo.rs:818`): construct it from the script thread, or delete the variant and
   its arm. Leave no variant that nothing sends.
3. **Fix the fuzz crate for real (B10)** — the previous "Completed" claim was wrong:
   `[dependencies.servo_url] path = "../components/servo_url"` → **`../components/url`**, and
   change `use net::cookie::{CookieSource, ServoCookie};` to import `CookieSource` from
   **`net_traits`** (`use net_traits::CookieSource;`). Gate with
   `cargo +nightly fuzz build fuzz_cookie_parser` (or delete the crate).
4. **Correct the messaging docs** so they match the code: remove the `CustomEvent` / `servo-message`
   claim (or implement it) in `docs/embedding/ARCHITECTURE.md` §5.4, `docs/embedding/C_API_GUIDE.md`,
   and the module comment in `components/servo/message_router.rs`; drop the `OnScriptMessage`
   "routes messages" sentence unless step 2 wires it.

**Acceptance gate once a toolchain exists:** `cargo check --workspace` →
`cargo test -p servo-net` → `cargo clippy --workspace -- -D warnings` → `mach test-tidy` →
`cargo cinstall -p servo-capi --features multiprocess` → dispatch `capi-artifacts.yml` and
`windows-wpt.yml`.

---

## 7. Follow-up — R2 pass, verified on real CI (added after the R2 work)

> This section supersedes the "never run anywhere" caveats above. Blockers B8–B10 were fixed in the R2
> pass (`7dc1b158ecc`, `a6752cc7b93`, `07000a43a77`, `f3937fbdd5e`), and the branch was then
> **compiled, linked and tested end-to-end** by `.github/workflows/embedding-api.yml` on Linux, macOS
> and Windows. Doing so surfaced two further defects (B11, B12) that no amount of source review had
> caught.

### 7.1 R2 resolved the Pass-2 blockers

| Blocker | Resolution |
|---|---|
| B8 (`serde_json` undeclared) | Redundant native→JS path deleted — `serde_json` no longer referenced in `components/constellation`. |
| B9 (`JavaScriptEvaluationId::new`) | Deleted together with the same path. |
| B10 (fuzz crate) | `servo_url` path → `../components/url`; `CookieSource` imported from `net_traits`. |
| `EmbedderMsg::OnScriptMessage` | Variant and its match arm deleted (no variant that nothing sends). |
| Doc overclaims (`CustomEvent`) | Messaging docs corrected to match the listener-callback implementation. |

### 7.2 B11 — `error[E0507]` in `servo-net` (SameSite work) — fixed in `d65c20886e2`

The first real compile of the branch failed on **all three platforms**:

```
error[E0507]: cannot move out of `context`, a captured variable in an `FnMut` closure
   --> components/net/cookie_storage.rs:304:78
```

`appropriate_for_url_with_context` takes its `SameSiteContext` **by value**, but `SameSiteContext`
carries `partition_key: Option<String>` and therefore is **not `Copy`** (it derives only `Clone`). The
`Iterator::filter` closure is `FnMut`, so moving the captured `context` out on every call cannot
compile. (Parse-only checks on a branch that has never been type-checked cannot see this class of
error.)

**Fix (`d65c20886e2`):** keep capturing the context by value — the returned `impl Iterator + 'a`
outlives the call — and clone it per cookie: `context.clone()`. Verified locally with
`cargo check -p servo-net` (exit 0, 0 warnings). All three platforms then passed the compile step.

### 7.3 B12 — Linux-only teardown `SIGSEGV` (reference cycle) — fixed in `35aed4fa8ee`

With B11 fixed, macOS and Windows went green but **Linux failed 3/3 times**, *after* every test had
passed:

```
RUN  test_script_message_router
test c_api_tests ... ok
mozilla::detail::MutexImpl::~MutexImpl: pthread_mutex_destroy failed: Device or resource busy
error: test failed ... (signal: 11, SIGSEGV)
```

This was a **regression, not a flake.** Dispatching the *same* workflow on the green baseline
`68719ce75c2` (run 39001108558) produced all three platforms green with zero occurrences of the
mozglue message; the branch runs 84 tests (baseline: 82) and crashes during process teardown, after
libtest prints its result.

**Root cause.** The messaging work made `WebView::poll_js_messages` capture a **strong** `WebView`
clone in its evaluation callback (`components/servo/webview.rs`):

```rust
let this = self.clone();
self.evaluate_javascript(MessageRouter::poll_script(), move |result| {
    ...
    delegate.notify_script_message(this.clone(), &name, &data_json);
```

A `WebView` holds an `Rc` to its `Servo` (`WebViewInner.servo`), and `ServoInner` owns the
`JavaScriptEvaluator` that stores this callback — producing the cycle

```
Servo → JavaScriptEvaluator → pending_evaluations → callback → WebView → Rc<Servo>
```

The C API harness never spins the event loop, so the evaluation never completes, the callback is
never removed, and neither the `WebView` nor the `Servo` is ever dropped. The leaked SpiderMonkey
state then crashed the process during static teardown (`pthread_mutex_destroy` on a still-live
mutex → `SIGSEGV`). macOS and Windows happen not to hit the race.

This is also a **real product bug**: any embedder that frees a `WebView` while an evaluation is
pending leaks the whole engine, because the pending callback keeps it alive.

**Fix (`35aed4fa8ee`):** capture a `Weak` handle and upgrade inside the callback, so freeing the
`WebView` releases it and the `Servo`:

```rust
let weak_webview = self.weak_handle();
self.evaluate_javascript(MessageRouter::poll_script(), move |result| {
    let Some(webview) = weak_webview.upgrade().map(WebView) else {
        return; // The WebView was freed before this evaluation completed.
    };
    ...
    delegate.notify_script_message(webview.clone(), &name, &data_json);
```

`post_message`'s callback captures nothing, and the deprecated `servo_webview_evaluate_javascript*`
closures capture only the C fn-pointer and `user_data`, so `poll_js_messages` was the only cycle.

### 7.4 Final CI result

Run **39040087937** on `35aed4fa8ee` — **all three platforms green**:

| Job | Result |
|---|---|
| C API (Linux) | ✅ 84 tests, 84 passed, clean exit (0 `MutexImpl`, 0 `SIGSEGV`) |
| C API (macOS) | ✅ |
| C API (Windows) | ✅ |

**Still not verified by CI:** `capi-artifacts.yml` and `windows-wpt.yml` have never been dispatched,
and the `fuzz/` crate is a separate workspace that `embedding-api.yml` does not build (B10 remains
source-verified only).

---

## Appendix — commands actually run in this review

| Command | Result |
|---|---|
| `git log --oneline 13cad2c3a45..HEAD` | 14 commits |
| `git diff --stat 13cad2c3a45..HEAD` | 34 files, +1231 / −218 |
| `cbindgen … ffi/capi` | exit 0 |
| `gcc -std=c11 -fsyntax-only <header>` | exit 0 |
| `g++ -std=c++17 -Werror -pedantic -fsyntax-only <header>` | exit 0 |
| `gcc -std=c11 -Wextra -fsyntax-only -I<gen> tests/capi/c/test_api.c` | exit 0 |
| `rustc --edition 2024 --emit=metadata --cfg 'feature="multiprocess"' cfgtail.rs` | exit 0 (Windows `create_sandbox` tail is sound) |
| `git grep "serialize|serde_json" components/constellation` | only use at `constellation.rs:3288`; **no manifest entry** → B8 |
| `git grep "impl JavaScriptEvaluationId"` | **empty** → B9 |
| `git grep "OnScriptMessage"` | definition + match arm only; **no constructor** |
| `git grep "post_message_to_script"` | definition only; **no callers** |
