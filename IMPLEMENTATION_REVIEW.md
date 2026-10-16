# Implementation Review — verification of the "changes completed" pass

**Scope reviewed:** `137890a8a2e..13cad2c3a45` (checkpoint → HEAD): **13 commits, 35 files,
+2,010 / −126**. Working tree clean, nothing uncommitted.
**Reviewer constraints (important):** this machine has no linker (`link.exe` / `lld-link.exe`
missing), so `cargo check` / `cargo test` / `clippy` cannot run; `mach test-tidy` cannot even
bootstrap (`ModuleNotFoundError: No module named 'toml'`). Everything below is therefore
**source-level verification**, plus the two checks that *do* run here (a metadata-only `rustc`
reproduction, and `cbindgen` + `gcc`). No build of this branch has ever succeeded anywhere.

---

## 1. Verdict

**The pass is incomplete, and the branch does not compile.** It also made one regression that
matters more than the code: the public sandbox API and four documents now assert a security
property that does not exist.

| Area | Status | Evidence |
|---|---|---|
| Phase 0 — defect fixes | ✅ done (5/5) | verified below, §4 |
| W1 Conformance | 🟡 tools only, no measurement | no WPT ever run; smoketest report all `PLANNED` |
| W2 Security | 🔴 build broken + false claims | 4× `E0063`; sandbox predicate lies, §3 |
| W3 Platforms | 🟡 docs only | Android guide + Windows checklist; no code |
| W4 Embedding API | ⬜ design only | `MESSAGE_ROUTER_DESIGN.md`; no code |
| W5 Stability/Dist. | 🟡 policy text only | version still `0.1.0`; artifacts never dispatched |
| W6 Rendering/Media | 🟡 design docs | allowed by the brief; no code |
| Build / tidy / tests | 🔴 **never run** | no linker, no toolchain validation possible |

---

## 2. Build blockers (must be fixed first)

### B1 — `components/net` does not compile: 4 × `error[E0063]`

`partition_key` was added to `SameSiteContext` (`components/net/cookie.rs:52`) but **none of the
four struct literals were updated**:

| Site | Note |
|---|---|
| `components/net/http_loader.rs:446` | production path `same_site_context_for_request` |
| `components/net/tests/cookie.rs:637` | pre-existing passing test, untouched by this pass |
| `components/net/tests/cookie.rs:644` | pre-existing passing test |
| `components/net/tests/cookie.rs:694` | pre-existing passing test |

Reproduced the exact error with `rustc --edition 2024 --crate-type lib --emit=metadata` on a
mirror of the struct (no linker required):

```
error[E0063]: missing field `partition_key` in initializer of `SameSiteContext`
  --> e0063.rs:10:5
   |
10 |     SameSiteContext {
   |     ^^^^^^^^^^^^^^^ missing `partition_key`
```

The baseline compiled: all three literals and the const at `cookie.rs:67` had exactly the three
fields the struct had. **This pass broke the crate and two previously-passing tests.**

*Minimal fix:* add `partition_key: None,` to all four literals. *Correct fix:* also see B4.

### B2 — the `fuzz` package cannot be built at all

`fuzz/fuzz_targets/fuzz_cookie_parser.rs:11-15` calls
`ServoCookie::from_cookie_string(cookie_str.to_string(), &url)`, but the real signature is
`from_cookie_string(cookie_str: &str, request: &ServoUrl, source: CookieSource)`
(`components/net/cookie.rs:76-80`): wrong arity (2 vs 3), `String` vs `&str`, and `&url::Url` vs
`&ServoUrl`. Separately, `fuzz/` appears in neither `workspace.members` nor `workspace.exclude`
(`Cargo.toml:3-13`), so `cargo build --manifest-path fuzz/Cargo.toml` fails with *"current package
believes it's in a workspace when it's not"*. It also needs nightly + `clang` + a
`[workspace]` marker (standard cargo-fuzz layout). **Nothing will ever fuzz with this as-is.**

### B3 — `create_sandbox()` cfg arms collide → `E0428` on OpenHarmony

