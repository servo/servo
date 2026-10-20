# Servo Embedding — Hardening Plan (Stability & FFI)

Status: **partially implemented** (A.1 landed; A.2's FFI-entry inventory landed — the rest of this document is still design) · Branch: `embedding/cef-parity` · Baseline
commit: `68719ce75c2`

This document turns the *recommendations* in
[`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) §9 (items 2, 3, 5) and
[`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) §9 (items 2, 3) into concrete,
bounded engineering steps with exact touch points.

> **Most of this document is not yet implemented.** What has landed is marked
> in place: **A.1** (background hang monitor on by default, with a green gate)
> and **A.2.1** (the FFI-entry reachable-panic inventory, with the one reachable
> site fixed). The remainder are designs, written to be reviewable and
> executable by someone who can build and test the engine (the environment this
> plan was produced in could not: no MSVC/clang/GStreamer, empty `target/`).
> Each step names the files to change and the tests that must gate it.

---

## Part A — Stability

### A.1 Background hang monitor (BHM)

**Status: implemented.** Both flags default to `true` and a permanent hang
terminates the hung component's script; the embedder observes the interrupted
JavaScript call completing with an error. Landed in `115948c8334` (defaults flip)
plus `5963519e4a8` (monotonic clock + ≥3 s lower bound), gated green on
Linux/macOS/Windows in runs `39163255415` and `39165923943`. Honest eval→kill
latencies: Linux ~5019 ms, Windows ~5047 ms, macOS ~5122 ms, from
`tests/capi/c/test_integration.c::test_hang_monitor_kills_hung_script`.
The design text below is kept for provenance; claims that no longer hold are
corrected in place.

**Where it lives**

- Config flags: `components/config/opts.rs` — `background_hang_monitor` and
  `hang_monitor_kill_script`, **both default `true`**.
- C API: `ffi/capi/options.rs` — `servo_options_create()` returns
  `Opts::default()`, so C embedders get both on; the individual setters
  (`servo_options_set_background_hang_monitor`,
  `servo_options_set_hang_monitor_kill_script`) remain the opt-outs.
- Shell flags: `ports/servoshell/prefs.rs` — `-B` / `--bhm` and
  `--bhm-kill-hung-scripts`. servoshell parses these as explicit CLI booleans
  (default `false`) and **overrides** `Opts::default()`, so the shell stays
  opt-in. Changing the shell's CLI default requires a bpaf two-flag construct
  and cannot be verified by this branch's CI gate (no workflow compiles
  `ports/servoshell` on a push to `embedding/cef-parity`), so it is left as a
  follow-up.
- Worker: `components/background_hang_monitor/background_hang_monitor.rs`.
- Alert API: `components/shared/background_hang_monitor/lib.rs`
  (`HangAlert::{Transient,Permanent}`, `MonitoredComponentId`,
  `BackgroundHangMonitorExitSignal`, `BackgroundHangMonitorControlMsg`).
- Consumer: `components/constellation/constellation.rs`
  `handle_request_from_background_hang_monitor` and
  `components/constellation/event_loop.rs`
  `send_message_to_background_hang_monitor`.

**How a kill plays out (verified end to end, W5.1)**

1. The BHM worker ticks every 100ms while monitoring is enabled. For each
   registered component it compares `last_activity` against the transient (1s)
   and permanent (5s, hard-coded in `script_thread.rs`) timeouts, skipping
   components that are waiting for events. Each alert is sent at most once
   until activity resumes.
2. On `HangAlert::Permanent`, the constellation logs the alert and, when
   `hang_monitor_kill_script` is set, broadcasts
   `BackgroundHangMonitorControlMsg::SignalComponentExit(component_id)`.
3. The worker looks up the named component and calls
   `component.exit_signal.signal_to_exit()`. For a script thread this sets a
   `closing` flag and requests a JS interrupt; the interrupt callback runs
   `prepare_for_shutdown()` (cancelling all pending tasks) and returns
   `false`, which makes SpiderMonkey terminate the running script. An
   in-flight `servo_webview_evaluate_javascript` therefore completes **with an
   error** — that is the embedder-visible outcome of a kill.
