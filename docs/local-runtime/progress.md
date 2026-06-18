# Local Runtime Progress


## 2026-06-18 — Manual build smoke de-duplication

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so `push` and `pull_request` events still run the existing cheap `smoke` job for bootstrap, metadata, `servo-net`, and build-help checks only.
- Manual `workflow_dispatch` runs now skip the separate `smoke` job and run only the selected manual Linux build job, so artifact, debug, and release build attempts no longer repeat a separate queue/bootstrap cycle before building.
- Removed the manual Linux debug build's dependency on `smoke`; the build job keeps its own checkout, target-directory preparation, bootstrap, build, package, artifact upload, release upload, size diagnostics, symbol split, and Git safe-directory handling unchanged.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only workflow scheduling change.


## 2026-06-18 — Devcontainer ZIP timestamp packaging fix

- The latest manual Linux debug build reached the packaging/artifact stage after the existing build and package work had already progressed.
- The failure was caused by Python `zipfile` refusing extracted package entries with timestamps before 1980 (`ValueError: ZIP does not support timestamps before 1980`), not by Servo compilation or Rust code.
- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so both stripped-runtime and debug-symbol ZIP writers use `strict_timestamps=False`, allowing Python to clamp ZIP metadata safely for pre-1980 extracted files while preserving the two intended downloadable ZIP outputs.
- The ZIP archive names are now generated as sorted, package-relative POSIX paths, keeping archive entries simple and deterministic across the extracted runtime and symbol trees.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only packaging fix.


## 2026-06-18 — Devcontainer debug package ELF-tool fallback

- The manual Linux debug build itself succeeded with the existing `./mach build --debug` command, and the follow-up package diagnostics showed the original debug package was about 530M.
- The largest-file inventory showed the package is dominated by the debug `runtime/servo/servoshell` binary, reported at 2038099592 bytes before stripping.
- The previous diagnostic/package-splitting run failed after the successful build only because the Servo devcontainer did not have the `file` command installed, so ELF detection stopped at `file: command not found`.
- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so ELF detection uses `readelf -h` or `llvm-readelf -h`, and debug extraction/stripping uses whichever of `objcopy`/`llvm-objcopy` and `strip`/`llvm-strip` is available at runtime. The workflow now prints the selected tools and avoids depending on `file` while keeping the stripped runtime ZIP, debug-symbol ZIP, size diagnostics, and GitHub Release safe-directory fix.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only packaging fix.

## 2026-06-18 — Devcontainer debug package split and release diagnostics

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` inside the manual-only `linux-debug-build` packaging/release path while keeping the successful `./mach build --debug` and `./mach package --debug` flow intact.
- The debug package is now extracted into a temporary runtime tree, inspected, and split into a smaller stripped runtime ZIP plus a separate debug-symbol ZIP instead of throwing symbols away.
- Added size diagnostics before and after stripping, including original tarball size, largest-file inventories, ELF identification, section summaries for the largest ELF files, stripped runtime tree size, debug-symbol tree size, and processed ELF count, so future runs explain what is taking space.
- The workflow now uploads both `release/servo-linux-debug-runtime-stripped.zip` and `release/servo-linux-debug-symbols.zip` as Actions artifacts and to the stable `latest-local-runtime-devcontainer-linux-debug` release tag.
- Added `git config --global --add safe.directory "$GITHUB_WORKSPACE"` before `gh release ...` commands to fix the GitHub Release upload safe-directory failure observed in the container job.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only packaging/release change.

## 2026-06-17 — Devcontainer release upload GitHub CLI fix

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` to install the GitHub CLI inside the Servo devcontainer before the manual `linux-debug-build` release upload step runs.
- The failed command was in the release-publishing path after the build/package/artifact path, because GitHub Actions container-job `run` steps execute inside `ghcr.io/servo/servo/devcontainer-ubuntu:latest`, where `gh` is not guaranteed to be present even when it is available on the host runner.
- Kept the existing build, packaging, artifact upload, stable release tag, `contents: write` permission, and `GH_TOKEN: ${{ github.token }}` release-step authentication unchanged.
- No Servo crates/modules or local-runtime Rust code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only fix.



## 2026-06-17 — Manual Linux debug build workflow gate

