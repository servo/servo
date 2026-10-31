/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#define _POSIX_C_SOURCE 200809L // required by `nanosleep` in `<time.h>`
#include "servo/servo_capi.h"
#include <stdio.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <time.h>
#endif

#define log(fmt, ...)               \
    do {                            \
        printf(fmt, ##__VA_ARGS__); \
        fflush(stdout);             \
    } while (0)

static void sleep_ms(long period)
{
#ifdef _WIN32
    Sleep(period);
#else
    struct timespec delay;
    delay.tv_sec = period / 1000;
    delay.tv_nsec = (period % 1000) * 1000000;
    nanosleep(&delay, NULL);
#endif
}

typedef struct {
    int frames_requested;
    int frames_ready;
} test_state_t;

static void wake_callback(void) { }

static void on_new_frame_ready(struct WebView* webview, void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    state->frames_ready++;
    servo_webview_paint(webview);
}

static void on_frame_requested(void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    state->frames_requested++;
}

/* Integration test that
 *  1. Creates a WebView whose rendering context has a refresh driver
 *  2. Loads a simple page and waits for Servo to request a frame
 *  3. Starts the frame and waits for it to be ready
 */
int test_refresh_driver(void)
{
    log("running test_refresh_driver\n");
    int result = 1; // default to fail
    struct Servo* servo = NULL;
    struct WebView* webview = NULL;
    test_state_t state = { 0, 0 };
    ServoRefreshDriver* refresh_driver = servo_refresh_driver_create(&state, on_frame_requested);

    ServoBuilder* builder = servo_builder_create();
    struct ServoEventLoopWaker waker = { wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);
    ServoOptions* options = servo_options_create();
    servo_options_set_multiprocess(options, false);
    servo_builder_set_options(builder, options);
    servo = servo_builder_build(builder);

    RenderingContext* context = servo_rendering_context_create_software_with_refresh_driver(
        100, 100, refresh_driver);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software_with_refresh_driver returned NULL\n");
        goto cleanup;
    }

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    ServoWebViewDelegate delegate = { &state, NULL, on_new_frame_ready };
    servo_webview_builder_set_delegate(webview_builder, delegate);
    // A page whose animation never ends keeps asking for new frames.
    servo_webview_builder_set_url(webview_builder,
        "data:text/html,<html><body style='animation: fade 100ms infinite'><style>"
        "@keyframes fade { from { opacity: 0 } to { opacity: 1 } }</style></body></html>");
    webview = servo_webview_builder_build(webview_builder);

    // Spin the event loop, starting each frame Servo requests, until a started
    // frame is ready or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    int frames_started = 0;
    int frames_ready_before_start = 0;
    for (int elapsed_ms = 0; elapsed_ms < 30000; elapsed_ms += 15) {
        servo_spin_event_loop(servo);
        if (frames_started > 0 && state.frames_ready > frames_ready_before_start) {
            break;
        }
        if (frames_started < state.frames_requested) {
            frames_started = state.frames_requested;
            frames_ready_before_start = state.frames_ready;
            servo_refresh_driver_start_frame(refresh_driver);
        }
        sleep_ms(15);
    }

    if (frames_started == 0) {
        log("  FAIL: the refresh driver was never asked for a frame\n");
        goto cleanup;
    }
    if (state.frames_ready == frames_ready_before_start) {
        log("  FAIL: no frame was ready after starting %d frames\n", frames_started);
        goto cleanup;
    }

    result = 0;
    log("  PASS: a frame was ready after starting %d frames\n", frames_started);

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    if (servo) {
        servo_free(servo);
    }
    servo_refresh_driver_free(refresh_driver);
    return result;
}

// Entry point for the refresh driver integration test. This is called from the
// Rust-side test runner.
int run_c_refresh_driver_tests(void)
{
    int result = test_refresh_driver();
    log("%s\n", result == 0 ? "PASS" : "FAIL");
    return result;
}
