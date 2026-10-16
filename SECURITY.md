# Security Policy

## Reporting a vulnerability

Please submit security related issues as
[GitHub security reports](https://github.com/servo/servo/security/advisories/new).

Do **not** open a public issue for a suspected vulnerability. Include a
description, affected revision/commit, the platform, and (where possible) a
minimal reproduction. We will acknowledge reports through the private advisory
and coordinate disclosure once a fix is available.

## Vulnerability Handling & Response Process

1. **Intake SLA:** Security reports submitted through GitHub Security Advisories are
   acknowledged by maintainers within **48 hours**.
2. **Triage & Severity Assessment:** Maintainers assess vulnerabilities using
   **CVSS v3.1** scoring:
   - **Critical (9.0–10.0):** Remote code execution or memory corruption reachable
     from web content. Immediate patch branch opened; targeted advisory release
     within 7 days.
   - **High (7.0–8.9):** Sandbox escape, cross-origin data exposure, or bypass of
     Same-Origin Policy. Target remediation within 14 days.
   - **Medium (4.0–6.9):** Denial of service (process crash), cookie isolation bypass,
     or local spoofing. Target remediation within 30 days.
   - **Low (0.1–3.9):** Minor information disclosure or non-critical specification divergence.
3. **Coordinated Disclosure:** We adhere to a standard **90-day** disclosure window.
   If active exploitation in the wild is demonstrated, the window is accelerated to
   **7 days**. Coordinated advisories are published with CVE assignment.
4. **Supply Chain & Dependency Auditing:**
   - `cargo-deny` runs on every pull request to enforce license compliance and block
     crates with known security advisories (`deny.toml`).
   - Automated dependency vulnerability scans run weekly via GitHub Actions.
   - Security-critical dependencies (`rustls`, `spidermonkey`/`mozjs`, `webrender`)
     are audited on a quarterly cadence.

## Supported versions

Servo is under active development on the `main` branch and active embedding branches.
Security patches are backported to active embedding branches.

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
  Linux and macOS (x86-64/arm64) and Windows (mitigation policies PoC: DEP, ASLR,
  strict handle checking, extension point disabling); see
  `components/servo/servo.rs` (`create_sandbox`) and `docs/embedding/WINDOWS_SANDBOX_DESIGN.md`.
  Multiprocess mode and the sandbox are **off by default** (`components/config/opts.rs`).
- **Web-platform isolation.** CSP, CORS, HSTS, referrer policy,
  X-Frame-Options, Mixed Content, SRI, and Trusted Types are implemented with
  WPT coverage. Origin isolation architecture is documented in
  `docs/embedding/SITE_ISOLATION_DESIGN.md`.
- **Cookie & Storage Isolation.** Cookie selection enforces `SameSite=Strict`/`Lax`/`None`
  on transmission, and CHIPS (`Partitioned`) cookies are isolated by top-level partition
  key (`components/net/cookie.rs`, `components/net/cookie_storage.rs`).
- **Embedding FFI (C ABI).** Every `extern "C"` entry point is wrapped in a
  `catch_unwind` boundary and returns an error code (`ffi/capi/lib.rs`).
  Handles are opaque and thread-affine, validated in debug builds via
  `debug_checks`.

## Known security limitations

1. **Sandboxing coverage.** Windows sandbox operates in in-process mitigation
   policy mode; full broker AppContainer/Job-object isolation is in development.
   On OpenHarmony and RISC-V, content runs unsandboxed; query
   `servo_capi_is_sandboxing_supported()`.
2. **Site vs Origin isolation.** Constellation currently pools event loops by
   registrable domain (eTLD+1). Full origin-keyed isolation and out-of-process
   iframes (OOPIF) are detailed in `docs/embedding/SITE_ISOLATION_DESIGN.md`.
3. **Certificate-error bypasses.** `servo_options_set_ignore_certificate_errors`
   disables TLS verification globally if explicitly enabled by embedders.
   Off by default.
4. **Fuzzing & Telemetry.** Initial in-tree fuzz targets are established under
   `/fuzz` (`fuzz_cookie_parser`), and cross-platform crash handling is
   installed in `ports/servoshell/crash_handler.rs`. Continuous OSS-Fuzz integration
   is planned.

See `docs/embedding/SECURITY_AUDIT.md` and `docs/embedding/HARDENING_PLAN.md`
for further architectural details.