- Extended `.github/workflows/local-runtime-devcontainer-smoke.yml` with a Linux-only `linux-debug-build` job that runs in `ghcr.io/servo/servo/devcontainer-ubuntu:latest` only for `workflow_dispatch` and only after the existing `smoke` bootstrap/metadata/servo-net check job succeeds.
- The exact build command is `./mach build --debug`, chosen from `./mach build --help`, where Servo documents `--dev`, `--debug`, and `-d` as development-mode build aliases.
- This is a full Linux debug/development Servo build, not a partial crate or package check. It is kept manual-only to avoid spending push/PR CI time on documentation or small scouting commits.
- Passing this job proves that the current checkout can bootstrap in the published Servo devcontainer image and complete a Linux debug Servo build after the smoke checks, including `cargo check -p servo-net --locked`, pass.
- Passing this job does not prove local-runtime policy behavior, package-scoped asset loading, denied remote schemes, runtime rendering correctness, macOS/Windows support, ARM support, release/profile builds, or test-suite pass status.
- No Servo Rust crates/modules or local-runtime Rust code were touched; no resource paths were loaded, logged, or denied by this workflow-only change.


## 2026-06-17 — Devcontainer CI shell selection fix

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` to set the GitHub Actions default run shell to `bash` for the smoke workflow while leaving the job steps and Servo/local-runtime code unchanged.
- The previous smoke failure occurred before Servo bootstrap: GitHub Actions invoked a `/bin/sh`-compatible shell in the container, and `/bin/sh` rejected the runner's `-o pipefail` option with `set: Illegal option -o pipefail`.
- No Servo crates/modules were touched, and no runtime resource paths were loaded, logged, or denied by this CI-only fix.


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


## 2026-06-17 — Devcontainer debug build release capture

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so the manual `linux-debug-build` job packages the completed `./mach build --debug` output, wraps the package as `release/servo-linux-debug.zip`, uploads it as an Actions artifact, and publishes/clobbers it on a stable GitHub Release tag named `latest-local-runtime-devcontainer-linux-debug`.
- Added workflow `contents: write` permission so the existing `github.token` can create or update the Release from the successful manual build.
- Kept the existing workflow triggers, smoke job, devcontainer image, debug build command, and build environment unchanged.
- No Servo crates/modules or local-runtime Rust code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only release capture change.

## 2026-06-18 — Debian 12 manual Linux artifact target and GLIBC floor reporting

- The known-good Ubuntu/devcontainer manual debug-symbol-split build is size-successful: it produces the stripped runtime ZIP and debug-symbol ZIP, with `servo-linux-debug-runtime-stripped.zip` observed at about 109 MB compressed and about 454 MB unpacked.
- That Ubuntu/devcontainer artifact is not a Debian 12/bookworm keeper artifact because its packaged `servoshell` requires `GLIBC_2.38`, while Debian 12/bookworm provides glibc 2.36.
- Debian 12/bookworm is now the compatibility target for keeper Linux artifacts so the Linux runtime can be built against the Debian 12 glibc floor instead of inheriting Ubuntu 24.04's newer floor.
- Linux artifacts inherit the glibc floor of the build container; building in `debian:12` is therefore the manual path for artifacts intended to run on Debian 12/glibc 2.36 systems.
- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so `workflow_dispatch` exposes a `build_kind` choice and only the selected full build runs; push and pull_request still run only the cheap smoke job.
- Manual artifact builds no longer repeat or wait on the smoke job before building; each manual build path performs its own checkout, bootstrap, build, package, artifact upload, and release upload.
- The workflow now reports the highest required `GLIBC_X.Y` symbol version found in the packaged `servoshell` and summarizes whether that floor should run on Debian 12/glibc 2.36.
- The Ubuntu/devcontainer build only reports a too-new GLIBC floor, while the Debian 12 build fails clearly if its produced artifact still requires newer than `GLIBC_2.36`.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only artifact compatibility change.

## 2026-06-18 — Debian 12 pre-bootstrap uv preflight

- The manual Debian 12 debug symbol-split build failed before Servo bootstrap because `./mach` re-execs through `uv run --frozen ...`, and the plain `debian:12` container did not provide `uv`.
- Inspected Servo's actual prerequisites before changing the Debian path: `.devcontainer/devcontainer.json`, `.devcontainer/Ubuntu.Dockerfile`, `./mach`, `python/servo/platform/linux.py`, `python/servo/platform/linux_packages/apt/*.txt`, and the existing Linux/local-runtime workflows.
- Added `ci/local-runtime/debian12-preflight.sh` for the Debian 12 manual build only. The script prints OS/kernel/user/glibc context, provides `uv` before any `./mach ...` command, and verifies the pre-mach commands plus local-runtime packaging/ABI/release tooling before long build steps begin.
- Replaced the Debian job's broad hand-written bootstrap package block with the focused preflight script. Servo's full Linux build dependency set remains owned by `./mach bootstrap` and `python/servo/platform/linux_packages/apt/*.txt`, rather than being duplicated as a guessed Debian package list in the workflow.
- Future missing-tool failures for this Debian manual build should now happen in preflight with a clear missing-command message before bootstrap, build, package, or artifact-splitting work runs.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only preflight fix.

## 2026-06-18 — Debian 12 Rust toolchain preflight

- The Debian 12 manual build advanced past the missing-`uv` failure, then Servo bootstrap failed while installing `cargo-nextest` because `rustup`, `rustc`, and `cargo` were not available in the plain `debian:12` container.
- Updated the Debian 12 preflight to provide the Rust toolchain from Servo's own sources of truth: `.devcontainer/Ubuntu.Dockerfile` installs rustup under `/usr/local` with the pinned `rust-toolchain.toml` channel and components, and `python/servo/platform/base.py` expects `rustup`/`cargo` before it can install `cargo-nextest`, `taplo-cli`, `cargo-deny`, and `support/crown` during `./mach bootstrap --yes`.
- The preflight now verifies `rustup`, `rustc`, and `cargo` before returning success, and writes `/usr/local/cargo/bin` to `GITHUB_PATH` so subsequent Debian workflow steps can find the toolchain before long build/package steps begin.

## 2026-06-18 — Separate Debian 12 build-room image workflow

- The manual Debian 12 debug symbol-split path successfully produced a Debian-compatible Servo artifact: highest required GLIBC symbol reported as `GLIBC_2.35`, which is compatible with Debian 12/bookworm's glibc 2.36.
- That successful run reported a stripped runtime tree of about 458M, a debug-symbol tree of about 1.6G, and 1 processed ELF file.
- The current artifact workflow still has separate Release-upload ceremony problems; this change deliberately does not modify that artifact workflow or its manual build menu behavior.
- Added a separate reusable Debian 12/bookworm build-room image definition under `ci/local-runtime/debian12-build-image/` for GHCR publishing. The image contains Debian/Servo build prerequisites, `uv`, Rust bootstrap tooling, and packaging/ELF inspection tools, but not Servo source, `/var/servo-cargo-target` output, compiled Servo artifacts, artifact archives, debug symbols, or release-upload logic.
- Added a new manual-only `Local Runtime Debian 12 Build Image` workflow that builds, verifies, and pushes `ghcr.io/${{ github.repository_owner }}/servo-debian12-build:bookworm` and `:latest` without building Servo.
- Future work can switch the Debian 12 artifact build job to use this image after the image workflow is proven, keeping this change as an amputation/separation step rather than a behavior change to current artifact builds.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI/image-only separation.

## 2026-06-18 — Debian 12 build-room image as default artifact path

- Updated `.github/workflows/local-runtime-devcontainer-smoke.yml` so the manual Debian 12 debug symbol-split artifact job now runs in `ghcr.io/thesepeoplearenotyourfriends/servo-debian12-build:bookworm` instead of starting from plain `debian:12`.
- Removed the Debian-only preflight/package-install bootstrap step from the artifact job because the reusable build-room image now carries the Debian/Servo build prerequisites; the job keeps a small image verification step that prints `/etc/os-release`, glibc, `uv`, Clang, Python, Rust, GitHub CLI, and ELF tooling versions before any full build begins.
- Made `debian12-debug-symbol-split` the default `workflow_dispatch` build kind because the proven Debian 12 path produced a `GLIBC_2.35` Servo debug artifact that is compatible with Debian 12/bookworm glibc 2.36 and newer glibc systems.
- Kept the Ubuntu/devcontainer debug symbol-split path as an explicit fallback/proof/comparison option rather than the default artifact path.
- Confirmed the workflow shape keeps push and pull_request on the cheap smoke job only, while manual full build jobs are selected independently and do not repeat or depend on the smoke job first.
- Kept the Debian 12 runtime and debug-symbol ZIP outputs named separately as `servo-linux-debian12-debug-runtime-stripped.zip` and `servo-linux-debian12-debug-symbols.zip`, with GNU debug link handling, compressed size reporting, tree size reporting, and largest-file inventories.
- Extended Debian 12 ABI reporting to include highest required `GLIBC_*`, `GLIBCXX_*`, and `CXXABI_*` symbols where present, plus `ldd` missing-library reporting; the Debian path fails clearly if `servoshell` requires newer than `GLIBC_2.36` or if `ldd` reports `not found`.
- Made Release upload more robust by changing release steps to `cd "$GITHUB_WORKSPACE"`, mark the workspace as a safe Git directory, and pass `--repo "$GITHUB_REPOSITORY"` to `gh release` commands before uploading to the Debian-specific `latest-local-runtime-debian12-linux-debug` tag.
- Uploading GitHub Actions artifacts still happens before Release upload, so a Release ceremony failure should not hide a successful build/package result.
- The Debian 12 artifact path remains full-build manual-only and does not bake Servo source, compiled output, release archives, debug symbols, or upload logic into the build-room image.
- No Servo Rust crates/modules or local-runtime runtime code were touched; no runtime resource paths were loaded, logged, or denied by this CI-only artifact workflow update.
