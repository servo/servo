# Servo Full Browser Roadmap

> A plan to evolve Servo from a solid embeddable engine to a full, all-purpose browser engine on par with Blink/Chromium/CEF.

**Author/owner:** User (working with two AI CLI coding agents)
**Scope:** Ignore iOS compatibility for now. Goal is full web compatibility, a complete consumer browser, and all polish of big browsers. Aim to make Servo a full, all-purpose browser engine like Blink/CEF.
**Branch target:** `embedding/cef-parity` or subsequent branches as appropriate

This document is intended as a handoff to the Antigravity CLI agent. It defines goals, non-goals (iOS deferred), milestones, required workstreams, testing strategy, acceptance criteria, risks, and a concrete implementation plan that can be executed iteratively.

---

## 1. Mission & Vision

Make Servo a full, all-purpose browser engine (like Blink/Chromium/CEF) with:
- **Full web compatibility:** Pass the vast majority of modern web platform tests and work correctly on real-world sites.
- **Complete consumer browser:** Ship a complete end-user browser experience (UI, navigation, bookmarks/history, downloads, settings, profiles, etc.) built on Servo's embedding API.
- **Polish of big browsers:** Security UX, performance, stability, accessibility, internationalization, developer tools, updates, crash reporting, signing/distribution where applicable, and production-grade reliability.

Constraints: **iOS compatibility is ignored for now.**

---

## 2. Principles

1. **Build on the solid embedding foundation.** The embedding/CEF-parity work has established a clean C API, contracts, and documentation discipline (one commit per ticket, green gates, evidence). Extend that discipline.
2. **Test-driven parity.** Prefer fixing engine behavior over changing expectations. Measure with WPT and real-world regression tests.
3. **Incremental and shippable.** Milestones must be demonstrable (can run a real browser UI or pass meaningful test suites).
4. **Security-first.** Site isolation, sandboxing, CSP/COOP/COEP/CORP/XCTO correctness, SameSite/partitioning, secure contexts, and robust TLS remain non-negotiable.
5. **No regressions in embedding.** The C API and documented contracts must remain stable or evolve with clear ABI versioning per `ABI_STABILITY.md`.
6. **Cross-platform focus (excluding iOS).** Prioritize Linux, macOS, Windows, Android, OpenHarmony as feasible, with clear platform-specific gaps tracked.

---

## 3. Core Workstreams

### 3.1 Web Platform Compatibility (highest leverage)
- **WPT regression tracking:** Establish and maintain a baseline of passing/failing tests across key suites (html, dom, css, fetch, xhr, js, websockets, service-workers, webrtc, wasm, webgl/webgpu, etc.). Publish per-commit deltas.
- **Interop/real-world conformance:** Target top sites and common patterns; fix spec violations systematically.
- **CSS/Layout:** Continue hardening layout, flexbox/grid, text shaping, fonts, animations, contain/paint, transforms.
- **JS/DOM:** Improve bindings, performance, structured clone, exceptions, event loop timing, and microtask semantics.
- **Networking/Fetch:** CORS, COEP/COOP/CORP, mixed content, referrer policy, HSTS, CHIPS/partitioned cookies, service workers, streams.
- **Media/WebRTC/WebAudio:** Playback, capture, codecs support matrix, DRM considerations (track separately).

### 3.2 Site Isolation & Process Model
- **Site/origin isolation:** Move from registrable-domain grouping toward per-site/origin process splitting with clear policy.
- **Multiprocess robustness:** Harden content process launch, IPC, sandboxing across Linux/macOS/Windows/Android. Expand CI coverage for multiprocess paths (currently limited).
- **Sandbox coverage:** Extend sandboxing beyond Linux/macOS x86_64 to Windows and Android where feasible; document remaining limitations.

### 3.3 Security Hardening
- **Cookie partitioning (CHIPS):** Implement partition key tracking in cookie storage and send logic end-to-end.
- **COEP/COOP enforcement:** Complete and test cross-origin isolation paths.
- **Certificate handling:** Strengthen per-host override model; add regression tests for bypass paths.
- **XFO/CSP/CORP correctness:** Ongoing verification against WPT.
- **Handle liveness:** Extend registry adoption to remaining handle types (`ServoBuilder`, `RenderingContext`, `ServoValue`, `ServoUserContentManager`, `ServoWebViewBuilder`, `Servo`, `ServoSiteDataManager`, `ServoRequestContext`).
- **FFI hardening:** Callback lifetime/poisoning detection in debug, temporary-pointer contracts enforced/tested.

