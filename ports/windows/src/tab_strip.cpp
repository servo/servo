/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "tab_strip.h"
#include "browser_window.h"
#include "servo_view_host.h"
#include "fluent_theme.h"
#include <windowsx.h>
#include <algorithm>

static const wchar_t* TAB_STRIP_CLASS_NAME = L"ServoTabStripWindow";
static const int TAB_HEIGHT = 38;
static const int TAB_MAX_WIDTH = 220;
static const int TAB_MIN_WIDTH = 100;
static const int ADD_BTN_WIDTH = 34;

void TabStrip::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = TabStrip::WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = nullptr;
    wc.lpszClassName = TAB_STRIP_CLASS_NAME;
    RegisterClassExW(&wc);
}

TabStrip::TabStrip(HWND parentHwnd, BrowserWindow* browserWindow)
    : m_hwnd(nullptr)
    , m_browserWindow(browserWindow)
    , m_activeIndex(-1)
    , m_nextTabId(1)
    , m_hoveredIndex(-1)
    , m_iconFont(nullptr)
    , m_isDark(true)
{
    HINSTANCE hInstance = GetModuleHandleW(nullptr);
    m_hwnd = CreateWindowExW(
        0,
        TAB_STRIP_CLASS_NAME,
        L"",
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 800, TAB_HEIGHT,
        parentHwnd,
        nullptr,
        hInstance,
        this
    );

    m_font = fluent::CreateFluentFont(13);
    m_iconFont = fluent::CreateFluentIconFont(11);
}

TabStrip::~TabStrip() {
    if (m_font) {
        DeleteObject(m_font);
    }
    if (m_iconFont) {
        DeleteObject(m_iconFont);
    }
    if (m_hwnd && IsWindow(m_hwnd)) {
        DestroyWindow(m_hwnd);
    }
}

