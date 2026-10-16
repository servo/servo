# Servo on iOS — Feasibility Spike

Status: **Spike plan — not implemented** · Branch: `embedding/cef-parity` ·
Baseline commit: `68719ce75c2`

This document is the deliberate answer to
[`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) §9 item 4: *decide the iOS
question instead of pretending it works.* It records what exists today, why iOS
is qualitatively harder than the other four target platforms, and a time-boxed
spike that would produce a real go/no-go decision.

> **Nothing here is implemented, and no claim is made that Servo runs on iOS.**
> There is no `ports/ios`, no iOS CI, and no iOS release. This is a plan to find
> out whether a port is viable.

---

## 1. Current state (evidence)

- **No iOS port.** `ports/` contains only `servoshell`. There is no `ports/ios`,
  no Xcode project, no `aarch64-apple-ios` build target, and no iOS job in
  `.github/workflows/`. `README.md` does not list iOS.
- **iOS is explicitly excluded from several subsystems**, alongside
  Windows/Android/ARM/RISC-V:
  - Sandboxing: `components/servo/servo.rs:1474-1486` — `create_sandbox()`
    `panic!`s on iOS.
  - The `gaol` sandbox import is gated out on iOS
    (`components/servo/servo.rs:16-26`).
  - `default_config_dir()` is gated out on iOS
    (`ports/servoshell/prefs.rs:141-147`).
- **The cfg gates exist but only to *exclude* iOS**, i.e. the tree is written so
  that non-desktop Unix-like targets don't accidentally take desktop paths. That
  is not a port; it is the absence of one made explicit.
- **No mobile embedding API.** Android ships only the `servoshell` app, not a
  reusable library (see `PRODUCTION_READINESS.md` §4). iOS today has nothing at
  all. The `ffi/capi` crate is the most promising embeddable surface, but it
  depends on `servo`, which carries the platform gates above.

---

## 2. Why iOS is not "one more target"

The other four platforms are variations on running a JIT-based, multi-process
engine on a general-purpose OS. iOS actively forbids several of the assumptions
such an engine makes:

1. **JIT and dynamic code generation.** App Store apps may not generally map
   memory as writable-and-executable; the JIT entitlement is not available to
   third-party browser engines. Servo's JavaScript engine (SpiderMonkey) ships
   baseline + optimizing JITs. Running interpreter-only is possible but is a
   large performance regression and needs to be validated end-to-end.
2. **App Store policy.** Apple requires apps that browse the web to use
   WebKit/`WKWebView`. This constrains what a Servo-on-iOS embedding may legally
   ship as, independent of the engineering.
3. **No general process spawning / sandboxing.** Servo's content-process
   isolation relies on `fork`/`spawn` and the `gaol` sandbox — neither exists in
   the same form on iOS. The entire security model would have to change, or be
   dropped and disclosed via `servo_capi_is_sandboxing_supported()`.
4. **Graphics and media.** Rendering must be proven on iOS's GL ES / Metal stack
   (Servo's webrender path is GL-oriented today; a Metal/ANGLE path must be
   validated). Media uses GStreamer on desktop; iOS would need an AVFoundation
   backend or a GStreamer iOS build. Both are workstreams, not config.
5. **Lifecycle and memory.** iOS apps are suspended/terminated aggressively and
   have tighter memory budgets; the engine's thread model and memory reporting
   would need an iOS story.

These are why `PRODUCTION_READINESS.md` calls iOS "the single largest platform
gap" and "a major, dedicated work stream" — not a checklist item.

---

## 3. Time-boxed spike: the questions to answer

The spike is deliberately small and read-only: it produces a go/no-go and a
sizing estimate, not a port. Each question has a concrete experiment.

| # | Question | Experiment | Success signal |
|---|----------|-----------|----------------|
| Q1 | Does the core Rust engine even compile for `aarch64-apple-ios`? | `rustup target add aarch64-apple-ios`, then `cargo build -p servo --target aarch64-apple-ios` (interpreter-only if needed) and record every failing crate/dep. | A finite, categorized error list — no fundamental blocker (e.g. a dependency with no iOS support) |
| Q2 | Can the C API compile and link as an iOS static/dynamic library? | `cargo build -p servo-capi --target aarch64-apple-ios --release`; link the `examples/c-embedder` sample into a minimal Xcode app. | A `.a`/`.dylib` that a trivial Xcode project links and loads |
| Q3 | Does SpiderMonkey run without JIT on iOS? | Build with the JS engine's JIT disabled and run a DOM benchmark + the script test suite. | Correct execution with a measured (acceptable) performance delta |
| Q4 | Can we render offscreen on iOS? | Prove the software rendering context produces frames on device/simulator; scope the GL ES / Metal path. | A non-empty screenshot from `servo_webview_take_screenshot` |
| Q5 | What breaks without a sandbox? | Enumerate the code paths that assume `create_sandbox()` and either gate them out or document the loss. | A documented, compile-clean single-process iOS build |
| Q6 | What is the effort to a *credible* embedding? | Turn Q1–Q5 results into a work breakdown. | An estimate a maintainer can trust |

**Exit criteria:** a written report with the Q1–Q6 answers, a categorized list of
blockers, and a recommendation (proceed / defer / decline). If Q1 or Q3 has no
answer, the recommendation is "defer".

---

## 4. Recommended decision (to be confirmed by the spike)

Until a spike answers the questions above:

1. **Declare iOS unsupported, explicitly and publicly.** State it in
   `README.md`, `docs/embedding/C_API_GUIDE.md`, and
   `docs/embedding/COMPATIBILITY.md`. The runtime capability query should report
   it: `servo_capi_is_sandboxing_supported()` already returns false on iOS, but a
   plain "is this platform supported?" signal would be clearer.
2. **Do not add iOS to CI for the current `servoshell` app.** A CI job that only
   proves "the tree happens to compile" would imply support that does not exist.
   Run the spike's Q1 build in a scratch/workflow_dispatch job instead.
3. **Revisit only with a funded workstream.** iOS is a multi-quarter project
   (JIT policy, rendering backend, media backend, embedding API, security model).
   It should be an explicit roadmap decision, not an implied capability.
4. **Prefer the embedding API as the iOS vehicle** *if* the spike proceeds: a
   small C API that a host app links is a more realistic iOS integration than a
   full `servoshell` app, and it reuses the surface this branch already invests
   in (`ffi/capi`, `examples/c-embedder`).

---

## 5. Immediate, safe follow-ups (independent of the spike outcome)

- Add the explicit "iOS: not supported" statement to the docs listed in §4.1 so
  no reader infers support.
- Ensure the capability query surface lets an embedder detect unsupported
  platforms rather than assuming (see `HARDENING_PLAN.md` and
  `PRODUCTION_READINESS.md` §10 item 3).
- Keep the `target_os = "ios"` cfg gates in place; they are correct placeholders
  and should not be removed until Q1 passes.

## See also

- [`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md) §4, §9 item 4 — the iOS
  gap and the "decide deliberately" instruction.
- [`ROADMAP.md`](ROADMAP.md) — existing phased plan for the desktop/mobile
  embedding work.
- [`HARDENING_PLAN.md`](HARDENING_PLAN.md) — capability signalling and FFI
  hardening.