4. Afterwards the script thread is inert, not dead: `handle_msgs` only services
   `ExitScriptThread` / `ExitPipeline` and drops everything else, and
   rendering updates stop. It exits cleanly when the constellation drops its
   `EventLoop` (`EventLoop::drop` sends `ExitScriptThread`), which happens
   once the killed pipelines are closed — normally at embedder teardown, or
   earlier if the embedder frees the WebViews.
5. Shutdown is not blocked by a killed thread: sends to its queue fail fast and
   `close_pipeline` treats an unreachable pipeline as already exited;
   `handle_shutdown`'s joins return normally because the kill is cooperative.
   (The `run_content_process` join sites named in the original W5.1 ticket,
   `components/servo/servo.rs`, only panic if the script thread itself
   panicked — unchanged behaviour — and that `#[cfg(feature = "multiprocess")]`
   code is not compiled by this branch's CI gate, so no speculative edit was
   made there.)

**Known limitations (documented, not fixed)**

- The kill is cooperative: it interrupts **JavaScript execution**. A script
  thread wedged in native code that never re-enters JS never sees the
  interrupt; recovery then relies on embedder-level timeouts. True force-kill
  needs OS-level thread termination, which is out of scope.
- After a kill the event loop's script thread stays shut down for good: its
  WebViews are inert (no further delegate callbacks, no frames). The embedder
  recovery recipe is to free the killed WebViews and create new ones (a fresh
  script thread is spawned), or to exit and restart the engine; in
  multiprocess builds, letting the browser process reap the content process is
  the equivalent.
- Transient alerts (1s) never kill; they are diagnostics only.

**Tests**

- `components/background_hang_monitor/tests/hang_monitor_tests.rs::test_hang_monitoring_signal_component_exit`
  covers the worker arm: `SignalComponentExit` fires the named component's
  exit signal exactly once and leaves other components alone.
- `tests/capi/c/test_integration.c::test_hang_monitor_kills_hung_script`
  covers the end-to-end path with the shipped defaults: evaluating
  `while (true) {}` is interrupted with an error just after the 5s permanent
  timeout. The test also rejects an interruption earlier than 3s, since an
  unrelated shutdown signal could complete the evaluation early and would not
  prove the permanent-hang path. The suite's final `servo_free` proves teardown
  stays clean.

### A.2 Remaining reachable panics

The one web-content-triggerable panic found in the audit
(`components/script/dom/bindings/error.rs`) is already fixed; see
`SECURITY_AUDIT.md` §4.1. The audit listed candidates that were **not** changed
because they need case-by-case review. The process to clear them:

1. Enumerate `panic!`/`unwrap`/`expect`/`unreachable!` on paths reachable from
   web content (script, net, layout) — the audit's §4 table is the starting
   inventory.
2. For each, write a failing fuzz/regression case or a unit test that reaches it
   from untrusted input. If it cannot be reached from untrusted input and is a
   genuine invariant, mark it explicitly (a short comment naming the invariant)
   rather than changing it.
3. Convert the genuinely reachable ones to fallible paths that surface an error
   to the embedder or produce the specified failure (e.g. a network error), as
   was done for the bindings panic.

This is a long-tail task; prioritise `components/net` and the DOM bindings,
which take the most attacker-controlled input.

#### A.2.1 FFI entry paths — tracked inventory (W5.5)

Inventory command:

```sh
git grep -nE "\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(|assert!\(|assert_eq!\(|assert_ne!\(" \
  -- ffi/capi components/servo/servo.rs components/servo/webview.rs components/servo/lib.rs
```

Counts: **23 sites before** W5.5 (1 in `ffi/capi`, 22 in `components/servo`),
**22 after** (the one genuinely reachable site is fixed; the rest are tracked
below). Wave C then added three more to `ffi/capi` —
`webview_delegate.rs:1508/1518/1526`, each `label_cstrings.last().unwrap()`
immediately after the `push` that created that element, so provably non-empty —
which leaves `ffi/capi` at 3 sites today, all of that shape. Classification and
status:

