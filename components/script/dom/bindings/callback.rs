/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use dom::bindings::global::global_object_for_js_object;
use dom::bindings::js::JSRef;
use dom::bindings::trace::Traceable;
use dom::bindings::utils::Reflectable;
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JS_ObjectIsCallable};
use js::jsapi::JS_GetProperty;
use js::jsval::{JSVal, UndefinedValue};

use std::ptr;

/// The exception handling used for a call.
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    ReportExceptions,
    /// Throw an exception to the caller code if the thrown exception is a
    /// binding object for a DOMError from the caller's scope, otherwise report
    /// it.
    RethrowContentExceptions,
    /// Throw any exception to the caller code.
    RethrowExceptions
}

/// A common base class for representing IDL callback function types.
#[deriving(Clone,PartialEq)]
#[jstraceable]
pub struct CallbackFunction {
    object: CallbackObject
}

impl CallbackFunction {
    pub fn new(callback: *mut JSObject) -> CallbackFunction {
        CallbackFunction {
            object: CallbackObject {
                callback: Traceable::new(callback)
            }
        }
    }
}

/// A common base class for representing IDL callback interface types.
#[deriving(Clone,PartialEq)]
#[jstraceable]
pub struct CallbackInterface {
    object: CallbackObject
}

/// A common base class for representing IDL callback function and
/// callback interface types.
#[allow(raw_pointer_deriving)]
#[deriving(Clone,PartialEq)]
#[jstraceable]
struct CallbackObject {
    /// The underlying `JSObject`.
    callback: Traceable<*mut JSObject>,
}

/// A trait to be implemented by concrete IDL callback function and
/// callback interface types.
pub trait CallbackContainer {
    /// Create a new CallbackContainer object for the given `JSObject`.
    fn new(callback: *mut JSObject) -> Self;
    /// Returns the underlying `JSObject`.
    fn callback(&self) -> *mut JSObject;
}

impl CallbackInterface {
    /// Returns the underlying `JSObject`.
    pub fn callback(&self) -> *mut JSObject {
        *self.object.callback
    }
}

impl CallbackFunction {
    /// Returns the underlying `JSObject`.
    pub fn callback(&self) -> *mut JSObject {
        *self.object.callback
    }
}

impl CallbackInterface {
    /// Create a new CallbackInterface object for the given `JSObject`.
    pub fn new(callback: *mut JSObject) -> CallbackInterface {
        CallbackInterface {
            object: CallbackObject {
                callback: Traceable::new(callback)
            }
        }
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or `Err(())` otherwise. If it returns `Err(())`, a JSAPI exception is
    /// pending.
    pub fn GetCallableProperty(&self, cx: *mut JSContext, name: &str) -> Result<JSVal, ()> {
        let mut callable = UndefinedValue();
        unsafe {
            let name = name.to_c_str();
            if JS_GetProperty(cx, self.callback(), name.as_ptr(), &mut callable) == 0 {
                return Err(());
            }

            if !callable.is_object() ||
               JS_ObjectIsCallable(cx, callable.to_object()) == 0 {
                // FIXME(#347)
                //ThrowErrorMessage(cx, MSG_NOT_CALLABLE, description.get());
                return Err(());
            }
        }
        Ok(callable)
    }
}

/// Wraps the reflector for `p` into the compartment of `cx`.
pub fn WrapCallThisObject<T: Reflectable>(cx: *mut JSContext,
                                          p: JSRef<T>) -> *mut JSObject {
    let mut obj = p.reflector().get_jsobject();
    assert!(obj.is_not_null());

    unsafe {
        if JS_WrapObject(cx, &mut obj) == 0 {
            return ptr::null_mut();
        }
    }

    return obj;
}

/// A class that performs whatever setup we need to safely make a call while
/// this class is on the stack. After `new` returns, the call is safe to make.
pub struct CallSetup {
    /// The `JSContext` used for the call.
    cx: *mut JSContext,
    /// The exception handling used for the call.
    _handling: ExceptionHandling
}

impl CallSetup {
    /// Performs the setup needed to make a call.
    #[allow(unrooted_must_root)]
    pub fn new<T: CallbackContainer>(callback: T, handling: ExceptionHandling) -> CallSetup {
        let global = global_object_for_js_object(callback.callback());
        let global = global.root();
        let cx = global.root_ref().get_cx();
        CallSetup {
            cx: cx,
            _handling: handling
        }
    }

    /// Returns the `JSContext` used for the call.
    pub fn GetContext(&self) -> *mut JSContext {
        self.cx
    }
}
