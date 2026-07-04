/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Native representation of JS Promise values.
//!
//! This implementation differs from the traditional Rust DOM object, because the reflector
//! is provided by SpiderMonkey and has no knowledge of an associated native representation
//! (ie. dom::Promise). This means that native instances use native reference counting (Rc)
//! to ensure that no memory is leaked, which means that there can be multiple instances of
//! native Promise values that refer to the same JS value yet are distinct native objects
//! (ie. address equality for the native objects is meaningless).

use std::cell::{Cell, RefCell};
use std::ops::DerefMut;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::{ConversionResult, FromJSValConvertibleRc};
use js::jsapi::{
    CallArgs, GetFunctionNativeReserved, Heap, JS_GetFunctionObject, JSContext as RawJSContext,
    JSObject, PromiseState, PromiseUserInputEventHandlingState, RemoveRawValueRoot,
    SetFunctionNativeReserved,
};
use js::jsval::{Int32Value, JSVal, NullValue, ObjectValue, UndefinedValue};
use js::realm::{AutoRealm, CurrentRealm};
use js::rust::wrappers2::{
    AddPromiseReactions, AddRawValueRoot, CallOriginalPromiseReject, CallOriginalPromiseResolve,
    GetPromiseIsHandled, GetPromiseState, IsPromiseObject, JS_ClearPendingException,
    JS_NewFunction, NewFunctionWithReserved, NewPromiseObject, RejectPromise, ResolvePromise,
    SetAnyPromiseIsHandled, SetPromiseUserInputEventHandlingState,
};
use js::rust::{HandleObject, HandleValue, MutableHandleObject, Runtime};
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::reflector::{DomObject, MutDomObject, Reflector};
use script_bindings::settings_stack::run_a_script;

use crate::DomTypeHolder;
use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{AsHandleValue, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::microtask::{Microtask, MicrotaskRunnable};
use crate::realms::enter_auto_realm;
use crate::script_thread::ScriptThread;

#[dom_struct]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_in_rc)]
pub(crate) struct Promise {
    reflector: Reflector,
    /// Since Promise values are natively reference counted without the knowledge of
    /// the SpiderMonkey GC, an explicit root for the reflector is stored while any
    /// native instance exists. This ensures that the reflector will never be GCed
    /// while native code could still interact with its native representation.
    #[ignore_malloc_size_of = "SM handles JS values"]
    permanent_js_root: Heap<JSVal>,
}

/// Private helper to enable adding new methods to `Rc<Promise>`.
trait PromiseHelper {
    fn initialize(&self, cx: &mut JSContext);
}

impl PromiseHelper for Rc<Promise> {
    #[expect(unsafe_code)]
    fn initialize(&self, cx: &mut JSContext) {
        let obj = self.reflector().get_jsobject();
        self.permanent_js_root.set(ObjectValue(*obj));
        unsafe {
            assert!(AddRawValueRoot(
                cx,
                self.permanent_js_root.get_unsafe(),
                c"Promise::root".as_ptr(),
            ));
        }
    }
}

// Promise objects are stored inside Rc values, so Drop is run when the last Rc is dropped,
// rather than when SpiderMonkey runs a GC. This makes it safe to interact with the JS engine unlike
// Drop implementations for other DOM types.
impl Drop for Promise {
    #[expect(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            let object = self.permanent_js_root.get().to_object();
            assert!(!object.is_null());
            if let Some(cx) = Runtime::get() {
                RemoveRawValueRoot(cx.as_ptr(), self.permanent_js_root.get_unsafe());
            }
        }
    }
}

impl Promise {
    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> Rc<Promise> {
        let mut realm = enter_auto_realm(cx, global);
        let cx = &mut realm.current_realm();
        Promise::new_in_realm(cx)
    }

    pub(crate) fn new_in_realm(current_realm: &mut CurrentRealm) -> Rc<Promise> {
        let cx = current_realm.deref_mut();
        rooted!(&in(cx) let mut obj = ptr::null_mut::<JSObject>());
        Promise::create_js_promise(cx, obj.handle_mut());
        Promise::new_with_js_promise(cx, obj.handle())
    }

    pub(crate) fn duplicate(&self, cx: &mut JSContext) -> Rc<Promise> {
        Promise::new_with_js_promise(cx, self.reflector().get_jsobject())
    }