| Location | Site | Classification |
|---|---|---|
| `components/servo/servo.rs` `Servo::setup_logging` (was `:1141`) | `log::set_boxed_logger(..).expect("Failed to set logger.")` | **Reachable from C** — `servo_setup_logging` called twice panicked, surfacing `SERVO_ERROR_INTERNAL_PANIC` through `catch_ffi`. **Fixed:** `Servo::try_setup_logging() -> Result<(), log::SetLoggerError>`; the FFI maps the error to `SERVO_ERROR_INVALID_STATE`; C test `test_servo_setup_logging_repeated`. |
| `ffi/capi/webview.rs:351` | `.expect("A builder without a create request must have a Servo")` | Unreachable by construction. `servo_webview_builder_create` sets `servo: Some`; `servo_create_new_webview_builder_create` sets `create_new_request: Some`, so `create_new_request == None ⟹ servo.is_some()`. |
| `components/servo/webview.rs:239` | `.expect("Should always be able to parse 'about:blank'.")` | Unreachable by construction: parse of a constant, always-valid URL. |
| `components/servo/webview.rs:513` and `:521` | `.expect("BUG: invalid WebView instance")` (`show`/`hide`) | Unreachable for a valid handle: every `WebViewBuilder::build` registers with paint (`webview.rs:204`). An invalid handle is UB, not this panic. |
| `components/servo/webview.rs:1053` | `unreachable!("…routed through the FileManagerThread")` | Unreachable by construction: `FilePicker` is routed via `ForwardEmbedderControlResponseFromFileManager` (`document_embedder_controls.rs:152`); only SelectElement/ColorPicker/InputMethod/ContextMenu send `ShowEmbedderControl`. |
| `components/servo/servo.rs:1281` | `GenericCallback::new(..).expect("Failed to create callback")` | Unreachable in practice (callback-registry exhaustion is not reachable from input); also guarded by `catch_ffi`. |
| `components/servo/servo.rs:1590` | `.expect("Failed to activate sandbox!")` | Guarded: compiled only with `multiprocess` on x86_64, and `servo_create_sandbox` calls it inside `catch_unwind`. |
| `components/servo/servo.rs:1642` | `panic!("Sandboxing is not supported on single-process, RISC-V, or OpenHarmony targets.")` | Dead/guarded: never called in a single-process build (`servo_create_sandbox` returns `false`), and gated + `catch_unwind`d under `multiprocess`. |
| `components/servo/servo.rs:201`/`:203`/`:205` | `debug_assert_eq!` on literal channel indices | Unreachable by construction (literal indices); debug-only. |
| `components/servo/servo.rs:720`/`:737` | `.expect("Could not send message via callback")` (`gamepad`) | Feature-gated (`gamepad`); not compiled by `embedding-api.yml`. |
| `components/servo/servo.rs:1399`/`:1407`/`:1409`/`:1412`/`:1414`/`:1457`/`:1460`/`:1725` | content-process setup, IPC `unwrap`s, thread joins, memory-reporter callback | Feature-gated (`multiprocess`); not compiled by `embedding-api.yml`. |
| `components/servo/servo.rs:128` | `std::env::current_exe().unwrap()` | Feature-gated (`media-gstreamer`, only under `mod media_platform`); not compiled by `embedding-api.yml`, and not reachable from C input. |

`components/servo/lib.rs` has **0** sites (re-exports only).

### A.3 Fuzzing and crash telemetry (projects, not one-liners)

- **Fuzzing.** There is no libFuzzer/AFL harness today. Add `cargo-fuzz` targets
  for the highest-value parsers first: HTML/CSS tokenizers, the URL parser, the
  image decoders, and the HTTP header parser. Run them on a schedule in CI.
- **Crash telemetry.** Only a POSIX signal handler that prints a backtrace
  (`ports/servoshell/crash_handler.rs`, macOS/Linux only). At minimum, make the
  handler cross-platform and symbolicate; minidump/symbol-server integration is a
  larger follow-on.

These are quarter-scale; they are recorded here so they are not silently dropped.

---

## Part B — FFI boundary hardening

The C ABI (`ffi/capi`) is the one boundary this branch owns. It is already
panic-safe: every fallible entry point is wrapped in `catch_unwind` and returns
`ServoError::InternalPanic` / a null pointer (`ffi/capi/lib.rs`). The residual
risks are all **the embedder's misuse of opaque handles**, which the engine
cannot detect today.

### B.0 `struct_size` coverage across the ABI (W2.5 inventory)

