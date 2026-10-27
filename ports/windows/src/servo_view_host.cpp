/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "servo_view_host.h"
#include "servo_raw_window.h"
#include "browser_window.h"
#include <windowsx.h>
#include <commdlg.h>
#include <vector>

static const wchar_t* SERVO_VIEW_CLASS_NAME = L"ServoViewHostWindow";

void ServoViewHost::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
    wc.lpfnWndProc = ServoViewHost::WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = nullptr; // Let Servo render directly
    wc.lpszClassName = SERVO_VIEW_CLASS_NAME;
    RegisterClassExW(&wc);
}

ServoViewHost::ServoViewHost(HWND parentHwnd, struct Servo* servo, BrowserWindow* browserWindow, const std::string& initialUrl)
    : m_hwnd(nullptr)
    , m_parentHwnd(parentHwnd)
    , m_servo(servo)
    , m_webview(nullptr)
    , m_context(nullptr)
    , m_browserWindow(browserWindow)
    , m_trackingMouse(false)
    , m_currentCursor(LoadCursorW(nullptr, IDC_ARROW))
{
    HINSTANCE hInstance = GetModuleHandleW(nullptr);

    LogDebug("ServoViewHost creating child window...");
    // Create child window
    m_hwnd = CreateWindowExW(
        0,
        SERVO_VIEW_CLASS_NAME,
        L"",
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 800, 600,
        parentHwnd,
        nullptr,
        hInstance,
        this
    );
    LogDebug("ServoViewHost child window created: 0x%p", m_hwnd);

    // Create windowed rendering context
    RECT rc = {};
    GetClientRect(m_hwnd, &rc);
    uint32_t width = rc.right > 0 ? rc.right : 800;
    uint32_t height = rc.bottom > 0 ? rc.bottom : 600;

    LogDebug("Creating rendering context width=%u height=%u...", width, height);
    m_context = servo_win::create_win32_rendering_context(m_hwnd, width, height);
    if (!m_context) {
        LogDebug("Win32 window context failed, attempting software rendering context...");
        m_context = servo_rendering_context_create_software(width, height);
    }
    LogDebug("RenderingContext ready: 0x%p", m_context);

    // Build the WebView
    LogDebug("Building Servo WebView with URL %s...", initialUrl.c_str());
    ServoWebViewBuilder* builder = servo_webview_builder_create(m_servo, m_context);
    if (builder) {
        ServoError urlErr = servo_webview_builder_set_url(builder, initialUrl.empty() ? "about:blank" : initialUrl.c_str());
        LogDebug("servo_webview_builder_set_url returned %d", (int)urlErr);

        ServoWebViewDelegate delegate = {};
        delegate.struct_size = sizeof(ServoWebViewDelegate);
        delegate.user_data = this;
        delegate.notify_new_frame_ready = OnNewFrameReady;
        delegate.notify_url_changed = OnUrlChanged;
        delegate.notify_page_title_changed = OnTitleChanged;
        delegate.notify_load_status_changed = OnLoadStatusChanged;
        delegate.notify_cursor_changed = OnCursorChanged;
        delegate.show_alert_dialog = reinterpret_cast<decltype(delegate.show_alert_dialog)>(OnAlertDialog);
        delegate.show_confirm_dialog = reinterpret_cast<decltype(delegate.show_confirm_dialog)>(OnConfirmDialog);
        delegate.show_prompt_dialog = reinterpret_cast<decltype(delegate.show_prompt_dialog)>(OnPromptDialog);
        delegate.show_file_picker = reinterpret_cast<decltype(delegate.show_file_picker)>(OnFilePicker);
        delegate.show_context_menu = reinterpret_cast<decltype(delegate.show_context_menu)>(OnContextMenu);
        delegate.on_before_download = reinterpret_cast<decltype(delegate.on_before_download)>(OnBeforeDownload);
        delegate.on_download_updated = reinterpret_cast<decltype(delegate.on_download_updated)>(OnDownloadUpdated);

        ServoError delErr = servo_webview_builder_set_delegate(builder, delegate);
        LogDebug("servo_webview_builder_set_delegate returned %d", (int)delErr);
        m_webview = servo_webview_builder_build(builder);
        LogDebug("servo_webview_builder_build returned: 0x%p", m_webview);
    }
}

