# WPT High-Impact Triage & Remediation Tickets

Status: **Active Triage** · Milestone: `M1 / M4`
Companion docs: [`FULL_BROWSER_ROADMAP.md`](FULL_BROWSER_ROADMAP.md), [`PARITY_MATRIX.md`](PARITY_MATRIX.md), [`WPT_CONFORMANCE_BASELINE.md`](WPT_CONFORMANCE_BASELINE.md).

---

## 1. Overview & Strategy

Analysis of the 154,583 non-pass metadata entries across `tests/wpt/meta` reveals that failures are concentrated heavily in a small set of foundational suites. To reach modern web compatibility and support real-world consumer sites, remediation is prioritized by high-leverage user impact rather than sheer volume.

```
+-----------------------------------------------------------------------+
| Priority 1: Web Fundamentals (Blockers for top 100 consumer websites) |
|   - Fetch & Streams (Streaming body, chunked transfer, redirect)      |
|   - HTML Forms, Canvas & Media (Interactive controls, 2D contexts)   |
|   - CSS Flexbox & Grid Layout (Subgrid, intrinsic sizing, alignment)  |
|   - DOM & Event Timing (Microtask ordering, event dispatch)          |
+-----------------------------------------------------------------------+
| Priority 2: Modern Web Storage & Offline (PWA / Application sites)   |
|   - IndexedDB & Cache API (Local database transactions)              |
|   - Service Workers (Lifecycle, fetch routing)                        |
+-----------------------------------------------------------------------+
| Priority 3: Rich Graphics & Audio                                     |
|   - WebGL 2 & WebGPU conformance                                     |
|   - WebAudio & Media Source Extensions (MSE)                          |
+-----------------------------------------------------------------------+
| Defer / Non-blocking (Low consumer leverage)                          |
|   - Editing / execCommand legacy surface (23k failures)               |
|   - WebNN (spec in high flux)                                         |
+-----------------------------------------------------------------------+
```

---

## 2. High-Impact Remediation Tickets

### Ticket WPT-01: WHATWG Fetch & Streams Conformance
- **Target Suite:** `tests/wpt/meta/fetch/`, `tests/wpt/meta/streams/`
- **Current Failures:** 1,415 (`fetch`) + partial streams
- **Real-World Impact:** Modern sites (YouTube, GitHub, Twitter/X) rely on `fetch` response streaming, CORS redirect handling, and `ReadableStream` upload/download bodies.
- **Key Focus Areas:**
  1. Stream pipe-to / tee operations in `components/script/dom/readablestream.rs`.
  2. CORS header parsing edge cases in `components/net/cors_cache.rs`.
  3. Chunked transfer decoding and keep-alive handling in `components/net/http_loader.rs`.
- **Milestone:** M1

---

### Ticket WPT-02: CSS Grid & Flexbox Edge Cases
- **Target Suite:** `tests/wpt/meta/css/css-grid/`, `tests/wpt/meta/css/css-flexbox/`
- **Current Failures:** ~8,500 subset in `css`
- **Real-World Impact:** Broken page layout, overlapping text, and misaligned navigation bars across e-commerce and media sites.
- **Key Focus Areas:**
  1. Intrinsic sizing (`min-content`, `max-content`, `fit-content`) calculations in `components/layout/`.
  2. Grid track auto-placement and subgrid alignment.
  3. Flexbox baseline alignment with nested items.
- **Milestone:** M1 / M4

---

### Ticket WPT-03: HTML5 Form Validation & Interactive Controls
- **Target Suite:** `tests/wpt/meta/html/semantics/forms/`
- **Current Failures:** ~4,200 in `html`
- **Real-World Impact:** Broken login forms, date pickers, client-side validation errors preventing form submission.
- **Key Focus Areas:**
  1. Constraint validation API (`checkValidity()`, `reportValidity()`, `:invalid` pseudo-class).
  2. Embedder control event forwarding (`components/servo/lib.rs:EmbedderControl`).
  3. `<dialog>` element modal backdrop and focus trapping.
- **Milestone:** M1

---

### Ticket WPT-04: IndexedDB Transaction & Key-Path Conformance
- **Target Suite:** `tests/wpt/meta/IndexedDB/`
- **Current Failures:** 1,270 failures, 187 errors
- **Real-World Impact:** Web apps failing to load offline data or corrupting local stores (e.g. Google Docs, Slack web).
- **Key Focus Areas:**
  1. IndexedDB transaction lifecycle and auto-commit semantics in `components/storage/`.
  2. Keypath extraction for compound keys and binary arrays.
  3. Error code propagation matching W3C IndexedDB 3.0.
- **Milestone:** M4

---

### Ticket WPT-05: DOM Microtask & MutationObserver Ordering
- **Target Suite:** `tests/wpt/meta/dom/`
- **Current Failures:** 1,239 failures, 27 errors
- **Real-World Impact:** Race conditions in single-page apps (React, Vue, Svelte) causing blank screens or stale state.
- **Key Focus Areas:**
  1. Microtask checkpoint execution order in `components/script/dom/eventtarget.rs`.
  2. MutationObserver delivery timing relative to custom element reactions.
  3. Event bubbling and capture across shadow boundaries.
- **Milestone:** M1
