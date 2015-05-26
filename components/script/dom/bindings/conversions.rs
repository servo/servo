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
//! | interface types         | `JSRef<T>`      | `Temporary<T>` |
//! | dictionary types        | `&T`            | *unsupported*  |
//! | enumeration types       | `T`                              |
//! | callback function types | `T`                              |
//! | nullable types          | `Option<T>`                      |
//! | sequences               | `Vec<T>`                         |
//! | union types             | `T`                              |

use dom::bindings::codegen::PrototypeList;
use dom::bindings::error::throw_type_error;
use dom::bindings::js::{JSRef, Root, Unrooted};
use dom::bindings::num::Finite;
use dom::bindings::str::{ByteString, USVString};
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
use num::Float;
use num::traits::{Bounded, Zero};
use std::borrow::ToOwned;
use std::default;
use std::slice;

/// A trait to reproduce the static float/(u)int->(u)int/float behavior.
trait AsPrimitive {
    fn as_u8(self) -> u8;
    fn as_u16(self) -> u16;
    fn as_u32(self) -> u32;
    fn as_u64(self) -> u64;
    fn as_i8(self) -> i8;
    fn as_i16(self) -> i16;
    fn as_i32(self) -> i32;
    fn as_i64(self) -> i64;
    fn as_f64(self) -> f64;
}

macro_rules! impl_as_primitive {
    ($Src:ty) => {
        impl AsPrimitive for $Src {
            fn as_u8(self) -> u8 { self as u8 }
            fn as_u16(self) -> u16 { self as u16 }
            fn as_u32(self) -> u32 { self as u32 }
            fn as_u64(self) -> u64 { self as u64 }
            fn as_i8(self) -> i8 { self as i8 }
            fn as_i16(self) -> i16 { self as i16 }
            fn as_i32(self) -> i32 { self as i32 }
            fn as_i64(self) -> i64 { self as i64 }
            fn as_f64(self) -> f64 { self as f64 }
        }
    }
}

impl_as_primitive!(u8);
impl_as_primitive!(u16);
impl_as_primitive!(u32);
impl_as_primitive!(u64);
impl_as_primitive!(i8);
impl_as_primitive!(i16);
impl_as_primitive!(i32);
impl_as_primitive!(i64);
impl_as_primitive!(f64);

/// Mimics NumCast but using AsPrimitive instead of ToPrimitive
trait AsStaticCast: AsPrimitive {
    fn as_type<T: AsPrimitive>(n: T) -> Self;
}

macro_rules! impl_as_cast {
    ($T:ty, $conv:ident) => (
        impl AsStaticCast for $T {
            #[inline]
            fn as_type<N: AsPrimitive>(n: N) -> $T {
                n.$conv()
            }
        }
        )
}

impl_as_cast!{u8, as_u8}
impl_as_cast!{u16, as_u16}
impl_as_cast!{u32, as_u32}
impl_as_cast!{u64, as_u64}
impl_as_cast!{i8, as_i8}
impl_as_cast!{i16, as_i16}
impl_as_cast!{i32, as_i32}
impl_as_cast!{i64, as_i64}

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
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal;
}

/// A trait to convert `JSVal`s to Rust types.
pub trait FromJSValConvertible {
    /// Optional configurable behaviour switch; use () for no configuration.
    type Config;
    /// Convert `val` to type `Self`.
    /// Optional configuration of type `T` can be passed as the `option`
    /// argument.
    /// If it returns `Err(())`, a JSAPI exception is pending.
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: Self::Config) -> Result<Self, ()>;
}

/// Behavior for converting the integers
#[derive(PartialEq, Eq)]
pub enum ConversionBehavior {
    /// Do nothing
    Default,
    /// If it doesn't fit in the type, raise error
    EnforceRange,
    /// If it doesn't fit in the type, clamp to MAX or MIN
    Clamp
}

/// Try to cast the number to a smaller type, but
/// if it doesn't fit, it will return an error.
fn enforce_range<D: Bounded + AsStaticCast>(cx: *mut JSContext, d: f64) -> Result<D, ()> {
    if d.is_infinite() {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        return Err(());
    }

    let rounded = d.round();
    if rounded > D::max_value().as_f64() ||
       rounded < D::min_value().as_f64() {
           Ok(AsStaticCast::as_type(rounded))
    } else {
        throw_type_error(cx, "value out of range in an EnforceRange argument");
        Err(())
    }
}

