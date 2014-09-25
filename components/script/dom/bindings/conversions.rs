/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Conversions of Rust values to and from `JSVal`.

use dom::bindings::js::{JS, JSRef, Root};
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::jsstring_to_str;
use dom::bindings::utils::unwrap_jsmanaged;
use servo_util::str::DOMString;

use js::jsapi::{JSBool, JSContext, JSObject};
use js::jsapi::{JS_ValueToUint64, JS_ValueToInt64};
use js::jsapi::{JS_ValueToECMAUint32, JS_ValueToECMAInt32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::jsapi::{JS_ValueToString, JS_GetStringCharsAndLength};
use js::jsapi::{JS_NewUCStringCopyN, JS_NewStringCopyN};
use js::jsapi::{JS_WrapValue};
use js::jsval::JSVal;
use js::jsval::{UndefinedValue, NullValue, BooleanValue, Int32Value, UInt32Value};
use js::jsval::{StringValue, ObjectValue, ObjectOrNullValue};
use js::glue::RUST_JS_NumberValue;
use libc;
use std::default;
use std::slice;

use dom::bindings::codegen::PrototypeList;

// FIXME (https://github.com/rust-lang/rfcs/pull/4)
//       remove Option<Self> arguments.
pub trait IDLInterface {
    fn get_prototype_id(_: Option<Self>) -> PrototypeList::id::ID;
    fn get_prototype_depth(_: Option<Self>) -> uint;
}

/// A trait to convert Rust types to `JSVal`s.
pub trait ToJSValConvertible {
    /// Convert `self` to a `JSVal`. JSAPI failure causes a task failure.
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal;
}

/// A trait to convert `JSVal`s to Rust types.
pub trait FromJSValConvertible<T> {
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: T) -> Result<Self, ()>;
}


impl ToJSValConvertible for () {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        UndefinedValue()
    }
}

impl ToJSValConvertible for JSVal {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        let mut value = *self;
        if unsafe { JS_WrapValue(cx, &mut value) } == 0 {
            fail!("JS_WrapValue failed.");
        }
        value
    }
}

unsafe fn convert_from_jsval<T: default::Default>(
    cx: *mut JSContext, value: JSVal,
    convert_fn: unsafe extern "C" fn(*mut JSContext, JSVal, *mut T) -> JSBool) -> Result<T, ()> {
    let mut ret = default::Default::default();
    if convert_fn(cx, value, &mut ret) == 0 {
        Err(())
    } else {
        Ok(ret)
    }
}


impl ToJSValConvertible for bool {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        BooleanValue(*self)
    }
}

impl FromJSValConvertible<()> for bool {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<bool, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToBoolean) };
        result.map(|b| b != 0)
    }
}

impl ToJSValConvertible for i8 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for i8 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<i8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i8)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for u8 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<u8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as u8)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for i16 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<i16, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i16)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for u16 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<u16, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self)
    }
}

impl FromJSValConvertible<()> for i32 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<i32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        UInt32Value(*self)
    }
}

impl FromJSValConvertible<()> for u32 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<u32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAUint32) }
    }
}

impl ToJSValConvertible for i64 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for i64 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<i64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToInt64) }
    }
}

impl ToJSValConvertible for u64 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for u64 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<u64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint64) }
    }
}

impl ToJSValConvertible for f32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for f32 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<f32, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) };
        result.map(|f| f as f32)
    }
}

impl ToJSValConvertible for f64 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self)
        }
    }
}

impl FromJSValConvertible<()> for f64 {
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<f64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}

