/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RenderingContext RenderingContext;
typedef struct Servo Servo;
typedef struct WebView WebView;

#define SERVO_THEME_LIGHT 0
#define SERVO_THEME_DARK 1

#define SERVO_MODIFIER_ALT 0x01
#define SERVO_MODIFIER_CONTROL 0x08
#define SERVO_MODIFIER_SHIFT 0x200

inline RenderingContext* servo_rendering_context_create_window(
    void* display_handle,
    void* window_handle,
    uint32_t width,
    uint32_t height
) {
    (void)display_handle;
    (void)window_handle;
    (void)width;
    (void)height;
    static int dummy = 42;
    return (RenderingContext*)&dummy;
}

inline int servo_webview_set_page_zoom(WebView* webview, float zoom) {
    (void)webview;
    (void)zoom;
    return 0;
}

inline float servo_webview_page_zoom(const WebView* webview) {
    (void)webview;
    return 1.0f;
}

#ifdef __cplusplus
}
#endif