ServoViewHost::~ServoViewHost() {
    if (m_webview) {
        servo_webview_free(m_webview);
        m_webview = nullptr;
    }
    if (m_hwnd && IsWindow(m_hwnd)) {
        DestroyWindow(m_hwnd);
        m_hwnd = nullptr;
    }
}

void ServoViewHost::SetBounds(int x, int y, int width, int height) {
    if (m_hwnd) {
        SetWindowPos(m_hwnd, nullptr, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

void ServoViewHost::Show(bool show) {
    if (m_hwnd) {
        ShowWindow(m_hwnd, show ? SW_SHOW : SW_HIDE);
        if (show && m_webview) {
            servo_webview_show(m_webview);
        } else if (!show && m_webview) {
            servo_webview_hide(m_webview);
        }
    }
}

void ServoViewHost::SetFocus() {
    if (m_hwnd) {
        ::SetFocus(m_hwnd);
        if (m_webview) {
            servo_webview_set_focused(m_webview, true);
        }
    }
}

void ServoViewHost::LoadUrl(const std::string& url) {
    if (m_webview && !url.empty()) {
        servo_webview_load(m_webview, url.c_str());
    }
}

void ServoViewHost::Reload() {
    if (m_webview) {
        servo_webview_reload(m_webview);
    }
}

void ServoViewHost::Stop() {
    if (m_webview) {
        servo_webview_stop(m_webview);
    }
}

void ServoViewHost::GoBack() {
    if (m_webview && servo_webview_can_go_back(m_webview)) {
        servo_webview_go_back(m_webview, 1);
    }
}

void ServoViewHost::GoForward() {
    if (m_webview && servo_webview_can_go_forward(m_webview)) {
        servo_webview_go_forward(m_webview, 1);
    }
}

bool ServoViewHost::CanGoBack() const {
    return m_webview ? servo_webview_can_go_back(m_webview) : false;
}

bool ServoViewHost::CanGoForward() const {
    return m_webview ? servo_webview_can_go_forward(m_webview) : false;
}

std::string ServoViewHost::GetUrl() const {
    if (!m_webview) return "";
    uintptr_t len = servo_webview_get_url(m_webview, nullptr, 0);
    if (len <= 1) return "";
    std::vector<char> buf(len);
    servo_webview_get_url(m_webview, buf.data(), len);
    return std::string(buf.data());
}

std::string ServoViewHost::GetTitle() const {
    if (!m_webview) return "";
    uintptr_t len = servo_webview_get_title(m_webview, nullptr, 0);
    if (len <= 1) return "";
    std::vector<char> buf(len);
    servo_webview_get_title(m_webview, buf.data(), len);
    return std::string(buf.data());
}

uint32_t ServoViewHost::GetServoModifiers() const {
    uint32_t mod = 0;
    if (GetKeyState(VK_SHIFT) & 0x8000)   mod |= SERVO_MODIFIER_SHIFT;
    if (GetKeyState(VK_CONTROL) & 0x8000) mod |= SERVO_MODIFIER_CONTROL;
    if (GetKeyState(VK_MENU) & 0x8000)    mod |= SERVO_MODIFIER_ALT;
    return mod;
}

LRESULT CALLBACK ServoViewHost::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    ServoViewHost* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<ServoViewHost*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<ServoViewHost*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT ServoViewHost::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC hdc = BeginPaint(m_hwnd, &ps);
        if (m_webview) {
            ServoError err = servo_webview_paint(m_webview);
            LogDebug("servo_webview_paint returned %d", (int)err);
        }

        // 1. Try EGL swap (used when Servo is backed by ANGLE)
        HMODULE hEgl = GetModuleHandleW(L"libEGL.dll");
        if (hEgl) {
            typedef void* (*PFN_eglGetCurrentDisplay)(void);
            typedef void* (*PFN_eglGetCurrentSurface)(int readdraw);
            typedef unsigned int (*PFN_eglSwapBuffers)(void* dpy, void* surface);

            auto pfnGetCurrentDisplay = (PFN_eglGetCurrentDisplay)GetProcAddress(hEgl, "eglGetCurrentDisplay");
            auto pfnGetCurrentSurface = (PFN_eglGetCurrentSurface)GetProcAddress(hEgl, "eglGetCurrentSurface");
            auto pfnSwapBuffers = (PFN_eglSwapBuffers)GetProcAddress(hEgl, "eglSwapBuffers");

            if (pfnGetCurrentDisplay && pfnGetCurrentSurface && pfnSwapBuffers) {
                void* dpy = pfnGetCurrentDisplay();
                void* surf = pfnGetCurrentSurface(0x3059); // EGL_DRAW
                if (dpy && surf) {
                    unsigned int ok = pfnSwapBuffers(dpy, surf);
                    LogDebug("eglSwapBuffers(0x%p, 0x%p) returned %u", dpy, surf, ok);
                } else {
                    LogDebug("eglGetCurrentDisplay or Surface was null: dpy=0x%p surf=0x%p", dpy, surf);
                }
            }
        }

        // 2. Try GDI / WGL swap (used when Servo is backed by WGL)
        BOOL gdiOk = SwapBuffers(hdc);
        LogDebug("SwapBuffers(hdc) returned %d", gdiOk);

        EndPaint(m_hwnd, &ps);
        return 0;
    }

    case WM_ERASEBKGND:
        // Return 1 to prevent GDI flicker; Servo paints entire region
        return 1;

    case WM_SIZE: {
        uint32_t w = LOWORD(lParam);
        uint32_t h = HIWORD(lParam);
        if (m_webview && w > 0 && h > 0) {
            servo_webview_resize(m_webview, w, h);
        }
        return 0;
    }

    case WM_SETFOCUS:
        if (m_webview) {
            servo_webview_set_focused(m_webview, true);
        }
        return 0;

    case WM_KILLFOCUS:
        if (m_webview) {
            servo_webview_set_focused(m_webview, false);
        }
        return 0;

    case WM_SETCURSOR:
        if (LOWORD(lParam) == HTCLIENT && m_currentCursor) {
            ::SetCursor(m_currentCursor);
            return TRUE;
        }
        break;

    case WM_MOUSEMOVE: {
        int x = GET_X_LPARAM(lParam);
        int y = GET_Y_LPARAM(lParam);
        if (!m_trackingMouse) {
            TRACKMOUSEEVENT tme = {};
            tme.cbSize = sizeof(TRACKMOUSEEVENT);
            tme.dwFlags = TME_LEAVE;
            tme.hwndTrack = m_hwnd;
            TrackMouseEvent(&tme);
            m_trackingMouse = true;
        }
        if (m_webview) {
            servo_webview_notify_mouse_move_event(m_webview, x, y, GetServoModifiers());
        }
        return 0;
    }

    case WM_MOUSELEAVE:
        m_trackingMouse = false;
        if (m_webview) {
            servo_webview_notify_mouse_leave_event(m_webview, false);
        }
        return 0;

    case WM_LBUTTONDOWN:
    case WM_RBUTTONDOWN:
    case WM_MBUTTONDOWN: {
        ::SetFocus(m_hwnd);
        int x = GET_X_LPARAM(lParam);
        int y = GET_Y_LPARAM(lParam);
        int32_t button = (msg == WM_LBUTTONDOWN) ? SERVO_MOUSE_BUTTON_PRIMARY
                       : (msg == WM_RBUTTONDOWN) ? SERVO_MOUSE_BUTTON_SECONDARY
                       : SERVO_MOUSE_BUTTON_AUXILIARY;
        if (m_webview) {
            servo_webview_notify_mouse_button_event(
                m_webview, SERVO_MOUSE_EVENT_DOWN, button, x, y, GetServoModifiers()
            );
        }
        SetCapture(m_hwnd);
        return 0;
    }

    case WM_LBUTTONUP:
    case WM_RBUTTONUP:
    case WM_MBUTTONUP: {
        int x = GET_X_LPARAM(lParam);
        int y = GET_Y_LPARAM(lParam);
        int32_t button = (msg == WM_LBUTTONUP) ? SERVO_MOUSE_BUTTON_PRIMARY
                       : (msg == WM_RBUTTONUP) ? SERVO_MOUSE_BUTTON_SECONDARY
                       : SERVO_MOUSE_BUTTON_AUXILIARY;
        if (m_webview) {
            servo_webview_notify_mouse_button_event(
                m_webview, SERVO_MOUSE_EVENT_UP, button, x, y, GetServoModifiers()
            );
        }
        ReleaseCapture();
        return 0;
    }

    case WM_MOUSEWHEEL: {
        short delta = GET_WHEEL_DELTA_WPARAM(wParam);
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        ScreenToClient(m_hwnd, &pt);
        if (m_webview) {
            servo_webview_notify_wheel_event(
                m_webview, 0, -delta, 0, SERVO_WHEEL_DELTA_LINE, pt.x, pt.y
            );
        }
        return 0;
    }

    case WM_KEYDOWN:
    case WM_SYSKEYDOWN: {
        bool ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
        bool alt = (GetKeyState(VK_MENU) & 0x8000) != 0;

        // Forward global browser shortcuts up to top-level window
        if (ctrl || alt || wParam == VK_F5 || wParam == VK_F6 || wParam == VK_F11 || wParam == VK_F12 || wParam == VK_ESCAPE) {
            SendMessageW(m_parentHwnd, msg, wParam, lParam);
            return 0;
        }

        if (m_webview) {
            char keyStr[2] = { static_cast<char>(wParam & 0x7F), 0 };
            servo_webview_notify_key_event(
                m_webview, SERVO_KEY_EVENT_DOWN, keyStr, "", 0, GetServoModifiers(), false, false
            );
        }
        return 0;
    }

    case WM_KEYUP:
    case WM_SYSKEYUP: {
        if (m_webview) {
            char keyStr[2] = { static_cast<char>(wParam & 0x7F), 0 };
            servo_webview_notify_key_event(
                m_webview, SERVO_KEY_EVENT_UP, keyStr, "", 0, GetServoModifiers(), false, false
            );
        }
        return 0;
    }
    }

    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}