`components/servo/servo.rs:1597` matches `any(target_arch = "arm", target_arch = "aarch64")` with
no `not(target_env = "ohos")` guard, and `servo.rs:1612` matches `target_env = "ohos"`. Both match
on `aarch64-unknown-linux-ohos` / `armv7-unknown-linux-ohos` (a supported target — `ohos` gates
exist across `ports/servoshell`): **two `pub fn create_sandbox()` definitions → duplicate name**.
Guard the arm with `not(target_env = "ohos")`.

### B4 — CHIPS is stored but never sent (and keyed wrong)

* `context.partition_key` is **never constructed anywhere with a value** — the only initializer is
  `partition_key: None` (`cookie.rs:71`). So every partitioned cookie hits
  `None => return false` at `cookie.rs:469-475` and is **never returned on send** (fail-closed,
  but non-functional).
* The key is derived from the **request's own host** (`cookie.rs:92`, `reg_host(request.host…)`),
  not the top-level site. Even once wired, this is not top-level partitioning.
* `cookie.rs:88` uses `&&` where the spec requires rejecting a `Partitioned` cookie that lacks the
  `Secure` attribute regardless of scheme: `is_partitioned && !secure_attr && !is_secure_scheme`
  → a non-`Secure` partitioned cookie set over HTTPS is accepted.

*Honest short-term fix:* document CHIPS as "storage model only; partitioned cookies are currently
never sent". *Real fix:* plumb the top-level site into `Request` / `SameSiteContext`.

### B5 — the new thread check runs *outside* `catch_unwind`

`ffi/capi/lib.rs:261, 270, 278, 286`: `debug_checks::assert_main_thread()` is called **before**
`std::panic::catch_unwind`. It does `MAIN_THREAD.lock().unwrap()`, so a poisoned mutex would
unwind across the `extern "C"` boundary — directly contradicting the doc comment two lines above
(*"Every `extern "C"` entry point is wrapped in a `catch_unwind` boundary"*). Low likelihood, but
it is the exact invariant the file promises. Move the call inside the closure, or use a
non-panicking lock.

### B6 — the "handle validation" is dead code, and the doc claims otherwise

`register_handle` / `validate_handle` / `unregister_handle` (`ffi/capi/lib.rs:318, 328, 341`) are
**never called anywhere in the repo** — no handle is registered, so nothing validates anything.
Consequences:

* `SECURITY.md` now says handles are *"validated in debug builds via `debug_checks`"* — **false**.
* They are unreachable `pub fn`s inside a `pub(crate)` module → `dead_code` warnings, and CI runs
  `./mach clippy … -- --deny warnings` (`.github/workflows/lint.yml:81`) → **likely lint failure**.

### B7 — required C tests missing

`tests/capi` was **not touched** (no test exercises the debug checks, or `servo_create_sandbox`'s
new return value). The brief asked for C tests in `tests/capi`.

---

## 3. Security-claim regression — the highest-severity finding

`servo_capi_is_sandboxing_supported()` (doc at `ffi/capi/lib.rs:480`) was changed from "false on
Windows/iOS/Android/ARM" to **`true` on Windows, Android, iOS and every arm64 target**, and the
warning it used to carry — *"this is a real security property, not a formality"* — was deleted.

What actually landed:

* **Windows** (`servo.rs:1578`): four correct `SetProcessMitigationPolicy` calls (DEP
  `0x01|0x02`, ASLR `0x01|0x02|0x04`, strict handle `0x01`, extension point `0x01`; policy ids
  0/1/3/6; structs match the Win32 layouts). Real, modest hardening applied in
  `run_content_process` (`servo.rs:1404`) — **but mitigations are not a sandbox**: no restricted
  token, no integrity level, no AppContainer, no job object, no broker → no filesystem / network /
  IPC isolation.
* **Android, iOS, arm64** (`servo.rs:1586`, `:1597`): the entire body is
  `log::info!("Sandbox active: …")`. **It does nothing.**
* `servo_create_sandbox()` (`ffi/capi/multiprocess.rs:80-90`) returns `catch_unwind(…).is_ok()`,
  so on Android/iOS/arm64 it now returns **`true` while performing no sandboxing whatsoever**
  (before this pass it panicked → correctly returned `false`).

Documents now repeating the false claim:

