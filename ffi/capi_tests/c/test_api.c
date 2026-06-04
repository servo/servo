/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "servo/servo_capi.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// The test runner expects us to return 0 for success and
// non-zero for failure.

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

    struct ServoEventLoopWaker waker = { noop_wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_free(builder);
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

    struct ServoEventLoopWaker waker = { noop_wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);

    ServoOptions* options = servo_options_create();
    if (options != NULL) {
        servo_options_set_multiprocess(options, false);
        servo_builder_set_options(builder, options);
    }

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
    return (result == -1) ? 0 : 1;
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

int run_c_api_tests(void)
{
    tests_run = 0;
    tests_failed = 0;

    RUN_TEST(test_builder_create_and_free);
    RUN_TEST(test_builder_set_options);
    RUN_TEST(test_builder_set_preferences);
    RUN_TEST(test_builder_set_event_loop_waker);
    RUN_TEST(test_options_create_and_free);
    RUN_TEST(test_options_setters);
    RUN_TEST(test_preferences_create_and_free);
    RUN_TEST(test_preferences_bool_roundtrip);
    RUN_TEST(test_preferences_i64_roundtrip);
    RUN_TEST(test_preferences_u64_roundtrip);
    RUN_TEST(test_preferences_string_roundtrip);
    RUN_TEST(test_rendering_context_create_and_free);
    RUN_TEST(test_rendering_context_zero_size_returns_null);
    RUN_TEST(test_webview_builder_create_and_free);
    RUN_TEST(test_webview_builder_set_url_valid);
    RUN_TEST(test_webview_builder_set_url_invalid);
    RUN_TEST(test_webview_builder_build);

    free_test_servo();

    printf("\n%d tests, %d passed, %d failed\n", tests_run,
        tests_run - tests_failed, tests_failed);
    return tests_failed > 0 ? 1 : 0;
}
