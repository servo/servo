cmake_minimum_required(VERSION 3.20)
project(servo_browser CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_compile_definitions(UNICODE _UNICODE NOMINMAX WIN32_LEAN_AND_MEAN)

if(NOT DEFINED SERVO_SYSROOT OR SERVO_SYSROOT STREQUAL "")
    set(SERVO_SYSROOT "$ENV{SERVO_SYSROOT}")
endif()
if(SERVO_SYSROOT AND NOT CMAKE_PREFIX_PATH)
    list(APPEND CMAKE_PREFIX_PATH "${SERVO_SYSROOT}")
endif()

set(SOURCES
    src/main.cpp
    src/browser_window.cpp
    src/servo_view_host.cpp
    src/tab_strip.cpp
    src/omnibox.cpp
    src/find_bar.cpp
    src/download_manager.cpp
    src/browser.rc
)

set(HEADERS
    src/browser_window.h
    src/servo_view_host.h
    src/servo_raw_window.h
    src/tab_strip.h
    src/omnibox.h
    src/find_bar.h
    src/download_manager.h
    src/fluent_theme.h
)

add_executable(servo_browser WIN32 ${SOURCES} ${HEADERS})

target_include_directories(servo_browser PRIVATE src ${CMAKE_PREFIX_PATH}/include)

# 1. Try finding Servo package config
find_package(Servo CONFIG QUIET)

if(TARGET Servo::servo_capi)
    message(STATUS "Found Servo C API via CMake package config")
    target_link_libraries(servo_browser PRIVATE Servo::servo_capi)
else()
    # 2. Try direct sysroot include and library search
    find_path(SERVO_INCLUDE_DIR servo/servo_capi.h
        HINTS ${CMAKE_PREFIX_PATH}/include ${SERVO_SYSROOT}/include
    )
    find_library(SERVO_LIBRARY NAMES servo_capi
        HINTS ${CMAKE_PREFIX_PATH}/lib ${SERVO_SYSROOT}/lib
    )

    if(SERVO_INCLUDE_DIR AND SERVO_LIBRARY)
        message(STATUS "Found Servo C API directly at ${SERVO_LIBRARY}")
        target_include_directories(servo_browser PRIVATE ${SERVO_INCLUDE_DIR})
        target_link_libraries(servo_browser PRIVATE ${SERVO_LIBRARY})
    else()
        message(WARNING "Servo C API not found locally. Ensure CMAKE_PREFIX_PATH or SERVO_SYSROOT is set when building.")
    endif()
endif()

# Windows system libraries
target_link_libraries(servo_browser PRIVATE
    dwmapi
    uxtheme
    comctl32
    shlwapi
    user32
    gdi32
    ole32
    shell32
)

set_target_properties(servo_browser PROPERTIES
    OUTPUT_NAME "servo_browser"
    WIN32_EXECUTABLE TRUE
)

# Unit test suite
enable_testing()
add_executable(test_browser_unit tests/test_browser_unit.cpp)
target_include_directories(test_browser_unit PRIVATE src tests/mock_servo)
add_test(NAME BrowserUnitTests COMMAND test_browser_unit)

