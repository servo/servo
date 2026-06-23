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

## 2026-06-18 — Retire plain Debian 12 preflight from live CI

- PR #18 produced an inherited Servo Linux artifact that was locally tested.
- The tested `servoshell` emitted `[local-runtime resource-request]` entries for Document, Image, and Script fetches.
- This confirms the first instrumented Servo runtime seam is alive.
- The old Debian plain-base preflight script has been retired from live CI and preserved as recovery documentation in `docs/local-runtime/debian12-plain-base-preflight-recovery.md`.
- Next agenda: build a “reusable remote test bench” that consumes an already-built runtime arena and runs experiments without rebuilding Servo.

## 2026-06-18 — Local-runtime CI release housekeeping

- Audited local-runtime workflow/script references for the retired `ci/local-runtime/debian12-preflight.sh` path; no active workflow or script references remain, and the live script is absent.
- Preserved the old plain-`debian:12` preflight knowledge as recovery-only documentation in `docs/local-runtime/debian12-plain-base-preflight-recovery.md` for use if the Debian 12 build-room image has to be reconstructed.
- Consolidated local-runtime GitHub Release upload ceremony into `ci/local-runtime/upload-release-assets.sh`, which enters `GITHUB_WORKSPACE`, marks it as a safe Git directory, and passes `--repo "${GITHUB_REPOSITORY}"` to `gh release view`, `gh release create`, and `gh release upload`.
- Kept artifact names, packaging behavior, Servo runtime behavior, and local-runtime resource logging behavior unchanged.
- Next agenda remains the reusable remote test bench that consumes an already-built runtime arena and runs experiments without rebuilding Servo.

## 2026-06-18 — Manual checked-release Linux probe artifact workflow

- Inherited Main has been made manual-only to avoid accidental full upstream matrix runs, and local-runtime probe artifacts should not invoke that inherited Main workflow.
- Added `.github/workflows/local-runtime-linux-probe-artifact.yml` as a manual-only workflow that calls the existing reusable Linux workflow directly with the `checked-release` profile and with WPT, Bencher, coverage, C API, libservo, unit-test, and devtools-test jobs disabled.
- The local-runtime probe artifact path now uses the existing Linux checked-release package path directly: `./mach build --use-crown --locked --profile checked-release`, `./mach package --profile checked-release`, and the uploaded `checked-release-binary-linux` artifact from `target/checked-release/servo-tech-demo.tar.gz`, without invoking Main.
- The publish job downloads `checked-release-binary-linux`, republishes it as `servo-linux-local-runtime-probe.tar.gz`, and uploads it through `ci/local-runtime/upload-release-assets.sh` to the stable `latest-local-runtime-probe-linux` Release for explicit Codex/runtime arena probe tasks.
- Debug Debian symbol-split artifacts remain manual rescue/debug/ABI artifacts, not the default Codex arena runtime.
- This is a CI/workflow-only change: no Servo runtime code, network/resource loader code, or `components/net/resource_thread.rs` was touched; no local package resource paths were loaded, logged, or denied.
- Next agenda remains Codex-friendly runtime probing using the probe artifact on explicit request.

## 2026-06-19 — Remote HTTP(S) request denial gate

- Inspected `components/net/resource_thread.rs` at the existing `CoreResourceManager::fetch` local-runtime logging seam before the request enters the legacy fetch/http/protocol machinery.
- Touched `components/net/resource_thread.rs` to add the first real local-runtime policy gate for resolved `http://` and `https://` final URLs only.
- Remote HTTP(S) fetches now still emit the existing `[local-runtime resource-request]` block, but the decision is `denied`, the reason is `RemoteSchemeDeniedByLocalRuntime`, and `deny_kind` is `RequestDenied`.
- Denied HTTP(S) requests are completed through Servo's existing network-error response callbacks instead of panicking, crashing the page, or attempting the actual network load.
- Left `file://` behavior unchanged in this pass, including legacy-path logging and existing local file document/resource handling.
- Still reaches old network/resource assumptions: `file://`, `ws://`, `wss://`, `ftp://`, `store://`, `asset://`, and `bundle://` are not yet enforced by this gate.
- Open question: whether a dedicated local-runtime `NetworkError`/denial kind should replace the temporary `ResourceLoadError("RemoteSchemeDeniedByLocalRuntime")` representation once error taxonomy is formalized across callers.

## 2026-06-19 — `asset://` package loading v0

