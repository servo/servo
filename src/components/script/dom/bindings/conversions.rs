/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSBool, JSContext};
use js::jsapi::{JS_ValueToUint64, JS_ValueToInt64};
use js::jsapi::{JS_ValueToECMAUint32, JS_ValueToECMAInt32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::jsval::JSVal;
use js::jsval::{NullValue, BooleanValue, Int32Value, UInt32Value};
use js::glue::RUST_JS_NumberValue;

pub trait ToJSValConvertible {
    fn to_jsval(&self) -> JSVal;
}

pub trait FromJSValConvertible {
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


impl ToJSValConvertible for bool {
    fn to_jsval(&self) -> JSVal {
        BooleanValue(*self)
    }
}

impl FromJSValConvertible for bool {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<bool, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToBoolean) };
        result.map(|b| b != 0)
    }
}

impl ToJSValConvertible for i8 {
    fn to_jsval(&self) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for i8 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i8)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for u8 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as u8)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for i16 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i16, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i16)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for u16 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u16, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self) -> JSVal {
        Int32Value(*self)
    }
}

impl FromJSValConvertible for i32 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        UInt32Value(*self)
    }
}

impl FromJSValConvertible for u32 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAUint32) }
    }
}

impl ToJSValConvertible for i64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible for i64 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<i64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToInt64) }
    }
}

impl ToJSValConvertible for u64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible for u64 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<u64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint64) }
    }
}

impl ToJSValConvertible for f32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible for f32 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<f32, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) };
        result.map(|f| f as f32)
    }
}

impl ToJSValConvertible for f64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self)
        }
    }
}

impl FromJSValConvertible for f64 {
    fn from_jsval(cx: *JSContext, val: JSVal) -> Result<f64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self) -> JSVal {
        match self {
            &Some(ref value) => value.to_jsval(),
            &None => NullValue(),
        }
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Option<T> {
    fn from_jsval(cx: *JSContext, value: JSVal) -> Result<Option<T>, ()> {
        if value.is_null_or_undefined() {
            Ok(None)
        } else {
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value);
            result.map(|v| Some(v))
        }
    }
}
