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

use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::str::{ByteString, USVString};
use dom::bindings::utils::DOMClass;
use js;
pub use js::conversions::{FromJSValConvertible, ToJSValConvertible, ConversionBehavior};
use js::error::throw_type_error;
use js::glue::{GetProxyPrivate, IsWrapper};
use js::glue::{RUST_JSID_IS_STRING, RUST_JSID_TO_STRING, UnwrapObject};
use js::jsapi::{HandleId, HandleObject, HandleValue, JS_GetClass};
use js::jsapi::{JSClass, JSContext, JSObject, MutableHandleValue};
use js::jsapi::{JS_GetLatin1StringCharsAndLength, JS_GetReservedSlot};
use js::jsapi::{JS_GetObjectAsArrayBufferView, JS_GetArrayBufferViewType};
use js::jsapi::{JS_GetTwoByteStringCharsAndLength, JS_IsArrayObject, JS_NewStringCopyN};
use js::jsapi::{JS_StringHasLatin1Chars, JS_WrapValue};
use js::jsapi::{Type};
use js::jsval::{ObjectValue, StringValue};
use js::rust::ToString;
use libc;
use num::Float;
use std::{ptr, mem, slice};
pub use util::non_geckolib::{StringificationBehavior, jsstring_to_str};
use util::str::DOMString;

/// A trait to check whether a given `JSObject` implements an IDL interface.
pub trait IDLInterface {
    /// Returns whether the given DOM class derives that interface.
    fn derives(&'static DOMClass) -> bool;
}

/// A trait to mark an IDL interface as deriving from another one.
#[rustc_on_unimplemented = "The IDL interface `{Self}` is not derived from `{T}`."]
pub trait DerivedFrom<T: Castable>: Castable {}

impl<T: Float + ToJSValConvertible> ToJSValConvertible for Finite<T> {
    #[inline]
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let value = **self;
        value.to_jsval(cx, rval);
    }
}

impl<T: Float + FromJSValConvertible<Config=()>> FromJSValConvertible for Finite<T> {
    type Config = ();

    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         option: ())
                         -> Result<Finite<T>, ()> {
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

impl <T: Reflectable + IDLInterface> FromJSValConvertible for Root<T> {
    type Config = ();

    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         _config: Self::Config)
                         -> Result<Root<T>, ()> {
        let result = root_from_handlevalue(value);
        if let Err(()) = result {
            throw_type_error(cx, "value is not an object");
        }
        result
    }
}

/// Convert the given `jsid` to a `DOMString`. Fails if the `jsid` is not a
/// string, or if the string does not contain valid UTF-16.
pub fn jsid_to_str(cx: *mut JSContext, id: HandleId) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id));
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

// http://heycam.github.io/webidl/#es-USVString
impl ToJSValConvertible for USVString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.0.to_jsval(cx, rval);
    }
}

// http://heycam.github.io/webidl/#es-USVString
impl FromJSValConvertible for USVString {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, value: HandleValue, _: ()) -> Result<USVString, ()> {
        let jsstr = ToString(cx, value);
        if jsstr.is_null() {
            debug!("ToString failed");
            return Err(());
        }
        let latin1 = JS_StringHasLatin1Chars(jsstr);
        if latin1 {
            // FIXME(ajeffrey): Convert directly from DOMString to USVString
            return Ok(USVString(String::from(jsstring_to_str(cx, jsstr))));
        }
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), jsstr, &mut length);
        assert!(!chars.is_null());
        let char_vec = slice::from_raw_parts(chars as *const u16, length as usize);
        Ok(USVString(String::from_utf16_lossy(char_vec)))
    }
}

// http://heycam.github.io/webidl/#es-ByteString
impl ToJSValConvertible for ByteString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let jsstr = JS_NewStringCopyN(cx,
                                      self.as_ptr() as *const libc::c_char,
                                      self.len() as libc::size_t);
        if jsstr.is_null() {
            panic!("JS_NewStringCopyN failed");
        }
        rval.set(StringValue(&*jsstr));
    }
}

