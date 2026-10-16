# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# ServoConfig.cmake - CMake package configuration file for Servo C API
#
# Provides the imported target Servo::servo_capi (and alias servo_capi::servo_capi)
# linking against the installed servo-capi shared library.

get_filename_component(_SERVO_CAPI_DIR "${CMAKE_CURRENT_LIST_DIR}" PATH)
get_filename_component(_SERVO_PREFIX "${CMAKE_CURRENT_LIST_DIR}/../../.." ABSOLUTE)

if(NOT TARGET Servo::servo_capi)
    add_library(Servo::servo_capi SHARED IMPORTED)

    set_target_properties(Servo::servo_capi PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${_SERVO_PREFIX}/include"
    )

    if(WIN32)
        find_file(_SERVO_CAPI_DLL
            NAMES servo_capi.dll
            PATHS "${_SERVO_PREFIX}/bin" "${_SERVO_PREFIX}/lib"
            NO_DEFAULT_PATH)

        find_library(_SERVO_CAPI_IMPLIB
            NAMES servo_capi.dll.a servo_capi.lib servo_capi
            PATHS "${_SERVO_PREFIX}/lib"
            NO_DEFAULT_PATH)

        if(_SERVO_CAPI_IMPLIB)
            set_target_properties(Servo::servo_capi PROPERTIES
                IMPORTED_IMPLIB "${_SERVO_CAPI_IMPLIB}")
        endif()

        if(_SERVO_CAPI_DLL)
            set_target_properties(Servo::servo_capi PROPERTIES
                IMPORTED_LOCATION "${_SERVO_CAPI_DLL}")
        elseif(_SERVO_CAPI_IMPLIB)
            set_target_properties(Servo::servo_capi PROPERTIES
                IMPORTED_LOCATION "${_SERVO_CAPI_IMPLIB}")
        endif()
    elseif(APPLE)
        find_library(_SERVO_CAPI_LIB
            NAMES servo_capi libservo_capi.dylib
            PATHS "${_SERVO_PREFIX}/lib"
            NO_DEFAULT_PATH)
        if(_SERVO_CAPI_LIB)
            set_target_properties(Servo::servo_capi PROPERTIES
                IMPORTED_LOCATION "${_SERVO_CAPI_LIB}")
        endif()
    else()
        find_library(_SERVO_CAPI_LIB
            NAMES servo_capi libservo_capi.so
            PATHS "${_SERVO_PREFIX}/lib"
            NO_DEFAULT_PATH)
        if(_SERVO_CAPI_LIB)
            set_target_properties(Servo::servo_capi PROPERTIES
                IMPORTED_LOCATION "${_SERVO_CAPI_LIB}")
        endif()
    endif()

    if(NOT TARGET servo_capi::servo_capi)
        add_library(servo_capi::servo_capi ALIAS Servo::servo_capi)
    endif()
endif()

set(Servo_FOUND TRUE)
set(servo_capi_FOUND TRUE)
