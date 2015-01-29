/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use dom::bindings::global::global_object_for_js_object;
use dom::bindings::js::JSRef;
use dom::bindings::utils::Reflectable;
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JS_ObjectIsCallable, JS_GetGlobalObject};
use js::jsapi::{JS_GetProperty, JS_IsExceptionPending, JS_ReportPendingException};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::with_compartment;

use std::ffi::CString;
use std::ptr;

/// The exception handling used for a call.
#[derive(Copy, PartialEq)]
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    Report,
    /// Throw any exception to the caller code.
    Rethrow
}

/// A common base class for representing IDL callback function types.
#[derive(Copy, Clone,PartialEq)]
#[jstraceable]
pub struct CallbackFunction {
    object: CallbackObject
}

impl CallbackFunction {
    /// Create a new `CallbackFunction` for this object.
    pub fn new(callback: *mut JSObject) -> CallbackFunction {
        CallbackFunction {
            object: CallbackObject {
                callback: callback
            }
        }
    }
}

/// A common base class for representing IDL callback interface types.
#[derive(Copy, Clone,PartialEq)]
#[jstraceable]
pub struct CallbackInterface {
    object: CallbackObject
}

/// A common base class for representing IDL callback function and
/// callback interface types.
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone,PartialEq)]
#[jstraceable]
struct CallbackObject {
    /// The underlying `JSObject`.
    callback: *mut JSObject,
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
        self.object.callback
    }
}

impl CallbackFunction {
    /// Returns the underlying `JSObject`.
    pub fn callback(&self) -> *mut JSObject {
        self.object.callback
    }
}

impl CallbackInterface {
    /// Create a new CallbackInterface object for the given `JSObject`.
    pub fn new(callback: *mut JSObject) -> CallbackInterface {
        CallbackInterface {
            object: CallbackObject {
                callback: callback
            }
        }
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or `Err(())` otherwise. If it returns `Err(())`, a JSAPI exception is
    /// pending.
    pub fn get_callable_property(&self, cx: *mut JSContext, name: &str)
                                 -> Result<JSVal, ()> {
        let mut callable = UndefinedValue();
        unsafe {
            let name = CString::from_slice(name.as_bytes());
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
pub fn wrap_call_this_object<T: Reflectable>(cx: *mut JSContext,
                                             p: JSRef<T>) -> *mut JSObject {
    let mut obj = p.reflector().get_jsobject();
    assert!(!obj.is_null());

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
    _handling: ExceptionHandling,
}

impl CallSetup {
    /// Performs the setup needed to make a call.
    #[allow(unrooted_must_root)]
    pub fn new<T: CallbackContainer>(callback: T, handling: ExceptionHandling) -> CallSetup {
        let global = global_object_for_js_object(callback.callback());
        let global = global.root();
        let cx = global.r().get_cx();

        CallSetup {
            cx: cx,
            _handling: handling,
        }
    }

    /// Returns the `JSContext` used for the call.
    pub fn get_context(&self) -> *mut JSContext {
        self.cx
    }
}

impl Drop for CallSetup {
    fn drop(&mut self) {
        let need_to_deal_with_exception = unsafe { JS_IsExceptionPending(self.cx) } != 0;
        if need_to_deal_with_exception {
            unsafe {
                let old_global = JS_GetGlobalObject(self.cx);
                with_compartment(self.cx, old_global, || {
                    JS_ReportPendingException(self.cx)
                });
            }
        }
    }
}
