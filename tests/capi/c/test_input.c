/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#define _POSIX_C_SOURCE 200809L // required by `nanosleep` in `<time.h>`
#include "servo/servo_capi.h"
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

typedef enum {
    TEST_STEP_STARTING = 0,
    TEST_STEP_HEAD_PARSED,
    TEST_STEP_LOAD_COMPLETE,
} test_step_t;

// State passed to the delegate callbacks.
typedef struct {
    // `step` is not atomic since it is modified by the delegate
    // callbacks that are run on the main thread during spin.
    test_step_t step;
} test_state_t;

static atomic_int waker_triggered;

static void wake_callback(void) { atomic_store(&waker_triggered, 1); }

// WebView delegate callbacks
static void on_load_status_changed(struct WebView* webview, int32_t load_status,
    void* user_data)
{
    (void)webview;
    log("  [delegate] load_status_changed -> %d\n", load_status);
    test_state_t* state = (test_state_t*)user_data;
    test_step_t new_step;
    switch (load_status) {
    case SERVO_LOAD_STATUS_STARTED:
        new_step = TEST_STEP_STARTING;
        break;
    case SERVO_LOAD_STATUS_HEAD_PARSED:
        new_step = TEST_STEP_HEAD_PARSED;
        break;
    case SERVO_LOAD_STATUS_COMPLETE:
        new_step = TEST_STEP_LOAD_COMPLETE;
        break;
    default:
        fprintf(stderr, "  FAIL: unexpected load_status %d\n", load_status);
        abort();
    }
    state->step = new_step;
}

/* Integration test that
 *  1. Creates a Servo instance
 *  2. Opens a WebView and waits for its page to load
 *  3. Sends mouse, wheel and keyboard events to the WebView, including ones
 *     with invalid values that must be ignored
 *  4. Spins the event loop so that Servo processes them
 */
int test_webview_input_events(void)
{
    log("running test_webview_input_events\n");
    int result = 1; // default to fail
    struct Servo* servo = NULL;
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

    ServoBuilder* builder = servo_builder_create();

    struct ServoEventLoopWaker waker = { wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);

    ServoOptions* options = servo_options_create();
    servo_options_set_multiprocess(options, false);
    servo_builder_set_options(builder, options);

    servo = servo_builder_build(builder);
    if (!servo) {
        log("  FAIL: servo_builder_build returned NULL\n");
        goto cleanup;
    }

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software returned NULL\n");
        goto cleanup;
    }

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    if (!webview_builder) {
        log("  FAIL: servo_webview_builder_create returned NULL\n");
        goto cleanup;
    }

    ServoWebViewDelegate delegate = {
        &state,
        on_load_status_changed,
        NULL,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    const char* url = "data:text/html,<html><body><input autofocus></body></html>";
    if (servo_webview_builder_set_url(webview_builder, url) != 0) {
        log("  FAIL: servo_webview_builder_set_url failed for url '%s'\n", url);
        servo_webview_builder_free(webview_builder);
        goto cleanup;
    }

    webview = servo_webview_builder_build(webview_builder);
    if (!webview) {
        log("  FAIL: servo_webview_builder_build returned NULL\n");
        goto cleanup;
    }

    // Spin the event loop until the page loads or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);
            if (state.step == TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }
    if (state.step != TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: page never finished loading (step=%d)\n", (int)state.step);
        goto cleanup;
    }

    servo_webview_notify_mouse_move(webview, 10, 10);
    servo_webview_notify_mouse_button(webview, SERVO_MOUSE_BUTTON_ACTION_DOWN, 0, 10, 10);
    servo_webview_notify_mouse_button(webview, SERVO_MOUSE_BUTTON_ACTION_UP, 0, 10, 10);
    servo_webview_notify_wheel(webview, 0, 1, 0, SERVO_WHEEL_MODE_LINE, 10, 10);
    servo_webview_notify_keyboard(webview, SERVO_KEY_STATE_DOWN, "a", "KeyA", 0);
    servo_webview_notify_keyboard(webview, SERVO_KEY_STATE_UP, "a", "KeyA", 0);
    servo_webview_notify_keyboard(webview, SERVO_KEY_STATE_DOWN, "A", NULL,
        SERVO_KEY_MODIFIER_SHIFT);
    servo_webview_notify_keyboard(webview, SERVO_KEY_STATE_DOWN, "Enter", "Enter", 0);

    // Invalid values are ignored.
    servo_webview_notify_mouse_button(webview, 42, 0, 10, 10);
    servo_webview_notify_wheel(webview, 0, 1, 0, 42, 10, 10);
    servo_webview_notify_keyboard(webview, 42, "a", "KeyA", 0);
    servo_webview_notify_keyboard(webview, SERVO_KEY_STATE_DOWN, "NotAKey", NULL, 0);

    // Spin the event loop so that Servo processes the events.
    for (int i = 0; i < 20; i++) {
        servo_spin_event_loop(servo);
        sleep_ms(15);
    }

    if (state.step != TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: the page changed load status (step=%d)\n", (int)state.step);
        goto cleanup;
    }

    result = 0;
    log("  PASS: all input events sent\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    if (servo) {
        servo_free(servo);
    }
    return result;
}

// Entry point for the input integration test. This is called from the Rust-side test
// runner.
int run_c_input_tests(void)
{
    int result = test_webview_input_events();
    log("%s\n", result == 0 ? "PASS" : "FAIL");
    return result;
}
