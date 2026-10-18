# Servo Embedding — Agent Build Plan (W2–W6)

Status: **Executable work plan** · Branch: `embedding/cef-parity` · Written: 2026-10-18

This document exists so that **one AI coding session can pick up exactly one ticket and
deliver it to a verified-green CI state**, without needing the surrounding conversation.
It is the successor to the work-list section of
[`PRODUCTION_REPORT.md`](../../PRODUCTION_REPORT.md) §4, which stays as the strategic
view; this file is the tactical one.

**Scope:** workstreams **W2 (security), W3 (platforms), W4 (embedding API), W5
(stability/distribution), W6 (rendering/media)**.
**Out of scope:** **W1 (WPT conformance)** — deliberately. It is the blocking item and it
is measured in *years of a funded team*; no agent session "completes" it. See §11.

---

## 0. How to use this document

1. **Read §1 (Ground rules) in full before touching any code.** It contains the hard
   constraints of this repository (you almost certainly cannot build it locally) and the
   non-negotiable coding rules. Skipping it is how tickets fail CI.
2. **Pick one ticket**, from the ordered waves in §2.2. Do not run two tickets in one
   session — the verification gate is per-push and interleaving work makes failures
   unattributable.
3. Each ticket is self-contained: *State → Touch points → Steps → Acceptance → Tests →
   Gotchas*. Where a ticket says "verify first", **actually verify** — several state
   descriptions in older documents on this branch are stale and are corrected here.
4. **Line numbers in this document drift.** They are anchors, not authority. Re-locate a
   symbol with `git grep` before editing; the search command is given for anchors that
   are likely to move.
5. When the ticket's *Acceptance* and *Tests* are all green and CI has passed, stop.
   Report using the template in §1.5. Do not "improve" adjacent code.

### Ticket status vocabulary

| State | Meaning |
|---|---|
| `todo` | Not started. |
| `partial` | Some plumbing exists; ticket says what remains. |
| `unproven` | Code exists but has never been executed/validated. |
| `blocked` | Needs a human, hardware, or an upstream decision first. |

---

## 1. Ground rules & operating manual (mandatory)

### 1.1 Repository facts

| Fact | Value |
|---|---|
| Repo | `E:\servo`, remote `https://github.com/peter14l/servo.git` |
| Branch | `embedding/cef-parity` (fork of `servo/servo` with embedding work stacked) |
| Branch tip at writing | `6fe2549874b` |
| Last CI-green run | `39047950141` on `6fe2549874b` (`embedding-api.yml`) |
| Escape hatch (roll back everything after it) | `git reset --hard 13cad2c3a45` |
| Primary CI gate | `.github/workflows/embedding-api.yml` |
| `gh` CLI | authenticated as `peter14l` |

Commit style is Conventional Commits with a real body explaining *why*:
`feat(capi): …`, `fix(net): …`, `test(capi): …`, `docs(embedding): …`, `ci(capi): …`.
No `Signed-off-by` trailers are used on this branch.

### 1.2 Build & verification reality — **read this twice**

**You cannot do a full local build on this machine.** Disk is the binding constraint
(`E:` ≈ 5 GB volume with < 1 GB free; `C:` ≈ 22 GB free) and `mozjs`/the full workspace
will not fit. Past sessions burned hours discovering this. Therefore:

| What | Where it runs |
|---|---|
| Full build + C API tests + header + symbols | **CI only** (`embedding-api.yml`) |
| Cheap crate checks (`servo-net`, `servo-background-hang-monitor`, `servo-shared`) | Local, with `CARGO_TARGET_DIR` on `C:` |
| C/C++ syntax of test + example files | Local, MinGW `gcc`/`g++` |
| Header regeneration | Local (`cbindgen` 0.29.4 installed) |
| Windowed/GPU/real-hardware behaviour | **Human + real GPU machine** (never CI) |

Local commands that *do* work:

```powershell
# Rust syntax/type check of a leaf crate (cheap)
$env:CARGO_TARGET_DIR = "C:\Users\LOQ\AppData\Local\Temp\opencode\target"
cargo check -p servo-net
cargo check -p servo-background-hang-monitor

# C/C++ syntax-only gates used by this branch (MinGW at C:\ProgramData\mingw64\mingw64\bin)
gcc -std=c11  -Wall -Wextra -Werror -pedantic -fsyntax-only tests/capi/c/test_api.c
gcc -std=c11  -Wall -Wextra -Werror -pedantic -fsyntax-only tests/capi/c/test_integration.c
g++ -std=c++17 -Wall -Wextra -Werror -pedantic -fsyntax-only tests/capi/cpp/test_cpp_api.cpp

# Regenerate the public header (CI re-runs cbindgen and diffs)
cbindgen --config ffi/capi/cbindgen.toml --crate servo-capi --output <out.h>

# Symbol allowlist: #[no_mangle] count must equal ffi/capi/symbols.txt
git grep -c "no_mangle" -- ffi/capi
```

Pushing is the real gate:

```powershell
git push origin embedding/cef-parity
# watch:
gh run list -R peter14l/servo -L 5
gh run watch <run-id>          # or poll the run page
# job-level log (job logs are NOT in the run log):
gh api repos/peter14l/servo/actions/jobs/<job-id>/logs
```

`embedding-api.yml` has **path filters that exclude root-level `.md` files**: a docs-only
commit does not trigger it. If your change is docs-only, say so in the report instead of
waiting for a run that will never start.

### 1.3 Session workflow (every ticket)

1. `git status` + `git log -1` — confirm clean tree and tip. **If the working tree has
   edits you did not author, stop and report them**; concurrent work has appeared in this
   repo before and was folded into a commit only after verification.
2. Re-read §1.4 coding rules.
3. Implement **one** ticket, smallest correct change.
4. Local gates that apply (§1.2) — run them, paste output.
5. `git add <specific paths>` (never `git add -A`), commit with a real message.
6. Push. Watch CI to green (or to a failure you then fix *in place* — do not paper over).
7. Report with the §1.5 template, including run id + conclusion.

### 1.4 Non-negotiable coding rules

These encode patterns already enforced by review on this branch. Violating them is the
usual cause of a rejected ticket.

1. **Panic-free FFI boundary.** No `assert!`, `unwrap`, `expect`, `panic!`, `unreachable!`
   can be reached from a C caller in `ffi/capi`. Return `*mut` `NULL` or a
   `SERVO_ERROR_*` code, or a `bool`/`SERVO_*` status. Wrap the body in `catch_unwind`
   where an unwind could otherwise cross the boundary.
2. **Versioned structs.** Every struct handed across the ABI that may grow carries
   `struct_size` (+ `_reserved`/`_validated` as the existing ones do), and Servo validates
   it on entry, rejecting short/garbage values with `SERVO_ERROR_INVALID_ARGUMENT`.
   Follow the existing pattern in `ffi/capi/webview_delegate.rs` (it uses `CFFI_ASSERT`).
3. **Append-only enums.** New enum variants are added at the end; consumers must tolerate
   unknown values. Never reorder.
4. **Ownership + threading contract per function**, using the templates in
   `docs/embedding/ABI_STABILITY.md`. If you cannot state who frees it and on which
   thread, do not ship the function.
5. **Capability queries, not assumptions.** `servo_capi_is_sandboxing_supported()`,
   `servo_capi_is_multiprocess_enabled()`, `servo_capi_platform_name()` are the model
   (`ffi/capi/lib.rs:429-467`): the predicate must mirror *reality*, not aspiration.
6. **Honesty is a correctness requirement here.** Known anti-pattern already in-tree, do
   not propagate it: `components/servo/servo.rs:1586-1600` logs
   `"Sandbox active: Operating-system isolated container runtime enforced."` and
   `"AArch64 isolated sandbox PoC active."` when **no sandbox is applied at all**. Any
   log line, doc sentence, or API that claims a capability it does not have is a bug
   (see W2.2). Prefer `warn!`/`info!` that states what actually happened.
7. **Header is generated.** Never hand-edit the cbindgen output. Add exports via
   `#[unsafe(no_mangle)]` + `cbindgen.toml` `[export]` if needed.
8. **`ffi/capi/symbols.txt` is append-only.** Adding a symbol? append it and let CI
   verify. Removing/renaming one is an ABI break → see W5.4 first.
9. **Every new C function gets a C test** in `tests/capi/c/`, and where the function is
   observable through the event loop, an integration test in `test_integration.c` that
   drives it via `servo_spin_event_loop` and asserts on a real outcome (title, cookie
   value, callback count) — not merely that the call returned.
10. **Docs are part of the ticket.** Update the relevant `docs/embedding/*.md` section in
    the same commit when behaviour changes. Do not leave a doc claiming something you did
    not do (and do not mark a doc "done" unless you verified it).

### 1.5 Report template (end of every ticket)

```markdown
## <TICKET-ID> — <title>
- commits: <short-sha> (<subject>)
- CI: run <id> on <sha> → <conclusion>; relevant jobs: <names>
- local gates run: <commands + result>
- acceptance: item-by-item ✅/❌/n/a with evidence (file:line or run id)
- state changed: from <old> to <new>
- not done / deliberately deferred: <list>
- new risks found: <list>
```

Be conservative: an item you could not execute is `not done`, not "done". Every "done" in
this project has previously been found to be either CI-proven or explicitly marked
unproven — keep that property.

---

## 2. Portfolio

### 2.1 Everything in scope

Effort: **S** days · **M** 1–3 weeks · **L** 1–3 months · **XL** quarters+.

