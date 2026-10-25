# Native Windows Browser (Rust Servo Engine) — Agent Build Plan & Progress Tracker

Status: **Active Executable Plan** · Target: Windows 11 (22H2+ / Build 22621+) & Windows 10 Fallback · Written: 2026-10-25

---

## 0. Executive Overview

This plan defines the end-to-end engineering specification for building a modern, native Windows web browser using the **Rust Servo engine** via its C ABI (`servo-capi`). The application features full browser controls (multi-tab management, Omnibox, navigation history, find-in-page, downloads, native dialogs) with **Windows 11 Mica Alt (`DWMSBT_TABBEDWINDOW`)** visual styling.

This document is structured so that **any AI coding agent can pick up a single ticket, execute the implementation, trigger CI verification, and advance the tracker** without requiring previous conversational context.

---

## 1. Ground Rules & Agent Operating Manual (Mandatory)

### 1.1 The Golden Rule: Delegate Builds to GitHub Actions
> [!IMPORTANT]
> **DO NOT attempt a full local build of Servo on this machine.**
> Compiling the complete Servo engine (`mozjs`, `webrender`, `surfman`, and 400+ workspace crates) requires 30+ GB of disk space and significant CPU time. Local developer environments have strict disk constraints.
> 
> **Build Strategy:**
> 1. All engine builds and binary artifact generation are **delegated to GitHub Actions** using the dedicated workflow `.github/workflows/windows-browser.yml` (and `.github/workflows/capi-artifacts.yml`).
> 2. Local agents write code, update bindings, develop UI headers/sources/XAML, run lightweight syntax or unit checks, and commit/push to let GitHub Actions compile and link.
> 3. GitHub Actions runner `windows-2022` compiles `servo_capi.dll`, bundles ANGLE (`libEGL.dll`, `libGLESv2.dll`) and GStreamer, builds the browser executable, and uploads release artifacts.

### 1.2 Repository & Tooling Facts
- **Repository Root**: `D:\servo`
- **C API Crate**: `ffi/capi` (`servo-capi`), generates `servo/servo_capi.h` and `servo_capi.dll` / `servo_capi.lib`.
- **C++ RAII Header**: `ffi/capi/servo_capi.hpp` (header-only C++17 wrapper using `std::unique_ptr`).
- **Target OS / Architecture**: Windows 11 / Windows 10 (x86_64).
- **Commit Style**: Conventional Commits (`feat(windows): ...`, `fix(windows): ...`, `ci(windows): ...`, `docs(windows): ...`).

---

## 2. Master Progress Tracker

