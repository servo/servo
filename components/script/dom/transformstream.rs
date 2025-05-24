/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::ptr::{self};
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortIndex};
use constellation_traits::TransformStreamData;
use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};
use script_bindings::callback::ExceptionHandling;
use script_bindings::realms::InRealm;

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use super::bindings::structuredclone::StructuredData;
use super::bindings::transferable::Transferable;
use super::messageport::MessagePort;
use super::promisenativehandler::Callback;
use super::readablestream::CrossRealmTransformReadable;
use super::types::{TransformStreamDefaultController, WritableStream};
use super::writablestream::CrossRealmTransformWritable;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::TransformStreamBinding::TransformStreamMethods;
use crate::dom::bindings::codegen::Bindings::TransformerBinding::Transformer;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::{ReadableStream, create_readable_stream};
use crate::dom::types::PromiseNativeHandler;
use crate::dom::underlyingsourcecontainer::UnderlyingSourceType;
use crate::dom::writablestream::create_writable_stream;
use crate::dom::writablestreamdefaultcontroller::UnderlyingSinkType;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

impl js::gc::Rootable for TransformBackPressureChangePromiseFulfillment {}

/// Reacting to backpressureChangePromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TransformBackPressureChangePromiseFulfillment {
    /// The result of reacting to backpressureChangePromise.
    #[ignore_malloc_size_of = "Rc is hard"]
    result_promise: Rc<Promise>,

    #[ignore_malloc_size_of = "mozjs"]
    chunk: Box<Heap<JSVal>>,

    /// The writable used in the fulfillment steps
    writable: Dom<WritableStream>,

    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for TransformBackPressureChangePromiseFulfillment {
    /// Reacting to backpressureChangePromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Let writable be stream.[[writable]].
        // Let state be writable.[[state]].
        // If state is "erroring", throw writable.[[storedError]].
        if self.writable.is_erroring() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.writable.get_stored_error(error.handle_mut());
            self.result_promise.reject(cx, error.handle(), can_gc);
            return;
        }

        // Assert: state is "writable".
        assert!(self.writable.is_writable());

        // Return ! TransformStreamDefaultControllerPerformTransform(controller, chunk).
        rooted!(in(*cx) let mut chunk = UndefinedValue());
        chunk.set(self.chunk.get());
        let transform_result = self
            .controller
            .transform_stream_default_controller_perform_transform(
                cx,
                &self.writable.global(),
                chunk.handle(),
                can_gc,
            )
            .expect("perform transform failed");

        // PerformTransformFulfillment and PerformTransformRejection do not need
        // to be rooted because they only contain an Rc.
        let handler = PromiseNativeHandler::new(
            &self.writable.global(),
            Some(Box::new(PerformTransformFulfillment {
                result_promise: self.result_promise.clone(),
            })),
            Some(Box::new(PerformTransformRejection {
                result_promise: self.result_promise.clone(),
            })),
            can_gc,
        );

        let realm = enter_realm(&*self.writable.global());
        let comp = InRealm::Entered(&realm);
        transform_result.append_native_handler(&handler, comp, can_gc);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Reacting to fulfillment of performTransform as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
struct PerformTransformFulfillment {
    #[ignore_malloc_size_of = "Rc is hard"]
    result_promise: Rc<Promise>,
}

