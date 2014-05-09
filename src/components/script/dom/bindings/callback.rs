/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::Reflectable;
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JS_ObjectIsCallable, JSHandleObject};
use js::jsapi::{JS_GetProperty, JSTracer, JS_CallObjectTracer, JSMutableHandleObject};
use js::jsapi::JSMutableHandleValue;
use js::jsval::{JSVal, UndefinedValue};

use std::cast;
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

#[deriving(Clone,Eq)]
pub struct CallbackInterface {
    pub callback: *mut JSObject
}

impl<S: Encoder<E>, E> Encodable<S, E> for CallbackInterface {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        unsafe {
            let tracer: *mut JSTracer = cast::transmute(s);
            "callback".to_c_str().with_ref(|name| {
                //XXXjdm the arg can be moved
                let mut cb = self.callback;
                JS_CallObjectTracer(tracer, &mut cb, name);
            });
        };
        Ok(())
    }
}

pub trait CallbackContainer {
    fn callback(&self) -> *mut JSObject;
}

impl CallbackContainer for CallbackInterface {
    fn callback(&self) -> *mut JSObject {
        self.callback
    }
}

impl CallbackInterface {
    pub fn new(callback: *mut JSObject) -> CallbackInterface {
        CallbackInterface {
            callback: callback
        }
    }

    pub fn GetCallableProperty(&self, cx: *mut JSContext, name: &str) -> Result<JSVal, ()> {
        let mut callable = UndefinedValue();
        unsafe {
            let callback = JSHandleObject {
                unnamed_field1: &self.callback,
            };
            let callablehandle = JSMutableHandleValue {
                unnamed_field1: &mut callable,
            };
            if name.to_c_str().with_ref(|name| JS_GetProperty(cx, callback, name, callablehandle)) == 0 {
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

pub fn GetJSObjectFromCallback<T: CallbackContainer>(callback: &T) -> *mut JSObject {
    callback.callback()
}

pub fn WrapCallThisObject<T: 'static + CallbackContainer + Reflectable>(cx: *mut JSContext,
                                                                        _scope: *mut JSObject,
                                                                        p: ~T) -> *mut JSObject {
    let mut obj = GetJSObjectFromCallback(p);
    assert!(obj.is_not_null());

    let obj = JSMutableHandleObject {
        unnamed_field1: &mut obj,
    };
    unsafe {
        if JS_WrapObject(cx, obj) == 0 {
            return ptr::mut_null();
        }
        return *obj.unnamed_field1;
    }
}

pub struct CallSetup {
    pub cx: *mut JSContext,
    pub handling: ExceptionHandling
}

impl CallSetup {
    pub fn new(cx: *mut JSContext, handling: ExceptionHandling) -> CallSetup {
        CallSetup {
            cx: cx,
            handling: handling
        }
    }

    pub fn GetContext(&self) -> *mut JSContext {
        self.cx
    }
}
