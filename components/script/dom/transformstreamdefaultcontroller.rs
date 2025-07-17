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
    TransformerCancelCallback, TransformerFlushCallback, TransformerTransformCallback,
};
use super::types::TransformStream;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::TransformerBinding::Transformer;
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::textdecodercommon::TextDecoderCommon;
use crate::dom::textdecoderstream::{decode_and_enqueue_a_chunk, flush_and_enqueue};
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

/// The type of transformer algorithms we are using
#[derive(JSTraceable)]
pub(crate) enum TransformerType {
    /// Algorithms provided by Js callbacks
    Js {
        /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-cancelalgorithm>
        cancel: RefCell<Option<Rc<TransformerCancelCallback>>>,

        /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-flushalgorithm>
        flush: RefCell<Option<Rc<TransformerFlushCallback>>>,

        /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-transformalgorithm>
        transform: RefCell<Option<Rc<TransformerTransformCallback>>>,

        /// The JS object used as `this` when invoking sink algorithms.
        transform_obj: Heap<*mut JSObject>,
    },
    /// Algorithms supporting `TextDecoderStream` are implemented in Rust
    ///
    /// <https://encoding.spec.whatwg.org/#textdecodercommon>
    Decoder(Rc<TextDecoderCommon>),
}

impl TransformerType {
    pub(crate) fn new_from_transformer(transformer: &Transformer) -> TransformerType {
        TransformerType::new_js(
            transformer.cancel.clone(),
            transformer.flush.clone(),
            transformer.transform.clone(),
        )
    }

    pub(crate) fn new_js(
        cancel: Option<Rc<TransformerCancelCallback>>,
        flush: Option<Rc<TransformerFlushCallback>>,
        transform: Option<Rc<TransformerTransformCallback>>,
    ) -> TransformerType {
        TransformerType::Js {
            cancel: RefCell::new(cancel),
            flush: RefCell::new(flush),
            transform: RefCell::new(transform),
            transform_obj: Default::default(),
        }
    }
}

/// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller>
#[dom_struct]
pub struct TransformStreamDefaultController {
    reflector_: Reflector,

    /// The type of the underlying transformer used. Besides the JS variant,
    /// there will be other variant(s) for `TextDecoderStream`
    #[ignore_malloc_size_of = "transformer_type"]
    transformer_type: TransformerType,

    /// <https://streams.spec.whatwg.org/#TransformStreamDefaultController-stream>
    stream: MutNullableDom<TransformStream>,

    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-finishpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    finish_promise: DomRefCell<Option<Rc<Promise>>>,
}

impl TransformStreamDefaultController {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(transformer_type: TransformerType) -> TransformStreamDefaultController {
        TransformStreamDefaultController {
            reflector_: Reflector::new(),
            transformer_type,
            stream: MutNullableDom::new(None),
            finish_promise: DomRefCell::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        transformer_type: TransformerType,
        can_gc: CanGc,
    ) -> DomRoot<TransformStreamDefaultController> {
        reflect_dom_object(
            Box::new(TransformStreamDefaultController::new_inherited(
                transformer_type,
            )),
            global,
            can_gc,
        )
    }