    #[expect(unsafe_code)]
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new_with_js_promise(cx: &mut JSContext, obj: HandleObject) -> Rc<Promise> {
        unsafe {
            assert!(IsPromiseObject(obj));
        }
        let promise = Promise {
            reflector: Reflector::new(),
            permanent_js_root: Heap::default(),
        };
        let promise = Rc::new(promise);
        unsafe {
            promise.init_reflector_without_associated_memory(obj.get());
        }
        promise.initialize(cx);
        promise
    }

    #[expect(unsafe_code)]
    // The apparently-unused CanGc parameter reflects the fact that the JS API calls
    // like JS_NewFunction can trigger a GC.
    fn create_js_promise(cx: &mut JSContext, mut obj: MutableHandleObject) {
        unsafe {
            let do_nothing_func = JS_NewFunction(
                cx,
                Some(do_nothing_promise_executor),
                /* nargs = */ 2,
                /* flags = */ 0,
                ptr::null(),
            );
            assert!(!do_nothing_func.is_null());
            rooted!(&in(cx) let do_nothing_obj = JS_GetFunctionObject(do_nothing_func));
            assert!(!do_nothing_obj.is_null());
            obj.set(NewPromiseObject(cx, do_nothing_obj.handle()));
            assert!(!obj.is_null());
            let is_user_interacting = if ScriptThread::is_user_interacting() {
                PromiseUserInputEventHandlingState::HadUserInteractionAtCreation
            } else {
                PromiseUserInputEventHandlingState::DidntHaveUserInteractionAtCreation
            };
            SetPromiseUserInputEventHandlingState(obj.handle(), is_user_interacting);
        }
    }

    #[expect(unsafe_code)]
    pub(crate) fn new_resolved(
        cx: &mut JSContext,
        global: &GlobalScope,
        value: impl SafeToJSValConvertible,
    ) -> Rc<Promise> {
        let mut realm = enter_auto_realm(cx, global);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut rval = UndefinedValue());
        value.safe_to_jsval(cx, rval.handle_mut());
        rooted!(&in(cx) let p = unsafe { CallOriginalPromiseResolve(cx, rval.handle()) });
        assert!(!p.handle().is_null());
        Promise::new_with_js_promise(cx, p.handle())
    }

    #[expect(unsafe_code)]
    pub(crate) fn new_rejected(
        cx: &mut JSContext,
        global: &GlobalScope,
        value: impl SafeToJSValConvertible,
    ) -> Rc<Promise> {
        let mut realm = enter_auto_realm(cx, global);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut rval = UndefinedValue());
        value.safe_to_jsval(cx, rval.handle_mut());
        rooted!(&in(cx) let p = unsafe { CallOriginalPromiseReject(cx, rval.handle()) });
        assert!(!p.handle().is_null());
        Promise::new_with_js_promise(cx, p.handle())
    }

    pub(crate) fn resolve_native<T>(&self, cx: &mut JSContext, val: &T)
    where
        T: SafeToJSValConvertible,
    {
        let mut realm = enter_auto_realm(cx, self);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut v = UndefinedValue());
        val.safe_to_jsval(cx, v.handle_mut());
        self.resolve(cx, v.handle());
    }

    #[expect(unsafe_code)]
    pub(crate) fn resolve(&self, cx: &mut JSContext, value: HandleValue) {
        unsafe {
            if !ResolvePromise(cx, self.promise_obj(), value) {
                JS_ClearPendingException(cx);
            }
        }
    }

    pub(crate) fn reject_native<T>(&self, cx: &mut JSContext, val: &T)
    where
        T: SafeToJSValConvertible,
    {
        let mut realm = enter_auto_realm(cx, self);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut v = UndefinedValue());
        val.safe_to_jsval(cx, v.handle_mut());
        self.reject(cx, v.handle());
    }

    pub(crate) fn reject_error(&self, cx: &mut JSContext, error: Error) {
        let mut realm = enter_auto_realm(cx, self);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut v = UndefinedValue());
        error.to_jsval(cx, &self.global(), v.handle_mut());
        self.reject(cx, v.handle());
    }

    #[expect(unsafe_code)]
    pub(crate) fn reject(&self, cx: &mut JSContext, value: HandleValue) {
        unsafe {
            if !RejectPromise(cx, self.promise_obj(), value) {
                JS_ClearPendingException(cx);
            }
        }
    }

    #[expect(unsafe_code)]
    pub(crate) fn is_fulfilled(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Rejected | PromiseState::Fulfilled)
    }

    #[expect(unsafe_code)]
    pub(crate) fn is_rejected(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Rejected)
    }

    #[expect(unsafe_code)]
    pub(crate) fn is_pending(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Pending)
    }

    #[expect(unsafe_code)]
    pub(crate) fn promise_obj(&self) -> HandleObject<'_> {
        let obj = self.reflector().get_jsobject();
        unsafe {
            assert!(IsPromiseObject(obj));
        }
        obj
    }

    #[expect(unsafe_code)]
    pub(crate) fn append_native_handler(
        &self,
        cx: &mut CurrentRealm,
        handler: &PromiseNativeHandler,
    ) {
        let global = GlobalScope::from_current_realm(cx);
        run_a_script::<DomTypeHolder, _, _>(cx, &global, |cx| {
            rooted!(&in(cx) let resolve_func =
                create_native_handler_function(cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Resolve));

            rooted!(&in(cx) let reject_func =
                create_native_handler_function(cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Reject));

            unsafe {
                let ok = AddPromiseReactions(
                    cx,
                    self.promise_obj(),
                    resolve_func.handle(),
                    reject_func.handle(),
                );
                assert!(ok);
            }
        })
    }

    #[expect(unsafe_code)]
    pub(crate) fn get_promise_is_handled(&self) -> bool {
        unsafe { GetPromiseIsHandled(self.reflector().get_jsobject()) }
    }

    #[expect(unsafe_code)]
    pub(crate) fn set_promise_is_handled(&self, cx: &mut JSContext) -> bool {
        unsafe { SetAnyPromiseIsHandled(cx, self.reflector().get_jsobject()) }
    }
}

