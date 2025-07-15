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
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::{ConversionResult, FromJSValConvertibleRc};
use js::jsapi::{
    AddRawValueRoot, CallArgs, GetFunctionNativeReserved, Heap, JS_ClearPendingException,
    JS_GetFunctionObject, JS_NewFunction, JSAutoRealm, JSContext, JSObject,
    NewFunctionWithReserved, PromiseState, PromiseUserInputEventHandlingState, RemoveRawValueRoot,
    SetFunctionNativeReserved,
};
use js::jsval::{Int32Value, JSVal, NullValue, ObjectValue, UndefinedValue};
use js::rust::wrappers::{
    AddPromiseReactions, CallOriginalPromiseReject, CallOriginalPromiseResolve,
    GetPromiseIsHandled, GetPromiseState, IsPromiseObject, NewPromiseObject, RejectPromise,
    ResolvePromise, SetAnyPromiseIsHandled, SetPromiseUserInputEventHandlingState,
};
use js::rust::{HandleObject, HandleValue, MutableHandleObject, Runtime};
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::conversions::root_from_object;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{DomGlobal, DomObject, MutDomObject, Reflector};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
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
    fn initialize(&self, cx: SafeJSContext);
}

impl PromiseHelper for Rc<Promise> {
    #[allow(unsafe_code)]
    fn initialize(&self, cx: SafeJSContext) {
        let obj = self.reflector().get_jsobject();
        self.permanent_js_root.set(ObjectValue(*obj));
        unsafe {
            assert!(AddRawValueRoot(
                *cx,
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
    #[allow(unsafe_code)]
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
    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> Rc<Promise> {
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        Promise::new_in_current_realm(comp, can_gc)
    }

    pub(crate) fn new_in_current_realm(_comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut obj = ptr::null_mut::<JSObject>());
        Promise::create_js_promise(cx, obj.handle_mut(), can_gc);
        Promise::new_with_js_promise(obj.handle(), cx)
    }

    #[allow(unsafe_code)]
    pub(crate) fn duplicate(&self) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        Promise::new_with_js_promise(self.reflector().get_jsobject(), cx)
    }

    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_js_promise(obj: HandleObject, cx: SafeJSContext) -> Rc<Promise> {
        unsafe {
            assert!(IsPromiseObject(obj));
            let promise = Promise {
                reflector: Reflector::new(),
                permanent_js_root: Heap::default(),
            };
            let promise = Rc::new(promise);
            promise.init_reflector(obj.get());
            promise.initialize(cx);
            promise
        }
    }

    #[allow(unsafe_code)]
    // The apparently-unused CanGc parameter reflects the fact that the JS API calls
    // like JS_NewFunction can trigger a GC.
    fn create_js_promise(cx: SafeJSContext, mut obj: MutableHandleObject, _can_gc: CanGc) {
        unsafe {
            let do_nothing_func = JS_NewFunction(
                *cx,
                Some(do_nothing_promise_executor),
                /* nargs = */ 2,
                /* flags = */ 0,
                ptr::null(),
            );
            assert!(!do_nothing_func.is_null());
            rooted!(in(*cx) let do_nothing_obj = JS_GetFunctionObject(do_nothing_func));
            assert!(!do_nothing_obj.is_null());
            obj.set(NewPromiseObject(*cx, do_nothing_obj.handle()));
            assert!(!obj.is_null());
            let is_user_interacting = if ScriptThread::is_user_interacting() {
                PromiseUserInputEventHandlingState::HadUserInteractionAtCreation
            } else {
                PromiseUserInputEventHandlingState::DidntHaveUserInteractionAtCreation
            };
            SetPromiseUserInputEventHandlingState(obj.handle(), is_user_interacting);
        }
    }

    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_resolved(
        global: &GlobalScope,
        cx: SafeJSContext,
        value: impl SafeToJSValConvertible,
        _can_gc: CanGc,
    ) -> Rc<Promise> {
        let _ac = JSAutoRealm::new(*cx, global.reflector().get_jsobject().get());
        rooted!(in(*cx) let mut rval = UndefinedValue());
        value.safe_to_jsval(cx, rval.handle_mut());
        unsafe {
            rooted!(in(*cx) let p = CallOriginalPromiseResolve(*cx, rval.handle()));
            assert!(!p.handle().is_null());
            Promise::new_with_js_promise(p.handle(), cx)
        }
    }

    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_rejected(
        global: &GlobalScope,
        cx: SafeJSContext,
        value: impl SafeToJSValConvertible,
        _can_gc: CanGc,
    ) -> Rc<Promise> {
        let _ac = JSAutoRealm::new(*cx, global.reflector().get_jsobject().get());
        rooted!(in(*cx) let mut rval = UndefinedValue());
        value.safe_to_jsval(cx, rval.handle_mut());
        unsafe {
            rooted!(in(*cx) let p = CallOriginalPromiseReject(*cx, rval.handle()));
            assert!(!p.handle().is_null());
            Promise::new_with_js_promise(p.handle(), cx)
        }
    }

    pub(crate) fn resolve_native<T>(&self, val: &T, can_gc: CanGc)
    where
        T: SafeToJSValConvertible,
    {
        let cx = GlobalScope::get_cx();
        let _ac = enter_realm(self);
        rooted!(in(*cx) let mut v = UndefinedValue());
        val.safe_to_jsval(cx, v.handle_mut());
        self.resolve(cx, v.handle(), can_gc);
    }

    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn resolve(&self, cx: SafeJSContext, value: HandleValue, _can_gc: CanGc) {
        unsafe {
            if !ResolvePromise(*cx, self.promise_obj(), value) {
                JS_ClearPendingException(*cx);
            }
        }
    }

    pub(crate) fn reject_native<T>(&self, val: &T, can_gc: CanGc)
    where
        T: SafeToJSValConvertible,
    {
        let cx = GlobalScope::get_cx();
        let _ac = enter_realm(self);
        rooted!(in(*cx) let mut v = UndefinedValue());
        val.safe_to_jsval(cx, v.handle_mut());
        self.reject(cx, v.handle(), can_gc);
    }

    pub(crate) fn reject_error(&self, error: Error, can_gc: CanGc) {
        let cx = GlobalScope::get_cx();
        let _ac = enter_realm(self);
        rooted!(in(*cx) let mut v = UndefinedValue());
        error.to_jsval(cx, &self.global(), v.handle_mut(), can_gc);
        self.reject(cx, v.handle(), can_gc);
    }

    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn reject(&self, cx: SafeJSContext, value: HandleValue, _can_gc: CanGc) {
        unsafe {
            if !RejectPromise(*cx, self.promise_obj(), value) {
                JS_ClearPendingException(*cx);
            }
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn is_fulfilled(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Rejected | PromiseState::Fulfilled)
    }

    #[allow(unsafe_code)]
    pub(crate) fn is_rejected(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Rejected)
    }

    #[allow(unsafe_code)]
    pub(crate) fn is_pending(&self) -> bool {
        let state = unsafe { GetPromiseState(self.promise_obj()) };
        matches!(state, PromiseState::Pending)
    }

    #[allow(unsafe_code)]
    pub(crate) fn promise_obj(&self) -> HandleObject {
        let obj = self.reflector().get_jsobject();
        unsafe {
            assert!(IsPromiseObject(obj));
        }
        obj
    }

    #[allow(unsafe_code)]
    pub(crate) fn append_native_handler(
        &self,
        handler: &PromiseNativeHandler,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        let _ais = AutoEntryScript::new(&handler.global_(realm));
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let resolve_func =
                create_native_handler_function(*cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Resolve,
                                               can_gc));

        rooted!(in(*cx) let reject_func =
                create_native_handler_function(*cx,
                                               handler.reflector().get_jsobject(),
                                               NativeHandlerTask::Reject,
                                               can_gc));

        unsafe {
            let ok = AddPromiseReactions(
                *cx,
                self.promise_obj(),
                resolve_func.handle(),
                reject_func.handle(),
            );
            assert!(ok);
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_promise_is_handled(&self) -> bool {
        unsafe { GetPromiseIsHandled(self.reflector().get_jsobject()) }
    }

    #[allow(unsafe_code)]
    pub(crate) fn set_promise_is_handled(&self) -> bool {
        let cx = GlobalScope::get_cx();
        unsafe { SetAnyPromiseIsHandled(*cx, self.reflector().get_jsobject()) }
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn do_nothing_promise_executor(
    _cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);
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

#[allow(unsafe_code)]
unsafe extern "C" fn native_handler_callback(
    cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let cx = SafeJSContext::from_ptr(cx);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);

    let args = CallArgs::from_vp(vp, argc);
    rooted!(in(*cx) let v = *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER));
    assert!(v.get().is_object());

    let handler = root_from_object::<PromiseNativeHandler>(v.to_object(), *cx)
        .expect("unexpected value for native handler in promise native handler callback");

    rooted!(in(*cx) let v = *GetFunctionNativeReserved(args.callee(), SLOT_NATIVEHANDLER_TASK));
    match v.to_int32() {
        v if v == NativeHandlerTask::Resolve as i32 => handler.resolved_callback(
            *cx,
            HandleValue::from_raw(args.get(0)),
            InRealm::Already(&in_realm_proof),
            CanGc::note(),
        ),
        v if v == NativeHandlerTask::Reject as i32 => handler.rejected_callback(
            *cx,
            HandleValue::from_raw(args.get(0)),
            InRealm::Already(&in_realm_proof),
            CanGc::note(),
        ),
        _ => panic!("unexpected native handler task value"),
    };

    true
}

#[allow(unsafe_code)]
// The apparently-unused CanGc argument reflects the fact that the JS API calls
// like NewFunctionWithReserved can trigger a GC.
fn create_native_handler_function(
    cx: *mut JSContext,
    holder: HandleObject,
    task: NativeHandlerTask,
    _can_gc: CanGc,
) -> *mut JSObject {
    unsafe {
        let func = NewFunctionWithReserved(cx, Some(native_handler_callback), 1, 0, ptr::null());
        assert!(!func.is_null());

        rooted!(in(cx) let obj = JS_GetFunctionObject(func));
        assert!(!obj.is_null());
        SetFunctionNativeReserved(obj.get(), SLOT_NATIVEHANDLER, &ObjectValue(*holder));
        SetFunctionNativeReserved(obj.get(), SLOT_NATIVEHANDLER_TASK, &Int32Value(task as i32));
        obj.get()
    }
}

impl FromJSValConvertibleRc for Promise {
    #[allow(unsafe_code)]
    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
    ) -> Result<ConversionResult<Rc<Promise>>, ()> {
        if value.get().is_null() {
            return Ok(ConversionResult::Failure("null not allowed".into()));
        }

        let cx = SafeJSContext::from_ptr(cx);
        let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
        let global_scope = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));

        let promise = Promise::new_resolved(&global_scope, cx, value, CanGc::note());
        Ok(ConversionResult::Success(promise))
    }
}

