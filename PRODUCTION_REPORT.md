# Servo — Production Readiness Report & Master Work List

Status: **Report** · Branch: `embedding/cef-parity` · Date: 2026-10-16

This is the top-level answer to one question: *is Servo (this fork, this branch)
production-ready and feature-rich like Blink (Chromium), CEF, and WebKit — and if
not, **everything that has to be done** to get there, across Android, Linux,
macOS, Windows and iOS.*

It is the index document. Detailed analysis lives in `docs/embedding/`; this file
states the verdict with evidence, and then enumerates the full remaining work as
prioritised workstreams.

| Related document | What it holds |
|---|---|
| [`docs/embedding/PRODUCTION_READINESS.md`](docs/embedding/PRODUCTION_READINESS.md) | Scorecard vs the incumbents, criterion by criterion |
| [`docs/embedding/SECURITY_AUDIT.md`](docs/embedding/SECURITY_AUDIT.md) | Security findings and prioritised remediation |
| [`docs/embedding/HARDENING_PLAN.md`](docs/embedding/HARDENING_PLAN.md) | Stability (hang monitor) and FFI hardening designs |
| [`docs/embedding/IOS_PORT_SPIKE.md`](docs/embedding/IOS_PORT_SPIKE.md) | The iOS feasibility decision |
| [`docs/embedding/ROADMAP.md`](docs/embedding/ROADMAP.md) | Embedding API phases 0–5 |
| [`docs/embedding/CEF_PARITY.md`](docs/embedding/CEF_PARITY.md) | Concept-by-concept CEF gap list |
| [`docs/embedding/COMPATIBILITY.md`](docs/embedding/COMPATIBILITY.md) | What works on which platform/feature |
| [`IMPLEMENTATION_REVIEW.md`](IMPLEMENTATION_REVIEW.md) | Independent verification of the first implementation pass (blockers B1–B7) |
| [`IMPLEMENTATION_REVIEW_PASS2.md`](IMPLEMENTATION_REVIEW_PASS2.md) | Independent verification of the repair + W5.2 + W4.1 pass (blockers B8–B10, R2 follow-up) |
| [`docs/embedding/AGENT_BUILD_PLAN.md`](docs/embedding/AGENT_BUILD_PLAN.md) | **Executable build plan for W2–W6** — ground rules, CI gates, per-ticket touch points and acceptance criteria |

---

## 1. Verdict

**No. Servo here is not production ready and is not comparable to Blink, CEF, or
WebKit — and it cannot be made so by a PR or a single development session.**

Evidence, not opinion:

- The project describes itself as *"a prototype web browser engine"*
  (`README.md` line 3).
- **Conformance.** `tests/wpt/meta` contains **18,753 expectation files** with
  **154,357 non-default expectation entries**, of which **146,721 are `FAIL`**,
  2,591 `ERROR`, 2,473 `NOTRUN`, 2,386 `TIMEOUT`, 122 `PRECONDITION_FAILED`,
  31 `CRASH`. (Counted directly from the tree this session.) Blink and WebKit
  clear roughly 95%+ of WPT; this is the gap that makes real-world sites break.
- **Sandboxing exists on only two platform/arch combinations.**
  `docs/embedding/COMPATIBILITY.md` §5: Linux x86_64 and macOS x86_64 → *yes*;
  Windows, Linux arm64, macOS arm64 (Apple Silicon), Android, iOS → *no*.
  The gate is `components/servo/servo.rs:1468` vs the `panic!` stub at
  `servo.rs:1484-1486`.
- **iOS does not exist as a port.** `ports/` contains only `servoshell`; there is
  no `aarch64-apple-ios` build, no iOS CI, and sandboxing is excluded by
  `cfg`.
- **The C API is version 0.1.0** (`ffi/capi/lib.rs:306-318`), pre-1.0, 218
  allowlisted symbols, no shipped prebuilt binaries.
- Servo has been in development since 2012; the incumbents have thousands of
  engineers on conformance and security. The gap is one of *scope*, not of a
  missing fix.