// http://heycam.github.io/webidl/#es-ByteString
impl FromJSValConvertible for ByteString {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         _option: ())
                         -> Result<ByteString, ()> {
        let string = ToString(cx, value);
        if string.is_null() {
            debug!("ToString failed");
            return Err(());
        }

        let latin1 = JS_StringHasLatin1Chars(string);
        if latin1 {
            let mut length = 0;
            let chars = JS_GetLatin1StringCharsAndLength(cx, ptr::null(), string, &mut length);
            assert!(!chars.is_null());

            let char_slice = slice::from_raw_parts(chars as *mut u8, length as usize);
            return Ok(ByteString::new(char_slice.to_vec()));
        }

        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), string, &mut length);
        let char_vec = slice::from_raw_parts(chars, length as usize);

        if char_vec.iter().any(|&c| c > 0xFF) {
            throw_type_error(cx, "Invalid ByteString");
            Err(())
        } else {
            Ok(ByteString::new(char_vec.iter().map(|&c| c as u8).collect()))
        }
    }
}


impl ToJSValConvertible for Reflector {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let obj = self.get_jsobject().get();
        assert!(!obj.is_null());
        rval.set(ObjectValue(&*obj));
        if !JS_WrapValue(cx, rval) {
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

/// Get the private pointer of a DOM object from a given reflector.
pub unsafe fn private_from_object(obj: *mut JSObject) -> *const libc::c_void {
    let clasp = JS_GetClass(obj);
    let value = if is_dom_class(clasp) {
        JS_GetReservedSlot(obj, DOM_OBJECT_SLOT)
    } else {
        debug_assert!(is_dom_proxy(obj));
        GetProxyPrivate(obj)
    };
    if value.is_undefined() {
        ptr::null()
    } else {
        value.to_private()
    }
}

/// Get the `DOMClass` from `obj`, or `Err(())` if `obj` is not a DOM object.
pub unsafe fn get_dom_class(obj: *mut JSObject) -> Result<&'static DOMClass, ()> {
    use dom::bindings::utils::DOMJSClass;
    use js::glue::GetProxyHandlerExtra;

    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        debug!("plain old dom object");
        let domjsclass: *const DOMJSClass = clasp as *const DOMJSClass;
        return Ok(&(&*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        debug!("proxy dom object");
        let dom_class: *const DOMClass = GetProxyHandlerExtra(obj) as *const DOMClass;
        return Ok(&*dom_class);
    }
    debug!("not a dom object");
    Err(())
}

/// Get a `*const libc::c_void` for the given DOM object, unwrapping any
/// wrapper around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not an object for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
#[inline]
pub unsafe fn private_from_proto_check<F>(mut obj: *mut JSObject,
                                          proto_check: F)
                                          -> Result<*const libc::c_void, ()>
    where F: Fn(&'static DOMClass) -> bool
{
    let dom_class = try!(get_dom_class(obj).or_else(|_| {
        if IsWrapper(obj) {
            debug!("found wrapper");
            obj = UnwrapObject(obj, /* stopAtWindowProxy = */ 0);
            if obj.is_null() {
                debug!("unwrapping security wrapper failed");
                Err(())
            } else {
                assert!(!IsWrapper(obj));
                debug!("unwrapped successfully");
                get_dom_class(obj)
            }
        } else {
            debug!("not a dom wrapper");
            Err(())
        }
    }));

    if proto_check(dom_class) {
        debug!("good prototype");
        Ok(private_from_object(obj))
    } else {
        debug!("bad prototype");
        Err(())
    }
}

/// Get a `*const T` for a DOM object accessible from a `JSObject`.
pub fn native_from_object<T>(obj: *mut JSObject) -> Result<*const T, ()>
    where T: Reflectable + IDLInterface
{
    unsafe {
        private_from_proto_check(obj, T::derives).map(|ptr| ptr as *const T)
    }
}

/// Get a `Root<T>` for the given DOM object, unwrapping any wrapper
/// around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub fn root_from_object<T>(obj: *mut JSObject) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    native_from_object(obj).map(|ptr| unsafe { Root::from_ref(&*ptr) })
}

/// Get a `*const T` for a DOM object accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub fn native_from_handlevalue<T>(v: HandleValue) -> Result<*const T, ()>
    where T: Reflectable + IDLInterface
{
    if !v.get().is_object() {
        return Err(());
    }
    native_from_object(v.get().to_object())
}

/// Get a `Root<T>` for a DOM object accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub fn root_from_handlevalue<T>(v: HandleValue) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    if !v.get().is_object() {
        return Err(());
    }
    root_from_object(v.get().to_object())
}

