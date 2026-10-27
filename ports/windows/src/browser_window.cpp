/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "browser_window.h"
#include "tab_strip.h"
#include "omnibox.h"
#include "find_bar.h"
#include "download_manager.h"
#include "servo_view_host.h"
#include <windowsx.h>
#include <vector>

#ifndef DWMWA_USE_IMMERSIVE_DARK_MODE
#define DWMWA_USE_IMMERSIVE_DARK_MODE 20
#endif

#ifndef DWMWA_SYSTEMBACKDROP_TYPE
#define DWMWA_SYSTEMBACKDROP_TYPE 38
#endif

#ifndef DWMSBT_TABBEDWINDOW
#define DWMSBT_TABBEDWINDOW 4 // Mica Alt
#endif

static const wchar_t* BROWSER_WINDOW_CLASS_NAME = L"ServoBrowserMainWindow";
static const int TOP_CHROME_HEIGHT = 82; // 38 (Tabs) + 44 (Omnibox)

void BrowserWindow::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = BrowserWindow::WndProc;
    wc.hInstance = hInstance;
    wc.hIcon = LoadIconW(nullptr, IDI_APPLICATION);
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = CreateSolidBrush(RGB(32, 32, 32));
    wc.lpszClassName = BROWSER_WINDOW_CLASS_NAME;
    RegisterClassExW(&wc);
}

BrowserWindow* BrowserWindow::Create(struct Servo* servo, const std::string& initialUrl) {
    HINSTANCE hInstance = GetModuleHandleW(nullptr);
    auto window = new BrowserWindow(servo);

    LogDebug("CreateWindowExW starting for BrowserWindow, hInstance=0x%p...", hInstance);
    HWND hwnd = CreateWindowExW(
        WS_EX_APPWINDOW,
        BROWSER_WINDOW_CLASS_NAME,
        L"Servo Browser",
        WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
        CW_USEDEFAULT, CW_USEDEFAULT, 1200, 800,
        nullptr,
        nullptr,
        hInstance,
        window
    );
    LogDebug("CreateWindowExW for BrowserWindow returned hwnd=0x%p, GetLastError=%lu", hwnd, GetLastError());

    if (!hwnd) {
        DWORD err = GetLastError();
        LogDebug("BrowserWindow creation failed with error %lu", err);
        wchar_t buf[256];
        swprintf_s(buf, L"Failed to create main browser window (CreateWindowExW error %lu).", err);
        MessageBoxW(nullptr, buf, L"Servo Browser Error", MB_ICONERROR);
        delete window;
        return nullptr;
    }

    window->m_hwnd = hwnd;
    LogDebug("Applying Mica Alt theme...");
    window->ApplyMicaAltTheme();

    LogDebug("Creating child UI components...");
    // Create child UI components
    window->m_tabStrip = std::make_unique<TabStrip>(hwnd, window);
    window->m_omnibox = std::make_unique<OmniboxToolbar>(hwnd, window);
    window->m_findBar = std::make_unique<FindBar>(hwnd, window);
    window->m_downloadManager = std::make_unique<DownloadManager>(hwnd);

    LogDebug("Adding initial tab...");
    // Create initial tab
    window->m_tabStrip->AddTab(initialUrl);
    window->LayoutChildren();

    LogDebug("Showing window...");
    ShowWindow(hwnd, SW_SHOWNORMAL);
    UpdateWindow(hwnd);
    SetForegroundWindow(hwnd);
    BringWindowToTop(hwnd);

    // Continuous 60 FPS tick timer to drive Servo event loop, animations and network progress
    SetTimer(hwnd, 1, 16, nullptr);

    LogDebug("BrowserWindow::Create completed successfully.");
    return window;
}

BrowserWindow::BrowserWindow(struct Servo* servo)
    : m_hwnd(nullptr)
    , m_servo(servo)
    , m_devtoolsPort(0)
    , m_isDark(true)
    , m_isFullscreen(false)
    , m_savedPlacement{ sizeof(WINDOWPLACEMENT) }
{
}

BrowserWindow::~BrowserWindow() = default;

void BrowserWindow::ApplyMicaAltTheme() {
    LogDebug("Mica Alt is currently disabled for testing.");
    // Disabled Mica Alt for testing
    /*
    if (!m_hwnd) return;

    // 1. Enable Windows 11 Mica Alt (DWMSBT_TABBEDWINDOW = 4)
    int backdrop = DWMSBT_TABBEDWINDOW;
    DwmSetWindowAttribute(m_hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &backdrop, sizeof(backdrop));

    // 2. Sync dark mode
    SyncDarkMode();

    // 3. Extend frame into client area for seamless titlebar
    MARGINS margins = { -1 };
    DwmExtendFrameIntoClientArea(m_hwnd, &margins);
    */
}

