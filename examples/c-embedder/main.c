/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* servo_embed - a minimal, standalone C11 embedder for Servo.
 *
 * This is the "hello world" embedder referenced by
 * docs/embedding/C_API_GUIDE.md. Unlike the in-tree C tests under tests/capi,
 * it does not depend on the Servo source tree: it builds entirely against an
 * installed servo-capi sysroot, which is what a third-party embedder does.
 *
 * Build (CMake variant in CMakeLists.txt):
 *
 *   cargo cinstall -p servo-capi --library-type cdylib --prefix /path/to/sysroot
 *   cc -std=c11 -Wall -Wextra main.c \
 *      -I/path/to/sysroot/include -L/path/to/sysroot/lib \
 *      -lservo_capi -o servo_embed
 *
 * On macOS, `cc` accepts the same command. On Windows with MSVC, add
 * `/experimental:c11atomics` (this file uses <stdatomic.h>), or build with
 * clang-cl / MinGW.
 *
 * At runtime the loader must find the shared library: LD_LIBRARY_PATH on Linux,
 * DYLD_LIBRARY_PATH on macOS, or servo_capi.dll next to the executable on
 * Windows.
 *
 * Usage: servo_embed [url]
 */

#define _POSIX_C_SOURCE 200809L /* for nanosleep on POSIX */
#include <servo/servo_capi.h>

#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <time.h>
#endif

/* ------------------------------------------------------------------ helpers */

static void sleep_ms(long period)
{
#ifdef _WIN32
    Sleep((DWORD)period);
#else
    struct timespec delay;
    delay.tv_sec = period / 1000;
    delay.tv_nsec = (period % 1000) * 1000000L;
    nanosleep(&delay, NULL);
#endif
}

/* ------------------------------------------------------------------- state */

/* Embedder state, passed to callbacks as user_data. It must outlive the
 * WebView, so it lives in main()'s frame (or statically). */
typedef struct {
    atomic_bool new_frame_pending;
    atomic_bool load_complete;
} app_state_t;

/* The waker may be called from any Servo-internal thread. Keep it tiny: just
 * record that work is pending. Do not touch Servo handles here. */
static atomic_bool g_wake_pending = ATOMIC_VAR_INIT(false);

static void on_wake(void)
{
    atomic_store_explicit(&g_wake_pending, true, memory_order_release);
}

/* --- Delegate callbacks. All run on the thread calling
 * servo_spin_event_loop. Never free a handle from inside a callback. --- */

static void on_load_status_changed(struct WebView* webview, int32_t load_status,
    void* user_data)
{
    (void)webview;
    app_state_t* app = (app_state_t*)user_data;
    const char* name = "unknown";
    switch (load_status) {
    case SERVO_LOAD_STATUS_STARTED:
        name = "started";
        break;
    case SERVO_LOAD_STATUS_HEAD_PARSED:
        name = "head-parsed";
        break;
    case SERVO_LOAD_STATUS_COMPLETE:
        name = "complete";
        atomic_store_explicit(&app->load_complete, true, memory_order_release);
        break;
    default:
        break;
    }
    printf("[load] %s\n", name);
    fflush(stdout);
}

static void on_url_changed(struct WebView* webview, const char* url, void* user_data)
{
    (void)webview;
    (void)user_data;
    /* `url` is only valid for the duration of this call. */
    printf("[url] %s\n", url);
    fflush(stdout);
}

static void on_page_title_changed(struct WebView* webview, const char* title,
    void* user_data)
{
    (void)webview;
    (void)user_data;
    /* `title` is only valid for the duration of this call. */
    printf("[title] %s\n", title);
    fflush(stdout);
}

static void on_new_frame_ready(struct WebView* webview, void* user_data)
{
    (void)webview;
    app_state_t* app = (app_state_t*)user_data;
    atomic_store_explicit(&app->new_frame_pending, true, memory_order_release);
}

static void on_screenshot(const uint8_t* data, uint32_t width, uint32_t height,
    int32_t error, void* user_data)
{
    (void)user_data;
    if (data == NULL) {
        printf("[screenshot] failed, error=%d\n", error);
        return;
    }
    printf("[screenshot] %ux%u RGBA (first pixel %u,%u,%u,%u)\n", width, height,
        data[0], data[1], data[2], data[3]);
    /* `data` is only valid for the duration of this call; copy it to keep it. */
}

static void on_javascript_result(ServoValue* result_value, const char* error_message,
    void* user_data)
{
    (void)user_data;
    if (error_message != NULL) {
        printf("[js] error: %s\n", error_message);
    } else if (result_value != NULL) {
        char buf[256];
        size_t len = servo_value_get_string(result_value, buf, sizeof(buf));
        if (len > 0) {
            printf("[js] result: %s\n", buf);
        } else {
            int32_t val_type = 0;
            servo_value_get_type(result_value, &val_type);
            printf("[js] result (type=%d)\n", (int)val_type);
        }
        servo_value_free(result_value);
    }
    /* `error_message` is only valid for the duration of this call. */
}

/* -------------------------------------------------------------------- main */