void ServoViewHost::OnNewFrameReady(struct WebView* webview, void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_hwnd) {
        InvalidateRect(self->m_hwnd, nullptr, FALSE);
        UpdateWindow(self->m_hwnd);
    }
}

void ServoViewHost::OnUrlChanged(struct WebView* webview, const char* url, void* userData) {
    LogDebug("ServoViewHost::OnUrlChanged: %s", url ? url : "(null)");
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_browserWindow && url) {
        self->m_browserWindow->OnActiveTabUrlChanged(url);
    }
}

void ServoViewHost::OnTitleChanged(struct WebView* webview, const char* title, void* userData) {
    LogDebug("ServoViewHost::OnTitleChanged: %s", title ? title : "(null)");
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_browserWindow && title) {
        self->m_browserWindow->OnActiveTabTitleChanged(title);
    }
}

void ServoViewHost::OnLoadStatusChanged(struct WebView* webview, int32_t status, void* userData) {
    LogDebug("ServoViewHost::OnLoadStatusChanged: %d", status);
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_browserWindow) {
        self->m_browserWindow->OnActiveTabLoadStatusChanged(status);
    }
}

void ServoViewHost::OnCursorChanged(struct WebView* webview, int32_t cursorType, void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (!self) return;

    LPCWSTR idc = IDC_ARROW;
    switch (cursorType) {
    case 1: idc = IDC_HAND; break;
    case 2: idc = IDC_IBEAM; break;
    case 3: idc = IDC_WAIT; break;
    case 4: idc = IDC_CROSS; break;
    case 5: idc = IDC_SIZENWSE; break;
    case 6: idc = IDC_SIZENESW; break;
    case 7: idc = IDC_SIZEWE; break;
    case 8: idc = IDC_SIZENS; break;
    case 9: idc = IDC_SIZEALL; break;
    case 10: idc = IDC_NO; break;
    default: idc = IDC_ARROW; break;
    }
    self->m_currentCursor = LoadCursorW(nullptr, idc);
    ::SetCursor(self->m_currentCursor);
}