void BrowserWindow::SyncDarkMode() {
    BOOL darkMode = TRUE; // Default to dark for modern Mica Alt
    DwmSetWindowAttribute(m_hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &darkMode, sizeof(darkMode));
    m_isDark = darkMode != FALSE;

    if (m_tabStrip) {
        m_tabStrip->SetDarkMode(m_isDark);
    }
    if (m_omnibox) {
        m_omnibox->SetDarkMode(m_isDark);
    }

    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        servo_webview_notify_theme_change(
            tab->viewHost->GetWebView(),
            m_isDark ? SERVO_THEME_DARK : SERVO_THEME_LIGHT
        );
    }
}

void BrowserWindow::LayoutChildren() {
    if (!m_hwnd) return;

    RECT rc;
    GetClientRect(m_hwnd, &rc);
    int width = rc.right;
    int height = rc.bottom;

    if (m_tabStrip) {
        m_tabStrip->SetBounds(0, 0, width, 38);
    }

    if (m_omnibox) {
        m_omnibox->SetBounds(0, 38, width, 44);
    }

    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        int contentY = TOP_CHROME_HEIGHT;
        int contentH = std::max(0, height - contentY);
        tab->viewHost->SetBounds(0, contentY, width, contentH);
    }

    if (m_findBar) {
        m_findBar->SetBounds(width - 340, 88, 320, 42);
    }

    if (m_downloadManager) {
        m_downloadManager->SetBounds(width - 380, 88, 360, 220);
    }
}

void BrowserWindow::OpenDevTools() {
    if (m_devtoolsPort > 0) {
        std::string devtoolsUrl = "http://localhost:" + std::to_string(m_devtoolsPort);
        if (!m_devtoolsToken.empty()) {
            devtoolsUrl += "/?token=" + m_devtoolsToken;
        }
        if (m_tabStrip) {
            m_tabStrip->AddTab(devtoolsUrl);
        }
    } else {
        MessageBoxW(m_hwnd, L"DevTools server is initializing or listening on local port.", L"DevTools", MB_OK | MB_ICONINFORMATION);
    }
}

void BrowserWindow::SetDevToolsInfo(uint16_t port, const std::string& token) {
    m_devtoolsPort = port;
    m_devtoolsToken = token;
}

void BrowserWindow::HandleNewDownload(struct ServoDownloadItem* download) {
    if (!m_downloadManager || !download) return;
    char urlBuf[1024] = { 0 };
    servo_download_item_get_url(download, urlBuf, sizeof(urlBuf));

    char nameBuf[256] = { 0 };
    servo_download_item_get_suggested_filename(download, nameBuf, sizeof(nameBuf));

    int len = MultiByteToWideChar(CP_UTF8, 0, nameBuf, -1, nullptr, 0);
    std::vector<wchar_t> wname(len);
    MultiByteToWideChar(CP_UTF8, 0, nameBuf, -1, wname.data(), len);

    static uint64_t s_dlId = 1;
    m_downloadManager->AddDownload(s_dlId++, urlBuf, wname.data());
}

void BrowserWindow::HandleDownloadUpdated(struct ServoDownloadItem* download) {
    if (!m_downloadManager || !download) return;
    uint64_t recv = servo_download_item_get_received_bytes(download);
    uint64_t total = servo_download_item_get_total_bytes(download);
    int state = servo_download_item_get_state(download);
    m_downloadManager->UpdateDownload(1, recv, total, state);
}

void BrowserWindow::NavigateUrl(const std::string& url) {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        tab->viewHost->LoadUrl(url);
    }
}

void BrowserWindow::NavigateBack() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        tab->viewHost->GoBack();
    }
}

void BrowserWindow::NavigateForward() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        tab->viewHost->GoForward();
    }
}

void BrowserWindow::NavigateReload() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        tab->viewHost->Reload();
    }
}

void BrowserWindow::NavigateStop() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        tab->viewHost->Stop();
    }
}

int BrowserWindow::FindInPage(const char* text, bool forward, bool matchCase) {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        return servo_webview_find(tab->viewHost->GetWebView(), text, forward, matchCase);
    }
    return -1;
}

int BrowserWindow::FindNext(int identifier, bool forward) {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        return servo_webview_find_next(tab->viewHost->GetWebView(), identifier, forward);
    }
    return -1;
}

