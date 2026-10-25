/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <string>

class BrowserWindow;

class FindBar {
public:
    static void RegisterWindowClass(HINSTANCE hInstance);

    FindBar(HWND parentHwnd, BrowserWindow* browserWindow);
    ~FindBar();

    HWND GetHwnd() const { return m_hwnd; }
    void Show(bool show);
    bool IsVisible() const;
    void SetBounds(int x, int y, int width, int height);

    void UpdateMatchResult(int activeOrdinal, int totalMatches);

private:
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK EditSubclassProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam, UINT_PTR uIdSubclass, DWORD_PTR dwRefData);

    LRESULT HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam);
    void LayoutControls();
    void Paint(HDC hdc);

    void PerformFind(bool forward);

    HWND m_hwnd;
    BrowserWindow* m_browserWindow;
    HWND m_editHwnd;

    RECT m_prevBtnBounds;
    RECT m_nextBtnBounds;
    RECT m_closeBtnBounds;
    RECT m_statusBounds;

    int m_findIdentifier;
    std::wstring m_statusText;
    bool m_isDark;
    HFONT m_font;
};
