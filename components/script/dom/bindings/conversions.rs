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
//! | unrestricted float      | `f32`                            |
//! | float                   | `Finite<f32>`                    |
//! | unrestricted double     | `f64`                            |
//! | double                  | `Finite<f64>`                    |
//! | DOMString               | `DOMString`                      |
//! | USVString               | `USVString`                      |
//! | ByteString              | `ByteString`                     |
//! | object                  | `*mut JSObject`                  |
//! | interface types         | `&T`            | `Root<T>`      |
//! | dictionary types        | `&T`            | *unsupported*  |
//! | enumeration types       | `T`                              |
//! | callback function types | `Rc<T>`                          |
//! | nullable types          | `Option<T>`                      |
//! | sequences               | `Vec<T>`                         |
//! | union types             | `T`                              |

use dom::bindings::codegen::PrototypeList;
use dom::bindings::error::throw_type_error;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::str::{ByteString, USVString};
use dom::bindings::utils::{Reflectable, Reflector, DOMClass};
use util::str::DOMString;

use js;
use js::glue::{RUST_JSID_TO_STRING, RUST_JSID_IS_STRING};
use js::glue::RUST_JS_NumberValue;
use js::rust::{ToUint64, ToInt64};
use js::rust::{ToUint32, ToInt32};
use js::rust::{ToUint16, ToNumber, ToBoolean, ToString};
use js::jsapi::{JSContext, JSObject, JSString};
use js::jsapi::{JS_StringHasLatin1Chars, JS_GetLatin1StringCharsAndLength, JS_GetTwoByteStringCharsAndLength};
use js::jsapi::{JS_NewUCStringCopyN, JS_NewStringCopyN};
use js::jsapi::{JS_WrapValue};
use js::jsapi::{JSClass, JS_GetClass};
use js::jsapi::{HandleId, RootedValue, HandleValue, HandleObject, MutableHandleValue};
use js::jsval::JSVal;
use js::jsval::{UndefinedValue, NullValue, BooleanValue, Int32Value, UInt32Value};
use js::jsval::{StringValue, ObjectValue, ObjectOrNullValue};

use libc;
use num::Float;
use std::borrow::ToOwned;
use std::default;
use std::slice;
use std::ptr;
use std::rc::Rc;
use core::nonzero::NonZero;

/// A trait to retrieve the constants necessary to check if a `JSObject`
/// implements a given interface.
pub trait IDLInterface {
    /// Returns the prototype ID.
    fn get_prototype_id() -> PrototypeList::ID;
    /// Returns the prototype depth, i.e., the number of interfaces this
    /// interface inherits from.
    fn get_prototype_depth() -> usize;
}

/// A trait to convert Rust types to `JSVal`s.
pub trait ToJSValConvertible {
    /// Convert `self` to a `JSVal`. JSAPI failure causes a task failure.
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue);
}

/// A trait to convert `JSVal`s to Rust types.
pub trait FromJSValConvertible {
    /// Optional configurable behaviour switch; use () for no configuration.
    type Config;
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: Self::Config) -> Result<Self, ()>;
}


impl ToJSValConvertible for () {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(UndefinedValue());
    }
}

impl ToJSValConvertible for JSVal {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        if unsafe { JS_WrapValue(cx, rval) } == 0 {
            panic!("JS_WrapValue failed.");
        }
    }
}

impl ToJSValConvertible for HandleValue {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        if unsafe { JS_WrapValue(cx, rval) } == 0 {
            panic!("JS_WrapValue failed.");
        }
    }
}

impl ToJSValConvertible for bool {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(BooleanValue(*self));
    }
}

impl FromJSValConvertible for bool {
    type Config = ();
    fn from_jsval(_cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<bool, ()> {
        Ok(ToBoolean(val))
    }
}

impl ToJSValConvertible for i8 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for i8 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<i8, ()> {
        let result = ToInt32(cx, val);
        result.map(|v| v as i8)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for u8 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<u8, ()> {
        let result = ToInt32(cx, val);
        result.map(|v| v as u8)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for i16 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<i16, ()> {
        let result = ToInt32(cx, val);
        result.map(|v| v as i16)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for u16 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<u16, ()> {
        ToUint16(cx, val)
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(Int32Value(*self));
    }
}

impl FromJSValConvertible for i32 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<i32, ()> {
        ToInt32(cx, val)
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(UInt32Value(*self));
    }
}

impl FromJSValConvertible for u32 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<u32, ()> {
        ToUint32(cx, val)
    }
}

impl ToJSValConvertible for i64 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self as f64));
        }
    }
}

impl FromJSValConvertible for i64 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<i64, ()> {
        ToInt64(cx, val)
    }
}

