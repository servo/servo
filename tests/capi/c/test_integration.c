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
#include <psapi.h> // `GetProcessMemoryInfo`, used by the W5.7 leak test
#elif defined(__APPLE__)
#include <time.h>
#include <mach/mach.h> // `task_info`, used by the W5.7 leak test
#else
#include <time.h>
#include <malloc.h> // `mallinfo2`, used by the W5.7 leak-test diagnostic
#include <unistd.h> // `sysconf`, used by the W5.7 leak test
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

// Monotonic milliseconds. Unlike a counter that adds a fixed step per spin
// iteration, this measures real elapsed time no matter how long each
// `servo_spin_event_loop` call takes.
static long now_ms(void)
{
#ifdef _WIN32
    return (long)GetTickCount64();
#else
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
#endif
}


// W5.6 observability metrics. `-1` means "not measured" and is written to the
// report as `null`, so a metric that stops being collected shows up as a gap
// instead of a silent zero. See docs/embedding/OBSERVABILITY.md.
static long g_metric_servo_build_ms = -1;
static long g_metric_load_complete_ms = -1;
static long g_metric_screenshot_start_ms = -1;
static long g_metric_screenshot_ms = -1;
static long g_metric_js_start_ms = -1;
static long g_metric_js_ms = -1;

// The fixed JavaScript workload timed by the `js_ms` metric. Same script and
// same iteration count on every platform and every run: the absolute number is
// not a benchmark, only its change against the committed baseline matters.
#define JS_WORKLOAD \
    "(function () { var s = 0; for (var i = 0; i < 100000; i++) { s += i; } return s; })()"

// Formats one metric as a JSON member, or as `"name":null` when it was never
// measured.
static void format_metric(char* out, size_t out_len, const char* name, long value)
{
    if (value < 0) {
        snprintf(out, out_len, "\"%s\":null", name);
    } else {
        snprintf(out, out_len, "\"%s\":%ld", name, value);
    }
}

