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

**Where it lives**

- Config flag: `components/config/opts.rs` — `background_hang_monitor: bool`,
  default `false` (line 251).
- Shell flag: `ports/servoshell/prefs.rs` — `-B` / `--bhm` (line 384), piped to
  `opts` at line 762.
- C API: `ffi/capi/options.rs` — `servo_options_set_background_hang_monitor`
  (line 287) already plumbs through to `Opts::background_hang_monitor`.
- Worker: `components/background_hang_monitor/background_hang_monitor.rs`.
- Alert API: `components/shared/background_hang_monitor/lib.rs`
  (`HangAlert::{Transient,Permanent}`, `MonitoredComponentId`,
  `BackgroundHangMonitorExitSignal`).
- Consumer: `components/constellation/constellation.rs`
  `handle_request_from_background_hang_monitor` (line 1225) and
  `components/constellation/event_loop.rs`
  `send_message_to_background_hang_monitor` (line 206).

**What it does today**

- It samples monitored script components and, when a component's
  `last_activity` exceeds its **transient** or **permanent** hang timeout, sends
  `HangAlert::Transient` / `HangAlert::Permanent` to the constellation
  (`background_hang_monitor.rs:375-407`). Each alert is sent at most once until
  activity resumes.
- The constellation's handler does one thing:
  `warn!("Component hang alert: {:?}", message)` — and carries a TODO for a
  "kill script" workflow (line 1226).
- The script thread **already implements the kill primitive**. It registers a
  `BHMExitSignal` (`script_thread.rs:427-437`) whose `signal_to_exit()` sets a
  `closing` flag and requests a JS interrupt callback; the registered
  `interrupt_callback` (`script_thread.rs:440-446`) then returns `false`, which
  makes `can_continue_running_inner()` false and runs
  `prepare_for_shutdown_inner()` (`script_thread.rs:997-1005`). In other words,
  calling `signal_to_exit` on a script thread **stops its script**, by design.
- The gap: `signal_to_exit` is only ever called during **shutdown** — on
  `BhmMessage::Exit`, on control-port disconnect, and on late registration while
  shutting down (`background_hang_monitor.rs:260-287, 316-323`). It is **never**
  called in response to a hang alert. So today a permanently hung script is
  reported and then left running.

#### A.1.1 Decide the default (policy, one line)

`background_hang_monitor` is off by default. Turning it on by default is a
one-line change in `components/config/opts.rs`, but it is a **behavioural**
change (new threads, new sampling) and should be made deliberately:

1. Turn it on by default for desktop embedders (it is already reachable from the
   C API), keeping `--bhm=false` / the C setter as the opt-out.
2. Gate it behind the existing `multiprocess`/sandbox story only if sampling
   proves to be a problem in single-process mode.
3. Land it with the kill-script workflow below so alerts have an effect.

#### A.1.2 Kill-script workflow (the real work)

Goal: when a **permanent** hang is reported for a script component, stop that
component's script instead of only logging.

The clean path reuses the existing exit-signal plumbing rather than inventing a
new cross-thread mechanism:

1. **Add a per-component control message.**
   `components/shared/background_hang_monitor/lib.rs`: extend
   `BackgroundHangMonitorControlMsg` (currently only `Exit`) with a variant that
   names the component, e.g.
   `SignalComponentExit(MonitoredComponentId)`.
2. **Handle it in the worker.**
   `components/background_hang_monitor/background_hang_monitor.rs`, next to
   `BhmMessage::Exit`: on `SignalComponentExit(id)`, look up
   `self.monitored_components.get_mut(&id)` and call
   `component.exit_signal.signal_to_exit()`. The existing script-thread signal
   does the rest.
3. **Trigger it from the constellation.**
   `components/constellation/constellation.rs:1225`: in
   `handle_request_from_background_hang_monitor`, on `HangAlert::Permanent`,
   send `SignalComponentExit` via
   `self.send_message_to_all_background_hang_monitors(..)` (single-process) or
   the per-process control sender (multiprocess). Keep the `warn!` (it is useful
   diagnostics and should log before the kill).
4. **Gate it.** Killing a page on a *permanent* hang is a user-visible,
   destructive recovery. Make it opt-in at first (a pref such as
   `hang_monitor_kill_script`, defaulting off), then consider defaulting it on for
   the shell once validated. Never kill on a **transient** alert — those are
   normal (GC pauses, slow layout) and recover on their own.

**Why per-component and not the global `Exit`:** `BhmMessage::Exit` signals
*every* component and is part of shutdown. A hang recovery must affect only the
hung component, or one bad page take down the whole embedder.

**Tests to add**

- A `background_hang_monitor` unit test mirroring
  `components/background_hang_monitor/tests/hang_monitor_tests.rs` that
  registers a component with a mock exit signal and asserts
  `SignalComponentExit` calls it exactly once and only for the named component.
- A `constellation` test (or a servoshell integration test) that a synthetic
  `HangAlert::Permanent` produces exactly one `SignalComponentExit` when the gate
  pref is on, and none when it is off.

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
