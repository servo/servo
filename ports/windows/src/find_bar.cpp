/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "find_bar.h"
#include "browser_window.h"
#include <commctrl.h>
#include <windowsx.h>
#include <vector>

static const wchar_t* FIND_BAR_CLASS_NAME = L"ServoFindBarWindow";
static const UINT_PTR FIND_EDIT_SUBCLASS_ID = 1002;

void FindBar::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = FindBar::WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = nullptr;
    wc.lpszClassName = FIND_BAR_CLASS_NAME;
    RegisterClassExW(&wc);
}

FindBar::FindBar(HWND parentHwnd, BrowserWindow* browserWindow)
    : m_hwnd(nullptr)
    , m_browserWindow(browserWindow)
    , m_editHwnd(nullptr)
    , m_findIdentifier(0)
    , m_isDark(true)
{
    HINSTANCE hInstance = GetModuleHandleW(nullptr);
    m_hwnd = CreateWindowExW(
        WS_EX_TOPMOST,
        FIND_BAR_CLASS_NAME,
        L"",
        WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 320, 42,
        parentHwnd,
        nullptr,
        hInstance,
        this
    );

    m_editHwnd = CreateWindowExW(
        0,
        L"EDIT",
        L"",
        WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL | ES_LEFT,
        8, 8, 160, 24,
        m_hwnd,
        nullptr,
        hInstance,
        nullptr
    );

    SetWindowSubclass(m_editHwnd, EditSubclassProc, FIND_EDIT_SUBCLASS_ID, reinterpret_cast<DWORD_PTR>(this));

    NONCLIENTMETRICSW ncm = { sizeof(NONCLIENTMETRICSW) };
    SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, sizeof(ncm), &ncm, 0);
    m_font = CreateFontIndirectW(&ncm.lfMessageFont);
    SendMessageW(m_editHwnd, WM_SETFONT, (WPARAM)m_font, TRUE);
}

FindBar::~FindBar() {
    if (m_editHwnd && IsWindow(m_editHwnd)) {
        RemoveWindowSubclass(m_editHwnd, EditSubclassProc, FIND_EDIT_SUBCLASS_ID);
    }
    if (m_font) DeleteObject(m_font);
    if (m_hwnd && IsWindow(m_hwnd)) {
        DestroyWindow(m_hwnd);
    }
}

void FindBar::Show(bool show) {
    if (m_hwnd) {
        ShowWindow(m_hwnd, show ? SW_SHOW : SW_HIDE);
        if (show) {
            ::SetFocus(m_editHwnd);
            SendMessageW(m_editHwnd, EM_SETSEL, 0, -1);
        } else {
            if (m_findIdentifier > 0) {
                m_browserWindow->ClearFind(m_findIdentifier);
                m_findIdentifier = 0;
            }
            m_statusText.clear();
        }
    }
}

bool FindBar::IsVisible() const {
    return m_hwnd ? (IsWindowVisible(m_hwnd) != FALSE) : false;
}

