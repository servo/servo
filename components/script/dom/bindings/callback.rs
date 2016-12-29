/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::bindings::settings_stack::AutoEntryScript;
use dom::globalscope::GlobalScope;
use js::jsapi::{Heap, MutableHandleObject};
use js::jsapi::{IsCallable, JSContext, JSObject, JS_WrapObject};
use js::jsapi::{JSCompartment, JS_EnterCompartment, JS_LeaveCompartment};
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_GetProperty;
use js::jsval::{JSVal, UndefinedValue};
use std::default::Default;
use std::ffi::CString;
use std::mem::drop;
use std::ptr;
use std::rc::Rc;

/// The exception handling used for a call.
#[derive(Copy, Clone, PartialEq)]
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    Report,
    /// Throw any exception to the caller code.
    Rethrow,
}

/// A common base class for representing IDL callback function types.
#[derive(JSTraceable, PartialEq)]
pub struct CallbackFunction {
    object: CallbackObject,
}

impl CallbackFunction {
    /// Create a new `CallbackFunction` for this object.
    pub fn new() -> CallbackFunction {
        CallbackFunction {
            object: CallbackObject {
                callback: Heap::default(),
            },
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
    object: CallbackObject,
}

/// A common base class for representing IDL callback function and
/// callback interface types.
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
                callback: Heap::default(),
            },
        }
    }

    /// Initialize the callback function with a value.
    /// Should be called once this object is done moving.
    pub fn init(&mut self, callback: *mut JSObject) {
        self.object.callback.set(callback);
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or an error otherwise.
    pub fn get_callable_property(&self, cx: *mut JSContext, name: &str) -> Fallible<JSVal> {
        rooted!(in(cx) let mut callable = UndefinedValue());
        rooted!(in(cx) let obj = self.callback());
        unsafe {
            let c_name = CString::new(name).unwrap();
            if !JS_GetProperty(cx, obj.handle(), c_name.as_ptr(), callable.handle_mut()) {
                return Err(Error::JSFailed);
            }

            if !callable.is_object() || !IsCallable(callable.to_object()) {
                return Err(Error::Type(format!("The value of the {} property is not callable",
                                               name)));
            }
        }
        Ok(callable.get())
    }
}

/// Wraps the reflector for `p` into the compartment of `cx`.
pub fn wrap_call_this_object<T: DomObject>(cx: *mut JSContext,
                                           p: &T,
                                           rval: MutableHandleObject) {
    rval.set(p.reflector().get_jsobject().get());
    assert!(!rval.get().is_null());

    unsafe {
        if !JS_WrapObject(cx, rval) {
            rval.set(ptr::null_mut());
        }
    }
}

/// A class that performs whatever setup we need to safely make a call while
/// this class is on the stack. After `new` returns, the call is safe to make.
pub struct CallSetup {
    /// The global for reporting exceptions. This is the global object of the
    /// (possibly wrapped) callback object.
    exception_global: Root<GlobalScope>,
    /// The `JSContext` used for the call.
    cx: *mut JSContext,
    /// The compartment we were in before the call.
    old_compartment: *mut JSCompartment,
    /// The exception handling used for the call.
    handling: ExceptionHandling,
    /// https://heycam.github.io/webidl/#es-invoking-callback-functions
    /// steps 8 and 18.2.
    entry_script: Option<AutoEntryScript>,
}

impl CallSetup {
    /// Performs the setup needed to make a call.
    #[allow(unrooted_must_root)]
    pub fn new<T: CallbackContainer>(callback: &T,
                                     handling: ExceptionHandling)
                                     -> CallSetup {
        let global = unsafe { GlobalScope::from_object(callback.callback()) };
        let cx = global.get_cx();

        let aes = AutoEntryScript::new(&global);
        CallSetup {
            exception_global: global,
            cx: cx,
            old_compartment: unsafe { JS_EnterCompartment(cx, callback.callback()) },
            handling: handling,
            entry_script: Some(aes),
        }
    }

    /// Returns the `JSContext` used for the call.
    pub fn get_context(&self) -> *mut JSContext {
        self.cx
    }
}

impl Drop for CallSetup {
    fn drop(&mut self) {
        unsafe {
            JS_LeaveCompartment(self.cx, self.old_compartment);
            if self.handling == ExceptionHandling::Report {
                let _ac = JSAutoCompartment::new(self.cx,
                                                 self.exception_global.reflector().get_jsobject().get());
                report_pending_exception(self.cx, true);
            }
            drop(self.entry_script.take().unwrap());
        }
    }
}
