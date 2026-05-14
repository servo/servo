/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "servo/servo_capi.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// The test runner expects us to return 0 for success and
// non-zero for failure.

static int tests_run;
static int tests_failed;

#define RUN_TEST(name) do {                               \
    tests_run++;                                          \
    printf("  RUN  %s\n", #name);                         \
    fflush(stdout);                                       \
    int ret = name();                                     \
    if (ret != 0) {                                       \
        printf("  FAIL %s (returned %d)\n", #name, ret);  \
        tests_failed++;                                   \
    }                                                     \
    fflush(stdout);                                       \
} while(0)

int test_builder_create_and_free(void) {
    ServoBuilder *builder = servo_builder_create();
    if (builder == NULL) return 1;
    servo_builder_free(builder);
    return 0;
}

int test_builder_set_options(void) {
    ServoBuilder *builder = servo_builder_create();
    ServoOptions *options = servo_options_create();
    if (builder == NULL || options == NULL) return 1;
    servo_builder_set_options(builder, options);
    // builder now owns options, so don't free it
    servo_builder_free(builder);
    return 0;
}

int test_builder_set_preferences(void) {
    ServoBuilder *builder = servo_builder_create();
    ServoPreferences *prefs = servo_preferences_create();
    if (builder == NULL || prefs == NULL) return 1;
    servo_builder_set_preferences(builder, prefs);
    // builder now owns prefs, so don't free it
    servo_builder_free(builder);
    return 0;
}

static void noop_wake_callback(void) {}

int test_builder_set_event_loop_waker(void) {
    ServoBuilder *builder = servo_builder_create();
    if (builder == NULL) return 1;

    struct ServoEventLoopWaker waker = { noop_wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_free(builder);
    return 0;
}

int test_options_create_and_free(void) {
    ServoOptions *options = servo_options_create();
    if (options == NULL) return 1;
    servo_options_free(options);
    return 0;
}

int test_options_setters(void) {
    ServoOptions *options = servo_options_create();
    if (options == NULL) return 1;
    servo_options_set_hard_fail(options, true);
    servo_options_set_multiprocess(options, false);
    servo_options_set_force_ipc(options, true);
    servo_options_set_background_hang_monitor(options, false);
    servo_options_set_sandbox(options, true);
    servo_options_set_temporary_storage(options, false);
    servo_options_set_ignore_certificate_errors(options, true);
    servo_options_set_unminify_js(options, false);
    servo_options_set_unminify_css(options, true);
    servo_options_set_debug_option(options, SERVO_DIAGNOSTICS_LOGGING_OPTION_STYLE_TREE, true);
    servo_options_enable_time_profiling_to_stdout(options, 0.5);
    servo_options_set_random_pipeline_closure_probability(options, 0.25f);
    servo_options_free(options);
    return 0;
}

int test_preferences_create_and_free(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;
    servo_preferences_free(prefs);
    return 0;
}

int test_preferences_bool_roundtrip(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;

    servo_preferences_set_bool(prefs, "dom_gamepad_enabled", false);
    if (servo_preferences_get_bool(prefs, "dom_gamepad_enabled") != false) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_set_bool(prefs, "dom_gamepad_enabled", true);
    if (servo_preferences_get_bool(prefs, "dom_gamepad_enabled") != true) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_free(prefs);
    return 0;
}

int test_preferences_i64_roundtrip(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;

    servo_preferences_set_i64(prefs, "layout_threads", 4);
    if (servo_preferences_get_i64(prefs, "layout_threads") != 4) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_set_i64(prefs, "layout_threads", -1);
    if (servo_preferences_get_i64(prefs, "layout_threads") != -1) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_free(prefs);
    return 0;
}

int test_preferences_u64_roundtrip(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;

    servo_preferences_set_u64(prefs, "network_http_cache_size", 12345);
    if (servo_preferences_get_u64(prefs, "network_http_cache_size") != 12345) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_free(prefs);
    return 0;
}

int test_preferences_string_roundtrip(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;

    servo_preferences_set_string(prefs, "user_agent", "ServoTest/1.0");

    char *user_agent = servo_preferences_get_string(prefs, "user_agent");
    if (user_agent == NULL) {
        servo_preferences_free(prefs);
        return 1;
    }
    if (strcmp(user_agent, "ServoTest/1.0") != 0) {
        servo_string_free(user_agent);
        servo_preferences_free(prefs);
        return 1;
    }
    servo_string_free(user_agent);
    servo_preferences_free(prefs);
    return 0;
}

int test_preferences_f64_array4_roundtrip(void) {
    ServoPreferences *prefs = servo_preferences_create();
    if (prefs == NULL) return 1;

    double rgba[4] = {0.2, 0.4, 0.6, 1.0};
    servo_preferences_set_f64_array_4(prefs, "shell_background_color_rgba", rgba);
    double out[4];
    servo_preferences_get_f64_array_4(prefs, "shell_background_color_rgba", out);
    if (out[0] != 0.2 || out[1] != 0.4 || out[2] != 0.6 || out[3] != 1.0) {
        servo_preferences_free(prefs);
        return 1;
    }

    servo_preferences_free(prefs);
    return 0;
}

int run_c_api_tests(void) {
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
    RUN_TEST(test_preferences_f64_array4_roundtrip);

    printf("\n%d tests, %d passed, %d failed\n",
           tests_run, tests_run - tests_failed, tests_failed);
    return tests_failed > 0 ? 1 : 0;
}
