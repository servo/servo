NDK_TOOLCHAIN_VERSION := clang
APP_MODULES := servojni
# APP_PLATFORM is provided by gradle (see servoview/build.gradle.kts)
APP_STL := c++_shared
APP_ABI := armeabi-v7a x86 x86_64
ifeq ($(NDK_DEBUG),1)
  APP_STRIP_MODE := none
endif
