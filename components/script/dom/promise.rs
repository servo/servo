/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Native representation of JS Promise values.
//!
//! This implementation differs from the traditional Rust DOM object, because the reflector
//! is provided by SpiderMonkey and has no knowledge of an associated native representation
//! (ie. dom::Promise). This means that native instances use native reference counting (Rc)
//! to ensure that no memory is leaked, which means that there can be multiple instances of
//! native Promise values that refer to the same JS value yet are distinct native objects
//! (ie. address equality for the native objects is meaningless).

use dom::bindings::conversions::root_from_object;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{DomObject, MutDomObject, Reflector};
use dom::bindings::utils::AsCCharPtrPtr;
use dom::globalscope::GlobalScope;
use dom::promisenativehandler::PromiseNativeHandler;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{CallOriginalPromiseResolve, CallOriginalPromiseReject};
use js::jsapi::{JSAutoCompartment, CallArgs, JS_GetFunctionObject, JS_NewFunction};
use js::jsapi::{JSContext, HandleValue, HandleObject, IsPromiseObject, GetFunctionNativeReserved};
use js::jsapi::{JS_ClearPendingException, JSObject, AddRawValueRoot, RemoveRawValueRoot, PromiseState};
use js::jsapi::{MutableHandleObject, NewPromiseObject, ResolvePromise, RejectPromise, GetPromiseState};
use js::jsapi::{SetFunctionNativeReserved, NewFunctionWithReserved, AddPromiseReactions};
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue, ObjectValue, Int32Value};
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct Promise {
    reflector: Reflector,
    /// Since Promise values are natively reference counted without the knowledge of
    /// the SpiderMonkey GC, an explicit root for the reflector is stored while any
    /// native instance exists. This ensures that the reflector will never be GCed
    /// while native code could still interact with its native representation.
    #[ignore_malloc_size_of = "SM handles JS values"]
    permanent_js_root: Heap<JSVal>,
}

/// Private helper to enable adding new methods to Rc<Promise>.
trait PromiseHelper {
    #[allow(unsafe_code)]
    unsafe fn initialize(&self, cx: *mut JSContext);
}

impl PromiseHelper for Rc<Promise> {
    #[allow(unsafe_code)]
    unsafe fn initialize(&self, cx: *mut JSContext) {
        let obj = self.reflector().get_jsobject();
        self.permanent_js_root.set(ObjectValue(*obj));
        assert!(AddRawValueRoot(cx,
                                self.permanent_js_root.get_unsafe(),
                                b"Promise::root\0".as_c_char_ptr()));
    }
}

impl Drop for Promise {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let cx = self.global().get_cx();
        unsafe {
            RemoveRawValueRoot(cx, self.permanent_js_root.get_unsafe());
        }
    }
}

impl Promise {
    #[allow(unsafe_code)]
    pub fn new(global: &GlobalScope) -> Rc<Promise> {
        let cx = global.get_cx();
        rooted!(in(cx) let mut obj = ptr::null_mut::<JSObject>());
        unsafe {
            Promise::create_js_promise(cx, HandleObject::null(), obj.handle_mut());
            Promise::new_with_js_promise(obj.handle(), cx)
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    pub fn duplicate(&self) -> Rc<Promise> {
        let cx = self.global().get_cx();
        unsafe {
            Promise::new_with_js_promise(self.reflector().get_jsobject(), cx)
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    unsafe fn new_with_js_promise(obj: HandleObject, cx: *mut JSContext) -> Rc<Promise> {
        assert!(IsPromiseObject(obj));
        let promise = Promise {
            reflector: Reflector::new(),
            permanent_js_root: Heap::default(),
        };
        let mut promise = Rc::new(promise);
        Rc::get_mut(&mut promise).unwrap().init_reflector(obj.get());
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
    pub unsafe fn new_resolved(
        global: &GlobalScope,
        cx: *mut JSContext,
        value: HandleValue,
    ) -> Fallible<Rc<Promise>> {
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        rooted!(in(cx) let p = CallOriginalPromiseResolve(cx, value));
        assert!(!p.handle().is_null());
        Ok(Promise::new_with_js_promise(p.handle(), cx))
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub unsafe fn new_rejected(
        global: &GlobalScope,
        cx: *mut JSContext,
        value: HandleValue,
    ) -> Fallible<Rc<Promise>> {
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        rooted!(in(cx) let p = CallOriginalPromiseReject(cx, value));
        assert!(!p.handle().is_null());
        Ok(Promise::new_with_js_promise(p.handle(), cx))
    }

    #[allow(unsafe_code)]
    pub fn resolve_native<T>(&self, val: &T) where T: ToJSValConvertible {
        let cx = self.global().get_cx();
        let _ac = JSAutoCompartment::new(cx, self.reflector().get_jsobject().get());
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            val.to_jsval(cx, v.handle_mut());
            self.resolve(cx, v.handle());
        }
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub unsafe fn resolve(&self, cx: *mut JSContext, value: HandleValue) {
        if !ResolvePromise(cx, self.promise_obj(), value) {
            JS_ClearPendingException(cx);
        }
    }

    #[allow(unsafe_code)]
    pub fn reject_native<T>(&self, val: &T) where T: ToJSValConvertible {
        let cx = self.global().get_cx();
        let _ac = JSAutoCompartment::new(cx, self.reflector().get_jsobject().get());
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            val.to_jsval(cx, v.handle_mut());
            self.reject(cx, v.handle());
        }
    }

    #[allow(unsafe_code)]
    pub fn reject_error(&self, error: Error) {
        let cx = self.global().get_cx();
        let _ac = JSAutoCompartment::new(cx, self.reflector().get_jsobject().get());
        rooted!(in(cx) let mut v = UndefinedValue());
        unsafe {
            error.to_jsval(cx, &self.global(), v.handle_mut());
            self.reject(cx, v.handle());
        }
    }

    #[allow(unrooted_must_root, unsafe_code)]
    pub unsafe fn reject(&self, cx: *mut JSContext, value: HandleValue) {
        if !RejectPromise(cx, self.promise_obj(), value) {
            JS_ClearPendingException(cx);
        }
    }

    #[allow(unsafe_code)]
    pub fn is_fulfilled(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        match state {
            PromiseState::Rejected | PromiseState::Fulfilled => true,
            _ => false
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
        let cx = self.global().get_cx();
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

    let handler = root_from_object::<PromiseNativeHandler>(v.to_object())
        .expect("unexpected value for native handler in promise native handler callback");

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
                                  &ObjectValue(*holder));
        SetFunctionNativeReserved(obj.get(),
                                  SLOT_NATIVEHANDLER_TASK,
                                  &Int32Value(task as i32));
        obj.get()
    }
}

