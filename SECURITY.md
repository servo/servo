# Security Policy

## Reporting a vulnerability

Please submit security related issues as
[GitHub security reports](https://github.com/servo/servo/security/advisories/new).

Do **not** open a public issue for a suspected vulnerability. Include a
description, affected revision/commit, the platform, and (where possible) a
minimal reproduction. We will acknowledge reports through the private advisory
and coordinate disclosure once a fix is available.

## Supported versions

Servo is a prototype engine under active development on `main` and active
embedding branches. Security fixes are applied to active development branches;
there are no maintained long-term-support (LTS) releases.

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
  Linux and macOS (x86_64) but is **not supported on Windows, iOS, Android,
  OpenHarmony, or any ARM/ARM64 or RISC-V target**; see
  `components/servo/servo.rs` (`create_sandbox`). On Windows, in-process mitigation
  policies (DEP, ASLR, strict handle checking, extension point disabling) are
  applied when `--sandbox` is enabled, but mitigations are not an OS sandbox (no
  broker, token restriction, or AppContainer; see `docs/embedding/WINDOWS_SANDBOX_DESIGN.md`).
  Multiprocess mode and the sandbox are **off by default** (`components/config/opts.rs`).
- **Web-platform isolation.** CSP, CORS, HSTS, referrer policy,
  X-Frame-Options, Mixed Content, SRI, and Trusted Types are implemented with
  WPT coverage. Origin isolation architecture is documented in
  `docs/embedding/SITE_ISOLATION_DESIGN.md`.
- **Cookie & Storage Isolation.** Cookie selection enforces `SameSite=Strict`/`Lax`/`None`
  on transmission (`components/net/cookie.rs`, `components/net/http_loader.rs`).
  CHIPS (`Partitioned`) cookies are parsed and stored with a partition key, but
  because `SameSiteContext.partition_key` is not yet populated from top-level
  browsing context, partitioned cookies are currently stored but never sent (fail-closed).
- **Embedding FFI (C ABI).** Every `extern "C"` entry point is wrapped in a
  `catch_unwind` boundary and returns an error code (`ffi/capi/lib.rs`).
  Handles are opaque and thread-affine; main-thread affinity is checked in debug
  builds via `debug_checks::assert_main_thread`.

## Known security limitations

1. **Sandboxing coverage.** Sandboxing is available only on Linux x86_64 and
   macOS x86_64. Windows applies in-process mitigation policies (Phase 1) but
   lacks an OS sandbox (no broker/token lockdown). Android, iOS, OpenHarmony,
   and all ARM/RISC-V targets do not have renderer sandboxing. Query
   `servo_capi_is_sandboxing_supported()` at runtime (returns false on those
   platforms).
2. **Site vs Origin isolation.** Constellation currently pools event loops by
   registrable domain (eTLD+1). Full origin-keyed isolation and out-of-process
   iframes (OOPIF) are detailed in `docs/embedding/SITE_ISOLATION_DESIGN.md`.
3. **Partitioned (CHIPS) cookies stored but not sent.** Partitioned cookies
   are parsed with a `Secure` requirement and stored with a partition key, but
   because `SameSiteContext.partition_key` is not yet populated on request contexts,
   partitioned cookies are not currently returned on transmission.
4. **Certificate-error bypasses.** `servo_options_set_ignore_certificate_errors`
   disables TLS verification globally if explicitly enabled by embedders.
   Off by default.
4. **Fuzzing & Telemetry.** Initial in-tree fuzz targets are established under
   `/fuzz` (`fuzz_cookie_parser`), and cross-platform crash handling is
   installed in `ports/servoshell/crash_handler.rs`. Continuous OSS-Fuzz integration
   is planned.

See `docs/embedding/SECURITY_AUDIT.md` and `docs/embedding/HARDENING_PLAN.md`
for further architectural details.
