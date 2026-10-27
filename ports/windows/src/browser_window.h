/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <dwmapi.h>
#include <string>
#include <memory>
#include <servo/servo_capi.h>

void LogDebug(const char* fmt, ...);

class TabStrip;
class OmniboxToolbar;
class FindBar;
class ServoViewHost;

class BrowserWindow {
public:
    static void RegisterWindowClass(HINSTANCE hInstance);
    static BrowserWindow* Create(struct Servo* servo, const std::string& initialUrl = "https://servo.org");

    BrowserWindow(struct Servo* servo);
    ~BrowserWindow();

    HWND GetHwnd() const { return m_hwnd; }
    struct Servo* GetServo() const { return m_servo; }

    void NavigateUrl(const std::string& url);
    void NavigateBack();
    void NavigateForward();
    void NavigateReload();
    void NavigateStop();

    int FindInPage(const char* text, bool forward, bool matchCase);
    int FindNext(int identifier, bool forward);
    void ClearFind(int identifier);

    void OnActiveTabUrlChanged(const char* url);
    void OnActiveTabTitleChanged(const char* title);
    void OnActiveTabLoadStatusChanged(int32_t status);
    void UpdateActiveTabUI(const std::string& url, const std::wstring& title);

    void OpenDevTools();
    void SetDevToolsInfo(uint16_t port, const std::string& token);
    void HandleNewDownload(struct ServoDownloadItem* download);
    void HandleDownloadUpdated(struct ServoDownloadItem* download);

    void ZoomIn();
    void ZoomOut();
    void ZoomReset();
    void ToggleFullscreen();

private:
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    LRESULT HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam);

    void LayoutChildren();
    void ApplyMicaAltTheme();
    void SyncDarkMode();

    HWND m_hwnd;
    struct Servo* m_servo;

    std::unique_ptr<TabStrip> m_tabStrip;
    std::unique_ptr<OmniboxToolbar> m_omnibox;
    std::unique_ptr<FindBar> m_findBar;
    std::unique_ptr<class DownloadManager> m_downloadManager;

    uint16_t m_devtoolsPort;
    std::string m_devtoolsToken;
    bool m_isDark;
    bool m_isFullscreen;
    WINDOWPLACEMENT m_savedPlacement;
};

