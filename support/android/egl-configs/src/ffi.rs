/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_camel_case_types)]

use std::os::raw::*;

pub type NativeDisplayType = *const c_void;
pub type NativePixmapType = *const c_void;
pub type NativeWindowType = *const c_void;

pub type EGLNativeDisplayType = NativeDisplayType;
pub type EGLNativePixmapType = NativePixmapType;
pub type EGLNativeWindowType = NativeWindowType;

pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
pub type khronos_uint64_t = u64;
pub type khronos_ssize_t = c_long;
pub type EGLint = i32;

#[link(name = "EGL")]
extern "C" {}

include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));
