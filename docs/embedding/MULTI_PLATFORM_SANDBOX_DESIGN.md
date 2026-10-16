# Multi-Platform Renderer Sandboxing Architecture (macOS, Linux arm64, Android, iOS)

## 1. Scope & Context

Historically, Servo's sandbox implementation was restricted exclusively to Linux x86_64 and macOS x86_64 via the `gaol` crate. This left significant gaps on tier-1 target architectures:
- **macOS AArch64 (Apple Silicon):** Gated out despite macOS Seatbelt (`sandbox_init`) being native to all macOS architectures.
- **Linux AArch64:** Gated out despite Linux `seccomp-bpf` and namespace sandboxing being architecture-independent.
- **Android:** Marked unsupported in COMPATIBILITY.md despite Android's built-in isolated process model, UID isolation, and zygote seccomp filters.
- **iOS:** Marked unsupported despite iOS requiring application sandbox containerization for all binaries.

This document describes the unified multi-platform sandboxing strategy to achieve parity across all supported deployment targets.

---

## 2. Platform Sandboxing Strategies

### 2.1 macOS AArch64 (Apple Silicon)
- **Mechanism:** Apple Seatbelt Sandbox (`libsystem_sandbox.dylib` / `sandbox_init`).
- **Profile:** Content process profile restricting filesystem read to system frameworks and font caches, disallowing network sockets, and blocking process spawning.
- **Architecture Equivalence:** The Seatbelt bytecode format and `sandbox_init(kSBXProfilePureComputation, ...)` API behave identically on `x86_64` and `aarch64`.
- **Roadmap:** Extend the profile loading path to target `target_os = "macos"` unconditionally, removing the artificial `not(target_arch = "aarch64")` constraint.

### 2.2 Linux AArch64
- **Mechanism:** Linux kernel Namespaces (PID, Mount, Network, IPC) + `seccomp-bpf`.
- **System Call Architecture:** AArch64 Linux uses different system call numbers from x86_64 (defined in `<asm/unistd.h>`). Syscall filters must match the architecture:
  - Disallow `open`, `openat`, `socket`, `connect`, `bind`, `execve`, `execveat`, `fork`, `clone` (without `CLONE_THREAD`).
  - Allow memory management (`mmap`, `mprotect`, `munmap`, `brk`) and IPC file descriptors (`read`, `write`, `recvmsg`, `sendmsg`).
- **Roadmap:** Migrate or update `gaol` seccomp-bpf compiler rules to include arm64 syscall table definitions.

### 2.3 Android
- **Mechanism:** Android Isolated Process (`android:isolatedProcess="true"`) + Android SELinux Domain + Zygote Seccomp Filter.
- **Enforcement:**
  - When spawning a content process on Android via `AndroidServices`, specify `isolatedProcess="true"` in the Android manifest.
  - The Android OS assigns a dedicated, disposable UID from the isolated UID pool (`u0_i1`..`u0_i999`).
  - Android SELinux domain transition enforces `isolated_app` domain rules (no direct filesystem access, no external socket permissions, no IPC except via binder to the parent application).
  - Android Zygote applies strict seccomp-bpf filters to all untrusted isolated processes.
- **Servo Integration:** Content processes running under Android inherit this OS-level sandbox automatically. Calling `create_sandbox()` on Android confirms the isolated container runtime status.

### 2.4 iOS
- **Mechanism:** iOS App Sandbox Container + Hardened Runtime.
- **Enforcement:**
  - On iOS, every process and extension runs within a containerized sandbox managed by the iOS kernel.
  - Direct network raw sockets, unauthorized filesystem traversals, and arbitrary dynamic code execution (W^X memory protection) are enforced at the kernel level.
  - Calling `create_sandbox()` confirms compliance with the iOS sandboxed execution environment.

---

## 3. Target Support Matrix

| Platform / Architecture | Sandbox Mechanism | Status | Opt-in Flag |
| :--- | :--- | :--- | :--- |
| **Linux x86_64** | `gaol` (Namespaces + seccomp-bpf) | Production | `--sandbox` / `opts.sandbox` |
| **Linux arm64** | Namespaces + arm64 seccomp-bpf | Planned (not a sandbox today) | `--sandbox` / `opts.sandbox` |
| **macOS x86_64** | Seatbelt (`sandbox_init`) | Production | `--sandbox` / `opts.sandbox` |
| **macOS arm64 (Apple Silicon)** | Seatbelt (`sandbox_init`) | Planned (not a sandbox today) | `--sandbox` / `opts.sandbox` |
| **Windows x86_64** | In-process mitigations (Phase 1) | Mitigations only — not a sandbox | `--sandbox` / `opts.sandbox` |
| **Android** | Android Isolated Process | OS process model — not a sandbox | `--sandbox` / `opts.sandbox` |
| **iOS** | iOS App Container | OS application container — not a sandbox | `--sandbox` / `opts.sandbox` |
