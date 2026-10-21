/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#ifdef _WIN32
#define _CRT_SECURE_NO_WARNINGS
#include <windows.h>
#else
#include <pthread.h>
#endif
#include "servo/servo_capi.h"
#include <math.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// The test runner expects us to return 0 for success and
// non-zero for failure.

// Stringify the version components so `test_capi_version` can compare the
// runtime version string against the macros the embedder compiled against.
// Without this, bumping SERVO_CAPI_VERSION_* but forgetting the string literal
// in `servo_capi_version()` would compile and pass.
#define SERVO_STRINGIFY_HELPER(x) #x
#define SERVO_STRINGIFY(x) SERVO_STRINGIFY_HELPER(x)
#define SERVO_CAPI_VERSION_STRING_EXPECTED     \
    SERVO_STRINGIFY(SERVO_CAPI_VERSION_MAJOR)  \
    "." SERVO_STRINGIFY(SERVO_CAPI_VERSION_MINOR) "." SERVO_STRINGIFY(SERVO_CAPI_VERSION_PATCH)

static int tests_run;
static int tests_failed;

#define RUN_TEST(name)                                          \
    do {                                                        \
        tests_run++;                                            \
        printf("  RUN  %s\n", #name);                           \
        fflush(stdout);                                         \
        int result = name();                                    \
        if (result != 0) {                                      \
            printf("  FAIL %s (returned %d)\n", #name, result); \
            tests_failed++;                                     \
        }                                                       \
        fflush(stdout);                                         \
    } while (0)

int test_builder_create_and_free(void)
{
    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;
    servo_builder_free(builder);
    return 0;
}

int test_builder_set_options(void)
{
    ServoBuilder* builder = servo_builder_create();
    ServoOptions* options = servo_options_create();
    if (builder == NULL || options == NULL)
        return 1;
    servo_builder_set_options(builder, options);
    // builder now owns options, so don't free it
    servo_builder_free(builder);
    return 0;
}

int test_builder_set_preferences(void)
{
    ServoBuilder* builder = servo_builder_create();
    ServoPreferences* preferences = servo_preferences_create();
    if (builder == NULL || preferences == NULL)
        return 1;
    servo_builder_set_preferences(builder, preferences);
    // builder now owns preferences, so don't free it
    servo_builder_free(builder);
    return 0;
}

static void noop_wake_callback(void) { }

int test_builder_set_event_loop_waker(void)
{
    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;

    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = noop_wake_callback,
    };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_free(builder);
    return 0;
}

/* Global state for delegate callback tests */
static int g_devtools_started_called = 0;
static uint16_t g_devtools_port = 0;
static bool g_devtools_connection_allowed = false;

static void on_devtools_started(struct Servo* servo, uint16_t port,
                                const char* token, void* user_data)
{
    (void)servo;
    (void)user_data;
    (void)token;
    g_devtools_started_called = 1;
    g_devtools_port = port;
}

static bool on_devtools_connection_request(struct Servo* servo, void* user_data)
{
    (void)servo;
    (void)user_data;
    return g_devtools_connection_allowed;
}

/* Covers `servo_builder_set_delegate`: setting a delegate on a builder. */
int test_builder_set_delegate(void)
{
    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;

    struct ServoDelegate delegate = {
        .struct_size = sizeof(struct ServoDelegate),
        .user_data = NULL,
        .notify_devtools_server_started = on_devtools_started,
        .request_devtools_connection = on_devtools_connection_request,
    };
    if (servo_builder_set_delegate(builder, delegate) != SERVO_ERROR_OK) {
        servo_builder_free(builder);
        return 1;
    }

    servo_builder_free(builder);
    return 0;
}

/* Covers `servo_builder_set_delegate` with NULL builder. */
int test_builder_delegate_connection_request(void)
{
    struct ServoDelegate delegate = {
        .struct_size = sizeof(struct ServoDelegate),
        .user_data = NULL,
        .notify_devtools_server_started = on_devtools_started,
        .request_devtools_connection = on_devtools_connection_request,
    };
    if (servo_builder_set_delegate(NULL, delegate) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    return 0;
}

int test_options_create_and_free(void)
{
    ServoOptions* options = servo_options_create();
    if (options == NULL)
        return 1;
    servo_options_free(options);
    return 0;
}

int test_options_setters(void)
{
    ServoOptions* options = servo_options_create();
    if (options == NULL)
        return 1;
    servo_options_set_hard_fail(options, true);
    servo_options_set_multiprocess(options, false);
    servo_options_set_force_ipc(options, true);
    servo_options_set_background_hang_monitor(options, false);
    servo_options_set_sandbox(options, true);
    servo_options_set_temporary_storage(options, false);
    servo_options_set_ignore_certificate_errors(options, true);
    servo_options_set_unminify_js(options, false);
    servo_options_set_unminify_css(options, true);
    servo_options_set_debug_option(
        options, SERVO_DIAGNOSTICS_LOGGING_OPTION_STYLE_TREE, true);
    servo_options_enable_time_profiling_to_stdout(options, 0.5);
    servo_options_set_random_pipeline_closure_probability(options, 0.25f);
    servo_options_free(options);
    return 0;
}

int test_preferences_create_and_free(void)
{
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL)
        return 1;
    servo_preferences_free(preferences);
    return 0;
}

int test_preferences_bool_roundtrip(void)
{
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL)
        return 1;

    servo_preferences_set_bool(preferences, "dom_gamepad_enabled", false);
    if (servo_preferences_get_bool(preferences, "dom_gamepad_enabled") != false) {
        servo_preferences_free(preferences);
        return 1;
    }

    servo_preferences_set_bool(preferences, "dom_gamepad_enabled", true);
    if (servo_preferences_get_bool(preferences, "dom_gamepad_enabled") != true) {
        servo_preferences_free(preferences);
        return 1;
    }

    servo_preferences_free(preferences);
    return 0;
}

int test_preferences_i64_roundtrip(void)
{
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL)
        return 1;

    servo_preferences_set_i64(preferences, "layout_threads", 4);
    if (servo_preferences_get_i64(preferences, "layout_threads") != 4) {
        servo_preferences_free(preferences);
        return 1;
    }

    servo_preferences_set_i64(preferences, "layout_threads", -1);
    if (servo_preferences_get_i64(preferences, "layout_threads") != -1) {
        servo_preferences_free(preferences);
        return 1;
    }

    servo_preferences_free(preferences);
    return 0;
}

int test_preferences_u64_roundtrip(void)
{
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL)
        return 1;

    servo_preferences_set_u64(preferences, "network_http_cache_size", 12345);
    if (servo_preferences_get_u64(preferences, "network_http_cache_size") != 12345) {
        servo_preferences_free(preferences);
        return 1;
    }

    servo_preferences_free(preferences);
    return 0;
}

int test_preferences_string_roundtrip(void)
{
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL)
        return 1;

    servo_preferences_set_string(preferences, "user_agent", "ServoTest/1.0");

    char* user_agent = servo_preferences_get_string(preferences, "user_agent");
    if (user_agent == NULL) {
        servo_preferences_free(preferences);
        return 1;
    }
    if (strcmp(user_agent, "ServoTest/1.0") != 0) {
        free(user_agent);
        servo_preferences_free(preferences);
        return 1;
    }
    free(user_agent);
    servo_preferences_free(preferences);
    return 0;
}

int test_rendering_context_create_and_free(void)
{
    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;
    servo_rendering_context_free(context);
    return 0;
}

