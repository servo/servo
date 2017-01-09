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
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::str::{ByteString, DOMString, USVString};
use dom::bindings::utils::DOMClass;
use js;
pub use js::conversions::{FromJSValConvertible, ToJSValConvertible, ConversionResult};
pub use js::conversions::ConversionBehavior;
use js::conversions::latin1_to_string;
use js::error::throw_type_error;
use js::glue::{GetProxyPrivate, IsWrapper};
use js::glue::{RUST_JSID_IS_INT, RUST_JSID_TO_INT};
use js::glue::{RUST_JSID_IS_STRING, RUST_JSID_TO_STRING, UnwrapObject};
use js::jsapi::{HandleId, HandleObject, HandleValue, JSContext};
use js::jsapi::{JSObject, JSString, JS_GetArrayBufferViewType};
use js::jsapi::{JS_GetLatin1StringCharsAndLength, JS_GetObjectAsArrayBuffer, JS_GetObjectAsArrayBufferView};
use js::jsapi::{JS_GetReservedSlot, JS_GetTwoByteStringCharsAndLength};
use js::jsapi::{JS_IsArrayObject, JS_NewStringCopyN, JS_StringHasLatin1Chars};
use js::jsapi::{JS_NewFloat32Array, JS_NewFloat64Array};
use js::jsapi::{JS_NewInt8Array, JS_NewInt16Array, JS_NewInt32Array};
use js::jsapi::{JS_NewUint8Array, JS_NewUint16Array, JS_NewUint32Array};
use js::jsapi::{MutableHandleValue, Type};
use js::jsval::{ObjectValue, StringValue};
use js::rust::{ToString, get_object_class, is_dom_class, is_dom_object, maybe_wrap_value};
use libc;
use num_traits::Float;
use servo_config::opts;
use std::{char, mem, ptr, slice};

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
                         -> Result<ConversionResult<Finite<T>>, ()> {
        let result = match FromJSValConvertible::from_jsval(cx, value, option) {
            Ok(ConversionResult::Success(v)) => v,
            Ok(ConversionResult::Failure(error)) => {
                throw_type_error(cx, &error);
                return Err(());
            }
            _ => return Err(()),
        };
        match Finite::new(result) {
            Some(v) => Ok(ConversionResult::Success(v)),
            None => {
                throw_type_error(cx, "this argument is not a finite floating-point value");
                Err(())
            },
        }
    }
}

impl <T: DomObject + IDLInterface> FromJSValConvertible for Root<T> {
    type Config = ();

    unsafe fn from_jsval(_cx: *mut JSContext,
                         value: HandleValue,
                         _config: Self::Config)
                         -> Result<ConversionResult<Root<T>>, ()> {
        Ok(match root_from_handlevalue(value) {
            Ok(result) => ConversionResult::Success(result),
            Err(()) => ConversionResult::Failure("value is not an object".into()),
        })
    }
}

/// Convert `id` to a `DOMString`, assuming it is string-valued.
///
/// Handling of invalid UTF-16 in strings depends on the relevant option.
///
/// # Panics
///
/// Panics if `id` is not string-valued.
pub fn string_jsid_to_string(cx: *mut JSContext, id: HandleId) -> DOMString {
    unsafe {
        assert!(RUST_JSID_IS_STRING(id));
        jsstring_to_str(cx, RUST_JSID_TO_STRING(id))
    }
}

/// Convert `id` to a `DOMString`. Returns `None` if `id` is not a string or
/// integer.
///
/// Handling of invalid UTF-16 in strings depends on the relevant option.
pub unsafe fn jsid_to_string(cx: *mut JSContext, id: HandleId) -> Option<DOMString> {
    if RUST_JSID_IS_STRING(id) {
        return Some(jsstring_to_str(cx, RUST_JSID_TO_STRING(id)));
    }

    if RUST_JSID_IS_INT(id) {
        return Some(RUST_JSID_TO_INT(id).to_string().into());
    }

    None
}

// http://heycam.github.io/webidl/#es-USVString
impl ToJSValConvertible for USVString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.0.to_jsval(cx, rval);
    }
}

/// Behavior for stringification of `JSVal`s.
#[derive(PartialEq, Clone)]
pub enum StringificationBehavior {
    /// Convert `null` to the string `"null"`.
    Default,
    /// Convert `null` to the empty string.
    Empty,
}

// https://heycam.github.io/webidl/#es-DOMString
impl ToJSValConvertible for DOMString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval);
    }
}