// W5.6. Writes the metrics as a single-line JSON object to the file named by
// `SERVO_OBSERVABILITY_FILE`. The "Collect observability metrics" step of
// `embedding-api.yml` reads that file and adds the cdylib size, which the C
// side cannot see. When the variable is not set (a plain `cargo test` run) the
// numbers are only logged.
static void write_observability_metrics(void)
{
    char startup[64];
    char load_complete[64];
    char screenshot[64];
    char js[64];
    char json[320];

    format_metric(startup, sizeof(startup), "startup_ms", g_metric_servo_build_ms);
    format_metric(load_complete, sizeof(load_complete), "load_complete_ms",
        g_metric_load_complete_ms);
    format_metric(screenshot, sizeof(screenshot), "screenshot_ms", g_metric_screenshot_ms);
    format_metric(js, sizeof(js), "js_ms", g_metric_js_ms);

    snprintf(json, sizeof(json), "{%s,%s,%s,%s}", startup, load_complete, screenshot, js);
    log("  [observability] %s\n", json);

    const char* path = getenv("SERVO_OBSERVABILITY_FILE");
    if (path == NULL || path[0] == '\0') {
        return;
    }

    FILE* file = fopen(path, "w");
    if (file == NULL) {
        log("  [observability] could not write '%s'\n", path);
        return;
    }
    fprintf(file, "%s\n", json);
    fclose(file);
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
    // W5.6: the fixed JavaScript workload that the `js_ms` metric times.
    int js_workload_requested;
    int js_workload_received;
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

// W5.6: completion callback for the fixed JavaScript workload. The value is
// only freed here; the metric is the time between the request and this call.
static void on_javascript_workload(struct ServoValue* result_value, const char* error_message,
    void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    if (result_value == NULL) {
        log("  [js] workload failed: %s\n", error_message ? error_message : "(no message)");
        return;
    }
    servo_value_free(result_value);
    state->js_workload_received = 1;
}

static void on_screenshot(const uint8_t* data, uint32_t width, uint32_t height,
    int32_t error, void* user_data);

static void on_new_frame_ready(struct WebView* webview, void* user_data)
{
    log("  [delegate] new_frame_ready\n");

    test_state_t* state = (test_state_t*)user_data;
    state->step = TEST_STEP_FRAME_READY;
    servo_webview_paint(webview);
    g_metric_screenshot_start_ms = now_ms();
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
        .struct_size = sizeof(ServoWebViewDelegate),
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
    long load_start_ms = now_ms();
    log("  Spinning event loop... (timeout 30s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 30000) {
            servo_spin_event_loop(servo);

            // W5.6: record the observability metrics as the page settles.
            if (state.step >= TEST_STEP_LOAD_COMPLETE && g_metric_load_complete_ms < 0) {
                g_metric_load_complete_ms = now_ms() - load_start_ms;
            }
            if (state.step == TEST_STEP_SCREENSHOT_DONE && g_metric_screenshot_ms < 0
                && g_metric_screenshot_start_ms >= 0) {
                g_metric_screenshot_ms = now_ms() - g_metric_screenshot_start_ms;
            }
            if (state.js_workload_received && g_metric_js_ms < 0 && g_metric_js_start_ms >= 0) {
                g_metric_js_ms = now_ms() - g_metric_js_start_ms;
            }

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

            // W5.6: time a fixed JavaScript workload, so the report has a
            // throughput sentinel next to the page-load and paint numbers.
            if (state.js_error_received && !state.js_workload_requested) {
                state.js_workload_requested = 1;
                log("  Evaluating the JavaScript workload...\n");
                g_metric_js_start_ms = now_ms();
                servo_webview_evaluate_javascript_v2(webview, JS_WORKLOAD,
                    on_javascript_workload, &state);
            }

            if (state.step >= TEST_STEP_SCREENSHOT_DONE && state.js_error_received
                && state.js_workload_received) {
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
        .struct_size = sizeof(ServoWebViewDelegate),
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

/* Spins the event loop until the counter at `counter` reaches `target` or
 * `timeout_ms` elapses. Returns 1 if the counter reached the target, 0 if the
 * timeout expired first. */
static int spin_until_count(struct Servo* servo, const int* counter, int target, int timeout_ms)
{
    int waited_ms = 0;
    const int poll_ms = 5;
    while (*counter < target && waited_ms < timeout_ms) {
        servo_spin_event_loop(servo);
        sleep_ms(poll_ms);
        waited_ms += poll_ms;
    }
    return *counter >= target;
}

/* State for the W4.2 clipboard round-trip test. */
typedef struct {
    int load_complete;
    int closed;
    /* Calls received by the embedder's clipboard delegate. */
    int get_text_calls;
    int set_text_calls;
    char set_text_value[128];
    /* Number of completed JavaScript evaluations and the last result. */
    int js_calls;
    char js_result[64];
} clipboard_test_state_t;

/* The text the delegate reports as the clipboard contents. */
static const char CLIPBOARD_TEXT[] = "servo-clipboard-round-trip";

/* The editing shortcuts are `Cmd` based on macOS and `Ctrl` based everywhere
 * else, matching `CMD_OR_CONTROL` in
 * `components/script/dom/document/editing.rs`. */
#ifdef __APPLE__
#define CLIPBOARD_SHORTCUT_MODIFIER SERVO_MODIFIER_META
#else
#define CLIPBOARD_SHORTCUT_MODIFIER SERVO_MODIFIER_CONTROL
#endif

static void on_clipboard_load_status(struct WebView* webview, int32_t load_status, void* user_data)
{
    (void)webview;
    if (load_status == SERVO_LOAD_STATUS_COMPLETE) {
        ((clipboard_test_state_t*)user_data)->load_complete = 1;
    }
}

static void on_clipboard_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    ((clipboard_test_state_t*)user_data)->closed = 1;
}

static void on_clipboard_js(const char* result_json, const char* error_message, void* user_data)
{
    clipboard_test_state_t* state = (clipboard_test_state_t*)user_data;
    state->js_calls++;
    state->js_result[0] = '\0';
    if (result_json != NULL) {
        strncpy(state->js_result, result_json, sizeof(state->js_result) - 1);
        state->js_result[sizeof(state->js_result) - 1] = '\0';
    } else {
        log("  [js] evaluation failed: %s\n", error_message ? error_message : "(null)");
    }
}

static void on_clipboard_get_text(struct WebView* webview, ServoClipboardStringRequest* request,
    void* user_data)
{
    (void)webview;
    clipboard_test_state_t* state = (clipboard_test_state_t*)user_data;
    state->get_text_calls++;
    log("  [clipboard] get_text\n");

    // Answer right away. The script thread blocks waiting for this request to
    // be resolved, so deferring the answer past this callback would deadlock
    // the page.
    ServoError error = servo_webview_clipboard_request_success(request, CLIPBOARD_TEXT);
    if (error != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_clipboard_request_success returned %d\n", (int)error);
    }
}

static void on_clipboard_set_text(struct WebView* webview, const char* text, void* user_data)
{
    (void)webview;
    clipboard_test_state_t* state = (clipboard_test_state_t*)user_data;
    state->set_text_calls++;
    log("  [clipboard] set_text('%s')\n", text ? text : "(null)");
    if (text != NULL) {
        strncpy(state->set_text_value, text, sizeof(state->set_text_value) - 1);
        state->set_text_value[sizeof(state->set_text_value) - 1] = '\0';
    }
}

/* Proves that the `ServoClipboardDelegate` installed on a builder is the
 * delegate the engine actually consults, in both directions.
 *
 * The page is a focused `<textarea>`. A paste shortcut makes the engine ask the
 * embedder for the clipboard contents (`get_text`); the delegate answers with
 * `CLIPBOARD_TEXT`, and the test then asks the page whether that text landed in
 * the textarea. A copy shortcut makes the engine hand the embedder the selected
 * text (`set_text`), which the test compares against `CLIPBOARD_TEXT`.
 *
 * Those assertions only hold if the delegate replaced Servo's built-in system
 * clipboard implementation, which is what W4.2 requires. */
static int test_clipboard_round_trip(struct Servo* servo)
{
    log("running test_clipboard_round_trip\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    clipboard_test_state_t state;
    memset(&state, 0, sizeof(state));

    // The rendering context is owned by the WebView once the builder is
    // created, so only the WebView is freed (see C_API_GUIDE.md).
    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software returned NULL\n");
        goto cleanup;
    }

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (!builder) {
        log("  FAIL: servo_webview_builder_create returned NULL\n");
        goto cleanup;
    }

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_clipboard_load_status,
        .notify_closed = on_clipboard_closed,
    };
    ServoClipboardDelegate clipboard_delegate = {
        .struct_size = sizeof(ServoClipboardDelegate),
        .user_data = &state,
        .clear = NULL,
        .get_text = on_clipboard_get_text,
        .set_text = on_clipboard_set_text,
    };

    if (servo_webview_builder_set_delegate(builder, delegate) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_delegate failed\n");
        servo_webview_builder_free(builder);
        goto cleanup;
    }
    if (servo_webview_builder_set_clipboard_delegate(builder, clipboard_delegate)
        != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_clipboard_delegate failed\n");
        servo_webview_builder_free(builder);
        goto cleanup;
    }

    // A delegate compiled against a smaller struct must be rejected instead of
    // being read past its end (ABI_STABILITY.md section 3.3).
    {
        ServoClipboardDelegate truncated_delegate = clipboard_delegate;
        truncated_delegate.struct_size = 0;
        if (servo_webview_builder_set_clipboard_delegate(builder, truncated_delegate)
            != SERVO_ERROR_INVALID_ARGUMENT) {
            log("  FAIL: set_clipboard_delegate accepted struct_size = 0\n");
            servo_webview_builder_free(builder);
            goto cleanup;
        }
    }

    const char* url = "data:text/html,<html><body><textarea id='t'></textarea></body></html>";
    if (servo_webview_builder_set_url(builder, url) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_url failed\n");
        servo_webview_builder_free(builder);
        goto cleanup;
    }

    webview = servo_webview_builder_build(builder);
    if (!webview) {
        log("  FAIL: servo_webview_builder_build returned NULL\n");
        goto cleanup;
    }

    if (!spin_until_count(servo, &state.load_complete, 1, 10000)) {
        log("  FAIL: the page never finished loading\n");
        goto cleanup;
    }
    servo_webview_set_focused(webview, true);

    // Put the cursor in the textarea so that the paste has a target.
    servo_webview_evaluate_javascript(webview,
        "const t = document.getElementById('t'); t.focus(); t.setSelectionRange(0, 0);",
        on_clipboard_js, &state);
    if (!spin_until_count(servo, &state.js_calls, 1, 10000)) {
        log("  FAIL: focusing the textarea timed out\n");
        goto cleanup;
    }

    // Paste: the engine has to ask the embedder for the clipboard contents.
    log("  Sending the paste shortcut...\n");
    servo_webview_notify_key_event(webview, SERVO_KEY_EVENT_DOWN, "v", "KeyV", 0,
        CLIPBOARD_SHORTCUT_MODIFIER, false, false);
    servo_webview_notify_key_event(webview, SERVO_KEY_EVENT_UP, "v", "KeyV", 0,
        CLIPBOARD_SHORTCUT_MODIFIER, false, false);
    if (!spin_until_count(servo, &state.get_text_calls, 1, 10000)) {
        log("  FAIL: the paste never reached ServoClipboardDelegate::get_text\n");
        goto cleanup;
    }

    // Read the textarea back from the page: the delegate's text is only visible
    // here if the engine actually inserted it.
    servo_webview_evaluate_javascript(webview,
        "document.getElementById('t').value === 'servo-clipboard-round-trip'", on_clipboard_js,
        &state);
    if (!spin_until_count(servo, &state.js_calls, 2, 10000)) {
        log("  FAIL: reading the textarea back timed out\n");
        goto cleanup;
    }
    if (strcmp(state.js_result, "true") != 0) {
        log("  FAIL: the textarea value after paste is not the delegate's clipboard "
            "text (expression returned '%s')\n", state.js_result);
        goto cleanup;
    }

    // Copy: the engine has to hand the embedder the selected text.
    log("  Sending the copy shortcut...\n");
    servo_webview_evaluate_javascript(webview, "document.getElementById('t').select()",
        on_clipboard_js, &state);
    if (!spin_until_count(servo, &state.js_calls, 3, 10000)) {
        log("  FAIL: selecting the textarea timed out\n");
        goto cleanup;
    }
    servo_webview_notify_key_event(webview, SERVO_KEY_EVENT_DOWN, "c", "KeyC", 0,
        CLIPBOARD_SHORTCUT_MODIFIER, false, false);
    servo_webview_notify_key_event(webview, SERVO_KEY_EVENT_UP, "c", "KeyC", 0,
        CLIPBOARD_SHORTCUT_MODIFIER, false, false);
    if (!spin_until_count(servo, &state.set_text_calls, 1, 10000)) {
        log("  FAIL: the copy never reached ServoClipboardDelegate::set_text\n");
        goto cleanup;
    }
    if (strcmp(state.set_text_value, CLIPBOARD_TEXT) != 0) {
        log("  FAIL: set_text received '%s', expected '%s'\n", state.set_text_value,
            CLIPBOARD_TEXT);
        goto cleanup;
    }

    log("  PASS: clipboard get_text/set_text round-tripped through the C API\n");
    result = 0;

cleanup:
    if (webview) {
        // Close cooperatively and wait for `notify_closed` before freeing, as
        // documented in C_API_GUIDE.md.
        servo_webview_close(webview);
        spin_until_count(servo, &state.closed, 1, 10000);
        servo_webview_free(webview);
    }
    return result;
}

/* State for the W6.4 accessibility-exposure test. */
typedef struct {
    int load_complete;
    int closed;
    /* Number of accessibility tree updates delivered to the delegate. */
    int update_calls;
    /* How many of those carried a dictionary payload (the `ServoValue` shape
     * documented for `notify_accessibility_tree_update`). */
    int updates_with_dictionary;
    /* How many carried at least one AccessKit node. */
    int updates_with_nodes;
} accessibility_test_state_t;

static void on_a11y_load_status(struct WebView* webview, int32_t load_status, void* user_data)
{
    (void)webview;
    if (load_status == SERVO_LOAD_STATUS_COMPLETE) {
        ((accessibility_test_state_t*)user_data)->load_complete = 1;
    }
}

static void on_a11y_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    ((accessibility_test_state_t*)user_data)->closed = 1;
}

/* The callback owns the payload, so it is inspected and freed here. */
static void on_a11y_tree_update(struct WebView* webview, ServoValue* tree_update, void* user_data)
{
    (void)webview;
    accessibility_test_state_t* state = (accessibility_test_state_t*)user_data;
    state->update_calls++;

    if (tree_update == NULL) {
        log("  [a11y] update with a NULL payload\n");
        return;
    }

    int32_t type = SERVO_VALUE_TYPE_INVALID;
    if (servo_value_get_type(tree_update, &type) == SERVO_ERROR_OK
        && type == SERVO_VALUE_TYPE_DICTIONARY) {
        state->updates_with_dictionary++;

        // AccessKit names the node list "nodes"; the focus-only update that
        // follows a graft carries an empty one.
        ServoDictionaryValue* dict = NULL;
        if (servo_value_get_dictionary(tree_update, &dict) == SERVO_ERROR_OK && dict != NULL) {
            ServoValue* nodes = NULL;
            if (servo_dictionary_get_key(dict, "nodes", &nodes) == SERVO_ERROR_OK && nodes != NULL) {
                ServoListValue* nodes_list = NULL;
                if (servo_value_get_list(nodes, &nodes_list) == SERVO_ERROR_OK
                    && nodes_list != NULL) {
                    uintptr_t node_count = 0;
                    if (servo_list_get_size(nodes_list, &node_count) == SERVO_ERROR_OK
                        && node_count > 0) {
                        state->updates_with_nodes++;
                    }
                    servo_list_free(nodes_list);
                }
                servo_value_free(nodes);
            }
            servo_dictionary_free(dict);
        }
    } else {
        log("  [a11y] update payload was not a dictionary (type %d)\n", (int)type);
    }

    if (state->update_calls == 1) {
        // Show the first payload once, so a failure in this test can be read
        // straight out of the CI log.
        size_t needed = servo_value_to_json(tree_update, NULL, 0);
        if (needed > 1) {
            char* json = (char*)malloc(needed);
            if (json != NULL) {
                if (servo_value_to_json(tree_update, json, needed) > 0) {
                    log("  [a11y] first update (%zu bytes): %.400s%s\n", needed, json,
                        needed > 401 ? "..." : "");
                }
                free(json);
            }
        }
    }

    servo_value_free(tree_update);
}

/* Proves that a C embedder can receive the page's AccessKit tree.
 *
 * Accessibility is off by default and gated on the `accessibility_enabled`
 * preference, so the test turns both on, loads a small page, and waits for the
 * delegate to report an update carrying nodes. Before W6.4 there was no C API
 * for any of this, although ACCESSIBILITY_AND_PRINTING.md section 1 claimed
 * "C Embedding Exposure" existed. */
static int test_accessibility_tree_update(struct Servo* servo)
{
    log("running test_accessibility_tree_update\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    accessibility_test_state_t state;
    memset(&state, 0, sizeof(state));

    // Off by default: without this the activation below is a silent no-op.
    if (servo_set_preference_bool(servo, "accessibility_enabled", true) != SERVO_ERROR_OK) {
        log("  FAIL: could not set the accessibility_enabled preference\n");
        goto cleanup;
    }

    RenderingContext* context = servo_rendering_context_create_software(200, 200);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software returned NULL\n");
        goto cleanup;
    }

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (!builder) {
        log("  FAIL: servo_webview_builder_create returned NULL\n");
        goto cleanup;
    }

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_a11y_load_status,
        .notify_closed = on_a11y_closed,
        .notify_accessibility_tree_update = on_a11y_tree_update,
    };

    if (servo_webview_builder_set_delegate(builder, delegate) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_delegate failed\n");
        servo_webview_builder_free(builder);
        goto cleanup;
    }

    const char* url =
        "data:text/html,<html><body><button id='b'>Press me</button></body></html>";
    if (servo_webview_builder_set_url(builder, url) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_url failed\n");
        servo_webview_builder_free(builder);
        goto cleanup;
    }

    webview = servo_webview_builder_build(builder);
    if (!webview) {
        log("  FAIL: servo_webview_builder_build returned NULL\n");
        goto cleanup;
    }

    // Activate before the page loads, so the tree is built with the first
    // layout rather than requiring a second round trip.
    if (servo_webview_set_accessibility_active(webview, true) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_set_accessibility_active returned an error\n");
        goto cleanup;
    }

    if (!spin_until_count(servo, &state.update_calls, 1, 15000)) {
        log("  FAIL: no accessibility tree update reached the delegate\n");
        goto cleanup;
    }
    if (!spin_until_count(servo, &state.updates_with_nodes, 1, 15000)) {
        log("  FAIL: no accessibility tree update carried any nodes\n");
        goto cleanup;
    }
    if (state.updates_with_dictionary != state.update_calls) {
        log("  FAIL: %d of %d updates did not carry a dictionary payload\n",
            state.update_calls - state.updates_with_dictionary, state.update_calls);
        goto cleanup;
    }

    log("  PASS: %d accessibility tree update(s) reached the delegate, "
        "%d carrying nodes\n", state.update_calls, state.updates_with_nodes);
    result = 0;

cleanup:
    if (webview) {
        // Close cooperatively and wait for `notify_closed` before freeing, as
        // documented in C_API_GUIDE.md.
        servo_webview_close(webview);
        spin_until_count(servo, &state.closed, 1, 10000);
        servo_webview_free(webview);
    }
    // Leave the shared Servo as it was found.
    servo_set_preference_bool(servo, "accessibility_enabled", false);
    return result;
}