- Inspected and touched `components/net/resource_thread.rs`, the current late Fetch request seam immediately after `RequestBuilder::build()` and before legacy protocol/http dispatch.
- Added environment-gated package mode using `SERVORENA_PACKAGE_ID` and `SERVORENA_PACKAGE_ROOT`; when both are present, resolved `asset://` URLs are treated as first-milestone package assets.
- Implemented `asset://{package}/...` host checks, path segment normalization, traversal/root escape denial, filesystem mapping under the configured package root, extension-based MIME assignment for `.html`, `.css`, `.js`, `.png`, and `.woff2`, and byte delivery through Servo's existing `FetchTaskTarget` response/chunk/eof path.
- Logged allowed package assets with `decision: package-asset`, `reason: PackageAssetAllowed`, MIME, and the canonical `local_path` used for the package file.
- Denied, in package mode, remote `http://`, `https://`, `ws://`, and `wss://`, `file://`, `store://`, wrong-package `asset://`, missing package files, package root canonicalization failures, I/O failures, and traversal/root escape attempts before legacy fetch/protocol handling.
- Existing `file://` local development behavior remains outside package mode because the new `file://` denial only applies when both package environment variables are present.
- Still reaches old assumptions: the provider boundary is still late in `components/net/resource_thread.rs`, does not yet preserve original unresolved attribute text, and `ftp://`, `bundle://`, `data:`, and `blob:` need follow-up classification at this seam or earlier provider seams.

## 2026-06-19 — encoded `asset://` traversal denial

- Inspected `components/net/resource_thread.rs` package-mode `asset://` authorization after encoded traversal-shaped URLs were observed reaching the loader as normalized package-local assets.
- Added a pre-segment raw-path traversal guard that rejects raw `..` segments, percent-encoded dot-dot segments, and single-percent-decoded paths that introduce separators plus `..` before filesystem mapping.
- Preserved the existing canonical package-root containment check so allowed assets still resolve through the configured `SERVORENA_PACKAGE_ROOT` and symlink/root escapes remain denied.
- Denied encoded traversal attempts now log `decision: denied`, `reason: PackagePathTraversalDenied`, `deny_kind: RequestDenied`, and `local_path: none` from the package-mode seam.
- Open question: this seam can inspect `ServoUrl::path()` after Servo URL parsing, but a future earlier ResourceProvider boundary should carry the exact author-supplied request string so pre-parser policy checks do not rely on URL crate serialization behavior.

## 2026-06-19 — Raw URL laundering fixture and early parse/join logging

Commit range: pending.

Inspected:

- `components/url/lib.rs::ServoUrl::parse_with_base`, `ServoUrl::parse`, and `ServoUrl::join`.
- `components/script/dom/document/document.rs::Document::encoding_parse_a_url`, which calls the URL parser directly with document base URL and encoding override.
- `components/script/script_module.rs::ModuleTree::resolve_url_like_module_specifier`, which routes URL-like static and dynamic module specifiers through `ServoUrl::parse_with_base` or `ServoUrl::parse`.
- The existing late package gate in `components/net/resource_thread.rs::local_runtime_path_has_traversal`.

Findings:

- A fixture now pins the laundering concern: joining `./%2e%2e/secret.txt` against `asset://com.example.app/app/index.html` serializes as `asset://com.example.app/secret.txt`, so a late gate that only sees `url.path()` no longer sees a `..` segment.
- Normal percent encoding remains allowed by the raw-text classifier, e.g. `%20` for spaces and `%7B...%7D` in a filename.
- Suspicious raw path-boundary encodings are classified before URL normalization: encoded dot, slash, backslash, NUL, and a second percent layer immediately before those bytes.

Touched crates/modules:

- `components/url`: added raw URL obfuscation classification, early log entries in `ServoUrl::parse`, `ServoUrl::parse_with_base`, and `ServoUrl::join`, and package-mode denial before URL normalization.
- `components/url/tests`: added fixtures for ordinary percent encoding, suspicious raw path-boundary encodings, the exact join normalization behavior, and package-mode denial.

Logged successfully:

- Raw suspicious input passing through `ServoUrl::parse`, `ServoUrl::parse_with_base`, or `ServoUrl::join` emits `[local-runtime raw-url-request]` with requested text, base URL, module, decision, and reason.

Denied successfully:

- In package mode (`SERVORENA_PACKAGE_ID` and `SERVORENA_PACKAGE_ROOT` set), suspicious raw URL text is denied at parse/join time before the normalized URL can hide the original text.

Still reaches old assumptions:

- `Document::encoding_parse_a_url` still calls `url::Url::options().parse(...)` directly rather than `ServoUrl::parse_with_base`, but now uses the shared raw obfuscation classifier immediately before the encoding-aware parse.
- Some non-script resource callers use direct base `join(...)` and are covered by `ServoUrl::join`, but their higher-level destination labels are not yet represented in the raw log.

