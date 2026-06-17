# Local Runtime Progress


## 2026-06-17 — Net resource-thread request logging

- Inspected `components/net/resource_thread.rs` as the central `CoreResourceManager::fetch` seam where `RequestBuilder` becomes a concrete Fetch `Request` before dispatch into the existing fetch/http/protocol machinery.
- Touched `components/net/resource_thread.rs` to add a `[local-runtime resource-request]` log before changing or denying behavior.
- The log currently captures destination, requested/current URL, referrer-derived base/initiator when present, inferred package for `asset://` and `bundle://`, the Servo module, provisional decision, final URL, MIME placeholder, and reason.
- Discovered that this seam is after callers have already resolved the current URL; it does not yet expose the original unresolved attribute text (`./styles.css`) or stylesheet-vs-document base distinction. Those must be carried earlier from script/style loaders in a later pass.
- No resource is denied by this change. Remote/file/store schemes are explicitly logged as still taking the legacy path rather than being hidden behind an implicit fetch failure.
- Open question: where to attach the host-owned ResourceProvider so `asset://` and `bundle://` are handled before existing protocol/http dispatch while preserving response MIME and final URL identity.

## 2026-06-17 — Devcontainer CI smoke workflow

- Added `.github/workflows/local-runtime-devcontainer-smoke.yml` as a minimal GitHub Actions smoke job for local-runtime follow-up work.
- The workflow runs inside Servo's published devcontainer image, `ghcr.io/servo/servo/devcontainer-ubuntu:latest`, and mirrors the key devcontainer environment variables from `.devcontainer/devcontainer.json`: `CC=clang`, `CXX=clang++`, `CARGO_TARGET_DIR=/var/servo-cargo-target`, and `UV_PROJECT_ENVIRONMENT=.devcontainer-venv`.
- The job proves that a checkout can enter the same published container family used by the devcontainer, run `./mach bootstrap --yes`, record the compiler/Rust/Python/uv toolchain versions, resolve Cargo workspace metadata with `cargo metadata --format-version 1 --locked --no-deps`, and print `./mach build --help` without attempting a full Servo build.
- No runtime resource paths are loaded or denied by this workflow; it is intentionally only a repeatable environment and toolchain smoke check before deeper package-scoped loader work.
- If this smoke job passes, the next build command to try in CI is `./mach build --dev` as a separate follow-up job or manually-dispatched workflow step, not as part of this first smoke gate.