/// The success steps of <https://webidl.spec.whatwg.org/#wait-for-all>
type WaitForAllSuccessSteps = Rc<dyn Fn(Vec<HandleValue>)>;

/// The failure steps of <https://webidl.spec.whatwg.org/#wait-for-all>
type WaitForAllFailureSteps = Rc<dyn Fn(HandleValue)>;

/// The fulfillment handler for the list of promises in
/// <https://webidl.spec.whatwg.org/#wait-for-all>.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct WaitForAllFulfillmentHandler {
    /// The steps to call when all promises are resolved.
    #[ignore_malloc_size_of = "Rc is hard"]
    #[no_trace]
    success_steps: WaitForAllSuccessSteps,

    /// The results of the promises.
    #[ignore_malloc_size_of = "Rc is hard"]
    #[allow(clippy::vec_box)]
    result: Rc<RefCell<Vec<Box<Heap<JSVal>>>>>,

    /// The index identifying which promise this handler is attached to.
    promise_index: usize,

    /// A count of fulfilled promises.
    #[ignore_malloc_size_of = "Rc is hard"]
    fulfilled_count: Rc<RefCell<usize>>,
}

impl Callback for WaitForAllFulfillmentHandler {
    #[allow(unsafe_code)]
    fn callback(&self, _cx: SafeJSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
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
            // Safety: the values are kept alive by the Heap
            // while their handles are passed to the the success steps.
            let result_handles: Vec<HandleValue> = unsafe {
                self.result
                    .borrow()
                    .iter()
                    .map(|val| HandleValue::from_raw(val.handle()))
                    .collect()
            };
            (self.success_steps)(result_handles);
        }
    }
}

