/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Conversions of Rust values to and from `JSVal`.
//!
//! | IDL type                | Argument type   | Return type    |
//! |-------------------------|-----------------|----------------|
//! | any                     | `JSVal`         |                |
//! | boolean                 | `bool`          |                |
//! | byte                    | `i8`            |                |
//! | octet                   | `u8`            |                |
//! | short                   | `i16`           |                |
//! | unsigned short          | `u16`           |                |
//! | long                    | `i32`           |                |
//! | unsigned long           | `u32`           |                |
//! | long long               | `i64`           |                |
//! | unsigned long long      | `u64`           |                |
//! | unrestricted float      | `f32`           |                |
//! | float                   | `Finite<f32>`   |                |
//! | unrestricted double     | `f64`           |                |
//! | double                  | `Finite<f64>`   |                |
//! | DOMString               | `DOMString`     |                |
//! | USVString               | `USVString`     |                |
//! | ByteString              | `ByteString`    |                |
//! | object                  | `*mut JSObject` |                |
//! | interface types         | `&T`            | `DomRoot<T>`   |
//! | dictionary types        | `&T`            | *unsupported*  |
//! | enumeration types       | `T`             |                |
//! | callback function types | `Rc<T>`         |                |
//! | nullable types          | `Option<T>`     |                |
//! | sequences               | `Vec<T>`        |                |
//! | union types             | `T`             |                |

use std::ffi;

pub(crate) use js::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use js::glue::GetProxyReservedSlot;
use js::jsapi::{Heap, IsWindowProxy, JS_IsExceptionPending, JSContext, JSObject};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{IsArrayObject, JS_GetProperty, JS_HasProperty};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
pub(crate) use script_bindings::conversions::*;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};

impl<T: ToJSValConvertible + JSTraceable> ToJSValConvertible for RootedTraceableBox<T> {
    #[inline]
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        let value = &**self;
        value.to_jsval(cx, rval);
    }
}

impl<T> FromJSValConvertible for RootedTraceableBox<Heap<T>>
where
    T: FromJSValConvertible + js::rust::GCMethods + Copy,
    Heap<T>: JSTraceable + Default,
{
    type Config = T::Config;

    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        config: Self::Config,
    ) -> Result<ConversionResult<Self>, ()> {
        T::from_jsval(cx, value, config).map(|result| match result {
            ConversionResult::Success(inner) => {
                ConversionResult::Success(RootedTraceableBox::from_box(Heap::boxed(inner)))
            },
            ConversionResult::Failure(msg) => ConversionResult::Failure(msg),
        })
    }
}

pub(crate) use script_bindings::conversions::is_dom_proxy;

/// Get a `DomRoot<T>` for the given DOM object, unwrapping any wrapper
/// around it first, and checking if the object is of the correct type.
///
/// Returns Err(()) if `obj` is an opaque security wrapper or if the object is
/// not a reflector for a DOM object of the given type (as defined by the
/// proto_id and proto_depth).
pub(crate) fn root_from_object_static<T>(obj: *mut JSObject) -> Result<DomRoot<T>, ()>
where
    T: DomObject + IDLInterface,
{
    native_from_object_static(obj).map(|ptr| unsafe { DomRoot::from_ref(&*ptr) })
}

/// Get a `DomRoot<T>` for a DOM object accessible from a `HandleObject`.
pub(crate) fn root_from_handleobject<T>(
    obj: HandleObject,
    cx: *mut JSContext,
) -> Result<DomRoot<T>, ()>
where
    T: DomObject + IDLInterface,
{
    unsafe { root_from_object(obj.get(), cx) }
}

/// Returns whether `value` is an array-like object (Array, FileList,
/// HTMLCollection, HTMLFormControlsCollection, HTMLOptionsCollection,
/// NodeList).
pub(crate) unsafe fn is_array_like<D: crate::DomTypes>(cx: *mut JSContext, value: HandleValue) -> bool {
    let mut is_array = false;
    assert!(IsArrayObject(cx, value, &mut is_array));
    if is_array {
        return true;
    }

    let object: *mut JSObject = match FromJSValConvertible::from_jsval(cx, value, ()).unwrap() {
        ConversionResult::Success(object) => object,
        _ => return false,
    };

    if root_from_object::<D::FileList>(object, cx).is_ok() {
        return true;
    }
    if root_from_object::<D::HTMLCollection>(object, cx).is_ok() {
        return true;
    }
    if root_from_object::<D::HTMLFormControlsCollection>(object, cx).is_ok() {
        return true;
    }
    if root_from_object::<D::HTMLOptionsCollection>(object, cx).is_ok() {
        return true;
    }
    if root_from_object::<D::NodeList>(object, cx).is_ok() {
        return true;
    }

    false
}

/// Get a property from a JS object.
pub(crate) unsafe fn get_property_jsval(
    cx: *mut JSContext,
    object: HandleObject,
    name: &str,
    mut rval: MutableHandleValue,
) -> Fallible<()> {
    rval.set(UndefinedValue());
    let cname = match ffi::CString::new(name) {
        Ok(cname) => cname,
        Err(_) => return Ok(()),
    };
    let mut found = false;
    if JS_HasProperty(cx, object, cname.as_ptr(), &mut found) && found {
        JS_GetProperty(cx, object, cname.as_ptr(), rval);
        if JS_IsExceptionPending(cx) {
            return Err(Error::JSFailed);
        }
        Ok(())
    } else if JS_IsExceptionPending(cx) {
        Err(Error::JSFailed)
    } else {
        Ok(())
    }
}

/// Get a property from a JS object, and convert it to a Rust value.
pub(crate) unsafe fn get_property<T>(
    cx: *mut JSContext,
    object: HandleObject,
    name: &str,
    option: T::Config,
) -> Fallible<Option<T>>
where
    T: FromJSValConvertible,
{
    debug!("Getting property {}.", name);
    rooted!(in(cx) let mut result = UndefinedValue());
    get_property_jsval(cx, object, name, result.handle_mut())?;
    if result.is_undefined() {
        debug!("No property {}.", name);
        return Ok(None);
    }
    debug!("Converting property {}.", name);
    match T::from_jsval(cx, result.handle(), option) {
        Ok(ConversionResult::Success(value)) => Ok(Some(value)),
        Ok(ConversionResult::Failure(_)) => Ok(None),
        Err(()) => Err(Error::JSFailed),
    }
}

/// Get a `DomRoot<T>` for a WindowProxy accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub(crate) unsafe fn windowproxy_from_handlevalue<D: crate::DomTypes>(
    v: HandleValue,
    _cx: *mut JSContext,
) -> Result<DomRoot<D::WindowProxy>, ()> {
    if !v.get().is_object() {
        return Err(());
    }
    let object = v.get().to_object();
    if !IsWindowProxy(object) {
        return Err(());
    }
    let mut value = UndefinedValue();
    GetProxyReservedSlot(object, 0, &mut value);
    let ptr = value.to_private() as *const D::WindowProxy;
    Ok(DomRoot::from_ref(&*ptr))
}
