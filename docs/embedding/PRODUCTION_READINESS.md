# Servo Embedding — Production Readiness Assessment

Status: **Assessment** · Branch: `embedding/cef-parity` · Baseline commit: `0072748a623`

This document answers a blunt question honestly: *is Servo (this fork, this
branch) production ready, and is it feature-rich and powerful like Blink
(Chromium), CEF, and WebKit?* It separates the **engine** (libservo + the
`servoshell`/embedder surface) from the **embedding C API** this branch is
building (`ffi/capi`, `servo-capi`), and it compares both against what shipping
engines actually provide.

It complements [`AUDIT.md`](AUDIT.md) (inventory and technical debt),
[`CEF_PARITY.md`](CEF_PARITY.md) (concept-by-concept gap analysis), and
[`ROADMAP.md`](ROADMAP.md) (phased plan). Those documents describe *the embedding
API*. This one steps back to the whole engine and to production criteria.

> **Documentation only, plus one small hardening change.** Nothing here is a
> claim about code beyond what is cited. The only code shipped alongside this
> document is described in §10.

---

## 1. Bottom line

**No. Servo is not production ready and is not feature-comparable to Blink,
CEF, or WebKit — and it cannot be made so by a change, a PR, or a single
development session.**

Concretely:

- The project itself calls Servo "a prototype web browser engine" (`README.md`,
  line 3).
- It has **no iOS port** and **no iOS CI**. `ports/` contains only `servoshell`.
  Sandboxing explicitly is not supported on iOS, Windows, Android, ARM, or
  RISC-V (`components/servo/servo.rs:1485`).
- The C embedding API is at **version 0.1.0** (`ffi/capi/lib.rs:248-254`),
  pre-1.0, with no ABI stability guarantee yet.
- The engine's own compatibility work is tracked but far from Blink's; Servo
  has been in development since 2012 and, even with recent investment, remains
  a fraction of Chromium's conformance, security architecture, and platform
  reach.

Reaching "production ready like Blink/CEF on Android, Linux, macOS, Windows and
iOS" is a **multi-year effort by a sizeable, funded team** — not a task that can
be completed here. Claiming otherwise, or generating a large volume of
speculative code to look like progress, would be dishonest and would likely
destabilise the branch.

What *is* achievable, and what this pass delivers, is:

1. An accurate, evidence-based statement of where things stand (§3–§7).
2. A bounded, real hardening fix to the C API (§10).
3. A prioritised, honest path forward (§9).

---

## 2. What "production ready" actually means

These are the criteria against which shipping engines are judged. They are the
bar Blink, WebKit, and CEF clear; they are what an embedder means by "ready".

| # | Criterion | What it requires |
|---|-----------|------------------|
| C1 | **Web-platform conformance** | Near-complete HTML/CSS/DOM/JS behavior; high WPT pass rates; not observable as "that site doesn't work". |
| C2 | **Security architecture** | Multi-process sandboxing, site isolation, a hardened IPC boundary, regular security response, fuzzing. |
| C3 | **Platform coverage** | First-class Linux, macOS, Windows, **Android, iOS**; the ability to be embedded in host apps on each. |
| C4 | **Stable embedding API** | A versioned, ABI-stable surface (CEF/Chromium Embedded Framework) an embedder can link against for years. |
| C5 | **Rendering & media** | GPU-accelerated compositing, WebGL/WebGPU, video/audio playback, DRM, accessibility. |
| C6 | **Stability & performance** | Crash-free operation, predictable performance, memory discipline, long uptime. |
| C7 | **Distribution** | Prebuilt binaries, toolchains, documentation, release cadence, support. |

---

## 3. Scorecard against the incumbents

Ratings are qualitative and deliberately conservative. "CEF" is a framework
around Chromium, so it inherits C1/C2/C5.

| Criterion | Blink (Chromium / Android WebView) | WebKit (Safari / iOS WKWebView) | CEF | **Servo (this fork)** |
|-----------|-----------------------------------|--------------------------------|-----|----------------------|
| C1 Conformance | ★★★★★ | ★★★★★ | ★★★★★ | ★★☆☆☆ |
| C2 Security arch | ★★★★★ | ★★★★★ | ★★★★★ | ★★☆☆☆ |
| C3 Platforms | ★★★★★ | ★★★★★ | ★★★★☆ | ★★★☆☆ |
| C4 Stable embed API | ★★★★★ | ★★★★★ | ★★★★★ | ★★☆☆☆ |
| C5 Rendering/media | ★★★★★ | ★★★★★ | ★★★★★ | ★★★☆☆ |
| C6 Stability/perf | ★★★★★ | ★★★★★ | ★★★★★ | ★★☆☆☆ |
| C7 Distribution | ★★★★★ | ★★★★☆ | ★★★★★ | ★★☆☆☆ |

