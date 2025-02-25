/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::ptr::{self};
use std::rc::Rc;
use std::collections::VecDeque;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};
use js::typedarray::ArrayBufferViewU8;

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::{
    ReadableStreamGetReaderOptions, ReadableStreamMethods, ReadableStreamReaderMode, StreamPipeOptions
};
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultController_Binding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::writablestream::WritableStream;
use crate::dom::bindings::codegen::UnionTypes::ReadableStreamDefaultReaderOrReadableStreamBYOBReader as ReadableStreamReader;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom, Dom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::readablestreambyobreader::ReadableStreamBYOBReader;
use crate::dom::readablestreamdefaultcontroller::ReadableStreamDefaultController;
use crate::dom::readablestreamdefaultreader::{ReadRequest, ReadableStreamDefaultReader};
use crate::dom::defaultteeunderlyingsource::TeeCancelAlgorithm;
use crate::dom::types::DefaultTeeUnderlyingSource;
use crate::dom::underlyingsourcecontainer::UnderlyingSourceType;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use crate::js::conversions::FromJSValConvertible;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderReadOptions;
use super::readablestreambyobreader::ReadIntoRequest;

/// State Machine for `PipeTo`.
/// Note that during normal operations we do not have to wait for writes to completes
/// before reading the next chunk;
/// we must only respect backpressure by waiting for the writer to be ready.
#[derive(Clone)]
enum PipeToState {
    /// Waiting for the writer to be ready
    PendingReady,
    /// Waiting for a read to resolve.
    PendingRead,
    /// Waiting until all pending writes have completed,
    /// at which point we can `finalize`.
    ShuttingDown,
}

/// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown-with-action>
enum ShutdownAction {
    WritableStreamAbort,
    ReadableStreamCancel,
    WritableStreamDefaultWriterCloseWithErrorPropagation,
}

impl js::gc::Rootable for PipeTo {}

/// The "in parallel, but not really" part of
/// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PipeTo {
    /// <https://streams.spec.whatwg.org/#ref-for-readablestream%E2%91%A7%E2%91%A0>
    source: Dom<ReadableStream>,

    /// <https://streams.spec.whatwg.org/#ref-for-acquire-writable-stream-default-writer>
    writer: Dom<WritableStreamDefaultWriter>,

    /// Pending writes are needed when shutting down(with an action),
    /// because we can only finalize when all writes are finished.
    #[ignore_malloc_size_of = "Rc are hard"]
    pending_writes: Rc<RefCell<VecDeque<Rc<Promise>>>>,

    /// The state machine.
    #[ignore_malloc_size_of = "Rc are hard"]
    #[no_trace]
    state: Rc<RefCell<PipeToState>>,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventabort>
    prevent_abort: bool,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventcancel>
    prevent_cancel: bool,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventclose>
    prevent_close: bool,
}

impl PipeTo {
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    /// Errors must be propagated forward.
    fn check_and_propagate_errors_forward(&self, cx: SafeJSContext) {
        // if source.[[state]] is or becomes "errored", then
        if self.source.is_errored() {
            rooted!(in(*cx) let original_error = self.source.stored_error.get());
            // If prevent_abort is false,
            if !self.prevent_abort {
                // shutdown with an action of ! WritableStreamAbort(dest, source.[[storedError]])
                // and with source.[[storedError]].
                self.shutdown_with_an_action(
                    ShutdownAction::WritableStreamAbort,
                    Some(original_error.handle()),
                )
            } else {
                // Otherwise, shutdown with source.[[storedError]].
                self.shutdown(Some(original_error.handle()));
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    /// Errors must be propagated backward
    fn check_and_propagate_errors_backward(&self, cx: SafeJSContext) {
        // if dest.[[state]] is or becomes "errored", then
        let dest = self.writer.get_stream().expect("Stream must be set");
        if dest.is_errored() {
            rooted!(in(*cx) let mut original_error = UndefinedValue());
            dest.get_stored_error(original_error.handle_mut());
            // If preventCancel is false,
            if !self.prevent_cancel {
                // shutdown with an action of ! ReadableStreamCancel(source, dest.[[storedError]])
                // and with dest.[[storedError]].
                self.shutdown_with_an_action(
                    ShutdownAction::ReadableStreamCancel,
                    Some(original_error.handle()),
                )
            } else {
                // Otherwise, shutdown with dest.[[storedError]].
                self.shutdown(Some(original_error.handle()));
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown-with-action>
    fn shutdown_with_an_action(
        &self,
        action: ShutdownAction,
        original_error: Option<SafeHandleValue>,
    ) {
    }

    /// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown>
    fn shutdown(&self, original_error: Option<SafeHandleValue>) {}
}

/// The fulfillment handler for the reacting to sourceCancelPromise part of
/// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct SourceCancelPromiseFulfillmentHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    result: Rc<Promise>,
}

impl Callback for SourceCancelPromiseFulfillmentHandler {
    /// The fulfillment handler for the reacting to sourceCancelPromise part of
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        self.result.resolve_native(&(), can_gc);
    }
}

/// The rejection handler for the reacting to sourceCancelPromise part of
/// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct SourceCancelPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    result: Rc<Promise>,
}

impl Callback for SourceCancelPromiseRejectionHandler {
    /// The rejection handler for the reacting to sourceCancelPromise part of
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        self.result.reject_native(&v, can_gc);
    }
}

/// <https://streams.spec.whatwg.org/#readablestream-state>
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ReadableStreamState {
    #[default]
    Readable,
    Closed,
    Errored,
}

/// <https://streams.spec.whatwg.org/#readablestream-controller>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ControllerType {
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
    Byte(MutNullableDom<ReadableByteStreamController>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
    Default(MutNullableDom<ReadableStreamDefaultController>),
}

/// <https://streams.spec.whatwg.org/#readablestream-readerr>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ReaderType {
    /// <https://streams.spec.whatwg.org/#readablestreambyobreader>
    #[allow(clippy::upper_case_acronyms)]
    BYOB(MutNullableDom<ReadableStreamBYOBReader>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
    Default(MutNullableDom<ReadableStreamDefaultReader>),
}

impl Eq for ReaderType {}
impl PartialEq for ReaderType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ReaderType::BYOB(_), ReaderType::BYOB(_)) |
                (ReaderType::Default(_), ReaderType::Default(_))
        )
    }
}

