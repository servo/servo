/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use super::bindings::reflector::reflect_dom_object;
use super::types::{TransformStream, UnderlyingSourceContainer};
use super::underlyingsourcecontainer::UnderlyingSourceType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use script_bindings::codegen::GenericBindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;

#[dom_struct]
pub struct TransformStreamDefaultController {
    reflector_: Reflector,
    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-cancelalgorithm>
    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-flushalgorithm>
    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-transformalgorithm
    underlying_source: MutNullableDom<UnderlyingSourceContainer>,
    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-stream>
    stream: MutNullableDom<TransformStream>,
    /// <https://streams.spec.whatwg.org/#transformstreamdefaultcontroller-flushalgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    finish_promise: DomRefCell<Rc<Promise>>,
}

impl TransformStreamDefaultController {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> TransformStreamDefaultController {
        TransformStreamDefaultController {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            underlying_source: MutNullableDom::new(Some(&*UnderlyingSourceContainer::new(
                global,
                underlying_source_type,
                can_gc,
            ))),
            finish_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        underlying_source: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> DomRoot<TransformStreamDefaultController> {
        reflect_dom_object(
            Box::new(TransformStreamDefaultController::new_inherited(
                global,
                underlying_source,
                can_gc,
            )),
            global,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-enqueue>
    fn enqueue(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
        chunk: js::gc::HandleValue,
    ) -> Fallible<()> {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().ok_or(Error::Type(
            "TransformStreamDefaultController stream is null".to_string(),
        ))?;

        // Let readableController be stream.[[readable]].[[controller]].
        let readable_controller = stream.readable().get_default_controller();

        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(readableController) is false, throw a TypeError exception.
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

            stream.error_writable_and_unblock_write(global, cx, can_gc, rval.handle())?;

            // Throw stream.[[readable]].[[storedError]].
            let readable = stream.readable();
            rooted!(in(*cx) let mut error = UndefinedValue());
            readable.get_stored_error(error.handle_mut());
            // TODO how to Throw storedError here
            return Err(Error::Type(
                "TransformStreamDefaultController enqueue error".to_string(),
            ));
        }

        // Let backpressure be ! ReadableStreamDefaultControllerHasBackpressure(readableController).
        let backpressure = readable_controller.has_backpressure();

        // If backpressure is not stream.[[backpressure]],
        if Some(backpressure) != stream.backpressure() {
            // Assert: backpressure is true.
            assert!(backpressure);

            // Perform ! TransformStreamSetBackpressure(stream, true).
            stream.set_backpressure(true, global, can_gc);
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-clear-algorithms>
    pub(crate) fn clear_algorithms(&self) {
        // Set controller.[[transformAlgorithm]] to undefined.
        // Set controller.[[flushAlgorithm]] to undefined.
        // Set controller.[[cancelAlgorithm]] to undefined.
        self.underlying_source.set(None);
    }

    /// <https://streams.spec.whatwg.org/#transform-stream-default-controller-error>
    pub(crate) fn error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
        reason: js::gc::HandleValue,
    ) -> Fallible<()> {
        // Perform ! TransformStreamError(controller.[[stream]], e).
        self.stream
            .get()
            .ok_or(Error::Type(
                "TransformStreamDefaultController stream is null".to_string(),
            ))?
            .error(cx, global, can_gc, reason)
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
        stream.error_writable_and_unblock_write(global, cx, can_gc, error.handle())
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
        self.enqueue(cx, &self.global(), can_gc, chunk)
    }

    fn Error(&self, cx: SafeJSContext, reason: js::gc::HandleValue, can_gc: CanGc) -> Fallible<()> {
        // Perform ? TransformStreamDefaultControllerError(this, e).
        self.error(cx, &self.global(), can_gc, reason)
    }

    fn Terminate(&self, can_gc: CanGc) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        let global = self.global();
        // Perform ? TransformStreamDefaultControllerTerminate(this).
        self.terminate(cx, &global, can_gc)
    }
}