**Interpretation.** Servo's genuine strengths are a modern, memory-safe Rust
codebase, an independent engine (not a Chromium fork), and a clean embedding
foundation. Its weaknesses are exactly the ones that make an engine
"production": conformance breadth, the security architecture, mobile reach, and
ecosystem maturity.

---

## 4. Platform support matrix

| Platform | Builds / runs today | Sandbox / multi-process | Notes |
|----------|--------------------|--------------------------|-------|
| **Linux (x86-64/arm64)** | Yes; CI (`linux.yml`, `linux-arm64.yml`) | Sandbox path exists on Linux | Best-supported target. |
| **macOS (x86-64/arm64)** | Yes; CI (`mac.yml`, `mac-arm64.yml`) | Sandboxed content-process support | Mature desktop target. |
| **Windows (x86-64)** | Yes; CI (`windows.yml`) | **Sandbox not supported** (`servo.rs:1485`) | Runs single-process; security gap. |
| **Android** | Yes; CI (`android.yml`) | **Sandbox not supported** | Embedding story is the `servoshell` app, not a reusable AAR library. |
| **iOS** | **No port, no CI** | Unsupported (`constellation/sandboxing.rs`, `not(target_os="ios")`) | **The single largest platform gap.** No `ports/ios`; README does not list iOS. |
| **OpenHarmony (ohos)** | Partial; CI (`ohos.yml`) | Unsupported on ARM | Experimental. |

**iOS is not a "small remaining step."** A production iOS embedding (like
`WKWebView`) requires an iOS port, an `aarch64-apple-ios` build, JIT/entitlement
handling under Apple's runtime rules, an embedding API that fits UIKit, and a
sandboxing story — none of which exist. This alone is a major, dedicated work
stream.

---

## 5. Security posture

**Memory safety.** Servo is written in Rust, so the large class of
memory-corruption bugs that plague C/C++ engines is largely absent by
construction. This is Servo's single biggest structural advantage. It does *not*
by itself make the engine secure: logic bugs, sanitisation gaps, and the C FFI
boundary remain in scope.

**Sandboxing and site isolation.** A content-process sandbox exists on Linux and
macOS but is **explicitly unsupported on Windows, iOS, Android, ARM, and
RISC-V** (`components/servo/servo.rs:1485` panics there). Modern engines assume
a sandboxed renderer as the primary defence-in-depth layer; Servo cannot rely on
that on three of the five platforms the goal names.

**The C FFI boundary.** This is the one security surface this branch directly
owns, and it is in reasonable shape:

- Fallible entry points return a `ServoError` and are wrapped in a
  `catch_unwind` boundary (`ffi/capi/lib.rs`), and no profile sets
  `panic = "abort"`, so panics do not cross `extern "C"` uncaught.
- Pointer arguments are null-checked; string inputs are UTF-8 validated.
- Handles are opaque and documented as main-thread-affine.

Residual FFI risks (inherent to a C ABI, shared with CEF): the embedder can pass
dangling or double-freed pointers, and thread-affinity is documented rather than
enforced. Mitigations (handle "magic" cookies, debug validation) are possible
but not implemented; see §9.

**Dependencies.** Supply-chain policy exists (`deny.toml`); keeping it current is
an ongoing operational task, not a one-time fix.

---

## 6. Feature completeness (engine)