| Ticket ID | Workstream / Task | Target | Dependencies | Status | Verification Gate |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **WIN-1.1** | GitHub Actions Browser Build Workflow | CI / Workflows | None | `done` | Workflow created at `.github/workflows/windows-browser.yml` |
| **WIN-1.2** | Sysroot & ANGLE/GStreamer Packaging Pipeline | CI / Tooling | WIN-1.1 | `done` | Automated artifact assembly in CI workflow |
| **WIN-2.1** | Win32 `RawWindowHandle` v0.6 FFI Bridge | C++ / C# FFI | WIN-1.2 | `done` | Implemented in `servo_raw_window.h` and `ServoCapi.cs` |
| **WIN-2.2** | Event Loop Waker & Windows Message Pump | Win32 / WinUI | WIN-2.1 | `done` | Implemented in `main.cpp` with `WM_APP_SERVO_WAKE` |
| **WIN-2.3** | Child HWND WebViewport Container Control | Win32 / C# | WIN-2.2 | `done` | Implemented in `servo_view_host.cpp` and `servo_view_host.h` |
| **WIN-3.1** | Windows 11 Mica Alt Backdrop & Dark Mode | DWM / WinUI | WIN-2.3 | `done` | Implemented in `browser_window.cpp` with `DWMSBT_TABBEDWINDOW` |
| **WIN-3.2** | Tab Strip & Non-Client Area Window Caption | UI Shell | WIN-3.1 | `done` | Implemented in `tab_strip.cpp` and `tab_strip.h` |
| **WIN-3.3** | Omnibox (Address Bar) & Navigation Toolbar | UI Shell | WIN-3.2 | `done` | Implemented in `omnibox.cpp` and `omnibox.h` |
| **WIN-4.1** | Input Event Forwarding (Mouse, Key, Wheel, IME) | Interaction | WIN-3.3 | `done` | Implemented in `servo_view_host.cpp` |
| **WIN-4.2** | Multi-Tab / Popup Lifecycle (`request_create_new`)| Multi-View | WIN-3.3 | `done` | Implemented in `tab_strip.cpp` and `browser_window.cpp` |
| **WIN-4.3** | Modal Dialogs, Context Menu & File Picker | Dialogs | WIN-3.3 | `done` | Implemented in `servo_view_host.cpp` (MessageBox, Common Dialog, PopupMenu) |
| **WIN-4.4** | Find-in-Page Floating Bar | Features | WIN-3.3 | `done` | Implemented in `find_bar.cpp` and `find_bar.h` |
| **WIN-4.5** | Download Manager Popout & Progress Tracking | Features | WIN-3.3 | `done` | Implemented in `download_manager.cpp` and `download_manager.h` |
| **WIN-4.6** | DevTools Protocol & Diagnostic Console | Features | WIN-3.3 | `done` | Implemented in `main.cpp` and `browser_window.cpp` (F12 / Inspect) |
| **WIN-5.1** | Automated Integration Test Suite in CI | CI / Test | All above | `done` | Automated unit test suite via `ctest` in `.github/workflows/windows-browser.yml` |
| **WIN-5.2** | Release Distribution Bundle (Zip / MSIX) | Packaging | WIN-5.1 | `done` | Automated self-signing (`sign_and_package.ps1`), installer (`install.ps1`), and CI upload |

---

## 3. Architectural Blueprint

### 3.1 Layered Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│  Host Shell Window (Windows 11 Mica Alt Tabbed Window)                 │
│  - DWM Attribute: DWMSBT_TABBEDWINDOW (Mica Alt backdrop)              │
│  - Custom Non-Client Caption: Custom Tab Strip + Min/Max/Close Buttons │
│  - Navigation Bar: Back, Forward, Reload/Stop, Omnibox, Menu           │
├────────────────────────────────────────────────────────────────────────┤
│  Browser Viewport Host (Child HWND: `ServoViewHost`)                   │
│  - Receives WM_SIZE, WM_PAINT, input messages                          │
│  - Intercepts DPI changes (WM_DPICHANGED)                              │
├────────────────────────────────────────────────────────────────────────┤
│  Servo Win32 Interop Layer (`servo_win32_bridge`)                      │
│  - Constructs raw-window-handle 0.6 DisplayHandle & WindowHandle       │
│  - Converts Windows messages to servo_webview_notify_* calls           │
│  - Event loop waker: PostMessage(WM_APP_SERVO_WAKE)                    │
├────────────────────────────────────────────────────────────────────────┤
│  Servo C ABI (`servo_capi.dll`)                                        │
│  - servo_builder_*, servo_options_*, servo_preferences_*               │
│  - servo_rendering_context_create_window                               │
│  - servo_webview_builder_*, servo_webview_*                            │
├────────────────────────────────────────────────────────────────────────┤
│  Servo Rust Core (ANGLE GLES / Direct3D 11 Surface + SpiderMonkey)     │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Workstream Breakdown & Agent Tickets

### Workstream 1: CI Build Delegation & GitHub Actions Pipeline

#### Ticket WIN-1.1: GitHub Actions Browser Build Workflow
- **State**: `done`
- **Objective**: Create `.github/workflows/windows-browser.yml` to compile the Windows native browser embedder in CI, caching Rust dependencies and uploading runnable artifacts.
- **Touch Points**:
  - `.github/workflows/windows-browser.yml`
