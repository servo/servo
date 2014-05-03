/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, JSRef, Root};
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::jsstring_to_str;
use dom::bindings::utils::unwrap_jsmanaged;
use servo_util::str::DOMString;

use js::jsapi::{JSBool, JSContext};
use js::jsapi::{JS_ValueToUint64, JS_ValueToInt64};
use js::jsapi::{JS_ValueToECMAUint32, JS_ValueToECMAInt32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::jsapi::{JS_ValueToString, JS_GetStringCharsAndLength};
use js::jsapi::{JS_NewUCStringCopyN, JS_NewStringCopyN};
use js::jsapi::{JS_WrapValue};
use js::jsval::JSVal;
use js::jsval::{UndefinedValue, NullValue, BooleanValue, Int32Value, UInt32Value};
use js::jsval::{StringValue, ObjectValue};
use js::glue::RUST_JS_NumberValue;
use libc;
use std::default::Default;
use std::slice;

use dom::bindings::codegen::PrototypeList;

// FIXME (https://github.com/rust-lang/rfcs/pull/4)
//       remove Option<Self> arguments.
pub trait IDLInterface {
    fn get_prototype_id(_: Option<Self>) -> PrototypeList::id::ID;
    fn get_prototype_depth(_: Option<Self>) -> uint;
}

pub trait ToJSValConvertible {
    fn to_jsval(&self, cx: *JSContext) -> JSVal;
}

pub trait FromJSValConvertible<T> {
    fn from_jsval(cx: *JSContext, val: JSVal, option: T) -> Result<Self, ()>;
}


impl ToJSValConvertible for () {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        UndefinedValue()
    }
}

impl ToJSValConvertible for JSVal {
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        let mut value = *self;
        if unsafe { JS_WrapValue(cx, &mut value as *mut JSVal as *JSVal) } == 0 {
            fail!("JS_WrapValue failed.");
        }
        value
    }
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
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        BooleanValue(*self)
    }
}

impl FromJSValConvertible<()> for bool {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<bool, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToBoolean) };
        result.map(|b| b != 0)
    }
}

impl ToJSValConvertible for i8 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for i8 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<i8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i8)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for u8 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<u8, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as u8)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for i16 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<i16, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) };
        result.map(|v| v as i16)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible<()> for u16 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<u16, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        Int32Value(*self)
    }
}

impl FromJSValConvertible<()> for i32 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<i32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        UInt32Value(*self)
    }
}

impl FromJSValConvertible<()> for u32 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<u32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAUint32) }
    }
}

impl ToJSValConvertible for i64 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for i64 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<i64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToInt64) }
    }
}

impl ToJSValConvertible for u64 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for u64 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<u64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint64) }
    }
}

impl ToJSValConvertible for f32 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self as f64)
        }
    }
}

impl FromJSValConvertible<()> for f32 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<f32, ()> {
        let result = unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) };
        result.map(|f| f as f32)
    }
}

impl ToJSValConvertible for f64 {
    fn to_jsval(&self, _cx: *JSContext) -> JSVal {
        unsafe {
            RUST_JS_NumberValue(*self)
        }
    }
}

impl FromJSValConvertible<()> for f64 {
    fn from_jsval(cx: *JSContext, val: JSVal, _option: ()) -> Result<f64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}

impl ToJSValConvertible for DOMString {
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        unsafe {
            let string_utf16 = self.to_utf16();
            let jsstr = JS_NewUCStringCopyN(cx, string_utf16.as_ptr(), string_utf16.len() as libc::size_t);
            if jsstr.is_null() {
                fail!("JS_NewUCStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

#[deriving(Eq)]
pub enum StringificationBehavior {
    Default,
    Empty,
}

impl Default for StringificationBehavior {
    fn default() -> StringificationBehavior {
        Default
    }
}

impl FromJSValConvertible<StringificationBehavior> for DOMString {
    fn from_jsval(cx: *JSContext, value: JSVal, nullBehavior: StringificationBehavior) -> Result<DOMString, ()> {
        if nullBehavior == Empty && value.is_null() {
            Ok("".to_owned())
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
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        unsafe {
            let slice = self.as_slice();
            let jsstr = JS_NewStringCopyN(cx, slice.as_ptr() as *libc::c_char,
                                          slice.len() as libc::size_t);
            if jsstr.is_null() {
                fail!("JS_NewStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

impl FromJSValConvertible<()> for ByteString {
    fn from_jsval(cx: *JSContext, value: JSVal, _option: ()) -> Result<ByteString, ()> {
        unsafe {
            let string = JS_ValueToString(cx, value);
            if string.is_null() {
                debug!("JS_ValueToString failed");
                return Err(());
            }

            let mut length = 0;
            let chars = JS_GetStringCharsAndLength(cx, string, &mut length as *mut _ as *_);
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
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        let obj = self.get_jsobject();
        assert!(obj.is_not_null());
        let mut value = ObjectValue(unsafe { &*obj });
        if unsafe { JS_WrapValue(cx, &mut value as *mut JSVal as *JSVal) } == 0 {
            fail!("JS_WrapValue failed.");
        }
        value
    }
}

impl<T: Reflectable+IDLInterface> FromJSValConvertible<()> for JS<T> {
    fn from_jsval(_cx: *JSContext, value: JSVal, _option: ()) -> Result<JS<T>, ()> {
        if !value.is_object() {
            return Err(());
        }
        unwrap_jsmanaged(value.to_object(),
                         IDLInterface::get_prototype_id(None::<T>),
                         IDLInterface::get_prototype_depth(None::<T>))
    }
}

impl<'a, 'b, T: Reflectable> ToJSValConvertible for Root<'a, 'b, T> {
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        self.reflector().to_jsval(cx)
    }
}

impl<'a, T: Reflectable> ToJSValConvertible for JSRef<'a, T> {
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        self.reflector().to_jsval(cx)
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self, cx: *JSContext) -> JSVal {
        match self {
            &Some(ref value) => value.to_jsval(cx),
            &None => NullValue(),
        }
    }
}

impl<X: Default, T: FromJSValConvertible<X>> FromJSValConvertible<()> for Option<T> {
    fn from_jsval(cx: *JSContext, value: JSVal, _: ()) -> Result<Option<T>, ()> {
        if value.is_null_or_undefined() {
            Ok(None)
        } else {
            let option: X = Default::default();
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value, option);
            result.map(Some)
        }
    }
}