#[expect(unsafe_code)]
unsafe extern "C" fn do_nothing_promise_executor(
    _cx: *mut RawJSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = unsafe { CallArgs::from_vp(vp, argc) };
    args.rval().set(UndefinedValue());
    true
}

const SLOT_NATIVEHANDLER: usize = 0;
const SLOT_NATIVEHANDLER_TASK: usize = 1;

#[derive(PartialEq)]
enum NativeHandlerTask {
    Resolve = 0,
    Reject = 1,
}

#[expect(unsafe_code)]
unsafe extern "C" fn native_handler_callback(
    cx: *mut RawJSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
    let mut cx = CurrentRealm::assert(&mut cx);
    let cx = &mut cx;

    let args = unsafe { CallArgs::from_vp(vp, argc) };
    let native_handler_value =
        unsafe { *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER) };
    rooted!(&in(cx) let native_handler_value = native_handler_value);
    assert!(native_handler_value.get().is_object());

    let handler =
        unsafe { root_from_object::<PromiseNativeHandler>(cx, native_handler_value.to_object()) }
            .expect("unexpected value for native handler in promise native handler callback");

    let native_handler_task_value =
        unsafe { *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER_TASK) };
    rooted!(&in(cx) let native_handler_task_value = native_handler_task_value);
    match native_handler_task_value.to_int32() {
        native_handler_task_value
            if native_handler_task_value == NativeHandlerTask::Resolve as i32 =>
        {
            handler.resolved_callback(cx, unsafe { HandleValue::from_raw(args.get(0)) })
        },
        native_handler_task_value
            if native_handler_task_value == NativeHandlerTask::Reject as i32 =>
        {
            handler.rejected_callback(cx, unsafe { HandleValue::from_raw(args.get(0)) })
        },
        _ => panic!("unexpected native handler task value"),
    };

    true
}