/* W4.2b. A haptic effect request only exists when the engine was built with the
 * `gamepad` feature and a gamepad is physically connected. CI has neither, so
 * the round trip itself cannot be exercised here - see the "Gamepad delegate"
 * section of C_API_GUIDE.md, which records that rather than implying the round
 * trip is covered.
 *
 * What can be checked without hardware is the ABI contract every other delegate
 * is held to: a well-formed struct is accepted, a struct compiled against a
 * smaller header is rejected instead of being read past its end, and a NULL
 * builder is reported rather than dereferenced. Building a WebView with the
 * delegate installed also proves the `Rc<dyn GamepadDelegate>` wiring does not
 * panic on the way in. */
static int test_gamepad_delegate_abi(struct Servo* servo)
{
    log("running test_gamepad_delegate_abi\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software returned NULL\n");
        return 1;
    }

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (!builder) {
        log("  FAIL: servo_webview_builder_create returned NULL\n");
        return 1;
    }

    ServoGamepadDelegate delegate = {
        .struct_size = sizeof(ServoGamepadDelegate),
        .user_data = NULL,
        // No handler: an unhandled request is failed immediately by the engine,
        // which is the documented default.
        .handle_haptic_effect_request = NULL,
    };

    if (servo_webview_builder_set_gamepad_delegate(NULL, delegate)
        != SERVO_ERROR_NULL_ARGUMENT) {
        log("  FAIL: set_gamepad_delegate did not report a NULL builder\n");
        goto cleanup;
    }
    if (servo_webview_builder_set_gamepad_delegate(builder, delegate) != SERVO_ERROR_OK) {
        log("  FAIL: set_gamepad_delegate rejected a well-formed struct\n");
        goto cleanup;
    }

    // A delegate compiled against a smaller struct must be rejected instead of
    // being read past its end (ABI_STABILITY.md section 3.3).
    {
        ServoGamepadDelegate truncated_delegate = delegate;
        truncated_delegate.struct_size = 0;
        if (servo_webview_builder_set_gamepad_delegate(builder, truncated_delegate)
            != SERVO_ERROR_INVALID_ARGUMENT) {
            log("  FAIL: set_gamepad_delegate accepted struct_size = 0\n");
            goto cleanup;
        }
    }

    webview = servo_webview_builder_build(builder);
    if (!webview) {
        log("  FAIL: servo_webview_builder_build returned NULL\n");
        goto cleanup;
    }

    log("  PASS: the gamepad delegate is ABI-versioned like the other delegates\n");
    result = 0;

cleanup:
    if (webview) {
        servo_webview_close(webview);
        servo_webview_free(webview);
    } else {
        servo_webview_builder_free(builder);
    }
    return result;
}

/* State for the W4.2c embedder-control round-trip test. */
typedef struct {
    int load_complete;
    int closed;
    /* Number of completed JavaScript evaluations and the last result. */
    int js_calls;
    char js_result[64];
    /* Calls received by the embedder's show_color_picker/show_file_picker. */
    int color_picker_calls;
    int file_picker_calls;
    /* The color the engine said the control was opened with. */
    char opened_with_color[16];
} embedder_control_state_t;

/* The two picker callbacks below do *not* receive the delegate's `user_data`:
 * their last argument is the opaque responder the C API handed over, which must
 * be passed straight back into `callback`. The test's own state therefore has to
 * be reachable another way, and a file-scope pointer is what a single-instance
 * host uses too (a host with several WebViews would key its state on `webview`).
 * See "Embedder controls" in C_API_GUIDE.md. */
static embedder_control_state_t* g_ec_state = NULL;

/* The color the delegate answers with. The page's input starts at #000000, so
 * landing on this value is only possible if the host's answer travelled back
 * into the engine. */
static const ServoRgbColor EMBEDDER_CHOSEN_COLOR = { 0x12, 0x34, 0x56 };

/* A real file for the file picker to hand back. It is created in the working
 * directory so that the path resolves for the file-manager thread exactly as it
 * does here: the manager opens every path the delegate returns and reports an
 * empty selection when one cannot be opened, so a fabricated path cannot make
 * the test pass. */
#define EMBEDDER_PICKER_FILE "capi_embedder_controls_test.tmp"

static void on_ec_load_status(struct WebView* webview, int32_t load_status, void* user_data)
{
    (void)webview;
    if (load_status == SERVO_LOAD_STATUS_COMPLETE) {
        ((embedder_control_state_t*)user_data)->load_complete = 1;
    }
}

static void on_ec_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    ((embedder_control_state_t*)user_data)->closed = 1;
}

static void on_ec_js(const char* result_json, const char* error_message, void* user_data)
{
    embedder_control_state_t* state = (embedder_control_state_t*)user_data;
    state->js_calls++;
    state->js_result[0] = '\0';
    if (result_json != NULL) {
        strncpy(state->js_result, result_json, sizeof(state->js_result) - 1);
        state->js_result[sizeof(state->js_result) - 1] = '\0';
    } else {
        log("  [js] evaluation failed: %s\n", error_message ? error_message : "(null)");
    }
}

static void on_ec_color_picker(struct WebView* webview, const ServoRgbColor* current_color,
    ServoColorPickerCallback callback, void* responder)
{
    (void)webview;
    embedder_control_state_t* state = g_ec_state;
    state->color_picker_calls++;
    if (current_color != NULL) {
        snprintf(state->opened_with_color, sizeof(state->opened_with_color), "#%02x%02x%02x",
            current_color->red, current_color->green, current_color->blue);
    } else {
        state->opened_with_color[0] = '\0';
    }
    log("  [color] show_color_picker (opened with '%s')\n",
        state->opened_with_color[0] ? state->opened_with_color : "(no color)");

    /* Answer right away, handing the responder back untouched: it is the token
     * Rust needs to resolve this particular control, not our own state. */
    if (callback != NULL) {
        callback(&EMBEDDER_CHOSEN_COLOR, responder);
    }
}

static void on_ec_file_picker(struct WebView* webview, bool allow_multiple,
    ServoFilePickerCallback callback, void* responder)
{
    (void)webview;
    embedder_control_state_t* state = g_ec_state;
    state->file_picker_calls++;
    log("  [file] show_file_picker (allow_multiple=%d)\n", allow_multiple ? 1 : 0);

    /* Hand back a file this test created: it exists wherever the suite runs
     * from, and the file manager drops paths it cannot open by returning an
     * empty selection. A non-empty `files` list below is therefore only
     * reachable if the host's answer really did travel back into the engine. */
    const char* const files[] = { EMBEDDER_PICKER_FILE };
    if (callback != NULL) {
        callback(files, 1, responder);
    }
}

