# Build the standalone servo-capi embedder against an installed sysroot.
#
#   cargo cinstall -p servo-capi --library-type cdylib --prefix /path/to/sysroot
#   cmake -S examples/c-embedder -B build -DSERVO_SYSROOT=/path/to/sysroot
#   cmake --build build
#
# SERVO_SYSROOT may also be provided through the environment.

cmake_minimum_required(VERSION 3.18)
project(servo_embed C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT DEFINED SERVO_SYSROOT OR SERVO_SYSROOT STREQUAL "")
    set(SERVO_SYSROOT "$ENV{SERVO_SYSROOT}")
endif()
if(SERVO_SYSROOT AND NOT CMAKE_PREFIX_PATH)
    list(APPEND CMAKE_PREFIX_PATH "${SERVO_SYSROOT}")
endif()
if(SERVO_SYSROOT)
    set(ENV{PKG_CONFIG_PATH} "${SERVO_SYSROOT}/lib/pkgconfig:$ENV{PKG_CONFIG_PATH}")
endif()

add_executable(servo_embed main.c)

# 1. Prefer CMake package configuration (find_package)
find_package(Servo CONFIG QUIET)

if(TARGET Servo::servo_capi)
    message(STATUS "Found Servo C API via CMake package config (Servo::servo_capi)")
    target_link_libraries(servo_embed PRIVATE Servo::servo_capi)
else()
    # 2. Try pkg-config
    find_package(PkgConfig QUIET)
    if(PKG_CONFIG_FOUND)
        pkg_check_modules(SERVO_CAPI QUIET IMPORTED_TARGET servo_capi)
    endif()

    if(TARGET PkgConfig::SERVO_CAPI)
        message(STATUS "Found Servo C API via pkg-config (PkgConfig::SERVO_CAPI)")
        target_link_libraries(servo_embed PRIVATE PkgConfig::SERVO_CAPI)
    else()
        # 3. Fallback to direct sysroot search
        if(SERVO_SYSROOT STREQUAL "")
            message(FATAL_ERROR
                "Could not find Servo C API. Provide CMAKE_PREFIX_PATH or SERVO_SYSROOT "
                "pointing to the installed servo-capi sysroot prefix.")
        endif()
        message(STATUS "Using fallback direct paths from SERVO_SYSROOT: ${SERVO_SYSROOT}")
        target_include_directories(servo_embed PRIVATE "${SERVO_SYSROOT}/include")
        find_library(SERVO_CAPI_LIBRARY
            NAMES servo_capi
            PATHS "${SERVO_SYSROOT}/lib"
            REQUIRED)
        target_link_libraries(servo_embed PRIVATE "${SERVO_CAPI_LIBRARY}")
    endif()
endif()

if(MSVC)
    # <stdatomic.h> requires C11 atomics support in the MSVC toolchain.
    target_compile_options(servo_embed PRIVATE /W4 /experimental:c11atomics)
else()
    target_compile_options(servo_embed PRIVATE -Wall -Wextra)
endif()

# On Windows the loader looks for servo_capi.dll next to the executable. Copy it
# there if the sysroot provides one, so `build/servo_embed.exe` just runs.
if(WIN32)
    file(GLOB SERVO_CAPI_DLLS "${SERVO_SYSROOT}/bin/*.dll")
    if(NOT SERVO_CAPI_DLLS AND CMAKE_PREFIX_PATH)
        file(GLOB SERVO_CAPI_DLLS "${CMAKE_PREFIX_PATH}/bin/*.dll")
    endif()
    foreach(dll ${SERVO_CAPI_DLLS})
        add_custom_command(TARGET servo_embed POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${dll}" "$<TARGET_FILE_DIR:servo_embed>")
    endforeach()
endif()
