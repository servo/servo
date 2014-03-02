/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSVal, JSBool, JSContext};
use js::jsapi::{JS_ValueToInt64, JS_ValueToECMAInt32, JS_ValueToECMAUint32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::{JSVAL_FALSE, JSVAL_TRUE};
use js::glue::{RUST_UINT_TO_JSVAL, RUST_DOUBLE_TO_JSVAL};

pub trait JSValConvertible {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<Self>;
}


unsafe fn convert_from_jsval<T: Default>(
    cx: *JSContext, value: JSVal,
    convert_fn: extern "C" unsafe fn(*JSContext, JSVal, *T) -> JSBool) -> Option<T> {
    let mut ret = Default::default();
    if convert_fn(cx, value, &mut ret as *mut T as *T) == 0 {
        None
    } else {
        Some(ret)
    }
}


impl JSValConvertible for i64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<i64> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToInt64) }
    }
}

impl JSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<u32> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAUint32) }
    }
}

impl JSValConvertible for i32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<i32> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl JSValConvertible for u16 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<u16> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl JSValConvertible for bool {
    fn to_jsval(&self) -> JSVal {
        if *self {
            JSVAL_TRUE
        } else {
            JSVAL_FALSE
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<bool> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToBoolean) };
        result.map(|b| b != 0)
    }
}

impl JSValConvertible for f32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<f32> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) };
        result.map(|f| f as f32)
    }
}

impl JSValConvertible for f64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<f64> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}
