/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSVal, JSBool, JSContext};
use js::jsapi::{JS_ValueToUint64, JS_ValueToInt64};
use js::jsapi::{JS_ValueToECMAUint32, JS_ValueToECMAInt32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::{JSVAL_FALSE, JSVAL_TRUE, JSVAL_NULL};
use js::glue::{RUST_INT_TO_JSVAL, RUST_UINT_TO_JSVAL, RUST_JS_NumberValue};
use js::glue::{RUST_JSVAL_IS_NULL, RUST_JSVAL_IS_VOID};

pub trait JSValConvertible {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<Self, ()>;
}


unsafe fn convert_from_jsval<T: Default>(
    cx: *JSContext, value: JSVal,
    convert_fn: extern "C" unsafe fn(*JSContext, JSVal, *T) -> JSBool) -> Result<T, ()> {
    let mut ret = Default::default();
    if convert_fn(cx, value, &mut ret as *mut T as *T) == 0 {
        Err(())
    } else {
        Ok(ret)
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

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<bool, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToBoolean) };
        result.map(|b| b != 0)
    }
}

impl JSValConvertible for i8 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_INT_TO_JSVAL(*self as i32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i8)
    }
}

impl JSValConvertible for u8 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_INT_TO_JSVAL(*self as i32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as u8)
    }
}

impl JSValConvertible for i16 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_INT_TO_JSVAL(*self as i32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i16, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i16)
    }
}

impl JSValConvertible for u16 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u16, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl JSValConvertible for i32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_INT_TO_JSVAL(*self)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl JSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAUint32) }
    }
}

impl JSValConvertible for i64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToInt64) }
    }
}

impl JSValConvertible for u64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint64) }
    }
}

impl JSValConvertible for f32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<f32, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) };
        result.map(|f| f as f32)
    }
}

impl JSValConvertible for f64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self)
        }
    }

    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<f64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}

impl<T: JSValConvertible> JSValConvertible for Option<T> {
    fn to_jsval(&self) -> JSVal {
        match self {
            &Some(ref value) => value.to_jsval(),
            &None => JSVAL_NULL,
        }
    }

    fn from_jsval(cx: *JSContext, value: JSVal) -> Result<Option<T>, ()> {
        if unsafe { RUST_JSVAL_IS_NULL(value) != 0 || RUST_JSVAL_IS_VOID(value) != 0 } {
            Ok(None)
        } else {
            let result: Result<T, ()> = JSValConvertible::from_jsval(cx, value);
            result.map(|v| Some(v))
        }
    }
}