impl ToJSValConvertible for u64 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self as f64));
        }
    }
}

impl FromJSValConvertible for u64 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<u64, ()> {
        ToUint64(cx, val)
    }
}

impl ToJSValConvertible for f32 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self as f64));
        }
    }
}

impl FromJSValConvertible for f32 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<f32, ()> {
        let result = ToNumber(cx, val);
        result.map(|f| f as f32)
    }
}

impl ToJSValConvertible for f64 {
    fn to_jsval(&self, _cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self));
        }
    }
}

impl FromJSValConvertible for f64 {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, val: HandleValue, _option: ()) -> Result<f64, ()> {
        ToNumber(cx, val)
    }
}

impl<T: Float + ToJSValConvertible> ToJSValConvertible for Finite<T> {
    #[inline]
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        let value = **self;
        value.to_jsval(cx, rval);
    }
}

impl<T: Float + FromJSValConvertible<Config=()>> FromJSValConvertible for Finite<T> {
    type Config = ();

    fn from_jsval(cx: *mut JSContext, value: HandleValue, option: ()) -> Result<Finite<T>, ()> {
        let result = try!(FromJSValConvertible::from_jsval(cx, value, option));
        match Finite::new(result) {
            Some(v) => Ok(v),
            None => {
                throw_type_error(cx, "this argument is not a finite floating-point value");
                Err(())
            },
        }
    }
}

impl ToJSValConvertible for str {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            let string_utf16: Vec<u16> = self.utf16_units().collect();
            let jsstr = JS_NewUCStringCopyN(cx, string_utf16.as_ptr() as *const i16,
                                            string_utf16.len() as libc::size_t);
            if jsstr.is_null() {
                panic!("JS_NewUCStringCopyN failed");
            }
            rval.set(StringValue(&*jsstr));
        }
    }
}

impl ToJSValConvertible for DOMString {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval);
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
    let mut length = 0;
    let latin1 = unsafe { JS_StringHasLatin1Chars(s) != 0 };
    if latin1 {
        let chars = unsafe {
            JS_GetLatin1StringCharsAndLength(cx, ptr::null(), s, &mut length)
        };
        assert!(!chars.is_null());

        let mut buf = String::with_capacity(length as usize);
        let mut i = 0;
        for i in 0..(length as isize) {
            unsafe {
                buf.push(*chars.offset(i) as char);
            }
        }
        buf
    } else {
        let chars = unsafe {
            JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), s, &mut length)
        };
        assert!(!chars.is_null());
        let char_vec = unsafe {
            slice::from_raw_parts(chars as *const u16, length as usize)
        };
        String::from_utf16(char_vec).unwrap()
    }
}

/// Convert the given `jsid` to a `DOMString`. Fails if the `jsid` is not a
/// string, or if the string does not contain valid UTF-16.
pub fn jsid_to_str(cx: *mut JSContext, id: HandleId) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id) != 0);
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

impl FromJSValConvertible for DOMString {
    type Config = StringificationBehavior;
    fn from_jsval(cx: *mut JSContext, value: HandleValue,
                  null_behavior: StringificationBehavior)
                  -> Result<DOMString, ()> {
        if null_behavior == StringificationBehavior::Empty &&
           value.get().is_null() {
            Ok("".to_owned())
        } else {
            let jsstr = ToString(cx, value);
            if jsstr.is_null() {
                debug!("ToString failed");
                Err(())
            } else {
                Ok(jsstring_to_str(cx, jsstr))
            }
        }
    }
}

impl ToJSValConvertible for USVString {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        self.0.to_jsval(cx, rval);
    }
}

impl FromJSValConvertible for USVString {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, value: HandleValue, _: ())
                  -> Result<USVString, ()> {
        let jsstr = ToString(cx, value);
        if jsstr.is_null() {
            debug!("ToString failed");
            return Err(());
        }
        let latin1 = unsafe { JS_StringHasLatin1Chars(jsstr) != 0 };
        if latin1 {
            return Ok(USVString(jsstring_to_str(cx, jsstr)));
        }
        unsafe {
            let mut length = 0;
            let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), jsstr, &mut length);
            assert!(!chars.is_null());
            let char_vec = slice::from_raw_parts(chars as *const u16, length as usize);
            Ok(USVString(String::from_utf16_lossy(char_vec)))
        }
    }
}

impl ToJSValConvertible for ByteString {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        unsafe {
            let jsstr = JS_NewStringCopyN(cx, self.as_ptr() as *const libc::c_char,
                                          self.len() as libc::size_t);
            if jsstr.is_null() {
                panic!("JS_NewStringCopyN failed");
            }
            rval.set(StringValue(&*jsstr));
        }
    }
}