/// <https://streams.spec.whatwg.org/#create-readable-stream>
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
fn create_readable_stream(
    global: &GlobalScope,
    underlying_source_type: UnderlyingSourceType,
    queuing_strategy: QueuingStrategy,
    can_gc: CanGc,
) -> DomRoot<ReadableStream> {
    // If highWaterMark was not passed, set it to 1.
    let high_water_mark = queuing_strategy.highWaterMark.unwrap_or(1.0);

    // If sizeAlgorithm was not passed, set it to an algorithm that returns 1.
    let size_algorithm = queuing_strategy
        .size
        .unwrap_or(extract_size_algorithm(&QueuingStrategy::empty(), can_gc));

    // Assert: ! IsNonNegativeNumber(highWaterMark) is true.
    assert!(high_water_mark >= 0.0);

    // Let stream be a new ReadableStream.
    // Perform ! InitializeReadableStream(stream).
    let stream = ReadableStream::new_with_proto(global, None, can_gc);

    // Let controller be a new ReadableStreamDefaultController.
    let controller = ReadableStreamDefaultController::new(
        global,
        underlying_source_type,
        high_water_mark,
        size_algorithm,
        can_gc,
    );

    // Perform ? SetUpReadableStreamDefaultController(stream, controller, startAlgorithm,
    // pullAlgorithm, cancelAlgorithm, highWaterMark, sizeAlgorithm).
    controller
        .setup(stream.clone(), can_gc)
        .expect("Setup of default controller cannot fail");

    // Return stream.
    stream
}

/// <https://streams.spec.whatwg.org/#rs-class>
#[dom_struct]
pub(crate) struct ReadableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestream-controller>
    /// Note: the inner `MutNullableDom` should really be an `Option<Dom>`,
    /// because it is never unset once set.
    controller: RefCell<Option<ControllerType>>,

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    #[ignore_malloc_size_of = "mozjs"]
    stored_error: Heap<JSVal>,

    /// <https://streams.spec.whatwg.org/#readablestream-disturbed>
    disturbed: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestream-reader>
    reader: RefCell<Option<ReaderType>>,

    /// <https://streams.spec.whatwg.org/#readablestream-state>
    state: Cell<ReadableStreamState>,
}