- **Implementation Steps**:
  1. Trigger on `push` to branches touching `ports/windows/**`, `examples/windows-browser/**`, or manual `workflow_dispatch`.
  2. Use runner `windows-2022`.
  3. Install `cargo-c`, `cbindgen`, and MSVC tools.
  4. Run `cargo cinstall -p servo-capi --library-type cdylib --prefix sysroot`.
  5. Build the Windows host application using CMake/MSBuild linking against `sysroot`.
  6. Bundle `servo_capi.dll`, ANGLE DLLs, GStreamer, and the executable into `servo-windows-browser.zip`.
  7. Publish as a GitHub Actions workflow artifact.
- **Acceptance Criteria**:
  - Workflow passes on GitHub Actions without disk exhaustion.
  - Generates downloadable zip artifact containing executable and DLLs.

#### Ticket WIN-1.2: Sysroot & Dependency Packaging Pipeline
- **State**: `done`
- **Objective**: Ensure all Windows runtime dependencies (`servo_capi.dll`, `libEGL.dll`, `libGLESv2.dll`, GStreamer runtime plugins) are correctly gathered and structured.
- **Touch Points**:
  - `ports/windows/CMakeLists.txt` or build scripts
  - `.github/workflows/windows-browser.yml`
- **Acceptance Criteria**:
  - The output directory contains:
    - `servo-browser.exe`
    - `servo_capi.dll`
    - `libEGL.dll` & `libGLESv2.dll` (from ANGLE)
    - `resources/` (fonts, certs, servo icons)
  - Executable launches on clean Windows 10/11 VM without missing DLL errors.

---

### Workstream 2: FFI Surface & Window Handle Binding

#### Ticket WIN-2.1: Win32 `RawWindowHandle` v0.6 FFI Bridge
- **State**: `done`
- **Context**: `servo_rendering_context_create_window` accepts opaque pointers to Rust `raw_window_handle::RawDisplayHandle` and `RawWindowHandle`.
- **Touch Points**:
  - `ports/windows/src/servo_raw_window.h` (or `.cpp` / `.cs`)
  - `ffi/capi/rendering_context.rs`
- **Implementation Steps**:
  1. Define the exact ABI-compatible C/C++ representation for `raw_window_handle 0.6.2`:
     ```c
     // RawDisplayHandle::Windows(WindowsDisplayHandle) -> tag for Windows
     typedef struct {
         uint32_t tag; // Tag corresponding to Windows
     } ServoRawDisplayHandle;

     // RawWindowHandle::Win32(Win32WindowHandle) -> tag + HWND + HINSTANCE
     typedef struct {
         uint32_t tag; // Tag for Win32
         void* hwnd;
         void* hinstance;
     } ServoRawWindowHandle;
     ```
  2. Implement helper `servo_create_win32_rendering_context(HWND hwnd, uint32_t width, uint32_t height)`.
- **Acceptance Criteria**:
  - Calling `servo_rendering_context_create_window` with the struct successfully returns a non-null `RenderingContext*` handle on Windows.

#### Ticket WIN-2.2: Event Loop Waker & Windows Message Pump
- **State**: `done`
- **Objective**: Wire `ServoEventLoopWaker` to the Windows message queue.
- **Touch Points**:
  - `ports/windows/src/main.cpp` (or `Program.cs`)
- **Implementation Steps**:
  1. Register a custom Windows message:
     ```cpp
     #define WM_SERVO_WAKE (WM_APP + 101)
     ```
  2. Implement `wake_callback`:
     ```cpp
     void on_servo_wake(void) {
         PostMessage(g_main_hwnd, WM_SERVO_WAKE, 0, 0);
     }
     ```
  3. In the Windows `WndProc`:
     ```cpp
     case WM_SERVO_WAKE:
         servo_spin_event_loop(g_servo);
         return 0;
     ```
- **Acceptance Criteria**:
  - Network and rendering callbacks process asynchronously without blocking the UI thread or freezing animations.