// https://heycam.github.io/webidl/#es-DOMString
impl FromJSValConvertible for DOMString {
    type Config = StringificationBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         null_behavior: StringificationBehavior)
                         -> Result<ConversionResult<DOMString>, ()> {
        if null_behavior == StringificationBehavior::Empty &&
           value.get().is_null() {
            Ok(ConversionResult::Success(DOMString::new()))
        } else {
            let jsstr = ToString(cx, value);
            if jsstr.is_null() {
                debug!("ToString failed");
                Err(())
            } else {
                Ok(ConversionResult::Success(jsstring_to_str(cx, jsstr)))
            }
        }
    }
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
pub unsafe fn jsstring_to_str(cx: *mut JSContext, s: *mut JSString) -> DOMString {
    let latin1 = JS_StringHasLatin1Chars(s);
    DOMString::from_string(if latin1 {
        latin1_to_string(cx, s)
    } else {
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), s, &mut length);
        assert!(!chars.is_null());
        let potentially_ill_formed_utf16 = slice::from_raw_parts(chars, length as usize);
        let mut s = String::with_capacity(length as usize);
        for item in char::decode_utf16(potentially_ill_formed_utf16.iter().cloned()) {
            match item {
                Ok(c) => s.push(c),
                Err(_) => {
                    // FIXME: Add more info like document URL in the message?
                    macro_rules! message {
                        () => {
                            "Found an unpaired surrogate in a DOM string. \
                             If you see this in real web content, \
                             please comment on https://github.com/servo/servo/issues/6564"
                        }
                    }
                    if opts::get().replace_surrogates {
                        error!(message!());
                        s.push('\u{FFFD}');
                    } else {
                        panic!(concat!(message!(), " Use `-Z replace-surrogates` \
                            on the command line to make this non-fatal."));
                    }
                }
            }
        }
        s
    })
}

// http://heycam.github.io/webidl/#es-USVString
impl FromJSValConvertible for USVString {
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, value: HandleValue, _: ())
                         -> Result<ConversionResult<USVString>, ()> {
        let jsstr = ToString(cx, value);
        if jsstr.is_null() {
            debug!("ToString failed");
            return Err(());
        }
        let latin1 = JS_StringHasLatin1Chars(jsstr);
        if latin1 {
            // FIXME(ajeffrey): Convert directly from DOMString to USVString
            return Ok(ConversionResult::Success(
                USVString(String::from(jsstring_to_str(cx, jsstr)))));
        }
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), jsstr, &mut length);
        assert!(!chars.is_null());
        let char_vec = slice::from_raw_parts(chars as *const u16, length as usize);
        Ok(ConversionResult::Success(USVString(String::from_utf16_lossy(char_vec))))
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
                         -> Result<ConversionResult<ByteString>, ()> {
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
            return Ok(ConversionResult::Success(ByteString::new(char_slice.to_vec())));
        }

        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), string, &mut length);
        let char_vec = slice::from_raw_parts(chars, length as usize);

        if char_vec.iter().any(|&c| c > 0xFF) {
            throw_type_error(cx, "Invalid ByteString");
            Err(())
        } else {
            Ok(ConversionResult::Success(
                ByteString::new(char_vec.iter().map(|&c| c as u8).collect())))
        }
    }
}


impl ToJSValConvertible for Reflector {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let obj = self.get_jsobject().get();
        assert!(!obj.is_null());
        rval.set(ObjectValue(obj));
        maybe_wrap_value(cx, rval);
    }
}