| ID | Title | Effort | State | Depends on | Gate |
|---|---|---|---|---|---|
| **W2.1** | Windows renderer sandbox (real containment) | L–XL | `partial` (mitigation-only PoC) | — | windows.yml |
| **W2.2** | Android/iOS/arm64 sandbox honesty + real path | XL | `partial` (false log claims) | W2.1 learnings | android.yml |
| **W2.3** | Site isolation / origin-keyed processes | XL | `todo` | — | linux.yml |
| **W2.4** | Multiprocess bootstrap for C embedders | L | `partial` | — | embedding-api.yml |
| **W2.5** | FFI hardening Part B (cookies/pointers/threads) | M | `partial` | — | embedding-api.yml |
| **W2.6** | CHIPS: derive request partition key (never sent today) | M | `partial` (store+fail-closed done, key never derived) | — | embedding-api.yml |
| **W2.7** | CSP/COOP/CORP/XCTO/TLS behaviour review | L | `todo` | — | linux.yml + WPT |
| **W2.8** | Fuzzing + security regression in CI | L | `partial` | — | embedding-api.yml |
| **W2.9** | Operationalise `SECURITY.md` | S | ✅ verified (`31f9a22abdc`; docs/process — no CI gate) | — | — |
| **W3.1** | Android embeddable AAR (not just a demo app) | L | `partial` | — | android.yml |
| **W3.2** | iOS: run spike → decide (commit or declare unsupported) | S | `blocked` (needs decision) | human | — |
| **W3.3** | arm64 sandbox gap | XL | `todo` | W2.1/W2.2 | — |
| **W3.4** | Signed/notarized release pipeline | L | `todo` | W5.3 | release.yml |
| **W3.5** | Windowed/GPU validation harness + checklist | S | `partial` (harness exists, never run) | human+GPU | manual |
| **W4.2** | Clipboard + gamepad + embedder controls in C | L | `partial` | — | embedding-api.yml |
| **W4.4** | DevTools/console exposure: verify + document + test | S | ✅ verified (run `39152343582`, 3 platforms) | — | embedding-api.yml |
| **W4.5** | Manual windowed/GPU validation on real hardware | S | `blocked` (needs GPU) | W3.5 | manual |
| **W4.6** | `Send`/`Sync` handles *or* documented main-thread contract | M | `todo` | W2.5 | embedding-api.yml |
| **W4.7** | Language bindings beyond C/C++ | L | `partial` (C++ RAII exists) | W5.4 | — |
| **W4.8** | CEF P2 leftovers: drag/drop, audio, frame rate, task post, user-gesture JS | L | `todo` | — | embedding-api.yml |
| **W5.1** | Hang monitor: on by default + kill hung scripts | M | `partial` (impl, pref off) | — | all platforms |
| **W5.2** | Prove `capi-artifacts.yml` (prebuilt C API artifacts) | M | ✅ verified (run `39145918170`, 3 platforms) | — | capi-artifacts.yml |
| **W5.3** | Prebuilt signed binaries (5 targets) + CMake/pkg-config | L | `todo` | W5.2 | release workflows |
| **W5.4** | C API 0.1.0 → 1.0 ABI commitment | M | `partial` | W5.2 | embedding-api.yml |
| **W5.5** | Remaining reachable-panic review (tracked list) | M | `partial` | W2.5 | embedding-api.yml |
| **W5.6** | Observability: size/startup/throughput/crash in CI | M | `todo` | — | new job |
| **W5.7** | Long-uptime / repeated load-unload leak testing | L | `todo` | — | embedding-api.yml |
| **W6.1** | GPU/WebGL/WebGPU validation + honest status doc | L | `partial` (claims untested) | W4.5 | manual |
| **W6.2** | Video/audio playback readiness (GStreamer packaging) | L | `partial` | W5.3 | — |
| **W6.3** | DRM/EME: define CDM adapter interface, declare non-support | L | `todo` (design only) | — | — |
| **W6.4** | Accessibility exposure to C embedders | M | `unproven` | — | embedding-api.yml |
| **W6.5** | Printing / `servo_webview_print_to_pdf` | L | `todo` (design only) | — | — |
| **W6.6** | Extensions / enterprise policy | XL | `deferred` | decision | — |

### 2.2 Recommended execution order (waves)

Each wave is independently shippable; do not start a wave before the previous one is green.

| Wave | Tickets | Why this order |
|---|---|---|
| **A — Prove what exists** | W5.2, W4.4, W2.9 | Turns `unproven` into evidence cheaply; nothing else is trustworthy until CI-covered workflows have actually run. |
| **B — Truthfulness & quick wins** | W5.1, W2.2a (log honesty), W4.6, W5.5 | Small, high-value, mostly docs/pref/default changes with clear tests. |
| **C — Embedding API gaps** | W4.2, W4.8, W5.4, W5.7 | The surface embedders actually feel. |
| **D — Distribution** | W5.3, W3.4, W3.1 | Depends on A/C being stable. |
| **E — Security depth** | W2.5, W2.6, W2.4, W2.8, W2.7 | Each is a real project; sequence after the cheap wins. |
| **F — Platform & media** | W2.1, W2.2b, W2.3, W3.3, W3.5/W4.5, W6.x | Needs hardware, human decisions, or deep OS work. |

### 2.3 Explicitly not in this document

- **W1 conformance** (WPT). See §11.
- Anything requiring a full local build. If your ticket needs one, restructure it so the
  proof runs in CI.
- Upstreaming to `servo/servo` — desirable and ongoing, but a separate track.

## 3. W2 — Security architecture

### 3.0 W2 current-state brief (read before any W2 ticket)

Facts verified at `6fe2549874b`:

- `create_sandbox()` has **five** `cfg`-selected definitions in
  `components/servo/servo.rs`:
  | Line | Predicate | What it actually does |
  |---|---|---|
  | `1571` | `multiprocess`, not windows/ios/android/arm/aarch64/riscv/ohos | `gaol` `ChildSandbox::new(content_process_sandbox_profile()).activate()` — **real** (Linux/macOS x86_64 only) |
  | `1578` | `multiprocess` + windows | `windows_sandbox::apply_windows_sandbox()` (`mod` at `1458`, fn at `1496`) — **four `SetProcessMitigationPolicy` calls (DEP, ASLR, strict handle check, extension point disable). Not a sandbox: no broker, no restricted token, no AppContainer, no job object, no IPC ACL.** |
  | `1586` | `multiprocess` + android/ios | **no-op that logs** `"Sandbox active: Operating-system isolated container runtime enforced."` |
  | `1598` | `multiprocess` + arm/aarch64 (not android/ios/ohos) | **no-op that logs** `"AArch64 isolated sandbox PoC active."` |
  | `1613` | `not(multiprocess)` or riscv/ohos | `panic!("Sandboxing is not supported on single-process, RISC-V, or OpenHarmony targets.")` |
- Activation call site: `run_content_process()` — `cfg(feature = "multiprocess")` at
  `1388`, `if opts::get().sandbox { create_sandbox(); }` at `1403-1404`.
  `Opts.sandbox` defaults to `false` (`components/config/opts.rs:55`, `:258`).
- **The capability query is honest and is the pattern to preserve:**
  `servo_capi_is_sandboxing_supported()` (`ffi/capi/lib.rs:455-467`) returns `false` on
  windows/ios/android/arm/aarch64/riscv/ohos, and its doc comment at `449-451` states
  that Windows mitigations are not an OS sandbox. `ffi/capi/multiprocess.rs:95` refuses
  to proceed when the query is false.
- Therefore: **the C API does not lie about the sandbox. The log lines at `servo.rs:1586`
  and `servo.rs:1598` do.** That asymmetry is W2.2a.

Search command for these anchors (they will move):
`git grep -n "fn create_sandbox\|apply_windows_sandbox\|Sandbox active" -- components/servo/servo.rs`

---

#### W2.1 — Windows renderer sandbox (real containment)

**State:** `partial` · **Effort:** L–XL · **Gate:** `windows.yml` (+ a new containment test)

**Goal.** Replace the mitigation-policy PoC with actual OS containment for the content
process, then let `servo_capi_is_sandboxing_supported()` start returning `true` on
Windows — *only* when containment is real.

**Touch points**
- `components/servo/servo.rs:1457-1580` (`mod windows_sandbox`, `create_sandbox`)
- `components/servo/servo.rs:1388-1404` (activation in `run_content_process`)
- `components/constellation/sandboxing.rs` (seccomp/profile equivalent on Unix)
- `ffi/capi/lib.rs:455-467` (capability predicate — **last thing you change**)
- `docs/embedding/COMPATIBILITY.md` §5 (sandbox matrix), `docs/embedding/SECURITY_AUDIT.md`
- New: `docs/embedding/WINDOWS_SANDBOX_DESIGN.md`

**Steps**
1. **Design first, code second.** Write `WINDOWS_SANDBOX_DESIGN.md`: threat model (what
   the content process must be unable to do), chosen model — Chromium-style
   **restricted token + job object + brokered IPC** is the sane default; AppContainer is
   stronger but changes package assumptions — and what is explicitly *out* of scope.
   Because this needs OS expertise and cannot be validated headlessly, the design doc is
   a legitimate ticket outcome on its own; mark the ticket `partial` if only the design
   lands.
2. Keep the existing mitigation policies (they are cheap and harmless) but move them into
   the new model rather than deleting them.
3. Implement broker-side handle restriction: the content process gets a restricted token
   with no `SeDebugPrivilege`, a job object limiting breakaway, and a handle table that
   excludes the embedder's secrets. Servo's content-process IPC must be brokered through
   whatever the embedder/process boundary already provides.
4. Add a **containment test** that fails if containment is absent: e.g. spawn the content
   process and assert it cannot open a file outside an allowlisted temp dir / cannot see
   the embedder's environment. A test that merely checks "the function returned true" is
   not acceptable.
5. Only then flip the Windows branch of `servo_capi_is_sandboxing_supported()` and update
   `COMPATIBILITY.md`.

**Acceptance**
- [ ] A Windows content process runs under a restricted token + job object (proved by a
      test, not by a log line).
- [ ] `servo_capi_is_sandboxing_supported()` still returns `false` on Windows until that
      test exists and passes in `windows.yml`.
- [ ] `WINDOWS_SANDBOX_DESIGN.md` states model, threats covered, threats **not** covered.
- [ ] Single-process builds (`not(feature = "multiprocess")`) keep working; the `panic!`
      at `servo.rs:1613` is unreachable from any embedder-reachable path.

**Tests**
- New Windows-only test in `windows.yml` (unit or smoketest extension) asserting the
  containment property.
- `mach smoketest` still passes with `--sandbox` (it starts a real page on Windows CI).

