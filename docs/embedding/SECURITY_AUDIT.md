# Servo — Security & Robustness Audit

Status: **Audit** · Branch: `embedding/cef-parity` · Baseline commit: `68719ce75c2`

This document complements [`PRODUCTION_READINESS.md`](PRODUCTION_READINESS.md)
(engine/embedding readiness), [`AUDIT.md`](AUDIT.md) (embedding-surface
inventory), and [`CEF_PARITY.md`](CEF_PARITY.md) (CEF gap analysis). Where those
cover the *embedding API*, this one covers the **security and robustness of the
engine and the embedding boundary**, with file/line evidence and a priority
list. It is the companion to the repository-level [`SECURITY.md`](../../SECURITY.md).

> Numbers below are from static search over this revision (non-test + test code
> combined unless stated) and are approximate. They indicate scale, not a defect
> count. Nothing here is a claim beyond what is cited.

---

## 1. Verdict

Servo's **web-platform security layer is largely present** (CSP, CORS, HSTS,
referrer policy, X-Frame-Options, Mixed Content, SRI, Trusted Types — all with
WPT coverage), and its **Rust memory safety** removes the dominant bug class of
C/C++ engines. However, its **process-level defences are incomplete**: the
sandbox is opt-in and only functional on Linux/macOS x86-64, there is no site
isolation, and cookie isolation (SameSite/CHIPS) is not enforced on send. The
embedding C ABI is wrapped in a panic boundary and documented, but is safe only
by contract. These are the same gaps that keep the engine from production
readiness; see `PRODUCTION_READINESS.md` §5.

This pass also fixed one concrete web-content-triggerable panic (§4.1).

---

## 2. Sandboxing & site isolation — weakest area

- Content-process sandbox profiles: `components/constellation/sandboxing.rs`;
  creation in `components/servo/servo.rs` (`create_sandbox`).
- **Supported only on Linux and macOS (x86-64).** On Windows, iOS, Android,
  OpenHarmony and all ARM/ARM64 and RISC-V targets the code panics when a
  sandbox is requested (`components/servo/servo.rs`, `create_sandbox`).
- **Off by default:** `components/config/opts.rs` — `sandbox = false`,
  `multiprocess = false`.
- **No site isolation.** No per-origin/per-site process partitioning; event
  loops are keyed by registrable domain (`components/shared/net/pub_domains.rs`).
- Backend `gaol` is pinned at an old version (`Cargo.toml`).

**vs Chromium/WebKit:** both ship a sandboxed renderer by default on every
desktop platform and partition processes by site/origin. Servo does neither by
default, and cannot on three of the five target platforms.

---

## 3. Web-platform content security — mostly implemented

| Feature | Where | Notes |
|---|---|---|
| CSP | external `content-security-policy` crate; enforced in `components/net/fetch/methods.rs`; parsed in `components/script/dom/security/csp.rs` | functional, WPT-covered |
| CORS | `components/net/http_loader.rs`, `components/net/cors_cache.rs` | functional, unit + WPT tested |
| HSTS | `components/net/hsts.rs` (+ preload list) | functional |
| Referrer-Policy | `components/net/http_loader.rs` | functional |
| X-Frame-Options | `components/script/dom/security/xframeoptions.rs` | functional |
| Mixed Content | `components/net/fetch/methods.rs` | **changed/verified** — target-browsing-context (step 1.4) implemented for both fetch/response checks (`should_request_be_blocked_as_mixed_content`, `should_response_be_blocked_as_mixed_content`); spec-1.3 clause remains a UA policy opt-out (no user control exposed). Unit test: `components/net/tests/fetch.rs::test_mixed_content_document_navigation_blocking`. |
| Same-origin / principals | `components/script_bindings/proxyhandler.rs` | functional |
| SRI / Trusted Types | engine + WPT dirs | functional |

**Gap:** Mixed Content step 1.4 (top-level vs nested document navigation) is now implemented; step 1.3 (UA-instructed allowance) remains unimplemented by design (no embedder/user control exposed).

---

## 4. Robustness: panics on web-controlled input

Panic counts (approximate, non-test + test code):

| Area | `.unwrap()` | `.expect()` | `panic!` | `unreachable!` |
|---|---|---|---|---|
| `components/script` | ~1240 | ~720 | ~71 | ~265 |
| `components/net` | ~87 | ~33 | ~9 | ~19 |
| `components/constellation` | 0 | ~41 | 0 | 1 |
| `ports/servoshell` | ~72 | ~91 | ~8 | ~7 |
| `ffi/capi` | 0 | 1 | 0 | 0 |