**Honest effort shape** (details in §4):

| Goal | Shape of the work |
|---|---|
| Embeddable on Linux/macOS/Windows with a documented C API | **Months** — mostly executing `ROADMAP.md` Phases 1–5 |
| Security-hardened with site isolation everywhere claimed | **Quarters** — the Windows/Android sandbox alone is a project |
| **Production ready like Blink/CEF on all five platforms** | **Years, funded team** — conformance is never "finished" |

---

## 2. Where things actually stand

### 2.1 What works today (evidence)

- **Windows is a first-class CI target.** `.github/workflows/windows.yml`
  builds `servoshell`, runs `mach smoketest` — which really starts the browser on
  `tests/html/close-on-load.html` (a page whose `window.onload` calls
  `window.close()`), so startup, parse, JS execution, load lifecycle and clean
  shutdown are all exercised — then `mach test-unit`, `mach package` (a real
  installer artifact), and an optional `capi` job running
  `cargo test -p servo-capi-tests --locked`.
- **The embedding C API covers navigation, input and rendering.**
  `servo_webview_load`/`load_request`, back/forward/scroll, key/mouse/wheel/touch
  events, focus, IME, zoom, find, JS evaluation, custom schemes, resource
  interception, and three rendering contexts (windowed / offscreen-OSR /
  software). Header is `cpp_compat` with a C++17 RAII wrapper.
- **Capability queries exist**, so an embedder can ask — not assume — whether
  multiprocess/sandboxing are available (`servo_capi_is_sandboxing_supported()`),
  which is the correct design given §1's platform reality.
- **Memory safety by construction** for the large class of C/C++ memory bugs —
  Servo's single biggest structural advantage over Blink/WebKit's C++ cores.

### 2.2 What is verified vs unverified right now

| Claim | Verification state |
|---|---|
| Windows builds, smoketests, unit-tests, packages | **Verified by CI on every push** |
| C API compiles + tests on Linux/macOS/Windows | **Verified** (`embedding-api.yml`, `windows.yml:capi`) |
| Web conformance on Windows | **Not measured** — `windows-wpt.yml` now exists in `.github/workflows/` but has never been dispatched (the workflow API returns 404 for it) |
| Windowed/GPU present path on Windows | **Not covered by CI** — runners are headless, tests assert windowed context returns `NULL` |
| This pass's Rust changes (SameSite, panic fix) | **Unverified** — no toolchain here; must be proven by `cargo test -p servo-net` + WPT `cookies`/`same-site`/`fetch` |
| `examples/c-embedder` vs the real generated header | **Unverified** — only compiled against a stub |
| `.github/workflows/capi-artifacts.yml` | **Verified** — green run `39145918170` on Linux/macOS/Windows (prebuilt `servo-capi` artifacts + `examples/c-embedder` compiled and linked against the installed sysroot) |

### 2.3 Embedding into an existing Windows browser (blanc-tauri)

Asked during this pass; short version of the answer:

- **Not a drop-in.** Tabs are created with `tauri::WebviewBuilder::new(...)`
  (`blanc-tauri/src-tauri/src/webview.rs:327`) → Tauri runtime → wry →
  **WebView2**; Tauri exposes no engine-swap hook.
- blanc additionally binds **directly to WebView2 COM**
  (`webview2-com`, `ICoreWebView2`/`_2`/`_4`) for request interception,
  downloads and `CapturePreview` screenshots — each would need reimplementation.
- Tauri's IPC (`window.__TAURI__`, `blanc:*` events) is wry-provided and would
  not cross into a Servo tab.
- **Feasible as an experimental side-by-side tab** (offscreen `servo-capi`
  rendering alongside WebView2), **not as a production engine swap** — and for a
  browser, §1's conformance gap is disqualifying for real sites.

---

## 3. What was done in this pass

All changes are **uncommitted** on `embedding/cef-parity`.

**Shipped code (needs CI validation):**

