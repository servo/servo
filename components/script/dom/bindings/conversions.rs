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
use js::glue::{GetProxyPrivate, IsWrapper, RUST_JS_NumberValue};
use js::glue::{RUST_JSID_IS_STRING, RUST_JSID_TO_STRING, UnwrapObject};
use js::jsapi::{HandleId, HandleObject, HandleValue, JS_GetClass};
use js::jsapi::{JSClass, JSContext, JSObject, JSString, MutableHandleValue};
use js::jsapi::{JS_GetLatin1StringCharsAndLength, JS_GetReservedSlot};
use js::jsapi::{JS_GetTwoByteStringCharsAndLength, JS_NewStringCopyN};
use js::jsapi::{JS_NewUCStringCopyN, JS_StringHasLatin1Chars, JS_WrapValue};
use js::jsval::JSVal;
use js::jsval::{StringValue, ObjectValue, ObjectOrNullValue};
use js::jsval::{UndefinedValue, NullValue, BooleanValue, Int32Value, UInt32Value};
use js::rust::{ToUint16, ToNumber, ToBoolean, ToString};
use js::rust::{ToUint32, ToInt32};
use js::rust::{ToUint64, ToInt64};

use core::nonzero::NonZero;
use libc;
use num::Float;
use num::traits::{Bounded, Zero};
use std::borrow::ToOwned;
use std::char;
use std::ptr;
use std::rc::Rc;
use std::slice;

trait As<O>: Copy {
    fn cast(self) -> O;
}

macro_rules! impl_as {
    ($I:ty, $O:ty) => (
        impl As<$O> for $I {
            fn cast(self) -> $O {
                self as $O
            }
        }
    )
}

impl_as!(f64, u8);
impl_as!(f64, u16);
impl_as!(f64, u32);
impl_as!(f64, u64);
impl_as!(f64, i8);
impl_as!(f64, i16);
impl_as!(f64, i32);
impl_as!(f64, i64);

impl_as!(u8, f64);
impl_as!(u16, f64);
impl_as!(u32, f64);
impl_as!(u64, f64);
impl_as!(i8, f64);
impl_as!(i16, f64);
impl_as!(i32, f64);
impl_as!(i64, f64);

impl_as!(i32, i8);
impl_as!(i32, u8);
impl_as!(i32, i16);
impl_as!(u16, u16);
impl_as!(i32, i32);
impl_as!(u32, u32);
impl_as!(i64, i64);
impl_as!(u64, u64);

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
pub trait FromJSValConvertible: Sized {
    /// Optional configurable behaviour switch; use () for no configuration.
    type Config;
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: Self::Config) -> Result<Self, ()>;
}

/// Behavior for converting out-of-range integers.
#[derive(PartialEq, Eq)]
pub enum ConversionBehavior {
    /// Wrap into the integer's range.
    Default,
    /// Throw an exception.
    EnforceRange,
    /// Clamp into the integer's range.
    Clamp
}

/// Try to cast the number to a smaller type, but
/// if it doesn't fit, it will return an error.
fn enforce_range<D>(cx: *mut JSContext, d: f64) -> Result<D, ()>
    where D: Bounded + As<f64>,
          f64: As<D>
{
    if d.is_infinite() {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        return Err(());
    }

    let rounded = d.round();
    if D::min_value().cast() <= rounded && rounded <= D::max_value().cast() {
        Ok(rounded.cast())
    } else {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        Err(())
    }
}

/// Try to cast the number to a smaller type, but if it doesn't fit,
/// round it to the MAX or MIN of the source type before casting it to
/// the destination type.
fn clamp_to<D>(d: f64) -> D
    where D: Bounded + As<f64> + Zero,
          f64: As<D>
{
    if d.is_nan() {
        D::zero()
    } else if d > D::max_value().cast() {
        D::max_value()
    } else if d < D::min_value().cast() {
        D::min_value()
    } else {
        d.cast()
    }
}

impl ToJSValConvertible for () {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(UndefinedValue());
    }
}

impl ToJSValConvertible for JSVal {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(*self);
        if unsafe { JS_WrapValue(cx, rval) } == 0 {
            panic!("JS_WrapValue failed.");
        }
    }
}

impl ToJSValConvertible for HandleValue {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(self.get());
        if unsafe { JS_WrapValue(cx, rval) } == 0 {
            panic!("JS_WrapValue failed.");
        }
    }
}