impl Callback for PerformTransformFulfillment {
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Fulfilled: resolve the outer promise
        self.result_promise.resolve_native(&(), can_gc);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Reacting to rejection of performTransform as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
struct PerformTransformRejection {
    #[ignore_malloc_size_of = "Rc is hard"]
    result_promise: Rc<Promise>,
}

impl Callback for PerformTransformRejection {
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Stream already errored in perform_transform, just reject result_promise
        self.result_promise.reject(cx, v, can_gc);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Reacting to rejection of backpressureChangePromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
struct BackpressureChangeRejection {
    #[ignore_malloc_size_of = "Rc is hard"]
    result_promise: Rc<Promise>,
}

impl Callback for BackpressureChangeRejection {
    fn callback(&self, cx: SafeJSContext, reason: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        self.result_promise.reject(cx, reason, can_gc);
    }
}

impl js::gc::Rootable for CancelPromiseFulfillment {}

/// Reacting to fulfillment of the cancelpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct CancelPromiseFulfillment {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
    #[ignore_malloc_size_of = "mozjs"]
    reason: Box<Heap<JSVal>>,
}

impl Callback for CancelPromiseFulfillment {
    /// Reacting to backpressureChangePromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If readable.[[state]] is "errored", reject controller.[[finishPromise]] with readable.[[storedError]].
        if self.readable.is_errored() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.readable.get_stored_error(error.handle_mut());
            self.controller
                .get_finish_promise()
                .expect("finish promise is not set")
                .reject_native(&error.handle(), can_gc);
        } else {
            // Otherwise:
            // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], reason).
            rooted!(in(*cx) let mut reason = UndefinedValue());
            reason.set(self.reason.get());
            self.readable
                .get_default_controller()
                .error(reason.handle(), can_gc);

            // Resolve controller.[[finishPromise]] with undefined.
            self.controller
                .get_finish_promise()
                .expect("finish promise is not set")
                .resolve_native(&(), can_gc);
        }
    }
}

impl js::gc::Rootable for CancelPromiseRejection {}

/// Reacting to rejection of cancelpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct CancelPromiseRejection {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for CancelPromiseRejection {
    /// Reacting to backpressureChangePromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], r).
        self.readable.get_default_controller().error(v, can_gc);

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish promise is not set")
            .reject(cx, v, can_gc);
    }
}

impl js::gc::Rootable for SourceCancelPromiseFulfillment {}

/// Reacting to fulfillment of the cancelpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct SourceCancelPromiseFulfillment {
    writeable: Dom<WritableStream>,
    controller: Dom<TransformStreamDefaultController>,
    stream: Dom<TransformStream>,
    #[ignore_malloc_size_of = "mozjs"]
    reason: Box<Heap<JSVal>>,
}

impl Callback for SourceCancelPromiseFulfillment {
    /// Reacting to backpressureChangePromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If cancelPromise was fulfilled, then:
        let finish_promise = self
            .controller
            .get_finish_promise()
            .expect("finish promise is not set");

        let global = &self.writeable.global();
        // If writable.[[state]] is "errored", reject controller.[[finishPromise]] with writable.[[storedError]].
        if self.writeable.is_errored() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.writeable.get_stored_error(error.handle_mut());
            finish_promise.reject(cx, error.handle(), can_gc);
        } else {
            // Otherwise:
            // Perform ! WritableStreamDefaultControllerErrorIfNeeded(writable.[[controller]], reason).
            rooted!(in(*cx) let mut reason = UndefinedValue());
            reason.set(self.reason.get());
            self.writeable.get_default_controller().error_if_needed(
                cx,
                reason.handle(),
                global,
                can_gc,
            );

            // Perform ! TransformStreamUnblockWrite(stream).
            self.stream.unblock_write(global, can_gc);

            // Resolve controller.[[finishPromise]] with undefined.
            finish_promise.resolve_native(&(), can_gc);
        }
    }
}

impl js::gc::Rootable for SourceCancelPromiseRejection {}

/// Reacting to rejection of cancelpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct SourceCancelPromiseRejection {
    writeable: Dom<WritableStream>,
    controller: Dom<TransformStreamDefaultController>,
    stream: Dom<TransformStream>,
}

impl Callback for SourceCancelPromiseRejection {
    /// Reacting to backpressureChangePromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Perform ! WritableStreamDefaultControllerErrorIfNeeded(writable.[[controller]], r).
        let global = &self.writeable.global();

        self.writeable
            .get_default_controller()
            .error_if_needed(cx, v, global, can_gc);