**Gotchas**
- `SetProcessMitigationPolicy` returning 0 is currently only `warn!`ed
  (`servo.rs:1510`, `1523`, `1538`, `1552`) — **fail-open**. Under a real sandbox,
  failure to apply must be fatal, or the capability query must go false.
- Do not touch the capability predicate "to make the docs match". The docs match reality;
  change reality first.

---

#### W2.2 — Android / iOS / arm64 sandbox: honesty first, then reality

**State:** `partial` · **Effort:** XL overall (**W2.2a is S**) · **Gate:** `android.yml` / `embedding-api.yml`

Split into two tickets so the cheap, mandatory half is not held hostage by the expensive half.

##### W2.2a — Remove false "sandbox active" claims (S, do first)

**Touch points.** `components/servo/servo.rs:1586-1588`, `1598-1600`.

**Steps.** Replace both `log::info!` no-ops with an accurate message, e.g.
`warn!("No OS sandbox is enforced on this target; running the content process unsandboxed.")`,
and add a `TODO` referencing `WINDOWS_SANDBOX_DESIGN.md`/the platform ticket. Grep the
tree for other claims: `git grep -n "Sandbox active\|sandbox enforced\|sandboxing is" --
components ffi docs`.

**Acceptance**
- [ ] No code path prints a message asserting sandboxing that does not occur.
- [ ] `docs/embedding/COMPATIBILITY.md`, `C_API_GUIDE.md`, and `MULTI_PLATFORM_SANDBOX_DESIGN.md`
      agree with the capability query (all say: no sandbox on windows/ios/android/arm*/riscv/ohos).
- [ ] CI green; no behaviour change.

##### W2.2b — Real per-platform sandbox paths (XL)

**Goal.** Give Android and Linux-arm64 an actual containment story; iOS is out of scope
until W3.2 says otherwise.

**Touch points.** `components/servo/servo.rs` (the `1586`/`1598` variants),
`components/constellation/sandboxing.rs`, `support/android/apk/jni`,
`docs/embedding/MULTI_PLATFORM_SANDBOX_DESIGN.md`.

**Notes.** On Android the content process, when spawned as a separate process under the
app UID, inherits the app sandbox — but Servo currently runs it *in-process* with the
app on some configurations; verify first (`git grep -n "target_os = \"android\"" --
components/servo components/constellation`) and document what is true. Linux-arm64 needs
`gaol` to actually support aarch64 — check upstream `gaol` before writing any profile.

**Acceptance**
- [ ] Each platform either enforces a named mechanism (with a test) or reports `false`
      from the capability query and says so in `COMPATIBILITY.md`.
- [ ] No platform reports `true` without a containment test.

---

#### W2.3 — Site isolation / origin-keyed processes

**State:** `todo` · **Effort:** XL · **Gate:** `linux.yml`

**Goal.** Understand, document, then incrementally move from "sandboxed origins handled"
to origin-keyed process reuse.

**Touch points.** `components/constellation/constellation.rs` (event-loop / pipeline
creation), `components/constellation/sandboxing.rs`, `docs/embedding/ARCHITECTURE.md`.

**Steps**
1. Write the *current* model into `ARCHITECTURE.md`: how event loops are allocated per
   pipeline vs. reused, what "sandboxed origins" already does, and where
   cross-origin-isolation decisions happen. This alone is a valid first ticket — today
   nobody has written it down on this branch.
2. Add a preference-gated origin-keyed reuse policy (one event loop per site, reuse only
   for same-site) and measure process count on a fixed test page.
3. Only then discuss COOP/COEP-derived isolation (ties into W2.7).

**Acceptance.** Documented current model; a test that asserts two cross-site iframes land
in different event loops when the pref is on. **Do not claim Blink-class isolation.**

**Gotchas.** Process-count regressions break CI resource budgets — measure before/after
in the same run.

---

#### W2.4 — Multiprocess bootstrap for C embedders

**State:** `partial` · **Effort:** L · **Gate:** `embedding-api.yml`

**Goal.** An embedder that links `servo-capi` can get content-process isolation without
copying `ports/servoshell`'s bootstrap.

**Touch points**
- `ffi/capi/multiprocess.rs` (2 exports; `95` validates against the capability query)
- `ffi/capi/lib.rs:429-437` (`servo_capi_is_multiprocess_enabled`)
- `components/servo/servo.rs:1388-1404` (`run_content_process`)
- `docs/embedding/ARCHITECTURE.md` §8.1 (prescribed design)

**Steps**
1. Verify what `servo-capi` currently expects of its embedder: does the child process have
   to re-enter Servo via a documented entry point? Read `ARCHITECTURE.md` §8.1 and say so
   in `C_API_GUIDE.md` — an accurate "here is how you launch the child" section is the
   minimum deliverable.
2. If the child entry needs a C-callable function, export one
   (`servo_run_content_process(...)`) wired to `run_content_process`, guarded by the
   `multiprocess` feature and the capability query.
3. C test that exercises the single-process path still works, plus a documented snippet
   for the multiprocess path (CI cannot spawn the real child in the capi harness — say
   so rather than faking it).

**Acceptance**
- [ ] `C_API_GUIDE.md` documents the exact embedder-side steps for multiprocess.
- [ ] Any new export appended to `symbols.txt`; header regenerates; tests pass.
- [ ] The interim reality is stated plainly: on Windows `multiprocess` gives **crash
      isolation without OS containment** (`opts.sandbox` + mitigation PoC only).

---

#### W2.5 — FFI hardening: Part B (debug cookies, temporary pointers, thread contracts)

**State:** `partial` · **Effort:** M · **Gate:** `embedding-api.yml`

**Goal.** Finish `docs/embedding/HARDENING_PLAN.md` **Part B**: B.1 debug handle
validation (`164`), B.2 temporary pointers & thread affinity (`198`), B.3 `Send`/`Sync`
assertions (`212`).

**Touch points**
- `ffi/capi/servo.rs`, `ffi/capi/lib.rs` — debug handle validation exists at ~2 sites in
  `lib.rs` and ~3 in `servo.rs`
- `ffi/capi/cert_error.rs` — hand-rolled magic-cookie check worth folding into the shared
  helper
- `ffi/capi/webview_delegate.rs` — `struct_size` validation + `CFFI_ASSERT`
- `components/servo/{servo,webview}.rs` — thread-affinity assertions

**Steps**
1. Inventory: `git grep -n "struct_size" -- ffi/capi` vs. every `#[repr(C)]` struct
   exposed by the header. Produce the list of structs lacking validation and put it in the
   ticket report.
2. Extract one shared helper for magic/state cookie + `struct_size` validation; use it at
   every handle entry point instead of ad-hoc checks.
3. Temporary-pointer contract: functions taking a borrowed `*const c_char`/buffer must
   document that the pointer is only used synchronously, and must copy before any
   deferred use. Find deferred uses:
   `git grep -n "from_raw\|to_string_lossy" -- ffi/capi` and audit each for escaping the
   call.
4. Thread affinity: assert, on entry to every main-thread-only function, that we are on
   the thread Servo ran on (the mechanism already exists — extend it).

**Acceptance**
- [ ] Every `#[repr(C)]` struct crossing the ABI has `struct_size` validation or a
      written reason it cannot grow.
- [ ] A negative C test passes: pass a garbage handle / zero `struct_size` / handle from a
      freed object → error code, **process survives**.
- [ ] `HARDENING_PLAN.md` Part B checkboxes updated to reflect reality.

**Gotchas.** Debug-only cookie checks that vanish in release builds are a footgun; state
in the doc which checks are compiled out in release.

---

#### W2.6 — CHIPS: derive the request partition key (currently never sent)

**State:** `partial` — parse ✅ store ✅ fail-closed send ✅ **key derivation ✗** · **Effort:** M · **Gate:** `embedding-api.yml`