#### Ticket WIN-2.3: Child HWND WebViewport Container Control
- **State**: `done`
- **Objective**: Create a dedicated Win32 child window class (`ServoViewHost`) that acts as the physical rendering target.
- **Touch Points**:
  - `ports/windows/src/view_host.cpp` / `view_host.h`
- **Implementation Steps**:
  1. Register `ServoViewHost` window class with `CS_HREDRAW | CS_VREDRAW` and `WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS`.
  2. Handle `WM_PAINT`: validate update rect and call `servo_webview_paint(webview)`.
  3. Handle `WM_ERASEBKGND`: return `1` to eliminate flicker.
  4. Handle `WM_SIZE`: call `servo_webview_resize(webview, LOWORD(lParam), HIWORD(lParam))`.
  5. Handle `WM_DPICHANGED` / monitor DPI change: update HiDPI scale factor via `servo_webview_builder_set_hidpi_scale_factor`.
- **Acceptance Criteria**:
  - Web content draws smoothly inside the child window; resizing adjusts layout without visual tearing.

---

### Workstream 3: Modern Windows Shell & Mica Alt UI

#### Ticket WIN-3.1: Windows 11 Mica Alt Backdrop & Theme Synchronization
- **State**: `done`
- **Objective**: Enable Windows 11 Mica Alt (Tabbed backdrop) and dark/light theme switching.
- **Touch Points**:
  - `ports/windows/src/browser_window.cpp`
- **Implementation Steps**:
  1. Query OS version. If Windows 11 Build >= 22621:
     ```cpp
     int backdrop_type = 4; // DWMSBT_TABBEDWINDOW (Mica Alt)
     DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &backdrop_type, sizeof(backdrop_type));
     ```
  2. Sync dark/light mode with OS:
     ```cpp
     BOOL is_dark = ShouldUseDarkMode();
     DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &is_dark, sizeof(is_dark));
     servo_webview_notify_theme_change(webview, is_dark ? SERVO_THEME_DARK : SERVO_THEME_LIGHT);
     ```
  3. Fallback gracefully to solid acrylic or standard theme on Windows 10.
- **Acceptance Criteria**:
  - Main window displays translucent Mica Alt material reflecting desktop wallpaper; reacts instantly to Windows Dark/Light mode changes.

#### Ticket WIN-3.2: Custom Titlebar & Tab Strip
- **State**: `done`
- **Objective**: Implement a modern edge-to-edge tab strip in the non-client titlebar area.
- **Touch Points**:
  - `ports/windows/src/tab_strip.cpp`
  - `ports/windows/src/browser_window.cpp` (`WM_NCHITTEST`, `WM_NCCALCSIZE`)
- **Implementation Steps**:
  1. Extend DWM frame into client area via `DwmExtendFrameIntoClientArea(hwnd, &margins)`.
  2. Handle `WM_NCCALCSIZE` to remove standard titlebar while preserving window resize borders.
  3. Handle `WM_NCHITTEST` returning `HTCAPTION` for draggable header space and `HTCLIENT` for tabs and buttons.
  4. Render Tab Strip with Add Tab (`+`), Tab Close (`x`), active tab accent indicator, and smooth tab switching.
  5. Hook `notify_page_title_changed` and `notify_favicon_changed` from `ServoWebViewDelegate` to update tab headers.
- **Acceptance Criteria**:
  - Tabs sit seamlessly on the Mica Alt surface; dragging moves the window, clicking tabs switches active `WebView`.

#### Ticket WIN-3.3: Omnibox (Address Bar) & Navigation Toolbar
- **State**: `done`
- **Objective**: Build the navigation toolbar containing Back, Forward, Reload/Stop, and the address bar.
- **Touch Points**:
  - `ports/windows/src/omnibox.cpp`