        // Perform ! TransformStreamUnblockWrite(stream).
        self.stream.unblock_write(global, can_gc);

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish promise is not set")
            .reject(cx, v, can_gc);
    }
}

impl js::gc::Rootable for FlushPromiseFulfillment {}

/// Reacting to fulfillment of the flushpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct FlushPromiseFulfillment {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for FlushPromiseFulfillment {
    /// Reacting to flushpromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If flushPromise was fulfilled, then:
        let finish_promise = self
            .controller
            .get_finish_promise()
            .expect("finish promise is not set");

        // If readable.[[state]] is "errored", reject controller.[[finishPromise]] with readable.[[storedError]].
        if self.readable.is_errored() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.readable.get_stored_error(error.handle_mut());
            finish_promise.reject(cx, error.handle(), can_gc);
        } else {
            // Otherwise:
            // Perform ! ReadableStreamDefaultControllerClose(readable.[[controller]]).
            self.readable.get_default_controller().close(can_gc);

            // Resolve controller.[[finishPromise]] with undefined.
            finish_promise.resolve_native(&(), can_gc);
        }
    }
}

impl js::gc::Rootable for FlushPromiseRejection {}
/// Reacting to rejection of flushpromise as part of
/// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct FlushPromiseRejection {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for FlushPromiseRejection {
    /// Reacting to flushpromise with the following fulfillment steps:
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If flushPromise was rejected with reason r, then:
        // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], r).
        self.readable.get_default_controller().error(v, can_gc);

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish promise is not set")
            .reject(cx, v, can_gc);
    }
}

impl js::gc::Rootable for CrossRealmTransform {}

/// A wrapper to handle `message` and `messageerror` events
/// for the message port used by the transfered stream.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum CrossRealmTransform {
    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
    Readable(CrossRealmTransformReadable),
    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
    Writable(CrossRealmTransformWritable),
}

/// <https://streams.spec.whatwg.org/#ts-class>
#[dom_struct]
pub struct TransformStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#transformstream-backpressure>
    backpressure: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#transformstream-backpressurechangepromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    backpressure_change_promise: DomRefCell<Option<Rc<Promise>>>,

    /// <https://streams.spec.whatwg.org/#transformstream-controller>
    controller: MutNullableDom<TransformStreamDefaultController>,

    /// <https://streams.spec.whatwg.org/#transformstream-detached>
    detached: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#transformstream-readable>
    readable: MutNullableDom<ReadableStream>,

    /// <https://streams.spec.whatwg.org/#transformstream-writable>
    writable: MutNullableDom<WritableStream>,
}

