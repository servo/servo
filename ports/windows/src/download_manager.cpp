/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "download_manager.h"
#include <windowsx.h>
#include <commctrl.h>
#include <shellapi.h>
#include <algorithm>

static const wchar_t* DOWNLOAD_MGR_CLASS_NAME = L"ServoDownloadManagerWindow";

void DownloadManager::RegisterWindowClass(HINSTANCE hInstance) {
    if (!hInstance) {
        hInstance = GetModuleHandleW(nullptr);
    }
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = DownloadManager::WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, MAKEINTRESOURCEW(32512));
    wc.hbrBackground = nullptr;
    wc.lpszClassName = DOWNLOAD_MGR_CLASS_NAME;
    RegisterClassExW(&wc);
}

DownloadManager::DownloadManager(HWND parentHwnd)
    : m_hwnd(nullptr)
    , m_parentHwnd(parentHwnd)
    , m_isDark(true)
{
    HINSTANCE hInstance = GetModuleHandleW(nullptr);
    m_hwnd = CreateWindowExW(
        WS_EX_TOPMOST,
        DOWNLOAD_MGR_CLASS_NAME,
        L"",
        WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 360, 240,
        parentHwnd,
        nullptr,
        hInstance,
        this
    );

    NONCLIENTMETRICSW ncm = { sizeof(NONCLIENTMETRICSW) };
    SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, sizeof(ncm), &ncm, 0);
    m_font = CreateFontIndirectW(&ncm.lfMessageFont);

    LOGFONTW boldLf = ncm.lfMessageFont;
    boldLf.lfWeight = FW_BOLD;
    m_boldFont = CreateFontIndirectW(&boldLf);
}

DownloadManager::~DownloadManager() {
    if (m_font) DeleteObject(m_font);
    if (m_boldFont) DeleteObject(m_boldFont);
    if (m_hwnd && IsWindow(m_hwnd)) {
        DestroyWindow(m_hwnd);
    }
}