`ABI_STABILITY.md` §3.3 requires every `#[repr(C)]` struct the **embedder fills
in** to carry `struct_size` first, and every entry point that reads one to
reject a size smaller than the one compiled into the library
(`is_compatible_struct_size`, `ffi/capi/lib.rs`). All 20 `#[repr(C)]` items in
`ffi/capi/` are accounted for below.

**Embedder fills, library reads — validated.**

| Struct | Passed | Checked at |
|---|---|---|
| `ServoDelegate` | by value | `lib.rs` `servo_builder_set_delegate` |
| `ServoEventLoopWaker` | by value | `lib.rs` `servo_builder_set_event_loop_waker` |
| `ServoWebViewDelegate` | by value | `webview.rs` `servo_webview_builder_set_delegate` |
| `ServoClipboardDelegate` | by value | `webview.rs` `..._builder_set_clipboard_delegate` |
| `ServoGamepadDelegate` | by value | `webview.rs` `..._builder_set_gamepad_delegate` |
| `ServoGamepadHapticEffectInfo` | via pointer (out) | `gamepad.rs` `..._haptic_effect_request_info` |
| `ServoCookieFilter` | via pointer (in) | `cookie_manager.rs` `servo_site_data_delete_all_cookies` — **added by W2.5** |
| `ServoProxyConfig` | via pointer (in) | `request_context.rs` `..._set_proxy_config` — **added by W2.5** |

The last two are the only structs the embedder allocates *and* passes by
pointer, so a caller built against a smaller header would have been read past
the end of its own allocation. Neither had a `struct_size`; W2.5 added one while
the API is still pre-1.0 and nothing in the tree called either struct, which is
why the change is a MINOR bump (0.2.0 → 0.3.0) instead of a break for an
existing population.

**Library fills, embedder reads — no runtime guard possible, and why that is
safe.** `ServoCookie`, `ServoSiteData`, `ServoCertificateError`,
`ServoScreenGeometry`, `ServoRgbColor`, `ServoContextMenuItem`,
`ServoContextMenuParams`, `ServoFileFilterPattern`, `ServoFilePickerParams`,
`ServoSelectElementOption`, `ServoSelectElementOptionOrOptgroup`,
`ServoSelectElementParams`.

Servo owns the storage for all twelve and hands out a pointer (or a by-value
copy) to its own struct; the embedder never supplies one, so there is no
caller-controlled size to check, and no path where Servo writes past memory the
embedder owns. A host built against an older header reads a *prefix* of the
struct, which is why these are append-only: adding, removing or reordering a
field is the ordinary breaking change of §2.3 and needs the version discipline,
not a runtime guard. `ServoScreenGeometry` does carry a `struct_size` (Servo
fills in the value) so a host can tell how many fields this build provided.

### B.1 Handle validation — liveness registry (W2.5)

**The problem.** Handles are raw `Box` pointers: created with
`Box::into_raw(Box::new(..))` and destroyed with `Box::from_raw(..)`
(`ffi/capi/options.rs::servo_options_create` / `::servo_options_free`, the same pair in
`ffi/capi/rendering_context.rs`, throughout `value.rs`). If an embedder frees a handle
twice, uses it after free,
or passes the wrong handle type, the engine reads freed or aliased memory. That
is undefined behaviour by contract, but it is silent and hard for embedders to
diagnose.

**What landed (W2.5).** `ffi/capi/lib.rs::handle_registry` is a
`Mutex<HashSet<(usize, HandleKind)>>` keyed on the handle's **address**, so a
check is a lookup and never a dereference — safe even on a pointer the process
never owned. `servo_options_create` registers; `servo_options_free`
unregisters, so a second free is `ServoError::InvalidArgument` instead of a
double free; `servo_builder_set_options`, which consumes the handle, does the
same; and each of the 20 `servo_options_set_*` / `servo_options_enable_*` entry
points returns `ServoError::InvalidArgument` when the handle is not live.
Verified by
`tests/capi/c/test_api.c::test_options_handle_validation` (garbage pointer
`(ServoOptions*)0x1`, freed handle, double free, consumed handle; the process
survives each).

Deviations from the sketch below, both deliberate:

- **Not gated on `debug_assertions`.** Release is where embedders ship, which is
  the same reason `entry_checks::check_main_thread` is compiled into every
  build. The cost is one mutex-guarded hash lookup per entry point of an adopted
  handle type; no other handle type pays anything.