Open questions:

- Whether CSS URL parsing has any direct `url` crate paths that bypass `ServoUrl` and need equivalent raw-text fixtures.
- Whether package-mode parse denial should return a more specific local-runtime error once URL parsing is moved behind a real host `ResourceProvider` boundary.

## 2026-06-20 — CSS @import and nested CSS URL base handling

- Inspected `components/script/stylesheet_loader.rs` for linked stylesheet parsing and Stylo `StylesheetLoader::request_stylesheet` handling of CSS `@import` rules.
- Inspected `components/fonts/font_context.rs` for `@font-face` `src: url(...)` loads started from parsed stylesheets.
- Discovered that CSS `@import` URLs are already resolved by Stylo against the stylesheet parser `UrlExtraData`, then Servo calls `ElementStylesheetLoader::load_with_element` with `Destination::Style`, which reaches the existing net package wall in `components/net/resource_thread.rs`.
- Added local-runtime request logging at the `@import` loader handoff so nested stylesheet loads are visible before they enter fetch.
- Discovered that web-font URL values are resolved from stylesheet data, but the fetch request referrer/base context used the document URL. Updated stylesheet-initiated web-font fetches to use the parsed stylesheet URL as the request referrer/base context and added a local-runtime log entry with `destination: Font`, stylesheet base, and document initiator.
- CSS image `url(...)` loads still need a separate pass through the style/layout image-request path; this pass did not identify or modify that caller.

## 2026-06-20 — Milestone 2 execution-shape planning

- Inspected `docs/local-runtime/usual_suspects.md`, `docs/local-runtime/loader-map.md`, and `docs/local-runtime/resource-provider-plan.md` against the achieved first-milestone status and the 16-target classification table.
- Rewrote the Milestone 2 addendum in `docs/local-runtime/usual_suspects.md` so the target matrix is not treated as a strict top-to-bottom execution order.
- Identified the next useful slice as external modules/static imports/dynamic `import()` plus raw author-text/base/resolved-URL logging at module and document seams, with WebSocket handled separately and early because it has a distinct resource-thread path.
- Split follow-up work into package machinery to preserve/prove (`modules`, CSS graph, fonts, and maybe dedicated workers later) and browser machinery to gut/defer (`WebSocket`, EventSource, beacon, XHR, service workers, and URL media).
- No Servo runtime crates were touched in this pass.
- No new resource paths were loaded, logged, or denied; this remains a planning/documentation pass to choose the next focused probe family before changing behavior.

## 2026-06-20 — Slice A addendum 1 module evaluation provenance

Commit range: pending local instrumentation change.

Inspected:
- `components/script/script_module.rs` top-level module execution, URL-like module specifier resolution, dynamic-import hook entry, and `fetch_a_single_module_script` module-map lookup/start-fetch path.
- `components/script/module_loading.rs` dynamic import continuation, host imported-module loading, link, evaluate, fulfill, and reject paths.

Touched crates/modules:
- `components/script/script_module.rs`
- `components/script/module_loading.rs`

Resource/module paths discovered or clarified:
- Dynamic import enters SpiderMonkey's host hook in `host_import_module_dynamically`, then uses `host_load_imported_module` to resolve the author specifier against the referencing module's `ModuleScript::base_url`.
- Imported module fetches still converge on `fetch_a_single_module_script`, whose module map is keyed by `(ServoUrl, ModuleType)` and distinguishes absent, fetching, and loaded entries before either starting a fetch, attaching a waiter, or reusing a loaded module.
- Top-level external module execution warning provenance is available in `ModuleTree::execute_module`, immediately around `ModuleEvaluate` and `ThrowOnModuleEvaluationFailure(..., ThrowModuleErrorsSync)`.

Logged successfully by this source pass:
- `[local-runtime module-resolution]` for local-runtime-relevant specifier edges, preserving raw-author-text at `resolve_module_specifier` / `host_load_imported_module` and resolver-input at `resolve_url_like_module_specifier`.
- `[local-runtime module-map]` for `asset://` and `bundle://` module map state/action before fetch/reuse/waiter behavior.
- `[local-runtime module-evaluation]` for top-level module `ModuleEvaluate`, `ThrowOnModuleEvaluationFailure`, dynamic import host-load/link/evaluate/fulfilled/rejected stages.

Denied successfully:
- No policy changes in this pass. Existing package-wall denial behavior is intentionally unchanged.