void DownloadManager::Show(bool show) {
    if (m_hwnd) {
        ShowWindow(m_hwnd, show ? SW_SHOW : SW_HIDE);
        if (show) InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

bool DownloadManager::IsVisible() const {
    return m_hwnd ? (IsWindowVisible(m_hwnd) != FALSE) : false;
}

void DownloadManager::SetBounds(int x, int y, int width, int height) {
    if (m_hwnd) {
        SetWindowPos(m_hwnd, nullptr, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

void DownloadManager::AddDownload(uint64_t id, const std::string& url, const std::wstring& filename) {
    ActiveDownload item = {};
    item.id = id;
    item.url = url;
    item.filename = filename.empty() ? L"download" : filename;
    item.receivedBytes = 0;
    item.totalBytes = 0;
    item.state = 0; // InProgress

    // Default download directory: Downloads folder
    wchar_t downloadPath[MAX_PATH];
    if (GetEnvironmentVariableW(L"USERPROFILE", downloadPath, MAX_PATH) > 0) {
        item.localPath = std::wstring(downloadPath) + L"\\Downloads\\" + item.filename;
    }

    m_downloads.insert(m_downloads.begin(), item);
    Show(true);
    InvalidateRect(m_hwnd, nullptr, FALSE);
}

void DownloadManager::UpdateDownload(uint64_t id, uint64_t receivedBytes, uint64_t totalBytes, int state) {
    for (auto& item : m_downloads) {
        if (item.id == id) {
            item.receivedBytes = receivedBytes;
            item.totalBytes = totalBytes;
            item.state = state;
            break;
        }
    }
    if (m_hwnd) {
        InvalidateRect(m_hwnd, nullptr, FALSE);
    }
}

void DownloadManager::Paint(HDC hdc) {
    RECT rc;
    GetClientRect(m_hwnd, &rc);

    HDC memDC = CreateCompatibleDC(hdc);
    HBITMAP memBM = CreateCompatibleBitmap(hdc, rc.right, rc.bottom);
    HBITMAP oldBM = (HBITMAP)SelectObject(memDC, memBM);

    // Card background
    HBRUSH bgBrush = CreateSolidBrush(m_isDark ? RGB(45, 45, 45) : RGB(255, 255, 255));
    HPEN borderPen = CreatePen(PS_SOLID, 1, m_isDark ? RGB(70, 70, 70) : RGB(210, 210, 210));
    HBRUSH oldB = (HBRUSH)SelectObject(memDC, bgBrush);
    HPEN oldP = (HPEN)SelectObject(memDC, borderPen);

    RoundRect(memDC, 0, 0, rc.right, rc.bottom, 12, 12);

    SelectObject(memDC, oldB);
    SelectObject(memDC, oldP);
    DeleteObject(bgBrush);
    DeleteObject(borderPen);

    SetBkMode(memDC, TRANSPARENT);

    // Header: "Downloads"
    SelectObject(memDC, m_boldFont);
    SetTextColor(memDC, m_isDark ? RGB(255, 255, 255) : RGB(0, 0, 0));
    RECT titleRect = { 16, 12, rc.right - 16, 32 };
    DrawTextW(memDC, L"Downloads", -1, &titleRect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

    SelectObject(memDC, m_font);

    if (m_downloads.empty()) {
        SetTextColor(memDC, m_isDark ? RGB(160, 160, 160) : RGB(120, 120, 120));
        RECT emptyRect = { 16, 50, rc.right - 16, rc.bottom - 16 };
        DrawTextW(memDC, L"No active downloads.", -1, &emptyRect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    } else {
        int y = 42;
        int maxItems = std::min(static_cast<int>(m_downloads.size()), 3);

        for (int i = 0; i < maxItems; ++i) {
            const auto& dl = m_downloads[i];

            // Filename
            SetTextColor(memDC, m_isDark ? RGB(230, 230, 230) : RGB(20, 20, 20));
            RECT nameRect = { 16, y, rc.right - 80, y + 20 };
            DrawTextW(memDC, dl.filename.c_str(), -1, &nameRect, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);

            // Status string (e.g. "Complete" or "2.4 MB / 10 MB")
            std::wstring status;
            if (dl.state == 1) {
                status = L"Complete";
            } else if (dl.state == 2) {
                status = L"Cancelled";
            } else if (dl.totalBytes > 0) {
                double mbRecv = dl.receivedBytes / (1024.0 * 1024.0);
                double mbTotal = dl.totalBytes / (1024.0 * 1024.0);
                wchar_t sbuf[64];
                swprintf_s(sbuf, L"%.1f MB / %.1f MB", mbRecv, mbTotal);
                status = sbuf;
            } else {
                status = L"Downloading...";
            }

            SetTextColor(memDC, m_isDark ? RGB(160, 160, 160) : RGB(100, 100, 100));
            RECT statRect = { rc.right - 140, y, rc.right - 16, y + 20 };
            DrawTextW(memDC, status.c_str(), -1, &statRect, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

            // Progress Bar Track
            int barY = y + 24;
            RECT trackRect = { 16, barY, rc.right - 16, barY + 6 };
            HBRUSH trackBrush = CreateSolidBrush(m_isDark ? RGB(70, 70, 70) : RGB(220, 220, 220));
            FillRect(memDC, &trackRect, trackBrush);
            DeleteObject(trackBrush);

            // Progress Bar Fill
            if (dl.totalBytes > 0) {
                float pct = std::clamp((float)dl.receivedBytes / (float)dl.totalBytes, 0.0f, 1.0f);
                int fillWidth = static_cast<int>((rc.right - 32) * pct);
                RECT fillRect = { 16, barY, 16 + fillWidth, barY + 6 };
                HBRUSH fillBrush = CreateSolidBrush(m_isDark ? RGB(96, 205, 255) : RGB(0, 120, 215));
                FillRect(memDC, &fillRect, fillBrush);
                DeleteObject(fillBrush);
            }

            y += 48;
        }
    }

    BitBlt(hdc, 0, 0, rc.right, rc.bottom, memDC, 0, 0, SRCCOPY);

    SelectObject(memDC, oldBM);
    DeleteObject(memBM);
    DeleteDC(memDC);
}

LRESULT CALLBACK DownloadManager::WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    DownloadManager* self = nullptr;
    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCTW*>(lParam);
        self = reinterpret_cast<DownloadManager*>(cs->lpCreateParams);
        if (self) {
            self->m_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        }
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    } else {
        self = reinterpret_cast<DownloadManager*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    }

    if (self) {
        return self->HandleMessage(msg, wParam, lParam);
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

LRESULT DownloadManager::HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam) {
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

    case WM_LBUTTONDOWN: {
        // Clicking a completed download opens the file in Windows Explorer
        if (!m_downloads.empty() && m_downloads[0].state == 1) {
            ShellExecuteW(nullptr, L"open", m_downloads[0].localPath.c_str(), nullptr, nullptr, SW_SHOW);
        }
        return 0;
    }
    }
    return DefWindowProcW(m_hwnd, msg, wParam, lParam);
}
