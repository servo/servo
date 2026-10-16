# Security Policy

## Reporting a vulnerability

Please submit security related issues as
[GitHub security reports](https://github.com/servo/servo/security/advisories/new).

Do **not** open a public issue for a suspected vulnerability. Include a
description, affected revision/commit, the platform, and (where possible) a
minimal reproduction. We will acknowledge reports through the private advisory
and coordinate disclosure once a fix is available.

## Supported versions

Servo is a prototype engine under active development. Only the current `main`
branch (and this fork's active development branch) receives security fixes;
there are no maintained release branches or long-term-support versions yet.

## Threat model

Servo is a browser engine: the primary untrusted input is **web content**
(HTML/CSS/JavaScript, network responses, media, and the DOM/IPC surface), and
the security goal is that a malicious page must not compromise the host, read
data from other origins, or crash the browser process. Relevant surfaces:

- **Memory safety.** The engine is written in Rust; the large class of
  memory-corruption bugs common to C/C++ engines is largely absent by
  construction. This is Servo's main structural advantage. It does not remove
  logic/sanitisation bugs or the C FFI boundary from scope.
- **Process isolation and sandboxing.** A content-process sandbox exists on
  Linux and macOS (x86-64) but is **not supported on Windows, iOS, Android,
  OpenHarmony, or any ARM/ARM64 or RISC-V target**; see
  `components/servo/servo.rs` (`create_sandbox`). Multiprocess mode and the
  sandbox are **off by default** (`components/config/opts.rs`).
- **Web-platform isolation.** CSP, CORS, HSTS, referrer policy,
  X-Frame-Options, Mixed Content, SRI, and Trusted Types are implemented with
  WPT coverage. There is currently **no site isolation** (processes are keyed
  by site, not origin) and SameSite/partitioned cookie rules are not yet
  enforced when selecting cookies to send (see "Known limitations").
- **Embedding FFI (C ABI).** Every `extern "C"` entry point is wrapped in a
  `catch_unwind` boundary and returns an error code (`ffi/capi/lib.rs`).
  Handles are opaque and thread-affine. Pointer validity, aliasing, and
  double-free avoidance are **contractual** (documented in
  `docs/embedding/`) and not enforced at runtime.

## Known security limitations

These are deliberate, documented gaps — not unknown bugs — and are the priority
list for hardening:

1. **Sandboxing coverage.** Opt-in, and unavailable on Windows/iOS/Android and
   all ARM targets. Do not assume a sandbox when embedding on those platforms;
   query `servo_capi_is_sandboxing_supported()`.
2. **No site isolation.** All origins share content-process(es).
3. **Cookie `SameSite` is now enforced on send, but partitioned (CHIPS)
   cookies are not.** Cookie selection applies `SameSite=Strict`/`Lax`/`None`
   (and defaults an absent attribute to `Lax`) when a request is cross-site
   (`components/net/cookie.rs`, `components/net/http_loader.rs`). Partitioned
   (`Partitioned`) cookies are still parsed but not isolated, because the
   storage model does not yet keep a partition key
   (`components/net/cookie.rs`, `components/net/cookie_storage.rs`).
4. **Certificate-error bypasses are exposed to embedders.**
   `servo_options_set_ignore_certificate_errors` disables TLS verification
   globally, and the certificate-error override path trusts a matching
   certificate regardless of host. Both are off by default; enabling them
   removes transport authenticity.
5. **Supply chain.** `cargo-deny` runs in CI, but there is no `cargo-vet`
   attestation and some advisories are still on the ignore list
   (`deny.toml`).
6. **No fuzzing harness and no crash telemetry.** There is no in-tree
   libFuzzer/AFL target for the engine, and no minidump/crash-upload pipeline.

See `docs/embedding/SECURITY_AUDIT.md` for the evidence, counts, and a
prioritised remediation plan.
