/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::Reflectable;
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JS_ObjectIsCallable};
use js::jsapi::{JS_GetProperty, JSTracer, JS_CallTracer};
use js::jsval::{JSVal, UndefinedValue};
use js::JSTRACE_OBJECT;

use libc;
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
    pub callback: *JSObject
}

impl<S: Encoder<E>, E> Encodable<S, E> for CallbackInterface {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        unsafe {
            let tracer: *mut JSTracer = cast::transmute(s);
            "callback".to_c_str().with_ref(|name| {
                (*tracer).debugPrinter = ptr::null();
                (*tracer).debugPrintIndex = -1;
                (*tracer).debugPrintArg = name as *libc::c_void;
                JS_CallTracer(tracer as *JSTracer, self.callback, JSTRACE_OBJECT as u32);
            });
        };
        Ok(())
    }
}

pub trait CallbackContainer {
    fn callback(&self) -> *JSObject;
}

impl CallbackContainer for CallbackInterface {
    fn callback(&self) -> *JSObject {
        self.callback
    }
}

impl CallbackInterface {
    pub fn new(callback: *JSObject) -> CallbackInterface {
        CallbackInterface {
            callback: callback
        }
    }

    pub fn GetCallableProperty(&self, cx: *JSContext, name: &str) -> Result<JSVal, ()> {
        let mut callable = UndefinedValue();
        unsafe {
            if name.to_c_str().with_ref(|name| JS_GetProperty(cx, self.callback, name, &mut callable as *mut JSVal as *JSVal)) == 0 {
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

pub fn GetJSObjectFromCallback<T: CallbackContainer>(callback: &T) -> *JSObject {
    callback.callback()
}

pub fn WrapCallThisObject<T: 'static + CallbackContainer + Reflectable>(cx: *JSContext,
                                                                        _scope: *JSObject,
                                                                        p: ~T) -> *JSObject {
    let obj = GetJSObjectFromCallback(p);
    assert!(obj.is_not_null());

    unsafe {
        if JS_WrapObject(cx, &obj) == 0 {
            return ptr::null();
        }
    }

    return obj;
}

pub struct CallSetup {
    pub cx: *JSContext,
    pub handling: ExceptionHandling
}

impl CallSetup {
    pub fn new(cx: *JSContext, handling: ExceptionHandling) -> CallSetup {
        CallSetup {
            cx: cx,
            handling: handling
        }
    }

    pub fn GetContext(&self) -> *JSContext {
        self.cx
    }
}
