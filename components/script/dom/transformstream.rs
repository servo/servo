/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self};
use std::rc::Rc;

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::HandleObject as SafeHandleObject;
use script_bindings::codegen::GenericBindings::TransformStreamBinding::TransformStreamMethods;
use script_bindings::codegen::GenericBindings::TransformerBinding::Transformer;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

use super::readablestream::ReadableStream;
use super::transformstreamdefaultcontroller::TransformStreamDefaultController;
use super::writablestream::WritableStream;

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

    pub(crate) fn writable(&self) -> DomRoot<WritableStream> {
        self.writable.get().expect("writable stream is null")
    }

    pub(crate) fn backpressure(&self) -> Option<bool> {
        self.backpressure.get()
    }

    pub(crate) fn backpressure_change_promise(&self) -> Option<Rc<Promise>> {
        self.backpressure_change_promise.borrow().clone()
    }

    pub(crate) fn set_backpressure_change_promise(&self, promise: Rc<Promise>) {
        *self.backpressure_change_promise.borrow_mut() = Some(promise);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-sink-write-algorithm>
    fn default_sink_write_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
        chunk: &Box<Heap<JSVal>>,
    ) -> Fallible<()> {
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
                rooted!(in(*cx) let mut val = UndefinedValue());
                let error = Error::Type("writable stream is erroring".to_string());
                writable.get_stored_error(val.handle_mut());
                // error.to_jsval(cx, global, val.handle_mut(), can_gc);
                // TODO: Set the error to the writable stream stored error.
                return Err(error);
            }

            // Assert: state is "writable".
            assert!(writable.is_writable());

            // Return ! TransformStreamDefaultControllerPerformTransform(controller, chunk).
        }

        todo!();
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-error-writable-and-unblock-write>
    pub(crate) fn error_writable_and_unblock_write(
        &self,
        global: &GlobalScope,
        cx: SafeJSContext,
        can_gc: CanGc,
        reason: js::gc::HandleValue,
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
    fn unblock_write(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<()> {
        // If stream.[[backpressure]] is true, perform ! TransformStreamSetBackpressure(stream, false).
        if self.backpressure.get().unwrap_or(false) {
            self.set_backpressure(false, global, can_gc);
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-set-backpressure>
    pub(crate) fn set_backpressure(
        &self,
        backpressure: bool,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Assert: stream.[[backpressure]] is not backpressure.
        assert!(self.backpressure.get() != Some(backpressure));

        // If stream.[[backpressureChangePromise]] is not undefined, resolve stream.[[backpressureChangePromise]] with undefined.
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
        can_gc: CanGc,
        reason: js::gc::HandleValue,
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
        self.error_writable_and_unblock_write(global, cx, can_gc, reason)
    }

    /// <https://streams.spec.whatwg.org/#initialize-transform-stream>
    fn initialize(
        &self,
        global: &GlobalScope,
        can_gc: CanGc,
        strategy_size: Rc<QueuingStrategySize>,
    ) -> Fallible<()> {
        // Let startAlgorithm be an algorithm that returns startPromise.
        // Let writeAlgorithm be the following steps, taking a chunk argument:
        // Let abortAlgorithm be the following steps, taking a reason argument:
        // Let closeAlgorithm be the following steps:

        // Set stream.[[backpressure]] and stream.[[backpressureChangePromise]] to undefined.
        self.backpressure.set(None);
        *self.backpressure_change_promise.borrow_mut() = None;

        // Perform ! TransformStreamSetBackpressure(stream, true).
        self.backpressure.set(Some(true));

        // Set stream.[[controller]] to undefined.
        self.controller.set(None);
        
        Ok(())
    }
}

impl TransformStreamMethods<crate::DomTypeHolder> for TransformStream {
    /// <https://streams.spec.whatwg.org/#ts-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        transformer: Option<*mut JSObject>,
        writable_srategy: &QueuingStrategy,
        readable_strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<TransformStream>> {
        // step 1. If transformer is missing, set it to null.
        rooted!(in(*cx) let transformer_obj = transformer.unwrap_or(ptr::null_mut()));

        // step 2. Let transformerDict be transformer, converted to an IDL value of type Transformer.
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

        if !transformer_dict.readableType.handle().is_undefined() {
            // step 3. If transformerDict["readableType"] exists, throw a RangeError exception.
            return Err(Error::Range("readableType is set".to_string()));
        }

        // step 4. If transformerDict["writableType"] exists, throw a RangeError exception.
        if !transformer_dict.writableType.handle().is_undefined() {
            return Err(Error::Range("writableType is set".to_string()));
        }

        // step 5. Let readableHighWaterMark be ? ExtractHighWaterMark(readableStrategy, 0).
        let readable_high_water_mark = extract_high_water_mark(readable_strategy, 0.0)?;

        // step 6. Let readableSizeAlgorithm be ! ExtractSizeAlgorithm(readableStrategy).
        let readable_size_algorithm = extract_size_algorithm(readable_strategy);

        // step 7. Let readableSizeAlgorithm be ! ExtractSizeAlgorithm(readableStrategy).
        let writable_high_water_mark = extract_high_water_mark(writable_srategy, 0.0)?;

        // step 8. Let writableSizeAlgorithm be ! ExtractSizeAlgorithm(writableStrategy).
        let writable_size_algorithm = extract_size_algorithm(writable_srategy);

        // step 9. Let startPromise be a new promise.
        let start_promise = Promise::new(global, can_gc);

        // step 10. Perform ! InitializeTransformStream(this, startPromise, writableHighWaterMark, writableSizeAlgorithm, readableHighWaterMark, readableSizeAlgorithm).

        todo!()
    }

    fn Readable(&self) -> DomRoot<ReadableStream> {
        todo!()
    }

    fn Writable(&self) -> DomRoot<WritableStream> {
        todo!()
    }
}
