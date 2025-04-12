/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{ExceptionStackBehavior, IsPromiseObject, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::wrappers::JS_SetPendingException;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};
use script_bindings::callback::ExceptionHandling;
use script_bindings::codegen::GenericBindings::TransformStreamBinding::TransformStreamMethods;
use script_bindings::root::Dom;

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use super::bindings::reflector::DomGlobal;
use super::readablestream::{ReadableStream, create_readable_stream};
use super::transformstreamdefaultcontroller::TransformStreamDefaultController;
use super::types::TransformUnderlyingSource;
use super::underlyingsourcecontainer::UnderlyingSourceType;
use super::writablestream::{WritableStream, create_writable_stream};
use super::writablestreamdefaultcontroller::UnderlyingSinkType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::TransformerBinding::Transformer;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The resolution handler for the reacting to cancelPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct SourceReadableCancelPromiseResolutionHandler {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl js::gc::Rootable for SourceReadableCancelPromiseResolutionHandler {}

impl Callback for SourceReadableCancelPromiseResolutionHandler {
    /// The resolution handler for the reacting to cancelPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, cx: SafeJSContext, reason: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        if self.readable.is_errored() {
            // If readable.[[state]] is "errored", reject controller.[[finishPromise]] with readable.[[storedError]].
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.readable.get_stored_error(error.handle_mut());
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .reject_native(&error.handle(), can_gc);
        } else {
            // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], reason).
            self.readable.get_default_controller().error(reason, can_gc);

            // Resolve controller.[[finishPromise]] with undefined.
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .resolve_native(&(), can_gc);
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// The rejection handler for the reacting to cancelPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
struct SourceReadableCancelPromiseRejectionHandler {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl js::gc::Rootable for SourceReadableCancelPromiseRejectionHandler {}

impl Callback for SourceReadableCancelPromiseRejectionHandler {
    /// The rejection handler for the reacting to cancelPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(
        &self,
        _cx: SafeJSContext,
        reason: SafeHandleValue,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], r).
        self.readable.get_default_controller().error(reason, can_gc);

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish_promise should be set")
            .reject_native(&reason, can_gc);
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// The resolution handler for the reacting to flushPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>
struct SourceFlushPromiseResolutionHandler {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl js::gc::Rootable for SourceFlushPromiseResolutionHandler {}

impl Callback for SourceFlushPromiseResolutionHandler {
    /// The resolution handler for the reacting to flushPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(
        &self,
        cx: SafeJSContext,
        _reason: SafeHandleValue,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        // If readable.[[state]] is "errored", reject controller.[[finishPromise]] with readable.[[storedError]].
        if self.readable.is_errored() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.readable.get_stored_error(error.handle_mut());
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .reject_native(&error.handle(), can_gc);
        } else {
            // Perform ! ReadableStreamDefaultControllerClose(readable.[[controller]]).
            self.readable.get_default_controller().close(can_gc);

            // Resolve controller.[[finishPromise]] with undefined.
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .resolve_native(&(), can_gc);
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// The rejection handler for the reacting to flushPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>
struct SourceFlushPromiseRejectionHandler {
    readable: Dom<ReadableStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl js::gc::Rootable for SourceFlushPromiseRejectionHandler {}

impl Callback for SourceFlushPromiseRejectionHandler {
    /// The rejection handler for the reacting to flushPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(
        &self,
        _cx: SafeJSContext,
        reason: SafeHandleValue,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        // Perform ! ReadableStreamDefaultControllerError(readable.[[controller]], r).
        self.readable.get_default_controller().error(reason, can_gc);

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish_promise should be set")
            .reject_native(&reason, can_gc);
    }
}

/// The resolution handler for the reacting to cancelPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct SourceWritableCancelPromiseResolutionHandler {
    writable: Dom<WritableStream>,
    stream: Dom<TransformStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl js::gc::Rootable for SourceWritableCancelPromiseResolutionHandler {}

impl Callback for SourceWritableCancelPromiseResolutionHandler {
    /// The resolution handler for the reacting to cancelPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, cx: SafeJSContext, reason: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If writable.[[state]] is "errored", reject controller.[[finishPromise]] with writable.[[storedError]].
        if self.writable.is_errored() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            self.writable.get_stored_error(error.handle_mut());
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .reject_native(&error.handle(), can_gc);
        } else {
            // Perform ! WritableStreamDefaultControllerErrorIfNeeded(writable.[[controller]], reason).
            self.writable.get_default_controller().error_if_needed(
                cx,
                reason,
                &self.writable.global(),
                can_gc,
            );

            // Perform ! TransformStreamUnblockWrite(stream).
            self.stream
                .unblock_write(&self.writable.global(), can_gc)
                .expect("unblock_write should not fail");

            // Resolve controller.[[finishPromise]] with undefined.
            self.controller
                .get_finish_promise()
                .expect("finish_promise should be set")
                .resolve_native(&(), can_gc);
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// The rejection handler for the reacting to cancelPromise part of
/// part of <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
struct SourceWritableCancelPromiseRejectionHandler {
    writable: Dom<WritableStream>,
    stream: Dom<TransformStream>,
    controller: Dom<TransformStreamDefaultController>,
}

impl Callback for SourceWritableCancelPromiseRejectionHandler {
    /// The rejection handler for the reacting to cancelPromise: part of
    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(
        &self,
        _cx: SafeJSContext,
        reason: SafeHandleValue,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        // Perform ! WritableStreamDefaultControllerErrorIfNeeded(writable.[[controller]], r).
        self.writable.get_default_controller().error_if_needed(
            _cx,
            reason,
            &self.writable.global(),
            can_gc,
        );

        // Perform ! TransformStreamUnblockWrite(stream).
        self.stream
            .unblock_write(&self.writable.global(), can_gc)
            .expect("unblock_write should not fail");

        // Reject controller.[[finishPromise]] with r.
        self.controller
            .get_finish_promise()
            .expect("finish_promise should be set")
            .reject_native(&reason, can_gc);
    }
}

/// <https://streams.spec.whatwg.org/#ts-class>
#[dom_struct]
pub struct TransformStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#transformstream-backpressure>
    backpressure: Cell<Option<bool>>,

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
    fn new_inherited() -> TransformStream {
        TransformStream {
            reflector_: Reflector::new(),
            backpressure: Cell::new(None),
            backpressure_change_promise: DomRefCell::new(None),
            controller: MutNullableDom::new(None),
            detached: Cell::new(false),
            readable: MutNullableDom::new(None),
            writable: MutNullableDom::new(None),
        }
    }

    fn new_with_proto(
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

    pub(crate) fn readable(&self) -> DomRoot<ReadableStream> {
        self.readable.get().expect("readable stream is null")
    }

    pub(crate) fn backpressure(&self) -> Option<bool> {
        self.backpressure.get()
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller>
    pub(crate) fn assert_no_controller(&self) {
        assert!(self.controller.get().is_none());
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controlle>
    pub(crate) fn set_default_controller(&self, controller: &TransformStreamDefaultController) {
        self.controller.set(Some(controller));
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
    #[allow(unsafe_code)]
    pub(crate) fn default_sink_write_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Assert: stream.[[writable]].[[state]] is "writable".
        assert!(self.writable.get().unwrap().is_writable());

        // Let controller be stream.[[controller]].
        let controller = self.controller.get().unwrap();

        // If stream.[[backpressure]] is true,
        if self.backpressure.get().unwrap_or(false) {
            // Let writable be stream.[[writable]].
            let writable = self.writable.get().unwrap();

            // Let state be writable.[[state]].
            // If state is "erroring", throw writable.[[storedError]].
            if writable.is_erroring() {
                rooted!(in(*cx) let mut error = UndefinedValue());
                writable.get_stored_error(error.handle_mut());
                unsafe {
                    JS_SetPendingException(*cx, error.handle(), ExceptionStackBehavior::Capture)
                };
                return Err(Error::JSFailed);
            }

            // Assert: state is "writable".
            assert!(writable.is_writable());

            // Return ! TransformStreamDefaultControllerPerformTransform(controller, chunk).
            return controller.perform_transform(cx, global, chunk, can_gc);
        }

        // Return ! TransformStreamDefaultControllerPerformTransform(controller, chunk).
        controller.perform_transform(cx, global, chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-abort-algorithm>
    pub(crate) fn default_sink_abort_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self.controller.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let readable be stream.[[readable]].
        let readable = self.readable.get().ok_or(Error::Type(
            "TransformStream readable stream is null".to_string(),
        ))?;

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let cancelPromise be the result of performing controller.[[cancelAlgorithm]], passing reason.
        let cancel_promise = controller.call_cancel(cx, global, reason, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to cancelPromise:

        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(SourceReadableCancelPromiseResolutionHandler {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            Some(Box::new(SourceReadableCancelPromiseRejectionHandler {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return controller.[[finishPromise]].
        controller.get_finish_promise().ok_or(Error::Type(
            "TransformStreamDefaultController finish_promise is null".to_string(),
        ))
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-close-algorithm>
    pub(crate) fn default_sink_close_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self.controller.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let readable be stream.[[readable]].
        let readable = self.readable.get().ok_or(Error::Type(
            "TransformStream readable stream is null".to_string(),
        ))?;

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let flushPromise be the result of performing controller.[[flushAlgorithm]].
        let flush_promise = controller.call_flush(cx, global, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to flushPromise:
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(SourceFlushPromiseResolutionHandler {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            Some(Box::new(SourceFlushPromiseRejectionHandler {
                readable: Dom::from_ref(&readable),
                controller: Dom::from_ref(&controller),
            })),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        flush_promise.append_native_handler(&handler, comp, can_gc);

        // Return controller.[[finishPromise]].
        controller.get_finish_promise().ok_or(Error::Type(
            "TransformStreamDefaultController finish_promise is null".to_string(),
        ))
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-pull>
    pub(crate) fn default_source_pull_algorithm(
        &self,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Assert: stream.[[backpressure]] is true.
        assert!(self.backpressure.get().unwrap_or(false));

        // Assert: stream.[[backpressureChangePromise]] is not undefined.
        assert!(self.backpressure_change_promise.borrow().is_some());

        // Perform ! TransformStreamSetBackpressure(stream, false).
        self.set_backpressure(global, false, can_gc)?;

        // Return stream.[[backpressureChangePromise]].
        self.backpressure_change_promise
            .borrow()
            .clone()
            .ok_or(Error::Type(
                "TransformStream backpressure_change_promise is null".to_string(),
            ))
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-source-cancel>
    pub(crate) fn default_source_cancel_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Let controller be stream.[[controller]].
        let controller = self.controller.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // If controller.[[finishPromise]] is not undefined, return controller.[[finishPromise]].
        if let Some(finish_promise) = controller.get_finish_promise() {
            return Ok(finish_promise);
        }

        // Let writable be stream.[[writable]].
        let writable = self.writable.get().ok_or(Error::Type(
            "TransformStream writable stream is null".to_string(),
        ))?;

        // Let controller.[[finishPromise]] be a new promise.
        controller.set_finish_promise(Promise::new(global, can_gc));

        // Let cancelPromise be the result of performing controller.[[cancelAlgorithm]], passing reason.
        let cancel_promise = controller.call_cancel(cx, global, reason, can_gc)?;

        // Perform ! TransformStreamDefaultControllerClearAlgorithms(controller).
        controller.clear_algorithms();

        // React to cancelPromise:
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(SourceWritableCancelPromiseResolutionHandler {
                writable: Dom::from_ref(&writable),
                stream: Dom::from_ref(self),
                controller: Dom::from_ref(&controller),
            })),
            Some(Box::new(SourceWritableCancelPromiseRejectionHandler {
                writable: Dom::from_ref(&writable),
                stream: Dom::from_ref(self),
                controller: Dom::from_ref(&controller),
            })),
            can_gc,
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return controller.[[finishPromise]].
        controller.get_finish_promise().ok_or(Error::Type(
            "TransformStreamDefaultController finish_promise is null".to_string(),
        ))
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-error-writable-and-unblock-write>
    pub(crate) fn error_writable_and_unblock_write(
        &self,
        global: &GlobalScope,
        cx: SafeJSContext,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Perform ! TransformStreamDefaultControllerClearAlgorithms(stream.[[controller]]).
        self.controller
            .get()
            .ok_or(Error::Type(
                "TransformStreamDefaultController stream is null".to_string(),
            ))?
            .clear_algorithms();

        // Perform ! WritableStreamDefaultControllerErrorIfNeeded(stream.[[writable]].[[controller]], e).
        let stream = self.writable.get().ok_or(Error::Type(
            "TransformStream writable stream is null".to_string(),
        ))?;
        stream
            .get_default_controller()
            .error_if_needed(cx, reason, global, can_gc);

        // Perform ! TransformStreamUnblockWrite(stream).
        self.unblock_write(global, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-unblock-write>
    pub(crate) fn unblock_write(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<()> {
        // If stream.[[backpressure]] is true, perform ! TransformStreamSetBackpressure(stream, false).
        if self.backpressure.get().unwrap_or(false) {
            self.set_backpressure(global, false, can_gc)?;
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-set-backpressure>
    pub(crate) fn set_backpressure(
        &self,
        global: &GlobalScope,
        backpressure: bool,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Assert: stream.[[backpressure]] is not backpressure.
        assert!(self.backpressure.get() != Some(backpressure));

        // If stream.[[backpressureChangePromise]] is not undefined,
        // resolve stream.[[backpressureChangePromise]] with undefined.
        {
            if let Some(backpressure_change_promise) =
                self.backpressure_change_promise.borrow().clone()
            {
                backpressure_change_promise.resolve_native(&(), can_gc);
            }
        }

        // Set stream.[[backpressureChangePromise]] to a new promise.
        self.backpressure_change_promise
            .borrow_mut()
            .replace(Promise::new(global, can_gc));

        // Set stream.[[backpressure]] to backpressure.
        self.backpressure.set(Some(backpressure));

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-error>
    pub(crate) fn error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: js::gc::HandleValue,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Perform ! ReadableStreamDefaultControllerError(stream.[[readable]].[[controller]], e).
        self.readable
            .get()
            .ok_or(Error::Type(
                "TransformStream readable stream is null".to_string(),
            ))?
            .get_default_controller()
            .error(reason, can_gc);

        // Perform ! TransformStreamErrorWritableAndUnblockWrite(stream, e).
        self.error_writable_and_unblock_write(global, cx, reason, can_gc)
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
        // Return ! TransformStreamDefaultSinkWriteAlgorithm(stream, chunk).

        // Let abortAlgorithm be the following steps, taking a reason argument:
        // Return ! TransformStreamDefaultSinkAbortAlgorithm(stream, reason).

        // Let closeAlgorithm be the following steps:
        // Return ! TransformStreamDefaultSinkCloseAlgorithm(stream).

        // Set stream.[[writable]] to ! CreateWritableStream(startAlgorithm, writeAlgorithm,
        //  closeAlgorithm, abortAlgorithm, writableHighWaterMark, writableSizeAlgorithm).
        let transform_source = TransformUnderlyingSource::new(self, start_promise, can_gc);
        let writable = create_writable_stream(
            cx,
            global,
            can_gc,
            writable_high_water_mark,
            writable_size_algorithm,
            UnderlyingSinkType::Transform(Dom::from_ref(&transform_source)),
        )?;

        self.writable.set(Some(&writable));

        // Let pullAlgorithm be the following steps:
        // Return ! TransformStreamDefaultSourcePullAlgorithm(stream).
        // Let cancelAlgorithm be the following steps, taking a reason argument:
        // Return ! TransformStreamDefaultSourceCancelAlgorithm(stream, reason).
        let readable = create_readable_stream(
            &self.global(),
            UnderlyingSourceType::Transform(Dom::from_ref(&transform_source)),
            Some(readable_size_algorithm),
            Some(readable_high_water_mark),
            can_gc,
        );
        self.readable.set(Some(&readable));

        // Set stream.[[backpressure]] and stream.[[backpressureChangePromise]] to undefined.
        self.backpressure.set(None);
        *self.backpressure_change_promise.borrow_mut() = None;

        // Perform ! TransformStreamSetBackpressure(stream, true).
        self.set_backpressure(global, true, can_gc)?;

        // Set stream.[[controller]] to undefined.
        self.controller.set(None);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#set-up-transform-stream-default-controller-from-transformer>
    pub(crate) fn set_up_transform_stream_default_controller_from_transformer(
        &self,
        global: &GlobalScope,
        transformer_obj: SafeHandleObject,
        transformer: &Transformer,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TransformStreamDefaultController>> {
        // Let controller be a new TransformStreamDefaultController.
        let controller = TransformStreamDefaultController::new(global, transformer, can_gc);

        // Let transformAlgorithm be the following steps, taking a chunk argument:

        // Let result be TransformStreamDefaultControllerEnqueue(controller, chunk).

        // If result is an abrupt completion, return a promise rejected with result.[[Value]].

        // Otherwise, return a promise resolved with undefined.

        // Let flushAlgorithm be an algorithm which returns a promise resolved with undefined.

        // Let cancelAlgorithm be an algorithm which returns a promise resolved with undefined.

        // If transformerDict["transform"] exists, set transformAlgorithm to an algorithm which
        // takes an argument chunk and returns the result of invoking transformerDict["transform"]
        // with argument list « chunk, controller » and callback this value transformer.

        // If transformerDict["flush"] exists, set flushAlgorithm to an algorithm which returns the result
        // of invoking transformerDict["flush"] with argument list « controller » and callback this value transformer.

        // If transformerDict["cancel"] exists, set cancelAlgorithm to an algorithm which takes an argument
        //  reason and returns the result of invoking transformerDict["cancel"] with argument list « reason »
        //  and callback this value transformer.

        // Note: All these algorithms are set up in the constructor.

        // Note: this must be done before `setup`,
        // otherwise `thisOb` is null in the start callback.
        controller.set_transformer_this_object(transformer_obj);

        // Perform ! SetUpTransformStreamDefaultController(stream, controller,
        // transformAlgorithm, flushAlgorithm, cancelAlgorithm).
        controller.set_up(self)?;

        Ok(controller)
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
        writable_srategy: &QueuingStrategy,
        readable_strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<TransformStream>> {
        // If transformer is missing, set it to null.
        rooted!(in(*cx) let transformer_obj = transformer.unwrap_or(ptr::null_mut()));

        // Let transformerDict be transformer, converted to an IDL value of type Transformer.
        let transformer_dict = if !transformer_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(transformer_obj.get()));
            match Transformer::new(cx, obj_val.handle()) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => {
                    return Err(Error::Type(error.to_string()));
                },
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            Transformer::empty()
        };

        if !transformer_dict.readableType.handle().is_undefined() {
            // If transformerDict["readableType"] exists, throw a RangeError exception.
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

        // Let readableSizeAlgorithm be ! ExtractSizeAlgorithm(readableStrategy).
        let writable_high_water_mark = extract_high_water_mark(writable_srategy, 0.0)?;

        // Let writableSizeAlgorithm be ! ExtractSizeAlgorithm(writableStrategy).
        let writable_size_algorithm = extract_size_algorithm(writable_srategy, can_gc);

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
        let controlller = stream.set_up_transform_stream_default_controller_from_transformer(
            global,
            transformer_obj.handle(),
            &transformer_dict,
            can_gc,
        )?;

        if let Some(start) = &transformer_dict.start {
            // If transformerDict["start"] exists, then resolve startPromise with
            // the result of invoking transformerDict["start"]
            // with argument list « this.[[controller]] » and callback this value transformer.

            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = transformer_obj.get());
            start.Call_(
                &this_object.handle(),
                &controlller,
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

            let new_promise = if is_promise {
                Promise::new_with_js_promise(result_object.handle(), cx)
            } else {
                Promise::new_resolved(global, cx, result.get(), can_gc)
            };

            start_promise.resolve_native(&new_promise, can_gc);
        } else {
            // Otherwise, resolve startPromise with undefined.
            start_promise.resolve_native(&(), can_gc);
        }

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#ts-readable>
    fn Readable(&self) -> DomRoot<ReadableStream> {
        self.readable.get().expect("readable stream is null")
    }

    /// <https://streams.spec.whatwg.org/#ts-writable>
    fn Writable(&self) -> DomRoot<WritableStream> {
        self.writable.get().expect("writable stream is null")
    }
}