### 3.4 Complete Consumer Browser
- **Browser UI:** Build a minimal but complete cross-platform browser shell (not just servoshell) with tabs, address bar, navigation (back/forward/reload/stop), bookmarks, history, downloads manager, settings/preferences UI.
- **Profiles & sessions:** User profiles, session restore, incognito/private mode with proper storage isolation.
- **Downloads:** Full download manager with progress, destination choice, security checks (mixed downloads blocked), resume considerations.
- **Search & omnibox:** Default search, suggestions, URL parsing/autofill.
- **Permissions:** Geolocation, camera/mic, notifications, clipboard, file access with user prompts.
- **Printing & accessibility:** Complete AccessKit action forwarding (W6.4 gap) and printing flows.
- **Internationalization & localization:** UI i18n, RTL, font fallback.

### 3.5 Polish & Production Readiness
- **Crash reporting & symbolication:** Minidumps/backtraces with symbols, opt-in crash telemetry pipeline.
- **Updates & distribution:** Auto-updates strategy (platform-specific), signed builds, packaging for Linux (deb/rpm/AppImage), macOS (signed/notarized), Windows (MSIX/installer), Android (APK/AAB).
- **Performance:** Benchmarks (Speedometer/JetStream/MotionMark/WebXPRT), memory usage, startup time, jank reduction, GPU acceleration validation across real hardware.
- **DevTools:** Full DevTools protocol support and UI integration.
- **Extensions:** Design and implement extension API surface (manifest v3 compatible where practical) — large scope, tracked separately.
- **Enterprise policies:** Configurable policies where needed.
- **Reliability:** Hang monitor default-on + actions, leak tracking and reduction, telemetry for health.

---

## 4. Milestones (incremental)

| Milestone | Goal | Success Criteria | Target (rough) |
|---|---|---|---|
| M0: Baseline | Lock in current state, document WPT baseline | Green gates on current work, publish WPT pass-rate snapshot | Now |
| M1: Core Web Stability | Fix top regressions blocking real sites | +X% on key WPT suites, 5–10 real sites load/readable without blockers | Short |
| M2: Multiprocess + Isolation | Harden spawn, sandbox coverage, basic site isolation design | Multiprocess CI expanded, documented isolation model + prototype | Mid |
| M3: Minimal Complete Browser (MCB) | Ship a usable consumer browser shell (tabs+nav+downloads+settings) | Can browse common sites, downloads work, private mode works | Mid |
| M4: Compatibility Push | Drive WPT to competitive subset; CSS/JS layout fixes | Sustained WPT improvement, fewer site-specific hacks needed | Mid-Late |
| M5: Polish & Distribution | Crash reporting, updates, signed packages, performance tuning | Installable packages on 3 desktop platforms, measurable perf vs baseline | Late |
| M6: Full-Parity Readiness | Document remaining gaps vs Blink/CEF with concrete path | Public parity matrix with known deltas + acceptance criteria | Late |

---

## 5. Handoff Notes for Antigravity CLI

When starting from this document:
1. **Read existing docs first:** `AGENT_BUILD_PLAN.md`, `SECURITY_AUDIT.md`, `PRODUCTION_READINESS.md`, `ABI_STABILITY.md`, `ARCHITECTURE.md`, `CEF_PARITY.md`, `HARDENING_PLAN.md`, `IOS_PORT_SPIKE.md`.
2. **Follow the same discipline:** One logical change per commit where appropriate, evidence for acceptance, green CI before marking done, never fake results.
3. **Measure before changing:** Capture WPT baseline (`windows-wpt.yml` or equivalent) and record run IDs. Do not bulk-edit expectations; fix engine behavior.
4. **Track deltas vs goal:** Maintain a `docs/embedding/PARITY_MATRIX.md` (create/update) showing Blink/CEF parity by feature area with status (done/in-progress/deferred + evidence).
5. **Prioritize ruthlessly:** Focus on compatibility blockers that unlock real sites, then consumer browser completeness, then polish. Site isolation/security hardening run in parallel.

**Initial tasks to pick up:**
1. Create `docs/embedding/PARITY_MATRIX.md` with current state and target.
2. Establish WPT baseline infrastructure and capture first numbers.
3. Triage top failing WPT suites; create tickets per high-impact area.
4. Design MCB (minimal complete browser) architecture: UI layer vs embedding API usage.
5. Continue W2.7 completion systematically (CSP/COOP/CORP/XCTO/TLS review with full evidence).

---

## 6. Risks & Mitigations

- **Scope creep:** The full-parity goal is large. Mitigate via milestones and strict deferral tracking.
- **WPT churn:** Target stable subsets and track deltas over time.
- **Performance regression:** Gate with benchmarks; avoid speculative complexity.
- **Platform divergence:** Keep cross-platform paths shared; document platform-specific tradeoffs.
- **Resource constraints:** Prioritize high-leverage work; defer nice-to-haves explicitly.

---

*This is a living document. Update as work progresses, maintaining the same evidence-driven discipline.*