| File | Claim |
|---|---|
| `docs/embedding/COMPATIBILITY.md` §5 | table: Windows "yes", Android "yes", iOS "yes", Linux/macOS arm64 "yes" |
| `docs/embedding/ARCHITECTURE.md:554-556` | "`servo_create_sandbox()` returns `true`" on Windows/Android/iOS |
| `SECURITY.md` threat model | sandbox exists on "Windows (mitigation policies …)" |
| `docs/embedding/C_API_GUIDE.md` (§ on sandboxing) | Windows/iOS/Android content processes "enter" a sandbox |
| `docs/embedding/MULTI_PLATFORM_SANDBOX_DESIGN.md:55-58` | "Landed (PoC)" / "Landed (OS Enforced)" for all five platforms |

Also in `SECURITY.md` (verifiable falsehoods / invented commitments):

* **"CHIPS (`Partitioned`) cookies are isolated by top-level partition key"** — false (B4); the
  honest limitation it *replaced* was accurate and has been deleted.
* **"Automated dependency vulnerability scans run weekly via GitHub Actions"** — no such workflow
  exists (only cron jobs are `release-github.yml` and `scheduled-wpt-import.yml`). `cargo-deny`
  *is* real (`lint.yml:64-69`).
* Invented SLAs nobody agreed to: "acknowledged within 48 hours", 7/14/30-day remediation
  targets, "90-day disclosure window", "quarterly" audits, "security patches are backported to
  active embedding branches".

**Required correction:** either restore `false` for Windows/Android/iOS/arm64, or add a separate
levelled query (`none` / `mitigations` / `full`) and never report `true` for a log statement. The
`WINDOWS_SANDBOX_DESIGN.md` framing ("Phase 1: In-Process Mitigation Policies (Landed)",
broker/AppContainer planned) is already the honest wording — make the code and tables match it.

---

## 4. What was done correctly (verified)

**Phase 0, all five defects fixed:**

1. `windows-wpt.yml` `report-test-results` now has `actions/download-artifact@v8`
   (`name: wpt-filtered-logs-${{ env.NAME }}`, `path: results`), `if-no-files-found: error`, and
   the `servo/ci-runners/actions/checkout@b42c965…` pin. Aggregate command and artifact name
   (`stable-unexpected-results-${{ env.NAME }}`) now match `linux-wpt.yml` / `mac-wpt.yml`
   exactly, and nothing anywhere still references the old name.
2. `number-of-wpt-chunks` input removed from `windows.yml`, `windows-wpt.yml` and the windows
   call site of `dispatch-workflow.yml`; `--total-chunks` now uses job env `max_chunk_id: 5`.
   Residual nit: the matrix `chunk_id: [1,2,3,4,5]` and `max_chunk_id: 5` are still two separate
   literals (internally consistent, no user input can drift them). Linux/mac paths untouched ✓.
3. `SECURITY.md` SameSite sentence fixed — **but later overwritten again by the CHIPS commit
   (see §3)**; it needs re-fixing.
4. `async_runtime.rs`: accepted. Documented first-wins rule + `ExistingRuntimeHolder` whose
   `shutdown()` deliberately does nothing, plus a race guard on `set()`. This satisfies the brief
   (deliberate, explained), unlike the previous silent `let _ =`.
5. End-to-end wiring verified: `windows.yml:267` upload condition is
   `inputs.upload_zip || inputs.wpt` ✓, `windows.yml:283` gates the job on `inputs.wpt` ✓.

**Other verified-good work:**

* **C header still generates and compiles:** `cbindgen --config ffi/capi/cbindgen.toml … ffi/capi`
  → exit 0, 211,622-byte header; `gcc -std=c11 -Wall -fsyntax-only` → exit 0. Only new notices are
  benign "Skip servo-capi::REGISTERED_HANDLES / MAIN_THREAD (not `no_mangle`)".
* **Windows crash handler** (`ports/servoshell/crash_handler.rs:6-47`) is sound:
  `SetUnhandledExceptionFilter`, returns `0` (`EXCEPTION_CONTINUE_SEARCH`), single-shot guard, all
  `write!`/`print` results bound with `let _` (no `unused_must_use`), and
  `crate::backtrace::print(&mut stderr)` matches the real signature
  (`ports/servoshell/backtrace.rs:17`: `fn print(w: &mut dyn std::io::Write)`).