impl ReadableStream {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#initialize-readable-stream>
    fn new_inherited() -> ReadableStream {
        ReadableStream {
            reflector_: Reflector::new(),
            controller: RefCell::new(None),
            stored_error: Heap::default(),
            disturbed: Default::default(),
            reader: RefCell::new(None),
            state: Cell::new(Default::default()),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStream> {
        reflect_dom_object_with_proto(
            Box::new(ReadableStream::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub(crate) fn set_default_controller(&self, controller: &ReadableStreamDefaultController) {
        *self.controller.borrow_mut() = Some(ControllerType::Default(MutNullableDom::new(Some(
            controller,
        ))));
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
    pub(crate) fn set_byte_controller(&self, controller: &ReadableByteStreamController) {
        *self.controller.borrow_mut() =
            Some(ControllerType::Byte(MutNullableDom::new(Some(controller))));
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub(crate) fn assert_no_controller(&self) {
        let has_no_controller = self.controller.borrow().is_none();
        assert!(has_no_controller);
    }

    /// Build a stream backed by a Rust source that has already been read into memory.
    pub(crate) fn new_from_bytes(
        global: &GlobalScope,
        bytes: Vec<u8>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        let stream = ReadableStream::new_with_external_underlying_source(
            global,
            UnderlyingSourceType::Memory(bytes.len()),
            can_gc,
        )?;
        stream.enqueue_native(bytes, can_gc);
        stream.controller_close_native(can_gc);
        Ok(stream)
    }

    /// Build a stream backed by a Rust underlying source.
    /// Note: external sources are always paired with a default controller.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_external_underlying_source(
        global: &GlobalScope,
        source: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        assert!(source.is_native());
        let stream = ReadableStream::new_with_proto(global, None, can_gc);
        let controller = ReadableStreamDefaultController::new(
            global,
            source,
            1.0,
            extract_size_algorithm(&QueuingStrategy::empty(), can_gc),
            can_gc,
        );
        controller.setup(stream.clone(), can_gc)?;
        Ok(stream)
    }

    /// Call into the release steps of the controller,
    pub(crate) fn perform_release_steps(&self) -> Fallible<()> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                let controller = controller
                    .get()
                    .ok_or_else(|| Error::Type("Stream should have controller.".to_string()))?;
                controller.perform_release_steps()
            },
            Some(ControllerType::Byte(controller)) => {
                let controller = controller
                    .get()
                    .ok_or_else(|| Error::Type("Stream should have controller.".to_string()))?;
                controller.perform_release_steps()
            },
            None => Err(Error::Type("Stream should have controller.".to_string())),
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub(crate) fn perform_pull_steps(
        &self,
        cx: SafeJSContext,
        read_request: &ReadRequest,
        can_gc: CanGc,
    ) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_steps(read_request, can_gc),
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_steps(cx, read_request, can_gc),
            None => {
                unreachable!("Stream does not have a controller.");
            },
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-byob-reader-read>
    pub(crate) fn perform_pull_into(
        &self,
        cx: SafeJSContext,
        read_into_request: &ReadIntoRequest,
        view: HeapBufferSource<ArrayBufferViewU8>,
        options: &ReadableStreamBYOBReaderReadOptions,
        can_gc: CanGc,
    ) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_into(cx, read_into_request, view, options, can_gc),
            _ => {
                unreachable!(
                    "Pulling a chunk from a stream with a default controller using a BYOB reader"
                )
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub(crate) fn add_read_request(&self, read_request: &ReadRequest) {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to add a read request without having first acquired a reader.");
                };

                // Assert: stream.[[state]] is "readable".
                assert!(self.is_readable());

                // Append readRequest to stream.[[reader]].[[readRequests]].
                reader.add_read_request(read_request);
            },
            _ => {
                unreachable!("Adding a read request can only be done on a default reader.")
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-into-request>
    pub(crate) fn add_read_into_request(&self, read_request: &ReadIntoRequest) {
        match self.reader.borrow().as_ref() {
            // Assert: stream.[[reader]] implements ReadableStreamBYOBReader.
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Attempt to add a read into request without having first acquired a reader."
                    );
                };

                // Assert: stream.[[state]] is "readable" or "closed".
                assert!(self.is_readable() || self.is_closed());

                // Append readRequest to stream.[[reader]].[[readIntoRequests]].
                reader.add_read_into_request(read_request);
            },
            _ => {
                unreachable!("Adding a read into request can only be done on a BYOB reader.")
            },
        }
    }

    /// Endpoint to enqueue chunks directly from Rust.
    /// Note: in other use cases this call happens via the controller.
    pub(crate) fn enqueue_native(&self, bytes: Vec<u8>, can_gc: CanGc) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .enqueue_native(bytes, can_gc),
            _ => {
                unreachable!(
                    "Enqueueing chunk to a stream from Rust on other than default controller"
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub(crate) fn error(&self, e: SafeHandleValue, can_gc: CanGc) {
        // Assert: stream.[[state]] is "readable".
        assert!(self.is_readable());

        // Set stream.[[state]] to "errored".
        self.state.set(ReadableStreamState::Errored);

        // Set stream.[[storedError]] to e.
        self.stored_error.set(e.get());

        // Let reader be stream.[[reader]].

        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    // If reader is undefined, return.
                    return;
                };

                // Perform ! ReadableStreamDefaultReaderErrorReadRequests(reader, e).
                reader.error(e, can_gc);
            },
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    // If reader is undefined, return.
                    return;
                };

                // Perform ! ReadableStreamBYOBReaderErrorReadIntoRequests(reader, e).
                reader.error_read_into_requests(e, can_gc);
            },
            None => {
                // If reader is undefined, return.
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    pub(crate) fn get_stored_error(&self, mut handle_mut: SafeMutableHandleValue) {
        handle_mut.set(self.stored_error.get());
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    /// Note: in other use cases this call happens via the controller.
    pub(crate) fn error_native(&self, error: Error, can_gc: CanGc) {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error_val = UndefinedValue());
        error.to_jsval(cx, &self.global(), error_val.handle_mut(), can_gc);
        self.error(error_val.handle(), can_gc);
    }

    /// Call into the controller's `Close` method.
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub(crate) fn controller_close_native(&self, can_gc: CanGc) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                let _ = controller
                    .get()
                    .expect("Stream should have controller.")
                    .Close(can_gc);
            },
            _ => {
                unreachable!("Native closing is only done on default controllers.")
            },
        }
    }