impl FromJSValConvertible for ByteString {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, value: HandleValue, _option: ()) -> Result<ByteString, ()> {
        let string = ToString(cx, value);
        if string.is_null() {
            debug!("ToString failed");
            return Err(());
        }

        let latin1 = unsafe { JS_StringHasLatin1Chars(string) != 0 };
        if latin1 {
            let mut length = 0;
            let chars = unsafe {
                JS_GetLatin1StringCharsAndLength(cx, ptr::null(),
                                                 string, &mut length)
            };
            assert!(!chars.is_null());

            let char_vec = unsafe {
                Vec::from_raw_buf(chars as *mut u8, length as usize)
            };

            return Ok(ByteString::new(char_vec));
        }

        unsafe {
            let mut length = 0;
            let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), string, &mut length);
            let char_vec = slice::from_raw_parts(chars, length as usize);

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
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        let obj = self.get_jsobject();
        assert!(!obj.is_null());
        rval.set(ObjectValue(unsafe { &*obj }));
        if unsafe { JS_WrapValue(cx, rval) } == 0 {
            panic!("JS_WrapValue failed.");
        }
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
    use js::glue::IsProxyHandlerFamily;
    unsafe {
        let clasp = JS_GetClass(obj);
        ((*clasp).flags & js::JSCLASS_IS_PROXY) != 0 && IsProxyHandlerFamily(obj) != 0
    }
}

/// The index of the slot wherein a pointer to the reflected DOM object is
/// stored for non-proxy bindings.
// We use slot 0 for holding the raw object.  This is safe for both
// globals and non-globals.
pub const DOM_OBJECT_SLOT: u32 = 0;

/// Get the DOM object from the given reflector.
pub unsafe fn native_from_reflector<T>(obj: *mut JSObject) -> *const T {
    use js::jsapi::JS_GetReservedSlot;
    use js::glue::GetProxyPrivate;

    let clasp = JS_GetClass(obj);
    let value = if is_dom_class(clasp) {
        JS_GetReservedSlot(obj, DOM_OBJECT_SLOT)
    } else {
        assert!(is_dom_proxy(obj));
        GetProxyPrivate(obj)
    };
    if value.is_undefined() {
        ptr::null()
    } else {
        value.to_private() as *const T
    }
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
pub fn native_from_reflector_jsmanaged<T>(mut obj: *mut JSObject) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    use js::glue::{IsWrapper, UnwrapObject};

    unsafe {
        let dom_class = try!(get_dom_class(obj).or_else(|_| {
            if IsWrapper(obj) == 1 {
                debug!("found wrapper");
                obj = UnwrapObject(obj, /* stopAtOuter = */ 0);
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

        let proto_id = <T as IDLInterface>::get_prototype_id();
        let proto_depth = <T as IDLInterface>::get_prototype_depth();
        if dom_class.interface_chain[proto_depth] == proto_id {
            debug!("good prototype");
            let native = native_from_reflector(obj);
            assert!(!native.is_null());
            Ok(Root::new(NonZero::new(native)))
        } else {
            debug!("bad prototype");
            Err(())
        }
    }
}

/// Get a Rooted<T> for a DOM object accessible from a HandleValue
pub fn native_from_handlevalue<T>(v: HandleValue) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    native_from_reflector_jsmanaged(v.get().to_object())
}

/// Get a Rooted<T> for a DOM object accessible from a HandleObject
pub fn native_from_handleobject<T>(obj: HandleObject) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    native_from_reflector_jsmanaged(obj.get())
}

impl<T: Reflectable> ToJSValConvertible for Root<T> {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        self.r().reflector().to_jsval(cx, rval);
    }
}

impl<'a, T: Reflectable> ToJSValConvertible for &'a T {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        self.reflector().to_jsval(cx, rval);
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        match self {
            &Some(ref value) => value.to_jsval(cx, rval),
            &None => rval.set(NullValue()),
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<Rc<T>> {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        match self {
            &Some(ref value) => (**value).to_jsval(cx, rval),
            &None => rval.set(NullValue()),
        }
    }
}

impl<X: default::Default, T: FromJSValConvertible<Config=X>> FromJSValConvertible for Option<T> {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, value: HandleValue, _: ()) -> Result<Option<T>, ()> {
        if value.get().is_null_or_undefined() {
            Ok(None)
        } else {
            let option: X = default::Default::default();
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value, option);
            result.map(Some)
        }
    }
}

impl ToJSValConvertible for *mut JSObject {
    fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        rval.set(ObjectOrNullValue(*self));
        unsafe {
            assert!(JS_WrapValue(cx, rval) != 0);
        }
    }
}