/// The rejection handler for the list of promises in
/// <https://webidl.spec.whatwg.org/#wait-for-all>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct WaitForAllRejectionHandler {
    /// The steps to call if any promise rejects.
    #[ignore_malloc_size_of = "Rc is hard"]
    #[no_trace]
    failure_steps: WaitForAllFailureSteps,

    /// Whether any promises have been rejected already.
    rejected: Cell<bool>,
}

impl Callback for WaitForAllRejectionHandler {
    fn callback(&self, _cx: SafeJSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        // Let rejectionHandlerSteps be the following steps given arg:

        if self.rejected.replace(true) {
            // If rejected is true, abort these steps.
            return;
        }

        // Set rejected to true.
        // Done above with `replace`.
        (self.failure_steps)(v);
    }
}

/// <https://webidl.spec.whatwg.org/#wait-for-all>
pub(crate) fn wait_for_all(
    cx: SafeJSContext,
    global: &GlobalScope,
    promises: Vec<Rc<Promise>>,
    success_steps: WaitForAllSuccessSteps,
    failure_steps: WaitForAllFailureSteps,
    realm: InRealm,
    can_gc: CanGc,
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
    // Queue a microtask to perform successSteps given « ».
    // TODO: #37259

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
            rooted!(in(*cx) let null_value = NullValue());
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
            global,
            Some(Box::new(WaitForAllFulfillmentHandler {
                success_steps: success_steps.clone(),
                result,
                promise_index,
                fulfilled_count: fulfilled_count.clone(),
            })),
            Some(Box::new(rejection_handler.clone())),
            can_gc,
        );
        promise.append_native_handler(&handler, realm, can_gc);

        // Set index to index + 1.
        // Note: done above with `enumerate`.
    }
}

