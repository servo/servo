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

static void noop_wake_callback(void) {}

int test_builder_set_event_loop_waker(void) {
    ServoBuilder *builder = servo_builder_create();
    if (builder == NULL) return 1;

    struct ServoEventLoopWaker waker = { noop_wake_callback };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_free(builder);
    return 0;
}

int run_c_api_tests(void) {
    tests_run = 0;
    tests_failed = 0;

    RUN_TEST(test_builder_create_and_free);
    RUN_TEST(test_builder_set_event_loop_waker);

    printf("\n%d tests, %d passed, %d failed\n",
           tests_run, tests_run - tests_failed, tests_failed);
    return tests_failed > 0 ? 1 : 0;
}