#[expect(unsafe_code)]
fn create_native_handler_function(
    cx: &mut JSContext,
    holder: HandleObject,
    task: NativeHandlerTask,
) -> *mut JSObject {
    unsafe {
        let func = NewFunctionWithReserved(cx, Some(native_handler_callback), 1, 0, ptr::null());
        assert!(!func.is_null());

        rooted!(&in(cx) let obj = JS_GetFunctionObject(func));
        assert!(!obj.is_null());
        SetFunctionNativeReserved(obj.get(), SLOT_NATIVEHANDLER, &ObjectValue(*holder));
        SetFunctionNativeReserved(obj.get(), SLOT_NATIVEHANDLER_TASK, &Int32Value(task as i32));
        obj.get()
    }
}

impl FromJSValConvertibleRc for Promise {
    fn safe_from_jsval(
        cx: &mut JSContext,
        value: HandleValue,
    ) -> Result<ConversionResult<Rc<Promise>>, ()> {
        if value.get().is_null() {
            return Ok(ConversionResult::Failure(c"null not allowed".into()));
        }

        let mut realm = CurrentRealm::assert(cx);
        let global_scope = GlobalScope::from_current_realm(&mut realm);

        let promise = Promise::new_resolved(cx, &global_scope, value);
        Ok(ConversionResult::Success(promise))
    }
}

/// The success steps of <https://webidl.spec.whatwg.org/#wait-for-all>
type WaitForAllSuccessSteps = Rc<dyn Fn(&mut JSContext, Vec<HandleValue>)>;

/// The failure steps of <https://webidl.spec.whatwg.org/#wait-for-all>
type WaitForAllFailureSteps = Rc<dyn Fn(&mut JSContext, HandleValue)>;

/// The fulfillment handler for the list of promises in
/// <https://webidl.spec.whatwg.org/#wait-for-all>.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct WaitForAllFulfillmentHandler {
    /// The steps to call when all promises are resolved.
    #[ignore_malloc_size_of = "callbacks are hard"]
    #[no_trace]
    success_steps: WaitForAllSuccessSteps,

    /// The results of the promises.
    #[ignore_malloc_size_of = "mozjs"]
    #[expect(clippy::vec_box)]
    result: Rc<RefCell<Vec<Box<Heap<JSVal>>>>>,

    /// The index identifying which promise this handler is attached to.
    promise_index: usize,

    /// A count of fulfilled promises.
    #[conditional_malloc_size_of]
    fulfilled_count: Rc<RefCell<usize>>,
}

impl Callback for WaitForAllFulfillmentHandler {
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        // Let fulfillmentHandler be the following steps given arg:

        let equals_total = {
            // Set result[promiseIndex] to arg.
            let result = self.result.borrow_mut();
            result[self.promise_index].set(v.get());

            // Set fulfilledCount to fulfilledCount + 1.
            let mut fulfilled_count = self.fulfilled_count.borrow_mut();
            *fulfilled_count += 1;

            *fulfilled_count == result.len()
        };

        // If fulfilledCount equals total, then perform successSteps given result.
        if equals_total {
            let result_ref = self.result.borrow();
            let result_handles: Vec<HandleValue> =
                result_ref.iter().map(|v| v.as_handle_value()).collect();

            (self.success_steps)(cx, result_handles);
        }
    }
}

/// The rejection handler for the list of promises in
/// <https://webidl.spec.whatwg.org/#wait-for-all>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct WaitForAllRejectionHandler {
    /// The steps to call if any promise rejects.
    #[ignore_malloc_size_of = "callbacks are hard"]
    #[no_trace]
    failure_steps: WaitForAllFailureSteps,

    /// Whether any promises have been rejected already.
    rejected: Cell<bool>,
}

impl Callback for WaitForAllRejectionHandler {
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        // Let rejectionHandlerSteps be the following steps given arg:

        if self.rejected.replace(true) {
            // If rejected is true, abort these steps.
            return;
        }

        // Set rejected to true.
        // Done above with `replace`.
        (self.failure_steps)(cx, v);
    }
}

/// The microtask for performing successSteps given « » in
/// <https://webidl.spec.whatwg.org/#wait-for-all>.
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct WaitForAllSuccessStepsMicrotask {
    global: DomRoot<GlobalScope>,

    #[ignore_malloc_size_of = "Closure is hard"]
    #[no_trace]
    success_steps: WaitForAllSuccessSteps,
}

impl MicrotaskRunnable for WaitForAllSuccessStepsMicrotask {
    fn handler(&self, cx: &mut JSContext) {
        (self.success_steps)(cx, vec![]);
    }