/// <https://webidl.spec.whatwg.org/#waiting-for-all-promise>
pub(crate) fn wait_for_all_promise(
    cx: SafeJSContext,
    global: &GlobalScope,
    promises: Vec<Rc<Promise>>,
    realm: InRealm,
    can_gc: CanGc,
) -> Rc<Promise> {
    // Let promise be a new promise of type Promise<sequence<T>> in realm.
    let promise = Promise::new(global, can_gc);
    let success_promise = promise.clone();
    let failure_promise = promise.clone();

    // Let successSteps be the following steps, given results:
    let success_steps = Rc::new(move |results: Vec<HandleValue>| {
        // Resolve promise with results.
        success_promise.resolve_native(&results, can_gc);
    });

    // Let failureSteps be the following steps, given reason:
    let failure_steps = Rc::new(move |reason: HandleValue| {
        // Reject promise with reason.
        failure_promise.reject_native(&reason, can_gc);
    });

    if promises.is_empty() {
        // Note: part of `wait_for_all`.
        // Done here by using `resolve_native`.
        // TODO: #37259
        // If total is 0, then:
        // Queue a microtask to perform successSteps given « ».
        let empty_list: Vec<HandleValue> = vec![];
        promise.resolve_native(&empty_list, can_gc);
    } else {
        // Wait for all with promises, given successSteps and failureSteps.
        wait_for_all(
            cx,
            global,
            promises,
            success_steps,
            failure_steps,
            realm,
            can_gc,
        );
    }

    // Return promise.
    promise
}
