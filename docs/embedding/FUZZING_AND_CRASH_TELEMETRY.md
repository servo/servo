# Fuzzing and Crash Telemetry Architecture

## 1. Scope & Objective

To satisfy the production-readiness requirements in `HARDENING_PLAN.md` §A.3 and `SECURITY_AUDIT.md` §8:
1. Provide in-tree automated fuzzing targets for untrusted parser inputs (cookies, URLs, HTTP headers, HTML tokenizers).
2. Establish cross-platform crash handling and telemetry hooks across Linux, macOS, and Windows.

---

## 2. In-Tree Fuzzing Infrastructure

The in-tree fuzz targets live under `/fuzz` (a separate workspace, not built by
`embedding-api.yml`) and use `cargo-fuzz` (libFuzzer). The fuzz crate depends on
`servo-net`, `servo-net-traits`, and `servo-url` so targets can exercise real
cookie parsing and matching code paths.

### 2.1 Available Targets
- `fuzz_cookie_parser`: Exercises `net::cookie::ServoCookie::from_cookie_string` against arbitrary byte arrays, testing SameSite attribute parsing, CHIPS partitioned cookies, expiration dates, and malformed UTF-8/control characters.
- `fuzz_cookie_domain_joining`: Exercises `net::cookie::ServoCookie::domain_match` and domain-attribute joining / acceptance in `from_cookie_string` against arbitrary byte pairs, asserting reflexivity, the "matched but not equal => longer host" shape, that IP literals only match themselves, and that the parser never accepts a cookie whose Domain attribute does not domain-match the request host. It also calls `path_match` and `default_path` with fuzzed strings to catch panics.
- `fuzz_cookie_partitioning`: Exercises CHIPS partitioning end to end through `net::cookie_storage::CookieStorage`: a partitioned cookie is pushed via `ServoCookie::from_cookie_string`, then queried with `cookies_for_url_with_context` under `SameSiteContext`s that have no partition key, a different partition key, or the derived partition key. Properties asserted: a partitioned cookie is **not** sent when the request context has no partition key or a different key (fail-closed); the query paths do not panic; unpartitioned cookies remain routable under keyed contexts.

### 2.2 Running Fuzz Targets Locally
Install `cargo-fuzz`:
```bash
cargo install cargo-fuzz
```
Run a target:
```bash
cargo fuzz run fuzz_cookie_parser
cargo fuzz run fuzz_cookie_domain_joining
cargo fuzz run fuzz_cookie_partitioning
```
A minimal corpus lives under `fuzz/corpus/<target>/`; `cargo fuzz` will grow it over
time. To compile every fuzz target without running them, use
`cargo +nightly check --manifest-path fuzz/Cargo.toml --bins` (the same check that
CI enforces).

### 2.3 Continuous Integration

- **PR/push gate (`embedding-api.yml`):** a compile check runs on Linux
  (`cargo +nightly check --manifest-path fuzz/Cargo.toml --bins`) so the fuzz
  workspace cannot silently rot while other changes land.
- **Scheduled fuzzing (`.github/workflows/fuzzing.yml`):** runs weekly (`cron`,
  Mondays 04:00 UTC) *and* supports `workflow_dispatch` (with a configurable
  time budget in seconds). Each target is built with `cargo-fuzz` (nightly) and
  run against a checked-in corpus for `MAX_TOTAL_TIME` seconds. Crashes are
  uploaded as artifacts named `fuzz-artifacts-<target>` (the workflow **does not**
  auto-file GitHub issues); any crash causes the job to fail. Scheduled
  workflows only activate from the repository default branch, but the
  `workflow_dispatch` path is used to exercise the pipeline on a feature branch.

---

## 3. Crash Telemetry and Signal Handling

### 3.1 Cross-Platform Signal & Exception Handling
- **Linux & macOS (`ports/servoshell/crash_handler.rs`):** Installs POSIX signal handlers via `signal-hook-registry` for `SIGSEGV`, `SIGILL`, `SIGIOT`, and `SIGBUS`. Dumps thread names and native stack backtraces to unbuffered `stderr`, then reraises the signal to preserve core dumps.
- **Windows (`ports/servoshell/crash_handler.rs`):** Installs a top-level unhandled exception filter using Win32 `SetUnhandledExceptionFilter`. Captures crash backtraces on access violations, stack overflows, and invalid handle faults.

### 3.2 Integration with Embedder Telemetry
C API embedders can register an unhandled panic/crash handler or inspect process exit status via `servo_run_content_process` return codes and the Constellation's `EmbedderMsg::Panic` messages.