    /// Returns a boolean reflecting whether the stream has all data in memory.
    /// Useful for native source integration only.
    pub(crate) fn in_memory(&self) -> bool {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .in_memory(),
            _ => {
                unreachable!(
                    "Checking if source is in memory for a stream with a non-default controller"
                )
            },
        }
    }

    /// Return bytes for synchronous use, if the stream has all data in memory.
    /// Useful for native source integration only.
    pub(crate) fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .get_in_memory_bytes(),
            _ => {
                unreachable!("Getting in-memory bytes for a stream with a non-default controller")
            },
        }
    }

    /// Acquires a reader and locks the stream,
    /// must be done before `read_a_chunk`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-reader>
    pub(crate) fn acquire_default_reader(
        &self,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStreamDefaultReader>> {
        // Let reader be a new ReadableStreamDefaultReader.
        let reader = ReadableStreamDefaultReader::new(&self.global(), can_gc);

        // Perform ? SetUpReadableStreamDefaultReader(reader, stream).
        reader.set_up(self, &self.global(), can_gc)?;

        // Return reader.
        Ok(reader)
    }

    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-byob-reader>
    pub(crate) fn acquire_byob_reader(
        &self,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStreamBYOBReader>> {
        // Let reader be a new ReadableStreamBYOBReader.
        let reader = ReadableStreamBYOBReader::new(&self.global(), can_gc);
        // Perform ? SetUpReadableStreamBYOBReader(reader, stream).
        reader.set_up(self, &self.global(), can_gc)?;

        // Return reader.
        Ok(reader)
    }

    pub(crate) fn get_default_controller(&self) -> DomRoot<ReadableStreamDefaultController> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                controller.get().expect("Stream should have controller.")
            },
            _ => {
                unreachable!(
                    "Getting default controller for a stream with a non-default controller"
                )
            },
        }
    }

    pub(crate) fn get_default_reader(&self) -> DomRoot<ReadableStreamDefaultReader> {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().expect("Stream should have reader."),
            _ => {
                unreachable!("Getting default reader for a stream with a non-default reader")
            },
        }
    }

    /// Read a chunk from the stream,
    /// must be called after `start_reading`,
    /// and before `stop_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub(crate) fn read_a_chunk(&self, can_gc: CanGc) -> Rc<Promise> {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Attempt to read stream chunk without having first acquired a reader."
                    );
                };
                reader.Read(can_gc)
            },
            _ => {
                unreachable!("Native reading of a chunk can only be done with a default reader.")
            },
        }
    }

    /// Releases the lock on the reader,
    /// must be done after `start_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    pub(crate) fn stop_reading(&self, can_gc: CanGc) {
        let reader_ref = self.reader.borrow();

        match reader_ref.as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!("Attempt to stop reading without having first acquired a reader.");
                };

                drop(reader_ref);
                reader.release(can_gc).expect("Reader release cannot fail.");
            },
            _ => {
                unreachable!("Native stop reading can only be done with a default reader.")
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#is-readable-stream-locked>
    pub(crate) fn is_locked(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().is_some(),
            Some(ReaderType::BYOB(reader)) => reader.get().is_some(),
            None => false,
        }
    }

    pub(crate) fn is_disturbed(&self) -> bool {
        self.disturbed.get()
    }

    pub(crate) fn set_is_disturbed(&self, disturbed: bool) {
        self.disturbed.set(disturbed);
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.state.get() == ReadableStreamState::Closed
    }

    pub(crate) fn is_errored(&self) -> bool {
        self.state.get() == ReadableStreamState::Errored
    }

    pub(crate) fn is_readable(&self) -> bool {
        self.state.get() == ReadableStreamState::Readable
    }

    pub(crate) fn has_default_reader(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().is_some(),
            _ => false,
        }
    }

    pub(crate) fn has_byob_reader(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => reader.get().is_some(),
            _ => false,
        }
    }

    pub(crate) fn has_byte_controller(&self) -> bool {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Byte(controller)) => controller.get().is_some(),
            _ => false,
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub(crate) fn get_num_read_requests(&self) -> usize {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when getting the number of read requests.");
                reader.get_num_read_requests()
            },
            _ => unreachable!(
                "Stream must have a default reader when get num read requests is called into."
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-into-requests>
    pub(crate) fn get_num_read_into_requests(&self) -> usize {
        assert!(self.has_byob_reader());

        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Stream must have a reader when get num read into requests is called into."
                    );
                };
                reader.get_num_read_into_requests()
            },
            _ => {
                unreachable!(
                    "Stream must have a BYOB reader when get num read into requests is called into."
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn fulfill_read_request(&self, chunk: SafeHandleValue, done: bool, can_gc: CanGc) {
        // step 1 - Assert: ! ReadableStreamHasDefaultReader(stream) is true.
        assert!(self.has_default_reader());

        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                // step 2 - Let reader be stream.[[reader]].
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when a read request is fulfilled.");
                // step 3 - Assert: reader.[[readRequests]] is not empty.
                assert_ne!(reader.get_num_read_requests(), 0);
                // step 4 & 5
                // Let readRequest be reader.[[readRequests]][0]. & Remove readRequest from reader.[[readRequests]].
                let request = reader.remove_read_request();

                if done {
                    // step 6 - If done is true, perform readRequest’s close steps.
                    request.close_steps(can_gc);
                } else {
                    // step 7 - Otherwise, perform readRequest’s chunk steps, given chunk.
                    let result = RootedTraceableBox::new(Heap::default());
                    result.set(*chunk);
                    request.chunk_steps(result, can_gc);
                }
            },
            _ => {
                unreachable!(
                    "Stream must have a default reader when fulfill read requests is called into."
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-into-request>
    pub(crate) fn fulfill_read_into_request(
        &self,
        chunk: SafeHandleValue,
        done: bool,
        can_gc: CanGc,
    ) {
        // Assert: ! ReadableStreamHasBYOBReader(stream) is true.
        assert!(self.has_byob_reader());

        // Let reader be stream.[[reader]].
        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Stream must have a reader when a read into request is fulfilled."
                    );
                };

                // Assert: reader.[[readIntoRequests]] is not empty.
                assert!(reader.get_num_read_into_requests() > 0);

                // Let readIntoRequest be reader.[[readIntoRequests]][0].
                // Remove readIntoRequest from reader.[[readIntoRequests]].
                let read_into_request = reader.remove_read_into_request();

                // If done is true, perform readIntoRequest’s close steps, given chunk.
                let result = RootedTraceableBox::new(Heap::default());
                if done {
                    result.set(*chunk);
                    read_into_request.close_steps(Some(result), can_gc);
                } else {
                    // Otherwise, perform readIntoRequest’s chunk steps, given chunk.
                    result.set(*chunk);
                    read_into_request.chunk_steps(result, can_gc);
                }
            },
            _ => {
                unreachable!(
                    "Stream must have a BYOB reader when fulfill read into requests is called into."
                );
            },
        };
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub(crate) fn close(&self, can_gc: CanGc) {
        // Assert: stream.[[state]] is "readable".
        assert!(self.is_readable());
        // Set stream.[[state]] to "closed".
        self.state.set(ReadableStreamState::Closed);
        // Let reader be stream.[[reader]].
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    // If reader is undefined, return.
                    return;
                };
                // step 5 & 6
                reader.close(can_gc);
            },
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    // If reader is undefined, return.
                    return;
                };

                reader.close(can_gc)
            },
            None => {
                // If reader is undefined, return.
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    #[allow(unsafe_code)]
    pub(crate) fn cancel(&self, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // Set stream.[[disturbed]] to true.
        self.disturbed.set(true);

        // If stream.[[state]] is "closed", return a promise resolved with undefined.
        if self.is_closed() {
            return Promise::new_resolved(&self.global(), GlobalScope::get_cx(), (), can_gc);
        }
        // If stream.[[state]] is "errored", return a promise rejected with stream.[[storedError]].
        if self.is_errored() {
            let promise = Promise::new(&self.global(), can_gc);
            unsafe {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                self.stored_error.to_jsval(*cx, rval.handle_mut());
                promise.reject_native(&rval.handle(), can_gc);
                return promise;
            }
        }
        // Perform ! ReadableStreamClose(stream).
        self.close(can_gc);

        // If reader is not undefined and reader implements ReadableStreamBYOBReader,
        if let Some(ReaderType::BYOB(reader)) = self.reader.borrow().as_ref() {
            if let Some(reader) = reader.get() {
                // step 6.1, 6.2 & 6.3 of https://streams.spec.whatwg.org/#readable-stream-cancel
                reader.cancel(can_gc);
            }
        }

        // Let sourceCancelPromise be ! stream.[[controller]].[[CancelSteps]](reason).

        let source_cancel_promise = match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_cancel_steps(reason, can_gc),
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_cancel_steps(reason, can_gc),
            None => {
                panic!("Stream does not have a controller.");
            },
        };

        // Create a new promise,
        // and setup a handler in order to react to the fulfillment of sourceCancelPromise.
        let global = self.global();
        let result_promise = Promise::new(&global, can_gc);
        let fulfillment_handler = Box::new(SourceCancelPromiseFulfillmentHandler {
            result: result_promise.clone(),
        });
        let rejection_handler = Box::new(SourceCancelPromiseRejectionHandler {
            result: result_promise.clone(),
        });
        let handler = PromiseNativeHandler::new(
            &global,
            Some(fulfillment_handler),
            Some(rejection_handler),
            can_gc,
        );
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        source_cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return the result of reacting to sourceCancelPromise
        // with a fulfillment step that returns undefined.
        result_promise
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_reader(&self, new_reader: Option<ReaderType>) {
        *self.reader.borrow_mut() = new_reader;
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn default_tee(
        &self,
        clone_for_branch_2: bool,
        can_gc: CanGc,
    ) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Assert: stream implements ReadableStream.

        // Assert: cloneForBranch2 is a boolean.
        let clone_for_branch_2 = Rc::new(Cell::new(clone_for_branch_2));

        // Let reader be ? AcquireReadableStreamDefaultReader(stream).
        let reader = self.acquire_default_reader(can_gc)?;
        self.set_reader(Some(ReaderType::Default(MutNullableDom::new(Some(
            &reader,
        )))));

        // Let reading be false.
        let reading = Rc::new(Cell::new(false));
        // Let readAgain be false.
        let read_again = Rc::new(Cell::new(false));
        // Let canceled1 be false.
        let canceled_1 = Rc::new(Cell::new(false));
        // Let canceled2 be false.
        let canceled_2 = Rc::new(Cell::new(false));

        // Let reason1 be undefined.
        let reason_1 = Rc::new(Heap::boxed(UndefinedValue()));
        // Let reason2 be undefined.
        let reason_2 = Rc::new(Heap::boxed(UndefinedValue()));
        // Let cancelPromise be a new promise.
        let cancel_promise = Promise::new(&self.global(), can_gc);

        let tee_source_1 = DefaultTeeUnderlyingSource::new(
            &reader,
            self,
            reading.clone(),
            read_again.clone(),
            canceled_1.clone(),
            canceled_2.clone(),
            clone_for_branch_2.clone(),
            reason_1.clone(),
            reason_2.clone(),
            cancel_promise.clone(),
            TeeCancelAlgorithm::Cancel1Algorithm,
            can_gc,
        );

        let underlying_source_type_branch_1 =
            UnderlyingSourceType::Tee(Dom::from_ref(&tee_source_1));

        let tee_source_2 = DefaultTeeUnderlyingSource::new(
            &reader,
            self,
            reading,
            read_again,
            canceled_1.clone(),
            canceled_2.clone(),
            clone_for_branch_2,
            reason_1,
            reason_2,
            cancel_promise.clone(),
            TeeCancelAlgorithm::Cancel2Algorithm,
            can_gc,
        );

        let underlying_source_type_branch_2 =
            UnderlyingSourceType::Tee(Dom::from_ref(&tee_source_2));

        // Set branch_1 to ! CreateReadableStream(startAlgorithm, pullAlgorithm, cancel1Algorithm).
        let branch_1 = create_readable_stream(
            &self.global(),
            underlying_source_type_branch_1,
            QueuingStrategy::empty(),
            can_gc,
        );
        tee_source_1.set_branch_1(&branch_1);
        tee_source_2.set_branch_1(&branch_1);

        // Set branch_2 to ! CreateReadableStream(startAlgorithm, pullAlgorithm, cancel2Algorithm).
        let branch_2 = create_readable_stream(
            &self.global(),
            underlying_source_type_branch_2,
            QueuingStrategy::empty(),
            can_gc,
        );
        tee_source_1.set_branch_2(&branch_2);
        tee_source_2.set_branch_2(&branch_2);

        // Upon rejection of reader.[[closedPromise]] with reason r,
        reader.append_native_handler_to_closed_promise(
            &branch_1,
            &branch_2,
            canceled_1,
            canceled_2,
            cancel_promise,
            can_gc,
        );

        // Return « branch_1, branch_2 ».
        Ok(vec![branch_1, branch_2])
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    fn pipe_to(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        dest: &WritableStream,
        prevent_abort: bool,
        prevent_cancel: bool,
        prevent_close: bool,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Assert: source implements ReadableStream.
        // Assert: dest implements WritableStream.
        // Assert: prevent_close, prevent_abort, and prevent_cancel are all booleans.
        // Done with method signature types.

        // If signal was not given, let signal be undefined.
        // Assert: either signal is undefined, or signal implements AbortSignal.
        // TODO: implement AbortSignal.

        // Assert: ! IsReadableStreamLocked(source) is false.
        assert!(!self.is_locked());

        // Assert: ! IsWritableStreamLocked(dest) is false.
        assert!(!dest.is_locked());

        // If source.[[controller]] implements ReadableByteStreamController,
        // let reader be either ! AcquireReadableStreamBYOBReader(source)
        // or ! AcquireReadableStreamDefaultReader(source),
        // at the user agent’s discretion.
        // Note: for now only supporting default readers.

        // Otherwise, let reader be ! AcquireReadableStreamDefaultReader(source).
        let reader = self
            .acquire_default_reader(can_gc)
            .expect("Acquiring a default reader for pipe_to cannot fail");

        // Let writer be ! AcquireWritableStreamDefaultWriter(dest).
        let writer = dest
            .aquire_default_writer(cx, global, can_gc)
            .expect("Acquiring a default writer for pipe_to cannot fail");

        // Set source.[[disturbed]] to true.
        self.disturbed.set(true);

        // Let shuttingDown be false.
        let shutting_down = false;

        // Let promise be a new promise.
        let promise = Promise::new(global, can_gc);

        // If signal is not undefined,
        // TODO: implement AbortSignal.

        // In parallel, but not really, using reader and writer, read all chunks from source and write them to dest.
        //
        // Note: the spec is flexible about how this is done, but requires the following constraints to apply:
        // - Public API must not be used: we'll only use the internal APIs.
        // - Backpressure must be enforced: we'll do this by pulling a chunk from `source`, and then writing it to `dest`,
        //   whenever `dest` is ready, which is when the ready promise resolves.
        // - Shutdown must stop activity: we'll do this by checking `shuttingDown` before performing any reads.
        // - Error and close states must be propagated: we'll do this by checking these states at every step.

        // TODO

        promise
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-tee>
    fn tee(
        &self,
        clone_for_branch_2: bool,
        can_gc: CanGc,
    ) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Assert: stream implements ReadableStream.
        // Assert: cloneForBranch2 is a boolean.

        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(_)) => {
                // Return ? ReadableStreamDefaultTee(stream, cloneForBranch2).
                self.default_tee(clone_for_branch_2, can_gc)
            },
            Some(ControllerType::Byte(_)) => {
                // If stream.[[controller]] implements ReadableByteStreamController,
                // return ? ReadableByteStreamTee(stream).
                Err(Error::Type(
                    "Teeing is not yet supported for byte streams".to_owned(),
                ))
            },
            None => {
                unreachable!("Stream should have a controller.");
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller-from-underlying-source>
    pub(crate) fn set_up_byte_controller(
        &self,
        global: &GlobalScope,
        underlying_source_dict: JsUnderlyingSource,
        underlying_source_handle: SafeHandleObject,
        stream: DomRoot<ReadableStream>,
        strategy_hwm: f64,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let pullAlgorithm be an algorithm that returns a promise resolved with undefined.
        // Let cancelAlgorithm be an algorithm that returns a promise resolved with undefined.
        // If underlyingSourceDict["start"] exists, then set startAlgorithm to an algorithm which returns the result
        // of invoking underlyingSourceDict["start"] with argument list « controller »
        // and callback this value underlyingSource.
        // If underlyingSourceDict["pull"] exists, then set pullAlgorithm to an algorithm which returns the result
        // of invoking underlyingSourceDict["pull"] with argument list « controller »
        // and callback this value underlyingSource.
        // If underlyingSourceDict["cancel"] exists, then set cancelAlgorithm to an algorithm which takes an
        // argument reason and returns the result of invoking underlyingSourceDict["cancel"] with argument list
        // « reason » and callback this value underlyingSource.

        // Let autoAllocateChunkSize be underlyingSourceDict["autoAllocateChunkSize"],
        // if it exists, or undefined otherwise.
        // If autoAllocateChunkSize is 0, then throw a TypeError exception.
        if let Some(0) = underlying_source_dict.autoAllocateChunkSize {
            return Err(Error::Type("autoAllocateChunkSize cannot be 0".to_owned()));
        }

        let controller = ReadableByteStreamController::new(
            UnderlyingSourceType::Js(underlying_source_dict, Heap::default()),
            strategy_hwm,
            global,
            can_gc,
        );

        // Note: this must be done before `setup`,
        // otherwise `thisOb` is null in the start callback.
        controller.set_underlying_source_this_object(underlying_source_handle);

        // Perform ? SetUpReadableByteStreamController(stream, controller, startAlgorithm,
        // pullAlgorithm, cancelAlgorithm, highWaterMark, autoAllocateChunkSize).
        controller.setup(global, stream, can_gc)
    }
}

impl ReadableStreamMethods<crate::DomTypeHolder> for ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        underlying_source: Option<*mut JSObject>,
        strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<Self>> {
        // If underlyingSource is missing, set it to null.
        rooted!(in(*cx) let underlying_source_obj = underlying_source.unwrap_or(ptr::null_mut()));
        // Let underlyingSourceDict be underlyingSource,
        // converted to an IDL value of type UnderlyingSource.
        let underlying_source_dict = if !underlying_source_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(underlying_source_obj.get()));
            match JsUnderlyingSource::new(cx, obj_val.handle()) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => return Err(Error::Type(error.to_string())),
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            JsUnderlyingSource::empty()
        };

        // Perform ! InitializeReadableStream(this).
        let stream = ReadableStream::new_with_proto(global, proto, can_gc);

        if underlying_source_dict.type_.is_some() {
            // If strategy["size"] exists, throw a RangeError exception.
            if strategy.size.is_some() {
                return Err(Error::Range(
                    "size is not supported for byte streams".to_owned(),
                ));
            }

            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 0).
            let strategy_hwm = extract_high_water_mark(strategy, 0.0)?;

            // Perform ? SetUpReadableByteStreamControllerFromUnderlyingSource(this,
            // underlyingSource, underlyingSourceDict, highWaterMark).
            stream.set_up_byte_controller(
                global,
                underlying_source_dict,
                underlying_source_obj.handle(),
                stream.clone(),
                strategy_hwm,
                can_gc,
            )?;
        } else {
            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
            let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

            // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
            let size_algorithm = extract_size_algorithm(strategy, can_gc);

            let controller = ReadableStreamDefaultController::new(
                global,
                UnderlyingSourceType::Js(underlying_source_dict, Heap::default()),
                high_water_mark,
                size_algorithm,
                can_gc,
            );

            // Note: this must be done before `setup`,
            // otherwise `thisOb` is null in the start callback.
            controller.set_underlying_source_this_object(underlying_source_obj.handle());

            // Perform ? SetUpReadableStreamDefaultControllerFromUnderlyingSource
            controller.setup(stream.clone(), can_gc)?;
        };

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#rs-locked>
    fn Locked(&self) -> bool {
        self.is_locked()
    }

    /// <https://streams.spec.whatwg.org/#rs-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        if self.is_locked() {
            // If ! IsReadableStreamLocked(this) is true,
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&self.global(), can_gc);
            promise.reject_error(Error::Type("stream is not locked".to_owned()), can_gc);
            promise
        } else {
            // Return ! ReadableStreamCancel(this, reason).
            self.cancel(reason, can_gc)
        }
    }

    /// <https://streams.spec.whatwg.org/#rs-get-reader>
    fn GetReader(
        &self,
        options: &ReadableStreamGetReaderOptions,
        can_gc: CanGc,
    ) -> Fallible<ReadableStreamReader> {
        // 1, If options["mode"] does not exist, return ? AcquireReadableStreamDefaultReader(this).
        if options.mode.is_none() {
            return Ok(ReadableStreamReader::ReadableStreamDefaultReader(
                self.acquire_default_reader(can_gc)?,
            ));
        }
        // 2. Assert: options["mode"] is "byob".
        assert!(options.mode.unwrap() == ReadableStreamReaderMode::Byob);

        // 3. Return ? AcquireReadableStreamBYOBReader(this).
        Ok(ReadableStreamReader::ReadableStreamBYOBReader(
            self.acquire_byob_reader(can_gc)?,
        ))
    }

    /// <https://streams.spec.whatwg.org/#rs-tee>
    fn Tee(&self, can_gc: CanGc) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Return ? ReadableStreamTee(this, false).
        self.tee(false, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rs-pipe-to>
    fn PipeTo(
        &self,
        destination: &WritableStream,
        options: &StreamPipeOptions,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        let global = GlobalScope::from_safe_context(cx, realm);

        // If ! IsReadableStreamLocked(this) is true,
        if self.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Source stream is locked".to_owned()), can_gc);
            return promise;
        }

        // If ! IsWritableStreamLocked(destination) is true,
        if destination.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Destination stream is locked".to_owned()), can_gc);
            return promise;
        }

        // Let signal be options["signal"] if it exists, or undefined otherwise.
        // TODO: implement AbortSignal.

        // Return ! ReadableStreamPipeTo.
        self.pipe_to(
            cx,
            &global,
            destination,
            options.preventAbort,
            options.preventCancel,
            options.preventClose,
            can_gc,
        )
    }
}

