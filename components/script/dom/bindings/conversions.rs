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

use js::context::JSContext;
pub(crate) use js::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use js::jsapi::JSObject;
use js::rust::HandleObject;
pub(crate) use script_bindings::conversions::{is_dom_proxy, *};
use script_bindings::reflector::DomObject;

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
    cx: &mut JSContext,
    obj: HandleObject,
) -> Result<DomRoot<T>, ()>
where
    T: DomObject + IDLInterface,
{
    unsafe { root_from_object(cx, obj.get()) }
}
