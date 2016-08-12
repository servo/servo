/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::CallbackContainer;
use dom::bindings::codegen::Bindings::PromiseBinding::AnyCallback;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::reflector::{Reflectable, Reflector};
use js::jsapi::{JSAutoCompartment, CallArgs, JS_GetFunctionObject, JS_NewFunction};
use js::jsapi::{JSContext, HandleValue, HandleObject, IsPromiseObject};
use js::jsapi::{CallOriginalPromiseResolve, CallOriginalPromiseReject, CallOriginalPromiseThen};
use js::jsapi::{MutableHandleObject, NewPromiseObject, ResolvePromise, RejectPromise, JS_ClearPendingException};
use js::jsval::{JSVal, UndefinedValue};
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct Promise {
    reflector: Reflector,
}

impl Promise {
    #[allow(unsafe_code)]
    pub fn new(global: GlobalRef) -> Rc<Promise> {
        let cx = global.get_cx();
        rooted!(in(cx) let mut obj = ptr::null_mut());
        unsafe {
            Promise::create_js_promise(cx, HandleObject::null(), obj.handle_mut());
        }
        Promise::new_with_js_promise(obj.handle())
    }

    #[allow(unsafe_code, unrooted_must_root)]
    fn new_with_js_promise(obj: HandleObject) -> Rc<Promise> {
        unsafe {
            assert!(IsPromiseObject(obj));
        }
        let mut promise = Promise {
            reflector: Reflector::new(),
        };
        promise.init_reflector(obj.get());
        Rc::new(promise)
    }

    #[allow(unsafe_code)]
    unsafe fn create_js_promise(cx: *mut JSContext, proto: HandleObject, obj: MutableHandleObject) {
        let do_nothing_func = JS_NewFunction(cx, Some(do_nothing_promise_executor), /* nargs = */ 2,
                                             /* flags = */ 0, ptr::null());
        assert!(!do_nothing_func.is_null());
        rooted!(in(cx) let do_nothing_obj = JS_GetFunctionObject(do_nothing_func));
        assert!(!do_nothing_obj.is_null());
        obj.set(NewPromiseObject(cx, do_nothing_obj.handle(), proto));
        assert!(!obj.is_null());
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn Resolve(global: GlobalRef,
                   cx: *mut JSContext,
                   value: HandleValue) -> Fallible<Rc<Promise>> {
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        rooted!(in(cx) let p = unsafe { CallOriginalPromiseResolve(cx, value) });
        assert!(!p.handle().is_null());
        Ok(Promise::new_with_js_promise(p.handle()))
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn Reject(global: GlobalRef,
                  cx: *mut JSContext,
                  value: HandleValue) -> Fallible<Rc<Promise>> {
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        rooted!(in(cx) let p = unsafe { CallOriginalPromiseReject(cx, value) });
        assert!(!p.handle().is_null());
        Ok(Promise::new_with_js_promise(p.handle()))
    }

    #[allow(unsafe_code)]
    pub fn maybe_resolve_native<T>(&self, cx: *mut JSContext, val: &T) where T: ToJSValConvertible {
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            val.to_jsval(cx, m.handle_mut());
        }
        self.maybe_resolve(cx, v.handle());
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn maybe_resolve(&self,
                         cx: *mut JSContext,
                         value: HandleValue) {
        unsafe {
            if !ResolvePromise(cx, self.promise_obj(), value) {
                JS_ClearPendingException(cx);
            }
        }
    }

    #[allow(unsafe_code)]
    pub fn maybe_reject_native<T>(&self, cx: *mut JSContext, val: &T) where T: ToJSValConvertible {
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            val.to_jsval(cx, m.handle_mut());
        }
        self.maybe_reject(cx, v.handle());
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn maybe_reject(&self,
                        cx: *mut JSContext,
                        value: HandleValue) {
        unsafe {
            if !RejectPromise(cx, self.promise_obj(), value) {
                JS_ClearPendingException(cx);
            }
        }
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn then(&self,
                cx: *mut JSContext,
                _callee: HandleObject,
                cb_resolve: AnyCallback,
                cb_reject: AnyCallback,
                result: MutableHandleObject) {
        let promise = self.promise_obj();
        rooted!(in(cx) let resolve = cb_resolve.callback());
        rooted!(in(cx) let reject = cb_reject.callback());
        unsafe {
            rooted!(in(cx) let res =
                CallOriginalPromiseThen(cx, promise, resolve.handle(), reject.handle()));
            result.set(*res);
        }
    }

    #[allow(unsafe_code)]
    fn promise_obj(&self) -> HandleObject {
        let obj = self.reflector().get_jsobject();
        unsafe {
            assert!(IsPromiseObject(obj));
        }
        obj
    }
}

#[allow(unsafe_code)]
unsafe extern fn do_nothing_promise_executor(_cx: *mut JSContext, argc: u32, vp: *mut JSVal) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    *args.rval() = UndefinedValue();
    true
}