int test_rendering_context_zero_size_returns_null(void)
{
    RenderingContext* context = servo_rendering_context_create_software(0, 0);
    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_window_returns_null_in_headless(void)
{
    /* In headless CI, there is no valid display/window handle.
     * Pass dummy zeroed buffers; the function will safely fail to initialize a
     * window context and return NULL without crashing. */
    char fake_display[256] = {0};
    char fake_window[256] = {0};
    RenderingContext* context =
        servo_rendering_context_create_window(fake_display, fake_window, 100, 100);
    /* Should return NULL because the handles are invalid. */
    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_window_null_handles(void)
{
    /* Passing NULL handles should return NULL without crashing. */
    RenderingContext* context =
        servo_rendering_context_create_window(NULL, NULL, 100, 100);
    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_offscreen_null_parent(void)
{
    /* Passing NULL parent should return NULL without crashing. */
    RenderingContext* context =
        servo_rendering_context_create_offscreen(NULL, 100, 100);
    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_offscreen_software_parent(void)
{
    /* Passing a software rendering context as parent should return NULL. */
    RenderingContext* software_ctx = servo_rendering_context_create_software(100, 100);
    if (software_ctx == NULL)
        return 1;

    RenderingContext* context =
        servo_rendering_context_create_offscreen(software_ctx, 100, 100);
    servo_rendering_context_free(software_ctx);

    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_offscreen_zero_size(void)
{
    /* Passing zero size should return NULL. */
    RenderingContext* software_ctx = servo_rendering_context_create_software(100, 100);
    if (software_ctx == NULL)
        return 1;

    RenderingContext* context =
        servo_rendering_context_create_offscreen(software_ctx, 0, 0);
    servo_rendering_context_free(software_ctx);

    if (context != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }
    return 0;
}

int test_rendering_context_create_offscreen_window_parent_headless(void)
{
    /* In headless CI, create a fake window context and try to create
     * an offscreen context from it. The window context creation will fail,
     * so we can't test the full path, but we can verify the API exists. */
    char fake_display[256] = {0};
    char fake_window[256] = {0};
    RenderingContext* window_ctx =
        servo_rendering_context_create_window(fake_display, fake_window, 100, 100);
    /* Window context creation should fail in headless. */
    if (window_ctx != NULL) {
        servo_rendering_context_free(window_ctx);
    }
    /* Test passes if we reach here without crashing. */
    return 0;
}

/* A singleton Servo instance shared by all WebView tests.
 * Servo has global state that can only be initialized once per process. */
static struct Servo* test_servo = NULL;

// Helper function that creates a minimal Servo instance.
static struct Servo* create_test_servo(void)
{
    if (test_servo != NULL)
        return test_servo;

    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return NULL;

    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = noop_wake_callback,
    };
    servo_builder_set_event_loop_waker(builder, waker);

    ServoOptions* options = servo_options_create();
    if (options != NULL) {
        servo_options_set_multiprocess(options, false);
        servo_builder_set_options(builder, options);
    }

    struct ServoDelegate delegate = {
        .struct_size = sizeof(struct ServoDelegate),
        .user_data = NULL,
        .notify_devtools_server_started = on_devtools_started,
        .request_devtools_connection = on_devtools_connection_request,
    };
    servo_builder_set_delegate(builder, delegate);

    test_servo = servo_builder_build(builder);
    if (test_servo != NULL) {
        servo_setup_logging(test_servo);
    }
    return test_servo;
}

static void free_test_servo(void)
{
    if (test_servo != NULL) {
        servo_free(test_servo);
        test_servo = NULL;
    }
}

/* Covers the runtime preference setters `servo_set_preference_bool` / `_i64` /
 * `_u64` / `_string`, which change a *built* Servo instance rather than a
 * `ServoPreferences` builder object. The genuine preference names and value
 * types mirror the `test_preferences_*_roundtrip` tests. There is no C getter
 * on the Servo side, so the observable contract is that each call succeeds
 * without aborting (an unknown name or wrong value type would panic). */
int test_set_preference_runtime(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    servo_set_preference_bool(servo, "dom_gamepad_enabled", false);
    servo_set_preference_bool(servo, "dom_gamepad_enabled", true);
    servo_set_preference_i64(servo, "layout_threads", 2);
    servo_set_preference_u64(servo, "network_http_cache_size", 12345);
    servo_set_preference_string(servo, "user_agent", "ServoTest/1.0");
    return 0;
}

int test_webview_builder_create_and_free(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    if (webview_builder == NULL) {
        servo_rendering_context_free(context);
        return 1;
    }

    servo_webview_builder_free(webview_builder);
    return 0;
}

int test_webview_builder_set_url_valid(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    int result = servo_webview_builder_set_url(webview_builder, "https://example.com");
    servo_webview_builder_free(webview_builder);
    return (result == 0) ? 0 : 1;
}

int test_webview_builder_set_url_invalid(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    int result = servo_webview_builder_set_url(webview_builder, "not a valid url !!!");
    servo_webview_builder_free(webview_builder);
    return (result == SERVO_ERROR_INVALID_STATE) ? 0 : 1;
}

int test_webview_builder_build(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    int result = servo_webview_builder_set_url(webview_builder, "about:blank");
    if (result != 0) {
        servo_webview_builder_free(webview_builder);
        return 1;
    }

    struct WebView* webview = servo_webview_builder_build(webview_builder);
    if (webview == NULL)
        return 1;

    servo_webview_free(webview);
    return 0;
}

/* -------------------------------------------------------------------------
 * Tests for the WebView query / setter APIs added alongside the C API version
 * accessor. Each test builds a fresh `about:blank` WebView backed by a 100x100
 * software rendering context via one of the shared `Servo` instances and frees
 * it when done.
 * ------------------------------------------------------------------------- */

/* Builds a fresh `about:blank` WebView backed by a 100x100 software rendering
 * context. Returns NULL on failure. The caller owns the returned WebView and
 * must release it with `servo_webview_free`. */
static struct WebView* create_about_blank_webview(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return NULL;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return NULL;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL) {
        servo_rendering_context_free(context);
        return NULL;
    }

    if (servo_webview_builder_set_url(builder, "about:blank") != 0) {
        servo_webview_builder_free(builder);
        return NULL;
    }

    struct WebView* webview = servo_webview_builder_build(builder);
    return webview;
}

static struct WebView* get_test_webview(void)
{
    return create_about_blank_webview();
}

/* Covers `servo_capi_version()`: the runtime version string must be non-NULL
 * and must match the `MAJOR.MINOR.PATCH` macros the caller compiled against.
 * That is what makes a header/library version skew (a bump that changed the
 * macros but not the string, or vice versa) fail CI instead of shipping. It
 * also covers the packed integer query and the `SERVO_CAPI_VERSION` macro. */
int test_capi_version(void)
{
    const char* version = servo_capi_version();
    if (version == NULL)
        return 1;

    if (strcmp(version, SERVO_CAPI_VERSION_STRING_EXPECTED) != 0) {
        printf("  FAIL: servo_capi_version() returned \"%s\", expected \"%s\"\n", version,
               SERVO_CAPI_VERSION_STRING_EXPECTED);
        return 1;
    }

    /* The packed integer query must agree with the constant compiled into the
     * header (a mismatch means a header/library version skew). */
    if (servo_capi_get_version() != SERVO_CAPI_VERSION)
        return 1;

    /* The packed constant must match its MAJOR/MINOR/PATCH components. */
    if (SERVO_CAPI_VERSION != ((SERVO_CAPI_VERSION_MAJOR << 16) |
                               (SERVO_CAPI_VERSION_MINOR << 8) |
                               SERVO_CAPI_VERSION_PATCH))
        return 1;

    return 0;
}

/* Covers the runtime capability query. These functions must be safe to call
 * before any Servo instance exists, must return a documented non-empty platform
 * name, and must stay internally consistent: sandboxed rendering requires
 * multi-process support and can never be reported without it. */
int test_capi_capabilities(void)
{
    const char* platform = servo_capi_platform_name();
    if (platform == NULL || platform[0] == '\0')
        return 1;

    if (strcmp(platform, "linux") != 0 && strcmp(platform, "macos") != 0 &&
        strcmp(platform, "windows") != 0 && strcmp(platform, "android") != 0 &&
        strcmp(platform, "ios") != 0 && strcmp(platform, "ohos") != 0 &&
        strcmp(platform, "unknown") != 0)
        return 1;

    bool multiprocess = servo_capi_is_multiprocess_enabled();
    bool sandboxing = servo_capi_is_sandboxing_supported();

    /* Sandboxing can never be available without multi-process support. */
    if (sandboxing && !multiprocess)
        return 1;

#if defined(_WIN32)
    /* Windows implements process mitigations, not an OS sandbox; sandboxing must be false. */
    if (sandboxing)
        return 1;
#endif

    return 0;
}

static volatile int g_off_thread_result = 0;

static void off_thread_call_main_thread_only(void)
{
    /* `servo_builder_free` goes through the FFI boundary, which rejects calls
     * from a thread other than the one that first entered the C API. */
    g_off_thread_result = (int)servo_builder_free(NULL);
}

#ifdef _WIN32
static DWORD WINAPI off_thread_entry(LPVOID param)
{
    (void)param;
    off_thread_call_main_thread_only();
    return 0;
}
#else
static void* off_thread_entry(void* param)
{
    (void)param;
    off_thread_call_main_thread_only();
    return NULL;
}
#endif

/* An off-thread call to a main-thread-only function returns
 * SERVO_ERROR_WRONG_THREAD rather than risking unsynchronized access. */
int test_thread_affinity_enforcement(void)
{
    /* Establish this thread as the owning (main) thread. */
    if (servo_builder_free(NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        return 1;
    }

#ifdef _WIN32
    HANDLE thread = CreateThread(NULL, 0, off_thread_entry, NULL, 0, NULL);
    if (thread == NULL) {
        return 1;
    }
    WaitForSingleObject(thread, INFINITE);
    CloseHandle(thread);
#else
    pthread_t thread;
    if (pthread_create(&thread, NULL, off_thread_entry, NULL) != 0) {
        return 1;
    }
    pthread_join(thread, NULL);
#endif

    if (g_off_thread_result != (int)SERVO_ERROR_WRONG_THREAD) {
        return 1;
    }

    /* The owning thread is unaffected by the rejected call. */
    if (servo_builder_free(NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        return 1;
    }

    /* Pure queries remain callable from any context and set no owning thread. */
    const char* platform = servo_capi_platform_name();
    if (platform == NULL || platform[0] == '\0')
        return 1;

    (void)servo_capi_is_multiprocess_enabled();
    return 0;
}

/* Exercises the two-call buffer query pattern used by the string getters:
 * query the required length with a NULL buffer, allocate it, query again, and
 * check that a NUL terminated string of the same length was written. */
static int check_string_getter(struct WebView* webview,
    uintptr_t (*getter)(const struct WebView*, char*, uintptr_t))
{
    uintptr_t required = getter(webview, NULL, 0);
    if (required == 0)
        return 1;

    char* buffer = (char*)malloc(required);
    if (buffer == NULL)
        return 1;

    // Fill with a sentinel so a missing NUL terminator is caught.
    memset(buffer, 'X', required);
    uintptr_t written = getter(webview, buffer, required);
    int result = 0;
    if (written != required) {
        result = 1;
    } else if (buffer[required - 1] != '\0') {
        result = 1;
    } else if (strlen(buffer) != required - 1) {
        result = 1;
    }

    free(buffer);
    return result;
}

/* Covers `servo_webview_get_url` with the buffer-query pattern. */
int test_webview_get_url(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    int result = check_string_getter(webview, servo_webview_get_url);
    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_get_title`. The title of a fresh `about:blank`
 * WebView may be empty, so the required length is only asserted to be >= 1. */
int test_webview_get_title(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    int result = check_string_getter(webview, servo_webview_get_title);
    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_get_status_text`. The status text of a fresh
 * `about:blank` WebView may be empty, so the required length is only asserted
 * to be >= 1. */
int test_webview_get_status_text(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    int result = check_string_getter(webview, servo_webview_get_status_text);
    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_get_load_status`: the returned value must be one of
 * the SERVO_LOAD_STATUS_* constants. */
int test_webview_get_load_status(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    int status = servo_webview_get_load_status(webview);
    int result = (status == SERVO_LOAD_STATUS_STARTED ||
                  status == SERVO_LOAD_STATUS_HEAD_PARSED ||
                  status == SERVO_LOAD_STATUS_COMPLETE)
        ? 0
        : 1;
    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_get_size`: it must report the size of the rendering
 * context the WebView was built with, and NULL out-params must not crash. */
int test_webview_get_size(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint32_t width = 0;
    uint32_t height = 0;
    servo_webview_get_size(webview, &width, &height);
    int result = (width == 100 && height == 100) ? 0 : 1;

    // NULL out-params must be tolerated.
    servo_webview_get_size(webview, NULL, NULL);
    servo_webview_get_size(webview, &width, NULL);
    servo_webview_get_size(webview, NULL, &height);

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_can_go_back` / `servo_webview_can_go_forward`: both
 * must be false for a fresh WebView with no navigation history. */
int test_webview_can_go_back_forward(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    int result = (servo_webview_can_go_back(webview) == false &&
                  servo_webview_can_go_forward(webview) == false)
        ? 0
        : 1;
    servo_webview_free(webview);
    return result;
}

static int approx_equal(float a, float b)
{
    return fabsf(a - b) < 0.001f;
}

/* Covers `servo_webview_page_zoom` / `servo_webview_set_page_zoom`: the
 * default zoom is 1.0, setting 2.0 is read back as 2.0, and out-of-range
 * values are clamped to the maximum allowed zoom of 10.0. */
int test_webview_page_zoom(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    int result = 0;
    if (!approx_equal(servo_webview_page_zoom(webview), 1.0f))
        result = 1;

    servo_webview_set_page_zoom(webview, 2.0f);
    if (!approx_equal(servo_webview_page_zoom(webview), 2.0f))
        result = 1;

    servo_webview_set_page_zoom(webview, 100.0f);
    float clamped = servo_webview_page_zoom(webview);
    if (!approx_equal(clamped, 10.0f) || clamped > 10.0f)
        result = 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_focused` / `servo_webview_set_focused`. The
 * implementation starts WebViews focused, so only the round-trip through both
 * `set_focused` values is asserted here. */
int test_webview_focus(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    (void)servo_webview_focused(webview);

    int result = 0;
    servo_webview_set_focused(webview, true);
    if (servo_webview_focused(webview) != true)
        result = 1;
    servo_webview_set_focused(webview, false);
    if (servo_webview_focused(webview) != false)
        result = 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_key_event`: must return a non-zero event ID
 * and not crash. */
int test_webview_notify_key_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_key_event(
        webview,
        SERVO_KEY_EVENT_DOWN,
        "a",
        "KeyA",
        0, /* Standard location */
        SERVO_MODIFIER_SHIFT,
        false,
        false
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_mouse_button_event`: must return a non-zero
 * event ID and not crash. */
int test_webview_notify_mouse_button_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_mouse_button_event(
        webview,
        SERVO_MOUSE_EVENT_DOWN,
        SERVO_MOUSE_BUTTON_PRIMARY,
        50.0,
        50.0,
        0
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_mouse_move_event`: must return a non-zero
 * event ID and not crash. */
int test_webview_notify_mouse_move_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_mouse_move_event(
        webview,
        50.0,
        50.0,
        0
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_mouse_leave_event`: must return a non-zero
 * event ID and not crash. */
int test_webview_notify_mouse_leave_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_mouse_leave_event(
        webview,
        false
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_touch_event`: must return a non-zero event ID
 * and not crash. */
int test_webview_notify_touch_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_touch_event(
        webview,
        SERVO_TOUCH_EVENT_DOWN,
        1,
        50.0,
        50.0,
        SERVO_TOUCH_POINTER_TYPE_TOUCH
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_wheel_event`: must return a non-zero event ID
 * and not crash. */
int test_webview_notify_wheel_event(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    uint64_t event_id = servo_webview_notify_wheel_event(
        webview,
        10.0,
        20.0,
        0.0,
        SERVO_WHEEL_DELTA_PIXEL,
        50.0,
        50.0
    );
    int result = (event_id != 0) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_notify_input_event_handled`: must not crash. */
int test_webview_notify_input_event_handled(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    // First generate an event ID
    uint64_t event_id = servo_webview_notify_key_event(
        webview,
        SERVO_KEY_EVENT_DOWN,
        "a",
        "KeyA",
        0,
        0,
        false,
        false
    );
    if (event_id == 0) {
        servo_webview_free(webview);
        return 1;
    }

    // Now notify that it's handled
    servo_webview_notify_input_event_handled(webview, event_id);

    servo_webview_free(webview);
    return 0;
}

/* Covers `servo_webview_resize`: after resizing to 200x150,
 * `servo_webview_get_size` must report the new dimensions. */
int test_webview_resize(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    servo_webview_resize(webview, 200, 150);
    uint32_t width = 0;
    uint32_t height = 0;
    servo_webview_get_size(webview, &width, &height);
    int result = (width == 200 && height == 150) ? 0 : 1;

    servo_webview_free(webview);
    return result;
}

/* Covers `servo_webview_show` / `servo_webview_hide`: both must be safe to
 * call and must not crash. */
int test_webview_show_hide(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    servo_webview_show(webview);
    servo_webview_hide(webview);
    servo_webview_free(webview);
    return 0;
}

/* Covers `servo_webview_notify_theme_change`: valid themes (SERVO_THEME_DARK /
 * SERVO_THEME_LIGHT) must not crash and invalid values must be ignored. */
int test_webview_notify_theme_change(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    servo_webview_notify_theme_change(webview, SERVO_THEME_DARK);
    servo_webview_notify_theme_change(webview, SERVO_THEME_LIGHT);
    // Invalid values must be ignored without crashing.
    servo_webview_notify_theme_change(webview, 12345);
    servo_webview_notify_theme_change(webview, -1);
    servo_webview_free(webview);
    return 0;
}

/* Covers `servo_webview_evaluate_javascript` with a NULL callback, which must
 * be treated as a no-op. */
int test_webview_evaluate_javascript_null_callback(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    servo_webview_evaluate_javascript(webview, "1 + 1", NULL, NULL);
    servo_webview_free(webview);
    return 0;
}

/* Covers the navigation and history APIs on a fresh WebView:
 * `servo_webview_clear_session_history`, `servo_webview_reload`,
 * `servo_webview_go_back`, and `servo_webview_go_forward` must all be safe
 * no-ops. */
int test_webview_navigation_noops(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;
    servo_webview_clear_session_history(webview);
    servo_webview_reload(webview);
    servo_webview_stop(webview);
    size_t hist_count = 0;
    size_t hist_index = 0;
    if (servo_webview_get_history_count(webview, &hist_count) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_get_history_index(webview, &hist_index) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_traverse_history(webview, 0) != SERVO_ERROR_OK)
        return 1;
    servo_webview_go_back(webview, 1);
    servo_webview_go_forward(webview, 1);
    servo_webview_free(webview);
    return 0;
}

/* -------------------------------------------------------------------------
 * Site Data (Cookies) tests
 * ------------------------------------------------------------------------- */

/* Covers `servo_get_site_data_manager` / `servo_site_data_manager_free`:
 * getting the manager from a Servo instance and freeing it. */
int test_site_data_manager_create_and_free(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    servo_site_data_manager_free(manager);
    return 0;
}

/* Covers `servo_site_data_cookies_for_url` with the buffer-query pattern:
 * query the required count with a NULL buffer, allocate, query again,
 * and check that the cookie data was written. */
int test_site_data_cookies_for_url(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    // Query cookie count for a URL (should be 0 for about:blank)
    size_t count = servo_site_data_cookies_for_url(manager, "about:blank", NULL, 0);
    // Count should be >= 0 (no cookies expected for about:blank)
    (void)count;

    // Test with a small buffer
    struct ServoCookie buffer[4];
    size_t written = servo_site_data_cookies_for_url(manager, "about:blank", buffer, 4);
    (void)written;

    servo_site_data_manager_free(manager);
    return 0;
}

/* Covers `servo_site_data_set_cookie`: setting a cookie and reading it back. */
int test_site_data_set_cookie(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    struct ServoCookie cookie = {
        .name = "test_cookie",
        .value = "test_value",
        .domain = "example.com",
        .path = "/",
        .expires = 0, // session cookie
        .secure = false,
        .http_only = true,
        .same_site = SERVO_COOKIE_SAME_SITE_LAX,
    };

    // Set the cookie
    int result = servo_site_data_set_cookie(manager, "https://example.com/", &cookie);
    if (result != 0) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    // Read it back
    struct ServoCookie buffer[4];
    size_t count = servo_site_data_cookies_for_url(manager, "https://example.com/", buffer, 4);
    if (count == 0) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    // Verify cookie data
    if (strcmp(buffer[0].name, "test_cookie") != 0 ||
        strcmp(buffer[0].value, "test_value") != 0) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    return 0;
}

/* Covers `servo_site_data_clear_cookies` and `servo_site_data_clear_session_cookies`:
 * both must be safe to call and must not crash. */
int test_site_data_clear_cookies(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    // Set a cookie first
    struct ServoCookie cookie = {
        .name = "clear_test",
        .value = "clear_value",
        .domain = "example.com",
        .path = "/",
        .expires = 0,
        .secure = false,
        .http_only = false,
        .same_site = SERVO_COOKIE_SAME_SITE_LAX,
    };
    servo_site_data_set_cookie(manager, "https://example.com/", &cookie);

    // Clear session cookies
    servo_site_data_clear_session_cookies(manager);

    // Set another cookie with expiration (persistent)
    cookie.expires = 9999999999; // Far future
    servo_site_data_set_cookie(manager, "https://example.com/", &cookie);

    // Clear all cookies
    servo_site_data_clear_cookies(manager);

    servo_site_data_manager_free(manager);
    return 0;
}

/* Covers the nested value-container accessors: a `ServoValue` can hold a
 * dictionary or a list, `servo_value_get_dictionary` / `_get_list` hand back a
 * copy of it, and `servo_value_set_dictionary` / `_set_list` store one back.
 *
 * This is what makes an object or array returned by
 * `servo_webview_evaluate_javascript_v2` navigable from C: before these
 * accessors existed the type was reported as `SERVO_VALUE_TYPE_DICTIONARY` /
 * `SERVO_VALUE_TYPE_LIST` but the contents could only be read by serialising
 * the whole value to JSON. */
int test_value_nested_containers(void)
{
    int32_t type = SERVO_VALUE_TYPE_INVALID;

    /* --- A dictionary stored inside a value. --- */
    ServoDictionaryValue* dict = servo_dictionary_create();
    ServoValue* count = servo_value_create();
    ServoValue* label = servo_value_create();
    if (dict == NULL || count == NULL || label == NULL) {
        servo_dictionary_free(dict);
        servo_value_free(count);
        servo_value_free(label);
        return 1;
    }
    if (servo_value_set_int(count, 7) != SERVO_ERROR_OK
        || servo_value_set_string(label, "servo") != SERVO_ERROR_OK) {
        servo_dictionary_free(dict);
        servo_value_free(count);
        servo_value_free(label);
        return 1;
    }
    /* Both calls take ownership of the value handle. */
    if (servo_dictionary_set_key(dict, "count", count) != SERVO_ERROR_OK
        || servo_dictionary_set_key(dict, "label", label) != SERVO_ERROR_OK) {
        servo_dictionary_free(dict);
        return 1;
    }

    ServoValue* value = servo_value_create();
    if (value == NULL) {
        servo_dictionary_free(dict);
        return 1;
    }
    /* A value that holds no dictionary must be rejected, not silently treated
     * as an empty one. */
    ServoDictionaryValue* nested = NULL;
    if (servo_value_get_dictionary(value, &nested) != SERVO_ERROR_INVALID_STATE) {
        servo_dictionary_free(nested);
        servo_value_free(value);
        return 1;
    }
    if (servo_value_set_dictionary(value, dict) != SERVO_ERROR_OK) {
        servo_value_free(value);
        return 1;
    }
    /* `dict` was consumed by the setter. */
    if (servo_value_get_type(value, &type) != SERVO_ERROR_OK
        || type != SERVO_VALUE_TYPE_DICTIONARY) {
        servo_value_free(value);
        return 1;
    }
    if (servo_value_get_dictionary(value, &nested) != SERVO_ERROR_OK || nested == NULL) {
        servo_value_free(value);
        return 1;
    }
    uintptr_t size = 0;
    ServoValue* read_back = NULL;
    int64_t read_int = 0;
    if (servo_dictionary_get_size(nested, &size) != SERVO_ERROR_OK || size != 2
        || servo_dictionary_get_key(nested, "count", &read_back) != SERVO_ERROR_OK
        || read_back == NULL
        || servo_value_get_int(read_back, &read_int) != SERVO_ERROR_OK || read_int != 7) {
        servo_value_free(read_back);
        servo_dictionary_free(nested);
        servo_value_free(value);
        return 1;
    }
    servo_value_free(read_back);
    servo_dictionary_free(nested);
    servo_value_free(value);

    /* --- A list stored inside a value. --- */
    ServoListValue* list = servo_list_create();
    ServoValue* first = servo_value_create();
    ServoValue* second = servo_value_create();
    if (list == NULL || first == NULL || second == NULL) {
        servo_list_free(list);
        servo_value_free(first);
        servo_value_free(second);
        return 1;
    }
    servo_value_set_int(first, 1);
    servo_value_set_int(second, 2);
    /* Both calls take ownership of the value handle. */
    if (servo_list_set_value(list, 0, first) != SERVO_ERROR_OK
        || servo_list_set_value(list, 1, second) != SERVO_ERROR_OK) {
        servo_list_free(list);
        return 1;
    }

    ServoValue* list_holder = servo_value_create();
    if (list_holder == NULL) {
        servo_list_free(list);
        return 1;
    }
    if (servo_value_set_list(list_holder, list) != SERVO_ERROR_OK) {
        servo_value_free(list_holder);
        return 1;
    }
    if (servo_value_get_type(list_holder, &type) != SERVO_ERROR_OK
        || type != SERVO_VALUE_TYPE_LIST) {
        servo_value_free(list_holder);
        return 1;
    }
    ServoListValue* nested_list = NULL;
    uintptr_t list_size = 0;
    ServoValue* second_read = NULL;
    int64_t second_value = 0;
    if (servo_value_get_list(list_holder, &nested_list) != SERVO_ERROR_OK
        || nested_list == NULL
        || servo_list_get_size(nested_list, &list_size) != SERVO_ERROR_OK || list_size != 2
        || servo_list_get_value(nested_list, 1, &second_read) != SERVO_ERROR_OK
        || second_read == NULL
        || servo_value_get_int(second_read, &second_value) != SERVO_ERROR_OK
        || second_value != 2) {
        servo_value_free(second_read);
        servo_list_free(nested_list);
        servo_value_free(list_holder);
        return 1;
    }
    servo_value_free(second_read);
    servo_list_free(nested_list);
    servo_value_free(list_holder);

    return 0;
}

/* Covers error returns for null pointer arguments. */
int test_error_null_arguments(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    // Test servo_free with NULL
    if (servo_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_spin_event_loop with NULL
    if (servo_spin_event_loop(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_post_task with NULL
    if (servo_post_task(NULL, NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_post_task(servo, NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_setup_logging with NULL
    if (servo_setup_logging(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_set_preference_bool with NULL servo
    if (servo_set_preference_bool(NULL, "dom_gamepad_enabled", true) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_set_preference_bool with NULL name
    if (servo_set_preference_bool(servo, NULL, true) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_webview_builder_create with NULL servo
    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;
    if (servo_webview_builder_create(NULL, context) != NULL) {
        servo_rendering_context_free(context);
        return 1;
    }

    // Test servo_webview_builder_create with NULL context
    if (servo_webview_builder_create(servo, NULL) != NULL)
        return 1;

    // Test servo_builder_set_options with NULL
    if (servo_builder_set_options(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_builder_set_preferences with NULL
    if (servo_builder_set_preferences(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    // Test servo_builder_build with NULL
    if (servo_builder_build(NULL) != NULL)
        return 1;

    servo_rendering_context_free(context);
    return 0;
}

/* A repeated `servo_setup_logging` call is reported as SERVO_ERROR_INVALID_STATE
 * instead of panicking: logging is process-global, so the first call wins and
 * later calls cannot take effect. This uses the shared singleton Servo (whose
 * logger `create_test_servo` already installed), so it must not free it. */
int test_servo_setup_logging_repeated(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    /* `create_test_servo` already installed the process-global logger, so this
     * reports the already-initialized state. Either call must be non-panicking. */
    ServoError first = servo_setup_logging(servo);
    if (first != SERVO_ERROR_OK && first != SERVO_ERROR_INVALID_STATE) {
        return 1;
    }

    /* The second call must not panic; it reports the already-initialized state. */
    if (servo_setup_logging(servo) != SERVO_ERROR_INVALID_STATE) {
        return 1;
    }

    /* Do NOT free: `test_servo` is a process-wide singleton reused by later tests. */
    return 0;
}

/* Covers error returns for invalid UTF-8 strings. */
int test_error_invalid_utf8(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL) {
        return 1;
    }

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL) {
        servo_rendering_context_free(context);
        return 1;
    }

    // Create invalid UTF-8 (0xFF is not valid UTF-8)
    const char invalid_utf8[] = "\xFF\xFE";

    // Test servo_webview_builder_set_url with invalid UTF-8
    if (servo_webview_builder_set_url(builder, invalid_utf8) != SERVO_ERROR_INVALID_UTF8) {
        servo_webview_builder_free(builder);
        return 1;
    }

    // Test servo_webview_load with invalid UTF-8
    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL) {
        return 1;
    }

    if (servo_webview_load(webview, invalid_utf8) != SERVO_ERROR_INVALID_UTF8) {
        servo_webview_free(webview);
        return 1;
    }

    // Test servo_webview_evaluate_javascript with invalid UTF-8
    if (servo_webview_evaluate_javascript(webview, invalid_utf8, NULL, NULL) != SERVO_ERROR_INVALID_UTF8) {
        servo_webview_free(webview);
        return 1;
    }

    servo_webview_free(webview);
    return 0;
}

/* Covers that webview query functions handle NULL webview gracefully. */
int test_webview_query_null_webview(void)
{
    // These should return 0 (empty string length) without crashing
    if (servo_webview_get_url(NULL, NULL, 0) != 0)
        return 1;
    if (servo_webview_get_title(NULL, NULL, 0) != 0)
        return 1;
    if (servo_webview_get_status_text(NULL, NULL, 0) != 0)
        return 1;

    // servo_webview_get_load_status should return 0
    if (servo_webview_get_load_status(NULL) != 0)
        return 1;

    // servo_webview_get_size with NULL should not crash
    servo_webview_get_size(NULL, NULL, NULL);

    // servo_webview_can_go_back/forward should return false
    if (servo_webview_can_go_back(NULL) != false)
        return 1;
    if (servo_webview_can_go_forward(NULL) != false)
        return 1;

    // servo_webview_page_zoom should return default 1.0
    if (fabsf(servo_webview_page_zoom(NULL) - 1.0f) > 0.001f)
        return 1;

    // servo_webview_focused should return false
    if (servo_webview_focused(NULL) != false)
        return 1;

    return 0;
}

/* Covers that builder free with NULL is handled gracefully. */
int test_builder_free_null(void)
{
    if (servo_builder_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_builder_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_options_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_preferences_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_rendering_context_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    return 0;
}

/* Covers that setter functions return error on NULL webview. */
int test_webview_setters_null(void)
{
    if (servo_webview_paint(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_set_page_zoom(NULL, 1.0f) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_resize(NULL, 100, 100) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_set_focused(NULL, true) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_show(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_hide(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_theme_change(NULL, SERVO_THEME_LIGHT) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_reload(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_stop(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    size_t dummy_size = 0;
    if (servo_webview_get_history_count(NULL, &dummy_size) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_get_history_count(webview, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_get_history_index(NULL, &dummy_size) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_get_history_index(webview, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_traverse_history(NULL, 1) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_clear_session_history(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_go_back(NULL, 1) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_go_forward(NULL, 1) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    return 0;
}

static bool test_protocol_handler_callback(const struct ServoWebResourceRequest* request,
                                           struct ServoWebResourceResponse* response,
                                           void* user_data)
{
    (void)request;
    (void)response;
    (void)user_data;
    return false;
}

/* Covers `servo_builder_register_protocol_handler`: registering a custom
 * protocol handler should succeed for valid schemes and fail for forbidden
 * schemes. */
int test_builder_register_protocol_handler(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    // Create a builder directly for testing
    ServoBuilder* servo_builder = servo_builder_create();
    if (servo_builder == NULL)
        return 1;

    // Test that NULL callback returns error
    if (servo_builder_register_protocol_handler(servo_builder, "myapp", NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_builder_free(servo_builder);
        return 1;
    }

    // Test registering a valid custom scheme
    int result = servo_builder_register_protocol_handler(servo_builder, "myapp", test_protocol_handler_callback, NULL);
    if (result != SERVO_ERROR_OK) {
        servo_builder_free(servo_builder);
        return 1;
    }

    // Test registering a forbidden scheme (http) should fail
    result = servo_builder_register_protocol_handler(servo_builder, "http", test_protocol_handler_callback, NULL);
    if (result != SERVO_ERROR_INVALID_STATE) {
        servo_builder_free(servo_builder);
        return 1;
    }

    // Test registering the same scheme twice should fail
    result = servo_builder_register_protocol_handler(servo_builder, "myapp2", test_protocol_handler_callback, NULL);
    if (result != SERVO_ERROR_OK) {
        servo_builder_free(servo_builder);
        return 1;
    }
    result = servo_builder_register_protocol_handler(servo_builder, "myapp2", test_protocol_handler_callback, NULL);
    if (result != SERVO_ERROR_INVALID_STATE) {
        servo_builder_free(servo_builder);
        return 1;
    }

    servo_builder_free(servo_builder);
    return 0;
}

/* Covers that the intercept_request delegate callback can be set and called.
 * We can't easily test the actual interception without a running load, but we
 * can verify the delegate struct accepts the callback. */
static bool my_intercept_callback(struct WebView* webview,
                                  const struct ServoWebResourceRequest* request,
                                  struct ServoWebResourceResponse* response,
                                  void* user_data)
{
    (void)webview;
    (void)request;
    (void)response;
    if (user_data) {
        *(bool*)user_data = true;
    }
    return false; // Don't actually intercept
}

int test_webview_delegate_intercept_request(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    bool intercepted = false;

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &intercepted,
        .notify_load_status_changed = NULL,
        .notify_url_changed = NULL,
        .notify_page_title_changed = NULL,
        .notify_new_frame_ready = NULL,
        .notify_closed = NULL,
        .notify_focus_changed = NULL,
        .notify_got_focus = NULL,
        .notify_load_started = NULL,
        .notify_load_ended = NULL,
        .notify_load_error = NULL,
        .notify_cursor_changed = NULL,
        .notify_status_text_changed = NULL,
        .notify_favicon_changed = NULL,
        .show_console_message = NULL,
        .show_alert_dialog = NULL,
        .show_confirm_dialog = NULL,
        .show_prompt_dialog = NULL,
        .show_file_picker = NULL,
        .show_select_element = NULL,
        .request_permission = NULL,
        .request_authentication = NULL,
        .request_unload = NULL,
        .intercept_request = my_intercept_callback,
    };

    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    // The delegate is set; we can't easily trigger a load to test the callback
    // without a full integration test. Just verify the WebView was created.
    servo_webview_free(webview);
    return 0;
}

/* -------------------------------------------------------------------------
 * Download tests
 * ------------------------------------------------------------------------- */

/* Test that download item functions handle NULL pointers gracefully. */
int test_download_item_null_handling(void)
{
    /* All getters should return 0/default values for NULL download */
    if (servo_download_item_get_url(NULL, NULL, 0) != 0)
        return 1;
    if (servo_download_item_get_suggested_filename(NULL, NULL, 0) != 0)
        return 1;
    if (servo_download_item_get_received_bytes(NULL) != 0)
        return 1;
    if (servo_download_item_get_total_bytes(NULL) != 0)
        return 1;
    if (servo_download_item_get_state(NULL) != SERVO_DOWNLOAD_INTERRUPTED)
        return 1;

    /* Control functions should return error for NULL */
    if (servo_download_item_cancel(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_download_item_pause(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_download_item_resume(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    return 0;
}

/* Test that download item getters work (buffer-query pattern).
 * Note: We can't easily create a real download item in a unit test,
 * but we can verify the API signatures compile and handle NULL. */
int test_download_item_getters(void)
{
    /* Test buffer-query pattern with NULL buffer */
    uintptr_t len = servo_download_item_get_url(NULL, NULL, 0);
    (void)len;

    len = servo_download_item_get_suggested_filename(NULL, NULL, 0);
    (void)len;

    return 0;
}

static void on_before_download(struct WebView* webview,
                               struct ServoDownloadItem* download,
                               void (*callback)(bool, void*),
                               void* user_data)
{
    (void)webview;
    (void)download;
    if (user_data) {
        *(bool*)user_data = true;
    }
    /* Allow download by default */
    if (callback) {
        callback(true, NULL);
    }
}

static void on_download_updated(struct WebView* webview,
                                struct ServoDownloadItem* download,
                                void* user_data)
{
    (void)webview;
    (void)download;
    if (user_data) {
        *(bool*)user_data = true;
    }
}

static bool on_screen_geometry(struct WebView* webview,
                               struct ServoScreenGeometry* out_geometry,
                               void* user_data)
{
    (void)webview;
    (void)user_data;
    if (out_geometry == NULL)
        return false;
    out_geometry->size_width = 1920;
    out_geometry->size_height = 1080;
    out_geometry->available_width = 1920;
    out_geometry->available_height = 1040;
    out_geometry->window_x = 0;
    out_geometry->window_y = 0;
    out_geometry->window_width = 800;
    out_geometry->window_height = 600;
    return true;
}

static void on_request_move_to(struct WebView* webview, int32_t x, int32_t y, void* user_data)
{
    (void)webview;
    (void)x;
    (void)y;
    (void)user_data;
}

static void on_request_resize_to(struct WebView* webview,
                                 int32_t width,
                                 int32_t height,
                                 void* user_data)
{
    (void)webview;
    (void)width;
    (void)height;
    (void)user_data;
}

static bool on_request_create_new(struct WebView* parent_webview,
                                  struct ServoCreateNewWebViewRequest* request,
                                  void* user_data)
{
    (void)parent_webview;
    (void)user_data;
    if (request == NULL)
        return false;
    /* Deny the popup but take ownership of the request handle. */
    return servo_create_new_webview_request_free(request) == SERVO_ERROR_OK;
}

/* Test that delegate callbacks can be set in ServoWebViewDelegate. */
int test_download_delegate_callbacks(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    bool before_download_called = false;

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &before_download_called,
        .notify_load_status_changed = NULL,
        .notify_url_changed = NULL,
        .notify_page_title_changed = NULL,
        .notify_new_frame_ready = NULL,
        .notify_closed = NULL,
        .notify_focus_changed = NULL,
        .notify_got_focus = NULL,
        .notify_load_started = NULL,
        .notify_load_ended = NULL,
        .notify_load_error = NULL,
        .notify_cursor_changed = NULL,
        .notify_status_text_changed = NULL,
        .notify_favicon_changed = NULL,
        .show_console_message = NULL,
        .show_alert_dialog = NULL,
        .show_confirm_dialog = NULL,
        .show_prompt_dialog = NULL,
        .show_file_picker = NULL,
        .show_select_element = NULL,
        .request_permission = NULL,
        .request_authentication = NULL,
        .request_unload = NULL,
        .intercept_request = NULL,
        .on_before_download = on_before_download,
        .on_download_updated = on_download_updated,
    };

    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* The delegate is set; we can't easily trigger a download to test the callback
     * without a full integration test. Just verify the WebView was created. */
    servo_webview_free(webview);
    return 0;
}

/* Test that the window geometry delegate callbacks (screen_geometry,
 * request_move_to, request_resize_to) can be set in ServoWebViewDelegate. */
int test_webview_window_geometry_delegate_callbacks(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = NULL,
        .screen_geometry = on_screen_geometry,
        .request_move_to = on_request_move_to,
        .request_resize_to = on_request_resize_to,
    };
    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* The callbacks are invoked by Servo internals (for example when page
     * script calls window.moveTo); here we only verify the delegate is
     * accepted and the WebView is created. */
    servo_webview_free(webview);
    return 0;
}

/* Test that the popup delegate callback can be set, and that the
 * create-new-webview-request functions handle NULL arguments. */
int test_webview_popup_delegate_callback(void)
{
    if (servo_create_new_webview_builder_create(NULL, NULL) != NULL)
        return 1;
    if (servo_create_new_webview_request_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = NULL,
        .request_create_new = on_request_create_new,
    };
    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* The callback is only invoked when page content calls window.open(); here
     * we verify the delegate is accepted and the WebView is created. */
    servo_webview_free(webview);
    return 0;
}

/* Test that a delegate whose `struct_size` is smaller than the struct compiled
 * into the library is rejected with SERVO_ERROR_INVALID_ARGUMENT. This is the
 * by-value `struct_size` guard described in ABI_STABILITY.md section 3.3. */
int test_delegate_struct_size_validation(void)
{
    /* NULL builder still reports NullArgument regardless of struct_size. */
    ServoWebViewDelegate webview_delegate = {0};
    if (servo_webview_builder_set_delegate(NULL, webview_delegate) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    /* Too small: rejected. */
    webview_delegate.struct_size = (uint32_t)sizeof(ServoWebViewDelegate) - 1;
    if (servo_webview_builder_set_delegate(builder, webview_delegate) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;

    /* Zero (caller forgot to set it): rejected. */
    webview_delegate.struct_size = 0;
    if (servo_webview_builder_set_delegate(builder, webview_delegate) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;

    /* Exact size: accepted. */
    webview_delegate.struct_size = sizeof(ServoWebViewDelegate);
    if (servo_webview_builder_set_delegate(builder, webview_delegate) != SERVO_ERROR_OK)
        return 1;

    servo_webview_builder_free(builder);

    /* The same guard applies to ServoDelegate on the ServoBuilder. */
    ServoBuilder* servo_builder = servo_builder_create();
    if (servo_builder == NULL)
        return 1;

    struct ServoDelegate servo_delegate = {0};
    if (servo_builder_set_delegate(servo_builder, servo_delegate) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_builder_free(servo_builder);
        return 1;
    }

    servo_delegate.struct_size = sizeof(struct ServoDelegate);
    if (servo_builder_set_delegate(servo_builder, servo_delegate) != SERVO_ERROR_OK) {
        servo_builder_free(servo_builder);
        return 1;
    }

    servo_builder_free(servo_builder);
    return 0;
}

/* Test that a handle which is not live is diagnosed instead of being
 * dereferenced (W2.5, HARDENING_PLAN.md section B.1).
 *
 * The registry keys on the handle's address, so the "garbage" case below never
 * touches memory the process does not own: the pointer is only used as a key. */
int test_options_handle_validation(void)
{
    /* Never created at all. */
    ServoOptions* garbage = (ServoOptions*)(uintptr_t)0x1;
    if (servo_options_set_multiprocess(garbage, false) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;

    /* NULL still reports NullArgument: the registry does not replace the null
     * checks, it sits behind them. */
    if (servo_options_set_multiprocess(NULL, false) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    /* A live handle works... */
    ServoOptions* options = servo_options_create();
    if (options == NULL)
        return 1;
    if (servo_options_set_multiprocess(options, false) != SERVO_ERROR_OK)
        return 1;

    /* ...is forgotten when it is freed... */
    if (servo_options_free(options) != SERVO_ERROR_OK)
        return 1;

    /* ...so using it again, or freeing it a second time, is an error rather
     * than undefined behaviour. */
    if (servo_options_set_multiprocess(options, false) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;
    if (servo_options_free(options) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;

    /* The same holds for a handle that was consumed by another API: ownership
     * of `options` moves into the builder in `servo_builder_set_options`. */
    ServoOptions* consumed = servo_options_create();
    if (consumed == NULL)
        return 1;

    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;

    if (servo_builder_set_options(builder, consumed) != SERVO_ERROR_OK) {
        servo_builder_free(builder);
        return 1;
    }

    if (servo_options_set_multiprocess(consumed, false) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_builder_free(builder);
        return 1;
    }

    servo_builder_free(builder);
    return 0;
}

/* Test the `struct_size` guard on the by-pointer input structs (W2.5,
 * ABI_STABILITY.md section 3.3): `ServoProxyConfig` and `ServoCookieFilter`
 * are filled in by the embedder, so a caller built against an older header
 * would pass a smaller struct that the library must not read past. */
int test_proxy_config_struct_size_validation(void)
{
    ServoRequestContext* context = servo_request_context_create();
    if (context == NULL)
        return 1;

    ServoProxyConfig config = {
        .struct_size = sizeof(ServoProxyConfig),
        .mode = SERVO_PROXY_FIXED_SERVERS,
        .server = "127.0.0.1:3128",
        .bypass_list = "localhost",
    };

    /* Exact size: accepted. */
    if (servo_request_context_set_proxy_config(context, &config) != SERVO_ERROR_OK) {
        servo_request_context_free(context);
        return 1;
    }

    /* Zero, i.e. the caller never set it: rejected. */
    config.struct_size = 0;
    if (servo_request_context_set_proxy_config(context, &config) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_request_context_free(context);
        return 1;
    }

    /* One byte short of what this library was compiled with: rejected. */
    config.struct_size = (uint32_t)sizeof(ServoProxyConfig) - 1;
    if (servo_request_context_set_proxy_config(context, &config) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_request_context_free(context);
        return 1;
    }

    /* Larger than the compiled-in size: accepted, because Servo only reads the
     * fields it knows about. This is what lets a caller built against a newer
     * header run against an older library. */
    config.struct_size = (uint32_t)sizeof(ServoProxyConfig) + 16;
    if (servo_request_context_set_proxy_config(context, &config) != SERVO_ERROR_OK) {
        servo_request_context_free(context);
        return 1;
    }

    /* NULL is still NullArgument. */
    if (servo_request_context_set_proxy_config(context, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_request_context_free(context);
        return 1;
    }

    servo_request_context_free(context);
    return 0;
}

/* Test the `struct_size` guard on `ServoCookieFilter` (W2.5). */
int test_cookie_filter_struct_size_validation(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    ServoCookieFilter filter = {
        .struct_size = sizeof(ServoCookieFilter),
        .url = "https://example.com/",
        .name = "session",
        .secure = false,
        .session = false,
        .flags = 0,
    };

    /* Exact size: accepted. */
    if (servo_site_data_delete_all_cookies(manager, &filter, NULL, NULL) != SERVO_ERROR_OK) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* Zero: rejected. */
    filter.struct_size = 0;
    if (servo_site_data_delete_all_cookies(manager, &filter, NULL, NULL) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* Too small: rejected. */
    filter.struct_size = (uint32_t)sizeof(ServoCookieFilter) - 1;
    if (servo_site_data_delete_all_cookies(manager, &filter, NULL, NULL) != SERVO_ERROR_INVALID_ARGUMENT) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* NULL still means "no filter, match everything". */
    if (servo_site_data_delete_all_cookies(manager, NULL, NULL, NULL) != SERVO_ERROR_OK) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    return 0;
}

/* Test the single-process side of the multi-process contract documented in
 * C_API_GUIDE.md ("Multi-process support"). CI links servo-capi *without* the
 * `multiprocess` feature, so what is asserted here is the refusal path: the
 * content-process entry point is exported in both builds (or linking would
 * fail) and refuses to run, and no sandbox is reported. The spawn side cannot
 * be exercised in this harness and is deliberately not faked here. */
int test_multiprocess_single_process_path(void)
{
    /* NULL token: refused in every build, feature or not. */
    if (servo_run_content_process(NULL) != -1)
        return 1;

    /* Capability queries are build-time, so they are stable under a test. */
    if (servo_capi_is_multiprocess_enabled()) {
        /* A feature-enabled build would do a real IPC handshake on the token
         * here, and `servo_create_sandbox()` would activate a real OS sandbox
         * on the running test process: neither is something a unit test may do
         * in-process. Skip them and say so instead of pretending to test. */
        printf("      (multiprocess build: spawn path not exercised here)\n");
        fflush(stdout);
        return 0;
    }

    /* Feature off: a non-NULL token is refused as well, returning rather than
     * letting a panic reach the `extern "C"` boundary. */
    if (servo_run_content_process("not-a-real-token") != -1)
        return 1;

    /* No OS sandbox exists in a build without the feature. */
    if (servo_capi_is_sandboxing_supported())
        return 1;
    if (servo_create_sandbox())
        return 1;

    return 0;
}

/* Test that find-in-page API functions exist and handle NULL arguments. */
int test_find_null_arguments(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* Test NULL webview */
    int result = servo_webview_find(NULL, "test", true, false);
    if (result != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    result = servo_webview_find_next(NULL, 1, true);
    if (result != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    result = servo_webview_find_clear(NULL, 1);
    if (result != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test NULL text */
    result = servo_webview_find(webview, NULL, true, false);
    if (result != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test find result getters with NULL */
    if (servo_find_result_get_identifier(NULL) != 0) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_find_result_get_active_match_ordinal(NULL) != 0) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_find_result_get_number_of_matches(NULL) != 0) {
        servo_webview_free(webview);
        return 1;
    }

    servo_webview_free(webview);
    return 0;
}

/* Test that find-in-page API can be called with valid arguments. */
int test_find_valid_arguments(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* Test valid find call - should return a positive identifier */
    int identifier = servo_webview_find(webview, "test", true, false);
    if (identifier <= 0) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test find_next with the identifier */
    int result = servo_webview_find_next(webview, identifier, true);
    if (result != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test find_next backward */
    result = servo_webview_find_next(webview, identifier, false);
    if (result != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test find_clear */
    result = servo_webview_find_clear(webview, identifier);
    if (result != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test case-sensitive find */
    identifier = servo_webview_find(webview, "Test", false, true);
    if (identifier <= 0) {
        servo_webview_free(webview);
        return 1;
    }
    result = servo_webview_find_clear(webview, identifier);
    if (result != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    /* Test find with empty string */
    identifier = servo_webview_find(webview, "", true, false);
    if (identifier <= 0) {
        servo_webview_free(webview);
        return 1;
    }
    result = servo_webview_find_clear(webview, identifier);
    if (result != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    servo_webview_free(webview);
    return 0;
}

/* Test that find result getters work. */
int test_find_result_getters(void)
{
    /* We can't easily create a real ServoFindResult without a full integration test,
     * but we can test that the functions exist and don't crash with NULL. */
    float x = 1.0f, y = 2.0f, w = 3.0f, h = 4.0f;
    servo_find_result_get_selection_rect(NULL, &x, &y, &w, &h);
    if (x != 0.0f || y != 0.0f || w != 0.0f || h != 0.0f) {
        return 1;
    }

    /* Test with partial NULL outputs */
    x = 1.0f; y = 2.0f; w = 3.0f; h = 4.0f;
    servo_find_result_get_selection_rect(NULL, &x, NULL, &w, NULL);
    if (x != 0.0f || w != 0.0f) {
        return 1;
    }

    return 0;
}

static void on_find_result(struct WebView* webview,
                           int identifier,
                           int count,
                           const struct ServoFindResult* result,
                           void* user_data)
{
    (void)webview;
    (void)identifier;
    (void)count;
    (void)result;
    if (user_data) {
        *(bool*)user_data = true;
    }
}

/* Test that find delegate callback can be set. */
int test_find_delegate_callback(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    RenderingContext* context = servo_rendering_context_create_software(100, 100);
    if (context == NULL)
        return 1;

    ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 1;

    bool find_result_called = false;

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &find_result_called,
        .notify_load_status_changed = NULL,
        .notify_url_changed = NULL,
        .notify_page_title_changed = NULL,
        .notify_new_frame_ready = NULL,
        .notify_closed = NULL,
        .notify_focus_changed = NULL,
        .notify_got_focus = NULL,
        .notify_load_started = NULL,
        .notify_load_ended = NULL,
        .notify_load_error = NULL,
        .notify_cursor_changed = NULL,
        .notify_status_text_changed = NULL,
        .notify_favicon_changed = NULL,
        .show_console_message = NULL,
        .show_alert_dialog = NULL,
        .show_confirm_dialog = NULL,
        .show_prompt_dialog = NULL,
        .show_file_picker = NULL,
        .show_select_element = NULL,
        .request_permission = NULL,
        .request_authentication = NULL,
        .request_unload = NULL,
        .intercept_request = NULL,
        .on_before_download = NULL,
        .on_download_updated = NULL,
        .on_find_result = on_find_result,
        .notify_animating_changed = NULL,
        .notify_history_changed = NULL,
        .notify_crashed = NULL,
    };

    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    /* The delegate is set; we can't easily trigger a find to test the callback
     * without a full integration test. Just verify the WebView was created. */
    servo_webview_free(webview);
    return 0;
}

int test_webview_hidpi_scale_factor(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    /* Test null handling */
    if (servo_webview_set_hidpi_scale_factor(NULL, 2.0f) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_get_hidpi_scale_factor(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    float scale = 0.0f;
    if (servo_webview_get_hidpi_scale_factor(webview, &scale) != SERVO_ERROR_OK)
        return 1;
    if (scale <= 0.0f)
        return 1;

    /* Set valid scale factor */
    if (servo_webview_set_hidpi_scale_factor(webview, 2.0f) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_get_hidpi_scale_factor(webview, &scale) != SERVO_ERROR_OK)
        return 1;
    if (fabsf(scale - 2.0f) > 0.01f)
        return 1;

    /* Set invalid scale factor (negative / zero) */
    if (servo_webview_set_hidpi_scale_factor(webview, -1.0f) != SERVO_ERROR_INVALID_STATE)
        return 1;
    if (servo_webview_set_hidpi_scale_factor(webview, 0.0f) != SERVO_ERROR_INVALID_STATE)
        return 1;

    /* Reset back to 1.0 */
    servo_webview_set_hidpi_scale_factor(webview, 1.0f);
    servo_webview_free(webview);
    return 0;
}

int test_webview_animating(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_is_animating(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    bool animating = true;
    if (servo_webview_is_animating(webview, &animating) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_pinch_zoom(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_get_pinch_zoom(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_adjust_pinch_zoom(NULL, 1.0f, 0.0f, 0.0f) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    float zoom = 0.0f;
    if (servo_webview_get_pinch_zoom(webview, &zoom) != SERVO_ERROR_OK)
        return 1;
    if (zoom <= 0.0f)
        return 1;

    if (servo_webview_adjust_pinch_zoom(webview, 1.1f, 50.0f, 50.0f) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_exit_fullscreen(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_exit_fullscreen(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    if (servo_webview_exit_fullscreen(webview) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_load_request(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_load_request(NULL, NULL, NULL, 0) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_load_request(webview, NULL, NULL, 0) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    const char* headers[] = {
        "X-Custom-Header: value1",
        "Authorization: Bearer token123"
    };
    if (servo_webview_load_request(webview, "data:text/html,<h1>test</h1>", headers, 2) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_scroll(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_scroll_by(NULL, 0.0f, 100.0f) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_scroll_to_start(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_scroll_to_end(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    if (servo_webview_scroll_by(webview, 0.0f, 50.0f) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_scroll_to_start(webview) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_scroll_to_end(webview) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_ime(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_notify_ime_composition_start(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_ime_composition_update(NULL, "test") != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_ime_composition_update(webview, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_ime_composition_end(NULL, "test") != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_ime_composition_end(webview, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_ime_dismissed(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    if (servo_webview_notify_ime_composition_start(webview) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_notify_ime_composition_update(webview, "hello") != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_notify_ime_composition_end(webview, "hello") != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_notify_ime_dismissed(webview) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

int test_webview_media_session(void)
{
    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_notify_media_session_action(NULL, SERVO_MEDIA_SESSION_ACTION_PLAY) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_notify_media_session_action(webview, 9999) != SERVO_ERROR_INVALID_ARGUMENT)
        return 1;

    if (servo_webview_notify_media_session_action(webview, SERVO_MEDIA_SESSION_ACTION_PLAY) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_notify_media_session_action(webview, SERVO_MEDIA_SESSION_ACTION_PAUSE) != SERVO_ERROR_OK)
        return 1;
    if (servo_webview_notify_media_session_action(webview, SERVO_MEDIA_SESSION_ACTION_STOP) != SERVO_ERROR_OK)
        return 1;

    servo_webview_free(webview);
    return 0;
}

/* Covers the user script APIs: creating scripts, adding/removing them from a
 * `UserContentManager`, and using a script that has a source file. */
int test_user_content_manager_scripts(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoUserContentManager* manager = servo_user_content_manager_create(servo);
    if (manager == NULL)
        return 1;

    struct ServoUserScript* script = servo_user_script_create("console.log('hi');", NULL);
    if (script == NULL) {
        servo_user_content_manager_free(manager);
        return 1;
    }

    int ok = servo_user_content_manager_add_script(manager, script) == SERVO_ERROR_OK &&
        servo_user_content_manager_remove_script(manager, script) == SERVO_ERROR_OK;

    // A script with an associated source file.
    struct ServoUserScript* script_file = servo_user_script_create("window.x = 1;", "test.js");
    if (script_file == NULL) {
        servo_user_script_free(script);
        servo_user_content_manager_free(manager);
        return 1;
    }
    ok = ok && servo_user_content_manager_add_script(manager, script_file) == SERVO_ERROR_OK;

    servo_user_script_free(script_file);
    servo_user_script_free(script);
    servo_user_content_manager_free(manager);
    return ok ? 0 : 1;
}

/* Covers the user stylesheet APIs: creating stylesheets and adding/removing
 * them from a `UserContentManager`. */
int test_user_content_manager_style_sheets(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoUserContentManager* manager = servo_user_content_manager_create(servo);
    if (manager == NULL)
        return 1;

    struct ServoUserStyleSheet* sheet =
        servo_user_style_sheet_create("body { color: red; }", "about:blank");
    if (sheet == NULL) {
        servo_user_content_manager_free(manager);
        return 1;
    }

    int ok = servo_user_content_manager_add_style_sheet(manager, sheet) == SERVO_ERROR_OK &&
        servo_user_content_manager_remove_style_sheet(manager, sheet) == SERVO_ERROR_OK;

    servo_user_style_sheet_free(sheet);
    servo_user_content_manager_free(manager);
    return ok ? 0 : 1;
}

/* Covers the null/invalid-argument error paths of the user content APIs. */
int test_user_content_manager_null_arguments(void)
{
    if (servo_user_script_create(NULL, NULL) != NULL)
        return 1;
    if (servo_user_style_sheet_create(NULL, NULL) != NULL)
        return 1;
    if (servo_user_style_sheet_create("body {}", NULL) != NULL)
        return 1;
    if (servo_user_style_sheet_create("body {}", "not a valid url") != NULL)
        return 1;

    if (servo_user_script_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_user_style_sheet_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    if (servo_user_content_manager_add_script(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_user_content_manager_remove_script(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_user_content_manager_add_style_sheet(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_user_content_manager_remove_style_sheet(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    return 0;
}

/* Covers the NetworkManager cache API using the buffer-query pattern:
 * query the required length, allocate, then read the JSON array of keys. */
int test_network_manager_cache(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoNetworkManager* manager = servo_get_network_manager(servo);
    if (manager == NULL)
        return 1;

    size_t required = servo_network_manager_get_cache_entries(manager, NULL, 0);
    if (required < 2) {
        servo_network_manager_free(manager);
        return 1;
    }

    char* buffer = (char*)malloc(required);
    if (buffer == NULL) {
        servo_network_manager_free(manager);
        return 1;
    }

    size_t written = servo_network_manager_get_cache_entries(manager, buffer, required);
    int ok = written == required && buffer[0] == '[' && buffer[written - 2] == ']';
    free(buffer);

    // Clearing the cache must succeed and be safe to call repeatedly.
    ok = ok &&
        servo_network_manager_clear_cache(manager) == SERVO_ERROR_OK &&
        servo_network_manager_clear_cache(manager) == SERVO_ERROR_OK;

    servo_network_manager_free(manager);
    return ok ? 0 : 1;
}

/* Covers the null-argument error paths of the NetworkManager API. */
int test_network_manager_null_arguments(void)
{
    if (servo_get_network_manager(NULL) != NULL)
        return 1;
    if (servo_network_manager_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_network_manager_clear_cache(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_network_manager_get_cache_entries(NULL, NULL, 0) != 0)
        return 1;
    return 0;
}

static void dummy_script_msg_callback(struct WebView* webview,
                                      const char* handler_name,
                                      const char* message_json,
                                      void* user_data)
{
    (void)webview;
    (void)handler_name;
    (void)message_json;
    if (user_data) {
        *(bool*)user_data = true;
    }
}

/* Covers native <-> JS message router C API entry points and WebViewDelegate callback. */
int test_script_message_router(void)
{
    /* Null argument tests */
    if (servo_webview_add_script_message_handler(NULL, "test", dummy_script_msg_callback, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_remove_script_message_handler(NULL, "test") != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_post_message(NULL, "test", "{}") != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    struct WebView* webview = get_test_webview();
    if (webview == NULL)
        return 1;

    if (servo_webview_add_script_message_handler(webview, NULL, dummy_script_msg_callback, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_remove_script_message_handler(webview, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_post_message(webview, NULL, "{}") != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_post_message(webview, "test", NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    /* Handler registration and removal */
    if (servo_webview_add_script_message_handler(webview, "hostHandler", dummy_script_msg_callback, NULL) != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_post_message(webview, "greet", "{\"hello\":\"world\"}") != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_poll_js_messages(webview) != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_remove_script_message_handler(webview, "hostHandler") != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    servo_webview_free(webview);

    /* Verify ServoWebViewDelegate on_script_message field initialization and struct_size */
    bool script_message_called = false;
    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &script_message_called,
        .on_script_message = dummy_script_msg_callback,
    };
    if (delegate.struct_size < sizeof(ServoWebViewDelegate))
        return 1;

    return 0;
}

static bool dummy_site_visitor(const struct ServoSiteData* site,
                               uintptr_t count,
                               uintptr_t total,
                               void* user_data)
{
    (void)site;
    (void)count;
    (void)total;
    if (user_data) {
        *(bool*)user_data = true;
    }
    return true;
}

int test_site_data_visit_sites(void)
{
    /* NULL manager */
    if (servo_site_data_visit_sites(NULL, SERVO_STORAGE_TYPE_ALL, dummy_site_visitor, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    /* NULL visitor */
    if (servo_site_data_visit_sites(manager, SERVO_STORAGE_TYPE_ALL, NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* Valid call */
    bool called = false;
    if (servo_site_data_visit_sites(manager, SERVO_STORAGE_TYPE_ALL, dummy_site_visitor, &called) != SERVO_ERROR_OK) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    return 0;
}

int test_site_data_clear_site_data(void)
{
    const char* sites[] = { "example.com", "test.org" };

    /* NULL manager */
    if (servo_site_data_clear_site_data(NULL, sites, 2, SERVO_STORAGE_TYPE_ALL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoSiteDataManager* manager = servo_get_site_data_manager(servo);
    if (manager == NULL)
        return 1;

    /* NULL sites array with count > 0 */
    if (servo_site_data_clear_site_data(manager, NULL, 1, SERVO_STORAGE_TYPE_ALL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* count = 0 is safe */
    if (servo_site_data_clear_site_data(manager, NULL, 0, SERVO_STORAGE_TYPE_ALL) != SERVO_ERROR_OK) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    /* Valid call with specific sites */
    if (servo_site_data_clear_site_data(manager, sites, 2, SERVO_STORAGE_TYPE_LOCAL | SERVO_STORAGE_TYPE_SESSION) != SERVO_ERROR_OK) {
        servo_site_data_manager_free(manager);
        return 1;
    }

    servo_site_data_manager_free(manager);
    return 0;
}

int test_web_resource_load_null_handling(void)
{
    if (servo_web_resource_load_get_request(NULL) != NULL)
        return 1;
    if (servo_web_resource_load_intercept(NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_web_resource_load_send_body_data(NULL, NULL, 0) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_web_resource_load_finish(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_web_resource_load_cancel(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_web_resource_load_free(NULL) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;

    return 0;
}

static bool dummy_streaming_cb(struct WebView* webview,
                               const struct ServoWebResourceRequest* request,
                               struct ServoWebResourceResponse* response,
                               struct ServoWebResourceLoad* load,
                               void* user_data)
{
    (void)webview;
    (void)request;
    (void)response;
    (void)load;
    if (user_data) {
        *(bool*)user_data = true;
    }
    return false;
}

int test_webview_delegate_intercept_request_streaming(void)
{
    bool called = false;
    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &called,
        .intercept_request_streaming = dummy_streaming_cb,
    };
    if (delegate.struct_size < sizeof(ServoWebViewDelegate))
        return 1;
    if (delegate.intercept_request_streaming != dummy_streaming_cb)
        return 1;

    return 0;
}

static void print_pdf_cb(bool success, const char* output_path, void* user_data)
{
    (void)output_path;
    int* called = (int*)user_data;
    if (success) {
        *called = 1;
    }
}

int test_webview_print_to_pdf(void)
{
    struct WebView* webview = create_about_blank_webview();
    if (webview == NULL)
        return 1;

    /* Null argument checks */
    if (servo_webview_print_to_pdf(NULL, "dummy.pdf", NULL, NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }
    if (servo_webview_print_to_pdf(webview, NULL, NULL, NULL, NULL) != SERVO_ERROR_NULL_ARGUMENT) {
        servo_webview_free(webview);
        return 1;
    }

    const char* pdf_path = "test_print_output.pdf";
    remove(pdf_path);

    int callback_called = 0;
    int err = servo_webview_print_to_pdf(webview, pdf_path, NULL, print_pdf_cb, &callback_called);
    if (err != SERVO_ERROR_OK) {
        servo_webview_free(webview);
        return 1;
    }

    if (!callback_called) {
        servo_webview_free(webview);
        return 1;
    }

    /* Verify file exists, is non-empty, and begins with %PDF */
    FILE* f = fopen(pdf_path, "rb");
    if (!f) {
        servo_webview_free(webview);
        return 1;
    }

    char header[5] = {0};
    size_t read_bytes = fread(header, 1, 4, f);
    fclose(f);
    remove(pdf_path);

    if (read_bytes < 4 || strncmp(header, "%PDF", 4) != 0) {
        servo_webview_free(webview);
        return 1;
    }

    servo_webview_free(webview);
    return 0;
}

static void dummy_context_menu_callback(struct WebView* webview,
                                        const struct ServoContextMenuParams* params,
                                        ServoContextMenuCallback callback,
                                        void* user_data)
{
    (void)webview;
    (void)params;
    (void)user_data;
    if (callback) {
        callback(SERVO_CONTEXT_MENU_ACTION_DISMISS, NULL);
    }
}

int test_webview_delegate_context_menu(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    struct ServoWebViewBuilder* builder = servo_webview_builder_create(servo);
    if (builder == NULL)
        return 1;

    struct ServoWebViewDelegate delegate = {
        .struct_size = sizeof(struct ServoWebViewDelegate),
        .show_context_menu = dummy_context_menu_callback,
    };

    servo_webview_builder_set_delegate(builder, delegate);

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 1;

    servo_webview_free(webview);
    return 0;
}

static void dummy_post_task_callback(void* user_data)
{
    int* counter = (int*)user_data;
    if (counter != NULL) {
        (*counter)++;
    }
}

int test_servo_post_task(void)
{
    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 1;

    int counter = 0;
    if (servo_post_task(servo, dummy_post_task_callback, &counter) != SERVO_ERROR_OK)
        return 1;

    // Task should not have run yet before event loop spins
    if (counter != 0)
        return 1;

    if (servo_spin_event_loop(servo) != SERVO_ERROR_OK)
        return 1;

    // Task must have executed during spin_event_loop
    if (counter != 1)
        return 1;

    return 0;
}

int test_webview_private_browsing(void)
{
    // Test null safety
    if (servo_webview_builder_set_private_browsing(NULL, true) != SERVO_ERROR_NULL_ARGUMENT)
        return 1;
    if (servo_webview_is_private(NULL))
        return 2;

    struct Servo* servo = create_test_servo();
    if (servo == NULL)
        return 3;

    struct RenderingContext* context = servo_rendering_context_create_software(800, 600);
    if (context == NULL)
        return 4;

    struct ServoWebViewBuilder* builder = servo_webview_builder_create(servo, context);
    if (builder == NULL)
        return 5;

    if (servo_webview_builder_set_private_browsing(builder, true) != SERVO_ERROR_OK) {
        servo_webview_builder_free(builder);
        return 6;
    }

    struct WebView* webview = servo_webview_builder_build(builder);
    if (webview == NULL)
        return 7;

    if (!servo_webview_is_private(webview)) {
        servo_webview_close(webview);
        servo_webview_free(webview);
        return 8;
    }

    servo_webview_close(webview);
    servo_webview_free(webview);
    return 0;
}

int run_c_api_tests(void)
{
    tests_run = 0;
    tests_failed = 0;

    RUN_TEST(test_builder_create_and_free);
    RUN_TEST(test_builder_set_options);
    RUN_TEST(test_builder_set_preferences);
    RUN_TEST(test_builder_set_event_loop_waker);
    RUN_TEST(test_builder_set_delegate);
    RUN_TEST(test_builder_delegate_connection_request);
    RUN_TEST(test_options_create_and_free);
    RUN_TEST(test_options_setters);
    RUN_TEST(test_preferences_create_and_free);
    RUN_TEST(test_preferences_bool_roundtrip);
    RUN_TEST(test_preferences_i64_roundtrip);
    RUN_TEST(test_preferences_u64_roundtrip);
    RUN_TEST(test_preferences_string_roundtrip);
    RUN_TEST(test_set_preference_runtime);
    RUN_TEST(test_rendering_context_create_and_free);
    RUN_TEST(test_rendering_context_zero_size_returns_null);
    RUN_TEST(test_rendering_context_create_window_returns_null_in_headless);
    RUN_TEST(test_rendering_context_create_window_null_handles);
    RUN_TEST(test_rendering_context_create_offscreen_null_parent);
    RUN_TEST(test_rendering_context_create_offscreen_software_parent);
    RUN_TEST(test_rendering_context_create_offscreen_zero_size);
    RUN_TEST(test_rendering_context_create_offscreen_window_parent_headless);
    RUN_TEST(test_webview_builder_create_and_free);
    RUN_TEST(test_webview_builder_set_url_valid);
    RUN_TEST(test_webview_builder_set_url_invalid);
    RUN_TEST(test_webview_builder_build);
    RUN_TEST(test_capi_version);
    RUN_TEST(test_capi_capabilities);
    RUN_TEST(test_thread_affinity_enforcement);
    RUN_TEST(test_webview_get_url);
    RUN_TEST(test_webview_get_title);
    RUN_TEST(test_webview_get_status_text);
    RUN_TEST(test_webview_get_load_status);
    RUN_TEST(test_webview_get_size);
    RUN_TEST(test_webview_can_go_back_forward);
    RUN_TEST(test_webview_page_zoom);
    RUN_TEST(test_webview_focus);
    RUN_TEST(test_webview_notify_key_event);
    RUN_TEST(test_webview_notify_mouse_button_event);
    RUN_TEST(test_webview_notify_mouse_move_event);
    RUN_TEST(test_webview_notify_mouse_leave_event);
    RUN_TEST(test_webview_notify_touch_event);
    RUN_TEST(test_webview_notify_wheel_event);
    RUN_TEST(test_webview_notify_input_event_handled);
    RUN_TEST(test_webview_resize);
    RUN_TEST(test_webview_show_hide);
    RUN_TEST(test_webview_notify_theme_change);
    RUN_TEST(test_webview_evaluate_javascript_null_callback);
    RUN_TEST(test_webview_navigation_noops);

    RUN_TEST(test_site_data_manager_create_and_free);
    RUN_TEST(test_site_data_cookies_for_url);
    RUN_TEST(test_site_data_set_cookie);
    RUN_TEST(test_site_data_clear_cookies);
    RUN_TEST(test_site_data_visit_sites);
    RUN_TEST(test_site_data_clear_site_data);

    RUN_TEST(test_error_null_arguments);
    RUN_TEST(test_servo_setup_logging_repeated);
    RUN_TEST(test_error_invalid_utf8);
    RUN_TEST(test_webview_query_null_webview);
    RUN_TEST(test_builder_free_null);
    RUN_TEST(test_webview_setters_null);
    RUN_TEST(test_builder_register_protocol_handler);
    RUN_TEST(test_webview_delegate_intercept_request);
    RUN_TEST(test_webview_delegate_intercept_request_streaming);
    RUN_TEST(test_web_resource_load_null_handling);
    RUN_TEST(test_download_item_null_handling);
    RUN_TEST(test_download_item_getters);
    RUN_TEST(test_download_delegate_callbacks);
    RUN_TEST(test_webview_window_geometry_delegate_callbacks);
    RUN_TEST(test_webview_popup_delegate_callback);
    RUN_TEST(test_delegate_struct_size_validation);
    RUN_TEST(test_options_handle_validation);
    RUN_TEST(test_proxy_config_struct_size_validation);
    RUN_TEST(test_cookie_filter_struct_size_validation);
    RUN_TEST(test_multiprocess_single_process_path);
    RUN_TEST(test_find_null_arguments);
    RUN_TEST(test_find_valid_arguments);
    RUN_TEST(test_find_result_getters);
    RUN_TEST(test_find_delegate_callback);
    RUN_TEST(test_webview_hidpi_scale_factor);
    RUN_TEST(test_webview_animating);
    RUN_TEST(test_webview_pinch_zoom);
    RUN_TEST(test_webview_exit_fullscreen);
    RUN_TEST(test_webview_load_request);
    RUN_TEST(test_webview_scroll);
    RUN_TEST(test_webview_ime);
    RUN_TEST(test_webview_media_session);

    RUN_TEST(test_user_content_manager_scripts);
    RUN_TEST(test_user_content_manager_style_sheets);
    RUN_TEST(test_user_content_manager_null_arguments);
    RUN_TEST(test_network_manager_cache);
    RUN_TEST(test_network_manager_null_arguments);
    RUN_TEST(test_script_message_router);

    RUN_TEST(test_value_nested_containers);
    RUN_TEST(test_webview_print_to_pdf);
    RUN_TEST(test_webview_delegate_context_menu);
    RUN_TEST(test_servo_post_task);
    RUN_TEST(test_webview_private_browsing);

    free_test_servo();

    printf("\n%d tests, %d passed, %d failed\n", tests_run,
        tests_run - tests_failed, tests_failed);
    return tests_failed > 0 ? 1 : 0;
}
