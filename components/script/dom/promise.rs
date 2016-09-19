/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::CallbackContainer;
use dom::bindings::codegen::Bindings::PromiseBinding::AnyCallback;
use dom::bindings::conversions::root_from_object;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::MutHeapJSVal;
use dom::bindings::reflector::{Reflectable, MutReflectable, Reflector};
use dom::promisenativehandler::PromiseNativeHandler;
use js::conversions::ToJSValConvertible;
use js::jsapi::{CallOriginalPromiseResolve, CallOriginalPromiseReject, CallOriginalPromiseThen};
use js::jsapi::{JSAutoCompartment, CallArgs, JS_GetFunctionObject, JS_NewFunction};
use js::jsapi::{JSContext, HandleValue, HandleObject, IsPromiseObject, GetFunctionNativeReserved};
use js::jsapi::{JS_ClearPendingException, JSObject, AddRawValueRoot, RemoveRawValueRoot};
use js::jsapi::{MutableHandleObject, NewPromiseObject, ResolvePromise, RejectPromise};
use js::jsapi::{SetFunctionNativeReserved, NewFunctionWithReserved, AddPromiseReactions};
use js::jsval::{JSVal, UndefinedValue, ObjectValue, Int32Value};
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct Promise {
    reflector: Reflector,
    #[ignore_heap_size_of = "SM handles JS values"]
    root: MutHeapJSVal,
}

trait PromiseHelper {
    #[allow(unsafe_code)]
    unsafe fn initialize(&self, cx: *mut JSContext);
}

impl PromiseHelper for Rc<Promise> {
    #[allow(unsafe_code)]
    unsafe fn initialize(&self, cx: *mut JSContext) {
        let obj = self.reflector().get_jsobject();
        self.root.set(ObjectValue(&**obj));
        assert!(AddRawValueRoot(cx,
                                self.root.get_unsafe(),
                                b"Promise::root\0" as *const _ as *const _));
    }
}

impl Drop for Promise {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let cx = self.global().r().get_cx();
        unsafe {
            RemoveRawValueRoot(cx, self.root.get_unsafe());
        }
    }
}

impl Promise {
    #[allow(unsafe_code)]
    pub fn new(global: GlobalRef) -> Rc<Promise> {
        let cx = global.get_cx();
        rooted!(in(cx) let mut obj = ptr::null_mut());
        unsafe {
            Promise::create_js_promise(cx, HandleObject::null(), obj.handle_mut());
            Promise::new_with_js_promise(obj.handle(), cx)
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    pub fn duplicate(&self) -> Rc<Promise> {
        let cx = self.global().r().get_cx();
        unsafe {
            Promise::new_with_js_promise(self.reflector().get_jsobject(), cx)
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    unsafe fn new_with_js_promise(obj: HandleObject, cx: *mut JSContext) -> Rc<Promise> {
        assert!(IsPromiseObject(obj));
        let mut promise = Promise {
            reflector: Reflector::new(),
            root: MutHeapJSVal::new(),
        };
        promise.init_reflector(obj.get());
        let promise = Rc::new(promise);
        promise.initialize(cx);
        promise
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
        unsafe {
            Ok(Promise::new_with_js_promise(p.handle(), cx))
        }
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub fn Reject(global: GlobalRef,
                  cx: *mut JSContext,
                  value: HandleValue) -> Fallible<Rc<Promise>> {
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        rooted!(in(cx) let p = unsafe { CallOriginalPromiseReject(cx, value) });
        assert!(!p.handle().is_null());
        unsafe {
            Ok(Promise::new_with_js_promise(p.handle(), cx))
        }
    }

    #[allow(unsafe_code)]
    pub fn maybe_resolve_native<T>(&self, cx: *mut JSContext, val: &T) where T: ToJSValConvertible {
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            val.to_jsval(cx, v.handle_mut());
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
            val.to_jsval(cx, v.handle_mut());
        }
        self.maybe_reject(cx, v.handle());
    }

    #[allow(unsafe_code)]
    pub fn maybe_reject_error(&self, cx: *mut JSContext, error: Error) {
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            error.to_jsval(cx, self.global().r(), v.handle_mut());
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

    #[allow(unsafe_code)]
    pub fn append_native_handler(&self, handler: &PromiseNativeHandler) {
        let global = self.global();
        let cx = global.r().get_cx();
        rooted!(in(cx) let resolve_func =
                create_native_handler_function(cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Resolve));

        rooted!(in(cx) let reject_func =
                create_native_handler_function(cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Reject));

        unsafe {
            let ok = AddPromiseReactions(cx,
                                         self.promise_obj(),
                                         resolve_func.handle(),
                                         reject_func.handle());
            assert!(ok);
        }
    }
}

#[allow(unsafe_code)]
unsafe extern fn do_nothing_promise_executor(_cx: *mut JSContext, argc: u32, vp: *mut JSVal) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    *args.rval() = UndefinedValue();
    true
}

const SLOT_NATIVEHANDLER: usize = 0;
const SLOT_NATIVEHANDLER_TASK: usize = 1;

#[derive(PartialEq)]
enum NativeHandlerTask {
    Resolve = 0,
    Reject = 1,
}

#[allow(unsafe_code)]
unsafe extern fn native_handler_callback(cx: *mut JSContext, argc: u32, vp: *mut JSVal) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    rooted!(in(cx) let v = *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER));
    assert!(v.get().is_object());

    let handler = match root_from_object::<PromiseNativeHandler>(v.to_object()) {
        Ok(h) => h,
        Err(_) => {
            //TODO throw an exception
            return false;
        }
    };

    rooted!(in(cx) let v = *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER_TASK));
    match v.to_int32() {
        v if v == NativeHandlerTask::Resolve as i32 => handler.resolved_callback(cx, args.get(0)),
        v if v == NativeHandlerTask::Reject as i32 => handler.rejected_callback(cx, args.get(0)),
        _ => panic!("unexpected native handler task value"),
    };

    true
}

#[allow(unsafe_code)]
fn create_native_handler_function(cx: *mut JSContext,
                                  holder: HandleObject,
                                  task: NativeHandlerTask) -> *mut JSObject {
    unsafe {
        let func = NewFunctionWithReserved(cx, Some(native_handler_callback), 1, 0, ptr::null());
        assert!(!func.is_null());

        rooted!(in(cx) let obj = JS_GetFunctionObject(func));
        assert!(!obj.is_null());
        SetFunctionNativeReserved(obj.get(),
                                  SLOT_NATIVEHANDLER,
                                  &ObjectValue(&**holder));
        SetFunctionNativeReserved(obj.get(),
                                  SLOT_NATIVEHANDLER_TASK,
                                  &Int32Value(task as i32));
        obj.get()
    }
}