/// Returns whether `obj` is a DOM object implemented as a proxy.
pub fn is_dom_proxy(obj: *mut JSObject) -> bool {
    use js::glue::IsProxyHandlerFamily;
    unsafe {
        let clasp = get_object_class(obj);
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
    let value = if is_dom_object(obj) {
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

    let clasp = get_object_class(obj);
    if is_dom_class(&*clasp) {
        trace!("plain old dom object");
        let domjsclass: *const DOMJSClass = clasp as *const DOMJSClass;
        return Ok(&(&*domjsclass).dom_class);
    }
    if is_dom_proxy(obj) {
        trace!("proxy dom object");
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
    where T: DomObject + IDLInterface
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
    where T: DomObject + IDLInterface
{
    native_from_object(obj).map(|ptr| unsafe { Root::from_ref(&*ptr) })
}

/// Get a `*const T` for a DOM object accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub fn native_from_handlevalue<T>(v: HandleValue) -> Result<*const T, ()>
    where T: DomObject + IDLInterface
{
    if !v.get().is_object() {
        return Err(());
    }
    native_from_object(v.get().to_object())
}

/// Get a `Root<T>` for a DOM object accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub fn root_from_handlevalue<T>(v: HandleValue) -> Result<Root<T>, ()>
    where T: DomObject + IDLInterface
{
    if !v.get().is_object() {
        return Err(());
    }
    root_from_object(v.get().to_object())
}

/// Get a `Root<T>` for a DOM object accessible from a `HandleObject`.
pub fn root_from_handleobject<T>(obj: HandleObject) -> Result<Root<T>, ()>
    where T: DomObject + IDLInterface
{
    root_from_object(obj.get())
}

impl<T: DomObject> ToJSValConvertible for Root<T> {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.reflector().to_jsval(cx, rval);
    }
}

/// A JS ArrayBufferView contents can only be viewed as the types marked with this trait
pub unsafe trait ArrayBufferViewContents: Clone {
    /// Check if the JS ArrayBufferView type is compatible with the implementor of the
    /// trait
    fn is_type_compatible(ty: Type) -> bool;

    /// Creates a typed array
    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject;
}

unsafe impl ArrayBufferViewContents for u8 {
    fn is_type_compatible(ty: Type) -> bool {
        match ty {
            Type::Uint8 |
            Type::Uint8Clamped => true,
            _ => false,
        }
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewUint8Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for i8 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int8 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewInt8Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for u16 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Uint16 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewUint16Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for i16 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int16 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewInt16Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for u32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Uint32 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewUint32Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for i32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Int32 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewInt32Array(cx, num)
    }
}

unsafe impl ArrayBufferViewContents for f32 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Float32 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewFloat32Array(cx, num)
    }
}
unsafe impl ArrayBufferViewContents for f64 {
    fn is_type_compatible(ty: Type) -> bool {
        ty as i32 == Type::Float64 as i32
    }

    unsafe fn new(cx: *mut JSContext, num: u32) -> *mut JSObject {
        JS_NewFloat64Array(cx, num)
    }
}

/// Returns a mutable slice of the Array Buffer View data, viewed as T, without
/// checking the real type of it.
pub unsafe fn array_buffer_view_data<'a, T>(abv: *mut JSObject) -> Option<&'a mut [T]>
    where T: ArrayBufferViewContents
{
    assert!(!abv.is_null());

    let mut byte_length = 0;
    let mut ptr = ptr::null_mut();
    let mut is_shared = false;
    let ret = JS_GetObjectAsArrayBufferView(abv, &mut byte_length, &mut is_shared, &mut ptr);
    assert!(!is_shared);
    if ret.is_null() {
        return None;
    }
    Some(slice::from_raw_parts_mut(ptr as *mut T, byte_length as usize / mem::size_of::<T>()))
}

/// Returns a copy of the ArrayBufferView data, viewed as T, without checking
/// the real type of it.
pub unsafe fn array_buffer_view_to_vec<T>(abv: *mut JSObject) -> Option<Vec<T>>
    where T: ArrayBufferViewContents
{
    array_buffer_view_data(abv).map(|data| data.to_vec())
}

/// Returns a mutable slice of the Array Buffer View data, viewed as T, checking
/// that the real type of it is ty.
pub unsafe fn array_buffer_view_data_checked<'a, T>(abv: *mut JSObject) -> Option<&'a mut [T]>
    where T: ArrayBufferViewContents
{
    array_buffer_view_data::<T>(abv).and_then(|data| {
        if T::is_type_compatible(JS_GetArrayBufferViewType(abv)) {
            Some(data)
        } else {
            None
        }
    })
}

/// Returns a copy of the ArrayBufferView data, viewed as T, checking that the
/// real type of it is ty.
pub unsafe fn array_buffer_view_to_vec_checked<T>(abv: *mut JSObject) -> Option<Vec<T>>
    where T: ArrayBufferViewContents
{
    array_buffer_view_data_checked(abv).map(|data| data.to_vec())
}

/// Similar API as the array_buffer_view_xxx functions, but for ArrayBuffer
/// objects.
pub unsafe fn array_buffer_data<'a, T>(ab: *mut JSObject) -> Option<&'a mut [T]>
    where T: ArrayBufferViewContents
{
    assert!(!ab.is_null());

    let mut byte_length = 0;
    let mut ptr = ptr::null_mut();
    let ret = JS_GetObjectAsArrayBuffer(ab, &mut byte_length, &mut ptr);
    if ret.is_null() {
        return None;
    }
    Some(slice::from_raw_parts_mut(ptr as *mut T, byte_length as usize / mem::size_of::<T>()))
}

/// Similar API to array_buffer_view_to_vec, but for ArrayBuffer objects.
pub unsafe fn array_buffer_to_vec<T>(ab: *mut JSObject) -> Option<Vec<T>>
    where T: ArrayBufferViewContents
{
    array_buffer_data(ab).map(|data| data.to_vec())
}

/// Returns whether `value` is an array-like object.
/// Note: Currently only Arrays are supported.
/// TODO: Expand this to support sequences and other array-like objects
pub unsafe fn is_array_like(cx: *mut JSContext, value: HandleValue) -> bool {
    let mut result = false;
    assert!(JS_IsArrayObject(cx, value, &mut result));
    result
}

/// Creates a typed JS array from a Rust slice
pub unsafe fn slice_to_array_buffer_view<T>(cx: *mut JSContext, data: &[T]) -> *mut JSObject
    where T: ArrayBufferViewContents
{
    let js_object = T::new(cx, data.len() as u32);
    assert!(!js_object.is_null());
    update_array_buffer_view(js_object, data);
    js_object
}

/// Updates a typed JS array from a Rust slice
pub unsafe fn update_array_buffer_view<T>(obj: *mut JSObject, data: &[T])
    where T: ArrayBufferViewContents
{
    let mut buffer = array_buffer_view_data(obj);
    if let Some(ref mut buffer) = buffer {
        ptr::copy_nonoverlapping(&data[0], &mut buffer[0], data.len())
    }
}
