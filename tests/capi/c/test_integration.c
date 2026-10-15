/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#ifdef _WIN32
#define _CRT_SECURE_NO_WARNINGS
#endif
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
    // Set by `notify_url_changed`. The URL string is copied because the
    // callback argument is only valid for the duration of the callback.
    int url_changed_fired;
    int url_matched;
    char last_url[256];
    // Set by `notify_page_title_changed`.
    int title_changed_fired;
    int title_matched;
    char last_title[256];
    // Set by `notify_load_started`.
    int load_started_fired;
    // Set by `notify_load_ended`.
    int load_ended_fired;
    // Set by `notify_load_error`.
    int load_error_fired;
    int load_error_code;
    char load_error_text[256];
    // Set by `notify_status_text_changed`.
    int status_text_fired;
    char last_status_text[256];
    // Set by `notify_favicon_changed`.
    int favicon_fired;
    char last_favicon_url[256];
    // Set by `notify_cursor_changed`.
    int cursor_fired;
    int32_t last_cursor_type;
    // Set by `show_console_message`.
    int console_fired;
    int32_t last_console_level;
    char last_console_message[256];
    // JavaScript evaluation results (callbacks run on the main thread
    // during spin, so no atomics are required).
    int js_sum_requested;
    int js_sum_received;
    char js_sum[64];
    int js_error_requested;
    int js_error_received;
    char js_error[256];
    // Close test state.
    int closed_fired;
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

