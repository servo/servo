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

class OmniboxToolbar {
public:
    static void RegisterWindowClass(HINSTANCE hInstance);

    OmniboxToolbar(HWND parentHwnd, BrowserWindow* browserWindow);
    ~OmniboxToolbar();

    HWND GetHwnd() const { return m_hwnd; }
    void SetBounds(int x, int y, int width, int height);

    void SetUrl(const std::string& url);
    void SetLoading(bool loading);
    void UpdateNavigationState(bool canBack, bool canForward);
    void FocusAndSelectAll();
    void FocusForSearch();
    void SetDarkMode(bool isDark);

private:
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    static LRESULT CALLBACK EditSubclassProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam, UINT_PTR uIdSubclass, DWORD_PTR dwRefData);

    LRESULT HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam);
    void LayoutControls();
    void Paint(HDC hdc);

    HWND m_hwnd;
    BrowserWindow* m_browserWindow;
    HWND m_editHwnd;

    RECT m_backBtnBounds;
    RECT m_forwardBtnBounds;
    RECT m_reloadBtnBounds;
    RECT m_homeBtnBounds;
    RECT m_omniboxBounds;

    bool m_canBack;
    bool m_canForward;
    bool m_isLoading;
    bool m_isDark;
    HFONT m_font;
    HFONT m_iconFont;
};