Still reaches old network/resource assumptions:
- This pass only logs script-module provenance. Actual module bytes still flow through the existing fetch/resource-thread wall and package-mode policy seam.

Failed approaches and why they failed:
- Did not inspect or stringify pending JS exceptions for richer messages because that risks clearing, consuming, or otherwise changing exception behavior before Servo's normal reporting path.

Open questions for the next pass:
- Which exact module URL is associated with the observed `fail to evaluate module` warning when the updated runtime is run against Servorena's `module-dynamic-relative` fixture?
- Do duplicate `entry.js` resource-thread lines correspond to a true second start-fetch action, an attach-waiter state, a reuse-loaded state, or a subtly different `(URL, ModuleType)` key?

## 2026-06-20 — top-level-await module evaluation semantics

* Inspected the pinned SpiderMonkey dependency used by this checkout: `mozjs = 0.16.3` and `mozjs_sys = 140.11.0-1` from `Cargo.lock`.
* Verified the exact pinned header at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mozjs_sys-140.11.0-1/mozjs/js/public/Modules.h`:
  * `ModuleEvaluate` returns a bool for API-call success and writes either `undefined` or an evaluation Promise to `rval`; if already evaluated, it returns the evaluation Promise.
  * `ReportModuleErrorsAsync` reports module evaluation errors asynchronously when the evaluation Promise is rejected and is used for web content.
  * `ThrowModuleErrorsSync` throws by setting a pending exception and explicitly does not support modules that use top-level await.
  * `ThrowOnModuleEvaluationFailure` requires module evaluation to have completed before synchronous rethrow semantics are used.
* Inspected implementation in the same pinned source at `mozjs/js/src/builtin/ModuleObject.cpp`: `ThrowModuleErrorsSync` assumes the evaluation Promise is already fulfilled or rejected, while `ReportModuleErrorsAsync` attaches an async rejection handler.
* Touched `components/script/script_module.rs` only for runtime behavior: top-level external module execution now chooses `ReportModuleErrorsAsync` when SpiderMonkey returns a pending evaluation Promise, preserving `ThrowModuleErrorsSync` for settled ordinary module evaluation.
* Cleaned local-runtime module-evaluation instrumentation labels so `ModuleEvaluate`'s bool is logged as API-call success rather than semantic fulfillment.
* Added focused WPT coverage for an external module whose top-level await resolves and one whose top-level await rejects with an attributable `Error`.
* No package policy, URL routing, relative module resolution, dynamic import behavior, networking behavior, or duplicate-request behavior was intentionally changed.

## 2026-06-20 — Slice B URL provenance instrumentation

Commit range: pending local instrumentation change.

Inspected:
- `components/url/lib.rs` for `ServoUrl::parse`, `ServoUrl::parse_with_base`, and `ServoUrl::join` before URL normalization removes raw spelling.
- `components/script/dom/document/document.rs` for `Document::encoding_parse_a_url`, including its direct `url::Url::options().parse(...)` encoding-aware path.
- `components/script/dom/html/htmlscriptelement.rs` for external script `src` resolution against the active document base URL.
- `components/script/script_module.rs` and `components/script/module_loading.rs` for URL-like module specifier resolution and host static/dynamic module import loading.
- `components/net/resource_thread.rs` only as the existing late resource-policy logging correlation endpoint; no policy was moved into URL resolution in this pass.

Touched crates/modules:
- `components/url/lib.rs`
- `components/script/dom/document/document.rs`
- `components/script/dom/html/htmlscriptelement.rs`
- `components/script/script_module.rs`
- `components/script/module_loading.rs`

Logged successfully by this source pass:
- Added `[local-runtime url-resolution]` records at low-level Servo URL parse/join seams with `resolver_input`, base used by that parser call, resolved URL or parse failure, and source seam.
- Added owning document URL-resolution records at `Document::encoding_parse_a_url` with `author_text`, document base URL, resolved URL or early raw-obfuscation rejection, and `destination: Document`.
- Added owning external script `src` records in `HTMLScriptElement::prepare` with raw `src` attribute text as `author_text`, the document base URL used by `base_url.join`, resolved URL or parse failure, and `destination: Script`.
- Tightened module-resolution records to the shared `[local-runtime url-resolution]` vocabulary for static URL-like module imports and host-loaded static/dynamic module imports, preserving raw specifier text where SpiderMonkey still exposes it and logging the importer/module base used for resolution.

Denied successfully:
- No new allow/deny policy was added. Existing package-mode raw-obfuscation denial remains unchanged; this pass only adds source-level provenance around it.

Still reaches old network/resource assumptions:
- Final fetch authorization still happens at the existing resource-thread package wall after requests have already been resolved to `ServoUrl`.
- Import-map helper paths can still operate on normalized URL strings; preserving every raw import-map mapping token would require a future import-map/request provenance shape rather than only resolver logging.

Normalization/raw-text findings:
- `ServoUrl::parse`, `ServoUrl::parse_with_base`, and `ServoUrl::join` are the earliest shared Rust seams before the `url` crate parses and serializes URL text.
- `Document::encoding_parse_a_url` bypasses `ServoUrl::parse_with_base` for encoding-aware parsing, so it must log at the owning document seam before its direct `url::Url::options().parse(...)` call.
- External script `src` raw author text is still available in `HTMLScriptElement::prepare` before `base_url.join(&src)`.
- Static and dynamic module raw specifier text is still available at `resolve_module_specifier` / `host_load_imported_module`; after `resolve_url_like_module_specifier` and import-map matching, later module-map/fetch layers mostly see normalized `ServoUrl` identity.

Open questions:
- Whether CSS image `url(...)` resolution has a comparable owning seam that can preserve raw token spelling before Stylo hands Servo an already-resolved URL.
- Whether a future host request type should carry both `author_text` and `resolver_input` through module-map/fetch boundaries so final policy logs can correlate without relying on adjacent log timing.

## 2026-06-20 - "Dial Tone" Test Series Results

The local-runtime fork has now been subjected to a broad browser-shaped escape barrage, not merely a few hand-picked URL denials.

Across the completed feature-backed escape sweep:

```text
Direct transport APIs:  30/30 completed
HTML loader routes:     54/54 completed
Worker routes:          30/30 completed
Weird transports:       0/0 feature-backed checks; APIs not exposed
```

The resulting observed escape success rate was:

```text
0 successful outbound escapes
0 AF_INET sockets observed
0 AF_INET6 sockets observed
0 AF_PACKET sockets observed
0 AF_VSOCK sockets observed
```

The combined field result was:

```text
ESCAPE SWEEP PASS
No AF_INET / AF_INET6 / AF_PACKET / AF_VSOCK activity observed.
```

This does not claim that every possible future engine path has been disproven forever. It does mean that the ordinary browser-shaped ways a document would normally try to acquire a network path have now been actively exercised and, in the current fork, failed to obtain one.

---

## Threat model being tested

The intended local-runtime rule is:

```text
local document program
→ may read only assets belonging to its own package
→ may later read explicitly runtime-owned local assets
→ may not read arbitrary filesystem paths
→ may not read another package
→ may not reach a remote URL
→ may not obtain a network transport
```

The test question is deliberately simple:

```text
Can a document-shaped program, through any ordinary browser organ,
make Servo obtain a phone line?
```

The answer from the current field battery is:

```text
No observed tested route obtained one.
```

The resource wall remains the primary source-side authority:

```text
engine asks for bytes
→ allowed same-package asset: provide bytes
→ anything else: deny
```

The kernel-side dial-tone check is the independent receipt:

```text
Did the process create or use an Internet-family socket anyway?
```

For the completed sweep, the answer was no.

---

## Evidence model

Each attempted escape has two separate kinds of evidence.

### 1. Source-layer evidence

The runtime records a request and applies the local package wall. Typical observed outcomes include:

```text
RemoteSchemeDeniedByLocalRuntime
FileSchemeDeniedByLocalRuntime
PackageIdMismatch
RequestDenied
```

This is evidence that the request reached the normal resource decision point and was rejected there rather than handed to an outbound loader.

### 2. Kernel-side dial-tone evidence

Each scan can run under:

```text
strace -f -e trace=network
```

The sweep treats the following as red-flag families:

```text
AF_INET
AF_INET6
AF_PACKET
AF_VSOCK
```

Routine local plumbing is intentionally not treated as a network escape:

```text
AF_UNIX
AF_NETLINK
```

Those are suppressed from the normal result because they are local IPC and kernel-notification traffic, not an Internet connection.

A clean dial-tone receipt therefore means:

```text
The tested process did not visibly create or use an Internet-family,
raw packet, or VSOCK network route during that probe.
```

---

## Forbidden target battery

The escape fixtures use a shared set of hostile targets chosen to test several distinct failures:

```text
http://127.0.0.1:9/escape-ipv4
http://[::1]:9/escape-ipv6
https://example.invalid/escape-https
file:///tmp/escape-file
asset://wrong.example.app/escape-wrong-package
//example.invalid/escape-protocol-relative
```

These are not all the same test in different spelling.

```text
127.0.0.1:9
→ verifies that loopback is not silently treated as permitted “local networking”