impl TransformStream {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#initialize-transform-stream>
    fn new_inherited() -> TransformStream {
        TransformStream {
            reflector_: Reflector::new(),
            backpressure: Default::default(),
            backpressure_change_promise: DomRefCell::new(None),
            controller: MutNullableDom::new(None),
            detached: Cell::new(false),
            readable: MutNullableDom::new(None),
            writable: MutNullableDom::new(None),
        }
    }

    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<TransformStream> {
        reflect_dom_object_with_proto(
            Box::new(TransformStream::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_controller(&self) -> DomRoot<TransformStreamDefaultController> {
        self.controller.get().expect("controller is not set")
    }

    pub(crate) fn get_writable(&self) -> DomRoot<WritableStream> {
        self.writable.get().expect("writable stream is not set")
    }

    pub(crate) fn get_readable(&self) -> DomRoot<ReadableStream> {
        self.readable.get().expect("readable stream is not set")
    }

    pub(crate) fn get_backpressure(&self) -> bool {
        self.backpressure.get()
    }

    /// <https://streams.spec.whatwg.org/#initialize-transform-stream>
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        start_promise: Rc<Promise>,
        writable_high_water_mark: f64,
        writable_size_algorithm: Rc<QueuingStrategySize>,
        readable_high_water_mark: f64,
        readable_size_algorithm: Rc<QueuingStrategySize>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let startAlgorithm be an algorithm that returns startPromise.
        // Let writeAlgorithm be the following steps, taking a chunk argument:
        //  Return ! TransformStreamDefaultSinkWriteAlgorithm(stream, chunk).
        // Let abortAlgorithm be the following steps, taking a reason argument:
        //  Return ! TransformStreamDefaultSinkAbortAlgorithm(stream, reason).
        // Let closeAlgorithm be the following steps:
        //  Return ! TransformStreamDefaultSinkCloseAlgorithm(stream).
        // Set stream.[[writable]] to ! CreateWritableStream(startAlgorithm, writeAlgorithm,
        // closeAlgorithm, abortAlgorithm, writableHighWaterMark, writableSizeAlgorithm).
        // Note: Those steps are implemented using UnderlyingSinkType::Transform.

        let writable = create_writable_stream(
            cx,
            global,
            writable_high_water_mark,
            writable_size_algorithm,
            UnderlyingSinkType::Transform(Dom::from_ref(self), start_promise.clone()),
            can_gc,
        )?;
        self.writable.set(Some(&writable));

        // Let pullAlgorithm be the following steps:

        // Return ! TransformStreamDefaultSourcePullAlgorithm(stream).

        // Let cancelAlgorithm be the following steps, taking a reason argument:

        // Return ! TransformStreamDefaultSourceCancelAlgorithm(stream, reason).

        // Set stream.[[readable]] to ! CreateReadableStream(startAlgorithm, pullAlgorithm,
        // cancelAlgorithm, readableHighWaterMark, readableSizeAlgorithm).

        let readable = create_readable_stream(
            global,
            UnderlyingSourceType::Transform(Dom::from_ref(self), start_promise.clone()),
            Some(readable_size_algorithm),
            Some(readable_high_water_mark),
            can_gc,
        );
        self.readable.set(Some(&readable));

        // Set stream.[[backpressure]] and stream.[[backpressureChangePromise]] to undefined.
        // Note: This is done in the constructor.

        // Perform ! TransformStreamSetBackpressure(stream, true).
        self.set_backpressure(global, true, can_gc);

        // Set stream.[[controller]] to undefined.
        self.controller.set(None);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-set-backpressure>
    pub(crate) fn set_backpressure(&self, global: &GlobalScope, backpressure: bool, can_gc: CanGc) {
        // Assert: stream.[[backpressure]] is not backpressure.
        assert!(self.backpressure.get() != backpressure);

        // If stream.[[backpressureChangePromise]] is not undefined, resolve
        // stream.[[backpressureChangePromise]] with undefined.
        if let Some(promise) = self.backpressure_change_promise.borrow_mut().take() {
            promise.resolve_native(&(), can_gc);
        }

        // Set stream.[[backpressureChangePromise]] to a new promise.;
        *self.backpressure_change_promise.borrow_mut() = Some(Promise::new(global, can_gc));

        // Set stream.[[backpressure]] to backpressure.
        self.backpressure.set(backpressure);
    }

    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller>
    fn set_up_transform_stream_default_controller(
        &self,
        controller: &TransformStreamDefaultController,
    ) {
        // Assert: stream implements TransformStream.
        // Note: this is checked with type.

        // Assert: stream.[[controller]] is undefined.
        assert!(self.controller.get().is_none());

        // Set controller.[[stream]] to stream.
        controller.set_stream(self);

        // Set stream.[[controller]] to controller.
        self.controller.set(Some(controller));

        // Set controller.[[transformAlgorithm]] to transformAlgorithm.
        // Set controller.[[flushAlgorithm]] to flushAlgorithm.
        // Set controller.[[cancelAlgorithm]] to cancelAlgorithm.
        // Note: These are set in the constructor.
    }

    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
    fn set_up_transform_stream_default_controller_from_transformer(
        &self,
        global: &GlobalScope,
        transformer_obj: SafeHandleObject,
        transformer: &Transformer,
        can_gc: CanGc,
    ) {
        // Let controller be a new TransformStreamDefaultController.
        let controller = TransformStreamDefaultController::new(global, transformer, can_gc);

        // Let transformAlgorithm be the following steps, taking a chunk argument:
        // Let result be TransformStreamDefaultControllerEnqueue(controller, chunk).
        // If result is an abrupt completion, return a promise rejected with result.[[Value]].
        // Otherwise, return a promise resolved with undefined.

        // Let flushAlgorithm be an algorithm which returns a promise resolved with undefined.
        // Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.

        // If transformerDict["transform"] exists, set transformAlgorithm to an algorithm which
        // takes an argument
        // chunk and returns the result of invoking transformerDict["transform"] with argument
        // list « chunk, controller »
        // and callback this value transformer.

        // If transformerDict["flush"] exists, set flushAlgorithm to an algorithm which returns
        // the result
        // of invoking transformerDict["flush"] with argument list « controller » and callback
        // this value transformer.

        // If transformerDict["cancel"] exists, set cancelAlgorithm to an algorithm which takes an argument
        // reason and returns the result of invoking transformerDict["cancel"] with argument list « reason »
        // and callback this value transformer.
        controller.set_transform_obj(transformer_obj);

        // Perform ! SetUpTransformStreamDefaultController(stream, controller,
        // transformAlgorithm, flushAlgorithm, cancelAlgorithm).
        self.set_up_transform_stream_default_controller(&controller);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
    pub(crate) fn transform_stream_default_sink_write_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Assert: stream.[[writable]].[[state]] is "writable".
        assert!(self.writable.get().is_some());

        // Let controller be stream.[[controller]].
        let controller = self.controller.get().expect("controller is not set");

        // If stream.[[backpressure]] is true,
        if self.backpressure.get() {
            // Let backpressureChangePromise be stream.[[backpressureChangePromise]].
            let backpressure_change_promise = self.backpressure_change_promise.borrow();

            // Assert: backpressureChangePromise is not undefined.
            assert!(backpressure_change_promise.is_some());

            // Return the result of reacting to backpressureChangePromise with the following fulfillment steps:
            let result_promise = Promise::new(global, can_gc);
            rooted!(in(*cx) let mut fulfillment_handler = Some(TransformBackPressureChangePromiseFulfillment {
                controller: Dom::from_ref(&controller),
                writable: Dom::from_ref(&self.writable.get().expect("writable stream")),
                chunk: Heap::boxed(chunk.get()),
                result_promise: result_promise.clone(),
            }));

            let handler = PromiseNativeHandler::new(
                global,
                fulfillment_handler.take().map(|h| Box::new(h) as Box<_>),
                Some(Box::new(BackpressureChangeRejection {
                    result_promise: result_promise.clone(),
                })),
                can_gc,
            );
            let realm = enter_realm(global);
            let comp = InRealm::Entered(&realm);
            backpressure_change_promise
                .as_ref()
                .expect("Promise must be some by now.")
                .append_native_handler(&handler, comp, can_gc);

            return Ok(result_promise);
        }

        // Return ! TransformStreamDefaultControllerPerformTransform(controller, chunk).
        controller.transform_stream_default_controller_perform_transform(cx, global, chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
    pub(crate) fn transform_stream_default_sink_abort_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self.controller.get().expect("controller is not set");

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let readable be stream.[[readable]].
        let readable = self.readable.get().expect("readable stream is not set");

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let cancelPromise be the result of performing controller.[[cancelAlgorithm]], passing reason.
        let cancel_promise = controller.perform_cancel(cx, global, reason, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to cancelPromise:
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(CancelPromiseFulfillment {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
                reason: Heap::boxed(reason.get()),
            })),
            Some(Box::new(CancelPromiseRejection {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return controller.[[finishPromise]].
        let finish_promise = controller
            .get_finish_promise()
            .expect("finish promise is not set");
        Ok(finish_promise)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>
    pub(crate) fn transform_stream_default_sink_close_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self
            .controller
            .get()
            .ok_or(Error::Type("controller is not set".to_string()))?;

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let readable be stream.[[readable]].
        let readable = self
            .readable
            .get()
            .ok_or(Error::Type("readable stream is not set".to_string()))?;

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let flushPromise be the result of performing controller.[[flushAlgorithm]].
        let flush_promise = controller.perform_flush(cx, global, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to flushPromise:
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(FlushPromiseFulfillment {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            Some(Box::new(FlushPromiseRejection {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            can_gc,
        );

        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        flush_promise.append_native_handler(&handler, comp, can_gc);
        // Return controller.[[finishPromise]].
        let finish_promise = controller
            .get_finish_promise()
            .expect("finish promise is not set");
        Ok(finish_promise)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
    pub(crate) fn transform_stream_default_source_cancel(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self
            .controller
            .get()
            .ok_or(Error::Type("controller is not set".to_string()))?;

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let writable be stream.[[writable]].
        let writable = self
            .writable
            .get()
            .ok_or(Error::Type("writable stream is not set".to_string()))?;

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let cancelPromise be the result of performing controller.[[cancelAlgorithm]], passing reason.
        let cancel_promise = controller.perform_cancel(cx, global, reason, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to cancelPromise:
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(SourceCancelPromiseFulfillment {
                writeable: Dom::from_ref(&writable),
                controller: Dom::from_ref(&controller),
                stream: Dom::from_ref(self),
                reason: Heap::boxed(reason.get()),
            })),
            Some(Box::new(SourceCancelPromiseRejection {
                writeable: Dom::from_ref(&writable),
                controller: Dom::from_ref(&controller),
                stream: Dom::from_ref(self),
            })),
            can_gc,
        );

        // Return controller.[[finishPromise]].
        let finish_promise = controller
            .get_finish_promise()
            .expect("finish promise is not set");
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        cancel_promise.append_native_handler(&handler, comp, can_gc);
        Ok(finish_promise)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-pull>
    pub(crate) fn transform_stream_default_source_pull(
        &self,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Assert: stream.[[backpressure]] is true.
        assert!(self.backpressure.get());

        // Assert: stream.[[backpressureChangePromise]] is not undefined.
        assert!(self.backpressure_change_promise.borrow().is_some());

        // Perform ! TransformStreamSetBackpressure(stream, false).
        self.set_backpressure(global, false, can_gc);

        // Return stream.[[backpressureChangePromise]].
        Ok(self
            .backpressure_change_promise
            .borrow()
            .clone()
            .expect("Promise must be some by now."))
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-error-writable-and-unblock-write>
    pub(crate) fn error_writable_and_unblock_write(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        // Perform ! TransformStreamDefaultControllerClearAlgorithms(stream.[[controller]]).
        self.get_controller().clear_algorithms();

        // Perform ! WritableStreamDefaultControllerErrorIfNeeded(stream.[[writable]].[[controller]], e).
        self.get_writable()
            .get_default_controller()
            .error_if_needed(cx, error, global, can_gc);

        // Perform ! TransformStreamUnblockWrite(stream).
        self.unblock_write(global, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-unblock-write>
    pub(crate) fn unblock_write(&self, global: &GlobalScope, can_gc: CanGc) {
        // If stream.[[backpressure]] is true, perform ! TransformStreamSetBackpressure(stream, false).
        if self.backpressure.get() {
            self.set_backpressure(global, false, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-error>
    pub(crate) fn error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        // Perform ! ReadableStreamDefaultControllerError(stream.[[readable]].[[controller]], e).
        self.get_readable()
            .get_default_controller()
            .error(error, can_gc);

        // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, e).
        self.error_writable_and_unblock_write(cx, global, error, can_gc);
    }
}

impl TransformStreamMethods<crate::DomTypeHolder> for TransformStream {
    /// <https://streams.spec.whatwg.org/#ts-constructor>
    #[allow(unsafe_code)]
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        transformer: Option<*mut JSObject>,
        writable_strategy: &QueuingStrategy,
        readable_strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<TransformStream>> {
        // If transformer is missing, set it to null.
        rooted!(in(*cx) let transformer_obj = transformer.unwrap_or(ptr::null_mut()));

        // Let underlyingSinkDict be underlyingSink,
        // converted to an IDL value of type UnderlyingSink.
        let transformer_dict = if !transformer_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(transformer_obj.get()));
            match Transformer::new(cx, obj_val.handle()) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => return Err(Error::Type(error.to_string())),
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            Transformer::empty()
        };

        // If transformerDict["readableType"] exists, throw a RangeError exception.
        if !transformer_dict.readableType.handle().is_undefined() {
            return Err(Error::Range("readableType is set".to_string()));
        }

        // If transformerDict["writableType"] exists, throw a RangeError exception.
        if !transformer_dict.writableType.handle().is_undefined() {
            return Err(Error::Range("writableType is set".to_string()));
        }

        // Let readableHighWaterMark be ? ExtractHighWaterMark(readableStrategy, 0).
        let readable_high_water_mark = extract_high_water_mark(readable_strategy, 0.0)?;

        // Let readableSizeAlgorithm be ! ExtractSizeAlgorithm(readableStrategy).
        let readable_size_algorithm = extract_size_algorithm(readable_strategy, can_gc);

        // Let writableHighWaterMark be ? ExtractHighWaterMark(writableStrategy, 1).
        let writable_high_water_mark = extract_high_water_mark(writable_strategy, 1.0)?;

        // Let writableSizeAlgorithm be ! ExtractSizeAlgorithm(writableStrategy).
        let writable_size_algorithm = extract_size_algorithm(writable_strategy, can_gc);

        // Let startPromise be a new promise.
        let start_promise = Promise::new(global, can_gc);

        // Perform ! InitializeTransformStream(this, startPromise, writableHighWaterMark,
        // writableSizeAlgorithm, readableHighWaterMark, readableSizeAlgorithm).
        let stream = TransformStream::new_with_proto(global, proto, can_gc);
        stream.initialize(
            cx,
            global,
            start_promise.clone(),
            writable_high_water_mark,
            writable_size_algorithm,
            readable_high_water_mark,
            readable_size_algorithm,
            can_gc,
        )?;

        // Perform ? SetUpTransformStreamDefaultControllerFromTransformer(this, transformer, transformerDict).
        stream.set_up_transform_stream_default_controller_from_transformer(
            global,
            transformer_obj.handle(),
            &transformer_dict,
            can_gc,
        );

        // If transformerDict["start"] exists, then resolve startPromise with the
        // result of invoking transformerDict["start"]
        // with argument list « this.[[controller]] » and callback this value transformer.
        if let Some(start) = &transformer_dict.start {
            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = transformer_obj.get());
            start.Call_(
                &this_object.handle(),
                &stream.get_controller(),
                result.handle_mut(),
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
            let promise = if is_promise {
                let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                promise
            } else {
                Promise::new_resolved(global, cx, result.get(), can_gc)
            };
            start_promise.resolve_native(&promise, can_gc);
        } else {
            // Otherwise, resolve startPromise with undefined.
            start_promise.resolve_native(&(), can_gc);
        };

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#ts-readable>
    fn Readable(&self) -> DomRoot<ReadableStream> {
        // Return this.[[readable]].
        self.readable.get().expect("readable stream is not set")
    }

    /// <https://streams.spec.whatwg.org/#ts-writable>
    fn Writable(&self) -> DomRoot<WritableStream> {
        // Return this.[[writable]].
        self.writable.get().expect("writable stream is not set")
    }
}

/// <https://streams.spec.whatwg.org/#ts-transfer>
impl Transferable for TransformStream {
    type Index = MessagePortIndex;
    type Data = TransformStreamData;

    fn transfer(&self) -> Result<(MessagePortId, TransformStreamData), ()> {
        let global = self.global();
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        let cx = GlobalScope::get_cx();
        let can_gc = CanGc::note();

        // Let readable be value.[[readable]].
        let readable = self.get_readable();

        // Let writable be value.[[writable]].
        let writable = self.get_writable();

        // If ! IsReadableStreamLocked(readable) is true, throw a "DataCloneError" DOMException.
        // If ! IsWritableStreamLocked(writable) is true, throw a "DataCloneError" DOMException.
        if readable.is_locked() || writable.is_locked() {
            return Err(());
        }

        // First port pair (readable → proxy writable)
        let port1 = MessagePort::new(&global, can_gc);
        global.track_message_port(&port1, None);
        let port1_peer = MessagePort::new(&global, can_gc);
        global.track_message_port(&port1_peer, None);
        global.entangle_ports(*port1.message_port_id(), *port1_peer.message_port_id());

        let proxy_readable = ReadableStream::new_with_proto(&global, None, can_gc);
        proxy_readable.setup_cross_realm_transform_readable(cx, &port1, can_gc);
        proxy_readable
            .pipe_to(cx, &global, &writable, false, false, false, comp, can_gc)
            .set_promise_is_handled();

        // Second port pair (proxy readable → writable)
        let port2 = MessagePort::new(&global, can_gc);
        global.track_message_port(&port2, None);
        let port2_peer = MessagePort::new(&global, can_gc);
        global.track_message_port(&port2_peer, None);
        global.entangle_ports(*port2.message_port_id(), *port2_peer.message_port_id());

        let proxy_writable = WritableStream::new_with_proto(&global, None, can_gc);
        proxy_writable.setup_cross_realm_transform_writable(cx, &port2, can_gc);

        // Pipe readable into the proxy writable (→ port_1)
        readable
            .pipe_to(
                cx,
                &global,
                &proxy_writable,
                false,
                false,
                false,
                comp,
                can_gc,
            )
            .set_promise_is_handled();

        // Set dataHolder.[[readable]] to ! StructuredSerializeWithTransfer(readable, « readable »).
        // Set dataHolder.[[writable]] to ! StructuredSerializeWithTransfer(writable, « writable »).
        Ok((
            *port1_peer.message_port_id(),
            TransformStreamData {
                readable: port1_peer.transfer()?,
                writable: port2_peer.transfer()?,
            },
        ))
    }

    fn transfer_receive(
        owner: &GlobalScope,
        _id: MessagePortId,
        data: TransformStreamData,
    ) -> Result<DomRoot<Self>, ()> {
        let can_gc = CanGc::note();
        let cx = GlobalScope::get_cx();

        let port1 = MessagePort::transfer_receive(owner, data.readable.0, data.readable.1)?;
        let port2 = MessagePort::transfer_receive(owner, data.writable.0, data.writable.1)?;

        // Let readableRecord be ! StructuredDeserializeWithTransfer(dataHolder.[[readable]], the current Realm).
        // Set value.[[readable]] to readableRecord.[[Deserialized]].
        // Let writableRecord be ! StructuredDeserializeWithTransfer(dataHolder.[[writable]], the current Realm).
        let proxy_readable = ReadableStream::new_with_proto(owner, None, can_gc);
        proxy_readable.setup_cross_realm_transform_readable(cx, &port2, can_gc);

        let proxy_writable = WritableStream::new_with_proto(owner, None, can_gc);
        proxy_writable.setup_cross_realm_transform_writable(cx, &port1, can_gc);

        // Set value.[[readable]] to readableRecord.[[Deserialized]].
        // Set value.[[writable]] to writableRecord.[[Deserialized]].
        // Set value.[[backpressure]], value.[[backpressureChangePromise]], and value.[[controller]] to undefined.
        let stream = TransformStream::new_with_proto(owner, None, can_gc);
        stream.readable.set(Some(&proxy_readable));
        stream.writable.set(Some(&proxy_writable));

        Ok(stream)
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<MessagePortId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.transform_streams_port_impls,
            StructuredData::Writer(w) => &mut w.transform_streams_port,
        }
    }
}