Most are internal invariants. The engine also converts script-thread panics into
an in-process "sad tab" (`components/constellation/constellation.rs`,
`handle_panic`) rather than crashing the whole browser — which bounds risk — but
in the default single-process mode a main-thread panic is still fatal.

### 4.1 Fixed in this pass — web-content-triggerable panic

`ErrorInfo::from_value` (`components/script/dom/bindings/error.rs`) implements
the HTML "extract error information" step and is called while reporting **any
uncaught JavaScript exception** (`GlobalScope::report_an_exception`,
`components/script/dom/globalscope/globalscope.rs`, and the WebDriver path). Its
final arm was:

```rust
_ => panic!("uncaught exception: failed to stringify primitive"),
```

`USVString::from_jsval` fails when converting the thrown value to a string
throws — which happens for an object with a throwing `toString`, e.g.
`throw { get toString() { throw "x" } }`. That reachable path panicked the
engine. It now falls back to a stable error message instead of panicking.

### 4.2 Other reachable-panic candidates (not changed)

- `components/script/dom/bindings/structuredclone.rs` — panics when a
  transfer-received object has no serialized storage.
- `components/script/dom/node/node.rs` — `from_untrusted_node_address` panics on
  an internal layout→script pointer assumption.
- `components/net/http_loader.rs`, `components/net/fetch/methods.rs` — several
  `panic!`/`unwrap` on internal networking assumptions.

These need case-by-case review (some are genuine invariants); none were changed
here to avoid unverified engine behaviour changes.

### 4.3 Implemented in this pass — `SameSite` cookie enforcement

Cookie selection now honors the `SameSite` attribute when a request is
cross-site:

- `components/net/cookie.rs` gains a `SameSiteContext`
  (`cross_site` / `top_level_navigation` / `safe_method`) and
  `ServoCookie::appropriate_for_url_with_context`. `Strict` cookies are dropped
  on cross-site requests; `Lax` (and an absent attribute, which now defaults to
  `Lax`) are only sent on a cross-site **main-frame** navigation using a safe
  method; `None` is sent on all requests. The pre-existing
  `appropriate_for_url` delegates with a same-site context, so existing callers
  and tests are unaffected.
- `components/net/cookie_storage.rs` adds
  `cookies_for_url_with_context` / `cookies_data_for_url_with_context`.
- `components/net/http_loader.rs` computes the context from the request
  (`same_site_context_for_request`) and uses it for the cookie header. Only
  `Destination::Document` counts as a top-level navigation, so cross-site iframe
  navigations do **not** receive `Lax` cookies.

Unit tests were added in `components/net/tests/cookie.rs`
(`test_same_site_cookie_matching`, `test_same_site_cookie_storage_filtering`).

> **Not yet validated in CI.** This change could not be compiled or tested in
> the environment where it was made (no MSVC/clang/GStreamer, empty `target/`).
> It must be validated by `cargo test -p servo-net` and the `cookies`/
> `same-site`/`fetch` WPT suites before landing. Expected consequence: existing
> WPT expectations that assume no `SameSite` filtering will need updating.

---

## 5. TLS / certificate handling

Core: `components/net/connector.rs` (rustls + `rustls-platform-verifier`,
`aws-lc-rs`).

- **Validation real by default** — verified in `components/net/connector.rs` (`CertificateVerificationOverrideVerifier`, `create_tls_config`).
- **Bypass #1 (global):** `ignore_certificate_errors` accepts every certificate for every host. Default `false`; exposed to embedders via `servo_options_set_ignore_certificate_errors` (`ffi/capi/options.rs:176-190`). **Verified** (present by design; off by default).
- **Bypass #2 (per-certificate):** `CertificateErrorOverrideManager` accepts a matching **DER certificate** regardless of hostname/SAN (weaker than Chromium's per-host exception model). Failed certs are surfaced to the embedder (`components/net/http_loader.rs`, `ffi/capi/cert_error.rs`). **Verified**.
- The FFI certificate-error continuation (`ffi/capi/cert_error.rs`) carries a raw `user_data` pointer and a documented-but-unenforced "do not use after the callback returns" lifetime. **Documented/deferred** (see recommendation).