[::1]:9
→ verifies IPv6 loopback is not a separate escape hatch

https://example.invalid
→ verifies ordinary remote HTTPS does not get an outbound path

file:///tmp/...
→ verifies a document cannot turn the runtime into arbitrary filesystem access

asset://wrong.example.app/...
→ verifies package identity is enforced rather than merely using an asset-shaped URL

//example.invalid/...
→ verifies protocol-relative / URL-resolution ambiguity cannot turn into a usable external route
```

---

## Direct transport barrage: PASS, 30/30

The direct transport fixture exercised five browser-facing request APIs across the six-target battery:

```text
fetch
XMLHttpRequest
navigator.sendBeacon
EventSource
WebSocket
```

That produced:

```text
5 transport families × 6 hostile targets = 30 completed probe slots
```

The fixture accepted ordinary API-level rejection/error behavior as a correct containment outcome, including cases such as:

```text
TypeError: Network error: RemoteSchemeDeniedByLocalRuntime
constructor rejection
error event
close without opening
contained harness timeout
```

WebSocket was especially important because it has a dedicated loader/handshake path and cannot be treated as merely another fetch-shaped request.

Observed result:

```text
TRANSPORT-ESCAPE-SCAN 30/30
DIAL-TONE: PASS
```

No direct transport API in the tested set was observed to bypass the local resource wall or obtain an Internet-family socket.

---

## HTML loader barrage: PASS, 54/54

The HTML loader fixture exercised nine element-driven loading routes across the same six hostile targets:

```text
<img>
<script src>
<iframe src>
<object data>
<embed src>
<video src>
<video poster>
<audio src>
<form action=... target=hidden-iframe>
```

That produced:

```text
9 HTML loader families × 6 hostile targets = 54 completed probe slots
```

This matters because HTML loaders are not one generic API. Browser engines have historically accumulated many loader routes with different destinations, initiators, timing models, and backend handoffs.

The observed runtime logs included ordinary resource-wall denials even for media destinations, for example:

```text
destination: Video
decision: denied
reason: RemoteSchemeDeniedByLocalRuntime
```

and corresponding denials for filesystem and wrong-package targets.

The fixture originally ran the 54 slots serially. Some loader types, especially poster loading, waited for their local harness timeout, so the page could exceed its per-document time budget before emitting its final completion marker. That was a fixture scheduling problem, not a network escape.

The runner was changed to launch the existing probes concurrently and wait for all promises to settle. The completed result became:

```text
HTML-LOADER-ESCAPE-SCAN 54/54
DIAL-TONE: PASS
```

No tested HTML loader family obtained an observed Internet-family socket.

---

## Worker barrage: PASS, 30/30

The worker fixture tested both remote worker entry points and second-stage loading from an already package-local worker.

Routes exercised:

```text
remote classic Worker
remote module Worker
remote SharedWorker
package-local worker → importScripts(remote target)
package-local module worker → dynamic import(remote target)
```

Across the six-target battery:

```text
5 worker routes × 6 hostile targets = 30 completed probe slots
```

The local-worker cases matter because they test the more subtle possibility:

```text
document itself is properly local
→ document starts a package-local worker
→ worker attempts to acquire a new remote script on its own
```

That is exactly the kind of “I am already inside; perhaps I have a different door” path that cannot be assumed safe from the main-document result alone.

Observed result:

```text
WORKER-ESCAPE-SCAN 30/30
DIAL-TONE: PASS
```

No tested worker entry point or worker-side secondary import path was observed to obtain a network route.

---

## Weird transport barrage: NOT EXPOSED, clean trace

The weird-transport fixture checks for high-value APIs that could imply less ordinary transport behavior:

```text
WebTransport
RTCPeerConnection with STUN-shaped configuration
```

In the current runtime build, those constructors were not exposed to the document. The result is intentionally reported as:

```text
NOT EXPOSED — WEIRD-TRANSPORT-ESCAPE-SCAN 0/0 feature-backed checks
```

This is not inflated into fake feature coverage.

Correct interpretation:

```text
WebTransport / WebRTC transport behavior was not exercised because those APIs were absent.
Their absence itself means they did not present an available document-side escape route in this build.
The trace for that probe was still clean.
```

Future runtime changes that expose either API should turn this fixture into an active feature-backed test automatically.

---

## Earlier coverage already in place

The escape sweep sits on top of earlier package-wall and loader coverage rather than replacing it.

Existing field-tested areas include:

```text
same-package static assets
nested package assets
classic scripts
static modules
dynamic relative modules
bare module specifiers
remote module specifiers
cross-package module attempts
literal traversal
encoded traversal
encoded slash traversal
double-encoded traversal
CSS imports
nested CSS graphs
CSS URL-bearing properties
font loading paths
WebSocket denials
```

The CSS and font work matters because they exercise parser and loader paths that are not equivalent to a direct JavaScript `fetch()` call.

The CSS dial-tone work specifically tested hostile CSS-driven resource acquisition attempts while verifying that observed requests remained subject to the ordinary local resource wall and did not produce an Internet-family socket.

The dedicated WebSocket fixture established that WebSocket traffic is denied before it can become a handshake/backend path.

---

## What the 100% failure rate means

For an attacker trying the tested routes:

```text
tested escape attempts: failed to obtain a dial tone
observed successful outbound connections: 0
```

For the runtime:

```text
completed feature-backed probe slots: 114
completed escape sweep: 114/114
observed non-local socket families: 0
```

The 114 completed feature-backed slots are:

```text
30 direct transport attempts
54 HTML loader attempts
30 worker attempts
```

The feature-gated weird transport fixture is deliberately excluded from the 114 because its relevant constructors were not present.

This is not merely “the page got an error.” The important combined result is:

```text
source layer:
  hostile routes were denied/rejected/contained

