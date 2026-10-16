/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
 * Compiles the generated C header and the C++ RAII helpers (servo_capi.hpp)
 * as C++ and exercises them. This is what guarantees the public header stays
 * usable from C++ (see `cpp_compat` in ffi/capi/cbindgen.toml).
 */

#include "servo_capi.hpp"

#include <cstdio>
#include <cstdint>
#include <utility>

// The test runner expects us to return 0 for success and
// non-zero for failure.

static int tests_run;
static int tests_failed;

#define RUN_TEST(name)                                          \
    do {                                                        \
        tests_run++;                                            \
        std::printf("  RUN  %s\n", #name);                      \
        std::fflush(stdout);                                    \
        int result = name();                                    \
        if (result != 0) {                                      \
            std::printf("  FAIL %s (returned %d)\n", #name, result); \
            tests_failed++;                                     \
        }                                                       \
        std::fflush(stdout);                                    \
    } while (0)

/* RAII handles are released automatically, so a leak (or a crash from a
 * double free) would show up here. */
static int test_raii_handles(void)
{
    servo::ServoOptions options(servo_options_create());
    if (!options)
        return 1;

    servo::ServoPreferences preferences(servo_preferences_create());
    if (!preferences)
        return 1;

    servo::ServoValue value(servo_value_create());
    if (!value)
        return 1;

    servo::ServoDictionaryValue dictionary(servo_dictionary_create());
    if (!dictionary)
        return 1;

    servo::ServoListValue list(servo_list_create());
    if (!list)
        return 1;

    return 0;
}

/* Handles are move-only. */
static int test_raii_move(void)
{
    servo::ServoValue value(servo_value_create());
    if (!value)
        return 1;

    servo::ServoValue moved(std::move(value));
    if (value != nullptr)
        return 1;
    if (!moved)
        return 1;

    return 0;
}

/* `servo_builder_set_options` takes ownership of the options. `release()`
 * hands the raw pointer over so the wrapper no longer owns it. */
static int test_raii_release_transfers_ownership(void)
{
    servo::ServoBuilder builder(servo_builder_create());
    if (!builder)
        return 1;

    servo::ServoOptions options(servo_options_create());
    if (!options)
        return 1;

    if (servo_builder_set_options(builder.get(), options.release()) != SERVO_ERROR_OK)
        return 1;
    if (options != nullptr)
        return 1;

    return 0;
}

extern "C" void test_on_devtools_started(struct Servo* servo, uint16_t port,
    const char* token, void* user_data)
{
    (void)servo;
    (void)port;
    (void)token;
    (void)user_data;
}

extern "C" bool test_on_devtools_connection_request(struct Servo* servo, void* user_data)
{
    (void)servo;
    (void)user_data;
    return false;
}

/* The delegate structs are part of the public ABI and must be usable from C++.
 * (C++ has no C99 designated initializers before C++20, so zero-init first.) */
static int test_delegate_struct_usable_from_cpp(void)
{
    servo::ServoBuilder builder(servo_builder_create());
    if (!builder)
        return 1;

    ServoDelegate delegate = {};
    delegate.struct_size = sizeof(ServoDelegate);
    delegate.user_data = nullptr;
    delegate.notify_devtools_server_started = test_on_devtools_started;
    delegate.request_devtools_connection = test_on_devtools_connection_request;

    if (servo_builder_set_delegate(builder.get(), delegate) != SERVO_ERROR_OK)
        return 1;

    /* `ServoWebViewDelegate` is far larger; just make sure its fields are
     * addressable and assignable from C++. */
    ServoWebViewDelegate webview_delegate = {};
    webview_delegate.struct_size = sizeof(ServoWebViewDelegate);
    webview_delegate.user_data = nullptr;
    webview_delegate.notify_load_status_changed = nullptr;
    webview_delegate.notify_url_changed = nullptr;

    if (webview_delegate.struct_size != sizeof(ServoWebViewDelegate))
        return 1;

    return 0;
}

/* The packed integer version query must agree with the constant compiled into
 * the header, even when the header is consumed from C++. */
static int test_version_query(void)
{
    if (servo_capi_get_version() != SERVO_CAPI_VERSION)
        return 1;

    return 0;
}

extern "C" int run_cpp_api_tests(void)
{
    tests_run = 0;
    tests_failed = 0;

    RUN_TEST(test_raii_handles);
    RUN_TEST(test_raii_move);
    RUN_TEST(test_raii_release_transfers_ownership);
    RUN_TEST(test_delegate_struct_usable_from_cpp);
    RUN_TEST(test_version_query);

    std::printf("\n%d C++ tests, %d passed, %d failed\n", tests_run, tests_run - tests_failed,
        tests_failed);
    return tests_failed > 0 ? 1 : 0;
}