**Recommendation:** keep both off by default (done); require an explicit per-host decision for overrides; add regression tests for the bypass logic (currently untested in isolation). **Gap registered** (no isolated tests added in this pass).

---

## 6. Embedding C ABI (ffi/capi) safety

- **Panic boundary present:** every `extern "C"` entry is wrapped in
  `catch_unwind`, returning `ServoError::InternalPanic` / null
  (`ffi/capi/lib.rs`).
- Pointer arguments are null-checked; strings are UTF-8 validated; a single
  `write_c_string` helper is used.
- Residual, inherent-to-C-ABI risks: the embedder can pass dangling or
  double-freed handles (manual `Box::from_raw` ownership transfer sites), and
  several callbacks hand out **pointers to temporaries valid only during the
  call** (documented in `docs/embedding/CEF_PARITY.md`/`ARCHITECTURE.md`, not
  enforced).
- `unsafe impl Send/Sync` is asserted for at least one caller-supplied handler
  (`ffi/capi/protocol_handler.rs`) with no compiler-checkable guarantee.

These match the FFI residuals CEF also lives with, but CEF has years of
hardening; adding optional debug validation (handle state/magic cookies) would
be cheap and contained.

---

## 7. Dependency & supply-chain hygiene

- `cargo-deny` runs in CI (via `mach test-tidy`; config `deny.toml`).
- **No `cargo-vet`**: no vetted-audits attestation.
- Some advisories remain on the ignore list (`paste`, `unic-*`, `rsa`
  RUSTSEC-2023-0071, `bincode`, `ttf-parser`); one crypto dependency is pinned to
  a release candidate (`rsa = "=0.10.0-rc.18"`, `Cargo.toml`).
- **No hardcoded secrets** found; the only key material is a test fixture
  (`components/net/tests/privatekey_for_testing.key`).
- **No fuzzing harness** (no libFuzzer/AFL target for the engine).
- **No crash telemetry / minidumps**; only a POSIX signal handler that prints a
  backtrace (`ports/servoshell/crash_handler.rs`, macOS/Linux only).

---

## 8. Security regression tests

Present and meaningful: WPT suites for CSP, referrer-policy, mixed-content,
fetch, CORS, SRI, secure-contexts, trusted-types, WebCryptoAPI; unit tests for
origin/same-origin (`tests/unit/script/origin.rs`), CORS
(`components/net/tests/fetch.rs`), HSTS, and cookies.

Gaps: no isolated tests for the certificate-bypass logic (5. above), and no
fuzzing.

---

## 9. Prioritised remediation

Ordered by value-per-unit-risk:

1. **Enforce partitioned (CHIPS) cookies on send.** `SameSite` is now enforced
   (see §4.3), but `ServoCookie` still does not implement the `Partitioned`
   attribute: cookie selection has no partition key, so partitioned cookies are
   stored and sent unpartitioned. This requires extending the storage model
   (`components/net/cookie_storage.rs`) to key cookies by
   `(domain, partition_key)` and threading the top-level-site partition key
   through `http_loader`. `Cookie::partitioned()` is already parsed by
   `cookie` 0.18.
2. **Keep the JS-reachable panic fixed** (§4.1) and audit the remaining
   reachable-panic candidates (§4.2) into non-fatal paths.
3. **Enable the background hang monitor by default and act on hang alerts**
   (today it is opt-in and warn-only; see `PRODUCTION_READINESS.md`).
4. **Extend sandbox coverage** (Windows, then Android/iOS) and add **site
   isolation**; publish the capability honestly through
   `servo_capi_is_sandboxing_supported()`.
5. **Harden the FFI boundary**: optional debug handle validation; make the
   temporary-pointer and thread-affinity contracts explicit and tested.
6. **Add a fuzzing harness** (parser/network/DOM) and **crash symbolication**.
7. **Supply chain**: adopt `cargo-vet`; retire the `deny.toml` ignores and the
   RC crypto pin.

Items 1–3 and 5 are bounded and worth doing next; items 4 and 6 are
quarter-scale projects; item 7 is ongoing hygiene.

Reviewable, file-level designs for items 2, 3 and 5 (remaining panics, the hang
monitor + kill-script workflow, and FFI handle validation) are in
[`HARDENING_PLAN.md`](HARDENING_PLAN.md). The iOS question in item 4 is scoped in
[`IOS_PORT_SPIKE.md`](IOS_PORT_SPIKE.md).
