/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Conversions of Rust values to and from `JSVal`.
//!
//! | IDL type                | Argument type   | Return type    |
//! |-------------------------|-----------------|----------------|
//! | any                     | `JSVal`                          |
//! | boolean                 | `bool`                           |
//! | byte                    | `i8`                             |
//! | octet                   | `u8`                             |
//! | short                   | `i16`                            |
//! | unsigned short          | `u16`                            |
//! | long                    | `i32`                            |
//! | unsigned long           | `u32`                            |
//! | long long               | `i64`                            |
//! | unsigned long long      | `u64`                            |
//! | float                   | `f32`                            |
//! | double                  | `f64`                            |
//! | DOMString               | `DOMString`                      |
//! | ByteString              | `ByteString`                     |
//! | object                  | `*mut JSObject`                  |
//! | interface types         | `JSRef<T>`      | `Temporary<T>` |
//! | dictionary types        | `&T`            | *unsupported*  |
//! | enumeration types       | `T`                              |
//! | callback function types | `T`                              |
//! | nullable types          | `Option<T>`                      |
//! | sequences               | `Vec<T>`                         |
//! | union types             | `T`                              |

use dom::bindings::codegen::PrototypeList;
use dom::bindings::js::{JS, JSRef, Root, Unrooted};
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflectable, Reflector, DOMClass};
use util::str::DOMString;

use js;
use js::glue::{RUST_JSID_TO_STRING, RUST_JSID_IS_STRING};
use js::glue::RUST_JS_NumberValue;
use js::jsapi::{JSBool, JSContext, JSObject, JSString, jsid};
use js::jsapi::{JS_ValueToUint64, JS_ValueToInt64};
use js::jsapi::{JS_ValueToECMAUint32, JS_ValueToECMAInt32};
use js::jsapi::{JS_ValueToUint16, JS_ValueToNumber, JS_ValueToBoolean};
use js::jsapi::{JS_ValueToString, JS_GetStringCharsAndLength};
use js::jsapi::{JS_NewUCStringCopyN, JS_NewStringCopyN};
use js::jsapi::{JS_WrapValue};
use js::jsapi::{JSClass, JS_GetClass};
use js::jsval::JSVal;
use js::jsval::{UndefinedValue, NullValue, BooleanValue, Int32Value, UInt32Value};
use js::jsval::{StringValue, ObjectValue, ObjectOrNullValue};

use libc;
use std::borrow::ToOwned;
use std::default;
use std::slice;

/// A trait to retrieve the constants necessary to check if a `JSObject`
/// implements a given interface.
// FIXME (https://github.com/rust-lang/rfcs/pull/4)
//       remove Option<Self> arguments.
pub trait IDLInterface {
    /// Returns the prototype ID.
    fn get_prototype_id(_: Option<Self>) -> PrototypeList::ID;
    /// Returns the prototype depth, i.e., the number of interfaces this
    /// interface inherits from.
    fn get_prototype_depth(_: Option<Self>) -> uint;
}

/// A trait to convert Rust types to `JSVal`s.
pub trait ToJSValConvertible {
    /// Convert `self` to a `JSVal`. JSAPI failure causes a task failure.
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal;
}

/// A trait to convert `JSVal`s to Rust types.
pub trait FromJSValConvertible {
    type Config;
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: Self::Config) -> Result<Self, ()>;
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
            panic!("JS_WrapValue failed.");
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

impl FromJSValConvertible for bool {
    type Config = ();
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

impl FromJSValConvertible for i8 {
    type Config = ();
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

impl FromJSValConvertible for u8 {
    type Config = ();
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

impl FromJSValConvertible for i16 {
    type Config = ();
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

impl FromJSValConvertible for u16 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<u16, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToUint16) }
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self)
    }
}

impl FromJSValConvertible for i32 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<i32, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToECMAInt32) }
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        UInt32Value(*self)
    }
}