void BrowserWindow::ClearFind(int identifier) {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        servo_webview_find_clear(tab->viewHost->GetWebView(), identifier);
    }
}

void BrowserWindow::ZoomIn() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        float currentZoom = servo_webview_page_zoom(tab->viewHost->GetWebView());
        float nextZoom = std::min(currentZoom * 1.2f, 5.0f);
        servo_webview_set_page_zoom(tab->viewHost->GetWebView(), nextZoom);
    }
}

void BrowserWindow::ZoomOut() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        float currentZoom = servo_webview_page_zoom(tab->viewHost->GetWebView());
        float nextZoom = std::max(currentZoom / 1.2f, 0.25f);
        servo_webview_set_page_zoom(tab->viewHost->GetWebView(), nextZoom);
    }
}

void BrowserWindow::ZoomReset() {
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && tab->viewHost->GetWebView()) {
        servo_webview_set_page_zoom(tab->viewHost->GetWebView(), 1.0f);
    }
}

void BrowserWindow::ToggleFullscreen() {
    DWORD style = GetWindowLongW(m_hwnd, GWL_STYLE);
    if (!m_isFullscreen) {
        MONITORINFO mi = { sizeof(mi) };
        if (GetWindowPlacement(m_hwnd, &m_savedPlacement) &&
            GetMonitorInfoW(MonitorFromWindow(m_hwnd, MONITOR_DEFAULTTOPRIMARY), &mi)) {
            SetWindowLongW(m_hwnd, GWL_STYLE, style & ~WS_OVERLAPPEDWINDOW);
            SetWindowPos(m_hwnd, HWND_TOP,
                mi.rcMonitor.left, mi.rcMonitor.top,
                mi.rcMonitor.right - mi.rcMonitor.left,
                mi.rcMonitor.bottom - mi.rcMonitor.top,
                SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
            m_isFullscreen = true;
        }
    } else {
        SetWindowLongW(m_hwnd, GWL_STYLE, style | WS_OVERLAPPEDWINDOW);
        SetWindowPlacement(m_hwnd, &m_savedPlacement);
        SetWindowPos(m_hwnd, nullptr, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
        m_isFullscreen = false;
    }
}

void BrowserWindow::OnActiveTabUrlChanged(const char* url) {
    if (m_omnibox && url) {
        m_omnibox->SetUrl(url);
    }
    if (m_tabStrip && url) {
        m_tabStrip->UpdateActiveUrl(url);
    }
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost) {
        m_omnibox->UpdateNavigationState(tab->viewHost->CanGoBack(), tab->viewHost->CanGoForward());
    }
}

void BrowserWindow::OnActiveTabTitleChanged(const char* title) {
    if (!title) return;
    int len = MultiByteToWideChar(CP_UTF8, 0, title, -1, nullptr, 0);
    if (len > 0) {
        std::vector<wchar_t> wbuf(len);
        MultiByteToWideChar(CP_UTF8, 0, title, -1, wbuf.data(), len);
        if (m_tabStrip) {
            m_tabStrip->UpdateActiveTitle(wbuf.data());
        }
        std::wstring windowTitle = std::wstring(wbuf.data()) + L" — Servo";
        SetWindowTextW(m_hwnd, windowTitle.c_str());
    }
}

void BrowserWindow::OnActiveTabLoadStatusChanged(int32_t status) {
    if (m_omnibox) {
        m_omnibox->SetLoading(status == 0); // 0 = In progress
    }
    auto* tab = m_tabStrip ? m_tabStrip->GetActiveTab() : nullptr;
    if (tab && tab->viewHost && m_omnibox) {
        m_omnibox->UpdateNavigationState(tab->viewHost->CanGoBack(), tab->viewHost->CanGoForward());
    }
}

void BrowserWindow::UpdateActiveTabUI(const std::string& url, const std::wstring& title) {
    if (m_omnibox) {
        m_omnibox->SetUrl(url);
    }
    std::wstring windowTitle = title.empty() ? L"Servo Browser" : (title + L" — Servo");
    SetWindowTextW(m_hwnd, windowTitle.c_str());
    LayoutChildren();
}

LRESULT CALLBACK BrowserWindow::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    LogDebug("BrowserWindow::WndProc msg=0x%04X hwnd=0x%p", msg, hwnd);
    BrowserWindow* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<BrowserWindow*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<BrowserWindow*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT BrowserWindow::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_SIZE:
        LayoutChildren();
        return 0;

    case WM_KEYDOWN: {
        bool ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
        bool alt = (GetKeyState(VK_MENU) & 0x8000) != 0;
        bool shift = (GetKeyState(VK_SHIFT) & 0x8000) != 0;

        // --- Tab Management ---
        if (ctrl && shift && wParam == 'T') {
            if (m_tabStrip) m_tabStrip->ReopenClosedTab();
            return 0;
        }
        if (ctrl && (wParam == VK_TAB || wParam == VK_NEXT)) {
            if (m_tabStrip) {
                if (shift) {
                    m_tabStrip->PreviousTab();
                } else {
                    m_tabStrip->NextTab();
                }
            }
            return 0;
        }
        if (ctrl && wParam == VK_PRIOR) {
            if (m_tabStrip) m_tabStrip->PreviousTab();
            return 0;
        }
        if (ctrl && wParam >= '1' && wParam <= '8') {
            if (m_tabStrip) m_tabStrip->SelectTabByIndex(static_cast<int>(wParam - '1'));
            return 0;
        }
        if (ctrl && wParam == '9') {
            if (m_tabStrip && m_tabStrip->GetTabCount() > 0) {
                m_tabStrip->SelectTabByIndex(m_tabStrip->GetTabCount() - 1);
            }
            return 0;
        }
        if (ctrl && wParam == 'T') {
            if (m_tabStrip) m_tabStrip->AddTab("https://servo.org");
            return 0;
        }
        if ((ctrl && wParam == 'W') || (ctrl && wParam == VK_F4)) {
            if (m_tabStrip) m_tabStrip->CloseTab(m_tabStrip->GetActiveIndex());
            return 0;
        }

        // --- Omnibox Focus ---
        if ((ctrl && wParam == 'L') || (alt && wParam == 'D') || wParam == VK_F6) {
            if (m_omnibox) m_omnibox->FocusAndSelectAll();
            return 0;
        }
        if (ctrl && (wParam == 'E' || wParam == 'K')) {
            if (m_omnibox) m_omnibox->FocusForSearch();
            return 0;
        }

        // --- Navigation ---
        if (wParam == VK_ESCAPE) {
            if (m_findBar && m_findBar->IsVisible()) {
                m_findBar->Show(false);
            } else {
                NavigateStop();
            }
            return 0;
        }
        if ((ctrl && shift && wParam == 'R') || (ctrl && wParam == VK_F5)) {
            NavigateReload();
            return 0;
        }
        if ((ctrl && wParam == 'R') || wParam == VK_F5) {
            NavigateReload();
            return 0;
        }
        if (alt && wParam == VK_HOME) {
            NavigateUrl("https://servo.org");
            return 0;
        }
        if (alt && wParam == VK_LEFT) {
            NavigateBack();
            return 0;
        }
        if (alt && wParam == VK_RIGHT) {
            NavigateForward();
            return 0;
        }

        // --- Tools & Overlays ---
        if (ctrl && wParam == 'F') {
            if (m_findBar) m_findBar->Show(!m_findBar->IsVisible());
            return 0;
        }
        if (ctrl && wParam == 'J') {
            if (m_downloadManager) m_downloadManager->Show(!m_downloadManager->IsVisible());
            return 0;
        }
        if (wParam == VK_F12 || (ctrl && shift && wParam == 'I')) {
            OpenDevTools();
            return 0;
        }
        if (wParam == VK_F11) {
            ToggleFullscreen();
            return 0;
        }

        // --- Zoom Controls ---
        if (ctrl && (wParam == VK_OEM_PLUS || wParam == VK_ADD || wParam == '=')) {
            ZoomIn();
            return 0;
        }
        if (ctrl && (wParam == VK_OEM_MINUS || wParam == VK_SUBTRACT || wParam == '-')) {
            ZoomOut();
            return 0;
        }
        if (ctrl && (wParam == '0' || wParam == VK_NUMPAD0)) {
            ZoomReset();
            return 0;
        }

        break;
    }

    case WM_NCCALCSIZE: {
        // Mica Alt disabled: use default non-client framing
        break;
    }

    case WM_NCHITTEST: {
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        ScreenToClient(m_hwnd, &pt);

        // Top draggable header area outside tabs
        if (pt.y < 38) {
            RECT rc;
            GetClientRect(m_hwnd, &rc);
            // Caption buttons on right side
            if (pt.x > rc.right - 140) {
                return DefWindowProcW(m_hwnd, msg, wParam, lParam);
            }
            return HTCAPTION;
        }
        break;
    }

    case WM_CLOSE:
        LogDebug("BrowserWindow received WM_CLOSE");
        break;

    case WM_DESTROY:
        LogDebug("BrowserWindow received WM_DESTROY, posting quit message...");
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}