- **Implementation Steps**:
  1. Create controls: Back (`<-`), Forward (`->`), Reload/Stop (`⟳`/`✕`), and Omnibox (`EditControl`).
  2. Wire clicks to:
     - Back: `servo_webview_go_back(webview, 1)`
     - Forward: `servo_webview_go_forward(webview, 1)`
     - Reload/Stop: toggle `servo_webview_reload(webview)` and `servo_webview_stop(webview)` based on `servo_webview_get_load_status(webview)`.
  3. Wire Omnibox `VK_RETURN`: parse input (URL or search query) and call `servo_webview_load(webview, url)`.
  4. Wire delegate callbacks:
     - `notify_url_changed`: update Omnibox text.
     - `notify_load_started` / `notify_load_ended`: toggle Reload/Stop button icon.
     - `notify_history_changed`: enable/disable Back & Forward buttons using `servo_webview_can_go_back`/`can_go_forward`.
- **Acceptance Criteria**:
  - Typing a URL loads the page; navigation buttons reflect history state accurately.

---

### Workstream 4: Complete Browser Controls & Event Integration

#### Ticket WIN-4.1: Input Event Forwarding (Mouse, Keyboard, Wheel, IME)
- **State**: `done`
- **Objective**: Forward native Windows input events to the active Servo `WebView`.
- **Touch Points**:
  - `ports/windows/src/servo_view_host.cpp`