/* Evaluate `script` and wait for the evaluation to complete. */
static int evaluate_and_wait(struct Servo* servo, struct WebView* webview, const char* script,
    embedder_control_state_t* state, int timeout_ms)
{
    int before = state->js_calls;
    servo_webview_evaluate_javascript(webview, script, on_ec_js, state);
    return spin_until_count(servo, &state->js_calls, before + 1, timeout_ms);
}

/* Re-evaluate `script` until its JSON result equals `expected`, spinning the
 * event loop between attempts.
 *
 * The file picker needs this: its answer travels embedder -> file-manager
 * thread -> script, so the page can trail the delegate callback by a few spins.
 * Polling instead of reading once keeps the test off that timing. */
static int spin_until_js_equals(struct Servo* servo, struct WebView* webview, const char* script,
    const char* expected, embedder_control_state_t* state, int timeout_ms)
{
    int waited_ms = 0;
    while (waited_ms < timeout_ms) {
        int before = state->js_calls;
        state->js_result[0] = '\0';
        servo_webview_evaluate_javascript(webview, script, on_ec_js, state);
        if (!spin_until_count(servo, &state->js_calls, before + 1, 1000)) {
            return 0;
        }
        if (strcmp(state->js_result, expected) == 0) {
            return 1;
        }
        sleep_ms(10);
        waited_ms += 10;
    }
    return 0;
}

/* W4.2c. Drives two embedder controls end to end: the host is asked, the host
 * answers, and the page is read back to prove the answer arrived.
 *
 * ColorPicker. A synthetic click on `<input type=color value="#000000">` is
 * enough because neither `ColorInputActivation::activation_behavior` nor the
 * input activation path checks `IsTrusted`. The delegate reports the color the
 * control was opened with (proving the request payload crossed too) and answers
 * with `EMBEDDER_CHOSEN_COLOR`; reading `input.value` back must yield "#123456".
 *
 * FilePicker. A synthetic click on `<input type=file>` reaches the file manager,
 * which asks the embedder to pick. The delegate answers with a file this test
 * created, and the test expects `files.length == 1` - the manager only returns
 * entries for paths it could actually open, so this cannot pass by accident.
 *
 * SelectElement and ContextMenu are deliberately not driven here. Both are gated
 * on a *trusted* input event - `HTMLSelectElement::activation_behavior` returns
 * early for untrusted ones, and the context menu is only built from the
 * embedder's own mouse handling - so reaching them needs synthesised input at a
 * known layout position. Their wiring ships but is unverified in CI; see the
 * "Embedder controls" section of C_API_GUIDE.md. */
static int test_embedder_controls_round_trip(struct Servo* servo)
{
    log("running test_embedder_controls_round_trip\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    embedder_control_state_t state;
    memset(&state, 0, sizeof(state));
    g_ec_state = &state;

    // The rendering context is owned by the WebView once the builder is
    // created, so only the WebView is freed (see C_API_GUIDE.md).
    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (!context) {
        log("  FAIL: servo_rendering_context_create_software returned NULL\n");
        return 1;
    }

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (!builder) {
        log("  FAIL: servo_webview_builder_create returned NULL\n");
        return 1;
    }

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_ec_load_status,
        .notify_closed = on_ec_closed,
        .show_color_picker = on_ec_color_picker,
        .show_file_picker = on_ec_file_picker,
    };
    if (servo_webview_builder_set_delegate(builder, delegate) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_delegate failed\n");
        servo_webview_builder_free(builder);
        return 1;
    }

    // `#` must be escaped as `%23`: in a data: URL a literal `#` starts the
    // fragment, which would truncate the document at the color value and leave
    // the file input unparsed.
    const char* url =
        "data:text/html,<html><body>"
        "<input type='color' id='c' value='%23000000'>"
        "<input type='file' id='f'>"
        "</body></html>";
    if (servo_webview_builder_set_url(builder, url) != SERVO_ERROR_OK) {
        log("  FAIL: servo_webview_builder_set_url failed\n");
        servo_webview_builder_free(builder);
        return 1;
    }

    webview = servo_webview_builder_build(builder);
    if (!webview) {
        log("  FAIL: servo_webview_builder_build returned NULL\n");
        return 1;
    }

    if (!spin_until_count(servo, &state.load_complete, 1, 10000)) {
        log("  FAIL: the page never finished loading\n");
        goto cleanup;
    }

    // --- ColorPicker ---
    if (!evaluate_and_wait(servo, webview, "document.getElementById('c').click()", &state, 10000)) {
        log("  FAIL: clicking the color input timed out\n");
        goto cleanup;
    }
    if (!spin_until_count(servo, &state.color_picker_calls, 1, 10000)) {
        log("  FAIL: the color input never reached show_color_picker\n");
        goto cleanup;
    }
    if (strcmp(state.opened_with_color, "#000000") != 0) {
        log("  FAIL: show_color_picker reported the current color as '%s', expected "
            "'#000000'\n",
            state.opened_with_color);
        goto cleanup;
    }

    // The delegate's color is only visible here if the engine applied it.
    if (!spin_until_js_equals(servo, webview, "document.getElementById('c').value",
            "\"#123456\"", &state, 10000)) {
        log("  FAIL: color after the response is %s, expected \"#123456\"\n", state.js_result);
        goto cleanup;
    }

    // --- FilePicker ---
    // The file manager only accepts paths it can open, so create one.
    FILE* picker_file = fopen(EMBEDDER_PICKER_FILE, "w");
    if (!picker_file) {
        log("  FAIL: could not create %s\n", EMBEDDER_PICKER_FILE);
        goto cleanup;
    }
    fputs("servo capi embedder-controls round trip\n", picker_file);
    fclose(picker_file);

    if (!evaluate_and_wait(servo, webview, "document.getElementById('f').click()", &state, 10000)) {
        log("  FAIL: clicking the file input timed out\n");
        goto cleanup;
    }
    if (!spin_until_count(servo, &state.file_picker_calls, 1, 10000)) {
        log("  FAIL: the file input never reached show_file_picker\n");
        goto cleanup;
    }

    if (!spin_until_js_equals(servo, webview, "document.getElementById('f').files.length", "1",
            &state, 10000)) {
        log("  FAIL: files.length is %s, expected 1 - the host's file selection never "
            "reached the engine\n",
            state.js_result);
        goto cleanup;
    }

    log("  PASS: color picker and file picker round-tripped through the C API\n");
    result = 0;

cleanup:
    g_ec_state = NULL;
    remove(EMBEDDER_PICKER_FILE);
    if (webview) {
        // Close cooperatively and wait for `notify_closed` before freeing, as
        // documented in C_API_GUIDE.md.
        servo_webview_close(webview);
        spin_until_count(servo, &state.closed, 1, 10000);
        servo_webview_free(webview);
    }
    return result;
}

/* State for the hang-monitor test is the shared `test_state_t`; the
 * interrupted evaluation is reported through `js_error_received` / `js_error`. */
static void on_javascript_hang_kill(const char* result_json, const char* error_message, void* user_data)
{
    test_state_t* state = (test_state_t*)user_data;
    if (error_message != NULL) {
        state->js_error_received = 1;
        strncpy(state->js_error, error_message, sizeof(state->js_error) - 1);
        state->js_error[sizeof(state->js_error) - 1] = '\0';
        log("  [js] infinite loop interrupted by the hang monitor: %s\n", error_message);
    } else {
        log("  [js] infinite loop unexpectedly completed: %s\n",
            result_json ? result_json : "(null)");
    }
}

/* Proves that a component that stops servicing hang-monitor checkpoints is
 * terminated shortly after the permanent hang timeout (5s), not immediately
 * and not never, and that the embedder observes a defined outcome rather than
 * a silent hang.
 *
 * The background hang monitor and the hung-script kill are enabled by default
 * (`servo_options_create()` -> `Opts::default()`). Evaluating an infinite
 * JavaScript loop wedges the script thread; the background hang monitor
 * detects the permanent hang, the constellation broadcasts
 * `SignalComponentExit`, and the script thread's JS interrupt terminates the
 * loop. The observable is the interrupted evaluation completing with an error
 * (it would otherwise never return).
 *
 * This test runs last: a kill leaves the event loop's script thread inert for
 * good, so the WebViews of this Servo instance are dead afterwards. The
 * suite's final `servo_free` then doubles as the clean-shutdown assertion:
 * if the kill wedged teardown, the process would hang here and the CI job
 * would time out instead of printing PASS.
 */
