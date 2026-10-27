/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "omnibox.h"
#include "browser_window.h"
#include "fluent_theme.h"
#include <commctrl.h>
#include <windowsx.h>
#include <vector>

static const wchar_t* OMNIBOX_CLASS_NAME = L"ServoOmniboxToolbarWindow";
static const UINT_PTR EDIT_SUBCLASS_ID = 1001;

void OmniboxToolbar::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = OmniboxToolbar::WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = nullptr;
    wc.lpszClassName = OMNIBOX_CLASS_NAME;
    RegisterClassExW(&wc);
}

OmniboxToolbar::OmniboxToolbar(HWND parentHwnd, BrowserWindow* browserWindow)
    : m_hwnd(nullptr)
    , m_browserWindow(browserWindow)
    , m_editHwnd(nullptr)
    , m_canBack(false)
    , m_canForward(false)
    , m_isLoading(false)
    , m_isDark(true)
{
    HINSTANCE hInstance = GetModuleHandleW(nullptr);
    m_hwnd = CreateWindowExW(
        0,
        OMNIBOX_CLASS_NAME,
        L"",
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 800, 44,
        parentHwnd,
        nullptr,
        hInstance,
        this
    );

    // Create Edit control for address bar
    m_editHwnd = CreateWindowExW(
        0,
        L"EDIT",
        L"",
        WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL | ES_LEFT,
        0, 0, 400, 24,
        m_hwnd,
        nullptr,
        hInstance,
        nullptr
    );

    SetWindowSubclass(m_editHwnd, EditSubclassProc, EDIT_SUBCLASS_ID, reinterpret_cast<DWORD_PTR>(this));

    m_font = fluent::CreateFluentFont(15);
    m_iconFont = fluent::CreateFluentIconFont(14);

    SendMessageW(m_editHwnd, WM_SETFONT, (WPARAM)m_font, TRUE);
}

OmniboxToolbar::~OmniboxToolbar() {
    if (m_editHwnd && IsWindow(m_editHwnd)) {
        RemoveWindowSubclass(m_editHwnd, EditSubclassProc, EDIT_SUBCLASS_ID);
    }
    if (m_font) DeleteObject(m_font);
    if (m_iconFont) DeleteObject(m_iconFont);
    if (m_hwnd && IsWindow(m_hwnd)) {
        DestroyWindow(m_hwnd);
    }
}