1. **`SameSite` cookie enforcement per rfc6265bis** — `SameSiteContext` and
   `appropriate_for_url_with_context` (`components/net/cookie.rs`), context-aware
   filtering (`components/net/cookie_storage.rs`), request-derived context
   (`components/net/http_loader.rs`), tests
   (`components/net/tests/cookie.rs`). Lax is limited to
   `Destination::Document` (main frame) so Lax cookies never leak into cross-site
   iframes. **Out of scope and documented as such:** CHIPS/partitioned cookies
   (requires partition-key storage).
2. **Fixed a reachable panic** on web-content-triggerable input
   (`components/script/dom/bindings/error.rs`: `panic!` → graceful `ErrorInfo`
   fallback).

**Documentation:**

3. `SECURITY.md` — proper vulnerability-reporting policy (was 3 lines).
4. `docs/embedding/SECURITY_AUDIT.md` — findings + prioritised remediation (§9).
5. `docs/embedding/HARDENING_PLAN.md` — **design only**: hang-monitor
   kill-script and FFI hardening, with exact file/line touch points.
6. `docs/embedding/IOS_PORT_SPIKE.md` — **design only**: 6-experiment go/no-go
   spike; recommends declaring iOS unsupported until it passes.

**Distribution:**

7. `examples/c-embedder/{main.c,CMakeLists.txt,README.md}` — standalone C11
   embedder; passes tidy license/whitespace checks and
   `gcc -std=c11 -Wall -Wextra -Werror -pedantic` against a stub header.
8. `.github/workflows/capi-artifacts.yml` — prebuilt `servo-capi` artifacts for
   Linux/macOS/Windows; **green run `39145918170`** on all three platforms
   (artifacts named `servo-capi-0.1.0-<platform>-<sha>`).

**Explicitly *not* done here:** WPT conformance, the Windows sandbox, iOS, and
any change requiring a build to validate.

---

## 4. Everything that has to be done

Workstreams, ordered by priority. Effort: **S** days · **M** 1–3 weeks ·
**L** 1–3 months · **XL** quarters/years.

### W1 — Conformance (C1) · XL · *the blocking one*

The 146,721 `FAIL` expectations are not a bug list that can be "closed"; they
are the accumulated difference between an engine started in 2012 and a moving
spec target. Rewriting expectations would game the metric, not fix the engine.

| # | Task | Effort |
|---|---|---|
| W1.1 | **Add `.github/workflows/windows-wpt.yml`** so conformance is *measured* on Windows instead of only Linux/macOS. Today nothing proves Windows-specific rendering behaviour. | S |
| W1.2 | Produce a real **pass-rate baseline** (a full WPT run yields pass rate; expectation files only show known failures) and publish it, tracked over time. | S |
| W1.3 | **Target-site driven conformance**: pick a defined set of real sites/workloads, run them, triage failures by root cause, fix the top causes. This is how a small team moves the number meaningfully. | XL |
| W1.4 | **Upstream everything** to `servo/servo`; do not let this fork diverge (fork divergence makes every future rebase a conformance tax). | ongoing |
| W1.5 | Ongoing: keep WPT expectations current; investigate `ERROR`/`NOTRUN`/`TIMEOUT` clusters (2,591/2,473/2,386) — these often indicate harness/timing issues distinct from feature gaps. | L |

### W2 — Security architecture (C2) · XL · *second hardest*