void ServoViewHost::OnAlertDialog(struct WebView* webview, const char* message, void (*callback)(bool, void*), void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    int len = MultiByteToWideChar(CP_UTF8, 0, message ? message : "", -1, nullptr, 0);
    std::vector<wchar_t> wmsg(len);
    MultiByteToWideChar(CP_UTF8, 0, message ? message : "", -1, wmsg.data(), len);

    MessageBoxW(self ? self->m_hwnd : nullptr, wmsg.data(), L"Alert", MB_OK | MB_ICONINFORMATION);
    if (callback) callback(true, userData);
}

void ServoViewHost::OnConfirmDialog(struct WebView* webview, const char* message, void (*callback)(bool, void*), void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    int len = MultiByteToWideChar(CP_UTF8, 0, message ? message : "", -1, nullptr, 0);
    std::vector<wchar_t> wmsg(len);
    MultiByteToWideChar(CP_UTF8, 0, message ? message : "", -1, wmsg.data(), len);

    int res = MessageBoxW(self ? self->m_hwnd : nullptr, wmsg.data(), L"Confirm", MB_YESNO | MB_ICONQUESTION);
    if (callback) callback(res == IDYES, userData);
}

void ServoViewHost::OnPromptDialog(struct WebView* webview, const char* message, const char* defaultVal, void (*callback)(const char*, void*), void* userData) {
    // For prompt, return default value
    if (callback) callback(defaultVal ? defaultVal : "", userData);
}

