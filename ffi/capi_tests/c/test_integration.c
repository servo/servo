/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#define _POSIX_C_SOURCE 200809L
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
    TEST_STEP_FRAME_READY,
    TEST_STEP_SCREENSHOT_DONE,
    TEST_STEP_SCREENSHOT_FAILED,
} test_step_t;

// State passed to the delegate callbacks.
typedef struct {
    // `step` is not atomic since it is modified by the delegate
    // callbacks that are run on the main thread during spin.
    test_step_t step;
    // The dimensions of the software rendering context.
    uint32_t expected_width;
    uint32_t expected_height;
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

static void on_screenshot(const uint8_t* data, uint32_t width, uint32_t height,
    int32_t error, void* user_data);

static void on_new_frame_ready(struct WebView* webview, void* user_data)
{
    log("  [delegate] new_frame_ready\n");

    test_state_t* state = (test_state_t*)user_data;
    state->step = TEST_STEP_FRAME_READY;
    servo_webview_paint(webview);
    servo_webview_take_screenshot(webview, on_screenshot, user_data);
}

static void on_screenshot(const uint8_t* data, uint32_t width, uint32_t height,
    int32_t error, void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    if (data == NULL) {
        state->step = TEST_STEP_SCREENSHOT_FAILED;
        log("  [screenshot] failed with error=%d\n", error);
        return;
    }
    if (width != state->expected_width || height != state->expected_height) {
        state->step = TEST_STEP_SCREENSHOT_FAILED;
        log("  [screenshot] incorrect dimensions: expected %ux%u, got %ux%u\n",
            state->expected_width, state->expected_height, width, height);
        return;
    }
    uint32_t pixel_count = width * height;
    for (uint32_t i = 0; i < pixel_count; i++) {
        uint32_t offset = i * 4;
        if (data[offset] != 255 || data[offset + 1] != 0 || data[offset + 2] != 0 || data[offset + 3] != 255) {
            state->step = TEST_STEP_SCREENSHOT_FAILED;
            log("  [screenshot] non-red pixel at (%u, %u) (R=%d, G=%d, B=%d, A=%d)\n",
                i % width, i / width, data[offset], data[offset + 1],
                data[offset + 2], data[offset + 3]);
            return;
        }
    }
    state->step = TEST_STEP_SCREENSHOT_DONE;
    log("  [screenshot] all %u pixels red (255,0,0,255)  size=%ux%u\n",
        pixel_count, width, height);
}

/* Integration test that
 *  1. Creates a Servo instance
 *  2. Opens a WebView
 *  3. Load a simple page with `data:html` protocol
 *  4. Waits for rendering to complete
 *  5. Takes a screenshot and verifies the image has the expected pixel color.
 */
int test_webview_load_and_screenshot(void)
{
    log("running test_webview_load_and_screenshot\n");
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

    servo_setup_logging(servo);

    // Create the WebView
    state.expected_width = 100;
    state.expected_height = 100;
    RenderingContext* context = servo_rendering_context_create_software(
        state.expected_width, state.expected_height);
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
        on_new_frame_ready,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    // Load a simple red page
    const char* url = "data:text/html,<html><body style='background:red'></body></html>";
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

    // Spin the event loop until screenshot completes or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);
            if (state.step >= TEST_STEP_SCREENSHOT_DONE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    // Validate the screenshot
    test_step_t final_step = state.step;
    if (final_step == TEST_STEP_SCREENSHOT_FAILED) {
        log("  FAIL: screenshot failed\n");
        goto cleanup;
    }
    if (final_step != TEST_STEP_SCREENSHOT_DONE) {
        log("  FAIL: screenshot callback never fired (step=%d)\n", (int)final_step);
        goto cleanup;
    }

    if (!atomic_load(&waker_triggered)) {
        log("  FAIL: event loop waker was never called\n");
        goto cleanup;
    }

    result = 0;
    log("  PASS: all pixels verified\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    if (servo) {
        servo_free(servo);
    }
    return result;
}

// Entry point for the integration test. This is called from the Rust-side test
// runner.
int run_c_integration_tests(void)
{
    int ret = test_webview_load_and_screenshot();
    log("%s\n", ret == 0 ? "PASS" : "FAIL");
    return ret;
}