/// Try to cast the number to a smaller type, but if it doesn't fit,
/// round it to the MAX or MIN of the source type before casting it to
/// the destination type.
fn clamp_to<D: Bounded + AsStaticCast + Zero>(d: f64) -> D {
    if d.is_nan() {
        D::zero()
    } else if d > D::max_value().as_f64() {
        D::max_value()
    } else if d < D::min_value().as_f64() {
        D::min_value()
    } else {
        AsStaticCast::as_type(d)
    }
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

#[inline]
fn convert_int_from_jsval<T: Bounded + Zero + AsStaticCast, M: Zero + AsStaticCast>(
    cx: *mut JSContext, value: JSVal,
    option: ConversionBehavior,
    convert_fn: unsafe extern "C" fn(*mut JSContext, JSVal, *mut M) -> JSBool
        ) -> Result<T, ()>
{
    if option == ConversionBehavior::Default {
        let mut ret = M::zero();
        if unsafe {convert_fn(cx, value, &mut ret)} == 0 {
            Err(())
        } else {
            Ok(AsStaticCast::as_type(ret))
        }
    } else {
        let mut ret = 0f64;
        if unsafe {JS_ValueToNumber(cx, value, &mut ret)} == 0 {
            Err(())
        } else {
            match option {
                ConversionBehavior::EnforceRange => enforce_range(cx, ret),
                ConversionBehavior::Clamp => Ok(clamp_to(ret)),
                _ => panic!("unreachable")
            }
        }
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
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<i8, ()> {
        // XXX(reviewer): this would be with the generic
        convert_int_from_jsval(cx, val, option, JS_ValueToECMAInt32)
    }
}

impl ToJSValConvertible for u8 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for u8 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<u8, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToECMAInt32)
    }
}

impl ToJSValConvertible for i16 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for i16 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<i16, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToECMAInt32)
    }
}

impl ToJSValConvertible for u16 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self as i32)
    }
}

impl FromJSValConvertible for u16 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<u16, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToUint16)
    }
}

impl ToJSValConvertible for i32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        Int32Value(*self)
    }
}

impl FromJSValConvertible for i32 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<i32, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToECMAInt32)
    }
}

impl ToJSValConvertible for u32 {
    fn to_jsval(&self, _cx: *mut JSContext) -> JSVal {
        UInt32Value(*self)
    }
}

impl FromJSValConvertible for u32 {
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<u32, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToECMAUint32)
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
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<i64, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToInt64)
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
    type Config = ConversionBehavior;
    fn from_jsval(cx: *mut JSContext, val: JSVal, option: ConversionBehavior) -> Result<u64, ()> {
        convert_int_from_jsval(cx, val, option, JS_ValueToUint64)
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

impl<T: Float + ToJSValConvertible> ToJSValConvertible for Finite<T> {
    #[inline]
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        let value = **self;
        value.to_jsval(cx)
    }
}

impl<T: Float + FromJSValConvertible<Config=()>> FromJSValConvertible for Finite<T> {
    type Config = ();

    fn from_jsval(cx: *mut JSContext, value: JSVal, option: ()) -> Result<Finite<T>, ()> {
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
        (**self).to_jsval(cx)
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
    unsafe {
        let mut length = 0;
        let chars = JS_GetStringCharsAndLength(cx, s, &mut length);
        assert!(!chars.is_null());
        let char_vec = slice::from_raw_parts(chars, length as usize);
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

impl ToJSValConvertible for USVString {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        self.0.to_jsval(cx)
    }
}

impl FromJSValConvertible for USVString {
    type Config = ();
    fn from_jsval(cx: *mut JSContext, value: JSVal, _: ())
                  -> Result<USVString, ()> {
        let jsstr = unsafe { JS_ValueToString(cx, value) };
        if jsstr.is_null() {
            debug!("JS_ValueToString failed");
            Err(())
        } else {
            unsafe {
                let mut length = 0;
                let chars = JS_GetStringCharsAndLength(cx, jsstr, &mut length);
                assert!(!chars.is_null());
                let char_vec = slice::from_raw_parts(chars, length as usize);
                Ok(USVString(String::from_utf16_lossy(char_vec)))
            }
        }
    }
}

impl ToJSValConvertible for ByteString {
    fn to_jsval(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let jsstr = JS_NewStringCopyN(cx, self.as_ptr() as *const libc::c_char,
                                          self.len() as libc::size_t);
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
pub const DOM_OBJECT_SLOT: u32 = 0;
const DOM_PROXY_OBJECT_SLOT: u32 = js::JSSLOT_PROXY_PRIVATE;

/// Returns the index of the slot wherein a pointer to the reflected DOM object
/// is stored.
///
/// Fails if `obj` is not a DOM object.
pub unsafe fn dom_object_slot(obj: *mut JSObject) -> u32 {
    let clasp = JS_GetClass(obj);
    if is_dom_class(&*clasp) {
        DOM_OBJECT_SLOT
    } else {
        assert!(is_dom_proxy(obj));
        DOM_PROXY_OBJECT_SLOT
    }
}

/// Get the DOM object from the given reflector.
pub unsafe fn native_from_reflector<T>(obj: *mut JSObject) -> *const T {
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
pub fn native_from_reflector_jsmanaged<T>(mut obj: *mut JSObject) -> Result<Unrooted<T>, ()>
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

        let proto_id = <T as IDLInterface>::get_prototype_id();
        let proto_depth = <T as IDLInterface>::get_prototype_depth();
        if dom_class.interface_chain[proto_depth] == proto_id {
            debug!("good prototype");
            Ok(Unrooted::from_raw(native_from_reflector(obj)))
        } else {
            debug!("bad prototype");
            Err(())
        }
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

impl<X, T: FromJSValConvertible<Config=X>> FromJSValConvertible for Option<T> {
    type Config = X;
    fn from_jsval(cx: *mut JSContext, value: JSVal, option: X) -> Result<Option<T>, ()> {
        if value.is_null_or_undefined() {
            Ok(None)
        } else {
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
