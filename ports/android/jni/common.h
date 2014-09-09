#pragma once
#include <android/log.h>

#undef LOGI
#undef LOGW

#define LOG_TAG "android-dl"
#define LOGI(message, ...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "%s: " message, __FUNCTION__, ##__VA_ARGS__)
#define LOGW(message, ...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, "%s: " message, __FUNCTION__, ##__VA_ARGS__)
#define LOGE(message, ...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "%s: " message, __FUNCTION__, ##__VA_ARGS__)
#define LOGF(message, ...) __android_log_print(ANDROID_LOG_FATAL, LOG_TAG, "%s: " message, __FUNCTION__, ##__VA_ARGS__)

/* Defines the signature of the function that's callable through Java dlcall */
typedef int (*android_dlcall_func_t)(int, const char **);

void free_ptrarray(void **pa);