kernel layer:
  no AF_INET / AF_INET6 / AF_PACKET / AF_VSOCK activity observed
```

That two-part result is much stronger than a UI error or a JavaScript rejection alone.

---

## Current operational test shape

The workflow now has two different roles.

### Individual probes: diagnostic microscope

Individual probes retain detailed output for investigating a particular path:

```text
raw resource records
URL provenance
loader-specific logs
full page output
per-probe dial-tone receipt
```

Use these when a route fails, changes behavior, or needs source-level attribution.

### `all-escape-scans`: one-click leak detector

The aggregate sweep is the normal field check.

It runs:

```text
transport_escape_scan.html
html_loader_escape_scan.html
worker_escape_scan.html
weird_transport_escape_scan.html
```

Each document gets separate captured output and separate strace output.

On success, the user sees only the compact result:

```text
transport APIs: PASS
HTML loaders: PASS
workers: PASS
weird transports: NOT EXPOSED
DIAL-TONE: PASS across all four
ESCAPE SWEEP PASS
```

On failure, the workflow identifies the exact failing document and prints only:

```text
the compact summary
the failing document’s final raw-output tail
the matching Internet-family strace lines, if any
```

This replaces the former unusable behavior where a combined run produced thousands of lines of URL parsing and local IPC noise before the actual answer.

---

## Present conclusion

The project has passed an important threshold.

Before this barrage, the local-runtime claim was structurally plausible:

```text
remote requests should hit the package wall
```

After the barrage, it has a repeatable field receipt:

```text
ordinary document APIs were exercised
HTML loaders were exercised
worker secondary-import routes were exercised
WebSocket was exercised
CSS and font routes were exercised
no completed tested route obtained a dial tone
```

The current practical claim is therefore:

```text
Within the browser-shaped APIs and loader families exercised so far,
the local document runtime behaves as a sealed package runtime:
same-package local assets work;
wrong-package, filesystem, remote, and transport-shaped escape attempts
do not obtain an observed outbound route.
```

---

## Guidance for future code work

Treat the escape sweep as a preservation test, not a one-time ceremony.

Any change touching:

```text
components/net
resource routing
HTTP loader code
fetch code
WebSocket code
worker/module loading
CSS URL handling
media loading
protocol parsing
host capability plumbing
```

should preserve:

```text
ESCAPE SWEEP PASS
```

A future API becoming exposed is not automatically a failure. It is a new tested surface.

The correct pattern is:

```text
new browser-shaped exit appears
→ add a targeted fixture route
→ include it in the one-click sweep
→ require source denial plus clean dial-tone receipt
```

The central rule remains unchanged:

```text
A local document can ask for bytes.
The host may supply permitted local package bytes.
Everything else gets no phone line.
```


## 2026-06-21 — servoshell bare no-egui presentation mode

- Inspected `ports/servoshell/prefs.rs`, `ports/servoshell/desktop/headed_window.rs`, and `ports/servoshell/desktop/gui.rs` for the headed desktop startup, event routing, egui AccessKit initialization, and render-to-parent presentation seam.
- Added a headed desktop runtime preference/CLI path for `--no-egui` that leaves normal startup unchanged while allowing a native decorated window to present the active Servo WebView directly without constructing egui.
- Touched `ports/servoshell/desktop/headed_window.rs` so the GUI is explicitly absent in bare mode, toolbar height is zero through the existing accessor, Servo WebView resize/input coordinates use the full client area, and the existing offscreen render context plus `render_to_parent_callback()` are used for direct presentation.
- Bare mode now drops/dismisses GUI-only embedder controls through their existing safe responses: file picker dismissed, confirm/prompt dismissed, alert confirmed, permission denied, Bluetooth cancelled, authentication dropped, select/color/context menu resolved or dismissed through their existing defaults.
- No new resource acquisition paths were discovered; this change is presentation/event-hosting work rather than local package loader policy work.
- AccessKit remains egui-owned in normal mode. Bare mode does not initialize egui's AccessKit adapter and ignores accessibility app events after updating shell accessibility state, leaving direct AccessKit architecture deferred.
- Deferred validation: broad servoshell build/check remained too large for this agent run from the current checkout; parser coverage was added but the full focused command did not complete before being stopped.

## 2026-06-21 — Plain Debian 12 recovery workflow restoration

- Restored `ci/local-runtime/debian12-preflight.sh` from historical commit `9ce4a68580502f62723530eabeecea9557e92815` as active executable CI logic for the manual plain-Debian recovery path.
- Added `.github/workflows/local-runtime-debian12-recovery.yml` as a `workflow_dispatch`-only recipe-restoration workflow: stock `debian:12`, restored preflight, fail-fast Git/Rust/toolchain provenance gate, `./mach bootstrap --yes`, `./mach build --debug`, `./mach package --debug`, stripped runtime extraction, `nogit` rejection, and one uploaded runtime artifact containing `BUILD-RECEIPT.txt`.
- Added `ci/local-runtime/debian12-recovery-preflight.Dockerfile` as a saved remote preflight candidate kitchen only. It starts from `debian:12`, runs the restored preflight with only `rust-toolchain.toml` copied long enough to select Rust, and does not copy Servo source or run bootstrap/build/package.
- No Servo runtime code, local-runtime resource policy code, or `--no-egui` behavior was touched. No runtime resource paths were loaded, logged, or denied by this CI-only restoration.

## 2026-06-23 — Initial CPython embedding crate

- Inspected the public Servo embedding surface in `components/servo/lib.rs`, `components/servo/servo.rs`, `components/servo/webview.rs`, and the software-rendering test helper in `components/servo/tests/common/mod.rs` to identify the smallest in-process API for constructing `Servo`, `WebView`, and `SoftwareRenderingContext` without going through `ports/servoshell` or a spawned executable.
- Added `ports/severin-python/` as an experimental native CPython extension crate producing a CPython module named `severin`.
- The new Rust `severin.App` object owns the host-side `servo::Servo`, `servo::WebView`, and software rendering context directly. It does not bind a port and does not communicate over HTTP, WebSocket, Unix socket, localhost, or a helper process.
- Implemented the initial Python API shape: `App(width, height, bridge=None)`, `load_path(path)`, `run()`, `close()`, `write()`, and `read()`.
- `load_path` currently adapts a local entry file to the existing first-milestone package wall by setting `SERVORENA_PACKAGE_ID=com.example.app`, `SERVORENA_PACKAGE_ROOT` to the entry parent, and loading `asset://com.example.app/<entry filename>` in the owned WebView. This is a compatibility seam, not the final explicit multi-package provider shape.
- Documented the Python API, bridge transport model, cancellation/shutdown behavior, transport receipt lifetime, and Python GIL/threading rules in `docs/local-runtime/python-embedding.md`, with a shorter boundary note in `docs/local-runtime/resource-provider-plan.md`.
- No new resource category was routed through the final provider in this pass. The main old assumption that remains is the process-global package environment used by the current package wall; this must become per-App/per-package host state before multiple simultaneous Python apps are safe.
- Reworked the bridge design as a transport queue rather than an application protocol: Rust carries only valid serialized JSON frames plus private opaque receipts, while Python owns all application semantics and reply conventions.
- `App.read()` is now defined as the Python-side inbound mail slot, returning `None` or `(receipt, json_text)`. `App.write(receipt, json_text)` validates JSON and attempts to route the reply to that private receipt; unknown receipts are transport/lifetime failures, not application errors.
- Open questions: where to install the powerless page-visible Promise shim, which Servo script/user-content seam should enqueue JS JSON frames, how document teardown should reject affected Promises, and whether Python awaitables should be driven by Python's event loop or by a Rust-side transport executor.