**Verified current behaviour at `6fe2549874b`** (this corrects the older "stored but never
sent — needs partition-key storage" description; the storage part is already done):

| Stage | Status | Evidence |
|---|---|---|
| Parse `Partitioned` attribute | ✅ | `components/net/cookie.rs:83-85` |
| Enforce `Secure` + secure scheme on set (fail-closed) | ✅ | `cookie.rs:87-88` |
| Store the key on the cookie | ✅ | `cookie.rs:45` (`partition_key: Option<String>`), dedup includes it at `cookie_storage.rs:83` |
| Send-side match (fail-closed) | ✅ | `cookie.rs:468-477` — no key or a different key ⇒ `return false` |
| **Derive a partition key for an actual request** | ❌ | `http_loader.rs:452` hardcodes `partition_key: None`; `SameSiteContext::SAME_SITE` is `None` (`cookie.rs:71`) |

**Net effect: partitioned cookies are stored and can never be sent.** The fail-closed
rule is intact and correct; the feature simply does nothing end-to-end.

**Touch points**
- `components/net/http_loader.rs:435-454` (`same_site_context_for_request`) — the one
  place the key must become `Some`
- `components/net/cookie.rs:44-73` (structs), `:468-477` (send match), `:83-117` (set path)
- `components/constellation/constellation.rs:2658` — **existing precedent** for deriving a
  partition key for `localStorage`; reuse its notion of the top-level/site key
- `components/net/tests/cookie.rs:634-700` (existing `SameSiteContext` test constructors)

**Steps**
1. Determine what "top-level site" data the request path already has. CHIPS keys on the
   *top-level* site, not merely the initiator origin (`request.origin` at
   `http_loader.rs:439` is not the same thing). If the top-level site is not available on
   the request, it must be carried from the constellation the same way the localStorage
   key is — record which option you take and why.
2. Populate `SameSiteContext.partition_key` in `same_site_context_for_request`.
   **Keep the fail-closed default:** if the key cannot be determined, `None` must remain
   `None` (cookies then stay unsent — correct, not a bug).
3. Tests in `components/net/tests/cookie.rs`:
   - same-partition request ⇒ partitioned cookie **is** sent;
   - different-partition or missing key ⇒ **not** sent (already implied by
     `:468-477`, now assert it end-to-end through `http_loader`);
   - unpartitioned cookies unaffected (the `cookie_http_state` suites must stay green).
4. Add the missing fuzz target `fuzz_cookie_partitioning` — note `fuzz/fuzz_targets/`
   currently holds **only** `fuzz_cookie_parser.rs`.
5. Update the doc(s) that describe CHIPS as storage-only, and the `SECURITY_AUDIT.md`
   entry.

**Acceptance**
- [ ] A round-trip test proves a `Partitioned; Secure` cookie is actually transmitted on a
      matching-partition request, and is withheld otherwise.
- [ ] Unpartitioned and `SameSite` behaviour unchanged (existing suites green).
- [ ] Derivation failure degrades to "not sent", never to "sent".

**Gotchas**
- **Adjacent finding, fix while here:** `cookie_storage.rs:255` logs
  `info!(" === COOKIES SENT: {}", result)` — full cookie values at `info` level on every
  request. That is credential leakage into logs. Demote/remove it (keep a redacted
  count/name-only trace at most). This is a small, independently committable fix.
- Do not refactor `CookieStorage` while threading the key through; keep the diff to the
  context construction + tests.

---

#### W2.7 — Content-security review: CSP / COOP / CORP / XCTO / TLS

**State:** `todo` · **Effort:** L · **Gate:** `linux.yml` + WPT subsets

**Goal.** Close `docs/embedding/SECURITY_AUDIT.md` §3 and §5 with evidence.

**Steps.** Work from the audit's own findings list; for each: confirm the current
behaviour in code, run/locate the relevant WPT expectations (`tests/wpt/meta` for
`content-security-policy`, `coep`, `coop`, `corp`, `xcto`), fix what is wrong, and
record pass/fail counts. Prefer fixing engine behaviour over rewriting expectations —
**rewriting an expectation to green is not a fix** (that rule applies to W1 and here).

**Acceptance.** Audit §3/§5 items each marked *verified/changed/deferred* with file:line
or WPT expectation paths; no item silently closed.

---

#### W2.8 — Fuzzing + security regression in CI

**State:** `partial` · **Effort:** L · **Gate:** `embedding-api.yml` (+ a scheduled job)

**Goal.** Make the fuzzing story real instead of aspirational
(`SECURITY_AUDIT.md` §8, `HARDENING_PLAN.md` §A.3).

**Touch points.** `fuzz/` (**separate workspace — NOT built by `embedding-api.yml`**),
`docs/embedding/FUZZING_AND_CRASH_TELEMETRY.md`.

**Steps**
1. Add a CI step that at least **compiles** the fuzz crate:
   `cargo +nightly check --manifest-path fuzz/Cargo.toml` — this is the single cheapest
   fix and it converts B10 from "source-verified only" to build-verified.
2. Add the missing targets: `fuzz_cookie_partitioning` (see W2.6) and
   `fuzz_cookie_domain_joining`. `fuzz/fuzz_targets/` currently contains **only**
   `fuzz_cookie_parser.rs`.
3. Add a scheduled (weekly) job that runs each target for a fixed time budget against a
   checked-in corpus, and files an artifact (not an issue) on a crash.
4. Wire crash artefacts to the telemetry path described in
   `FUZZING_AND_CRASH_TELEMETRY.md`.

**Acceptance**
- [ ] `embedding-api.yml` fails if `fuzz/` stops compiling.
- [ ] ≥2 new parser/network targets present and exercised by the scheduled job.
- [ ] `FUZZING_AND_CRASH_TELEMETRY.md` describes the actual, not the planned, pipeline.

---

#### W2.9 — Operationalise `SECURITY.md`

**State:** ✅ verified (`31f9a22abdc` — docs/process, no CI gate) · **Effort:** S · **Gate:** none (docs/process)

**Goal.** Make the vulnerability policy operational: intake address, triage SLA, advisory
publication process, dependency cadence via `deny.toml` (verify it exists —
`git grep -n "deny.toml" -- .github Cargo.toml`).

**Acceptance.** `SECURITY.md` contains all four; any CI change (e.g. a `cargo deny`
scheduled job) actually runs at least once before the ticket is called done.

---

## 4. W3 — Platform coverage

### 4.0 Platform state brief (verified)

| Platform | Build/CI | Sandbox | Embedding surface | Real gap |
|---|---|---|---|---|
| Linux x86_64 | `linux.yml` | **yes** (`gaol`) | full | none structurally |
| macOS x86_64/arm64 | `mac.yml`, `mac-arm64.yml` | x86_64 only | full | arm64 sandbox (W3.3); notarization (W3.4) |
| Windows x86_64 | `windows.yml` (build, smoketest, test-unit, package, `capi`) | mitigation PoC only (W2.1) | full + `windows-wpt.yml` **never run** | hardware validation (W3.5/W4.5) |
| Android | `android.yml`; app + library modules under `support/android/apk/` | none | demo app only | **AAR** (W3.1) |
| iOS | **nothing** — no target, no port, no CI | excluded by `cfg` | none | **decision** (W3.2) |

Android tree: `support/android/apk/{servoapp, servoview, servoview-local, jni}`;
`servoview` is already an Android **library** module (namespace `org.servo.servoview`) —
so W3.1 is largely *packaging + CI + API documentation*, not a rewrite. Reference:
`docs/embedding/ANDROID_EMBEDDING_GUIDE.md`.

---

#### W3.1 — Android embeddable AAR

**State:** `partial` · **Effort:** L · **Gate:** `android.yml`

**Goal.** Ship `servoview` as a versioned **AAR artifact** that an app can depend on,
instead of only building a demo `servoapp`.

**Steps**
1. Verify the current module graph and what `servoview-local` is for
   (`git grep -rn "servoview" support/android --include=*.gradle*`).
2. Add a `android.yml` job (or extend the existing one) that produces
   `servoview-<version>.aar` and publishes it as a workflow artifact.
3. Make the AAR's API surface explicit and documented: which methods map to
   `servo-capi`/the Rust embedder API, threading contract, lifecycle (who owns
   start/shutdown).
4. Reconcile `ANDROID_EMBEDDING_GUIDE.md` with what actually builds; mark anything
   planned as planned.

**Acceptance.** AAR artifact produced by CI; a documented minimal integration snippet;
guide matches reality (no aspirational steps); `android_smoketest` still passes.

**Gotchas.** The Android JNI glue lives in `support/android/apk/jni/{Android.mk,Application.mk}`
— changing library boundaries there breaks the app build; keep the app building in the
same job.

---

#### W3.2 — iOS: run the spike, then decide (do not leave it ambiguous)

**State:** `blocked` (needs a human decision) · **Effort:** S · **Gate:** none (docs + process)

**Goal.** Replace ambiguity with a decision. `docs/embedding/IOS_PORT_SPIKE.md` already
contains 6 time-boxed experiments and **recommends declaring iOS unsupported until the
spike passes**. The current worst state is silent ambiguity.

**Steps**
1. Re-read the spike's §3/§4 and re-verify its claims against the tree
   (`git grep -rn "target_os = \"ios\"" -- components ffi ports`).
2. Produce the decision record: either *commit* (a dedicated iOS workstream: UIKit
   embedding, `aarch64-apple-ios` target, entitlements/JIT policy, CI) or *decline*.
3. **Write the outcome into every place that currently implies support:**
   `README.md`, `docs/embedding/COMPATIBILITY.md` (§5 sandbox matrix already says no —
   verify the platform matrix), `docs/embedding/C_API_GUIDE.md`,
   `docs/embedding/CEF_PARITY.md`, and this plan's §4.0 table.

**Acceptance.** One unambiguous statement in ≥4 documents; if "decline", no document
lists iOS as supported anywhere
(`git grep -in "ios" docs/embedding README.md | grep -vi "unsupported\|not supported\|no port"`).

---

#### W3.3 — arm64 sandbox gap (Linux aarch64 + Apple Silicon)

**State:** `todo` · **Effort:** XL · **Gate:** — · **Depends:** W2.1/W2.2b

**Goal.** `servo.rs:1598`'s arm/aarch64 variant currently does nothing but log. Either
make `gaol`-style containment work on aarch64, or keep reporting `false` and document
it as an accepted limitation with a mitigation list (memory-safe core, mitigations PoC
where applicable).

**First step (cheap, required):** research whether upstream `gaol` supports `aarch64`
linux/macos at all; if not, record that as the blocker in `WINDOWS_SANDBOX_DESIGN.md`
sibling notes rather than writing a fake profile.

**Acceptance.** Same honesty rule as W2.2b: enforced-with-test **or** reported-`false`
everywhere including docs.

---

#### W3.4 — Signed / notarized release pipeline

**State:** `todo` · **Effort:** L · **Gate:** `release.yml`, `release-github.yml` · **Depends:** W5.3

**Steps**
1. Read `.github/workflows/release.yml` and `release-github.yml`; report what artifacts
   they produce today, and **explicitly whether any signing/notarization step exists or
   is a stub** (`git grep -in "notarytool\|codesign\|signtool\|authenticode" .github`).
2. macOS: notarization + hardened runtime + stapling. Windows: Authenticode signing of
   the installer produced by `mach package`.
3. Emit checksums (SHA-256) for every published artifact; publish alongside the release.
4. `docs/embedding/ABI_STABILITY.md`'s signing/support-window claims must then be true.

**Acceptance.** A release workflow run where every artifact is signed and checksummed;
docs match the workflow. If secrets are unavailable, the ticket ends as `blocked` with a
precise list of required repo secrets — **not** as "done".

---

#### W3.5 — Windowed/GPU validation harness & checklist (prepares W4.5/W6.1)

**State:** `partial` (harness exists, never executed) · **Effort:** S · **Gate:** manual

**Goal.** CI runners are headless; windowed contexts are asserted to return `NULL` there.
Turn `docs/embedding/WINDOWS_HARDWARE_VALIDATION.md` into a checklist a human can run in
one sitting, with expected outputs recorded so W4.5 results are comparable.

**Steps.** Read the existing harness; enumerate the exact commands; write expected output
per scenario (windowed context creation, offscreen present, WebGL 1/2 render, cursor
change, HiDPI scale, `--sandbox` on Windows).

**Acceptance.** Checklist exists with expected results; marked `blocked` on a GPU machine
until someone runs it.

---

## 5. W4 — Embedding API parity

### 5.0 W4 current-state brief (verified at `6fe2549874b`)

**Already completed, no ticket needed:** **W4.1** native↔JS message router
(`ffi/capi/message_router.rs`, 6 exports; `be9606d2fca` + R2 fixes) and **W4.3**
networking/content control (`site_data.rs`, streaming interception in
`protocol_handler.rs`/`webview_delegate.rs`, custom schemes, cookies — CI run
`39047950141`). That is why this section starts at W4.2.

- `ffi/capi/` = 26 files; **231** `#[unsafe(no_mangle)]` items; **229** entries in
  `ffi/capi/symbols.txt` (the 2 extras are statics: `SERVO_RENDERING_CONTEXT_SOFTWARE`
  `rendering_context.rs:31`, `SERVO_RENDERING_CONTEXT_WINDOW` `:34`).
- Export counts by module: `webview.rs` 63 · `value.rs` 27 · `options.rs` 22 ·
  `protocol_handler.rs` 17 · `preferences.rs` 13 · `lib.rs` 12 ·
  `user_content_manager.rs` 12 · `download.rs` 8 · `request_context.rs` 8 ·
  `site_data.rs` 8 · `find.rs` 7 · `servo.rs` 7 · `cookie_manager.rs` 6 ·
  `message_router.rs` 6 · `rendering_context.rs` 6 · `network_manager.rs` 4 ·
  `javascript.rs` 2 · `multiprocess.rs` 2 · `cert_error.rs` 1.
- **Already wired (do not re-do):** navigation/history, focus/IME, zoom, find, JS eval +
  `ServoValue`, custom schemes, streaming resource interception, cookies, site data,
  HTTP cache, user scripts/stylesheets, message router, downloads, capability queries,
  `struct_size` versioning, cpp_compat header + C++ test, console/cursor/status/load
  delegates (`notify_cursor_changed` field `webview_delegate.rs:572`, impl `:1188`;
  `notify_history_changed` field `:884`, impl `:1877`; `show_embedder_control` impl
  `:1247`).
- **Missing (C side):** clipboard, gamepad — grep `git grep -in "clipboard\|gamepad" --
  ffi/capi` returns nothing, while the Rust API exists:
  `components/servo/clipboard_delegate.rs`, `components/servo/gamepad_delegate.rs`,
  `components/shared/embedder/embedder_controls.rs:35` (`EmbedderControlRequest`) /
  `:173` (`EmbedderControlResponse`).

---

#### W4.2 — Clipboard, gamepad, and embedder controls in C

**State:** `partial` · **Effort:** L · **Gate:** `embedding-api.yml`

**Goal.** Wrap the existing Rust delegates so a C host can implement copy/paste, gamepad
haptics, and the form-control dialogs (select/color/file/context menu/simple dialog).

**Touch points**
- Rust: `components/servo/clipboard_delegate.rs`, `components/servo/gamepad_delegate.rs`,
  `components/shared/embedder/embedder_controls.rs` (`EmbedderControlRequest`/`Response`),
  `components/servo/webview.rs` (delegate wiring), `ports/servoshell/desktop/gamepad.rs`
  and `desktop/dialog.rs` (**the behavioural reference** — this is how a real host does it)
- C: `ffi/capi/webview_delegate.rs` (extend the existing `show_embedder_control` at
  `:1247` with respond functions), new `ffi/capi/clipboard.rs` + `ffi/capi/gamepad.rs`
- Tests: `components/servo/tests/context_menu.rs` (Rust-side precedent),
  `tests/capi/c/test_integration.c`

**Steps**
1. Map `EmbedderControlRequest` → C tagged union with `struct_size`; each request gets a
   respond function that resolves it (mirror the drop-default responder pattern already
   used elsewhere in `webview_delegate.rs`).
2. Clipboard: expose the `StringRequest` responder pair
   (`servo_webview_clipboard_request_success` / `_failure`) as `ROADMAP.md` Phase 2
   prescribes; deliver callbacks during `servo_spin_event_loop`.
3. Gamepad: `GamepadDelegate` C struct + haptic-effect success/failure pair.
4. Append new exports to `symbols.txt`; regenerate header; add C tests per §1.4 rule 9 —
   at minimum a round-trip that asserts the callback fired **and** the host response
   reached the engine.
5. Update `C_API_GUIDE.md` and `ROADMAP.md` Phase 2 checkboxes.

**Acceptance**
- [ ] A C test triggers a select/file/context-menu control, responds, and observes the
      outcome through the event loop.
- [ ] Clipboard round-trip and gamepad haptic request both round-trip (or are documented
      as platform-unavailable with the capability/documented-behaviour rule).
- [ ] `symbols.txt` count matches `#[no_mangle]` count; all three CI jobs green.

**Gotchas.** `ROADMAP.md` Phase 2 lists `ffi/capi/input.rs` as *planned* but input events
already ship from `webview.rs` — **do not create a duplicate module**; put new exports in
dedicated modules only if they are not webview-scoped.

---

#### W4.4 — DevTools / console exposure: verify, document, test (re-scoped)

**State:** ✅ verified — run `39152343582` (all three platforms observed the callback) · **Effort:** S · **Gate:** `embedding-api.yml`

**Why re-scoped.** The callbacks already exist (`servo_builder_set_delegate` with
`notify_devtools_server_started` + `request_devtools_connection`, plus
`show_console_message`). What does not exist is evidence they work.

**Steps**
1. Exercise them: enable the DevTools server with
   `servo_preferences_set_devtools_server` (`servo_options_set_debug_option` only controls
   diagnostics logging, it does not touch the DevTools server), start the event loop, assert
   `notify_devtools_server_started` fires (integration test in
   `tests/capi/c/test_integration.c`).
2. Document the authorization flow (`request_devtools_connection` accept/deny) in
   `C_API_GUIDE.md`, including the security warning (authorizing a DevTools connection is
   a remote-code-execution grant — say that plainly).
3. Document console-callback severity/filtering.

**Acceptance.** An integration test that observes the devtools-started callback; docs
describe the flow and the risk. If the server cannot start in the CI harness, state that
and assert the *callback plumbing* at minimum.

---

#### W4.5 — Manual windowed/GPU validation on real hardware

**State:** `blocked` (needs a GPU machine + human) · **Effort:** S · **Gate:** manual

**Goal.** Prove the windowed present path, WebGL, and HiDPI on Windows and macOS
hardware. Nothing in CI can do this (`windows.yml` smoketest runs headless; windowed
context creation returns `NULL` there by design).

**Steps.** Depends on W3.5's checklist. Run it; attach outputs to the ticket report;
record results in `docs/embedding/WINDOWS_HARDWARE_VALIDATION.md` (or the macOS
equivalent) with date + machine + GPU.

**Acceptance.** Recorded, reproducible results — including failures. A failure recorded
honestly is a successful ticket; a claim without output is not.

---

#### W4.6 — Threading contract: `Send`/`Sync` handles *or* documented main-thread-only

**State:** `todo` · **Effort:** M · **Gate:** `embedding-api.yml` · **Depends:** W2.5

**Goal.** End the ambiguity about which C calls may be made from which thread. Per
`ROADMAP.md` Phase 5: evaluate first, **document rather than over-promise**.

**Steps**
1. Audit: `git grep -n "Thread safety" -- ffi/capi` — every exported function should
   already carry a contract block; find the ones that do not
   (`#[unsafe(no_mangle)]` count vs. `Thread safety` occurrences).
2. Decision record: `Servo`/`WebView` are `Rc`-based and thread-affine. Making them `Send`
   is an engine change; the acceptable outcomes are (a) a genuinely thread-safe handle, or
   (b) a **documented, asserted main-thread-only contract**. Choose (b) unless you can
   prove (a) in CI.
3. Enforce it: entry-point assertions (W2.5 step 4) so violating the contract returns an
   error instead of UB.
4. State the contract in `ABI_STABILITY.md` and `C_API_GUIDE.md` once, and reference it
   from the per-function blocks instead of repeating prose.

**Acceptance.** Every exported function has a threading contract; a C test that calls a
main-thread-only function off-thread gets an error (or is explicitly documented as
caller-error-only, with reasoning). No `Send` claim without a test that exercises it from
two threads.

---

#### W4.7 — Language bindings beyond C/C++

**State:** `partial` (C++ RAII covered by `tests/capi/cpp/test_cpp_api.cpp`) · **Effort:** L · **Depends:** W5.4

**Goal.** Rust wrapper is in-tree value; Python etc. belong out-of-tree.

**Steps**
1. Ensure the header is genuinely consumable from C++17 (already asserted by the C++ test
   — verify it runs on all three platforms in `embedding-api.yml`).
2. Add a documented `pkg-config`/CMake package consumer test (this overlaps W5.2/W5.3 —
   coordinate rather than duplicate).
3. For Python: write `docs/embedding/BINDINGS.md` describing the ABI rules a binding must
   honour (ownership, callbacks, threading) and explicitly keep bindings out-of-tree.

**Acceptance.** A sample consumer project builds against a generated header + packaged
library in CI. Out-of-tree bindings are a documented target, not a promise of shipped code.

---

#### W4.8 — CEF P2 leftovers

**State:** `todo` · **Effort:** L · **Gate:** `embedding-api.yml`

Items from `docs/embedding/CEF_PARITY.md` §9 P2 that remain. **Verify current state
before implementing each** — that section is stale in at least one known place.

| Sub-item | Verified state at writing | Notes |
|---|---|---|
| Downloads | **done** — `ffi/capi/download.rs` has 8 exports | *`CEF_PARITY.md` P2 #15 says "absent even in Rust" — stale; fix the doc* |
| Drag-and-drop | not present in `ffi/capi` | needs Rust-side delegate first |
| Audio (autoplay/output routing) | engine-level only | likely `deferred` with written reason |
| Shared/accelerated texture OSR + frame-rate control | not exposed | ties to `rendering_context.rs`; large |
| `CefRequestContext`-style isolation | `ffi/capi/request_context.rs` exists (8 exports) | **verify what it covers** before writing a ticket |
| Value containers (dict/list) | `value.rs` has 27 exports incl. `ServoValue` | verify whether nested containers exist |
| Reference-counted handles | absent | overlaps W4.6/W5.4 |
| Task-runner post API | absent | overlaps the `spin_event_loop` model |
| User-gesture flag for JS eval | absent | small, self-contained: add a flags param or a separate function (**append-only** — do not change an existing signature) |

**Steps.** One sub-item per commit. Each: state verified → implement → C test →
`symbols.txt` → docs.

**Acceptance.** The `CEF_PARITY.md` §9 P2 table reflects verified reality (fix stale
entries as part of the work — stale docs are defects here).

---

## 6. W5 — Stability, distribution, ecosystem

### 6.0 W5 current-state brief (verified)

- Hang monitor: **implemented but off**.
  - `components/config/opts.rs:52` `hang_monitor_kill_script: bool`;
    `:254` `background_hang_monitor: false` (**BHM itself defaults off**);
    `:255` `hang_monitor_kill_script: false`.
  - `components/background_hang_monitor/background_hang_monitor.rs:401-406` —
    `HangAlert::Permanent` emitted when `permanent_hang_timeout` is exceeded;
    `:266` (`BhmMessage::Exit`) and `:274-281` (`BhmMessage::SignalComponentExit`) call
    `signal_to_exit()`.
  - `components/constellation/constellation.rs:1227-1234` —
    `handle_request_from_background_hang_monitor` reacts to `HangAlert::Permanent` **only
    when `opts::get().hang_monitor_kill_script`**, then broadcasts
    `BackgroundHangMonitorControlMsg::SignalComponentExit`.
  - Test precedent: `components/background_hang_monitor/tests/hang_monitor_tests.rs`.
- `capi-artifacts.yml` — **runs and produces** (first green run `39145918170`, all three
  platforms, artifacts `servo-capi-0.1.0-<platform>-<sha>`; trigger fixed to a path-filtered
  push on this branch, and `workflow_dispatch` works after the first run registered it).
  `windows-wpt.yml` is still **never run**.
- C API version: `SERVO_CAPI_VERSION_MAJOR/MINOR/PATCH = 0/1/0` at
  `ffi/capi/lib.rs:361-367`, packed at `:373`. `SERVO_DEPRECATED` is already wired in
  `ffi/capi/cbindgen.toml:33-37` for `#[deprecated]` items.
- `examples/c-embedder/{main.c,CMakeLists.txt,README.md}` exists and was fixed in
  `6fe2549874b`; it is now compiled **and linked** against the real `cargo cinstall` sysroot
  on Linux/macOS/Windows by `capi-artifacts.yml` (run `39145918170`).

---

#### W5.1 — Hang monitor on by default + kill permanently-hung scripts

**State:** `partial` · **Effort:** M · **Gate:** every platform workflow · **Highest-value code fix remaining**

**Goal.** Implement `HARDENING_PLAN.md` §A.1 for real: BHM on, permanent hang ⇒ script
thread torn down, pref available to disable.

**Touch points.** `opts.rs:49/52/254/255`,
`background_hang_monitor.rs:401-406` (+ `:266`, `:274-281`),
`constellation.rs:1227-1234`,
`components/background_hang_monitor/tests/hang_monitor_tests.rs`,
plus wherever `Opts.background_hang_monitor` is read to start the monitor
(`git grep -n "background_hang_monitor\b" -- components/servo components/config`).

**Steps**
1. Verify end-to-end wiring first: with `--background-hang-monitor
   --hang-monitor-kill-script`, does the constellation actually terminate the component?
   Write down what happens *today* before changing defaults — the teardown path may leave
   the process wedged (joining a thread that never returns), and that must be known.
2. Flip defaults: `background_hang_monitor: true`, `hang_monitor_kill_script: true`
   (the second is meaningless without the first).
3. Make the kill path safe: after `SignalComponentExit`, the constellation must not block
   on joining the killed component — check the join sites in
   `components/servo/servo.rs:1439-1444` (`.expect("Failed to join on the script thread")`
   — a killed thread may never reach its join point; this is a plausible panic-on-shutdown).
4. Add a test: a component that stops servicing checkpoints is terminated within the
   permanent timeout and the embedder observes a defined outcome (not a hang, not an
   abort-without-message).
5. Document in `HARDENING_PLAN.md` §A.1 (it is currently *design*) and in
   `C_API_GUIDE.md` how an embedder sees this.

**Acceptance**
- [ ] Defaults are `true` for both flags; both remain individually overridable.
- [ ] A test proves termination **and** proves the process still exits cleanly (no
      deadlock on shutdown, no `expect()` panic).
- [ ] CI green on Linux/macOS/Windows.
- [ ] `HARDENING_PLAN.md` §A.1 marked implemented with the commit sha.

**Gotchas.** This is the one ticket here that can *widen* a failure mode if done badly
(killing the wrong component, or hanging shutdown). Keep the pref default change and the
teardown-safety fix in the same commit, and never ship the default flip without the
clean-shutdown test.

---

#### W5.2 — Prove `capi-artifacts.yml` (first dispatch)

**State:** ✅ verified — runs `39145918170` (push) + `39149442966` (dispatch) · **Effort:** M · **Gate:** `capi-artifacts.yml`

**Goal.** Convert a written-but-never-executed workflow into a producing one, and build
`examples/c-embedder` against a **real** sysroot rather than a stub header.

**Touch points.** `.github/workflows/capi-artifacts.yml`,
`examples/c-embedder/{main.c,CMakeLists.txt,README.md}`, `ffi/capi/Cargo.toml`
(`cargo cinstall`), `ffi/capi/cbindgen.toml`.

**Steps**
1. Dispatch it: `gh workflow run capi-artifacts.yml -r embedding/cef-parity`
   (`workflow_dispatch` exists). If the `push` trigger never fires, find out why — path
   filters, branch filters, or a disabled workflow — and fix the trigger so it is not
   manual-only.
2. Iterate on failures **in place**: read the failing job with
   `gh api repos/peter14l/servo/actions/jobs/<id>/logs`, fix, push, re-run. Repeat until
   all three platform jobs are green. (This exact loop produced `6fe2549874b` — it works.)
3. Ensure the workflow builds the example against the generated header + installed
   library (not a hand-written stub), and that the CMake/pkg-config metadata it publishes
   is what a downstream `find_package` would consume.
4. Publish artifacts with names including version + platform + sha.

**Acceptance**
- [ ] A green `capi-artifacts.yml` run id recorded in the report, with artifacts attached.
- [ ] `examples/c-embedder` compiled **and linked** against the real library on ≥1
      platform (all three is the goal).
- [ ] The workflow's `push` trigger demonstrably fires on a normal code push, or its
      manual-only status is documented deliberately.

**Gotchas.** Do not "fix" a failing step by deleting it. If a step cannot work (e.g. a
missing secret), replace it with an explicit `::warning::` + documented limitation.

---

#### W5.3 — Prebuilt signed binaries + CMake/pkg-config

**State:** `todo` · **Effort:** L · **Gate:** release workflows · **Depends:** W5.2, W5.4

**Goal.** The "minimum lovable embeddable engine" definition in `ROADMAP.md` requires
*prebuilt binaries* on Linux (x86_64/aarch64), macOS (x86_64/arm64), Windows (x86_64)
published with header + CMake package.

**Steps**
1. Start from `capi-artifacts.yml` (W5.2) rather than from `release.yml` — one pipeline,
   two consumers (CI artifacts + releases), not two divergent build scripts.
2. Publish per-platform: `servo-capi` library, generated `servo.h` (or the project's
   header name), CMake config, `.pc` file, `symbols.txt`, version, SHA-256.
3. Signing/notarization → W3.4 (same ticket series, separate verification).
4. Add a **consumer test job**: in a clean job, download the artifact, compile
   `examples/c-embedder` against *only* the downloaded files, run it. That is the proof
   that the packaging is self-sufficient.

**Acceptance.** Consumer-test job green on each platform; checksums published;
`COMPATIBILITY.md`/`ABI_STABILITY.md` support-window statements match what is published.

---

#### W5.4 — C API 0.1.0 → 1.0 ABI commitment

**State:** `partial` · **Effort:** M · **Gate:** `embedding-api.yml` · **Depends:** W5.2

**Goal.** Turn the *policy* in `ABI_STABILITY.md` into *enforced discipline*, then bump.

**Touch points.** `ffi/capi/lib.rs:361-373`, `ffi/capi/cbindgen.toml:33-37`,
`ffi/capi/symbols.txt`, `ABI_STABILITY.md`, `embedding-api.yml` symbol check.

**Steps**
1. State the current symbol contract precisely: append-only, `struct_size` for growth,
   enums append-only, breaking change ⇒ major bump. Verify CI actually fails on a removal
   (temporarily delete a line from `symbols.txt`, confirm the job goes red, restore —
   **do not commit the deletion**).
2. **Exercise the deprecation path once** (required by `ROADMAP.md` Phase 5 exit
   criteria): mark a genuinely redundant function `#[deprecated(note = "...")]`, confirm
   cbindgen emits `SERVO_DEPRECATED("...")` in the header and that the C/C++ tests still
   compile (they should, with warnings suppressed per cbindgen's generated macro), keep
   the symbol exported, and document the removal window in `ABI_STABILITY.md`.
3. Only then bump `SERVO_CAPI_VERSION` and state the support window.

**Acceptance**
- [ ] A CI-proven failure when a symbol is removed from the allowlist.
- [ ] One real, shipped deprecation with a documented removal release.
- [ ] Version bumped only after 1–2; header and `symbols.txt` consistent.

**Gotchas.** Bumping the version without an exercised deprecation path is exactly the
"docs claim policy, nothing enforces it" pattern this plan exists to eliminate.

---

#### W5.5 — Remaining reachable-panic review (tracked list)

**State:** `partial` · **Effort:** M · **Gate:** `embedding-api.yml` · **Depends:** W2.5

**Goal.** `HARDENING_PLAN.md` §A.2 — a tracked, owner-per-item list of every reachable
panic on an FFI path, not a one-line "audit done".

**Steps**
1. Inventory mechanically:
   `git grep -n "assert!\|unwrap()\|expect(\|panic!\|unreachable!" -- ffi/capi`
   and the same for the called-through `components/servo/{servo,webview,lib}.rs` paths.
2. Classify each: *unreachable by construction* (with the reason), *guarded by
   `catch_unwind`*, or *reachable → fix*. "Unreachable by construction" needs a written
   argument, not a hunch.
3. Land fixes incrementally, one file per commit, each with a negative C test where the
   input is expressible from C (null, garbage enum, freed handle, absurd length).
4. Record the list in `HARDENING_PLAN.md` §A.2 with status per item.

**Acceptance.** §A.2 shows counts before/after with item status; no *reachable* item left
untracked; `tests/capi` still passes.

**Gotchas.** `.to_str().unwrap()` on non-UTF-8 input is the classic reachable case — a
prior fix in this area (`tests/capi/c/test_api.c` invalid-UTF-8 literal) exists as a
precedent; look for siblings.

---

#### W5.6 — Observability: size / startup / throughput / crash rate in CI

**State:** `todo` · **Effort:** M · **Gate:** new CI job

**Goal.** `ROADMAP.md` Phase 5 deliverable; catch regressions instead of discovering them
in a release.

**Steps**
1. `bencher.yml` already exists — see whether it can carry the new metrics or whether a
   separate job is cleaner (`git grep -n "bencher" .github/workflows`).
2. Metrics to record per run: cdylib/artifact size, `mach smoketest` startup wall-time,
   a fixed-page paint/JS throughput sample, and job failure counts as a crash proxy.
3. Store as job output/artifact **plus** a committed baseline file so a diff is visible
   in the PR; do not start with hard gates (noisy) — start with reporting, then gate on
   the stable ones.

**Acceptance.** A CI job that emits the four metrics on every `embedding-api.yml` run and
a baseline file with the first recorded values.

---

#### W5.7 — Long-uptime / repeated load-unload leak testing

**State:** `todo` · **Effort:** L · **Gate:** `embedding-api.yml`

**Goal.** Prove that repeated create/load/close cycles do not grow unbounded — the
classic embedder defect.

**Steps**
1. Check the existing tracking allocator:
   `git grep -n "tracking" -- components/allocator` and whether it can be enabled in the
   capi test harness.
2. Add an integration test that loops N (e.g. 100) cycles of: create WebView → load a
   data/`about:blank`-style URL → wait for complete → close, sampling RSS/heap each
   iteration, and asserting growth stays under a documented bound.
3. If the harness cannot sample memory portably, use the Rust allocator counters instead
   of RSS and document the choice.
4. Keep runtime bounded — CI must not gain a 10-minute test; choose N from measurement.

**Acceptance.** A leak-cycling test runs in CI with a documented bound and a stable
runtime; the bound is justified by measured data, not chosen to make it pass.

---

## 7. W6 — Rendering & media

### 7.0 W6 current-state brief

- Graphics: `docs/embedding/GRAPHICS_AND_WEBGPU_STATUS.md` claims WebGL is
  **"Production"** on Linux/Windows/macOS/Android and WebGPU exists via
  `components/webgpu` + `components/script_webgpu`. **Those claims have no hardware
  evidence on this branch** — W3.5/W4.5 exist precisely because CI is headless.
- WebGPU pref: `dom_webgpu_enabled` declared `components/config/prefs.rs:115`, default
  **`false`** (`:479`); backend pref `dom_webgpu_wgpu_backend` at `:117`.
- Media: GStreamer via `servo-media`; `components/media/{webrtc,streams,traits}` present.
  `docs/embedding/MEDIA_AND_DRM_DESIGN.md` §2 names the packaging gap — Windows/macOS
  need a bundled/discovered GStreamer runtime, which collides with W5.3 distribution.
- DRM/EME: **design only** (`MEDIA_AND_DRM_DESIGN.md` §4) — and §4.1 correctly notes an
  open-source engine cannot bundle Widevine/FairPlay/PlayReady.
- Accessibility/printing: `docs/embedding/ACCESSIBILITY_AND_PRINTING.md` claims AccessKit
  integration in `components/script/dom/node.rs`; there is **no `components/accessibility`
  crate** in this tree, and printing is a roadmap item
  (`servo_webview_print_to_pdf`) with no implementation.
- Extensions/enterprise policy: none.

---

#### W6.1 — Validate GPU/WebGL/WebGPU and make the status doc honest

**State:** `partial` · **Effort:** L · **Gate:** manual (W3.5/W4.5) + `embedding-api.yml` for pref plumbing

**Steps**
1. Separate *architectural fact* (present in the doc) from *measured behaviour* (absent).
   Restructure `GRAPHICS_AND_WEBGPU_STATUS.md` with a column for **evidence** (test/run
   id) so an unvalidated row cannot read as "Production".
2. Turn WebGPU on in at least one integration test path and record the result
   (`dom_webgpu_enabled` default `false` — check why before flipping: likely
   `prefs.rs:479` + a feature/CI constraint).
3. Add C-level coverage: confirm what `ffi/capi` needs for an embedder to get a GL/D3D
   surface (ties to `rendering_context.rs` and W4.8 accelerated-texture item).

**Acceptance.** Status doc has an evidence column populated by real runs or marked
"unverified"; the word "Production" only appears next to a cited test.

---

#### W6.2 — Video/audio playback readiness

**State:** `partial` · **Effort:** L · **Depends:** W5.3

**Steps.** Verify what plays today on each platform (`<video>` mp4/webm, `<audio>`,
WebRTC presence via `components/media/webrtc`), document the GStreamer runtime
requirement per platform, and make W5.3 either bundle or detect it with a clear failure
message instead of a silent no-audio.

**Acceptance.** A matrix in `MEDIA_AND_DRM_DESIGN.md` §2 with *verified* playback support
per platform + container/codec, each backed by a test or a manual run record.

---

#### W6.3 — DRM/EME: define the adapter interface, declare non-support honestly

**State:** `todo` (design exists) · **Effort:** L · **Gate:** docs

**Goal.** Do not promise DRM. Do define the seam so an embedder *could* supply a CDM.

**Steps.** Turn `MEDIA_AND_DRM_DESIGN.md` §4.2 into a concrete, versioned header
sketch (`servo_cdm_adapter.h` style: open/close session, decrypt, key-status callback)
plus an explicit statement that Servo ships **no** CDM and that EME therefore returns
defined failures today.

**Acceptance.** A written interface contract + an explicit "not supported today" path in
the API docs (an error code, not a silent failure).

---

#### W6.4 — Accessibility exposure to embedders

**State:** `unproven` · **Effort:** M · **Gate:** `embedding-api.yml`

**Steps**
1. Verify the claim first: `git grep -n "accesskit" -- components Cargo.toml` and locate
   the adapter; confirm whether updates reach an embedder at all.
2. If yes: expose a delegate (`notify_accessibility_update` or a query for the root
   adapter handle) in C, with a C test asserting a callback fires after a known DOM
   mutation.
3. If no: correct `ACCESSIBILITY_AND_PRINTING.md` §1 (it currently reads as implemented)
   and record the gap.

**Acceptance.** Either a working, tested C exposure, or a corrected doc — both acceptable
outcomes; leaving the claim unverified is not.

---

#### W6.5 — Printing / `servo_webview_print_to_pdf`

**State:** `todo` (design only) · **Effort:** L

**Steps.** Assess `ACCESSIBILITY_AND_PRINTING.md` §2's proposed approach (WebRender vector
recording → PDF) against the actual render pipeline; if feasible, prototype behind a
function returning "unsupported" until it produces a file, then a C test that generates a
PDF and asserts the file is non-empty and starts with `%PDF`.

**Acceptance.** A dated feasibility note **or** a working function with the PDF test.

---

#### W6.6 — Extensions & enterprise policy

**State:** `deferred` · **Effort:** XL · **Gate:** decision

Deliberately deferred: Chrome-parity extensions are a multi-year subsystem and enterprise
policy is only relevant once a browser product exists. **Action for this ticket:** record
the deferral explicitly (in `docs/embedding/PRODUCTION_READINESS.md` once it is written —
see §D.2) so the absence is a decision rather than an omission.

---

## 8. Appendix A — Repository reference map

**Docs (`docs/embedding/`)** — read before writing a ticket; update as part of the ticket.

| File | Holds |
|---|---|
| `ARCHITECTURE.md` | Layers, object model, threading, messaging, §8.1 multiprocess bootstrap |
| `ROADMAP.md` | Embedding phases 0–5 with deliverables + exit criteria (the spec W4 executes) |
| `ABI_STABILITY.md` | Versioning, ownership/threading contract templates, deprecation policy |
| `CEF_PARITY.md` | Concept-by-concept CEF gap list §9 (P0/P1/P2) — *partly stale, see §11* |
| `C_API_GUIDE.md` | The integration guide (largest doc) |
| `COMPATIBILITY.md` | Platform/feature/symbol matrix; §5 sandbox matrix |
| `SECURITY_AUDIT.md` | Findings + prioritised remediation (§3, §5, §8, §9) |
| `HARDENING_PLAN.md` | §A.1 hang monitor, §A.2 panics, §A.3 fuzzing, Part B FFI hardening |
| `WINDOWS_HARDWARE_VALIDATION.md` | Manual GPU/hardware checklist (W3.5) |
| `MULTI_PLATFORM_SANDBOX_DESIGN.md` | Sandbox design across platforms |
| `MEDIA_AND_DRM_DESIGN.md`, `GRAPHICS_AND_WEBGPU_STATUS.md`, `ACCESSIBILITY_AND_PRINTING.md` | W6 |
| `IOS_PORT_SPIKE.md`, `ANDROID_EMBEDDING_GUIDE.md` | W3 |
| `FUZZING_AND_CRASH_TELEMETRY.md` | W2.8 |
| `PRODUCTION_READINESS.md` | **Currently a ~136-byte stub** while `PRODUCTION_REPORT.md` links it as the scorecard — needs writing (see §11) |
| `ABI_STABILITY.md`, `COMPATIBILITY.md`, `PRODUCTION_READINESS.md` | linked from `../../PRODUCTION_REPORT.md` |

**Code map**

| Area | Path |
|---|---|
| C ABI | `ffi/capi/*.rs` (26 files), `ffi/capi/cbindgen.toml`, `ffi/capi/symbols.txt` |
| Rust embedding API | `components/servo/{lib,servo,webview,webview_delegate,servo_delegate,clipboard_delegate,gamepad_delegate,site_data_manager,network_manager,user_content_manager,protocol_handler}.rs` |
| Config/prefs | `components/config/{opts,prefs}.rs` |
| Process isolation | `components/servo/servo.rs` (`create_sandbox`, `run_content_process`), `components/constellation/sandboxing.rs` |
| Cookies | `components/net/{cookie,cookie_storage,http_loader}.rs`, tests `components/net/tests/{cookie,cookie_http_state}.rs` |
| Hang monitor | `components/background_hang_monitor/background_hang_monitor.rs`, `components/constellation/constellation.rs` |
| C/C++ tests | `tests/capi/c/{test_api,test_integration}.c`, `tests/capi/cpp/test_cpp_api.cpp`, harness `tests/capi/{build.rs,tests/*.rs}` |
| Sample embedder | `examples/c-embedder/{main.c,CMakeLists.txt,README.md}` |
| Desktop reference host | `ports/servoshell/` (`desktop/{gamepad,dialog,headed_window}.rs` are the behavioural references) |
| Android | `support/android/apk/{servoapp,servoview,servoview-local,jni}` |
| Fuzz | `fuzz/` (**separate workspace**) |

---

## 9. Appendix B — CI workflows and commands

| Workflow | Triggers | What it proves | Notes |
|---|---|---|---|
| `embedding-api.yml` | push/PR, **path-filtered (root `.md` excluded)** | servo-capi builds; `cargo test -p servo-capi-tests --locked`; cbindgen header; `symbols.txt` diff; C & C++ tests | **the primary gate**; 53+ runs |
| `windows.yml` | push/PR | build, `mach smoketest`, `test-unit`, `mach package`, optional `capi` job | headless |
| `linux.yml` / `linux-common.yml` / `linux-wpt.yml` | push/PR | Linux build + unit/WPT | |
| `mac.yml` / `mac-arm64.yml` / `mac-wpt.yml` | push/PR | macOS builds | |
| `android.yml` | push/PR | Android build/smoketest | W3.1 extends it |
| `capi-artifacts.yml` | `push` (branch `embedding/cef-parity`, path-filtered) + `capi-v*` tags + `workflow_dispatch` (usable after the first run) | prebuilt `servo-capi` artifacts, builds **and links** `examples/c-embedder` against the installed sysroot | ✅ green run `39145918170` (3 platforms) → W5.2 |
| `windows-wpt.yml` | (file exists) | Windows WPT | **never run** → W1.1 (out of scope here, but dispatch is cheap) |
| `release.yml` / `release-github.yml` | release | packaging | signing status unverified → W3.4 |
| `bencher.yml` | | perf | candidate for W5.6 |

**Run-evidence rule.** Cite *run id + sha* for every green claim:
`concurrency: cancel-in-progress` means a later push cancels an in-flight run of the same
workflow. Live example: `embedding-api` run `39151946182` (sha `c3a81a6510f`) was
cancelled by the next push. The Wave A evidence runs are `39145918170` (push) and
`39149442966` (dispatch) on `7dba3701af9`, `39151946057` on `c3a81a6510f`, and
`39152343582` on `4e0eefbcb77`.

Commands:

```powershell
# dispatch
gh workflow run capi-artifacts.yml -r embedding/cef-parity
gh workflow run windows-wpt.yml    -r embedding/cef-parity

# status + watch
gh run list -R peter14l/servo -L 8
gh run view <run-id>

# why it failed: job-level logs are NOT in the run log
gh api repos/peter14l/servo/actions/jobs/<job-id>/logs

# does a workflow even exist / has it ever run? (404 = never registered)
gh api repos/peter14l/servo/actions/workflows/capi-artifacts.yml/runs
```

**Rule of thumb:** if CI fails, fix in place and re-push; never disable or delete a failing
step to get green.

---

## 10. Appendix C — Acceptance test patterns

1. **Unit/API test (C).** `tests/capi/c/test_api.c` — one block per function: call with
   valid args, assert return; call with garbage (null / zero `struct_size` / bad enum),
   assert error code **and process survival**. Compile gate used locally:
   `gcc -std=c11 -Wall -Wextra -Werror -pedantic -fsyntax-only <file>`.
2. **Integration test (C).** `tests/capi/c/test_integration.c` — spin the event loop, drive
   a real scenario, assert an observable outcome. Existing, proven-good examples to copy:
   `test_webview_http_interception` (asserts page title), `test_webview_streaming_interception`,
   `PASS: test_cookies_round_trip`, `PASS: test_site_storage`, `run_c_integration_tests`
   (register your test there).
3. **Rust-side tests** for engine changes: `cargo test -p <crate>` in CI (e.g.
   `components/net/tests/cookie.rs` for W2.6). Local `cargo check -p <crate>` first with
   `CARGO_TARGET_DIR` on `C:`.
4. **Symbol gate.** New export ⇒ append to `ffi/capi/symbols.txt`. Verify locally:
   `git grep -c "no_mangle" -- ffi/capi` should equal the file's line count (229 today,
   counting only real exports; 2 `no_mangle` items are statics).
