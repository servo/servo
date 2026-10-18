# Servo Embedding — Hardening Plan (Stability & FFI)

Status: **Plan — not implemented** · Branch: `embedding/cef-parity` · Baseline
commit: `68719ce75c2`

This document turns the *recommendations* in
[`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) §9 (items 2, 3, 5) and
[`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) §9 (items 2, 3) into concrete,
bounded engineering steps with exact touch points.

> **Nothing here is implemented.** These are designs, not landed code. They are
> written to be reviewable and executable by someone who can build and test the
> engine (the environment this plan was produced in could not: no MSVC/clang/
> GStreamer, empty `target/`). Each step names the files to change and the tests
> that must gate it.

---

## Part A — Stability

### A.1 Background hang monitor (BHM)

**Status: implemented.** Both flags default to `true` and a permanent hang
terminates the hung component's script; the embedder observes the interrupted
JavaScript call completing with an error. The design text below is kept for
provenance; claims that no longer hold are corrected in place.

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
  `while (true) {}` is interrupted with an error within the permanent timeout,
  and the suite's final `servo_free` proves teardown stays clean.

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

### B.1 Debug handle validation

**The problem.** Handles are raw `Box` pointers: created with
`Box::into_raw(Box::new(..))` and destroyed with `Box::from_raw(..)` (e.g.
`ffi/capi/options.rs:35,810`; `ffi/capi/rendering_context.rs:52,212`;
throughout `value.rs`). If an embedder frees a handle twice, uses it after free,
or passes the wrong handle type, the engine reads freed or aliased memory. That
is undefined behaviour by contract, but it is silent and hard for embedders to
diagnose.

**Design (opt-in, debug-only).**

1. Add a `cfg(debug_assertions)`-gated handle registry in `ffi/capi/lib.rs`: a
   `Mutex<HashSet<usize>>` (lock-free not needed) plus a small `HandleKind` tag.
   Register on creation, unregister on free.
2. Add checked constructors/destructors (e.g.
   `debug_register_handle(ptr, Kind::Options)` /
   `debug_take_handle(ptr, Kind::Options)`). On a miss — unknown pointer, wrong
   kind, or double free — emit a clear diagnostic (`ServoError::InvalidHandle`
   or a log line) instead of undefined behaviour. Do **not** change release
   behaviour or the ABI; keep it behind `debug_assertions`.
3. Adopt it incrementally: start with the handle types that cross into callbacks
   and have documented lifetime hazards (`ServoOptions`, `ServoBuilder`,
   `RenderingContext`, `ServoValue`, `ServoUserContentManager`,
   `ServoWebViewBuilder`, `Servo`).
4. Existing C tests can be extended to assert that a double-free and a
   use-after-free are *diagnosed* in debug builds, and skip that assertion in
   release.

**A cheaper first cut.** If a full registry is too invasive, embed a
`u32` magic value as the first field of each concrete handle struct and validate
it on entry/free under `debug_assertions`. It catches wrong-type and stale-handle
misuse (not address reuse) with a one-field change per struct.

### B.2 Temporary pointers and thread affinity (make contracts explicit)

Several callbacks pass pointers valid **only for the duration of the call**
(document the same rule as the CEF docs). Today this is prose only
(`docs/embedding/C_API_GUIDE.md`, `CEF_PARITY.md`). Steps:

- Centralise the rule in one place and reference it from every callback's
  doc-comment in `ffi/capi/webview_delegate.rs` and `cert_error.rs`.
- Add a debug mode that, when a callback returns, poisons the buffer it handed
  out (writes a sentinel) so a late read by the embedder is observable in tests.
- Document and test the **thread-affinity contract**: handles are
  main-thread-affine; assert `is_main_thread()` under `debug_assertions` in the
  entry macro in `ffi/capi/lib.rs`.

### B.3 `Send`/`Sync` assertions

`ffi/capi/protocol_handler.rs` asserts `unsafe impl Send/Sync` for a
caller-supplied handler, which the compiler cannot verify. Either (a) keep it but
document the requirement precisely at the FFI entry that installs the handler,
plus a test that a handler is only invoked from the expected thread, or (b)
marshal the handler calls onto the thread that installed them. Decide and
document; do not leave the assertion unexplained.

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