    fn enter_realm<'cx>(&self, cx: &'cx mut JSContext) -> AutoRealm<'cx> {
        enter_auto_realm(cx, &*self.global)
    }
}

/// <https://webidl.spec.whatwg.org/#wait-for-all>
#[cfg_attr(crown, expect(crown::unrooted_must_root))]
fn wait_for_all(
    cx: &mut CurrentRealm,
    global: &GlobalScope,
    promises: Vec<Rc<Promise>>,
    success_steps: WaitForAllSuccessSteps,
    failure_steps: WaitForAllFailureSteps,
) {
    // Let fulfilledCount be 0.
    let fulfilled_count: Rc<RefCell<usize>> = Default::default();

    // Let rejected be false.
    // Note: done below when constructing a rejection handler.

    // Let rejectionHandlerSteps be the following steps given arg:
    // Note: implemented with the `WaitForAllRejectionHandler`.

    // Let rejectionHandler be CreateBuiltinFunction(rejectionHandlerSteps, « »):
    // Note: done as part of attaching the `WaitForAllRejectionHandler` as native rejection handler.
    let rejection_handler = WaitForAllRejectionHandler {
        failure_steps,
        rejected: Default::default(),
    };

    // Let total be promises’s size.
    // Note: done using the len of result.

    // If total is 0, then:
    if promises.is_empty() {
        // Queue a microtask to perform successSteps given « ».
        global.enqueue_microtask(
            cx,
            Microtask::WaitForAllSuccessSteps(WaitForAllSuccessStepsMicrotask {
                global: DomRoot::from_ref(global),
                success_steps,
            }),
        );

        // Return.
        return;
    }

    // Let index be 0.
    // Note: done with `enumerate` below.

    // Let result be a list containing total null values.
    let result: Rc<RefCell<Vec<Box<Heap<JSVal>>>>> = Default::default();

    // For each promise of promises:
    for (promise_index, promise) in promises.into_iter().enumerate() {
        let result = result.clone();

        {
            // Note: adding a null value for this promise result.
            let mut result_list = result.borrow_mut();
            rooted!(&in(cx) let null_value = NullValue());
            result_list.push(Heap::boxed(null_value.get()));
        }

        // Let promiseIndex be index.
        // Note: done with `enumerate` above.

        // Let fulfillmentHandler be the following steps given arg:
        // Note: implemented with the `WaitForAllFulFillmentHandler`.

        // Let fulfillmentHandler be CreateBuiltinFunction(fulfillmentHandler, « »):
        // Note: passed below to avoid the need to root it.

        // Perform PerformPromiseThen(promise, fulfillmentHandler, rejectionHandler).
        let handler = PromiseNativeHandler::new(
            cx,
            global,
            Some(Box::new(WaitForAllFulfillmentHandler {
                success_steps: success_steps.clone(),
                result,
                promise_index,
                fulfilled_count: fulfilled_count.clone(),
            })),
            Some(Box::new(rejection_handler.clone())),
        );
        promise.append_native_handler(cx, &handler);

        // Set index to index + 1.
        // Note: done above with `enumerate`.
    }
}

/// <https://webidl.spec.whatwg.org/#waiting-for-all-promise>
pub(crate) fn wait_for_all_promise(
    cx: &mut CurrentRealm,
    global: &GlobalScope,
    promises: Vec<Rc<Promise>>,
) -> Rc<Promise> {
    // Let promise be a new promise of type Promise<sequence<T>> in realm.
    let promise = Promise::new(cx, global);
    let success_promise = promise.clone();
    let failure_promise = promise.clone();

    // Let successSteps be the following steps, given results:
    let success_steps = Rc::new(move |cx: &mut JSContext, results: Vec<HandleValue>| {
        // Resolve promise with results.
        success_promise.resolve_native(cx, &results);
    });

    // Let failureSteps be the following steps, given reason:
    let failure_steps = Rc::new(move |cx: &mut JSContext, reason: HandleValue| {
        // Reject promise with reason.
        failure_promise.reject_native(cx, &reason);
    });

    // Wait for all with promises, given successSteps and failureSteps.
    wait_for_all(cx, global, promises, success_steps, failure_steps);

    // Return promise.
    promise
}
