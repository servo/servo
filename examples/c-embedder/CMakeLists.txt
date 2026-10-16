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
if(SERVO_SYSROOT STREQUAL "")
    message(FATAL_ERROR
        "Set SERVO_SYSROOT (via -DSERVO_SYSROOT=... or the environment) to the "
        "servo-capi install prefix, i.e. the --prefix passed to `cargo cinstall`.")
endif()

add_executable(servo_embed main.c)

target_include_directories(servo_embed PRIVATE "${SERVO_SYSROOT}/include")

find_library(SERVO_CAPI_LIBRARY
    NAMES servo_capi
    PATHS "${SERVO_SYSROOT}/lib"
    REQUIRED)
target_link_libraries(servo_embed PRIVATE "${SERVO_CAPI_LIBRARY}")

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
    foreach(dll ${SERVO_CAPI_DLLS})
        add_custom_command(TARGET servo_embed POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${dll}" "$<TARGET_FILE_DIR:servo_embed>")
    endforeach()
endif()
