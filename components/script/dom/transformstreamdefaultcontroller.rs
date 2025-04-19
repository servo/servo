/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{ExceptionStackBehavior, Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::JS_SetPendingException;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};
use script_bindings::codegen::GenericBindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;
use script_bindings::root::Dom;

use super::bindings::codegen::Bindings::TransformerBinding::{
    Transformer, TransformerCancelCallback, TransformerFlushCallback, TransformerTransformCallback,
};
use super::bindings::reflector::reflect_dom_object;
use super::types::TransformStream;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The rejection handler for the reacting to transformPromise part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-controller-perform-transform>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct SourceTransformPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    stream: Dom<TransformStream>,
}

impl Callback for SourceTransformPromiseRejectionHandler {
    /// The rejection handler for the reacting to transformPromise part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-perform-transform>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    #[allow(unsafe_code)]
    fn callback(&self, cx: SafeJSContext, reason: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Perform ! TransformStreamError(controller.[[stream]], r).
        let _ = self.stream.error(cx, &self.stream.global(), reason, can_gc);

        // Throw r.
        unsafe { JS_SetPendingException(*cx, reason, ExceptionStackBehavior::Capture) };
    }
}

#[dom_struct]
/// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller>
pub struct TransformStreamDefaultController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-cancelalgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    cancel: RefCell<Option<Rc<TransformerCancelCallback>>>,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-flushalgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    flush: RefCell<Option<Rc<TransformerFlushCallback>>>,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-transformalgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    transform: RefCell<Option<Rc<TransformerTransformCallback>>>,

    /// The JS object used as `this` when invoking sink algorithms.
    #[ignore_malloc_size_of = "mozjs"]
    transformer_obj: Heap<*mut JSObject>,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-stream>
    stream: MutNullableDom<TransformStream>,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-finishpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    finish_promise: DomRefCell<Option<Rc<Promise>>>,
}

impl TransformStreamDefaultController {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(transformer: &Transformer) -> TransformStreamDefaultController {
        TransformStreamDefaultController {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            flush: RefCell::new(transformer.flush.clone()),
            cancel: RefCell::new(transformer.cancel.clone()),
            transform: RefCell::new(transformer.transform.clone()),
            transformer_obj: Default::default(),
            finish_promise: DomRefCell::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        transformer: &Transformer,
        can_gc: CanGc,
    ) -> DomRoot<TransformStreamDefaultController> {
        reflect_dom_object(
            Box::new(TransformStreamDefaultController::new_inherited(transformer)),
            global,
            can_gc,
        )
    }

    /// Setting the JS object after the heap has settled down.
    pub(crate) fn set_transformer_this_object(&self, this_object: SafeHandleObject) {
        self.transformer_obj.set(*this_object);
    }

    pub(crate) fn get_finish_promise(&self) -> Option<Rc<Promise>> {
        self.finish_promise.borrow().clone()
    }