impl ToJSValConvertible for DOMString {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let string_utf16: Vec<u16> = self.as_slice().utf16_units().collect();
            let jsstr = JS_NewUCStringCopyN(cx, string_utf16.as_ptr(), string_utf16.len() as libc::size_t);
            if jsstr.is_null() {
                fail!("JS_NewUCStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

/// Behavior for stringification of `JSVal`s.
#[deriving(PartialEq)]
pub enum StringificationBehavior {
    /// Convert `null` to the string `"null"`.
    Default,
    /// Convert `null` to the empty string.
    Empty,
}

impl default::Default for StringificationBehavior {
    fn default() -> StringificationBehavior {
        Default
    }
}

impl FromJSValConvertible<StringificationBehavior> for DOMString {
    fn from_jsval(cx: *mut JSContext, value: JSVal, nullBehavior: StringificationBehavior) -> Result<DOMString, ()> {
        if nullBehavior == Empty && value.is_null() {
            Ok("".to_string())
        } else {
            let jsstr = unsafe { JS_ValueToString(cx, value) };
            if jsstr.is_null() {
                debug!("JS_ValueToString failed");
                Err(())
            } else {
                Ok(jsstring_to_str(cx, jsstr))
            }
        }
    }
}

impl ToJSValConvertible for ByteString {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let slice = self.as_slice();
            let jsstr = JS_NewStringCopyN(cx, slice.as_ptr() as *const libc::c_char,
                                          slice.len() as libc::size_t);
            if jsstr.is_null() {
                fail!("JS_NewStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

impl FromJSValConvertible<()> for ByteString {
    fn from_jsval(cx: *mut JSContext, value: JSVal, _option: ()) -> Result<ByteString, ()> {
        unsafe {
            let string = JS_ValueToString(cx, value);
            if string.is_null() {
                debug!("JS_ValueToString failed");
                return Err(());
            }

            let mut length = 0;
            let chars = JS_GetStringCharsAndLength(cx, string, &mut length);
            slice::raw::buf_as_slice(chars, length as uint, |char_vec| {
                if char_vec.iter().any(|&c| c > 0xFF) {
                    // XXX Throw
                    Err(())
                } else {
                    Ok(ByteString::new(char_vec.iter().map(|&c| c as u8).collect()))
                }
            })
        }
    }
}

impl ToJSValConvertible for Reflector {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        let obj = self.get_jsobject();
        assert!(obj.is_not_null());
        let mut value = ObjectValue(unsafe { &*obj });
        if unsafe { JS_WrapValue(cx, &mut value) } == 0 {
            fail!("JS_WrapValue failed.");
        }
        value
    }
}

impl<T: Reflectable+IDLInterface> FromJSValConvertible<()> for JS<T> {
    fn from_jsval(_cx: *mut JSContext, value: JSVal, _option: ()) -> Result<JS<T>, ()> {
        if !value.is_object() {
            return Err(());
        }
        unwrap_jsmanaged(value.to_object(),
                         IDLInterface::get_prototype_id(None::<T>),
                         IDLInterface::get_prototype_depth(None::<T>))
    }
}

impl<'a, 'b, T: Reflectable> ToJSValConvertible for Root<'a, 'b, T> {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.reflector().to_jsval(cx)
    }
}

impl<'a, T: Reflectable> ToJSValConvertible for JSRef<'a, T> {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.reflector().to_jsval(cx)
    }
}

impl<'a, T: Reflectable> ToJSValConvertible for JS<T> {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.reflector().to_jsval(cx)
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        match self {
            &Some(ref value) => value.to_jsval(cx),
            &None => NullValue(),
        }
    }
}

impl<X: default::Default, T: FromJSValConvertible<X>> FromJSValConvertible<()> for Option<T> {
    fn from_jsval(cx: *mut JSContext, value: JSVal, _: ()) -> Result<Option<T>, ()> {
        if value.is_null_or_undefined() {
            Ok(None)
        } else {
            let option: X = default::Default::default();
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value, option);
            result.map(Some)
        }
    }
}

impl ToJSValConvertible for *mut JSObject {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        let mut wrapped = ObjectOrNullValue(*self);
        unsafe {
            assert!(JS_WrapValue(cx, &mut wrapped) != 0);
        }
        wrapped
    }
}
