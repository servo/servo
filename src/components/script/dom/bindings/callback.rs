/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::Reflectable;
use js::jsapi::{JSContext, JSObject, JS_WrapObject, JSVal, JS_ObjectIsCallable};
use js::jsapi::JS_GetProperty;
use js::{JSVAL_IS_OBJECT, JSVAL_TO_OBJECT};

use std::libc;
use std::ptr;

pub enum ExceptionHandling {
    // Report any exception and don't throw it to the caller code.
    eReportExceptions,
    // Throw an exception to the caller code if the thrown exception is a
    // binding object for a DOMError from the caller's scope, otherwise report
    // it.
    eRethrowContentExceptions,
    // Throw any exception to the caller code.
    eRethrowExceptions
}

#[deriving(Clone,Eq)]
pub struct CallbackInterface {
    callback: *JSObject
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

    pub fn GetCallableProperty(&self, cx: *JSContext, name: *libc::c_char, callable: &mut JSVal) -> bool {
        unsafe {
            if JS_GetProperty(cx, self.callback, name, &*callable) == 0 {
                return false;
            }

            if !JSVAL_IS_OBJECT(*callable) ||
               JS_ObjectIsCallable(cx, JSVAL_TO_OBJECT(*callable)) == 0 {
                //ThrowErrorMessage(cx, MSG_NOT_CALLABLE, description.get());
                return false;
            }

            return true;
        }
    }
}

pub fn GetJSObjectFromCallback<T: CallbackContainer>(callback: &T) -> *JSObject {
    callback.callback()
}

pub fn WrapCallThisObject<T: 'static + CallbackContainer + Reflectable>(cx: *JSContext,
                                                                        _scope: *JSObject,
                                                                        p: @mut T) -> *JSObject {
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
    cx: *JSContext,
    handling: ExceptionHandling
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