impl FromJSValConvertible for u32 {
    type Config = ();
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

impl FromJSValConvertible for i64 {
    type Config = ();
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

impl FromJSValConvertible for u64 {
    type Config = ();
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

impl FromJSValConvertible for f32 {
    type Config = ();
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

impl FromJSValConvertible for f64 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: JSVal, _option: ()) -> Result<f64, ()> {
        unsafe { convert_from_jsval(cx, val, JS_ValueToNumber) }
    }
}

impl ToJSValConvertible for str {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let string_utf16: Vec<u16> = self.utf16_units().collect();
            let jsstr = JS_NewUCStringCopyN(cx, string_utf16.as_ptr(), string_utf16.len() as libc::size_t);
            if jsstr.is_null() {
                panic!("JS_NewUCStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

impl ToJSValConvertible for DOMString {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.as_slice().to_jsval(cx)
    }
}

/// Behavior for stringification of `JSVal`s.
#[derive(PartialEq)]
pub enum StringificationBehavior {
    /// Convert `null` to the string `"null"`.
    Default,
    /// Convert `null` to the empty string.
    Empty,
}

impl default::Default for StringificationBehavior {
    fn default() -> StringificationBehavior {
        StringificationBehavior::Default
    }
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
pub fn jsstring_to_str(cx: *mut JSContext, s: *mut JSString) -> DOMString {
    unsafe {
        let mut length = 0;
        let chars = JS_GetStringCharsAndLength(cx, s, &mut length);
        let char_vec = slice::from_raw_parts(chars, length as uint);
        String::from_utf16(char_vec).unwrap()
    }
}

/// Convert the given `jsid` to a `DOMString`. Fails if the `jsid` is not a
/// string, or if the string does not contain valid UTF-16.
pub fn jsid_to_str(cx: *mut JSContext, id: jsid) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id) != 0);
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

impl FromJSValConvertible for DOMString {
    type Config = StringificationBehavior;
    fn from_jsval(cx: *mut JSContext, value: JSVal,
                  null_behavior: StringificationBehavior)
                  -> Result<DOMString, ()> {
        if null_behavior == StringificationBehavior::Empty && value.is_null() {
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
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let slice = self.as_slice();
            let jsstr = JS_NewStringCopyN(cx, slice.as_ptr() as *const libc::c_char,
                                          slice.len() as libc::size_t);
            if jsstr.is_null() {
                panic!("JS_NewStringCopyN failed");
            }
            StringValue(&*jsstr)
        }
    }
}

impl FromJSValConvertible for ByteString {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, value: JSVal, _option: ()) -> Result<ByteString, ()> {
        unsafe {
            let string = JS_ValueToString(cx, value);
            if string.is_null() {
                debug!("JS_ValueToString failed");
                return Err(());
            }

            let mut length = 0;
            let chars = JS_GetStringCharsAndLength(cx, string, &mut length);
            let char_vec = slice::from_raw_parts(chars, length as uint);

            if char_vec.iter().any(|&c| c > 0xFF) {
                // XXX Throw
                Err(())
            } else {
                Ok(ByteString::new(char_vec.iter().map(|&c| c as u8).collect()))
            }
        }
    }
}

impl ToJSValConvertible for Reflector {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        let obj = self.get_jsobject();
        assert!(!obj.is_null());
        let mut value = ObjectValue(unsafe { &*obj });
        if unsafe { JS_WrapValue(cx, &mut value) } == 0 {
            panic!("JS_WrapValue failed.");
        }
        value
    }
}

/// Returns whether the given `clasp` is one for a DOM object.
pub fn is_dom_class(clasp: *const JSClass) -> bool {
    unsafe {
        ((*clasp).flags & js::JSCLASS_IS_DOMJSCLASS) != 0
    }
}

/// Returns whether `obj` is a DOM object implemented as a proxy.
pub fn is_dom_proxy(obj: *mut JSObject) -> bool {
    use js::glue::{js_IsObjectProxyClass, js_IsFunctionProxyClass, IsProxyHandlerFamily};

    unsafe {
        (js_IsObjectProxyClass(obj) || js_IsFunctionProxyClass(obj)) &&
            IsProxyHandlerFamily(obj)
    }
}

/// The index of the slot wherein a pointer to the reflected DOM object is
/// stored for non-proxy bindings.
// We use slot 0 for holding the raw object.  This is safe for both
// globals and non-globals.
pub const DOM_OBJECT_SLOT: uint = 0;
const DOM_PROXY_OBJECT_SLOT: uint = js::JSSLOT_PROXY_PRIVATE as uint;

/// Returns the index of the slot wherein a pointer to the reflected DOM object
/// is stored.
///
/// Fails if `obj` is not a DOM object.
pub unsafe fn dom_object_slot(obj: *mut JSObject) -> u32 {
    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        DOM_OBJECT_SLOT as u32
    } else {
        assert!(is_dom_proxy(obj));
        DOM_PROXY_OBJECT_SLOT as u32
    }
}

