# Windows Renderer Sandbox Architecture and Design

## 1. Executive Summary

In Servo's multi-process architecture (`--multiprocess`), web content (DOM, CSS layout, JavaScript execution via SpiderMonkey, WebGL/WebGPU, and media decode) executes in unprivileged content processes. To achieve defense-in-depth parity with modern browser engines (Chromium/CEF and WebKit), content processes must be heavily sandboxed so that an arbitrary code execution vulnerability inside the renderer cannot compromise host files, registry, credentials, or network interfaces.

This document describes the Windows renderer sandbox architecture for Servo, transitioning from the previous behavior (where `opts.sandbox` unconditionally panicked on Windows) to a production-grade multi-tiered Windows sandbox.

---

## 2. Threat Model & Sandboxing Goals

### 2.1 Threat Vectors
- **Memory Corruption in Render Pipeline:** Vulnerabilities in SpiderMonkey JIT compiler, WebRender GPU translation, font parsing (FreeType/DirectWrite), image decoding, or DOM trees.
- **Unauthorized Filesystem Access:** Malicious content reading cookie storage, cache, user documents, or embedding host binaries.
- **Unauthorized Network Access:** Subverting Same-Origin Policy or intranet filtering by opening raw TCP/UDP sockets directly from the compromised child process.
- **Privilege Escalation:** Exploiting Windows kernel or GDI/DirectX drivers via win32k system calls.

### 2.2 Sandboxing Goals
1. **Zero Direct Filesystem Access:** Target process cannot read or write any host file or registry key directly without explicit broker mediation.
2. **Zero Direct Network Access:** Network operations are strictly confined to the network thread in the browser process; child has no socket creation rights.
3. **No Process / Thread Injection:** Disallow creating processes or opening handles to other processes.
4. **Kernel Attack Surface Reduction:** Block `win32k.sys` system calls for processes that do not directly manage native window handles.

---

## 3. Two-Tier Architecture: Broker vs. Target

```
+--------------------------------------------------------------+
|                     Browser Process (Broker)                 |
|                                                              |
|  - Host Application / C API Embedder                         |
|  - Constellation & Navigation Coordinator                    |
|  - Network System (fetch, cookies, cache, sockets)           |
|  - Window Compositor (direct GPU & desktop presentation)     |
+------------------------------+-------------------------------+
                               |
                   IPC Channel (Pipe / Shared Memory)
                               |
+------------------------------v-------------------------------+
|                    Content Process (Target)                  |
|                                                              |
|  - HTML / CSS Parser & Layout Factory                        |
|  - JavaScript Engine (SpiderMonkey)                          |
|  - Offscreen Surface Renderer                                |
|                                                              |
|  [Security Sandbox]:                                         |
|    * AppContainer / Restricted Token (Low Integrity SID)     |
|    * Job Object (Kill on close, process limit = 1)           |
|    * Process Mitigation Policies (DEP, ASLR, Handles, Hooks) |
|    * Win32k Lockdown (GDI/USER syscall disable)              |
+--------------------------------------------------------------+
```

### 3.1 Broker Process (Host / Constellation)
- Runs with normal user permissions.
- Owns all high-privilege OS resources: window handles, network sockets, persistent storage, and GPU device contexts.
- Spawns content processes using Windows Security APIs (`CreateProcessAsUserW` or `CreateProcessW` with `EXTENDED_STARTUPINFO_PRESENT`).
- Manages lifecycle through a dedicated Windows **Job Object**.

### 3.2 Target Process (Content Process)
- Spawns in an isolated security context.
- Communicates exclusively over Servo IPC channels (anonymous pipes / shared memory).
- Before executing any untrusted web content or parsing network buffers, activates in-process mitigation policies via `create_sandbox()`.

---

## 4. Sandbox Enforcement Layers