    /// Setting the JS object after the heap has settled down.
    ///
    /// Note that this has no effect if the transformer type is not `TransformerType::Js`
    pub(crate) fn set_transform_obj(&self, this_object: SafeHandleObject) {
        if let TransformerType::Js { transform_obj, .. } = &self.transformer_type {
            transform_obj.set(*this_object)
        }
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
        let result = match &self.transformer_type {
            // <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
            TransformerType::Js {
                transform,
                transform_obj,
                ..
            } => {
                // Step 5. If transformerDict["transform"] exists, set
                // transformAlgorithm to an algorithm which takes an argument
                // chunk and returns the result of invoking
                // transformerDict["transform"] with argument list « chunk,
                // controller » and callback this value transformer.
                let algo = transform.borrow().clone();
                if let Some(transform) = algo {
                    rooted!(in(*cx) let this_object = transform_obj.get());
                    transform
                        .Call_(
                            &this_object.handle(),
                            chunk,
                            self,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .unwrap_or_else(|e| {
                            let p = Promise::new(global, can_gc);
                            p.reject_error(e, can_gc);
                            p
                        })
                } else {
                    // Step 2. Let transformAlgorithm be the following steps, taking a chunk argument:
                    // Let result be TransformStreamDefaultControllerEnqueue(controller, chunk).
                    // If result is an abrupt completion, return a promise rejected with result.[[Value]].
                    if let Err(error) = self.enqueue(cx, global, chunk, can_gc) {
                        rooted!(in(*cx) let mut error_val = UndefinedValue());
                        error.to_jsval(cx, global, error_val.handle_mut(), can_gc);
                        Promise::new_rejected(global, cx, error_val.handle(), can_gc)
                    } else {
                        // Otherwise, return a promise resolved with undefined.
                        Promise::new_resolved(global, cx, (), can_gc)
                    }
                }
            },
            // <https://encoding.spec.whatwg.org/#dom-textdecoderstream>
            TransformerType::Decoder(decoder) => {
                // Step 7. Let transformAlgorithm be an algorithm which takes a
                // chunk argument and runs the decode and enqueue a chunk
                // algorithm with this and chunk.
                decode_and_enqueue_a_chunk(cx, global, chunk, decoder, self, can_gc)
                    .map(|_| Promise::new_resolved(global, cx, (), can_gc))
                    .unwrap_or_else(|e| {
                        let p = Promise::new(global, can_gc);
                        p.reject_error(e, can_gc);
                        p
                    })
            },
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
        let result = match &self.transformer_type {
            // <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
            TransformerType::Js {
                cancel,
                transform_obj,
                ..
            } => {
                // Step 7. If transformerDict["cancel"] exists, set
                // cancelAlgorithm to an algorithm which takes an argument
                // reason and returns the result of invoking
                // transformerDict["cancel"] with argument list « reason » and
                // callback this value transformer.
                let algo = cancel.borrow().clone();
                if let Some(cancel) = algo {
                    rooted!(in(*cx) let this_object = transform_obj.get());
                    cancel
                        .Call_(
                            &this_object.handle(),
                            chunk,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .unwrap_or_else(|e| {
                            let p = Promise::new(global, can_gc);
                            p.reject_error(e, can_gc);
                            p
                        })
                } else {
                    // Step 4. Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.
                    Promise::new_resolved(global, cx, (), can_gc)
                }
            },
            // <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
            TransformerType::Decoder(_) => {
                // `TextDecoderStream` does NOT specify a cancel algorithm.
                //
                // Step 4. Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.
                Promise::new_resolved(global, cx, (), can_gc)
            },
        };

        Ok(result)
    }

    pub(crate) fn perform_flush(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        let result = match &self.transformer_type {
            // <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
            TransformerType::Js {
                flush,
                transform_obj,
                ..
            } => {
                // Step 6. If transformerDict["flush"] exists, set flushAlgorithm to an
                // algorithm which returns the result of invoking
                // transformerDict["flush"] with argument list « controller »
                // and callback this value transformer.
                let algo = flush.borrow().clone();
                if let Some(flush) = algo {
                    rooted!(in(*cx) let this_object = transform_obj.get());
                    flush
                        .Call_(
                            &this_object.handle(),
                            self,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                        .unwrap_or_else(|e| {
                            let p = Promise::new(global, can_gc);
                            p.reject_error(e, can_gc);
                            p
                        })
                } else {
                    // Step 3. Let flushAlgorithm be an algorithm which returns a promise resolved with undefined.
                    Promise::new_resolved(global, cx, (), can_gc)
                }
            },
            // <https://encoding.spec.whatwg.org/#dom-textdecoderstream>
            TransformerType::Decoder(decoder) => {
                // Step 8. Let flushAlgorithm be an algorithm which takes no
                // arguments and runs the flush and enqueue algorithm with this.
                flush_and_enqueue(cx, global, decoder, self, can_gc)
                    .map(|_| Promise::new_resolved(global, cx, (), can_gc))
                    .unwrap_or_else(|e| {
                        let p = Promise::new(global, can_gc);
                        p.reject_error(e, can_gc);
                        p
                    })
            },
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
        if let TransformerType::Js {
            cancel,
            flush,
            transform,
            ..
        } = &self.transformer_type
        {
            // Set controller.[[transformAlgorithm]] to undefined.
            transform.replace(None);

            // Set controller.[[flushAlgorithm]] to undefined.
            flush.replace(None);

            // Set controller.[[cancelAlgorithm]] to undefined.
            cancel.replace(None);
        }
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

#[allow(non_snake_case)]
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