#[inline]
fn convert_int_from_jsval<T, M>(cx: *mut JSContext, value: HandleValue,
                                option: ConversionBehavior,
                                convert_fn: fn(*mut JSContext, HandleValue) -> Result<M, ()>)
                                -> Result<T, ()>
    where T: Bounded + Zero + As<f64>,
          M: Zero + As<T>,
          f64: As<T>
{
    match option {
        ConversionBehavior::Default => Ok(try!(convert_fn(cx, value)).cast()),
        ConversionBehavior::EnforceRange => enforce_range(cx, try!(ToNumber(cx, value))),
        ConversionBehavior::Clamp => Ok(clamp_to(try!(ToNumber(cx, value)))),
    }
}

impl ToJSValConvertible for bool {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for i8 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<i8, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for u8 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<u8, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for i16 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<i16, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self as i32));
    }
}

impl FromJSValConvertible for u16 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<u16, ()> {
        convert_int_from_jsval(cx, val, option, ToUint16)
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(Int32Value(*self));
    }
}

impl FromJSValConvertible for i32 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<i32, ()> {
        convert_int_from_jsval(cx, val, option, ToInt32)
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(UInt32Value(*self));
    }
}

impl FromJSValConvertible for u32 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<u32, ()> {
        convert_int_from_jsval(cx, val, option, ToUint32)
    }
}

impl ToJSValConvertible for i64 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self as f64));
        }
    }
}

impl FromJSValConvertible for i64 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<i64, ()> {
        convert_int_from_jsval(cx, val, option, ToInt64)
    }
}

impl ToJSValConvertible for u64 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
        unsafe {
            rval.set(RUST_JS_NumberValue(*self as f64));
        }
    }
}

impl FromJSValConvertible for u64 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: HandleValue, option: ConversionBehavior) -> Result<u64, ()> {
        convert_int_from_jsval(cx, val, option, ToUint64)
    }
}

impl ToJSValConvertible for f32 {
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, _cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
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
        let potentially_ill_formed_utf16 = unsafe {
            slice::from_raw_parts(chars as *const u16, length as usize)
        };
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
                    if ::util::opts::get().replace_surrogates {
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
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

            let char_slice = unsafe {
                slice::from_raw_parts(chars as *mut u8, length as usize)
            };

            return Ok(ByteString::new(char_slice.to_vec()));
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let obj = self.get_jsobject().get();
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

/// Get the private pointer of a DOM object from a given reflector.
unsafe fn private_from_reflector(obj: *mut JSObject) -> *const libc::c_void {
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

/// Get the DOM object from the given reflector.
pub unsafe fn native_from_reflector<T>(obj: *mut JSObject) -> *const T {
    private_from_reflector(obj) as *const T
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
    Err(())
}

/// Get a `*const libc::c_void` for the given DOM object, unwrapping any
/// wrapper around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not an object for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub unsafe fn private_from_proto_chain(mut obj: *mut JSObject,
                                       proto_id: u16, proto_depth: u16)
                                       -> Result<*const libc::c_void, ()> {
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

    if dom_class.interface_chain[proto_depth as usize] as u16 == proto_id {
        debug!("good prototype");
        Ok(private_from_reflector(obj))
    } else {
        debug!("bad prototype");
        Err(())
    }
}

/// Get a `Root<T>` for the given DOM object, unwrapping any wrapper
/// around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub fn native_from_reflector_jsmanaged<T>(obj: *mut JSObject) -> Result<Root<T>, ()>
    where T: Reflectable + IDLInterface
{
    let proto_id = <T as IDLInterface>::get_prototype_id() as u16;
    let proto_depth = <T as IDLInterface>::get_prototype_depth() as u16;
    unsafe {
        private_from_proto_chain(obj, proto_id, proto_depth).map(|obj| {
            Root::new(NonZero::new(obj as *const T))
        })
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
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.r().reflector().to_jsval(cx, rval);
    }
}

impl<'a, T: Reflectable> ToJSValConvertible for &'a T {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.reflector().to_jsval(cx, rval);
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<T> {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        match self {
            &Some(ref value) => value.to_jsval(cx, rval),
            &None => rval.set(NullValue()),
        }
    }
}

impl<T: ToJSValConvertible> ToJSValConvertible for Option<Rc<T>> {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        match self {
            &Some(ref value) => (**value).to_jsval(cx, rval),
            &None => rval.set(NullValue()),
        }
    }
}

impl<T: FromJSValConvertible> FromJSValConvertible for Option<T> {
    type Config = T::Config;
    fn from_jsval(cx: *mut JSContext, value: HandleValue, option: T::Config) -> Result<Option<T>, ()> {
        if value.get().is_null_or_undefined() {
            Ok(None)
        } else {
            let result: Result<T, ()> = FromJSValConvertible::from_jsval(cx, value, option);
            result.map(Some)
        }
    }
}

impl ToJSValConvertible for *mut JSObject {
    fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rval.set(ObjectOrNullValue(*self));
        unsafe {
            assert!(JS_WrapValue(cx, rval) != 0);
        }
    }
}
