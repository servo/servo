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

use std::{char, ffi, ptr, slice};

use js::conversions::latin1_to_string;
pub use js::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use js::error::throw_type_error;
use js::glue::{GetProxyReservedSlot, IsWrapper, JS_GetReservedSlot, UnwrapObjectDynamic};
use js::jsapi::{
    Heap, IsWindowProxy, JSContext, JSObject, JSString, JS_DeprecatedStringHasLatin1Chars,
    JS_GetLatin1StringCharsAndLength, JS_GetTwoByteStringCharsAndLength, JS_IsExceptionPending,
    JS_NewStringCopyN,
};
use js::jsval::{ObjectValue, StringValue, UndefinedValue};
use js::rust::wrappers::{IsArrayObject, JS_GetProperty, JS_HasProperty};
use js::rust::{
    get_object_class, is_dom_class, is_dom_object, maybe_wrap_value, HandleId, HandleObject,
    HandleValue, MutableHandleValue, ToString,
};
use num_traits::Float;
pub use script_bindings::conversions::*;
use servo_config::opts;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{ByteString, DOMString, USVString};
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::bindings::utils::DOMClass;
use crate::dom::filelist::FileList;
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlformcontrolscollection::HTMLFormControlsCollection;
use crate::dom::htmloptionscollection::HTMLOptionsCollection;
use crate::dom::nodelist::NodeList;
use crate::dom::windowproxy::WindowProxy;

/// Get a `DomRoot<T>` for a WindowProxy accessible from a `HandleValue`.
/// Caller is responsible for throwing a JS exception if needed in case of error.
pub unsafe fn windowproxy_from_handlevalue(
    v: HandleValue,
    cx: *mut JSContext,
) -> Result<DomRoot<WindowProxy>, ()> {
    /*if !v.get().is_object() {
        return Err(());
    }
    let object = v.get().to_object();
    if !IsWindowProxy(object) {
        return Err(());
    }
    let mut value = UndefinedValue();
    GetProxyReservedSlot(object, 0, &mut value);
    let ptr = value.to_private() as *const WindowProxy;
    Ok(DomRoot::from_ref(&*ptr))*/
    script_bindings::conversions::windowproxy_from_handlevalue::<crate::DomTypeHolder>(v, cx)
}