| # | Task | Effort |
|---|---|---|
| W2.1 | **Windows renderer sandbox.** Replace/port `gaol` (Linux/macOS-only) with a Windows broker — AppContainer/restricted-token model; prior art exists in Chromium. Today `create_sandbox()` panics on Windows (`servo.rs:1484`). | L–XL |
| W2.2 | **Android + iOS sandboxing**, and the arm64 gap (Linux arm64 and Apple-Silicon macOS are also currently *no*). | XL |
| W2.3 | **Site isolation / origin-keyed processes** — the constellation currently reuses event loops; sandboxed origins are handled but origin-keyed isolation is not Blink-class. | XL |
| W2.4 | **Multiprocess bootstrap for C embedders** (`ARCHITECTURE.md` §8.1) so embedders get process isolation without using `servoshell`. *Interim mitigation that already works:* `multiprocess` is available on Windows (only `opts.sandbox` gates the panic), giving crash isolation without OS containment. | L |
| W2.5 | **Ship the `HARDENING_PLAN.md` Part B FFI hardening**: handle magic/state cookies, temporary-pointer and thread-affinity contracts, `Send`/`Sync` assertions. | M |
| W2.6 | **CHIPS / partitioned cookies** (needs partition-key storage; SameSite is done, this is the documented remainder). | M |
| W2.7 | Finish the content-security review: CSP/COOP/CORP/CORP/XCTO/TLS behaviour and per-spec defaults (`SECURITY_AUDIT.md` §3, §5). | L |
| W2.8 | **Fuzzing + security regression tests** (`SECURITY_AUDIT.md` §8, `HARDENING_PLAN.md` §A.3) — process-level panic suppression plus coverage-guided fuzzing of parsers/network/IPC. | L |
| W2.9 | Operationalise `SECURITY.md`: intake, triage SLA, advisory publication, dependency cadence (`deny.toml`). | S |

### W3 — Platform coverage (C3) · XL · *all five targets*