/// Get the DOM object from the given reflector.
pub unsafe fn unwrap<T>(obj: *mut JSObject) -> *const T {
    use js::jsapi::JS_GetReservedSlot;

    let slot = dom_object_slot(obj);
    let value = JS_GetReservedSlot(obj, slot);
    value.to_private() as *const T
}

/// Get the `DOMClass` from `obj`, or `Err(())` if `obj` is not a DOM object.
unsafe fn get_dom_class(obj: *mut JSObject) -> Result<DOMClass, ()> {
    use dom::bindings::utils::DOMJSClass;
    use js::glue::GetProxyHandlerExtra;

    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        debug!("plain old dom object");
        let domjsclass: *const DOMJSClass = clasp as *const DOMJSClass;
        return Ok((*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        debug!("proxy dom object");
        let dom_class: *const DOMClass = GetProxyHandlerExtra(obj) as *const DOMClass;
        return Ok(*dom_class);
    }
    debug!("not a dom object");
    return Err(());
}

/// Get an `Unrooted<T>` for the given DOM object, unwrapping any wrapper
/// around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub fn unwrap_jsmanaged<T>(mut obj: *mut JSObject) -> Result<Unrooted<T>, ()>
    where T: Reflectable + IDLInterface
{
    use js::glue::{IsWrapper, UnwrapObject};
    use std::ptr;

    unsafe {
        let dom_class = try!(get_dom_class(obj).or_else(|_| {
            if IsWrapper(obj) == 1 {
                debug!("found wrapper");
                obj = UnwrapObject(obj, /* stopAtOuter = */ 0, ptr::null_mut());
                if obj.is_null() {
                    debug!("unwrapping security wrapper failed");
                    Err(())
                } else {
                    assert!(IsWrapper(obj) == 0);
                    debug!("unwrapped successfully");
                    get_dom_class(obj)
                }
            } else {
                debug!("not a dom wrapper");
                Err(())
            }
        }));

        let proto_id = IDLInterface::get_prototype_id(None::<T>);
        let proto_depth = IDLInterface::get_prototype_depth(None::<T>);
        if dom_class.interface_chain[proto_depth] == proto_id {
            debug!("good prototype");
            Ok(Unrooted::from_raw(unwrap(obj)))
        } else {
            debug!("bad prototype");
            Err(())
        }
    }
}

impl<T: Reflectable+IDLInterface> FromJSValConvertible for Unrooted<T> {
    type Config = ();
    fn from_jsval(_cx: *mut JSContext, value: JSVal, _option: ())
                  -> Result<Unrooted<T>, ()> {
        if !value.is_object() {
            return Err(());
        }
        unwrap_jsmanaged(value.to_object())
    }
}

impl<T: Reflectable> ToJSValConvertible for Root<T> {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.r().reflector().to_jsval(cx)
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

impl<'a, T: Reflectable> ToJSValConvertible for Unrooted<T> {
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

impl<X: default::Default, T: FromJSValConvertible<Config=X>> FromJSValConvertible for Option<T> {
    type Config = ();
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