static void on_url_changed(struct WebView* webview, const char* url, void* user_data)
{
    (void)webview;
    log("  [delegate] url_changed -> %s\n", url ? url : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->url_changed_fired = 1;
    if (url != NULL) {
        // The `url` argument is only valid for the duration of the callback.
        strncpy(state->last_url, url, sizeof(state->last_url) - 1);
        state->last_url[sizeof(state->last_url) - 1] = '\0';
        if (strstr(url, "data:text/html") != NULL) {
            state->url_matched = 1;
        }
    }
}

static void on_page_title_changed(struct WebView* webview, const char* title, void* user_data)
{
    (void)webview;
    log("  [delegate] page_title_changed -> %s\n", title ? title : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->title_changed_fired = 1;
    if (title != NULL) {
        // The `title` argument is only valid for the duration of the callback.
        strncpy(state->last_title, title, sizeof(state->last_title) - 1);
        state->last_title[sizeof(state->last_title) - 1] = '\0';
        if (strstr(title, "Embedded") != NULL) {
            state->title_matched = 1;
        }
    }
}

static void on_load_started(struct WebView* webview, void* user_data)
{
    (void)webview;
    log("  [delegate] load_started\n");
    test_state_t* state = (test_state_t*)user_data;
    state->load_started_fired = 1;
}

static void on_load_ended(struct WebView* webview, void* user_data)
{
    (void)webview;
    log("  [delegate] load_ended\n");
    test_state_t* state = (test_state_t*)user_data;
    state->load_ended_fired = 1;
}

static void on_load_error(struct WebView* webview, int32_t error_code, const char* error_text, void* user_data)
{
    (void)webview;
    log("  [delegate] load_error -> code=%d, text=%s\n", error_code, error_text ? error_text : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->load_error_fired = 1;
    state->load_error_code = error_code;
    if (error_text != NULL) {
        strncpy(state->load_error_text, error_text, sizeof(state->load_error_text) - 1);
        state->load_error_text[sizeof(state->load_error_text) - 1] = '\0';
    }
}

static void on_status_text_changed(struct WebView* webview, const char* text, void* user_data)
{
    (void)webview;
    log("  [delegate] status_text_changed -> %s\n", text ? text : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->status_text_fired = 1;
    if (text != NULL) {
        strncpy(state->last_status_text, text, sizeof(state->last_status_text) - 1);
        state->last_status_text[sizeof(state->last_status_text) - 1] = '\0';
    }
}

static void on_favicon_changed(struct WebView* webview, const char* url, void* user_data)
{
    (void)webview;
    log("  [delegate] favicon_changed -> %s\n", url ? url : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->favicon_fired = 1;
    if (url != NULL) {
        strncpy(state->last_favicon_url, url, sizeof(state->last_favicon_url) - 1);
        state->last_favicon_url[sizeof(state->last_favicon_url) - 1] = '\0';
    }
}

static void on_cursor_changed(struct WebView* webview, int32_t cursor_type, void* user_data)
{
    (void)webview;
    log("  [delegate] cursor_changed -> %d\n", cursor_type);
    test_state_t* state = (test_state_t*)user_data;
    state->cursor_fired = 1;
    state->last_cursor_type = cursor_type;
}

static void on_console_message(struct WebView* webview, int32_t level, const char* message, void* user_data)
{
    (void)webview;
    log("  [delegate] console_message -> level=%d, message=%s\n", level, message ? message : "(null)");
    test_state_t* state = (test_state_t*)user_data;
    state->console_fired = 1;
    state->last_console_level = level;
    if (message != NULL) {
        strncpy(state->last_console_message, message, sizeof(state->last_console_message) - 1);
        state->last_console_message[sizeof(state->last_console_message) - 1] = '\0';
    }
}

static void on_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    log("  [delegate] notify_closed\n");
    test_state_t* state = (test_state_t*)user_data;
    state->closed_fired = 1;
}

static void on_javascript_sum(const char* result_json, const char* error_message, void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    if (result_json != NULL) {
        state->js_sum_received = 1;
        strncpy(state->js_sum, result_json, sizeof(state->js_sum) - 1);
        state->js_sum[sizeof(state->js_sum) - 1] = '\0';
        log("  [js] '1 + 1' -> %s\n", result_json);
    } else {
        log("  [js] '1 + 1' unexpectedly failed: %s\n",
            error_message ? error_message : "(null)");
    }
}

static void on_javascript_error(const char* result_json, const char* error_message, void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    if (error_message != NULL) {
        state->js_error_received = 1;
        strncpy(state->js_error, error_message, sizeof(state->js_error) - 1);
        state->js_error[sizeof(state->js_error) - 1] = '\0';
        log("  [js] throwing script -> error: %s\n", error_message);
    } else {
        log("  [js] throwing script unexpectedly succeeded: %s\n",
            result_json ? result_json : "(null)");
    }
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
 *  1. Opens a WebView on the shared Servo instance
 *  2. Loads a simple page with the `data:html` protocol
 *  3. Waits for rendering to complete
 *  4. Takes a screenshot and verifies the image has the expected pixel color.
 */
int test_webview_load_and_screenshot(struct Servo* servo)
{
    log("running test_webview_load_and_screenshot\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_new_frame_ready = on_new_frame_ready,
        .notify_closed = on_closed,
        .notify_load_started = on_load_started,
        .notify_load_ended = on_load_ended,
        .notify_load_error = on_load_error,
        .notify_status_text_changed = on_status_text_changed,
        .notify_favicon_changed = on_favicon_changed,
        .notify_cursor_changed = on_cursor_changed,
        .show_console_message = on_console_message,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    // Load a simple red page with a title so that the page-title delegate
    // callback and `servo_webview_get_title` can be exercised too.
    const char* url = "data:text/html,<html><head><title>Embedded%20Test</title></head>"
                      "<body style='background:red'></body></html>";
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

    // Spin the event loop until screenshot completes and both JavaScript
    // evaluations have finished, or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);

            // Once the page has reached load completion, exercise the
            // asynchronous JavaScript evaluation API with a simple expression.
            if (state.step >= TEST_STEP_LOAD_COMPLETE && !state.js_sum_requested) {
                state.js_sum_requested = 1;
                log("  Evaluating '1 + 1'...\n");
                servo_webview_evaluate_javascript(webview, "1 + 1", on_javascript_sum, &state);
            }

            // Once the sum result came back, evaluate a script that throws to
            // exercise the error path of the javascript callback.
            if (state.js_sum_received && !state.js_error_requested) {
                state.js_error_requested = 1;
                log("  Evaluating throwing script...\n");
                servo_webview_evaluate_javascript(
                    webview, "throw new Error('intentional test error')",
                    on_javascript_error, &state);
            }

            if (state.step >= TEST_STEP_SCREENSHOT_DONE && state.js_error_received) {
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

    // Validate the URL-changed delegate callback.
    if (!state.url_changed_fired) {
        log("  FAIL: notify_url_changed never fired\n");
        goto cleanup;
    }
    if (!state.url_matched) {
        log("  FAIL: notify_url_changed never reported the loaded URL (last='%s')\n",
            state.last_url);
        goto cleanup;
    }

    // Validate the page-title delegate callback and `servo_webview_get_title`.
    if (!state.title_changed_fired) {
        log("  FAIL: notify_page_title_changed never fired\n");
        goto cleanup;
    }
    if (!state.title_matched) {
        log("  FAIL: notify_page_title_changed never reported the expected title "
            "(last='%s')\n", state.last_title);
        goto cleanup;
    }
    {
        char title_buffer[256];
        size_t title_required =
            servo_webview_get_title(webview, title_buffer, sizeof(title_buffer));
        if (title_required == 0 || title_required > sizeof(title_buffer)) {
            log("  FAIL: servo_webview_get_title returned required=%zu\n", title_required);
            goto cleanup;
        }
        if (title_buffer[0] == '\0') {
            log("  FAIL: servo_webview_get_title returned an empty result\n");
            goto cleanup;
        }
        if (strstr(title_buffer, "Embedded") == NULL) {
            log("  FAIL: servo_webview_get_title returned unexpected '%s'\n", title_buffer);
            goto cleanup;
        }
        log("  [title] servo_webview_get_title -> '%s'\n", title_buffer);
    }

    // Validate the load start/end delegate callbacks.
    if (!state.load_started_fired) {
        log("  FAIL: notify_load_started never fired\n");
        goto cleanup;
    }
    if (!state.load_ended_fired) {
        log("  FAIL: notify_load_ended never fired\n");
        goto cleanup;
    }
    // Note: load_error may not fire for successful loads, so we just log it.
    if (state.load_error_fired) {
        log("  [load] notify_load_error fired (code=%d, text='%s')\n",
            state.load_error_code, state.load_error_text);
    }

    // Validate the status text delegate callback (may not fire for data: URLs).
    if (state.status_text_fired) {
        log("  [status] notify_status_text_changed -> '%s'\n", state.last_status_text);
    }

    // Validate the favicon delegate callback (may not fire for data: URLs).
    if (state.favicon_fired) {
        log("  [favicon] notify_favicon_changed -> '%s'\n", state.last_favicon_url);
    }

    // Validate the cursor delegate callback.
    if (state.cursor_fired) {
        log("  [cursor] notify_cursor_changed -> %d\n", state.last_cursor_type);
    }

    // Validate the console message delegate callback.
    if (state.console_fired) {
        log("  [console] show_console_message -> level=%d, message='%s'\n",
            state.last_console_level, state.last_console_message);
    }

    // Validate the JavaScript '1 + 1' evaluation.
    if (!state.js_sum_received) {
        log("  FAIL: JavaScript '1 + 1' callback never fired\n");
        goto cleanup;
    }
    if (strcmp(state.js_sum, "2") != 0) {
        log("  FAIL: JavaScript '1 + 1' returned '%s', expected '2'\n", state.js_sum);
        goto cleanup;
    }

    // Validate the error path of the JavaScript callback.
    if (!state.js_error_received) {
        log("  FAIL: throwing JavaScript script did not invoke the error callback\n");
        goto cleanup;
    }
    log("  [js] captured error: %s\n", state.js_error);

    result = 0;
    log("  PASS: all pixels verified\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

/* Integration test that
 *  1. Opens a WebView on the shared Servo instance
 *  2. Loads a simple page
 *  3. Calls servo_webview_close
 *  4. Waits for notify_closed to fire
 *  5. Calls servo_webview_free to verify it still works after close
 */
int test_webview_close(struct Servo* servo)
{
    log("running test_webview_close\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_new_frame_ready = on_new_frame_ready,
        .notify_closed = on_closed,
        .notify_load_started = on_load_started,
        .notify_load_ended = on_load_ended,
        .notify_load_error = on_load_error,
        .notify_status_text_changed = on_status_text_changed,
        .notify_favicon_changed = on_favicon_changed,
        .notify_cursor_changed = on_cursor_changed,
        .show_console_message = on_console_message,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    // Load a simple page
    const char* url = "data:text/html,<html><head><title>Close Test</title></head>"
                      "<body>Close Test</body></html>";
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

    // Spin the event loop until load completes
    log("  Spinning event loop until load completes... (timeout 10s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 10000) {
            servo_spin_event_loop(servo);

            if (state.step >= TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    if (state.step < TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: page load did not complete in time\n");
        goto cleanup;
    }

    // Now call servo_webview_close
    log("  Calling servo_webview_close...\n");
    servo_webview_close(webview);

    // Spin the event loop until notify_closed fires
    log("  Spinning event loop until notify_closed fires... (timeout 10s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 10000) {
            servo_spin_event_loop(servo);

            if (state.closed_fired) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    if (!state.closed_fired) {
        log("  FAIL: notify_closed never fired\n");
        goto cleanup;
    }

    log("  notify_closed fired, now calling servo_webview_free...\n");
    servo_webview_free(webview);
    webview = NULL;

    result = 0;
    log("  PASS: close test completed successfully\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

// Entry point for the integration test. This is called from the Rust-side test
// runner.
//
// A single `Servo` instance is created and shared by all of the scenarios
// below. Servo's embedding runtime relies on process-global, one-shot
// initialization (the JavaScript engine, the async runtime, the logger and the
// configuration options), so only one `Servo` instance may exist per process.
int run_c_integration_tests(void)
{
    int result = 1; // default to fail
    struct Servo* servo = NULL;

    ServoBuilder* builder = servo_builder_create();

    struct ServoEventLoopWaker waker = { wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);

    ServoOptions* options = servo_options_create();
    servo_options_set_multiprocess(options, false);
    servo_builder_set_options(builder, options);

    servo = servo_builder_build(builder);
    if (!servo) {
        log("  FAIL: servo_builder_build returned NULL\n");
        return 1;
    }

    servo_setup_logging(servo);

    result = test_webview_load_and_screenshot(servo);
    if (result == 0) {
        result = test_webview_close(servo);
    }

    log("%s\n", result == 0 ? "PASS" : "FAIL");

    servo_free(servo);
    return result;
}