void TabStrip::SetBounds(int x, int y, int width, int height) {
    if (m_hwnd) {
        SetWindowPos(m_hwnd, nullptr, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
        LayoutTabs();
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

int TabStrip::AddTab(const std::string& url) {
    auto tab = std::make_unique<BrowserTab>();
    tab->id = m_nextTabId++;
    tab->title = L"New Tab";
    tab->url = url.empty() ? "https://servo.org" : url;

    // Create the ServoViewHost for this tab
    tab->viewHost = std::make_unique<ServoViewHost>(
        m_browserWindow->GetHwnd(),
        m_browserWindow->GetServo(),
        m_browserWindow,
        tab->url
    );

    int newIndex = static_cast<int>(m_tabs.size());
    m_tabs.push_back(std::move(tab));

    SelectTab(newIndex);
    LayoutTabs();
    InvalidateRect(m_hwnd, nullptr, FALSE);
    return newIndex;
}

void TabStrip::CloseTab(int index) {
    if (index < 0 || index >= static_cast<int>(m_tabs.size())) return;

    // Record in history for Ctrl+Shift+T
    m_closedHistory.push_back({ m_tabs[index]->url, m_tabs[index]->title });

    bool wasActive = (index == m_activeIndex);
    m_tabs[index]->viewHost->Show(false);
    m_tabs.erase(m_tabs.begin() + index);

    if (m_tabs.empty()) {
        // Closing last tab closes the browser
        PostMessage(m_browserWindow->GetHwnd(), WM_CLOSE, 0, 0);
        return;
    }

    if (wasActive) {
        int nextActive = std::min(index, static_cast<int>(m_tabs.size()) - 1);
        SelectTab(nextActive);
    } else if (m_activeIndex > index) {
        m_activeIndex--;
    }

    LayoutTabs();
    InvalidateRect(m_hwnd, nullptr, FALSE);
}

void TabStrip::NextTab() {
    if (m_tabs.empty()) return;
    int next = (m_activeIndex + 1) % static_cast<int>(m_tabs.size());
    SelectTab(next);
}

void TabStrip::PreviousTab() {
    if (m_tabs.empty()) return;
    int prev = (m_activeIndex - 1 + static_cast<int>(m_tabs.size())) % static_cast<int>(m_tabs.size());
    SelectTab(prev);
}

void TabStrip::SelectTabByIndex(int index) {
    if (index >= 0 && index < static_cast<int>(m_tabs.size())) {
        SelectTab(index);
    }
}

void TabStrip::ReopenClosedTab() {
    if (!m_closedHistory.empty()) {
        auto lastClosed = m_closedHistory.back();
        m_closedHistory.pop_back();
        AddTab(lastClosed.first);
    }
}

void TabStrip::SetDarkMode(bool isDark) {
    m_isDark = isDark;
    if (m_hwnd) {
        InvalidateRect(m_hwnd, nullptr, TRUE);
    }
}

void TabStrip::SelectTab(int index) {
    if (index < 0 || index >= static_cast<int>(m_tabs.size())) return;

    if (m_activeIndex >= 0 && m_activeIndex < static_cast<int>(m_tabs.size())) {
        m_tabs[m_activeIndex]->viewHost->Show(false);
    }

    m_activeIndex = index;
    auto* activeTab = m_tabs[m_activeIndex].get();
    activeTab->viewHost->Show(true);
    activeTab->viewHost->SetFocus();

    m_browserWindow->UpdateActiveTabUI(activeTab->url, activeTab->title);
    InvalidateRect(m_hwnd, nullptr, FALSE);
}

BrowserTab* TabStrip::GetActiveTab() {
    if (m_activeIndex >= 0 && m_activeIndex < static_cast<int>(m_tabs.size())) {
        return m_tabs[m_activeIndex].get();
    }
    return nullptr;
}

void TabStrip::UpdateActiveTitle(const std::wstring& title) {
    auto* tab = GetActiveTab();
    if (tab) {
        tab->title = title.empty() ? L"Untitled" : title;
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

void TabStrip::UpdateActiveUrl(const std::string& url) {
    auto* tab = GetActiveTab();
    if (tab) {
        tab->url = url;
    }
}

void TabStrip::LayoutTabs() {
    RECT rc;
    GetClientRect(m_hwnd, &rc);
    int totalWidth = rc.right - 8; // Leave margin for add button
    int count = static_cast<int>(m_tabs.size());
    if (count == 0) return;

    int tabWidth = (totalWidth - ADD_BTN_WIDTH) / count;
    tabWidth = std::clamp(tabWidth, TAB_MIN_WIDTH, TAB_MAX_WIDTH);

    int curX = 8;
    for (int i = 0; i < count; ++i) {
        m_tabs[i]->bounds = { curX, 4, curX + tabWidth - 2, TAB_HEIGHT };
        // Close button at right edge of tab
        m_tabs[i]->closeButtonBounds = {
            m_tabs[i]->bounds.right - 24, 10,
            m_tabs[i]->bounds.right - 6, 28
        };
        curX += tabWidth;
    }

    m_addTabButtonBounds = { curX + 4, 6, curX + 4 + ADD_BTN_WIDTH - 8, TAB_HEIGHT - 6 };
}

void TabStrip::Paint(HDC hdc) {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    // Double buffering
    HDC memDC = CreateCompatibleDC(hdc);
    HBITMAP memBM = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    HBITMAP oldBM = (HBITMAP)SelectObject(memDC, memBM);

    auto colors = m_isDark ? fluent::GetDarkColors() : fluent::GetLightColors();

    // Fill background with Mica-blend color
    HBRUSH bgBrush = CreateSolidBrush(colors.LayerBackground);
    FillRect(memDC, &rc, bgBrush);
    DeleteObject(bgBrush);

    HFONT oldFont = (HFONT)SelectObject(memDC, m_font);
    SetBkMode(memDC, TRANSPARENT);

    // Render Tabs with WinUI 3 TabView geometry
    for (size_t i = 0; i < m_tabs.size(); ++i) {
        bool isActive = (static_cast<int>(i) == m_activeIndex);
        bool isHovered = (static_cast<int>(i) == m_hoveredIndex);
        const auto& tab = m_tabs[i];

        // Tab surface
        COLORREF tabBg = isActive ? colors.TabBackgroundActive
                                  : (isHovered ? colors.TabBackgroundHover : colors.TabBackgroundInactive);

        HBRUSH tabBrush = CreateSolidBrush(tabBg);
        HPEN tabPen = CreatePen(PS_SOLID, 1, (isActive || isHovered) ? colors.TabBorder : tabBg);
        HBRUSH oldB = (HBRUSH)SelectObject(memDC, tabBrush);
        HPEN oldP = (HPEN)SelectObject(memDC, tabPen);

        // WinUI 3 TabView shape: rounded top corners (6px radius), extending below client bottom
        RoundRect(memDC, tab->bounds.left, tab->bounds.top, tab->bounds.right, tab->bounds.bottom + 6, 6, 6);

        SelectObject(memDC, oldB);
        SelectObject(memDC, oldP);
        DeleteObject(tabBrush);
        DeleteObject(tabPen);

        // Active tab top accent indicator line
        if (isActive) {
            HPEN accentPen = CreatePen(PS_SOLID, 2, colors.Accent);
            HPEN oldAP = (HPEN)SelectObject(memDC, accentPen);
            MoveToEx(memDC, tab->bounds.left + 4, tab->bounds.top, nullptr);
            LineTo(memDC, tab->bounds.right - 4, tab->bounds.top);
            SelectObject(memDC, oldAP);
            DeleteObject(accentPen);
        }

        // Tab text
        SetTextColor(memDC, isActive ? colors.TextPrimary : colors.TextSecondary);
        RECT textRect = { tab->bounds.left + 10, tab->bounds.top + 6, tab->closeButtonBounds.left - 4, tab->bounds.bottom - 4 };
        DrawTextW(memDC, tab->title.c_str(), -1, &textRect, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);

        // Close button using Segoe Fluent Icons Close
        SelectObject(memDC, m_iconFont);
        SetTextColor(memDC, isHovered ? colors.TextPrimary : colors.TextTertiary);
        RECT closeRect = tab->closeButtonBounds;
        DrawTextW(memDC, fluent::icons::Close, -1, &closeRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        SelectObject(memDC, m_font);
    }

    // Render Add Tab button '+' using Segoe Fluent Icons AddTab
    SelectObject(memDC, m_iconFont);
    SetTextColor(memDC, colors.TextSecondary);
    RECT addRect = m_addTabButtonBounds;
    DrawTextW(memDC, fluent::icons::AddTab, -1, &addRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

    BitBlt(hdc, 0, 0, rc.right, rc.bottom, memDC, 0, 0, SRCCOPY);

    SelectObject(memDC, oldFont);
    SelectObject(memDC, oldBM);
    DeleteObject(memBM);
    DeleteDC(memDC);
}

LRESULT CALLBACK TabStrip::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    TabStrip* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<TabStrip*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<TabStrip*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT TabStrip::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC hdc = BeginPaint(m_hwnd, &ps);
        Paint(hdc);
        EndPaint(m_hwnd, &ps);
        return 0;
    }

    case WM_MOUSEMOVE: {
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        int newHover = -1;
        for (size_t i = 0; i < m_tabs.size(); ++i) {
            if (PtInRect(&m_tabs[i]->bounds, pt)) {
                newHover = static_cast<int>(i);
                break;
            }
        }
        if (newHover != m_hoveredIndex) {
            m_hoveredIndex = newHover;
            InvalidateRect(m_hwnd, nullptr, FALSE);
            TRACKMOUSEEVENT tme = { sizeof(TRACKMOUSEEVENT), TME_LEAVE, m_hwnd, 0 };
            TrackMouseEvent(&tme);
        }
        return 0;
    }

    case WM_MOUSELEAVE: {
        if (m_hoveredIndex != -1) {
            m_hoveredIndex = -1;
            InvalidateRect(m_hwnd, nullptr, FALSE);
        }
        return 0;
    }

    case WM_LBUTTONDOWN: {
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };

        // Check Add Tab button
        if (PtInRect(&m_addTabButtonBounds, pt)) {
            AddTab("https://servo.org");
            return 0;
        }

        // Check Tabs and Close buttons
        for (size_t i = 0; i < m_tabs.size(); ++i) {
            if (PtInRect(&m_tabs[i]->closeButtonBounds, pt)) {
                CloseTab(static_cast<int>(i));
                return 0;
            }
            if (PtInRect(&m_tabs[i]->bounds, pt)) {
                SelectTab(static_cast<int>(i));
                return 0;
            }
        }
        return 0;
    }

    case WM_MBUTTONUP: {
        // Middle click closes tab
        POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
        for (size_t i = 0; i < m_tabs.size(); ++i) {
            if (PtInRect(&m_tabs[i]->bounds, pt)) {
                CloseTab(static_cast<int>(i));
                return 0;
            }
        }
        return 0;
    }

    case WM_ERASEBKGND:
        return 1;
    }

    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}
