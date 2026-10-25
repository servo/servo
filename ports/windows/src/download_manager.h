/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <string>
#include <vector>
#include <cstdint>

struct ActiveDownload {
    uint64_t id;
    std::wstring filename;
    std::string url;
    uint64_t receivedBytes;
    uint64_t totalBytes;
    int state; // 0: InProgress, 1: Complete, 2: Cancelled, 3: Interrupted
    std::wstring localPath;
};

class DownloadManager {
public:
    static void RegisterWindowClass(HINSTANCE hInstance);

    DownloadManager(HWND parentHwnd);
    ~DownloadManager();

    HWND GetHwnd() const { return m_hwnd; }
    void Show(bool show);
    bool IsVisible() const;
    void SetBounds(int x, int y, int width, int height);

    void AddDownload(uint64_t id, const std::string& url, const std::wstring& filename);
    void UpdateDownload(uint64_t id, uint64_t receivedBytes, uint64_t totalBytes, int state);

private:
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    LRESULT HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam);

    void LayoutControls();
    void Paint(HDC hdc);

    HWND m_hwnd;
    HWND m_parentHwnd;
    std::vector<ActiveDownload> m_downloads;
    bool m_isDark;
    HFONT m_font;
    HFONT m_boldFont;
};