/// Get a `Root<T>` for a DOM object accessible from a `HandleObject`.
pub fn root_from_handleobject<T>(obj: HandleObject) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    root_from_object(obj.get())
}

impl<T: Reflectable> ToJSValConvertible for Root<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.reflector().to_jsval(cx, rval);
    }
}

/// A JS ArrayBufferView contents can only be viewed as the types marked with this trait
pub unsafe trait ArrayBufferViewContents: Clone {
    /// Check if the JS ArrayBufferView type is compatible with the implementor of the
    /// trait
    fn is_type_compatible(ty: Type) -> bool;
}

unsafe impl ArrayBufferViewContents for u8 {
    fn is_type_compatible(ty: Type) -> bool {
        match ty {
            Type::Uint8 |
            Type::Uint8Clamped => true,
            _ => false,
        }
    }
}

unsafe impl ArrayBufferViewContents for i8 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int8 as i32
    }
}

unsafe impl ArrayBufferViewContents for u16 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Uint16 as i32
    }
}

unsafe impl ArrayBufferViewContents for i16 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int16 as i32
    }
}

unsafe impl ArrayBufferViewContents for u32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Uint32 as i32
    }
}

unsafe impl ArrayBufferViewContents for i32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int32 as i32
    }
}

unsafe impl ArrayBufferViewContents for f32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Float32 as i32
    }
}
unsafe impl ArrayBufferViewContents for f64 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Float64 as i32
    }
}

/// Returns a mutable slice of the Array Buffer View data, viewed as T, without checking the real
/// type of it.
pub unsafe fn array_buffer_view_data<'a, T: ArrayBufferViewContents>(abv: *mut JSObject) -> Option<&'a mut [T]> {
    let mut byte_length = 0;
    let mut ptr = ptr::null_mut();
    let ret = JS_GetObjectAsArrayBufferView(abv, &mut byte_length, &mut false, &mut ptr);
    if ret.is_null() {
        return None;
    }
    Some(slice::from_raw_parts_mut(ptr as *mut T, byte_length as usize / mem::size_of::<T>()))
}

/// Returns a copy of the ArrayBufferView data, viewed as T, without checking the real type of it.
pub fn array_buffer_view_to_vec<T: ArrayBufferViewContents>(abv: *mut JSObject) -> Option<Vec<T>> {
    unsafe {
        array_buffer_view_data(abv).map(|data| data.to_vec())
    }
}

/// Returns a mutable slice of the Array Buffer View data, viewed as T, checking that the real type
/// of it is ty.
pub unsafe fn array_buffer_view_data_checked<'a, T: ArrayBufferViewContents>(abv: *mut JSObject)
                                                                             -> Option<&'a mut [T]> {
    array_buffer_view_data::<T>(abv).and_then(|data| {
        if T::is_type_compatible(JS_GetArrayBufferViewType(abv)) {
            Some(data)
        } else {
            None
        }
    })
}

/// Returns a copy of the ArrayBufferView data, viewed as T, checking that the real type
/// of it is ty.
pub fn array_buffer_view_to_vec_checked<T: ArrayBufferViewContents>(abv: *mut JSObject) -> Option<Vec<T>> {
    unsafe {
        array_buffer_view_data_checked(abv).map(|data| data.to_vec())
    }
}

/// Returns whether `value` is an array-like object.
/// Note: Currently only Arrays are supported.
/// TODO: Expand this to support sequences and other array-like objects
pub unsafe fn is_array_like(cx: *mut JSContext, value: HandleValue) -> bool {
    let mut result = false;
    assert!(JS_IsArrayObject(cx, value, &mut result));
    result
}
