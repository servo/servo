#pragma once

#include <dlfcn.h>

#if defined(__cplusplus)
extern "C" {
#endif

__attribute__ ((visibility("default")))
char ** android_dlneeds(const char *library);

__attribute__ ((visibility("default")))
void * android_dlopen(const char *library);

__attribute__ ((visibility("default")))
void * android_dlsym(void *handle, const char *symbol);

__attribute__ ((visibility("default")))
int android_dladdr(void *addr, Dl_info *info);

__attribute__ ((visibility("default")))
int android_dlclose(void *handle);

__attribute__ ((visibility("default")))
const char * android_dl_get_last_error();

#if defined(__cplusplus)
} // extern "C"
#endif