| Platform | State | What must be done |
|---|---|---|
| **Linux** x86_64 | Best supported; sandbox **yes** | Keep as reference target; W1 conformance; arm64 sandbox gap (W2.2). |
| **macOS** x86_64/arm64 | Mature; sandbox only on x86_64 | arm64 sandbox (W2.2); signed + notarized releases (W5.3). |
| **Windows** x86_64 | Builds/smoketests/packages; sandbox **no** | W1.1 (WPT), W2.1 (sandbox), W4.5 (windowed/GPU path manually validated on real hardware), multiprocess for embedders (W2.4). |
| **Android** | Builds via `servoshell` app | Ship an **embeddable AAR library** instead of only a demo app; sandbox (W2.2); keep `android_smoketest`. |
| **iOS** | **No port, no CI** | Run `IOS_PORT_SPIKE.md` §3 (6 time-boxed experiments) → **decide**. Either commit a dedicated iOS workstream (UIKit embedding, `aarch64-apple-ios`, JIT/entitlement rules under Apple's runtime policy, sandboxing) **or declare iOS unsupported explicitly** in README, C API docs and CI. Pretending it works is the worst outcome. |

### W4 — Embedding API to parity (C4) · L · *execute `ROADMAP.md`*

Already done on this branch: Phase 0 foundations (docs, ABI policy, panic
audit harness, `embedding-api.yml`), read/control surface (Phase 1),
screen geometry / move / resize / popups, network-manager accessors, user
script/stylesheet injection, `struct_size` versioning, `symbols.txt` allowlist
(218 symbols), `cpp_compat` header + C++ test, capability queries.

Remaining:

| # | Task | Effort |
|---|---|---|
| W4.1 | **Native ↔ JS message router** — documented as *missing* (`ARCHITECTURE.md` §5.4). First phase requiring real engine plumbing (script object + `EmbedderMsg` variants). | L |
| W4.2 | Input/rendering parity completion (ROADMAP Phase 2): clipboard, gamepad, embedder controls (select/color/file/context menus/dialogs), cursor/status/history delegates. | L |
| W4.3 | Networking/content control (Phase 3): custom scheme handlers + resource interception + cookie/storage managers fully exposed in C. | L |
| W4.4 | DevTools/remote debugging exposure and console callbacks in C (Phase 4). | M |
| W4.5 | **Manual validation of the windowed/GPU rendering path on Windows and macOS real hardware** — CI is headless and cannot prove it. | S |
| W4.6 | `Send`/`Sync` handles *or* an explicit documented main-thread-only contract (Phase 5) — evaluate first, document rather than over-promise. | M |
| W4.7 | Language bindings beyond C/C++ (Python etc.), kept out-of-tree if they need their own cadence. | L |

### W5 — Stability, distribution, ecosystem (C6/C7) · L

| # | Task | Effort |
|---|---|---|
| W5.1 | **Hang monitor: enable by default and kill permanently-hung scripts** — implement `HARDENING_PLAN.md` §A.1 (`SignalComponentExit` → `signal_to_exit()` on `HangAlert::Permanent` → constellation tears down the script thread), behind a pref. Today it is opt-in and warn-only. | M |
| W5.2 | **Prove `capi-artifacts.yml`**: dispatch it, fix failures, then compile `examples/c-embedder` against a real `cargo cinstall` sysroot in CI. | M |
| W5.3 | **Prebuilt binaries per platform** (Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64) with header + CMake/pkg-config, **signed** (macOS notarization, Windows Authenticode), checksums, release cadence. | L |
| W5.4 | **Move the C API from 0.1.0 to a 1.0 ABI commitment**: freeze the symbol set, exercise the deprecation policy once, publish support window. This is policy + release discipline, enforced by the existing CI allowlist. | M |
| W5.5 | Remaining reachable-panic review (`HARDENING_PLAN.md` §A.2) as a tracked, owner-per-item list. | M |
| W5.6 | Observability: cdylib size, startup time, throughput, crash rate tracked in CI to catch regressions. | M |
| W5.7 | Long-uptime/memory-discipline testing (leak detection over repeated load/unload). | L |

### W6 — Rendering & media (C5) · XL

- Validate and document GPU compositing/WebGL/WebGPU state; close gaps.
- **Video/audio playback and DRM (EME)** — incumbents are years ahead; this is a
  whole subsystem (GStreamer pipeline work, platform decoders).
- **Accessibility** maturity (screen readers, a11y tree exposure on all platforms).
- Extensions and enterprise policy (if targeting Chrome-parity consumers).

---

## 5. The four blockers asked about — fixability summary

| Blocker | Fixable? | Where it lands |
|---|---|---|
| ~146k WPT `FAIL` | **No — not by a small team** | W1 (measure it, target sites, upstream). Years. |
| No Windows sandbox | **Yes — bounded but large** | W2.1; interim: `multiprocess` (crash isolation) already works on Windows |
| v0.1.0, no prebuilt binaries | **Yes — most tractable** | W5.2, W5.3, W5.4 |
| Hang monitor opt-in + warn-only | **Yes — most tractable code fix** | W5.1 (`HARDENING_PLAN.md` §A.1) |

---

## 6. Immediate next actions (next 1–2 weeks)

1. **Push this branch and get CI green.** Everything in §3 is unverified here:
   `cargo test -p servo-net`; WPT `cookies` / `same-site` / `fetch`;
   `cargo test -p servo-capi-tests --locked`; `mach test-tidy`.
2. **Dispatch `capi-artifacts.yml`** and fix it until it produces three working
   artifacts; then build `examples/c-embedder` against the real sysroot (W5.2).
3. **Add `windows-wpt.yml`** (W1.1) so the Windows conformance number exists.
4. **Implement W5.1** (hang-monitor kill) behind a pref — the highest-value code
   change still outstanding; requires CI validation of no regressions.
5. **Decide the iOS question** (`IOS_PORT_SPIKE.md` §4) and write the answer into
   README + `COMPATIBILITY.md`.
6. **Decide the blanc integration question** (§2.3): recommend *don't swap*;
   option A is a bounded experimental spike if engine independence is the goal.

---

## 7. Honest closing statement

This branch has produced a real, documented, memory-safe C embedding foundation,
proper security documentation, a SameSite implementation, a sample embedder and
a prebuilt-artifact workflow — plus, in this pass, an honest assessment and a
complete work list.

What it has **not** produced, and what no single session can produce, is Blink-class
conformance, a Windows/iOS/Android sandbox, or five-platform production maturity.
Those are measured in **years of a funded team**. Reporting anything else would be
dishonest and would set the project up for a worse failure later: shipping an
engine that appears ready and then breaks on ordinary websites.