int main(int argc, char** argv)
{
    /* A data: URL by default so the example works offline and deterministically;
     * pass a URL argument to load something else. */
    const char* url = (argc > 1)
        ? argv[1]
        : "data:text/html,<title>Servo embedder</title>"
          "<h1>Hello from Servo</h1><p>The C embedder is working.</p>";

    static app_state_t app; /* zero-initialized */

    printf("servo-capi version: %s\n", servo_capi_version());

    /* 1. Options: single-process, no sandbox, temporary storage. */
    ServoOptions* options = servo_options_create();
    if (options == NULL)
        return 1;
    servo_options_set_multiprocess(options, false);
    servo_options_set_sandbox(options, false);
    servo_options_set_temporary_storage(options, false);

    /* 2. Preferences: any Servo preference can be set by name. */
    ServoPreferences* preferences = servo_preferences_create();
    if (preferences == NULL) {
        servo_options_free(options);
        return 1;
    }
    servo_preferences_set_string(preferences, "user_agent", "ServoEmbed/1.0");

    /* 3. Builder: options/preferences transfer ownership; the waker is copied
     * by value. */
    ServoBuilder* builder = servo_builder_create();
    if (builder == NULL)
        return 1;
    struct ServoEventLoopWaker waker = {
        .struct_size = sizeof(struct ServoEventLoopWaker),
        .wake_callback = on_wake,
    };
    servo_builder_set_event_loop_waker(builder, waker);
    servo_builder_set_options(builder, options);         /* builder owns options now */
    servo_builder_set_preferences(builder, preferences); /* builder owns preferences now */

    /* 4. Build Servo (consumes the builder) and enable logging.
     * Only one Servo instance may exist per process. */
    struct Servo* servo = servo_builder_build(builder); /* builder is consumed */
    if (servo == NULL) {
        fprintf(stderr, "failed to build Servo\n");
        return 1;
    }
    servo_setup_logging(servo);

    /* 5. Software rendering context (headless; works on every platform). */
    const uint32_t width = 800;
    const uint32_t height = 600;
    RenderingContext* context =
        servo_rendering_context_create_software(width, height);
    if (context == NULL) {
        fprintf(stderr, "failed to create rendering context\n");
        servo_free(servo);
        return 1;
    }

    /* 6. WebView builder and delegate. The builder takes ownership of the
     * rendering context. */
    ServoWebViewBuilder* webview_builder = servo_webview_builder_create(servo, context);
    if (webview_builder == NULL) {
        servo_rendering_context_free(context); /* we still own it here */
        servo_free(servo);
        return 1;
    }

    ServoWebViewDelegate delegate = {
        .struct_size = sizeof(ServoWebViewDelegate),
        .user_data = &app,
        .notify_load_status_changed = on_load_status_changed,
        .notify_url_changed = on_url_changed,
        .notify_page_title_changed = on_page_title_changed,
        .notify_new_frame_ready = on_new_frame_ready,
    };
    servo_webview_builder_set_delegate(webview_builder, delegate);

    if (servo_webview_builder_set_url(webview_builder, url) != 0) {
        fprintf(stderr, "invalid initial URL: %s\n", url);
        servo_webview_builder_free(webview_builder);
        servo_free(servo);
        return 1;
    }

    /* 7. Build the WebView (consumes the builder). */
    struct WebView* webview = servo_webview_builder_build(webview_builder);
    if (webview == NULL) {
        fprintf(stderr, "failed to build WebView\n");
        servo_free(servo);
        return 1;
    }

    servo_webview_set_page_zoom(webview, 1.0f);
    servo_webview_set_focused(webview, true);
    servo_webview_show(webview);

    /* Asynchronous JavaScript evaluation; the result arrives during a spin. */
    servo_webview_evaluate_javascript_v2(webview, "document.title",
        on_javascript_result, &app);

    /* 8. Manual spin loop. A real embedder blocks on the waker; this example
     * polls. All WebView calls and painting happen on this same thread. */
    int screenshot_requested = 0;
    for (int i = 0; i < 400; i++) { /* ~6 s at 15 ms per iteration */
        servo_spin_event_loop(servo);
        servo_webview_poll_js_messages(webview);

        /* Wake handling: clear the flag. A blocking loop would use it to
         * decide when to wake up. */
        atomic_store_explicit(&g_wake_pending, false, memory_order_release);

        if (atomic_exchange_explicit(&app.new_frame_pending, false,
                memory_order_acq_rel)) {
            servo_webview_paint(webview); /* render into the context */
        }

        if (!screenshot_requested && i == 20) {
            servo_webview_take_screenshot(webview, on_screenshot, &app);
            screenshot_requested = 1;
        }

        if (atomic_load_explicit(&app.load_complete, memory_order_acquire) && i > 20)
            break;

        sleep_ms(15);
    }

    /* 9. Report the final URL and title with the buffer-query pattern. */
    uintptr_t url_len = servo_webview_get_url(webview, NULL, 0);
    if (url_len > 0) {
        char* buffer = (char*)malloc((size_t)url_len);
        if (buffer != NULL) {
            servo_webview_get_url(webview, buffer, url_len);
            printf("[final] url = %s\n", buffer);
            free(buffer);
        }
    }
    uintptr_t title_len = servo_webview_get_title(webview, NULL, 0);
    if (title_len > 0) {
        char* buffer = (char*)malloc((size_t)title_len);
        if (buffer != NULL) {
            servo_webview_get_title(webview, buffer, title_len);
            printf("[final] title = %s\n", buffer);
            free(buffer);
        }
    }

    /* 10. Tear down in reverse order. WebViews before Servo. The rendering
     * context is owned by the WebView and is not freed separately. */
    servo_webview_free(webview);
    servo_free(servo);
    return 0;
}