- **Implementation Steps**:
  1. Mouse events:
     - `WM_LBUTTONDOWN`, `WM_RBUTTONDOWN`, `WM_MBUTTONDOWN` -> `servo_webview_notify_mouse_button_event`
     - `WM_MOUSEMOVE` -> `servo_webview_notify_mouse_move_event`
     - `WM_MOUSELEAVE` -> `servo_webview_notify_mouse_leave_event`
  2. Wheel events:
     - `WM_MOUSEWHEEL`, `WM_MOUSEHWHEEL` -> `servo_webview_notify_wheel_event`
  3. Keyboard events:
     - `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, `WM_SYSKEYUP` -> map virtual key codes and modifiers to `servo_webview_notify_key_event`.
  4. Cursor changes:
     - Hook `notify_cursor_changed` in `ServoWebViewDelegate` and call `SetCursor(LoadCursor(...))`.
- **Acceptance Criteria**:
  - Web pages receive mouse clicks, scrolling, hover states, keyboard inputs, and text selection.

#### Ticket WIN-4.2: Multi-Tab & Popup Lifecycle
- **State**: `done`
- **Objective**: Support multiple webviews and popup windows via `request_create_new`.
- **Touch Points**:
  - `ports/windows/src/tab_strip.cpp`
  - `ports/windows/src/browser_window.cpp`
- **Implementation Steps**:
  1. Implement `request_create_new` in `ServoWebViewDelegate`:
     - If user preference is "Open in new tab", call `servo_create_new_webview_builder_create(request)` and add a new tab to the Tab Strip.
     - If popup window requested, spawn a new top-level `ServoBrowserWindow`.
     - Otherwise free with `servo_create_new_webview_request_free(request)`.
  2. Handle tab switching by hiding inactive child HWND and showing active child HWND (`ShowWindow(hwnd, SW_SHOW/SW_HIDE)`).
  3. Handle `servo_webview_close` and `notify_closed` when closing tabs.
- **Acceptance Criteria**:
  - Clicking external target `_blank` links opens a new tab without crashing.

#### Ticket WIN-4.3: Native Dialogs, Context Menu & File Picker
- **State**: `done`
- **Objective**: Wire web page interactions to native Windows dialogs and menus.
- **Touch Points**:
  - `ports/windows/src/servo_view_host.cpp`
- **Implementation Steps**:
  1. Alerts & Confirms: Implement `show_alert_dialog` (`MessageBoxW`), `show_confirm_dialog` (`MB_YESNO`), `show_prompt_dialog` (custom Win32 dialog).
  2. File Picker: Implement `show_file_picker` using Windows Common Item Dialog (`IFileOpenDialog` / `GetOpenFileNameW`).
  3. Context Menu: Implement `show_context_menu` invoking `TrackPopupMenuEx` with Back, Forward, Reload, Copy, Inspect options.
- **Acceptance Criteria**:
  - `<input type="file">` opens Windows file picker; `alert()` shows native message box.

#### Ticket WIN-4.4: Find-in-Page Floating Bar
- **State**: `done`
- **Objective**: Implement in-page search triggered by `Ctrl + F`.
- **Touch Points**:
  - `ports/windows/src/find_bar.cpp`
- **Implementation Steps**:
  1. Keyboard accelerator `Ctrl + F` reveals the floating find bar at the top-right of the viewport.
  2. On text change: call `servo_webview_find(webview, search_text, true, false)`.
  3. "Next" button: calls `servo_webview_find_next(webview, identifier, true)`.
  4. "Previous" button: calls `servo_webview_find_next(webview, identifier, false)`.
  5. Close button / Escape: calls `servo_webview_find_clear(webview, identifier)` and hides bar.
  6. Hook `on_find_result` callback to display match count (e.g., "3 of 12 matches").
- **Acceptance Criteria**:
  - Matches are highlighted in the webpage, and user can cycle through occurrences.

#### Ticket WIN-4.5: Download Manager Popout & Progress Tracking
- **State**: `done`
- **Objective**: Track file downloads initiated by the web engine.
- **Touch Points**:
  - `ports/windows/src/download_manager.cpp`
- **Implementation Steps**:
  1. Implement `on_before_download`: show download flyout and invoke `continue_callback(true, user_data)`.
  2. Implement `on_download_updated`: query `servo_download_item_get_received_bytes`, `get_total_bytes`, and `get_state`.
  3. Render download progress bar and provide "Open folder" or "Cancel" actions.
- **Acceptance Criteria**:
  - Downloading a file displays live byte progress and completes safely.

#### Ticket WIN-4.6: DevTools Remote Debugging Integration
- **State**: `done`
- **Objective**: Enable DevTools inspection for web developers.
- **Touch Points**:
  - `ports/windows/src/browser_window.cpp`
- **Implementation Steps**:
  1. Set preference `devtools_server_enabled` to `true` during `ServoBuilder` initialization.
  2. Implement `notify_devtools_server_started` in `ServoDelegate` to record port and auth token.
  3. Provide a "Developer Tools" menu option that opens the inspection URL in a dedicated window or browser tab.
- **Acceptance Criteria**:
  - Navigating to the DevTools URL inspects DOM elements, console logs, and network calls.

---

### Workstream 5: Verification, Packaging & Distribution

#### Ticket WIN-5.1: Automated Integration Test Suite in CI
- **State**: `done`
- **Objective**: Add automated smoke tests running against the Windows embedder in GitHub Actions.
- **Touch Points**:
  - `ports/windows/tests/**`
  - `.github/workflows/windows-browser.yml`
- **Acceptance Criteria**:
  - CI launches the compiled browser, navigates to a local test server, takes a screenshot, and validates exit code 0.

#### Ticket WIN-5.2: Release Distribution Bundle (Zip / MSIX)
- **State**: `done`
- **Objective**: Create clean distributable release packages.
- **Touch Points**:
  - `ports/windows/packaging/**`
- **Acceptance Criteria**:
  - Produces standalone portable `.zip` and/or Windows `.msix` installer package containing all assets, certificates, and runtime binaries.

---

## 5. Verification Checklist for AI Sessions

When working on any ticket in this plan:
1. **Never attempt a full local build.** Push changes or trigger workflow dispatch via:
   ```sh
   gh workflow run windows-browser.yml
   ```
2. Verify GitHub Actions build status using:
   ```sh
   gh run list --workflow=windows-browser.yml
   gh run view <run-id> --log-failed
   ```
3. Update the status column in the **Master Progress Tracker** table (§2) from `todo` -> `in_progress` -> `done`.
4. Ensure no regression in existing C ABI tests ([`tests/capi`](file:///D:/servo/tests/capi)).
