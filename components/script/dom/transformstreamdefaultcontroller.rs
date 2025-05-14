/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{
    ExceptionStackBehavior, Heap, JS_IsExceptionPending, JS_SetPendingException, JSObject,
};
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use super::bindings::cell::DomRefCell;
use super::bindings::codegen::Bindings::TransformerBinding::{
    Transformer, TransformerCancelCallback, TransformerFlushCallback, TransformerTransformCallback,
};
use super::types::TransformStream;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

impl js::gc::Rootable for TransformTransformPromiseRejection {}

/// Reacting to transformPromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-controller-perform-transform>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TransformTransformPromiseRejection {
    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for TransformTransformPromiseRejection {
    /// Reacting to transformPromise with the following fulfillment steps:
    #[allow(unsafe_code)]
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Perform ! TransformStreamError(controller.[[stream]], r).
        self.controller
            .error(cx, &self.controller.global(), v, can_gc);

        // Throw r.
        // Note: this is done part of perform_transform().
    }
}

/// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller>
#[dom_struct]
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
    transform_obj: Heap<*mut JSObject>,

    /// <https://streams.spec.whatwg.org/#TransformStreamDefaultController-stream>
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
            cancel: RefCell::new(transformer.cancel.clone()),
            flush: RefCell::new(transformer.flush.clone()),
            transform: RefCell::new(transformer.transform.clone()),
            finish_promise: DomRefCell::new(None),
            stream: MutNullableDom::new(None),
            transform_obj: Default::default(),
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
    pub(crate) fn set_transform_obj(&self, this_object: SafeHandleObject) {
        self.transform_obj.set(*this_object);
    }

    pub(crate) fn set_stream(&self, stream: &TransformStream) {
        self.stream.set(Some(stream));
    }

    pub(crate) fn get_finish_promise(&self) -> Option<Rc<Promise>> {
        self.finish_promise.borrow().clone()
    }

    pub(crate) fn set_finish_promise(&self, promise: Rc<Promise>) {
        *self.finish_promise.borrow_mut() = Some(promise);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-perform-transform>
    pub(crate) fn transform_stream_default_controller_perform_transform(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let transformPromise be the result of performing controller.[[transformAlgorithm]], passing chunk.
        let transform_promise = self.perform_transform(cx, global, chunk, can_gc)?;

        // Return the result of reacting to transformPromise with the following rejection steps given the argument r:
        rooted!(in(*cx) let mut reject_handler = Some(TransformTransformPromiseRejection {
            controller: Dom::from_ref(self),
        }));

        let handler = PromiseNativeHandler::new(
            global,
            None,
            reject_handler.take().map(|h| Box::new(h) as Box<_>),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        transform_promise.append_native_handler(&handler, comp, can_gc);

        Ok(transform_promise)
    }

    pub(crate) fn perform_transform(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // If transformerDict["transform"] exists, set transformAlgorithm to an algorithm which
        // takes an argument chunk and returns the result of invoking transformerDict["transform"] with argument list
        // « chunk, controller » and callback this value transformer.
        let algo = self.transform.borrow().clone();
        let result = if let Some(transform) = algo {
            rooted!(in(*cx) let this_object = self.transform_obj.get());
            let call_result = transform.Call_(
                &this_object.handle(),
                chunk,
                self,
                ExceptionHandling::Rethrow,
                can_gc,
            );
            match call_result {
                Ok(p) => p,
                Err(e) => {
                    let p = Promise::new(global, can_gc);
                    p.reject_error(e, can_gc);
                    p
                },
            }
        } else {
            // Let transformAlgorithm be the following steps, taking a chunk argument:
            // Let result be TransformStreamDefaultControllerEnqueue(controller, chunk).
            // If result is an abrupt completion, return a promise rejected with result.[[Value]].
            let promise = if let Err(error) = self.enqueue(cx, global, chunk, can_gc) {
                rooted!(in(*cx) let mut error_val = UndefinedValue());
                error.to_jsval(cx, global, error_val.handle_mut(), can_gc);
                Promise::new_rejected(global, cx, error_val.handle(), can_gc)
            } else {
                // Otherwise, return a promise resolved with undefined.
                Promise::new_resolved(global, cx, (), can_gc)
            };

            promise
        };

        Ok(result)
    }

    pub(crate) fn perform_cancel(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // If transformerDict["cancel"] exists, set cancelAlgorithm to an algorithm which takes an argument
        // reason and returns the result of invoking transformerDict["cancel"] with argument list « reason »
        // and callback this value transformer.
        let algo = self.cancel.borrow().clone();
        let result = if let Some(cancel) = algo {
            rooted!(in(*cx) let this_object = self.transform_obj.get());
            let call_result = cancel.Call_(
                &this_object.handle(),
                chunk,
                ExceptionHandling::Rethrow,
                can_gc,
            );
            match call_result {
                Ok(p) => p,
                Err(e) => {
                    let p = Promise::new(global, can_gc);
                    p.reject_error(e, can_gc);
                    p
                },
            }
        } else {
            // Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.
            Promise::new_resolved(global, cx, (), can_gc)
        };

        Ok(result)
    }

    pub(crate) fn perform_flush(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // If transformerDict["flush"] exists, set flushAlgorithm to an algorithm which returns the result of
        // invoking transformerDict["flush"] with argument list « controller » and callback this value transformer.
        let algo = self.flush.borrow().clone();
        let result = if let Some(flush) = algo {
            rooted!(in(*cx) let this_object = self.transform_obj.get());
            let call_result = flush.Call_(
                &this_object.handle(),
                self,
                ExceptionHandling::Rethrow,
                can_gc,
            );
            match call_result {
                Ok(p) => p,
                Err(e) => {
                    let p = Promise::new(global, can_gc);
                    p.reject_error(e, can_gc);
                    p
                },
            }
        } else {
            // Let flushAlgorithm be an algorithm which returns a promise resolved with undefined.
            Promise::new_resolved(global, cx, (), can_gc)
        };

        Ok(result)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-enqueue>
    #[allow(unsafe_code)]
    pub(crate) fn enqueue(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().expect("stream is null");

        // Let readableController be stream.[[readable]].[[controller]].
        let readable = stream.get_readable();
        let readable_controller = readable.get_default_controller();

        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(readableController)
        // is false, throw a TypeError exception.
        if !readable_controller.can_close_or_enqueue() {
            return Err(Error::Type(
                "ReadableStreamDefaultControllerCanCloseOrEnqueue is false".to_owned(),
            ));
        }

        // Let enqueueResult be ReadableStreamDefaultControllerEnqueue(readableController, chunk).
        // If enqueueResult is an abrupt completion,
        if let Err(error) = readable_controller.enqueue(cx, chunk, can_gc) {
            // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, enqueueResult.[[Value]]).
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            error
                .clone()
                .to_jsval(cx, global, rooted_error.handle_mut(), can_gc);
            stream.error_writable_and_unblock_write(cx, global, rooted_error.handle(), can_gc);

            // Throw stream.[[readable]].[[storedError]].
            unsafe {
                if !JS_IsExceptionPending(*cx) {
                    rooted!(in(*cx) let mut stored_error = UndefinedValue());
                    readable.get_stored_error(stored_error.handle_mut());

                    JS_SetPendingException(
                        *cx,
                        stored_error.handle().into(),
                        ExceptionStackBehavior::Capture,
                    );
                }
            }
            return Err(error);
        }

        // Let backpressure be ! ReadableStreamDefaultControllerHasBackpressure(readableController).
        let backpressure = readable_controller.has_backpressure();

        // If backpressure is not stream.[[backpressure]],
        if backpressure != stream.get_backpressure() {
            // Assert: backpressure is true.
            assert!(backpressure);

            // Perform ! TransformStreamSetBackpressure(stream, true).
            stream.set_backpressure(global, true, can_gc);
        }
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-error>
    pub(crate) fn error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) {
        // Perform ! TransformStreamError(controller.[[stream]], e).
        self.stream
            .get()
            .expect("stream is undefined")
            .error(cx, global, reason, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-clear-algorithms>
    pub(crate) fn clear_algorithms(&self) {
        // Set controller.[[transformAlgorithm]] to undefined.
        self.transform.replace(None);

        // Set controller.[[flushAlgorithm]] to undefined.
        self.flush.replace(None);

        // Set controller.[[cancelAlgorithm]] to undefined.
        self.cancel.replace(None);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-terminate>
    pub(crate) fn terminate(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().expect("stream is null");

        // Let readableController be stream.[[readable]].[[controller]].
        let readable = stream.get_readable();
        let readable_controller = readable.get_default_controller();

        // Perform ! ReadableStreamDefaultControllerClose(readableController).
        readable_controller.close(can_gc);

        // Let error be a TypeError exception indicating that the stream has been terminated.
        let error = Error::Type("stream has been terminated".to_owned());

        // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, error).
        rooted!(in(*cx) let mut rooted_error = UndefinedValue());
        error.to_jsval(cx, global, rooted_error.handle_mut(), can_gc);
        stream.error_writable_and_unblock_write(cx, global, rooted_error.handle(), can_gc);
    }
}

impl TransformStreamDefaultControllerMethods<crate::DomTypeHolder>
    for TransformStreamDefaultController
{
    /// <https://streams.spec.whatwg.org/#ts-default-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // Let readableController be this.[[stream]].[[readable]].[[controller]].
        let readable_controller = self
            .stream
            .get()
            .expect("stream is null")
            .get_readable()
            .get_default_controller();

        // Return ! ReadableStreamDefaultControllerGetDesiredSize(readableController).
        readable_controller.get_desired_size()
    }

    /// <https://streams.spec.whatwg.org/#ts-default-controller-enqueue>
    fn Enqueue(&self, cx: SafeJSContext, chunk: SafeHandleValue, can_gc: CanGc) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerEnqueue(this, chunk).
        self.enqueue(cx, &self.global(), chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#ts-default-controller-error>
    fn Error(&self, cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerError(this, e).
        self.error(cx, &self.global(), reason, can_gc);
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#ts-default-controller-terminate>
    fn Terminate(&self, can_gc: CanGc) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerTerminate(this).
        self.terminate(GlobalScope::get_cx(), &self.global(), can_gc);
        Ok(())
    }
}
