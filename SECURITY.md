# Security Policy

## Reporting a vulnerability

Please submit security related issues as
[GitHub security reports](https://github.com/servo/servo/security/advisories/new).

Do **not** open a public issue for a suspected vulnerability. Include a
description, affected revision/commit, the platform, and (where possible) a
minimal reproduction. We will acknowledge reports through the private advisory
and coordinate disclosure once a fix is available.

The private advisory form above is the only intake channel: reports are not
tracked in public issues, and there is no security mailing list. Private
vulnerability reporting is enabled on `servo/servo` (the form above) but not on
this fork, so branch-specific findings (the C API / embedding surface under
`ffi/capi` and `docs/embedding/`) go through the same advisory, naming the branch
and the affected revision.

## Response process

These are targets, not contractual terms; a report that needs a large
architectural change may take longer, in which case the reason is communicated to
the reporter.

| Stage | Target |
|---|---|
| Acknowledge the report | 3 working days |
| Initial triage: severity, affected revisions/platforms, reproducibility | 10 working days |
| Fix or documented mitigation for a confirmed issue | 30 days for high/critical, 90 days otherwise |
| Public advisory | within 14 days of a fix being available, or on the date agreed with the reporter |

## Publishing an advisory

1. Fix on a private branch and keep the report private until publication.
2. Publish the advisory through the GitHub security advisory for the affected
   repository, marking it as a vulnerability (GitHub is a CNA and can assign a
   CVE): state the affected revisions, the severity (CVSS vector), the fix
   commit, and the action an embedder must take (for example "update the pinned
   revision" or "stop passing `servo_options_set_ignore_certificate_errors`").
3. Credit the reporter unless they ask to stay anonymous, and link the advisory
   from the affected document under `docs/embedding/` so the claim that it
   corrects does not stay stale.

## Supported versions

Servo is a prototype engine under active development on `main` and active
embedding branches. Security fixes are applied to active development branches;
there are no maintained long-term-support (LTS) releases.

## Dependency and supply-chain cadence

- **Policy.** `deny.toml` at the repository root: RustSec advisories (each
  `ignore` entry carries a written rationale), the license allowlist, and
  duplicate/blocked-package bans.
- **Check.** `cargo deny check` is wired into `./mach test-tidy`
  (`python/tidy/tidy.py`, `run_cargo_deny_lints`), so it runs wherever tidy runs
  and is the same command an embedder can run before pinning a revision.
- **Updates.** `.github/dependabot.yml` opens grouped Cargo update PRs twice a
  week (Monday-Wednesday-Sunday cron `0 1 * * WED,SUN`, Europe/Berlin) with a
  7-day cooldown for every dependency, plus Cargo/`wgpu`-specific ignores.
  Advisories are not left to that cadence: a failing `cargo deny check` (or an
  advisory that requires an immediate bump) is handled out of band.
- **Current gap.** On this fork, `mach test-tidy` is only run by `lint.yml`,
  which is invoked on demand (`workflow_call`/`workflow_dispatch`, currently from
  `dispatch-workflow.yml`); the embedding workflows do not call it. Until a
  scheduled or push-triggered job is added, advisory and license checks are on
  demand, not continuous.

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
  Handles are opaque and thread-affine; main-thread affinity is enforced at the
  FFI boundary (`ffi/capi/lib.rs::entry_checks`) in all builds, and an off-thread
  call returns `SERVO_ERROR_WRONG_THREAD`.

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
5. **Fuzzing & Telemetry.** Initial in-tree fuzz targets are established under
   `/fuzz` (`fuzz_cookie_parser`), and cross-platform crash handling is
   installed in `ports/servoshell/crash_handler.rs`. Continuous OSS-Fuzz integration
   is planned.

See `docs/embedding/SECURITY_AUDIT.md` and `docs/embedding/HARDENING_PLAN.md`
for further architectural details.
