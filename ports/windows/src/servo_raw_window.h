/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <cstdint>
#include <servo/servo_capi.h>

namespace servo_win {

// Binary ABI layout matching raw-window-handle 0.6.2 on 64-bit Windows.
// In raw-window-handle 0.6.2:
// RawDisplayHandle has variant Windows at index 9.
// RawWindowHandle has variant Win32 at index 9.
// Win32WindowHandle contains:
//   hwnd: NonZeroIsize
//   hinstance: Option<NonZeroIsize> (0 represents None via niche optimization)

#pragma pack(push, 8)

struct RawDisplayHandleWindows {
    uintptr_t tag; // 9 = Windows
    uint8_t reserved[56];
};

struct RawWindowHandleWin32 {
    uintptr_t tag; // 9 = Win32
    intptr_t hwnd;
    intptr_t hinstance;
    uint8_t reserved[48];
};

#pragma pack(pop)

inline RenderingContext* create_win32_rendering_context(HWND hwnd, uint32_t width, uint32_t height) {
    if (!hwnd || width == 0 || height == 0) {
        return nullptr;
    }

    RawDisplayHandleWindows display_handle = {};
    display_handle.tag = 9; // RawDisplayHandle::Windows

    RawWindowHandleWin32 window_handle = {};
    window_handle.tag = 9; // RawWindowHandle::Win32
    window_handle.hwnd = reinterpret_cast<intptr_t>(hwnd);
    window_handle.hinstance = 0;

    return servo_rendering_context_create_window(
        &display_handle,
        &window_handle,
        width,
        height
    );
}

} // namespace servo_win