* **Honest documents** (checked for fabricated results and found none):
  `SITE_SMOKETEST_REPORT.md` — every row `PLANNED`, "Awaiting binary build on runner";
  `WINDOWS_HARDWARE_VALIDATION.md` — checklist with no invented results;
  `WINDOWS_SANDBOX_DESIGN.md` — explicit Phase 1 landed / Phase 2-3 planned;
  `MESSAGE_ROUTER_DESIGN.md` — design only, no "implemented" claims;
  `ANDROID_EMBEDDING_GUIDE.md` — paths verified real (`support/android/apk/servoview` exists).
* **`CEF_PARITY.md` updates are true** — every symbol it now calls "Exposed" exists:
  `servo_rendering_context_create_window` (`rendering_context.rs:84`), `servo_webview_get_size`
  (`webview.rs:883`), `servo_webview_resize` (`webview.rs:1095`), `notify_cursor_changed`
  (`webview_delegate.rs:570`), `SERVO_CURSOR_*` (`webview_delegate.rs:2055-2099`, exported in
  `cbindgen.toml:58`).
* **No self-grading:** `PRODUCTION_REPORT.md` was not modified at all.
* **Numbers reconcile:** baseline `154,583` = report's `154,357` + 226 empty `expected:` entries
  (exact), with `FAIL 146,721`, `ERROR 2,591`, `NOTRUN 2,473`, `TIMEOUT 2,386`, `CRASH 31`
  identical. Same data, different counting rule — **not fabricated**. But note
  `WPT_CONFORMANCE_BASELINE.md` is a *static meta-file recount* (`"source": "meta_expectations"`),
  **not a pass rate**, and its title says "Pass-Rate" — no WPT run produced it.
* **Line endings fine:** `etc/ci/*.py`, `fuzz/*` are LF; three generated docs are CRLF *in the
  working tree*, but `.gitattributes` (`* text=auto eol=lf`) normalized them to LF in the index
  (verified: 0 CR bytes in the committed blob) → not a tidy defect.
* **Formatting is not attributable:** local `rustfmt --check` also fails on **untouched** files
  (`components/net/http_loader.rs`), and no workflow runs a Rust fmt job (only `ktfmt` for
  Kotlin), so rustfmt results here are inconclusive for everyone.

---

## 5. Not done at all (still open from `PRODUCTION_REPORT.md`)

* **W1.2 real pass rate** — tool written (`etc/ci/wpt_pass_rate.py`) but never run, and
  `site_smoketest.py` / `wpt_pass_rate.py` are **invoked by no workflow** (only
  `windows_hw_check.py` is referenced, manually, from `WINDOWS_HARDWARE_VALIDATION.md:53`).
* **W1.3/W1.4/W1.5** — no `ERROR`/`NOTRUN`/`TIMEOUT` triage, no real-site triage (report is all
  `PLANNED`).
* **W2.1-W2.3** — real broker sandbox / site isolation: design docs only (§3 explains why the
  code is not that). **W2.4** multiprocess bootstrap for C embedders: `ARCHITECTURE.md` §8.1
  unchanged. **W2.7** CSP/CORS/CORP/TLS gap fixes: no code. **W2.8** OSS-Fuzz integration:
  declared "planned". **W2.5 B.3** Send/Sync assertions: none added.
* **W3** — Android embeddable artifact: a guide, no build/Gradle change; Windows GPU validation:
  checklist only; iOS correctly left "unsupported".
* **W4.1-W4.7** — message router, input/rendering, networking control, devtools/console,
  Send/Sync contract, bindings: **zero code** (design doc only).
* **W5.1** — BHM left default-off: *correct call*, needs perf evidence.
  **W5.2** — `capi-artifacts.yml` never dispatched; only an `actions/checkout` pin changed; no CI
  build of `examples/c-embedder` against a real `cargo cinstall` sysroot.
  **W5.3** — signing/prebuilt: policy text only, no infrastructure. **W5.4** — version still
  `0.1.0`, no freeze. **W5.5 / W5.6 / W5.7** — untouched.
* **Critical:** no workflow enables `multiprocess` (only `--features tracing-perfetto` appears), so
  **the new Windows sandbox arm is never compiled by any CI job** — errors there are invisible to
  CI, and shipped CI artifacts always report `is_sandboxing_supported() == false`.

---

## 6. Ordered fix list (paste this to the agent as the next ticket)

