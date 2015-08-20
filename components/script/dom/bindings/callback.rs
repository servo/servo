/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use dom::bindings::error::{Fallible, Error};
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::utils::Reflectable;
use js::jsapi::GetGlobalForObjectCrossCompartment;
use js::jsapi::{JSAutoCompartment};
use js::jsapi::{JSContext, JSObject, JS_WrapObject, IsCallable};
use js::jsapi::{JS_BeginRequest, JS_EndRequest};
use js::jsapi::{JS_EnterCompartment, JS_LeaveCompartment, JSCompartment};
use js::jsapi::{JS_GetProperty, JS_IsExceptionPending, JS_ReportPendingException};
use js::jsapi::{JS_SaveFrameChain, JS_RestoreFrameChain};
use js::jsapi::{RootedObject, RootedValue, MutableHandleObject, Heap};
use js::jsval::{JSVal, UndefinedValue};

use std::default::Default;
use std::ffi::CString;
use std::intrinsics::return_address;
use std::ptr;
use std::rc::Rc;

/// The exception handling used for a call.
#[derive(Copy, Clone, PartialEq)]
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    Report,
    /// Throw any exception to the caller code.
    Rethrow
}

/// A common base class for representing IDL callback function types.
#[derive(JSTraceable, PartialEq)]
pub struct CallbackFunction {
    object: CallbackObject
}

impl CallbackFunction {
    /// Create a new `CallbackFunction` for this object.
    pub fn new() -> CallbackFunction {
        CallbackFunction {
            object: CallbackObject {
                callback: Heap::default()
            }
        }
    }

    /// Initialize the callback function with a value.
    /// Should be called once this object is done moving.
    pub fn init(&mut self, callback: *mut JSObject) {
        self.object.callback.set(callback);
    }
}

/// A common base class for representing IDL callback interface types.
#[derive(JSTraceable, PartialEq)]
pub struct CallbackInterface {
    object: CallbackObject
}

/// A common base class for representing IDL callback function and
/// callback interface types.
#[allow(raw_pointer_derive)]
#[derive(JSTraceable)]
struct CallbackObject {
    /// The underlying `JSObject`.
    callback: Heap<*mut JSObject>,
}

impl PartialEq for CallbackObject {
    fn eq(&self, other: &CallbackObject) -> bool {
        self.callback.get() == other.callback.get()
    }
}

/// A trait to be implemented by concrete IDL callback function and
/// callback interface types.
pub trait CallbackContainer {
    /// Create a new CallbackContainer object for the given `JSObject`.
    fn new(callback: *mut JSObject) -> Rc<Self>;
    /// Returns the underlying `JSObject`.
    fn callback(&self) -> *mut JSObject;
}

impl CallbackInterface {
    /// Returns the underlying `JSObject`.
    pub fn callback(&self) -> *mut JSObject {
        self.object.callback.get()
    }
}

impl CallbackFunction {
    /// Returns the underlying `JSObject`.
    pub fn callback(&self) -> *mut JSObject {
        self.object.callback.get()
    }
}

impl CallbackInterface {
    /// Create a new CallbackInterface object for the given `JSObject`.
    pub fn new() -> CallbackInterface {
        CallbackInterface {
            object: CallbackObject {
                callback: Heap::default()
            }
        }
    }

    /// Initialize the callback function with a value.
    /// Should be called once this object is done moving.
    pub fn init(&mut self, callback: *mut JSObject) {
        self.object.callback.set(callback);
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or an error otherwise.
    pub fn get_callable_property(&self, cx: *mut JSContext, name: &str)
                                 -> Fallible<JSVal> {
        let mut callable = RootedValue::new(cx, UndefinedValue());
        let obj = RootedObject::new(cx, self.callback());
        unsafe {
            let c_name = CString::new(name).unwrap();
            if JS_GetProperty(cx, obj.handle(), c_name.as_ptr(),
                              callable.handle_mut()) == 0 {
                return Err(Error::JSFailed);
            }

            if !callable.ptr.is_object() ||
               IsCallable(callable.ptr.to_object()) == 0 {
                return Err(Error::Type(
                    format!("The value of the {} property is not callable", name)));
            }
        }
        Ok(callable.ptr)
    }
}

/// Wraps the reflector for `p` into the compartment of `cx`.
pub fn wrap_call_this_object<T: Reflectable>(cx: *mut JSContext,
                                             p: &T,
                                             rval: MutableHandleObject) {
    rval.set(p.reflector().get_jsobject().get());
    assert!(!rval.get().is_null());

    unsafe {
        if JS_WrapObject(cx, rval) == 0 {
            rval.set(ptr::null_mut());
        }
    }
}

/// A class that performs whatever setup we need to safely make a call while
/// this class is on the stack. After `new` returns, the call is safe to make.
pub struct CallSetup {
    /// The compartment for reporting exceptions.
    /// As a RootedObject, this must be the first field in order to
    /// determine the final address on the stack correctly.
    exception_compartment: RootedObject,
    /// The `JSContext` used for the call.
    cx: *mut JSContext,
    /// The compartment we were in before the call.
    old_compartment: *mut JSCompartment,
    /// The exception handling used for the call.
    handling: ExceptionHandling,
}

impl CallSetup {
    /// Performs the setup needed to make a call.
    #[allow(unrooted_must_root)]
    pub fn new<T: CallbackContainer>(callback: &T, handling: ExceptionHandling) -> CallSetup {
        let global = global_object_for_js_object(callback.callback());
        let cx = global.r().get_cx();
        unsafe { JS_BeginRequest(cx); }

        let exception_compartment = unsafe {
            GetGlobalForObjectCrossCompartment(callback.callback())
        };
        CallSetup {
            exception_compartment:
                RootedObject::new_with_addr(cx, exception_compartment,
                                            unsafe { return_address() }),
            cx: cx,
            old_compartment: unsafe { JS_EnterCompartment(cx, callback.callback()) },
            handling: handling,
        }
    }

    /// Returns the `JSContext` used for the call.
    pub fn get_context(&self) -> *mut JSContext {
        self.cx
    }
}

impl Drop for CallSetup {
    fn drop(&mut self) {
        unsafe { JS_LeaveCompartment(self.cx, self.old_compartment); }
        let need_to_deal_with_exception =
            self.handling == ExceptionHandling::Report &&
            unsafe { JS_IsExceptionPending(self.cx) } != 0;
        if need_to_deal_with_exception {
            unsafe {
                let old_global = RootedObject::new(self.cx, self.exception_compartment.ptr);
                let saved = JS_SaveFrameChain(self.cx) != 0;
                {
                    let _ac = JSAutoCompartment::new(self.cx, old_global.ptr);
                    JS_ReportPendingException(self.cx);
                }
                if saved {
                    JS_RestoreFrameChain(self.cx);
                }
            }
        }
        unsafe { JS_EndRequest(self.cx); }
    }
}
