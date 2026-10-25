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

static HWND g_main_hwnd = nullptr;
static BrowserWindow* g_browser_window = nullptr;

static void on_servo_wake_callback(void) {
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

    // 1. Initialize COM and Common Controls v6
    CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

    INITCOMMONCONTROLSEX icc = {};
    icc.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icc.dwICC = ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(&icc);

    // 2. Initialize Servo
    ServoBuilder* builder = servo_builder_create();
    if (!builder) {
        MessageBoxW(nullptr, L"Failed to initialize ServoBuilder.", L"Servo Browser Error", MB_ICONERROR);
        CoUninitialize();
        return 1;
    }

    // Set event loop waker
    ServoEventLoopWaker waker = {};
    waker.struct_size = sizeof(ServoEventLoopWaker);
    waker.wake_callback = on_servo_wake_callback;
    servo_builder_set_event_loop_waker(builder, waker);

    // Set startup options
    ServoOptions* options = servo_options_create();
    if (options) {
        servo_options_set_multiprocess(options, false);
        servo_builder_set_options(builder, options);
    }

    // Set preferences
    ServoPreferences* prefs = servo_preferences_create();
    if (prefs) {
        servo_preferences_set_bool(prefs, "devtools_server_enabled", true);
        servo_builder_set_preferences(builder, prefs);
    }

    // Set delegate for DevTools notifications
    ServoDelegate delegate = {};
    delegate.struct_size = sizeof(ServoDelegate);
    delegate.notify_devtools_server_started = on_devtools_server_started;
    servo_builder_set_delegate(builder, delegate);

    struct Servo* servo = servo_builder_build(builder);
    if (!servo) {
        MessageBoxW(nullptr, L"Failed to build Servo instance.", L"Servo Browser Error", MB_ICONERROR);
        CoUninitialize();
        return 1;
    }

    servo_setup_logging(servo);

    // 3. Register Window Classes
    BrowserWindow::RegisterWindowClass(hInstance);
    TabStrip::RegisterWindowClass(hInstance);
    OmniboxToolbar::RegisterWindowClass(hInstance);
    FindBar::RegisterWindowClass(hInstance);
    DownloadManager::RegisterWindowClass(hInstance);
    ServoViewHost::RegisterWindowClass(hInstance);

    // 4. Create and Show Browser Window
    BrowserWindow* browserWindow = BrowserWindow::Create(servo, "https://servo.org");
    if (!browserWindow) {
        MessageBoxW(nullptr, L"Failed to create main browser window.", L"Servo Browser Error", MB_ICONERROR);
        servo_free(servo);
        CoUninitialize();
        return 1;
    }

    g_browser_window = browserWindow;
    g_main_hwnd = browserWindow->GetHwnd();

    // 5. External Message Pump
    MSG msg;
    while (GetMessageW(&msg, nullptr, 0, 0)) {
        if (msg.message == WM_APP_SERVO_WAKE) {
            servo_spin_event_loop(servo);
        } else {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        // Let Servo make progress after handling inputs/events
        servo_spin_event_loop(servo);
    }

    // 6. Cleanup
    delete browserWindow;
    servo_free(servo);
    CoUninitialize();

    return static_cast<int>(msg.wParam);
}
