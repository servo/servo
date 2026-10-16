# Fuzzing and Crash Telemetry Architecture

## 1. Scope & Objective

To satisfy the production-readiness requirements in `HARDENING_PLAN.md` §A.3 and `SECURITY_AUDIT.md` §8:
1. Provide in-tree automated fuzzing targets for untrusted parser inputs (cookies, URLs, HTTP headers, HTML tokenizers).
2. Establish cross-platform crash handling and telemetry hooks across Linux, macOS, and Windows.

---

## 2. In-Tree Fuzzing Infrastructure

The in-tree fuzz targets live under `/fuzz` and use `cargo-fuzz` (libFuzzer):

### 2.1 Available Targets
- `fuzz_cookie_parser`: Exercises `net::cookie::ServoCookie::from_cookie_string` against arbitrary byte arrays, testing SameSite attribute parsing, CHIPS partitioned cookies, expiration dates, and malformed UTF-8/control characters.

### 2.2 Running Fuzz Targets
Install `cargo-fuzz`:
```bash
cargo install cargo-fuzz
```
Run the target:
```bash
cargo fuzz run fuzz_cookie_parser
```

---

## 3. Crash Telemetry and Signal Handling

### 3.1 Cross-Platform Signal & Exception Handling
- **Linux & macOS (`ports/servoshell/crash_handler.rs`):** Installs POSIX signal handlers via `signal-hook-registry` for `SIGSEGV`, `SIGILL`, `SIGIOT`, and `SIGBUS`. Dumps thread names and native stack backtraces to unbuffered `stderr`, then reraises the signal to preserve core dumps.
- **Windows (`ports/servoshell/crash_handler.rs`):** Installs a top-level unhandled exception filter using Win32 `SetUnhandledExceptionFilter`. Captures crash backtraces on access violations, stack overflows, and invalid handle faults.

### 3.2 Integration with Embedder Telemetry
C API embedders can register an unhandled panic/crash handler or inspect process exit status via `servo_run_content_process` return codes and the Constellation's `EmbedderMsg::Panic` messages.
