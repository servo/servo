/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <string>
#include <functional>
#include <servo/servo_capi.h>

class BrowserWindow;

class ServoViewHost {
public:
    static void RegisterWindowClass(HINSTANCE hInstance);

    ServoViewHost(HWND parentHwnd, struct Servo* servo, BrowserWindow* browserWindow, const std::string& initialUrl);
    ~ServoViewHost();

    HWND GetHwnd() const { return m_hwnd; }
    struct WebView* GetWebView() const { return m_webview; }

    void SetBounds(int x, int y, int width, int height);
    void Show(bool show);
    void SetFocus();

    void LoadUrl(const std::string& url);
    void Reload();
    void Stop();
    void GoBack();
    void GoForward();
    bool CanGoBack() const;
    bool CanGoForward() const;

    std::string GetUrl() const;
    std::string GetTitle() const;

private:
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    LRESULT HandleMessage(UINT msg, WPARAM wParam, LPARAM lParam);

    static void OnNewFrameReady(struct WebView* webview, void* userData);
    static void OnUrlChanged(struct WebView* webview, const char* url, void* userData);
    static void OnTitleChanged(struct WebView* webview, const char* title, void* userData);
    static void OnLoadStatusChanged(struct WebView* webview, int32_t status, void* userData);
    static void OnCursorChanged(struct WebView* webview, int32_t cursorType, void* userData);
    static bool OnRequestCreateNew(struct WebView* webview, struct ServoCreateNewWebViewRequest* request, void* userData);
    static void OnAlertDialog(struct WebView* webview, const char* message, void (*callback)(bool, void*), void* userData);
    static void OnConfirmDialog(struct WebView* webview, const char* message, void (*callback)(bool, void*), void* userData);
    static void OnPromptDialog(struct WebView* webview, const char* message, const char* defaultVal, void (*callback)(const char*, void*), void* userData);
    static void OnFilePicker(struct WebView* webview, bool allowMultiple, void (*callback)(const char* const*, size_t, void*), void* userData);
    static void OnContextMenu(struct WebView* webview, const struct ServoContextMenuParams* params, void (*callback)(int32_t, void*), void* userData);
    static void OnBeforeDownload(struct WebView* webview, struct ServoDownloadItem* download, void (*callback)(bool, void*), void* userData);
    static void OnDownloadUpdated(struct WebView* webview, struct ServoDownloadItem* download, void* userData);

    uint32_t GetServoModifiers() const;

    HWND m_hwnd;
    HWND m_parentHwnd;
    struct Servo* m_servo;
    struct WebView* m_webview;
    RenderingContext* m_context;
    BrowserWindow* m_browserWindow;
    bool m_trackingMouse;
    HCURSOR m_currentCursor;
};