void OmniboxToolbar::SetBounds(int x, int y, int width, int height) {
    if (m_hwnd) {
        SetWindowPos(m_hwnd, nullptr, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
        LayoutControls();
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

void OmniboxToolbar::SetUrl(const std::string& url) {
    if (!m_editHwnd) return;
    int len = MultiByteToWideChar(CP_UTF8, 0, url.c_str(), -1, nullptr, 0);
    if (len > 0) {
        std::vector<wchar_t> wbuf(len);
        MultiByteToWideChar(CP_UTF8, 0, url.c_str(), -1, wbuf.data(), len);
        SetWindowTextW(m_editHwnd, wbuf.data());
    }
}

void OmniboxToolbar::SetLoading(bool loading) {
    m_isLoading = loading;
    InvalidateRect(m_hwnd, &m_reloadBtnBounds, FALSE);
}

void OmniboxToolbar::UpdateNavigationState(bool canBack, bool canForward) {
    m_canBack = canBack;
    m_canForward = canForward;
    InvalidateRect(m_hwnd, &m_backBtnBounds, FALSE);
    InvalidateRect(m_hwnd, &m_forwardBtnBounds, FALSE);
}

void OmniboxToolbar::LayoutControls() {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    int btnSize = 32;
    int pad = 6;
    int x = 8;
    int y = (rc.bottom - btnSize) / 2;

    m_backBtnBounds = { x, y, x + btnSize, y + btnSize };
    x += btnSize + pad;

    m_forwardBtnBounds = { x, y, x + btnSize, y + btnSize };
    x += btnSize + pad;

    m_reloadBtnBounds = { x, y, x + btnSize, y + btnSize };
    x += btnSize + pad;

    m_homeBtnBounds = { x, y, x + btnSize, y + btnSize };
    x += btnSize + pad + 4;

    int editWidth = rc.right - x - 12;
    m_omniboxBounds = { x, y, x + editWidth, y + btnSize };

    if (m_editHwnd) {
        // Space 32px on the left for the HTTPS lock icon inside the omnibox
        SetWindowPos(m_editHwnd, nullptr, x + 32, y + 5, editWidth - 40, btnSize - 10, SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

void OmniboxToolbar::Paint(HDC hdc) {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    HDC memDC = CreateCompatibleDC(hdc);
    HBITMAP memBM = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    HBITMAP oldBM = (HBITMAP)SelectObject(memDC, memBM);

    auto colors = m_isDark ? fluent::GetDarkColors() : fluent::GetLightColors();

    // Toolbar background matching Mica Alt blend
    HBRUSH bgBrush = CreateSolidBrush(colors.LayerBackground);
    FillRect(memDC, &rc, bgBrush);
    DeleteObject(bgBrush);

    HFONT oldFont = (HFONT)SelectObject(memDC, m_iconFont);
    SetBkMode(memDC, TRANSPARENT);

    auto drawNavButton = [&](const RECT& bounds, const wchar_t* icon, bool enabled) {
        SetTextColor(memDC, enabled ? colors.TextPrimary : colors.TextTertiary);
        RECT r = bounds;
        DrawTextW(memDC, icon, -1, &r, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    };

    drawNavButton(m_backBtnBounds, fluent::icons::Back, m_canBack);
    drawNavButton(m_forwardBtnBounds, fluent::icons::Forward, m_canForward);
    drawNavButton(m_reloadBtnBounds, m_isLoading ? fluent::icons::Stop : fluent::icons::Refresh, true);
    drawNavButton(m_homeBtnBounds, fluent::icons::Home, true);

    // Omnibox frame with Fluent rounded corners
    HBRUSH boxBrush = CreateSolidBrush(colors.ControlBackground);
    HPEN boxPen = CreatePen(PS_SOLID, 1, colors.ControlBorder);
    HBRUSH oldB = (HBRUSH)SelectObject(memDC, boxBrush);
    HPEN oldP = (HPEN)SelectObject(memDC, boxPen);

    RoundRect(memDC, m_omniboxBounds.left, m_omniboxBounds.top, m_omniboxBounds.right, m_omniboxBounds.bottom, 8, 8);

    SelectObject(memDC, oldB);
    SelectObject(memDC, oldP);
    DeleteObject(boxBrush);
    DeleteObject(boxPen);

    // Draw HTTPS Security Lock Icon
    SetTextColor(memDC, colors.TextSecondary);
    RECT lockRect = { m_omniboxBounds.left + 8, m_omniboxBounds.top, m_omniboxBounds.left + 26, m_omniboxBounds.bottom };
    DrawTextW(memDC, fluent::icons::Lock, -1, &lockRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    BitBlt(hdc, 0, 0, rc.right, rc.bottom, memDC, 0, 0, SRCCOPY);

    SelectObject(memDC, oldFont);
    SelectObject(memDC, oldBM);
    DeleteObject(memBM);
    DeleteDC(memDC);
}

LRESULT CALLBACK OmniboxToolbar::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    OmniboxToolbar* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<OmniboxToolbar*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<OmniboxToolbar*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT OmniboxToolbar::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC hdc = BeginPaint(m_hwnd, &ps);
        Paint(hdc);
        EndPaint(m_hwnd, &ps);
        return 0;
    }

    case WM_ERASEBKGND:
        return 1;

    case WM_CTLCOLOREDIT: {
        HDC hdc = (HDC)wParam;
        auto colors = m_isDark ? fluent::GetDarkColors() : fluent::GetLightColors();
        SetTextColor(hdc, colors.TextPrimary);
        SetBkColor(hdc, colors.ControlBackground);
        static HBRUSH s_darkBrush = CreateSolidBrush(fluent::GetDarkColors().ControlBackground);
        static HBRUSH s_lightBrush = CreateSolidBrush(fluent::GetLightColors().ControlBackground);
        return (LRESULT)(m_isDark ? s_darkBrush : s_lightBrush);
    }

    case WM_LBUTTONDOWN: {
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        if (PtInRect(&m_backBtnBounds, pt) && m_canBack) {
            m_browserWindow->NavigateBack();
            return 0;
        }
        if (PtInRect(&m_forwardBtnBounds, pt) && m_canForward) {
            m_browserWindow->NavigateForward();
            return 0;
        }
        if (PtInRect(&m_reloadBtnBounds, pt)) {
            if (m_isLoading) {
                m_browserWindow->NavigateStop();
            } else {
                m_browserWindow->NavigateReload();
            }
            return 0;
        }
        if (PtInRect(&m_homeBtnBounds, pt)) {
            m_browserWindow->NavigateUrl("https://servo.org");
            return 0;
        }
        return 0;
    }
    }

    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}

LRESULT CALLBACK OmniboxToolbar::EditSubclassProc(
    HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam,
    UINT_PTR uIdSubclass, DWORD_PTR dwRefData)
{
    auto self = reinterpret_cast<OmniboxToolbar*>(dwRefData);

    if (msg == WM_KEYDOWN && wParam == VK_RETURN) {
        int len = GetWindowTextLengthW(hwnd);
        if (len > 0) {
            std::vector<wchar_t> wbuf(len + 1);
            GetWindowTextW(hwnd, wbuf.data(), len + 1);

            int utf8Len = WideCharToMultiByte(CP_UTF8, 0, wbuf.data(), -1, nullptr, 0, nullptr, nullptr);
            if (utf8Len > 0) {
                std::vector<char> utf8Buf(utf8Len);
                WideCharToMultiByte(CP_UTF8, 0, wbuf.data(), -1, utf8Buf.data(), utf8Len, nullptr, nullptr);
                std::string input(utf8Buf.data());

                // Auto-prefix URL or search
                std::string url;
                if (input.find("://") != std::string::npos || input.rfind("about:", 0) == 0) {
                    url = input;
                } else if (input.find('.') != std::string::npos && input.find(' ') == std::string::npos) {
                    url = "https://" + input;
                } else {
                    url = "https://duckduckgo.com/?q=" + input;
                }

                if (self && self->m_browserWindow) {
                    self->m_browserWindow->NavigateUrl(url);
                }
            }
        }
        return 0;
    }

    return DefSubclassProc(hwnd, msg, wParam, lParam);
}

void OmniboxToolbar::FocusAndSelectAll() {
    if (m_editHwnd) {
        SetFocus(m_editHwnd);
        SendMessageW(m_editHwnd, EM_SETSEL, 0, -1);
    }
}

void OmniboxToolbar::FocusForSearch() {
    if (m_editHwnd) {
        SetFocus(m_editHwnd);
        SetWindowTextW(m_editHwnd, L"? ");
        SendMessageW(m_editHwnd, EM_SETSEL, 2, 2);
    }
}

void OmniboxToolbar::SetDarkMode(bool isDark) {
    m_isDark = isDark;
    if (m_hwnd) {
        InvalidateRect(m_hwnd, nullptr, TRUE);
    }
    if (m_editHwnd) {
        InvalidateRect(m_editHwnd, nullptr, TRUE);
    }
}
