/*
 * Copyright (C) 2010 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

//BEGIN_INCLUDE(all)
#include <jni.h>
#include <errno.h>
#include <dlfcn.h>
#include <stdlib.h>

#include <android/log.h>
#include <android_native_app_glue.h>

#define LOGI(...) ((void)__android_log_print(ANDROID_LOG_INFO, "servo-wrapper", __VA_ARGS__))
#define LOGW(...) ((void)__android_log_print(ANDROID_LOG_WARN, "servo-wrapper", __VA_ARGS__))

/**
 * This is the main entry point of a native application that is using
 * android_native_app_glue.  It runs in its own thread, with its own
 * event loop for receiving input events and doing other things.
 */
void android_main(struct android_app* state) {
    LOGI("in android_main");
    void* libservo = dlopen("libservo.so", RTLD_NOW);
    if (libservo == NULL) {
        LOGI("failed to load servo lib: %s", dlerror());
        return;
    }

    LOGI("loaded libservo.so");
    void (*android_main)(struct android_app*);
    *(void**)(&android_main) = dlsym(libservo, "android_main");
    if (android_main) {
        LOGI("go into android_main()");
        (*android_main)(state);
        return;
    }
}
//END_INCLUDE(all)

int main(int argc, char* argv[])
{
    LOGI("WAT");
    return 0;
}
