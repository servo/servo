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

// State passed to `on_javascript_result`.
typedef struct {
    // The error the evaluation is expected to fail with, or -1 if it is
    // expected to produce
    // `[undefined, null, true, 7, 'abc', {blah: 123}, window, document.body]`.
    int32_t expected_error;
    // Set once the result has arrived, and whether it was the expected one.
    int done;
    int failed;
} javascript_state_t;

static void on_javascript_result(const ServoJSValue* value, int32_t error, void* user_data)
{
    javascript_state_t* state = (javascript_state_t*)user_data;
    state->done = 1;
    state->failed = 1;

    if (state->expected_error >= 0) {
        if (value != NULL || error != state->expected_error) {
            log("  [javascript] expected error %d, got %s (error=%d)\n",
                state->expected_error, value != NULL ? "a value" : "no value", error);
            return;
        }
        state->failed = 0;
        log("  [javascript] failed with the expected error %d\n", error);
        return;
    }

    if (value == NULL) {
        log("  [javascript] evaluation failed with error=%d\n", error);
        return;
    }
    if (servo_js_value_kind(value) != SERVO_JS_VALUE_KIND_ARRAY
        || servo_js_value_length(value) != 8) {
        log("  [javascript] expected an array of 8 items\n");
        return;
    }

    const ServoJSValue* item = servo_js_value_array_item(value, 0);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_UNDEFINED) {
        log("  [javascript] item 0 is not undefined\n");
        return;
    }
    item = servo_js_value_array_item(value, 1);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_NULL) {
        log("  [javascript] item 1 is not null\n");
        return;
    }
    item = servo_js_value_array_item(value, 2);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_BOOLEAN || !servo_js_value_boolean(item)) {
        log("  [javascript] item 2 is not true\n");
        return;
    }
    item = servo_js_value_array_item(value, 3);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_NUMBER
        || servo_js_value_number(item) != 7.0) {
        log("  [javascript] item 3 is not 7\n");
        return;
    }
    item = servo_js_value_array_item(value, 4);
    uintptr_t length = 0;
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_STRING) {
        log("  [javascript] item 4 is not a string\n");
        return;
    }
    const char* string = servo_js_value_string(item, &length);
    if (length != 3 || memcmp(string, "abc", 3) != 0) {
        log("  [javascript] item 4 is not 'abc'\n");
        return;
    }
    item = servo_js_value_array_item(value, 5);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_OBJECT
        || servo_js_value_length(item) != 1) {
        log("  [javascript] item 5 is not an object with 1 entry\n");
        return;
    }
    const char* key = servo_js_value_object_key(item, 0, &length);
    const ServoJSValue* entry = servo_js_value_object_value(item, 0);
    if (length != 4 || memcmp(key, "blah", 4) != 0
        || servo_js_value_kind(entry) != SERVO_JS_VALUE_KIND_NUMBER
        || servo_js_value_number(entry) != 123.0) {
        log("  [javascript] item 5 is not {blah: 123}\n");
        return;
    }
    // Identifiers differ between runs, so only check that there is one.
    item = servo_js_value_array_item(value, 6);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_WINDOW) {
        log("  [javascript] item 6 is not a window\n");
        return;
    }
    servo_js_value_string(item, &length);
    if (length == 0) {
        log("  [javascript] item 6 has no identifier\n");
        return;
    }
    item = servo_js_value_array_item(value, 7);
    if (servo_js_value_kind(item) != SERVO_JS_VALUE_KIND_ELEMENT) {
        log("  [javascript] item 7 is not an element\n");
        return;
    }
    servo_js_value_string(item, &length);
    if (length == 0) {
        log("  [javascript] item 7 has no identifier\n");
        return;
    }

    state->failed = 0;
    log("  [javascript] all values verified\n");
}

/* Integration test that
 *  1. Creates a Servo instance
 *  2. Opens a WebView and waits for its page to load
 *  3. Evaluates JavaScript and verifies the values it produces.
 */
int test_webview_evaluate_javascript(void)
{
    log("running test_webview_evaluate_javascript\n");
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

    const char* url = "data:text/html,<html><body></body></html>";
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

    // Evaluate a script that produces a value of each kind.
    javascript_state_t javascript_state = { -1, 0, 0 };
    const char* script = "[undefined, null, true, 7, 'abc', {blah: 123}, window, document.body]";
    servo_webview_evaluate_javascript(webview, script, on_javascript_result, &javascript_state);

    // Spin the event loop until the result arrives or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);
            if (javascript_state.done) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }
    if (!javascript_state.done) {
        log("  FAIL: evaluation callback never fired\n");
        goto cleanup;
    }
    if (javascript_state.failed) {
        log("  FAIL: unexpected evaluation result\n");
        goto cleanup;
    }

    // Evaluate a script that throws.
    javascript_state.expected_error = SERVO_JAVASCRIPT_EVALUATION_ERROR_EVALUATION_FAILURE;
    javascript_state.done = 0;
    servo_webview_evaluate_javascript(webview, "throw new Error()", on_javascript_result,
        &javascript_state);

    // Spin the event loop until the result arrives or we timeout.
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);
            if (javascript_state.done) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }
    if (!javascript_state.done) {
        log("  FAIL: evaluation callback never fired\n");
        goto cleanup;
    }
    if (javascript_state.failed) {
        log("  FAIL: unexpected evaluation result\n");
        goto cleanup;
    }

    result = 0;
    log("  PASS: all values verified\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    if (servo) {
        servo_free(servo);
    }
    return result;
}

// Entry point for the JavaScript integration test. This is called from the Rust-side test
// runner.
int run_c_javascript_tests(void)
{
    int result = test_webview_evaluate_javascript();
    log("%s\n", result == 0 ? "PASS" : "FAIL");
    return result;
}
