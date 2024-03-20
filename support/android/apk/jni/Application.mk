NDK_TOOLCHAIN_VERSION := clang
APP_MODULES := c++_shared servojni
APP_PLATFORM := android-30
APP_STL := c++_shared
APP_ABI := armeabi-v7a x86 x86_64
ifeq ($(NDK_DEBUG),1)
  APP_STRIP_MODE := none
endif