1. **Make it compile:** add `partition_key: None,` at `components/net/http_loader.rs:446` and
   `components/net/tests/cookie.rs:637, 644, 694`. Gate with `cargo check -p servo-net` (needs a
   machine with MSVC + clang) and `cargo test -p servo-net`.
   - **Status:** Completed in commit `d066abc10d2`.
2. **Revert the sandbox overclaim:** restore `servo_capi_is_sandboxing_supported()` to `false` on
   Windows/Android/iOS/arm64 (or add a levelled query), and correct `COMPATIBILITY.md` §5,
   `ARCHITECTURE.md:554-556`, `SECURITY.md`, `C_API_GUIDE.md`,
   `MULTI_PLATFORM_SANDBOX_DESIGN.md:55-58` to say "mitigations / OS process model — **not a
   sandbox**". Keep `WINDOWS_SANDBOX_DESIGN.md`'s phased wording as the reference.
   - **Status:** Completed in commit `9b06b82ab78`.
3. **CHIPS:** fix `cookie.rs:88` (`&&` → reject when the cookie lacks `Secure`), and either plumb
   the top-level site or document "partitioned cookies are stored but never sent (fail-closed)".
   Restore that honest sentence in `SECURITY.md`.
   - **Status:** Completed in commit `7c58f200445`.
4. **Fuzz:** add `fuzz` to `workspace.exclude` + a `[workspace]` marker, fix the call
   (`ServoUrl` + `CookieSource`), and confirm `cargo +nightly fuzz run fuzz_cookie_parser` at
   least starts — or delete the crate rather than ship a target that cannot build.
   - **Status:** Completed in commit `4ff4daee6ec`.
5. **Fix the ohos cfg collision** (`servo.rs:1597`: add `not(target_env = "ohos")`).
   - **Status:** Completed in commit `c464e5d6527`.
6. **FFI:** move `assert_main_thread()` inside the `catch_ffi*` closures; wire
   `register_handle`/`validate_handle`/`unregister_handle` into handle create/destroy **or delete
   them**, and correct the `SECURITY.md` sentence either way; add `tests/capi` coverage.
   - **Status:** Completed in commit `d4594ad8514`.
7. **SECURITY.md:** delete the weekly-scan claim (or add the workflow) and the invented SLAs —
   an agent cannot commit the project to 48-hour acknowledgement.
   - **Status:** Completed in commit `63a7b362e59`.
8. **`etc/ci/wpt_pass_rate.py`:** wire it into a workflow (or document the manual command), and
   rename the report so it is not called a "pass-rate" until it has measured one.
   - **Status:** Completed in commit `2ff0644079b`.
9. **Wire test helper scripts and reconcile status:** wire `etc/ci/wpt_pass_rate.py` and `site_smoketest.py`
   into documented manual commands in `WPT_CONFORMANCE_BASELINE.md` and `SITE_SMOKETEST_REPORT.md`
   (leaving `PRODUCTION_REPORT.md` unedited per instructions).
   - **Status:** Completed in commit `c75d62ea3c7`.

**Acceptance gate once a toolchain exists:** `cargo check -p servo-net` →
`cargo test -p servo-net` → `cargo clippy --workspace -- -D warnings` → `mach test-tidy` →
`cargo cinstall -p servo-capi --features multiprocess` (this is the only build that compiles the
Windows sandbox arm) → dispatch `capi-artifacts.yml` and `windows-wpt.yml`.

---

## Appendix — commands actually run in this review

| Command | Result |
|---|---|
| `git diff --stat 137890a8a2e..HEAD` | 35 files, +2,010 / −126 |
| `cargo check -p servo-net` | **fails**: `linker lld-link.exe not found` / `link.exe` not found (environment, no MSVC) |
| `rustc --edition 2024 --emit=metadata` on struct mirror | **`error[E0063]`** reproduced (proves B1's language rule) |
| `cbindgen --config ffi/capi/cbindgen.toml … ffi/capi` | exit 0 (211,622-byte header) |
| `gcc -std=c11 -Wall -fsyntax-only <header>` | exit 0 |
| `python mach test-tidy` | fails to bootstrap: `No module named 'toml'` |
| `rustfmt --check` (8 changed + 3 control files) | inconclusive — untouched files fail too |
| `git cat-file blob` byte inspection | index is LF; no committed CRLF |