- **Address lookup instead of a magic field.** A magic value has to be read
  through the pointer to find out whether the pointer is valid, which is exactly
  the dereference that must not happen. The trade-offs are that address reuse
  can defeat the check (a recycled address looks live) and that a live handle of
  another type is a miss only because of the `HandleKind` tag — the tag is
  unexercised until a second kind is adopted, and exists so that the next
  adopted type cannot silently satisfy another type's entry point.

**Still uncovered**, registered in `AGENT_BUILD_PLAN.md` §D.2.1: the other
handle types (`ServoBuilder`, `RenderingContext`, `ServoValue`,
`ServoUserContentManager`, `ServoWebViewBuilder`, `Servo`,
`ServoSiteDataManager`, `ServoRequestContext`). Each is the same mechanical
edit — register in the constructor, check in the entry points, unregister where
ownership moves — plus one `HandleKind` variant.

**Original design sketch (kept for the record).**

1. A `cfg(debug_assertions)`-gated `Mutex<HashSet<usize>>` with a `HandleKind`
   tag; register on creation, unregister on free.
2. Checked constructors/destructors emitting a diagnostic on a miss instead of
   undefined behaviour.
3. Adopt incrementally, starting with the handle types that cross into callbacks
   and have documented lifetime hazards.
4. Extend the C tests to assert that a double free and a use-after-free are
   *diagnosed*.

The cheaper alternative considered and rejected — a `u32` magic value as the
first field of each concrete handle struct — cannot work for the types this
matters most for, because `ServoOptions` is a type alias for `servo::Opts` and
`RenderingContext` for another upstream type: there is no field to add.

### B.2 Temporary pointers and thread affinity (make contracts explicit)

- **Thread affinity: landed.** Handles are main-thread-affine and the FFI
  boundary enforces the owning thread in all builds, returning
  `SERVO_ERROR_WRONG_THREAD` on violations (`ffi/capi/lib.rs::entry_checks`).
  The thread-agnostic exceptions (pure queries, plain callback accessors) are
  listed in `ABI_STABILITY.md` §5.1.
- **Temporary pointers: documented, not enforced.** The rule — a pointer handed
  to a callback is valid only for the duration of that call unless the
  callback's own documentation says otherwise — is stated on each delegate type
  and in `C_API_GUIDE.md`, and the callbacks that must retain anything copy it.
  Still missing is the second step: a debug mode that poisons a callback's
  buffer when it returns, so that a late read by the embedder is observable in a
  test. Registered in `AGENT_BUILD_PLAN.md` §D.2.1.

### B.3 `Send`/`Sync` assertions

`ffi/capi/protocol_handler.rs` asserts `unsafe impl Send/Sync` for a
caller-supplied handler, which the compiler cannot verify. The decision taken is
option (a): the requirement ("`callback` and `user_data` must be thread-safe
(`Send` + `Sync`) as protocol handlers may be called concurrently from network
or worker threads") is documented in the `# Safety` section of
`servo_builder_register_protocol_handler`, and the assertion itself carries the
rationale. The remaining piece of (a) — a test that exercises the contract under
concurrency — needs a real protocol-handler workload rather than a synchronous
assertion, so it belongs with the stress/fuzz work in W2.8 and is registered in
`AGENT_BUILD_PLAN.md` §D.2.1.

---

## Verification checklist (required before any of this lands)

- `cargo test -p servo-net` and the `cookies`/`same-site`/`fetch` WPT suites
  (they gate the already-landed `SameSite` change; see `SECURITY_AUDIT.md` §4.3).
- `cargo test -p servo-capi-tests --locked` for every FFI change (builds the
  cdylib with `cargo cinstall` and runs the C/C++ tests).
- `cargo test -p background_hang_monitor` for BHM changes.
- `mach test-tidy` (includes `cargo-deny`) and the `embedding-api.yml` symbol
  allowlist check (`ffi/capi/symbols.txt`) — any new C entry point must be added
  there deliberately.

## See also

- [`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) — findings and §9 priorities.
- [`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) — the wider assessment.
- [`ABI_STABILITY.md`](ABI_STABILITY.md) — the compatibility policy the FFI
  changes above must respect.
