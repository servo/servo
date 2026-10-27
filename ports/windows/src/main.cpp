/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <commctrl.h>
#include <objbase.h>

#include <servo/servo_capi.h>
#include "browser_window.h"
#include "tab_strip.h"
#include "omnibox.h"
#include "find_bar.h"
#include "download_manager.h"
#include "servo_view_host.h"

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "ole32.lib")

#define WM_APP_SERVO_WAKE (WM_APP + 101)

#include <cstdio>
#include <cstdarg>
#include <share.h>

void LogDebug(const char* fmt, ...) {
    static FILE* f = nullptr;
    if (!f) {
        wchar_t exePath[MAX_PATH] = {};
        GetModuleFileNameW(nullptr, exePath, MAX_PATH);
        wchar_t* lastSlash = wcsrchr(exePath, L'\\');
        if (lastSlash) {
            *(lastSlash + 1) = L'\0';
            wcscat_s(exePath, L"servo_browser.log");
            f = _wfsopen(exePath, L"a", _SH_DENYNO);
        } else {
            f = _wfsopen(L"servo_browser.log", L"a", _SH_DENYNO);
        }
    }
    if (f) {
        va_list args;
        va_start(args, fmt);
        vfprintf(f, fmt, args);
        fprintf(f, "\n");
        fflush(f);
        va_end(args);
    }
}

static DWORD g_main_thread_id = 0;
static HWND g_main_hwnd = nullptr;
static BrowserWindow* g_browser_window = nullptr;

static void on_servo_wake_callback(void) {
    if (g_main_thread_id != 0) {
        PostThreadMessageW(g_main_thread_id, WM_APP_SERVO_WAKE, 0, 0);
    }
    if (g_main_hwnd && IsWindow(g_main_hwnd)) {
        PostMessageW(g_main_hwnd, WM_APP_SERVO_WAKE, 0, 0);
    }
}

static void on_devtools_server_started(struct Servo* servo, uint16_t port, const char* token, void* user_data) {
    (void)servo;
    (void)user_data;
    if (g_browser_window) {
        g_browser_window->SetDevToolsInfo(port, token ? token : "");
    }
}

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
    (void)hPrevInstance;
    (void)lpCmdLine;
    (void)nCmdShow;

    g_main_thread_id = GetCurrentThreadId();

    LogDebug("--- Starting Servo Browser ---");

    // 1. Initialize COM and Common Controls v6
    HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    LogDebug("CoInitializeEx result: 0x%08X", hr);

    INITCOMMONCONTROLSEX icc = {};
    icc.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icc.dwICC = ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(&icc);

    // 2. Initialize Servo
    ServoBuilder* builder = servo_builder_create();
    if (!builder) {
        LogDebug("Failed to initialize ServoBuilder.");
        MessageBoxW(nullptr, L"Failed to initialize ServoBuilder.", L"Servo Browser Error", MB_ICONERROR);
        CoUninitialize();
        return 1;
    }
    LogDebug("ServoBuilder created.");

    // Set event loop waker
    ServoEventLoopWaker waker = {};
    waker.struct_size = sizeof(ServoEventLoopWaker);
    waker.wake_callback = on_servo_wake_callback;
    ServoError wakerErr = servo_builder_set_event_loop_waker(builder, waker);
    LogDebug("servo_builder_set_event_loop_waker returned: %d", (int)wakerErr);

    // Set startup options
    ServoOptions* options = servo_options_create();
    if (options) {
        servo_options_set_multiprocess(options, false);
        servo_builder_set_options(builder, options);
    }

    // Set preferences
    ServoPreferences* prefs = servo_preferences_create();
    if (prefs) {
        servo_preferences_set_bool(prefs, "devtools_server_enabled", false);
        servo_builder_set_preferences(builder, prefs);
    }

    // Set delegate for DevTools notifications
    ServoDelegate delegate = {};
    delegate.struct_size = sizeof(ServoDelegate);
    delegate.notify_devtools_server_started = on_devtools_server_started;
    servo_builder_set_delegate(builder, delegate);

    struct Servo* servo = servo_builder_build(builder);
    if (!servo) {
        LogDebug("Failed to build Servo instance.");
        MessageBoxW(nullptr, L"Failed to build Servo instance.", L"Servo Browser Error", MB_ICONERROR);
        CoUninitialize();
        return 1;
    }
    LogDebug("Servo instance built.");

    servo_setup_logging(servo);

    // 3. Register Window Classes
    BrowserWindow::RegisterWindowClass(hInstance);
    TabStrip::RegisterWindowClass(hInstance);
    OmniboxToolbar::RegisterWindowClass(hInstance);
    FindBar::RegisterWindowClass(hInstance);
    DownloadManager::RegisterWindowClass(hInstance);
    ServoViewHost::RegisterWindowClass(hInstance);
    LogDebug("Window classes registered.");

    // 4. Create and Show Browser Window
    BrowserWindow* browserWindow = BrowserWindow::Create(servo, "https://servo.org");
    if (!browserWindow) {
        LogDebug("BrowserWindow::Create failed!");
        servo_free(servo);
        CoUninitialize();
        return 1;
    }

    g_browser_window = browserWindow;
    g_main_hwnd = browserWindow->GetHwnd();
    LogDebug("Browser window created successfully. HWND: 0x%p", g_main_hwnd);

    // Kickstart the Servo event loop so initial navigation starts immediately
    PostMessageW(g_main_hwnd, WM_APP_SERVO_WAKE, 0, 0);

    // 5. External Message Pump
    LogDebug("Entering message pump...");
    MSG msg;
    while (GetMessageW(&msg, nullptr, 0, 0)) {
        if (msg.message == WM_APP_SERVO_WAKE) {
            ServoError err = servo_spin_event_loop(servo);
            if (err != SERVO_ERROR_OK) {
                LogDebug("servo_spin_event_loop returned error %d", (int)err);
            }
        } else {
            LogDebug("Dispatching msg=0x%04X hwnd=0x%p", msg.message, msg.hwnd);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            LogDebug("Dispatch done for 0x%04X, spinning servo...", msg.message);
            ServoError err = servo_spin_event_loop(servo);
            LogDebug("servo_spin_event_loop returned %d", (int)err);
        }
    }
    LogDebug("Message pump finished with code %d", (int)msg.wParam);

    // 6. Cleanup
    delete browserWindow;
    servo_free(servo);
    CoUninitialize();

    return static_cast<int>(msg.wParam);
}
