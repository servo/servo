/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self, NonNull};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::{ObjectValue, UndefinedValue, JSVal};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::{
    ReadableStreamGetReaderOptions, ReadableStreamMethods, ReadableStreamReaderMode,
};
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultController_Binding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultReaderOrReadableStreamBYOBReader as ReadableStreamReader;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::readablestreambyobreader::ReadableStreamBYOBReader;
use crate::dom::readablestreamdefaultcontroller::ReadableStreamDefaultController;
use crate::dom::readablestreamdefaultreader::{ReadRequest, ReadableStreamDefaultReader};
use crate::dom::underlyingsourcecontainer::UnderlyingSourceType;
use crate::js::conversions::FromJSValConvertible;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};

/// The fulfillment handler for the reacting to sourceCancelPromise part of
/// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct SourceCancelPromiseFulfillmentHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    result: Rc<Promise>,
}

impl Callback for SourceCancelPromiseFulfillmentHandler {
    /// The fulfillment handler for the reacting to sourceCancelPromise part of
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
        self.result.resolve_native(&());
    }
}

/// <https://streams.spec.whatwg.org/#readablestream-state>
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ReadableStreamState {
    #[default]
    Readable,
    Closed,
    Errored,
}

/// <https://streams.spec.whatwg.org/#readablestream-controller>
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub enum ControllerType {
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
    Byte(MutNullableDom<ReadableByteStreamController>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
    Default(MutNullableDom<ReadableStreamDefaultController>),
}

/// <https://streams.spec.whatwg.org/#readablestream-readerr>
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub enum ReaderType {
    /// <https://streams.spec.whatwg.org/#readablestreambyobreader>
    #[allow(clippy::upper_case_acronyms)]
    BYOB(MutNullableDom<ReadableStreamBYOBReader>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
    Default(MutNullableDom<ReadableStreamDefaultReader>),
}

/// <https://streams.spec.whatwg.org/#rs-class>
#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestream-controller>
    /// Note: the inner `MutNullableDom` should really be an `Option<Dom>`,
    /// because it is never unset once set.
    controller: ControllerType,

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    #[ignore_malloc_size_of = "mozjs"]
    stored_error: Heap<JSVal>,

    /// <https://streams.spec.whatwg.org/#readablestream-disturbed>
    disturbed: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestream-reader>
    reader: ReaderType,

    /// <https://streams.spec.whatwg.org/#readablestream-state>
    state: Cell<ReadableStreamState>,
}

impl ReadableStream {
    #[allow(crown::unrooted_must_root)]
    /// <https://streams.spec.whatwg.org/#initialize-readable-stream>
    fn new_inherited(controller: ControllerType) -> ReadableStream {
        let reader = match &controller {
            ControllerType::Default(_) => ReaderType::Default(MutNullableDom::new(None)),
            ControllerType::Byte(_) => ReaderType::BYOB(MutNullableDom::new(None)),
        };
        ReadableStream {
            reflector_: Reflector::new(),
            controller,
            stored_error: Heap::default(),
            disturbed: Default::default(),
            reader,
            state: Cell::new(Default::default()),
        }
    }

    #[allow(crown::unrooted_must_root)]
    fn new(global: &GlobalScope, controller: ControllerType) -> DomRoot<ReadableStream> {
        reflect_dom_object(Box::new(ReadableStream::new_inherited(controller)), global)
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub fn set_default_controller(&self, controller: &ReadableStreamDefaultController) {
        match self.controller {
            ControllerType::Default(ref ctrl) => ctrl.set(Some(controller)),
            ControllerType::Byte(_) => {
                unreachable!("set_default_controller called in setup of default controller.")
            },
        }
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub fn assert_no_controller(&self) {
        let has_no_controller = match self.controller {
            ControllerType::Default(ref ctrl) => ctrl.get().is_none(),
            ControllerType::Byte(ref ctrl) => ctrl.get().is_none(),
        };
        assert!(has_no_controller);
    }

    /// Used from RustCodegen.py
    /// TODO: remove here and its use in codegen.
    #[allow(unsafe_code)]
    pub unsafe fn from_js(
        _cx: SafeJSContext,
        _obj: *mut JSObject,
        _realm: InRealm,
    ) -> Result<DomRoot<ReadableStream>, ()> {
        Err(())
    }

    /// Build a stream backed by a Rust source that has already been read into memory.
    pub fn new_from_bytes(
        global: &GlobalScope,
        bytes: Vec<u8>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStream> {
        let stream = ReadableStream::new_with_external_underlying_source(
            global,
            UnderlyingSourceType::Memory(bytes.len()),
            can_gc,
        );
        stream.enqueue_native(bytes, can_gc);
        stream.close();
        stream
    }

    /// Build a stream backed by a Rust underlying source.
    /// Note: external sources are always paired with a default controller.
    #[allow(unsafe_code)]
    pub fn new_with_external_underlying_source(
        global: &GlobalScope,
        source: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStream> {
        assert!(source.is_native());
        let stream =
            ReadableStream::new(global, ControllerType::Default(MutNullableDom::new(None)));
        let _controller = ReadableStreamDefaultController::new(
            global,
            stream.clone(),
            source,
            1.0,
            extract_size_algorithm(&QueuingStrategy::empty()),
            can_gc,
        );
        stream
    }

    /// Call into the release steps of the controller,
    pub fn perform_release_steps(&self) {
        match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_release_steps(),
            ControllerType::Byte(_) => todo!(),
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_steps(read_request),
            ControllerType::Byte(_) => todo!(),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub fn add_read_request(&self, read_request: ReadRequest) {
        match self.reader {
            // Assert: stream.[[reader]] implements ReadableStreamDefaultReader.
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to add a read request without having first acquired a reader.");
                };

                // Assert: stream.[[state]] is "readable".
                assert!(self.is_readable());

                // Append readRequest to stream.[[reader]].[[readRequests]].
                reader.add_read_request(read_request);
            },
            ReaderType::BYOB(_) => {
                unreachable!("Adding a read request can only be done on a default reader.")
            },
        }
    }

    /// Get a pointer to the underlying JS object.
    /// TODO: remove,
    /// by using at call point the `ReadableStream` directly instead of a JSObject.
    pub fn get_js_stream(&self) -> NonNull<JSObject> {
        NonNull::new(*self.reflector().get_jsobject())
            .expect("Couldn't get a non-null pointer to JS stream object.")
    }
    /// Endpoint to enqueue chunks directly from Rust.
    /// Note: in other use cases this call happens via the controller.
    pub fn enqueue_native(&self, bytes: Vec<u8>, can_gc: CanGc) {
        match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .enqueue_native(bytes, can_gc),
            _ => unreachable!(
                "Enqueueing chunk to a stream from Rust on other than default controller"
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, e: SafeHandleValue) {
        // step 1
        assert!(self.is_readable());
        // step 2
        self.state.set(ReadableStreamState::Errored);
        // step 3
        self.stored_error.set(e.get());

        // step 4
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    // step 5
                    return;
                };

                // steps 6, 7, 8
                reader.error(e);
            },
            _ => todo!(),
        }
    }

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    #[allow(unsafe_code)]
    pub fn get_stored_error(&self, handle_mut: SafeMutableHandleValue) {
        unsafe {
            let cx = GlobalScope::get_cx();
            self.stored_error.to_jsval(*cx, handle_mut);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    /// Note: in other use cases this call happens via the controller.
    #[allow(unsafe_code)]
    pub fn error_native(&self, error: Error) {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        unsafe {
            error
                .clone()
                .to_jsval(*cx, &self.global(), rval.handle_mut())
        };
        self.error(rval.handle());
    }

    /// Call into the controller's `Close` method.
    pub fn close_native(&self) {
        match self.controller {
            ControllerType::Default(ref controller) => {
                let _ = controller
                    .get()
                    .expect("Stream should have controller.")
                    .Close();
            },
            ControllerType::Byte(_) => {
                unreachable!("Native closing is only done on default controllers.")
            },
        }
    }

    /// Returns a boolean reflecting whether the stream has all data in memory.
    /// Useful for native source integration only.
    pub fn in_memory(&self) -> bool {
        match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .in_memory(),
            ControllerType::Byte(_) => unreachable!(
                "Checking if source is in memory for a stream with a non-default controller"
            ),
        }
    }

    /// Return bytes for synchronous use, if the stream has all data in memory.
    /// Useful for native source integration only.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .get_in_memory_bytes(),
            ControllerType::Byte(_) => {
                unreachable!("Getting in-memory bytes for a stream with a non-default controller")
            },
        }
    }

    /// Acquires a reader and locks the stream,
    /// must be done before `read_a_chunk`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-reader>
    pub fn start_reading(&self) -> Result<(), ()> {
        // Let reader be a new ReadableStreamDefaultReader.
        // Perform ? SetUpReadableStreamDefaultReader(reader, stream).
        // Return reader.
        let reader = ReadableStreamDefaultReader::set_up(&self.global(), self, CanGc::note())
            .map_err(|_| ())?;

        self.set_reader(Some(&reader));

        Ok(())
    }

    /// Read a chunk from the stream,
    /// must be called after `start_reading`,
    /// and before `stop_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub fn read_a_chunk(&self, can_gc: CanGc) -> Rc<Promise> {
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to read stream chunk without having first acquired a reader.");
                };
                reader.Read(can_gc)
            },
            ReaderType::BYOB(_) => {
                unreachable!("Native reading of a chunk can only be done with a default reader.")
            },
        }
    }

    /// Releases the lock on the reader,
    /// must be done after `start_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    pub fn stop_reading(&self) {
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to stop reading without having first acquired a reader.");
                };
                reader.release();
            },
            ReaderType::BYOB(_) => {
                unreachable!("Native stop reading can only be done with a default reader.")
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#is-readable-stream-locked>
    pub fn is_locked(&self) -> bool {
        match self.reader {
            ReaderType::Default(ref reader) => reader.get().is_some(),
            ReaderType::BYOB(ref reader) => reader.get().is_some(),
        }
    }

    pub fn is_disturbed(&self) -> bool {
        self.disturbed.get()
    }

    pub fn set_is_disturbed(&self, disturbed: bool) {
        self.disturbed.set(disturbed);
    }

    pub fn is_closed(&self) -> bool {
        self.state.get() == ReadableStreamState::Closed
    }

    pub fn is_errored(&self) -> bool {
        self.state.get() == ReadableStreamState::Errored
    }

    pub fn is_readable(&self) -> bool {
        self.state.get() == ReadableStreamState::Readable
    }

    pub fn has_default_reader(&self) -> bool {
        match self.reader {
            ReaderType::Default(ref reader) => reader.get().is_some(),
            ReaderType::BYOB(_) => false,
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub fn get_num_read_requests(&self) -> usize {
        assert!(self.has_default_reader());
        match self.reader {
            ReaderType::Default(ref reader) => {
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when get num read requests is called into.");
                reader.get_num_read_requests()
            },
            ReaderType::BYOB(_) => unreachable!(
                "Stream must have a default reader when get num read requests is called into."
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn fulfill_read_request(&self, chunk: SafeHandleValue, done: bool) {
        // step 1 - Assert: ! ReadableStreamHasDefaultReader(stream) is true.
        assert!(self.has_default_reader());
        match self.reader {
            ReaderType::Default(ref reader) => {
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
                    request.close_steps();
                } else {
                    // step 7 - Otherwise, perform readRequest’s chunk steps, given chunk.
                    let result = RootedTraceableBox::new(Heap::default());
                    result.set(*chunk);
                    request.chunk_steps(result);
                }
            },
            ReaderType::BYOB(_) => unreachable!(
                "Stream must have a default reader when fulfill read requests is called into."
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close(&self) {
        // step 1
        assert!(self.is_readable());
        // step 2
        self.state.set(ReadableStreamState::Closed);
        // step 3
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    // step 4
                    return;
                };
                // step 5 & 6
                reader.close();
            },
            ReaderType::BYOB(ref _reader) => todo!(),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    #[allow(unsafe_code)]
    pub fn cancel(&self, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // step 1
        self.disturbed.set(true);

        // step 2
        if self.is_closed() {
            let promise = Promise::new(&self.reflector_.global(), can_gc);
            promise.resolve_native(&());
            return promise;
        }
        // step 3
        if self.is_errored() {
            let promise = Promise::new(&self.reflector_.global(), can_gc);
            unsafe {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                self.stored_error.to_jsval(*cx, rval.handle_mut());
                promise.reject_native(&rval.handle());
                return promise;
            }
        }
        // step 4
        self.close();
        // step 5, 6, 7, 8
        // TODO: run the bytes reader steps.

        // Let sourceCancelPromise be ! stream.[[controller]].[[CancelSteps]](reason).
        let source_cancel_promise = match self.controller {
            ControllerType::Default(ref controller) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_cancel_steps(reason, can_gc),
            ControllerType::Byte(_) => {
                todo!()
            },
        };

        // Create a new promise,
        // and setup a handler in order to react to the fulfillment of sourceCancelPromise.
        let global = self.reflector_.global();
        let result_promise = Promise::new(&global, can_gc);
        let fulfillment_handler = Box::new(SourceCancelPromiseFulfillmentHandler {
            result: result_promise.clone(),
        });
        let handler = PromiseNativeHandler::new(&global, Some(fulfillment_handler), None);
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        source_cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return the result of reacting to sourceCancelPromise
        // with a fulfillment step that returns undefined.
        result_promise
    }

    pub fn set_reader(&self, new_reader: Option<&ReadableStreamDefaultReader>) {
        match self.reader {
            ReaderType::Default(ref reader) => {
                reader.set(new_reader);
            },
            ReaderType::BYOB(_) => {
                unreachable!("Setting a reader can only be done on a default reader.")
            },
        }
    }
}

impl ReadableStreamMethods for ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
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
        let stream = if underlying_source_dict.type_.is_some() {
            ReadableStream::new(global, ControllerType::Byte(MutNullableDom::new(None)))
        } else {
            ReadableStream::new(global, ControllerType::Default(MutNullableDom::new(None)))
        };

        if underlying_source_dict.type_.is_some() {
            // TODO: If underlyingSourceDict["type"] is "bytes"
            todo!()
        } else {
            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
            let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

            // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
            let size_algorithm = extract_size_algorithm(strategy);

            // Perform ? SetUpReadableStreamDefaultControllerFromUnderlyingSource
            let _ = ReadableStreamDefaultController::new(
                global,
                stream.clone(),
                UnderlyingSourceType::Js(underlying_source_dict),
                high_water_mark,
                size_algorithm,
                can_gc,
            );
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
            let promise = Promise::new(&self.reflector_.global(), can_gc);
            promise.reject_error(Error::Type("stream is not locked".to_owned()));
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
            match self.reader {
                ReaderType::Default(ref reader) => {
                    reader.set(Some(&*ReadableStreamDefaultReader::set_up(
                        &self.global(),
                        self,
                        can_gc,
                    )?));
                    return Ok(ReadableStreamReader::ReadableStreamDefaultReader(
                        reader.get().unwrap(),
                    ));
                },
                _ => unreachable!("Getting BYOBReader can only be done when options.mode is set."),
            }
        }
        // 2. Assert: options["mode"] is "byob".
        assert!(options.mode.unwrap() == ReadableStreamReaderMode::Byob);

        // 3. Return ? AcquireReadableStreamBYOBReader(this).
        todo!();
    }
}

#[allow(unsafe_code)]
/// Get the `done` property of an object that a read promise resolved to.
pub fn get_read_promise_done(cx: SafeJSContext, v: &SafeHandleValue) -> Result<bool, Error> {
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
pub fn get_read_promise_bytes(cx: SafeJSContext, v: &SafeHandleValue) -> Result<Vec<u8>, Error> {
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