### Tier 1: Windows Job Object Isolation
Every sandboxed content process is assigned to an anonymous Windows Job Object (`CreateJobObjectW`):
- `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`: Automatically terminates child processes if the browser process crashes or exits.
- `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION`: Prevents user-facing crash dialogs from hanging the engine.
- `JOB_OBJECT_LIMIT_ACTIVE_PROCESS = 1`: Prevents the sandboxed child process from spawning any subprocesses (`CreateProcess` fails with `ERROR_ACCESS_DENIED`).
- Optional desktop restrictions (`SetInformationJobObject` with `JobObjectBasicUIRestrictions`):
  - `JOB_OBJECT_UILIMIT_DESKTOP`: Child cannot create or switch desktops.
  - `JOB_OBJECT_UILIMIT_HANDLES`: Child cannot access USER handles owned by other processes.
  - `JOB_OBJECT_UILIMIT_GLOBALATOMS`: Child cannot read or modify global atoms.

### Tier 2: Token Lockdown (Restricted Token & Low Integrity)
The child process token is created via `CreateRestrictedToken`:
1. **Disable Admin & Sensitive SIDs:** Removes `BUILTIN\Administrators`, `Network`, `Remote Interactive Logon`, and power user privileges.
2. **Drop Privileges:** All Windows privileges (`SE_*`) are removed except `SeChangeNotifyPrivilege`.
3. **Restricting SIDs:** Sets Restricting SIDs so that access checks require both the token and the restricted SID list to match.
4. **Integrity Level:** Set to **Low Integrity** (`S-1-16-4096`) or **AppContainer** (`S-1-15-2-...`). Any file, registry key, or named pipe without explicit Low Integrity write permissions cannot be written to.

### Tier 3: In-Process Mitigation Policies (`SetProcessMitigationPolicy`)
Applied in `create_sandbox()` inside the child process before JS engine initialization:
1. **DEP Policy (`ProcessDEPPolicy`):**
   - `Enable = 1`, `DisableAtlThunkEmulation = 1`, `Permanent = 1`.
2. **ASLR Policy (`ProcessASLRPolicy`):**
   - `EnableBottomUpRandomization = 1`, `EnableForceRelocateImages = 1`, `EnableHighEntropy = 1`, `DisallowStrippedImages = 1`.
3. **Strict Handle Checking (`ProcessStrictHandleCheckPolicy`):**
   - `RaiseExceptionOnInvalidHandleReference = 1`. Triggers immediate exception if an invalid or closed handle is referenced.
4. **Extension Point Disable (`ProcessExtensionPointDisablePolicy`):**
   - `DisableExtensionPoints = 1`. Prevents legacy global window hooks (`SetWindowsHookEx`) and DLL injection via AppInit_DLLs from injecting third-party code into the renderer.
5. **Win32k System Call Lockdown (`ProcessSystemCallDisablePolicy`):**
   - `DisallowWin32kSystemCalls = 1`. Prevents the renderer from calling `user32.dll`/`gdi32.dll` kernel entry points (`NtUser*`, `NtGdi*`), neutralizing ~70% of historical Windows kernel privilege escalation vulnerabilities.
   - *Requirement:* Offscreen software or GPU compositing via ANGLE/D3D11 shared surfaces where DirectX DXGI swapchains are brokered.

---

## 5. Current PoC Implementation & Phased Rollout

### Phase 1: In-Process Mitigation Policies (Landed)
- Replaced unconditional `panic!()` in `servo::create_sandbox()` on Windows.
- Dynamically configures `SetProcessMitigationPolicy` for DEP, ASLR, strict handle checking, and extension-point disabling.
- Controlled via existing `--sandbox` / `opts.sandbox` preference (default: off).

### Phase 2: Broker Job Object Assignment (Planned)
- Enhance `servo_constellation` pipeline spawner on Windows to create a Job Object per pipeline and bind `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and `JOB_OBJECT_LIMIT_ACTIVE_PROCESS`.

### Phase 3: Restricted Token & AppContainer Profile (Planned)
- Implement `CreateAppContainerProfile` for production builds with a dedicated package family name `org.servo.content`.
- Configure shared memory D3D/DirectComposition handles for zero-copy rendering without win32k access.