#[allow(unsafe_code)]
/// Get the `done` property of an object that a read promise resolved to.
pub(crate) fn get_read_promise_done(cx: SafeJSContext, v: &SafeHandleValue) -> Result<bool, Error> {
    if !v.is_object() {
        return Err(Error::Type("Unknown format for done property.".to_string()));
    }
    unsafe {
        rooted!(in(*cx) let object = v.to_object());
        rooted!(in(*cx) let mut done = UndefinedValue());
        match get_dictionary_property(*cx, object.handle(), "done", done.handle_mut()) {
            Ok(true) => match bool::from_jsval(*cx, done.handle(), ()) {
                Ok(ConversionResult::Success(val)) => Ok(val),
                Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.to_string())),
                _ => Err(Error::Type("Unknown format for done property.".to_string())),
            },
            Ok(false) => Err(Error::Type("Promise has no done property.".to_string())),
            Err(()) => Err(Error::JSFailed),
        }
    }
}

#[allow(unsafe_code)]
/// Get the `value` property of an object that a read promise resolved to.
pub(crate) fn get_read_promise_bytes(
    cx: SafeJSContext,
    v: &SafeHandleValue,
) -> Result<Vec<u8>, Error> {
    if !v.is_object() {
        return Err(Error::Type(
            "Unknown format for for bytes read.".to_string(),
        ));
    }
    unsafe {
        rooted!(in(*cx) let object = v.to_object());
        rooted!(in(*cx) let mut bytes = UndefinedValue());
        match get_dictionary_property(*cx, object.handle(), "value", bytes.handle_mut()) {
            Ok(true) => {
                match Vec::<u8>::from_jsval(*cx, bytes.handle(), ConversionBehavior::EnforceRange) {
                    Ok(ConversionResult::Success(val)) => Ok(val),
                    Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.to_string())),
                    _ => Err(Error::Type("Unknown format for bytes read.".to_string())),
                }
            },
            Ok(false) => Err(Error::Type("Promise has no value property.".to_string())),
            Err(()) => Err(Error::JSFailed),
        }
    }
}
