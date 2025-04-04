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
use js::jsapi::{JS_IsExceptionPending, JSContext, JSObject};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_GetProperty, JS_HasProperty};
use js::rust::{HandleObject, MutableHandleValue};
pub(crate) use script_bindings::conversions::{is_dom_proxy, *};

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;

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
    unsafe { native_from_object_static(obj).map(|ptr| DomRoot::from_ref(&*ptr)) }
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