    pub(crate) fn set_finish_promise(&self, promise: Rc<Promise>) {
        *self.finish_promise.borrow_mut() = Some(promise);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-enqueue>
    #[allow(unsafe_code)]
    fn enqueue(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // Let readableController be stream.[[readable]].[[controller]].
        let readable_controller = stream.readable().get_default_controller();

        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(readableController) is false
        // , throw a TypeError exception.
        if !readable_controller.can_close_or_enqueue() {
            return Err(Error::Type(
                "ReadableStreamDefaultControllerCanCloseOrEnqueue is false".to_string(),
            ));
        }

        // Let enqueueResult be ReadableStreamDefaultControllerEnqueue(readableController, chunk).
        // If enqueueResult is an abrupt completion,
        if let Err(error) = readable_controller.enqueue(cx, chunk, can_gc) {
            // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, enqueueResult.[[Value]]).
            rooted!(in(*cx) let mut rval = UndefinedValue());
            error
                .clone()
                .to_jsval(cx, global, rval.handle_mut(), can_gc);

            stream.error_writable_and_unblock_write(global, cx, rval.handle(), can_gc)?;

            // Throw stream.[[readable]].[[storedError]].
            let readable = stream.readable();
            rooted!(in(*cx) let mut error = UndefinedValue());
            readable.get_stored_error(error.handle_mut());
            unsafe { JS_SetPendingException(*cx, error.handle(), ExceptionStackBehavior::Capture) };
            return Err(Error::JSFailed);
        }

        // Let backpressure be ! ReadableStreamDefaultControllerHasBackpressure(readableController).
        let backpressure = readable_controller.has_backpressure();

        // If backpressure is not stream.[[backpressure]],
        if Some(backpressure) != stream.backpressure() {
            // Assert: backpressure is true.
            assert!(backpressure);

            // Perform ! TransformStreamSetBackpressure(stream, true).
            stream.set_backpressure(global, true, can_gc)?;
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-clear-algorithms>
    pub(crate) fn clear_algorithms(&self) {
        // Set controller.[[transformAlgorithm]] to undefined.
        self.flush.borrow_mut().take();
        // Set controller.[[flushAlgorithm]] to undefined.
        self.transform.borrow_mut().take();
        // Set controller.[[cancelAlgorithm]] to undefined.
        self.cancel.borrow_mut().take();
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-error>
    pub(crate) fn error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Perform ! TransformStreamError(controller.[[stream]], e).
        self.stream
            .get()
            .ok_or(Error::Type(
                "TransformStreamDefaultController stream is null".to_string(),
            ))?
            .error(cx, global, reason, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-terminate>
    pub(crate) fn terminate(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // Let readableController be stream.[[readable]].[[controller]].
        let readable_controller = stream.readable().get_default_controller();

        // Perform ! ReadableStreamDefaultControllerClose(readableController).
        readable_controller.close(can_gc);

        // Let error be a TypeError exception indicating that the stream has been terminated.
        rooted!(in(*cx) let mut error = UndefinedValue());
        Error::Type("TransformStreamDefaultController stream is terminated".to_string()).to_jsval(
            cx,
            global,
            error.handle_mut(),
            can_gc,
        );

        // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, error).
        stream.error_writable_and_unblock_write(global, cx, error.handle(), can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-perform-transform>
    #[allow(unsafe_code)]
    pub(crate) fn perform_transform(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let transformPromise be the result of performing controller.[[transformAlgorithm]], passing chunk.
        let transform_promise = self.call_transform(cx, global, chunk, can_gc)?;

        let stream = self.stream.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // Return the result of reacting to transformPromise with the following rejection steps given the argument r:;
        let handler = PromiseNativeHandler::new(
            global,
            None,
            Some(Box::new(SourceTransformPromiseRejectionHandler {
                stream: Dom::from_ref(&stream),
            })),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        transform_promise.append_native_handler(&handler, comp, can_gc);

        Ok(transform_promise)
    }

    /// call to transform function
    #[allow(unsafe_code)]
    pub(crate) fn call_transform(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let transformPromise be the result of performing controller.[[transformAlgorithm]], passing chunk.
        let algo = self.transform.borrow().clone();

        let transform_promise = if let Some(transform) = algo {
            // If transformerDict["transform"] exists, set transformAlgorithm to an algorithm
            //  which takes an argument chunk and returns
            // the result of invoking transformerDict["transform"] with argument
            // list « chunk, controller » and callback this value transformer.
            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = self.transformer_obj.get());
            transform.Call_(
                &this_object.handle(),
                chunk,
                self,
                ExceptionHandling::Rethrow,
                can_gc,
            )?;
            let is_promise = unsafe {
                if result.is_object() {
                    result_object.set(result.to_object());
                    IsPromiseObject(result_object.handle().into_handle())
                } else {
                    false
                }
            };
            if is_promise {
                let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                promise
            } else {
                Promise::new_resolved(global, cx, result.get(), can_gc)
            }
        } else {
            // Let transformAlgorithm be the following steps, taking a chunk argument:

            // Let result be TransformStreamDefaultControllerEnqueue(controller, chunk).
            // If result is an abrupt completion, return a promise rejected with result.[[Value]].
            if let Err(error) = self.enqueue(cx, global, chunk, can_gc) {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                error
                    .clone()
                    .to_jsval(cx, global, rval.handle_mut(), can_gc);
                return Ok(Promise::new_rejected(global, cx, rval.handle(), can_gc));
            }

            // Otherwise, return a promise resolved with undefined.
            rooted!(in(*cx) let mut rval = UndefinedValue());
            Promise::new_resolved(global, cx, rval.handle(), can_gc)
        };
        Ok(transform_promise)
    }

    /// call to cancel function
    #[allow(unsafe_code)]
    pub(crate) fn call_cancel(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let cancelPromise be the result of performing controller.[[cancelAlgorithm]], passing reason.
        let algo = self.cancel.borrow().clone();
        let cancel_promise = if let Some(cancel) = algo {
            // If transformerDict["cancel"] exists, set cancelAlgorithm to an algorithm which takes an
            // argument reason and returns the result of invoking transformerDict["cancel"] with argument
            // list « reason » and callback this value transformer.

            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = self.transformer_obj.get());
            cancel.Call_(
                &this_object.handle(),
                reason,
                ExceptionHandling::Rethrow,
                can_gc,
            )?;
            let is_promise = unsafe {
                if result.is_object() {
                    result_object.set(result.to_object());
                    IsPromiseObject(result_object.handle().into_handle())
                } else {
                    false
                }
            };
            if is_promise {
                let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                promise
            } else {
                Promise::new_resolved(global, cx, result.get(), can_gc)
            }
        } else {
            // Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.
            Promise::new_resolved(global, cx, (), can_gc)
        };
        Ok(cancel_promise)
    }

    /// call to flush function
    #[allow(unsafe_code)]
    pub(crate) fn call_flush(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let flushPromise be the result of performing controller.[[flushAlgorithm]].
        let algo = self.flush.borrow().clone();
        let flush_promise = if let Some(flush) = algo {
            // If transformerDict["flush"] exists, set flushAlgorithm to
            // an algorithm which returns the result of
            // invoking transformerDict["flush"] with argument list
            // « controller » and callback this value transformer.

            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = self.transformer_obj.get());
            flush.Call_(
                &this_object.handle(),
                self,
                ExceptionHandling::Rethrow,
                can_gc,
            )?;
            let is_promise = unsafe {
                if result.is_object() {
                    result_object.set(result.to_object());
                    IsPromiseObject(result_object.handle().into_handle())
                } else {
                    false
                }
            };
            if is_promise {
                let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                promise
            } else {
                Promise::new_resolved(global, cx, result.get(), can_gc)
            }
        } else {
            // Let flushAlgorithm be an algorithm which returns a promise resolved with undefined.
            Promise::new_resolved(global, cx, (), can_gc)
        };
        Ok(flush_promise)
    }

    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller>
    pub(crate) fn set_up(&self, stream: &TransformStream) -> Fallible<()> {
        // Assert: stream implements TransformStream.
        // Assert: stream.[[controller]] is undefined.
        stream.assert_no_controller();

        // Set controller.[[stream]] to stream.
        self.stream.set(Some(stream));

        // Set stream.[[controller]] to controller.
        stream.set_default_controller(self);

        // Set controller.[[transformAlgorithm]] to transformAlgorithm.

        // Set controller.[[flushAlgorithm]] to flushAlgorithm.

        // Set controller.[[cancelAlgorithm]] to cancelAlgorithm.
        // Note: above steps are done in `new_inherited`.

        Ok(())
    }
}

impl TransformStreamDefaultControllerMethods<crate::DomTypeHolder>
    for TransformStreamDefaultController
{
    fn GetDesiredSize(&self) -> Option<f64> {
        // Let readableController be this.[[stream]].[[readable]].[[controller]].
        let readable_controller = self.stream.get()?.readable().get_default_controller();

        // Return ! ReadableStreamDefaultControllerGetDesiredSize(readableController).
        readable_controller.get_desired_size()
    }

    fn Enqueue(
        &self,
        cx: SafeJSContext,
        chunk: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerEnqueue(this, chunk).
        self.enqueue(cx, &self.global(), chunk, can_gc)
    }

    fn Error(&self, cx: SafeJSContext, reason: js::gc::HandleValue, can_gc: CanGc) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerError(this, e).
        self.error(cx, &self.global(), reason, can_gc)
    }

    fn Terminate(&self, can_gc: CanGc) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        let global = self.global();
        // Perform ? TransformStreamDefaultControllerTerminate(this).
        self.terminate(cx, &global, can_gc)
    }
}
