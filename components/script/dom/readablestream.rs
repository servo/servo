/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self, NonNull};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::{
    ReadableStreamGetReaderOptions, ReadableStreamMethods,
};
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::import::module::UnionTypes::{
    ReadableStreamDefaultControllerOrReadableByteStreamController as Controller,
    ReadableStreamDefaultReaderOrReadableStreamBYOBReader as ReadableStreamReader,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
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
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readablestream-state>
#[derive(Clone, Copy, Default, JSTraceable, MallocSizeOf, PartialEq)]
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
    Byte(Dom<ReadableByteStreamController>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
    Default(Dom<ReadableStreamDefaultController>),
}

/// <https://streams.spec.whatwg.org/#readablestream-readerr>
#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub enum ReaderType {
    /// <https://streams.spec.whatwg.org/#readablestreambyobreader>
    BYOB(MutNullableDom<ReadableStreamBYOBReader>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
    Default(MutNullableDom<ReadableStreamDefaultReader>),
}

/// <https://streams.spec.whatwg.org/#rs-class>
#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestream-controller>
    controller: ControllerType,

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    /// TODO: check correctness of this.
    #[ignore_malloc_size_of = "mozjs"]
    stored_error: Heap<*mut JSObject>,

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
    fn new_inherited(controller: Controller) -> ReadableStream {
        let reader = match &controller {
            Controller::ReadableStreamDefaultController(_) => {
                ReaderType::Default(MutNullableDom::new(None))
            },
            Controller::ReadableByteStreamController(_) => {
                ReaderType::BYOB(MutNullableDom::new(None))
            },
        };
        ReadableStream {
            reflector_: Reflector::new(),
            controller: match controller {
                Controller::ReadableStreamDefaultController(root) => {
                    ControllerType::Default(Dom::from_ref(&*root))
                },
                Controller::ReadableByteStreamController(root) => {
                    ControllerType::Byte(Dom::from_ref(&*root))
                },
            },
            stored_error: Heap::default(),
            disturbed: Default::default(),
            reader: reader,
            state: Cell::new(ReadableStreamState::Readable),
        }
    }

    fn new(global: &GlobalScope, controller: Controller) -> DomRoot<ReadableStream> {
        reflect_dom_object(Box::new(ReadableStream::new_inherited(controller)), global)
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
        );
        stream.enqueue_native(bytes);
        stream.close();
        stream
    }

    /// Build a stream backed by a Rust underlying source.
    /// Note: external sources are always paired with a default controller.
    #[allow(unsafe_code)]
    pub fn new_with_external_underlying_source(
        global: &GlobalScope,
        source: UnderlyingSourceType,
    ) -> DomRoot<ReadableStream> {
        assert!(source.is_native());
        let controller = ReadableStreamDefaultController::new(
            global,
            source,
            1.0,
            extract_size_algorithm(&QueuingStrategy::empty()),
        );
        let stream = ReadableStream::new(
            global,
            Controller::ReadableStreamDefaultController(controller.clone()),
        );
        controller.set_stream(&stream);

        stream
    }

    /// Call into the release steps of the controller,
    pub fn perform_release_steps(&self) {
        match self.controller {
            ControllerType::Default(ref controller) => controller.perform_release_steps(),
            _ => todo!(),
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        match self.controller {
            ControllerType::Default(ref controller) => controller.perform_pull_steps(read_request),
            _ => todo!(),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub fn add_read_request(&self, read_request: ReadRequest) {
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to add a read request without having first acquired a reader.");
                };
                reader.add_read_request(read_request);
            },
            _ => unreachable!("Adding a read request can only be done on a default reader."),
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
    pub fn enqueue_native(&self, bytes: Vec<u8>) {
        match self.controller {
            ControllerType::Default(ref controller) => controller.enqueue_native(bytes),
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
        {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let object = e.to_object());
            self.stored_error.set(*object);
        }

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
    pub fn error_native(&self, _error: Error) {
        match self.controller {
            ControllerType::Default(ref controller) => {
                // TODO: use Error.
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                controller.error(rval.handle());
            },
            _ => unreachable!("Native closing a stream with a non-default controller"),
        }
    }

    /// Returns a boolean reflecting whether the stream has all data in memory.
    /// Useful for native source integration only.
    pub fn in_memory(&self) -> bool {
        match self.controller {
            ControllerType::Default(ref controller) => controller.in_memory(),
            _ => unreachable!(
                "Checking if source is in memory for a stream with a non-default controller"
            ),
        }
    }

    /// Return bytes for synchronous use, if the stream has all data in memory.
    /// Useful for native source integration only.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        match self.controller {
            ControllerType::Default(ref controller) => controller.get_in_memory_bytes(),
            _ => unreachable!("Getting in-memory bytes for a stream with a non-default controller"),
        }
    }

    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-reader>
    pub fn start_reading(&self) -> Result<DomRoot<ReadableStreamDefaultReader>, ()> {
        // step 1 & 2 & 3
        ReadableStreamDefaultReader::set_up(&*self.global(), self).map_err(|_| ())
    }

    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    /// TODO: restructure this on related methods so the caller reads from a reader?
    pub fn read_a_chunk(&self) -> Rc<Promise> {
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to read stream chunk without having first acquired a reader.");
                };
                reader.Read()
            },
            _ => unreachable!("Native reading of a chunk can only be done with a default reader."),
        }
    }

    /// Native call to
    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    /// TODO: restructure this on related methods so the caller releases a reader?
    pub fn stop_reading(&self) {
        match self.reader {
            ReaderType::Default(ref reader) => {
                let Some(rooted_reader) = reader.get() else {
                    panic!("Attempt to stop reading without having first acquired a reader.");
                };
                rooted_reader.ReleaseLock();
                reader.set(None);
            },
            _ => unreachable!("Native stop reading can only be done with a default reader."),
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
            _ => unreachable!(
                "Stream must have a default reader when get num read requests is called into."
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    #[allow(unsafe_code)]
    pub fn fulfill_read_request(&self, chunk: SafeHandleValue, done: bool) {
        assert!(self.has_default_reader());
        match self.reader {
            ReaderType::Default(ref reader) => {
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when a read request is fulfilled.");
                let request = reader.remove_read_request();
                if !done {
                    let cx = GlobalScope::get_cx();
                    rooted!(in(*cx) let mut rval = UndefinedValue());
                    let result = RootedTraceableBox::new(Heap::default());
                    result.set(*chunk);
                    request.chunk_steps(result);
                } else {
                    request.close_steps();
                }
            },
            _ => unreachable!(
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
    pub fn cancel(&self, promise: &Promise, _reason: SafeHandleValue) {
        // step 1
        self.disturbed.set(true);

        // step 2
        if self.is_closed() {
            return promise.resolve_native(&());
        }
        // step 3
        if self.is_errored() {
            unsafe {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                self.stored_error.to_jsval(*cx, rval.handle_mut());
                return promise.reject_native(&rval.handle());
            }
        }
        // step 4
        self.close();
        // step 5, 6, 7, 8
        // TODO: run the bytes reader steps.

        // TODO: react to sourceCancelPromise.
    }

    pub fn set_reader(&self, new_reader: Option<&ReadableStreamDefaultReader>) {
        match self.reader {
            ReaderType::Default(ref reader) => {
                reader.set(new_reader);
            },
            _ => unreachable!("Setting a reader can only be done on a default reader."),
        }
    }
}

impl ReadableStreamMethods for ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
        _can_gc: CanGc,
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
                    return Err(Error::Type(
                        "Unknown format for underlying source.".to_string(),
                    ))
                },
            }
        } else {
            JsUnderlyingSource::empty()
        };

        let controller = if underlying_source_dict.type_.is_some() {
            // TODO: If underlyingSourceDict["type"] is "bytes"
            todo!()
        } else {
            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
            let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

            // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
            let size_algorithm = extract_size_algorithm(strategy);

            // Perform ? SetUpReadableStreamDefaultControllerFromUnderlyingSource
            ReadableStreamDefaultController::new(
                global,
                UnderlyingSourceType::Js(underlying_source_dict),
                high_water_mark,
                size_algorithm,
            )
        };

        // Perform ! InitializeReadableStream(this).
        // Note: in the spec this step is done before
        // SetUpReadableStreamDefaultControllerFromUnderlyingSource
        Ok(ReadableStream::new(
            global,
            Controller::ReadableStreamDefaultController(controller),
        ))
    }

    /// <https://streams.spec.whatwg.org/#rs-locked>
    fn Locked(&self) -> bool {
        self.is_locked()
    }

    /// <https://streams.spec.whatwg.org/#rs-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue) -> Rc<Promise> {
        let promise = Promise::new(&self.reflector_.global(), CanGc::note());
        if !self.is_locked() {
            // TODO: reject with a TypeError exception.
            promise.reject_native(&());
        } else {
            self.cancel(&promise, reason);
        }
        promise
    }

    /// <https://streams.spec.whatwg.org/#rs-get-reader>
    fn GetReader(
        &self,
        _options: &ReadableStreamGetReaderOptions,
    ) -> Fallible<ReadableStreamReader> {
        if self.is_locked() {
            return Err(Error::Type("Stream is locked".to_string()));
        }
        match self.reader {
            ReaderType::Default(ref reader) => {
                reader.set(Some(&*ReadableStreamDefaultReader::set_up(
                    &*self.global(),
                    self,
                )?));
                return Ok(ReadableStreamReader::ReadableStreamDefaultReader(
                    reader.get().unwrap(),
                ));
            },
            _ => todo!(),
        }
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