int test_hang_monitor_kills_hung_script(struct Servo* servo)
{
    log("running test_hang_monitor_kills_hung_script\n");
    int result = 1; // default to fail
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    const char* url = "data:text/html,<html><head><title>Hang%20Monitor</title></head>"
                      "<body>before the hang</body></html>";
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

    // Let the trivial page load finish so the script thread is up and idle
    // before wedging it.
    log("  Spinning event loop until load completes... (timeout 10s)\n");
    {
        long deadline = now_ms() + 10000;
        while (now_ms() < deadline) {
            servo_spin_event_loop(servo);

            if (state.step >= TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(15);
        }
    }
    if (state.step < TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: page load did not complete in time\n");
        goto cleanup;
    }

    // Wedge the script thread. The callback fires exactly once: with an error
    // once the hang monitor interrupts the loop (expected), or never if the
    // kill is broken.
    log("  Evaluating an infinite loop; expecting the hang monitor to kill it\n");
    if (servo_webview_evaluate_javascript(webview, "while (true) {}",
            on_javascript_hang_kill, &state) != 0) {
        log("  FAIL: servo_webview_evaluate_javascript failed to queue\n");
        goto cleanup;
    }

    long eval_started = now_ms();
    log("  Spinning event loop until the interrupted evaluation reports... (timeout 12s)\n");
    {
        long deadline = eval_started + 12000;
        while (now_ms() < deadline) {
            servo_spin_event_loop(servo);

            if (state.js_error_received) {
                break;
            }
            sleep_ms(15);
        }
    }
    long observed_ms = now_ms() - eval_started;

    if (!state.js_error_received) {
        log("  FAIL: infinite loop was not interrupted within 12s "
            "(permanent hang timeout is 5s; hung-script kill not working?)\n");
        goto cleanup;
    }

    // Only a *permanent* hang alert kills (transient alerts are diagnostics
    // only), and the permanent timeout is 5s, so the interruption must land
    // after that timeout. A much earlier interruption would mean something
    // else ended the evaluation -- e.g. an unrelated shutdown signal -- and
    // would not prove the hang-monitor kill.
    if (observed_ms < 3000) {
        log("  FAIL: evaluation interrupted after only %ld ms, too early to be "
            "the permanent-hang kill (permanent timeout is 5s)\n",
            observed_ms);
        goto cleanup;
    }
    log("  hang observed after ~%ld ms\n", observed_ms);

    result = 0;
    log("  PASS: hung script terminated with a defined outcome\n");

cleanup:
    // Safe after the kill: closing the WebView is handled by the constellation,
    // and the inert script thread only serves exit messages.
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

static bool on_custom_protocol_handler(const struct ServoWebResourceRequest* request,
                                       struct ServoWebResourceResponse* response,
                                       void* user_data)
{
    (void)user_data;
    char url[256] = {0};
    servo_web_resource_request_get_url(request, url, sizeof(url));
    log("  [custom_protocol] intercepted request for '%s'\n", url);

    const char* html = "<!DOCTYPE html><html><head><title>Custom Scheme Page</title></head>"
                       "<body>Custom Scheme Body</body></html>";
    servo_web_resource_response_set_status(response, 200, "OK");
    servo_web_resource_response_set_header(response, "Content-Type", "text/html; charset=utf-8");
    servo_web_resource_response_set_body(response, (const uint8_t*)html, strlen(html));
    return true;
}

static int test_webview_custom_scheme(struct Servo* servo)
{
    log("running test_webview_custom_scheme\n");
    int result = 1;
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_load_started = on_load_started,
        .notify_load_ended = on_load_ended,
        .notify_load_error = on_load_error,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    const char* url = "custom://test-app/index.html";
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

    log("  Spinning event loop until custom scheme load completes... (timeout 15s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 15000) {
            servo_spin_event_loop(servo);
            if (state.step >= TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    if (state.step < TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: custom scheme page load did not complete in time\n");
        goto cleanup;
    }

    if (!state.title_changed_fired || strstr(state.last_title, "Custom Scheme Page") == NULL) {
        log("  FAIL: expected title 'Custom Scheme Page', got '%s'\n", state.last_title);
        goto cleanup;
    }

    result = 0;
    log("  PASS: custom scheme test completed successfully\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

static bool on_http_intercept(struct WebView* webview,
                              const struct ServoWebResourceRequest* request,
                              struct ServoWebResourceResponse* response,
                              void* user_data)
{
    (void)webview;
    (void)user_data;
    char url[256] = {0};
    servo_web_resource_request_get_url(request, url, sizeof(url));
    log("  [http_intercept] request url: %s\n", url);

    if (strstr(url, "intercept-me.test") != NULL) {
        const char* html = "<!DOCTYPE html><html><head><title>Intercepted Page</title></head>"
                           "<body>Intercepted Whole Body</body></html>";
        servo_web_resource_response_set_status(response, 200, "OK");
        servo_web_resource_response_set_header(response, "Content-Type", "text/html; charset=utf-8");
        servo_web_resource_response_set_header(response, "X-Intercepted", "True");
        servo_web_resource_response_set_body(response, (const uint8_t*)html, strlen(html));
        return true;
    }
    return false;
}

static int test_webview_http_interception(struct Servo* servo)
{
    log("running test_webview_http_interception\n");
    int result = 1;
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_load_started = on_load_started,
        .notify_load_ended = on_load_ended,
        .notify_load_error = on_load_error,
        .intercept_request = on_http_intercept,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    const char* url = "http://intercept-me.test/page.html";
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

    log("  Spinning event loop until intercepted load completes... (timeout 15s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 15000) {
            servo_spin_event_loop(servo);
            if (state.step >= TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    if (state.step < TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: intercepted load did not complete in time\n");
        goto cleanup;
    }

    if (!state.title_changed_fired || strstr(state.last_title, "Intercepted Page") == NULL) {
        log("  FAIL: expected title 'Intercepted Page', got '%s'\n", state.last_title);
        goto cleanup;
    }

    result = 0;
    log("  PASS: http interception test completed successfully\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

static bool on_http_stream_intercept(struct WebView* webview,
                                    const struct ServoWebResourceRequest* request,
                                    struct ServoWebResourceResponse* response,
                                    struct ServoWebResourceLoad* load,
                                    void* user_data)
{
    (void)webview;
    (void)user_data;
    char url[256] = {0};
    servo_web_resource_request_get_url(request, url, sizeof(url));
    log("  [http_stream_intercept] request url: %s\n", url);

    if (strstr(url, "stream-me.test") != NULL) {
        servo_web_resource_response_set_status(response, 200, "OK");
        servo_web_resource_response_set_header(response, "Content-Type", "text/html; charset=utf-8");
        servo_web_resource_response_set_header(response, "X-Streamed", "True");
        servo_web_resource_load_intercept(load, response);

        const char* chunk1 = "<!DOCTYPE html><html><head><title>Streamed Page</title></head>";
        const char* chunk2 = "<body>Streamed Body Chunk</body></html>";
        servo_web_resource_load_send_body_data(load, (const uint8_t*)chunk1, strlen(chunk1));
        servo_web_resource_load_send_body_data(load, (const uint8_t*)chunk2, strlen(chunk2));
        servo_web_resource_load_finish(load);
        return true;
    }
    return false;
}

static int test_webview_streaming_interception(struct Servo* servo)
{
    log("running test_webview_streaming_interception\n");
    int result = 1;
    struct WebView* webview = NULL;
    test_state_t state;
    memset(&state, 0, sizeof(state));

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
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &state,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_load_started = on_load_started,
        .notify_load_ended = on_load_ended,
        .notify_load_error = on_load_error,
        .intercept_request_streaming = on_http_stream_intercept,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    const char* url = "http://stream-me.test/page.html";
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

    log("  Spinning event loop until streaming load completes... (timeout 15s)\n");
    {
        int elapsed_ms = 0;
        int poll_ms = 15;
        while (elapsed_ms < 15000) {
            servo_spin_event_loop(servo);
            if (state.step >= TEST_STEP_LOAD_COMPLETE) {
                break;
            }
            sleep_ms(poll_ms);
            elapsed_ms += poll_ms;
        }
    }

    if (state.step < TEST_STEP_LOAD_COMPLETE) {
        log("  FAIL: streaming intercepted load did not complete in time\n");
        goto cleanup;
    }

    if (!state.title_changed_fired || strstr(state.last_title, "Streamed Page") == NULL) {
        log("  FAIL: expected title 'Streamed Page', got '%s'\n", state.last_title);
        goto cleanup;
    }

    result = 0;
    log("  PASS: streaming interception test completed successfully\n");

cleanup:
    if (webview) {
        servo_webview_free(webview);
    }
    return result;
}

typedef struct {
    int count;
    char names[8][64];
    char values[8][64];
} test_cookie_visitor_state_t;

static bool on_test_cookie_visit(const struct ServoCookie* cookie,
                                uintptr_t count,
                                uintptr_t total,
                                void* user_data)
{
    (void)count;
    (void)total;
    test_cookie_visitor_state_t* s = (test_cookie_visitor_state_t*)user_data;
    if (s->count < 8 && cookie && cookie->name && cookie->value) {
        strncpy(s->names[s->count], cookie->name, sizeof(s->names[0]) - 1);
        strncpy(s->values[s->count], cookie->value, sizeof(s->values[0]) - 1);
        s->count++;
    }
    return true;
}

static int test_cookies_round_trip(struct Servo* servo)
{
    log("running test_cookies_round_trip\n");
    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (!manager) {
        log("  FAIL: servo_get_site_data_manager returned NULL\n");
        return 1;
    }

    // Clear any leftover cookies first
    servo_site_data_clear_cookies(manager);

    // 1. Set cookie A
    struct ServoCookie cookieA = {
        .name = "token",
        .value = "secret123",
        .domain = "example.com",
        .path = "/",
        .expires = 0,
        .secure = false,
        .http_only = true,
        .same_site = SERVO_COOKIE_SAME_SITE_LAX,
    };
    if (servo_site_data_set_cookie(manager, "https://example.com/", &cookieA) != 0) {
        log("  FAIL: servo_site_data_set_cookie failed for cookieA\n");
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 2. Set cookie B
    struct ServoCookie cookieB = {
        .name = "theme",
        .value = "dark",
        .domain = "example.com",
        .path = "/",
        .expires = 0,
        .secure = false,
        .http_only = false,
        .same_site = SERVO_COOKIE_SAME_SITE_LAX,
    };
    if (servo_site_data_set_cookie(manager, "https://example.com/", &cookieB) != 0) {
        log("  FAIL: servo_site_data_set_cookie failed for cookieB\n");
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 3. Query cookies for URL
    struct ServoCookie cookies[4];
    size_t count = servo_site_data_cookies_for_url(manager, "https://example.com/", cookies, 4);
    if (count < 2) {
        log("  FAIL: expected at least 2 cookies for URL, got %zu\n", count);
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 4. Visit cookies with visitor
    test_cookie_visitor_state_t visitor_state;
    memset(&visitor_state, 0, sizeof(visitor_state));
    if (servo_site_data_visit_cookies_for_url(manager, "https://example.com/", on_test_cookie_visit, &visitor_state) != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_visit_cookies_for_url returned error\n");
        servo_site_data_manager_free(manager);
        return 1;
    }
    if (visitor_state.count < 2) {
        log("  FAIL: visitor saw %d cookies, expected at least 2\n", visitor_state.count);
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 5. Delete specific cookie (token)
    if (servo_site_data_delete_cookie(manager, "https://example.com/", "token", NULL, NULL) != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_delete_cookie returned error\n");
        servo_site_data_manager_free(manager);
        return 1;
    }

    // Verify token is deleted and only theme remains
    count = servo_site_data_cookies_for_url(manager, "https://example.com/", cookies, 4);
    if (count != 1 || strcmp(cookies[0].name, "theme") != 0) {
        log("  FAIL: expected 1 cookie ('theme') after deleting 'token', got %zu\n", count);
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 6. Delete all remaining cookies
    if (servo_site_data_delete_all_cookies(manager, NULL, NULL, NULL) != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_delete_all_cookies returned error\n");
        servo_site_data_manager_free(manager);
        return 1;
    }

    count = servo_site_data_cookies_for_url(manager, "https://example.com/", cookies, 4);
    if (count != 0) {
        log("  FAIL: expected 0 cookies after delete_all, got %zu\n", count);
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    log("  PASS: test_cookies_round_trip completed successfully\n");
    return 0;
}

typedef struct {
    int count;
    char sites[8][128];
    uint32_t storage_types[8];
} test_site_visitor_state_t;

static bool on_test_site_visit(const struct ServoSiteData* site,
                               uintptr_t count,
                               uintptr_t total,
                               void* user_data)
{
    (void)count;
    (void)total;
    test_site_visitor_state_t* s = (test_site_visitor_state_t*)user_data;
    if (s->count < 8 && site && site->site) {
        strncpy(s->sites[s->count], site->site, sizeof(s->sites[0]) - 1);
        s->storage_types[s->count] = site->storage_types;
        s->count++;
    }
    return true;
}

static int test_site_storage(struct Servo* servo)
{
    log("running test_site_storage\n");
    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (!manager) {
        log("  FAIL: servo_get_site_data_manager returned NULL\n");
        return 1;
    }

    // 1. Visit sites with SERVO_STORAGE_TYPE_ALL
    test_site_visitor_state_t vstate;
    memset(&vstate, 0, sizeof(vstate));
    ServoError err = servo_site_data_visit_sites(manager, SERVO_STORAGE_TYPE_ALL, on_test_site_visit, &vstate);
    if (err != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_visit_sites returned %d\n", err);
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 2. Clear site data for specific sites with Local & Session storage flags
    const char* sites[] = { "example.com", "other.org" };
    err = servo_site_data_clear_site_data(manager, sites, 2, SERVO_STORAGE_TYPE_LOCAL | SERVO_STORAGE_TYPE_SESSION);
    if (err != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_clear_site_data returned %d\n", err);
        servo_site_data_manager_free(manager);
        return 1;
    }

    // 3. Clear site data for all storage types
    err = servo_site_data_clear_site_data(manager, sites, 1, SERVO_STORAGE_TYPE_ALL);
    if (err != SERVO_ERROR_OK) {
        log("  FAIL: servo_site_data_clear_site_data with ALL returned %d\n", err);
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    log("  PASS: test_site_storage completed successfully\n");
    return 0;
}

/* DevTools server coverage.
 *
 * The DevTools server is a Servo-level feature: it is switched on with
 * `servo_preferences_set_devtools_server` and reported through the
 * `ServoDelegate` callbacks. The server is started while the `Servo` is being
 * built, but the start notification travels through the embedder event loop, so
 * it is only delivered once that loop is pumped.
 */
#define TEST_DEVTOOLS_PORT 9222

typedef struct {
    // Set by `notify_devtools_server_started`.
    int started_fired;
    int started_port;
    int token_empty;
    char token[64];
    // Set by `request_devtools_connection`.
    int connection_requests;
} test_devtools_state_t;

static test_devtools_state_t g_devtools_state;

static void on_devtools_server_started(struct Servo* servo, uint16_t port, const char* token, void* user_data)
{
    // `servo` is NULL by contract for this callback (see
    // `ffi/capi/servo_delegate.rs`): the Rust `ServoDelegate` trait does not
    // pass a handle, so there is nothing to check here.
    (void)servo;
    log("  [devtools] notify_devtools_server_started -> port=%u\n", port);
    test_devtools_state_t* state = (test_devtools_state_t*)user_data;
    state->started_fired = 1;
    state->started_port = port;
    if (token == NULL || token[0] == '\0') {
        state->token_empty = 1;
        return;
    }
    // The `token` argument is only valid for the duration of the callback.
    strncpy(state->token, token, sizeof(state->token) - 1);
    state->token[sizeof(state->token) - 1] = '\0';
}

static bool on_request_devtools_connection(struct Servo* servo, void* user_data)
{
    (void)servo;
    test_devtools_state_t* state = (test_devtools_state_t*)user_data;
    state->connection_requests++;
    log("  [devtools] request_devtools_connection -> denying\n");
    // Deny by default: authorizing a DevTools connection lets that client run
    // script in every WebView, so a real embedder has to ask its user first.
    return false;
}

/* Integration test that verifies the DevTools server enabled by the entry point
 * really started (the start notification is only sent when the listen address
 * could be bound) and that it carries a usable port and auth token.
 *
 * `request_devtools_connection` is deliberately not exercised here: reaching it
 * needs a client that completes the Remote Debugging Protocol connection without
 * the auth token, which this test does not implement. The deny-when-no-callback
 * default is covered in `ffi/capi/servo_delegate.rs`.
 */
static int test_devtools_server_started(struct Servo* servo)
{
    log("running test_devtools_server_started\n");
    int result = 1;
    int elapsed_ms = 0;
    const int poll_ms = 15;

    while (elapsed_ms < 10000 && !g_devtools_state.started_fired) {
        servo_spin_event_loop(servo);
        sleep_ms(poll_ms);
        elapsed_ms += poll_ms;
    }

    if (!g_devtools_state.started_fired) {
        log("  FAIL: notify_devtools_server_started never fired after %dms "
            "(is port %d free?)\n", elapsed_ms, TEST_DEVTOOLS_PORT);
        goto cleanup;
    }
    if (g_devtools_state.started_port != TEST_DEVTOOLS_PORT) {
        log("  FAIL: server reported port %d, expected %d\n",
            g_devtools_state.started_port, TEST_DEVTOOLS_PORT);
        goto cleanup;
    }
    if (g_devtools_state.token_empty) {
        log("  FAIL: server reported an empty auth token\n");
        goto cleanup;
    }

    log("  [devtools] auth token: '%s'\n", g_devtools_state.token);
    log("  PASS: DevTools server started and the delegate was notified\n");
    result = 0;

cleanup:
    return result;
}

/* W5.7 — repeated create/load/close leak test.
 *
 * Cycles a WebView through create -> load -> wait-for-complete -> close ->
 * wait-for-closed -> free and samples the process RSS after every cycle. A leak
 * in the per-WebView path (the classic embedder defect) shows up as monotonic
 * growth in those samples. The cooperative close is part of the cycle on
 * purpose: a WebView that is freed while still open leaves its engine-side
 * state registered on the Servo (see C_API_GUIDE.md "Close flow example").
 *
 * Why RSS rather than an internal allocator counter: RSS is what an embedder
 * actually watches, and it covers allocations made inside the `servo_capi`
 * shared library too. `servo-allocator`'s counting allocator is not usable
 * here — it lives behind the `allocation-tracking` feature (which records a
 * backtrace per allocation and is far too slow to run in CI) and its
 * `heap_reports()` is empty on Windows, so it is not portable across the three
 * jobs that run this test.
 *
 * RSS measures allocator and OS bookkeeping as well as live memory, so the
 * verdict needs a window that has left allocator warm-up behind. Two early runs
 * of this test show how large that warm-up can be: a 50-cycle run grew 42540
 * KiB over cycles 10..49, almost all of it in a spike between cycles ~23 and
 * ~38, and then settled to ~90 KiB per cycle; a 200-cycle run of the same code
 * grew only ~90 KiB per cycle from cycle 100 onwards. The spike was arena
 * growth, not retention, so a long warm-up is needed and early samples cannot
 * feed the verdict.
 *
 * Trimming the allocator before each sample does not fix this. An earlier
 * revision called `malloc_trim(0)` (glibc), `malloc_zone_pressure_relief`
 * (macOS) or emptied the working set (Windows) first; measured, it changed
 * nothing on Linux (9572 KiB of growth over cycles 100..199 with the trim
 * against 8604 KiB without it) and on Windows it collapsed the metric - every
 * sample read 124 KiB, because the trim evicted the very pages the sample was
 * supposed to be looking at. Trimming is gone and the samples are raw.
 *
 * `heap_in_use_bytes()` supplies a second, tighter signal on glibc: the bytes
 * the allocator considers in use, which excludes the free blocks that inflate
 * RSS. RSS growth with flat in-use bytes is allocator behaviour; growth in both
 * is retention. It is a diagnostic, not an assertion, because macOS and Windows
 * have no equivalent that is cheap enough to call once per cycle.
 *
 * The test runs THREE phases of the same cycle with one step taken away at a
 * time, so growth can be attributed instead of merely observed: cycling only
 * the software rendering context, cycling a WebView that is never navigated,
 * and the full create/load/close/free cycle. A phase whose slope matches the
 * full cycle's is where the memory is going; a phase that is flat while the
 * full cycle grows puts it in the navigation path.
 *
 * FINDINGS. The WebView lifecycle is not leak-free, so this test asserts a
 * bound rather than flatness. Two leaks are known, and they are what the bound
 * absorbs (measured rates are tabulated in the "Leak testing" section of
 * `docs/embedding/CONTRIBUTING_EMBEDDING.md`):
 *
 *   1. WebView create/close/free retains roughly 80 KiB of allocator heap per
 *      cycle on Linux and 105-133 KiB of RSS per cycle on macOS. It is
 *      identical with and without a page load, and the rendering-context-only
 *      phase is flat on Linux and macOS, so it sits in WebView/Painter
 *      teardown rather than in navigation. On Linux the in-use heap tracks
 *      RSS, so it is genuine retention and not allocator slack. Not yet
 *      attributed to a component.
 *   2. Windows leaks roughly 272 KiB of RSS per cycle from rendering-context
 *      create/free ALONE, with no WebView involved. It is linear, with both
 *      halves of the measured window equal, so it does not saturate. That
 *      teardown goes through surfman, an external crate, so it is tracked as
 *      an upstream issue rather than fixed here.
 *
 * A per-step heap sample was tried and then removed: WebView teardown is
 * asynchronous, so point samples taken inside a cycle measure in-flight churn
 * rather than retention - the per-step deltas summed to -490,317 KiB against a
 * true net of +16,141 KiB on the same run. Only the phase-level net is
 * meaningful.
 *
 * The bound below is a regression tripwire, not a target: it sits above the
 * growth measured on a healthy build, so ordinary noise cannot make the test
 * flaky while a genuine per-cycle leak still trips it. `LEAK_PHASE_WARMUP`
 * cycles are discarded because one-off initialisation dominates the earliest
 * samples.
 */

/* Cycles run in each phase: 200 measured after 100 of warm-up. That keeps the
 * whole test under ~90 s on the slowest platform while giving a slope stable
 * enough to compare phases (plan W5.7 asks for a bounded runtime). */
#define LEAK_PHASE_CYCLES 300
/* Cycles discarded before growth is measured, so that allocator warm-up cannot
 * reach the verdict (see the comment block above). */
#define LEAK_PHASE_WARMUP 100
/* Maximum allowed RSS growth in bytes over the measured window of one phase.
 *
 * Set from measurement, not chosen to pass. Growth of 200 measured cycles was
 * taken on all three platforms; the largest single figure was 84,568 KiB
 * (Windows, cycling a WebView that is never navigated), against 31,772 KiB for
 * the worst Linux phase and 26,576 KiB for the worst macOS phase. This bound is
 * 131,072 KiB (128 MiB), about 1.55x that worst case, which absorbs the
 * ~11,000 KiB run-to-run swing already seen on Windows while still failing on a
 * ~1.5x regression of the leak rate.
 *
 * Passing therefore means "no worse than the leak we have already measured and
 * registered", NOT "leak-free". See the comment block above. */
#define LEAK_TEST_MAX_GROWTH_BYTES (128 * 1024 * 1024)

/* Reports the bytes the allocator considers in use, or 0 where that is
 * unavailable. Unlike RSS this excludes the free blocks the allocator is
 * holding on to, so it separates engine retention from allocator high-water
 * marks. See the comment block above. */
static size_t heap_in_use_bytes(void)
{
    // `mallinfo2` is glibc 2.33+; everywhere else (and older glibc) this is 0.
    // The version is compared arithmetically because `__GLIBC_PREREQ` may be
    // undefined outside glibc, and a function-like macro in `#if` is a syntax
    // error rather than a false condition.
#if defined(__GLIBC__) && (__GLIBC__ * 1000 + __GLIBC_MINOR__) >= 2033
    return (size_t)mallinfo2().uordblks;
#else
    return 0;
#endif
}

/* Samples the resident set size of the process, or 0 if unavailable. */
static size_t process_rss_bytes(void)
{
#ifdef _WIN32
    PROCESS_MEMORY_COUNTERS counters;
    if (!GetProcessMemoryInfo(GetCurrentProcess(), &counters, (DWORD)sizeof(counters))) {
        return 0;
    }
    return (size_t)counters.WorkingSetSize;
#elif defined(__APPLE__)
    struct mach_task_basic_info info;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count)
        != KERN_SUCCESS) {
        return 0;
    }
    return (size_t)info.resident_size;
#else
    // Linux: field 2 of /proc/self/statm is the resident set size in pages.
    FILE* statm = fopen("/proc/self/statm", "r");
    if (!statm) {
        return 0;
    }
    long virtual_pages = 0;
    long resident_pages = 0;
    int scanned = fscanf(statm, "%ld %ld", &virtual_pages, &resident_pages);
    fclose(statm);
    (void)virtual_pages;
    long page_size = sysconf(_SC_PAGESIZE);
    if (scanned != 2 || page_size <= 0) {
        return 0;
    }
    return (size_t)resident_pages * (size_t)page_size;
#endif
}

typedef struct {
    int load_complete;
    int closed;
} leak_cycle_state_t;

static void on_leak_cycle_load_status(struct WebView* webview, int32_t load_status,
    void* user_data)
{
    (void)webview;
    if (load_status == SERVO_LOAD_STATUS_COMPLETE) {
        ((leak_cycle_state_t*)user_data)->load_complete = 1;
    }
}

static void on_leak_cycle_closed(struct WebView* webview, void* user_data)
{
    (void)webview;
    ((leak_cycle_state_t*)user_data)->closed = 1;
}

/* The three lifecycle shapes the leak test compares, each one step short of the
 * previous. See the comment block above. */
typedef enum {
    // Only `servo_rendering_context_create_software` + `_free`.
    LEAK_PHASE_RENDERING_CONTEXT,
    // A whole WebView that is never navigated.
    LEAK_PHASE_WEBVIEW_NO_LOAD,
    // The real thing: create, navigate, wait for the load, close, free.
    LEAK_PHASE_FULL,
} leak_phase_kind_t;

/* Runs `LEAK_PHASE_CYCLES` cycles of one shape, logging RSS and (on glibc)
 * in-use heap for every cycle, and returns 0 when the phase's growth stays
 * within the bound. */
static int run_leak_phase(struct Servo* servo, leak_phase_kind_t kind, const char* name)
{
    log("--- [leak] phase \"%s\": %d cycles (%d warm-up + %d measured) ---\n", name,
        LEAK_PHASE_CYCLES, LEAK_PHASE_WARMUP, LEAK_PHASE_CYCLES - LEAK_PHASE_WARMUP);

    int result = 1; // default to fail
    size_t samples[LEAK_PHASE_CYCLES];
    size_t heaps[LEAK_PHASE_CYCLES];

    for (int i = 0; i < LEAK_PHASE_CYCLES; i++) {
        leak_cycle_state_t state;
        memset(&state, 0, sizeof(state));

        if (kind == LEAK_PHASE_RENDERING_CONTEXT) {
            RenderingContext* context = servo_rendering_context_create_software(100, 100);
            if (!context) {
                log("  FAIL: cycle %d: servo_rendering_context_create_software returned NULL\n", i);
                goto cleanup;
            }
            servo_rendering_context_free(context);
        } else {
            // The rendering context is owned by the WebView once the builder is
            // created, so only the WebView is freed (see C_API_GUIDE.md).
            RenderingContext* context = servo_rendering_context_create_software(100, 100);
            if (!context) {
                log("  FAIL: cycle %d: servo_rendering_context_create_software returned NULL\n", i);
                goto cleanup;
            }

            ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
            if (!builder) {
                log("  FAIL: cycle %d: servo_webview_builder_create returned NULL\n", i);
                goto cleanup;
            }
            ServoWebViewDelegate delegate = {
                .struct_size = sizeof(ServoWebViewDelegate),
                .user_data = &state,
                .notify_load_status_changed = on_leak_cycle_load_status,
                .notify_closed = on_leak_cycle_closed,
            };
            servo_webview_builder_set_delegate(builder, delegate);

            if (kind == LEAK_PHASE_FULL
                && servo_webview_builder_set_url(builder, "data:text/html,<title>leak</title>cycle")
                    != SERVO_ERROR_OK) {
                log("  FAIL: cycle %d: servo_webview_builder_set_url failed\n", i);
                goto cleanup;
            }

            struct WebView* webview = servo_webview_builder_build(builder);
            if (!webview) {
                log("  FAIL: cycle %d: servo_webview_builder_build returned NULL\n", i);
                goto cleanup;
            }

            const int poll_ms = 5;
            int elapsed_ms = 0;
            if (kind == LEAK_PHASE_FULL) {
                while (!state.load_complete && elapsed_ms < 10000) {
                    servo_spin_event_loop(servo);
                    sleep_ms(poll_ms);
                    elapsed_ms += poll_ms;
                }
                if (!state.load_complete) {
                    log("  FAIL: cycle %d: load never completed after %dms\n", i, elapsed_ms);
                    servo_webview_free(webview);
                    goto cleanup;
                }
                elapsed_ms = 0;
            }

            // Close cooperatively and wait for `notify_closed` before freeing.
            // Freeing a still-open WebView leaves the engine-side `WebView` (and
            // its painter and rendering context) registered on the `Servo`,
            // which is a leak by construction. See C_API_GUIDE.md "Close flow
            // example".
            if (servo_webview_close(webview) != SERVO_ERROR_OK) {
                log("  FAIL: cycle %d: servo_webview_close failed\n", i);
                servo_webview_free(webview);
                goto cleanup;
            }
            while (!state.closed && elapsed_ms < 10000) {
                servo_spin_event_loop(servo);
                sleep_ms(poll_ms);
                elapsed_ms += poll_ms;
            }
            if (!state.closed) {
                log("  FAIL: cycle %d: notify_closed never fired after %dms\n", i, elapsed_ms);
                servo_webview_free(webview);
                goto cleanup;
            }
            servo_webview_free(webview);
        }

        // Raw samples: see the comment block above for why nothing is trimmed
        // out of them first.
        samples[i] = process_rss_bytes();
        heaps[i] = heap_in_use_bytes();
        if (samples[i] == 0) {
            log("  FAIL: cycle %d: process RSS sampling is unavailable "
                "(see process_rss_bytes in this file)\n", i);
            goto cleanup;
        }
        if (heaps[i] != 0) {
            log("  [leak] %s cycle %3d: rss=%zu KiB; heap_in_use=%zu KiB\n", name, i,
                samples[i] / 1024, heaps[i] / 1024);
        } else {
            log("  [leak] %s cycle %3d: rss=%zu KiB\n", name, i, samples[i] / 1024);
        }
        if ((i + 1) % 25 == 0) {
            if (i >= LEAK_PHASE_WARMUP) {
                log("  [leak] === %s %d cycles, rss=%zu KiB (last 25: %+ld KiB, since cycle %d: "
                    "%+ld KiB) ===\n",
                    name, i + 1, samples[i] / 1024,
                    (long)(samples[i] / 1024) - (long)(samples[i - 24] / 1024), LEAK_PHASE_WARMUP,
                    (long)(samples[i] / 1024) - (long)(samples[LEAK_PHASE_WARMUP] / 1024));
            } else {
                log("  [leak] === %s %d cycles, rss=%zu KiB ===\n", name, i + 1,
                    samples[i] / 1024);
            }
        }
    }

    size_t warmup_rss = samples[LEAK_PHASE_WARMUP];
    size_t final_rss = samples[LEAK_PHASE_CYCLES - 1];
    size_t growth = final_rss > warmup_rss ? final_rss - warmup_rss : 0;
    const int measured_cycles = LEAK_PHASE_CYCLES - LEAK_PHASE_WARMUP;

    log("  [leak] %s: baseline (cycle %d): %zu KiB; final (cycle %d): %zu KiB\n", name,
        LEAK_PHASE_WARMUP, warmup_rss / 1024, LEAK_PHASE_CYCLES - 1, final_rss / 1024);
    log("  [leak] %s: growth over cycles %d..%d: %zu KiB (bound: %d KiB)\n", name,
        LEAK_PHASE_WARMUP, LEAK_PHASE_CYCLES - 1, growth / 1024,
        LEAK_TEST_MAX_GROWTH_BYTES / 1024);

    // Both halves of the measured window: equal slopes mean the growth is
    // steady (retention), a shrinking second half means it is tailing off.
    const int midpoint = LEAK_PHASE_WARMUP + measured_cycles / 2;
    log("  [leak] %s: first half (cycles %d..%d): %+ld KiB; second half (cycles %d..%d): %+ld "
        "KiB\n",
        name, LEAK_PHASE_WARMUP, midpoint - 1,
        (long)(samples[midpoint] / 1024) - (long)(warmup_rss / 1024), midpoint,
        LEAK_PHASE_CYCLES - 1, (long)(final_rss / 1024) - (long)(samples[midpoint] / 1024));
    if (heaps[LEAK_PHASE_WARMUP] != 0) {
        log("  [leak] %s: in-use heap: baseline %zu KiB; final %zu KiB; growth %+ld KiB\n", name,
            heaps[LEAK_PHASE_WARMUP] / 1024, heaps[LEAK_PHASE_CYCLES - 1] / 1024,
            (long)(heaps[LEAK_PHASE_CYCLES - 1] / 1024)
                - (long)(heaps[LEAK_PHASE_WARMUP] / 1024));
    }

    // The rate `LEAK_TEST_MAX_GROWTH_BYTES` was derived from, printed so that a
    // later run can be compared with that measurement instead of having to
    // recompute it from two absolute samples.
    log("  [leak] %s: RSS growth %.1f KiB/cycle (bound allows %.1f KiB/cycle)\n", name,
        (double)growth / (double)measured_cycles,
        (double)LEAK_TEST_MAX_GROWTH_BYTES / (double)measured_cycles);

    if (growth > LEAK_TEST_MAX_GROWTH_BYTES) {
        log("  FAIL: phase \"%s\" RSS grew by %zu KiB over %d cycles, above the %d KiB bound\n",
            name, growth / 1024, measured_cycles, LEAK_TEST_MAX_GROWTH_BYTES / 1024);
        goto cleanup;
    }

    log("  PASS: phase \"%s\" stayed within the RSS growth bound\n", name);
    result = 0;

cleanup:
    return result;
}

static int test_repeated_load_cycles(struct Servo* servo)
{
    log("running test_repeated_load_cycles\n");

    // Least to most involved: whatever grows here is what the later phases add
    // on top of.
    if (run_leak_phase(servo, LEAK_PHASE_RENDERING_CONTEXT, "rendering-context-only") != 0) {
        return 1;
    }
    if (run_leak_phase(servo, LEAK_PHASE_WEBVIEW_NO_LOAD, "webview-no-load") != 0) {
        return 1;
    }
    if (run_leak_phase(servo, LEAK_PHASE_FULL, "webview-load-close-free") != 0) {
        return 1;
    }

    return 0;
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

    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = wake_callback,
    };
    servo_builder_set_event_loop_waker(builder, waker);

    ServoOptions* options = servo_options_create();
    servo_options_set_multiprocess(options, false);
    servo_builder_set_options(builder, options);

    // Enable the DevTools server before building the Servo: the server is
    // started inside `servo_builder_build` and `test_devtools_server_started`
    // then only has to wait for the notification to arrive.
    ServoPreferences* preferences = servo_preferences_create();
    if (servo_preferences_set_devtools_server(preferences, true, TEST_DEVTOOLS_PORT) != SERVO_ERROR_OK) {
        log("  FAIL: servo_preferences_set_devtools_server failed\n");
        return 1;
    }
    servo_builder_set_preferences(builder, preferences);

    ServoDelegate servo_delegate = {
        .struct_size = sizeof(ServoDelegate),
        .user_data = &g_devtools_state,
        .notify_devtools_server_started = on_devtools_server_started,
        .request_devtools_connection = on_request_devtools_connection,
    };
    if (servo_builder_set_delegate(builder, servo_delegate) != SERVO_ERROR_OK) {
        log("  FAIL: servo_builder_set_delegate failed\n");
        return 1;
    }

    // Register custom scheme protocol handler
    servo_builder_register_protocol_handler(builder, "custom", on_custom_protocol_handler, NULL);

    long build_start_ms = now_ms();
    servo = servo_builder_build(builder);
    g_metric_servo_build_ms = now_ms() - build_start_ms;
    if (!servo) {
        log("  FAIL: servo_builder_build returned NULL\n");
        return 1;
    }

    servo_setup_logging(servo);

    result = test_devtools_server_started(servo);
    if (result == 0) {
        result = test_webview_load_and_screenshot(servo);
    }
    if (result == 0) {
        result = test_webview_custom_scheme(servo);
    }
    if (result == 0) {
        result = test_webview_http_interception(servo);
    }
    if (result == 0) {
        result = test_webview_streaming_interception(servo);
    }
    if (result == 0) {
        result = test_cookies_round_trip(servo);
    }
    if (result == 0) {
        result = test_site_storage(servo);
    }
    if (result == 0) {
        result = test_webview_close(servo);
    }
    if (result == 0) {
        result = test_clipboard_round_trip(servo);
    }
    if (result == 0) {
        result = test_gamepad_delegate_abi(servo);
    }
    if (result == 0) {
        result = test_embedder_controls_round_trip(servo);
    }
    if (result == 0) {
        result = test_accessibility_tree_update(servo);
    }
    if (result == 0) {
        result = test_repeated_load_cycles(servo);
    }
    // Registered last: a hang-monitor kill leaves the script thread inert for
    // good, and the suite's servo_free() below then asserts clean shutdown.
    if (result == 0) {
        result = test_hang_monitor_kills_hung_script(servo);
    }

    log("%s\n", result == 0 ? "PASS" : "FAIL");

    // W5.6: report the metrics even when a test failed, so a regression in an
    // unrelated test does not hide the numbers.
    write_observability_metrics();

    servo_free(servo);
    return result;
}