5. **Header gate.** `cbindgen --config ffi/capi/cbindgen.toml --crate servo-capi --output <out.h>`
   must exit 0 with no error (warnings like the zero-sized-struct one should be eliminated,
   not suppressed — precedent: `6fe2549874b`).
6. **Negative tests are mandatory for W2.5/W5.5.** "It didn't crash when I passed NULL" is
   a test; "the code looks safe" is not.

---

## 11. Appendix D — Out of scope, and the stale-document register

### D.1 Why W1 is not in this plan

`PRODUCTION_REPORT.md` §1: `tests/wpt/meta` holds 18,753 expectation files with 146,721
`FAIL` entries. That is not a defect list a session closes; it is the accumulated
difference between a 2012-started engine and a moving spec target, against incumbents
with thousands of engineers. **Rewriting expectations to green the metric is explicitly
forbidden** — including for the W2.7 items. If a future session is handed W1, its correct
first action is to *measure* (dispatch `windows-wpt.yml`, publish a real pass-rate
baseline) and only then pick root causes — not to touch `tests/wpt/meta` wholesale.

### D.2 Stale documents found while writing this plan (each is a small ticket)

| Document | Stale claim | Reality at `6fe2549874b` |
|---|---|---|
| `PRODUCTION_REPORT.md` §2.2 | "no `windows-wpt.yml` exists (zero matches in `.github/workflows/`)" | `.github/workflows/windows-wpt.yml` exists (though never run) |
| `PRODUCTION_REPORT.md` §3 | "All changes are uncommitted" | ~40+ commits landed through `6fe2549874b`; tree clean |
| `PRODUCTION_REPORT.md` §1 / §5 | sandbox gate cited at `servo.rs:1468`, panic stub `servo.rs:1484-1486` | five `cfg` variants now at `servo.rs:1571-1615`; Windows has a mitigation PoC, not a panic |
| `docs/embedding/CEF_PARITY.md` §9 P2 #15 | "Downloads, drag-and-drop, audio … currently absent even in Rust" | `ffi/capi/download.rs` ships 8 exports |
| `docs/embedding/CEF_PARITY.md` §9 P0 #5 | "load start/end/error, display (cursor/favicon/console) callbacks are still missing" | `notify_cursor_changed`, `notify_history_changed`, `show_console_message`, load callbacks are wired in `webview_delegate.rs` |
| `docs/embedding/PRODUCTION_READINESS.md` | exists as a ~136-byte stub | linked from the master report as the scorecard; write it or unlink it |
| `docs/embedding/GRAPHICS_AND_WEBGPU_STATUS.md` | WebGL "Production" on 4 platforms | no hardware evidence recorded on this branch |
| `docs/embedding/ACCESSIBILITY_AND_PRINTING.md` §1 | AccessKit integration described as working incl. "C Embedding Exposure" | no `components/accessibility` crate; no a11y export in `ffi/capi`; unverified → W6.4 |
| `docs/embedding/AUDIT.md` | not updated for W4.1/W4.3 | inventory lags the branch |
| `docs/embedding/AUDIT.md` §3 | C API "still no … dialogs, DevTools" and "DevTools not reachable from C" | the DevTools server is reachable from C: `servo_preferences_set_devtools_server` plus the `ServoDelegate` callbacks `notify_devtools_server_started` / `request_devtools_connection` (exercised by the W4.4 integration test) |
| `PRODUCTION_REPORT.md` §3 and `IMPLEMENTATION_REVIEW.md` §2/§5, `IMPLEMENTATION_REVIEW_PASS2.md` §0/§5/§6 | `capi-artifacts.yml` "**Never run**"/"never dispatched" | ran green on all three platforms: run `39145918170`; artifacts `servo-capi-0.1.0-<platform>-7dba370`; `workflow_dispatch` proven (run `39149442966`)... a stale claim this plan's W5.2 closed |

**Rule:** fixing a stale document is a legitimate, committable outcome on its own
(`docs(embedding): …`). Prefer *correct the doc* over *leave the lie*.

### D.3 Definition of done for the whole plan

W2–W6 are finished — as a *plan* — when every ticket in §2.1 is either

- **verified** (acceptance + tests met, CI run id recorded), or
- **explicitly deferred** with a written reason in the responsible document,

and no document on the branch claims a capability that a test or a capability query does
not support. That standard is deliberately lower than "Blink parity" and deliberately
higher than "the code looks right" — it is the level at which this branch has actually
been maintained so far (see `../../IMPLEMENTATION_REVIEW.md`,
`../../IMPLEMENTATION_REVIEW_PASS2.md`).