Servo implements a large slice of the web platform, but "a large slice" is not
"all of it". Areas where embedding parity with CEF is explicitly incomplete
(cited from this branch's own docs):

- **Native ↔ JS messaging**: documented as *missing*
  (`ARCHITECTURE.md` §5.4).
- **Input & rendering parity, networking/content control, and JS binding**:
  these are whole phases that are still in progress
  (`ROADMAP.md` Phase 2–4).
- **Multi-process bootstrap for C embedders**: not yet provided
  (`ARCHITECTURE.md` §8.1).
- The C API surface is a **subset** of CEF's; `CEF_PARITY.md` is itself a gap
  analysis, i.e. an enumeration of what is not there.

Media (with DRM), WebGPU, mature accessibility, extensions, devtools parity, and
enterprise policy are all areas where the incumbents are years ahead.

---

## 7. This branch's C API, specifically

What this branch *has* achieved is real and worth stating plainly: a coherent,
documented, memory-safe C ABI over libservo with a consistent
`servo_<object>_<verb>` naming scheme, builder→object ownership rules, a
buffer-query string convention, a version query, and a documented
ABI-stability policy. That is a solid foundation.

It is, however, a **foundation**, not a CEF replacement. It is version 0.1.0,
covers a subset of the engine, and (as of this branch) has no shipped prebuilt
binaries, no long-term support commitment, and no iOS/mobile embedding path.

---

## 8. Honest effort estimate

| Goal | Rough shape of the work |
|------|------------------------|
| "Embeddable on Linux/macOS/Windows with a documented C API" | Months of focused work; largely continuations of `ROADMAP.md` Phases 1–5. Achievable with the current approach. |
| "Security-hardened with site isolation everywhere we claim support" | Quarters; sandbox work on Windows and Android is a project in itself. |
| **"Production ready like Blink/CEF on Android, Linux, macOS, Windows, iOS"** | **Years, and a funded team.** Conformance is effectively never "finished"; mobile ports and security architecture are multi-team efforts. |

The last row is the honest answer to the request. It is not a matter of effort
in one session; it is a matter of scope.

---

## 9. Highest-leverage next steps (bounded and actionable)

Ordered by value-per-unit-risk for *this* repository:

1. **Finish the documented roadmap.** `ROADMAP.md` Phases 1–5 are the correct,
   already-scoped plan. Executing them is the real path to a credible embedding
   API. (Effort: the roadmap itself.)
2. **Harden the FFI boundary further.** Add optional debug validation
   (handle magic/state cookies) and document dangling/double-free contracts
   precisely. Cheap, contained, directly improves the surface this branch owns.
3. **State platform support truthfully in the API.** Expose a runtime capability
   query (sandbox available? multi-process? platform) so embedders can't assume
   security properties that are absent on Windows/iOS/Android.
4. **Decide the iOS question deliberately.** Either (a) commit to an iOS port
   workstream (large, dedicated), or (b) declare iOS unsupported for now and say
   so in the docs, embedding guide, and CI. Pretending iOS works would be the
   worst outcome.
5. **Ship prebuilt artefacts + a hello-world embedder per platform.** This is
   what turns "builds from source" into "embeddable" (C7).
6. **Keep conformance and security work upstreamed.** Servo's conformance and
   sandboxing are community-scale efforts; this fork should not diverge.

---

## 10. What changed in this pass

Three contained, low-risk improvements to the C API were made alongside this
document. They are hardening, **not** a step toward Blink-class production
readiness, and are presented as such:

1. **Fixed undefined behavior in `servo_value_set_binary`**
   (`ffi/capi/value.rs`). The previous code called
   `std::slice::from_raw_parts(data, len)` unconditionally. When an embedder
   passed `data = NULL, len = 0`, that is UB in Rust (the slice constructor
   requires a non-null, aligned pointer even for zero length). The zero-length
   case is now handled explicitly.
2. **De-duplicated the string-writing helper.** The identical
   `unsafe fn write_c_string` existed in three places (`download.rs`,
   `webview.rs`, `value.rs`). Three copies of one `unsafe` function is three
   chances for one to drift and introduce a bug. It now lives once, in
   `ffi/capi/lib.rs`, and the safety contract is upheld in a single place.
   Behavior is unchanged.
3. **Added a runtime capability query** (`ffi/capi/lib.rs`):
   `servo_capi_platform_name()`, `servo_capi_is_multiprocess_enabled()`, and
   `servo_capi_is_sandboxing_supported()`. This addresses §9 item 3 directly:
   an embedder can now *ask* whether sandboxing exists on the running platform
   instead of assuming it does. The predicate mirrors the `cfg` gates on
   `servo::create_sandbox`, which excludes Windows, iOS, Android, OpenHarmony
   **and all ARM/AArch64 and RISC-V targets** — including Apple-Silicon macOS
   and ARM64 Linux. Covered by a new C test (`test_capi_capabilities`).

None of these changes alters the public ABI.

---

## See also

- [`AUDIT.md`](AUDIT.md) — inventory, readiness ratings, technical debt.
- [`CEF_PARITY.md`](CEF_PARITY.md) — CEF concept mapping and gap list.
- [`ROADMAP.md`](ROADMAP.md) — phased plan to a usable embedding API.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — how the embedding layer is built.
- [`ABI_STABILITY.md`](ABI_STABILITY.md) — compatibility policy.