void ServoViewHost::OnFilePicker(struct WebView* webview, bool allowMultiple, void (*callback)(const char* const*, size_t, void*), void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    wchar_t szFile[MAX_PATH * 4] = { 0 };

    OPENFILENAMEW ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = self ? self->m_hwnd : nullptr;
    ofn.lpstrFile = szFile;
    ofn.nMaxFile = sizeof(szFile) / sizeof(wchar_t);
    ofn.lpstrFilter = L"All Files (*.*)\0*.*\0HTML Files (*.html;*.htm)\0*.html;*.htm\0\0";
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | (allowMultiple ? OFN_ALLOWMULTISELECT | OFN_EXPLORER : 0);

    if (GetOpenFileNameW(&ofn)) {
        char utf8Path[MAX_PATH * 3] = { 0 };
        WideCharToMultiByte(CP_UTF8, 0, szFile, -1, utf8Path, sizeof(utf8Path), nullptr, nullptr);
        const char* paths[] = { utf8Path };
        if (callback) callback(paths, 1, userData);
    } else {
        if (callback) callback(nullptr, 0, userData);
    }
}

void ServoViewHost::OnContextMenu(struct WebView* webview, const struct ServoContextMenuParams* params, void (*callback)(int32_t, void*), void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (!self) return;

    HMENU hMenu = CreatePopupMenu();
    AppendMenuW(hMenu, MF_STRING, 1, L"Back\tAlt+Left");
    AppendMenuW(hMenu, MF_STRING, 2, L"Forward\tAlt+Right");
    AppendMenuW(hMenu, MF_STRING, 3, L"Reload\tF5");
    AppendMenuW(hMenu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(hMenu, MF_STRING, 4, L"Copy\tCtrl+C");
    AppendMenuW(hMenu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(hMenu, MF_STRING, 5, L"Inspect Element\tF12");

    POINT pt;
    GetCursorPos(&pt);
    int cmd = TrackPopupMenuEx(hMenu, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, self->m_hwnd, nullptr);
    DestroyMenu(hMenu);

    int32_t action = -1; // Dismiss
    switch (cmd) {
    case 1: action = 0; break; // GoBack
    case 2: action = 1; break; // GoForward
    case 3: action = 2; break; // Reload
    case 4: action = 8; break; // Copy
    case 5:
        if (self->m_browserWindow) {
            self->m_browserWindow->OpenDevTools();
        }
        break;
    }
    if (callback) callback(action, userData);
}

void ServoViewHost::OnBeforeDownload(struct WebView* webview, struct ServoDownloadItem* download, void (*callback)(bool, void*), void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_browserWindow) {
        self->m_browserWindow->HandleNewDownload(download);
    }
    if (callback) callback(true, userData);
}

void ServoViewHost::OnDownloadUpdated(struct WebView* webview, struct ServoDownloadItem* download, void* userData) {
    auto self = reinterpret_cast<ServoViewHost*>(userData);
    if (self && self->m_browserWindow) {
        self->m_browserWindow->HandleDownloadUpdated(download);
    }
}