void FindBar::SetBounds(int x, int y, int width, int height) {
    if (m_hwnd) {
        SetWindowPos(m_hwnd, nullptr, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
        LayoutControls();
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

void FindBar::UpdateMatchResult(int activeOrdinal, int totalMatches) {
    if (totalMatches > 0) {
        m_statusText = std::to_wstring(activeOrdinal) + L" of " + std::to_wstring(totalMatches);
    } else {
        m_statusText = L"0 of 0";
    }
    InvalidateRect(m_hwnd, &m_statusBounds, FALSE);
}

void FindBar::LayoutControls() {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    int btnSize = 26;
    int pad = 4;
    int x = rc.right - btnSize - 6;
    int y = (rc.bottom - btnSize) / 2;

    m_closeBtnBounds = { x, y, x + btnSize, y + btnSize };
    x -= btnSize + pad;

    m_nextBtnBounds = { x, y, x + btnSize, y + btnSize };
    x -= btnSize + pad;

    m_prevBtnBounds = { x, y, x + btnSize, y + btnSize };

    m_statusBounds = { x - 70, y, x - 4, y + btnSize };

    int editWidth = m_statusBounds.left - 12;
    if (m_editHwnd) {
        SetWindowPos(m_editHwnd, nullptr, 8, y + 2, editWidth, btnSize - 4, SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

void FindBar::Paint(HDC hdc) {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    HDC memDC = CreateCompatibleDC(hdc);
    HBITMAP memBM = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    HBITMAP oldBM = (HBITMAP)SelectObject(memDC, memBM);

    // Modern floating card background
    HBRUSH bgBrush = CreateSolidBrush(m_isDark ? RGB(45, 45, 45) : RGB(255, 255, 255));
    HPEN borderPen = CreatePen(PS_SOLID, 1, m_isDark ? RGB(70, 70, 70) : RGB(210, 210, 210));
    HBRUSH oldB = (HBRUSH)SelectObject(memDC, bgBrush);
    HPEN oldP = (HPEN)SelectObject(memDC, borderPen);

    RoundRect(memDC, 0, 0, rc.right, rc.bottom, 12, 12);

    SelectObject(memDC, oldB);
    SelectObject(memDC, oldP);
    DeleteObject(bgBrush);
    DeleteObject(borderPen);

    HFONT oldFont = (HFONT)SelectObject(memDC, m_font);
    SetBkMode(memDC, TRANSPARENT);

    // Status text (matches)
    SetTextColor(memDC, m_isDark ? RGB(180, 180, 180) : RGB(100, 100, 100));
    RECT sRect = m_statusBounds;
    DrawTextW(memDC, m_statusText.c_str(), -1, &sRect, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

    // Buttons
    SetTextColor(memDC, m_isDark ? RGB(230, 230, 230) : RGB(40, 40, 40));
    RECT pR = m_prevBtnBounds;
    DrawTextW(memDC, L"∧", -1, &pR, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    RECT nR = m_nextBtnBounds;
    DrawTextW(memDC, L"∨", -1, &nR, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    RECT cR = m_closeBtnBounds;
    DrawTextW(memDC, L"✕", -1, &cR, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    BitBlt(hdc, 0, 0, rc.right, rc.bottom, memDC, 0, 0, SRCCOPY);

    SelectObject(memDC, oldFont);
    SelectObject(memDC, oldBM);
    DeleteObject(memBM);
    DeleteDC(memDC);
}

void FindBar::PerformFind(bool forward) {
    if (!m_editHwnd) return;
    int len = GetWindowTextLengthW(m_editHwnd);
    if (len == 0) {
        if (m_findIdentifier > 0) {
            m_browserWindow->ClearFind(m_findIdentifier);
            m_findIdentifier = 0;
        }
        m_statusText.clear();
        InvalidateRect(m_hwnd, &m_statusBounds, FALSE);
        return;
    }

    std::vector<wchar_t> wbuf(len + 1);
    GetWindowTextW(m_editHwnd, wbuf.data(), len + 1);
    int utf8Len = WideCharToMultiByte(CP_UTF8, 0, wbuf.data(), -1, nullptr, 0, nullptr, nullptr);
    std::vector<char> utf8Buf(utf8Len);
    WideCharToMultiByte(CP_UTF8, 0, wbuf.data(), -1, utf8Buf.data(), utf8Len, nullptr, nullptr);

    if (m_findIdentifier <= 0) {
        m_findIdentifier = m_browserWindow->FindInPage(utf8Buf.data(), forward, false);
    } else {
        m_browserWindow->FindNext(m_findIdentifier, forward);
    }
}

LRESULT CALLBACK FindBar::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    FindBar* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<FindBar*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<FindBar*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT FindBar::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
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
        SetTextColor(hdc, m_isDark ? RGB(255, 255, 255) : RGB(0, 0, 0));
        SetBkColor(hdc, m_isDark ? RGB(45, 45, 45) : RGB(255, 255, 255));
        static HBRUSH s_editBrush = CreateSolidBrush(m_isDark ? RGB(45, 45, 45) : RGB(255, 255, 255));
        return (LRESULT)s_editBrush;
    }

    case WM_LBUTTONDOWN: {
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        if (PtInRect(&m_prevBtnBounds, pt)) {
            PerformFind(false);
            return 0;
        }
        if (PtInRect(&m_nextBtnBounds, pt)) {
            PerformFind(true);
            return 0;
        }
        if (PtInRect(&m_closeBtnBounds, pt)) {
            Show(false);
            return 0;
        }
        return 0;
    }
    }

    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}

LRESULT CALLBACK FindBar::EditSubclassProc(
    HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam,
    UINT_PTR uIdSubclass, DWORD_PTR dwRefData)
{
    auto self = reinterpret_cast<FindBar*>(dwRefData);

    if (msg == WM_KEYDOWN) {
        if (wParam == VK_ESCAPE) {
            if (self) self->Show(false);
            return 0;
        }
        if (wParam == VK_RETURN) {
            bool shift = (GetKeyState(VK_SHIFT) & 0x8000) != 0;
            if (self) self->PerformFind(!shift);
            return 0;
        }
    }

    if (msg == WM_KEYUP && wParam != VK_RETURN && wParam != VK_ESCAPE) {
        if (self) {
            self->m_findIdentifier = 0; // Reset search query on character change
            self->PerformFind(true);
        }
    }

    return DefSubclassProc(hwnd, msg, wParam, lParam);
}
