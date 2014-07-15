/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JSRef;
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, global_object_for_js_object};
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JS_ObjectIsCallable};
use js::jsapi::JS_GetProperty;
use js::jsval::{JSVal, UndefinedValue};

use std::ptr;

use serialize::{Encodable, Encoder};

pub enum ExceptionHandling {
    // Report any exception and don't throw it to the caller code.
    ReportExceptions,
    // Throw an exception to the caller code if the thrown exception is a
    // binding object for a DOMError from the caller's scope, otherwise report
    // it.
    RethrowContentExceptions,
    // Throw any exception to the caller code.
    RethrowExceptions
}

#[deriving(Clone,PartialEq,Encodable)]
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

#[deriving(Clone,PartialEq,Encodable)]
pub struct CallbackInterface {
    object: CallbackObject
}

#[allow(raw_pointer_deriving)]
#[deriving(Clone,PartialEq,Encodable)]
struct CallbackObject {
    callback: Traceable<*mut JSObject>,
}

pub trait CallbackContainer {
    fn new(callback: *mut JSObject) -> Self;
    fn callback(&self) -> *mut JSObject;
}

impl CallbackInterface {
    pub fn callback(&self) -> *mut JSObject {
        *self.object.callback
    }
}

impl CallbackFunction {
    pub fn callback(&self) -> *mut JSObject {
        *self.object.callback
    }
}

impl CallbackInterface {
    pub fn new(callback: *mut JSObject) -> CallbackInterface {
        CallbackInterface {
            object: CallbackObject {
                callback: Traceable::new(callback)
            }
        }
    }

    pub fn GetCallableProperty(&self, cx: *mut JSContext, name: &str) -> Result<JSVal, ()> {
        let mut callable = UndefinedValue();
        unsafe {
            if name.to_c_str().with_ref(|name| JS_GetProperty(cx, self.callback(), name, &mut callable)) == 0 {
                return Err(());
            }

            if !callable.is_object() ||
               JS_ObjectIsCallable(cx, callable.to_object()) == 0 {
                //ThrowErrorMessage(cx, MSG_NOT_CALLABLE, description.get());
                return Err(());
            }
        }
        Ok(callable)
    }
}

pub fn WrapCallThisObject<T: Reflectable>(cx: *mut JSContext,
                                          p: &JSRef<T>) -> *mut JSObject {
    let mut obj = p.reflector().get_jsobject();
    assert!(obj.is_not_null());

    unsafe {
        if JS_WrapObject(cx, &mut obj) == 0 {
            return ptr::mut_null();
        }
    }

    return obj;
}

pub struct CallSetup {
    cx: *mut JSContext,
    _handling: ExceptionHandling
}

impl CallSetup {
    pub fn new<T: CallbackContainer>(callback: &T, handling: ExceptionHandling) -> CallSetup {
        let global = global_object_for_js_object(callback.callback()).root();
        let cx = global.root_ref().get_cx();
        CallSetup {
            cx: cx,
            _handling: handling
        }
    }

    pub fn GetContext(&self) -> *mut JSContext {
        self.cx
    }
}
