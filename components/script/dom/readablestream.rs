/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self, NonNull};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::cell::DomRefCell;
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
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::readablestreamdefaultcontroller::{
    ReadableStreamDefaultController, UnderlyingSource,
};
use crate::dom::readablestreamdefaultreader::{ReadRequest, ReadableStreamDefaultReader};
use crate::js::conversions::FromJSValConvertible;
use crate::realms::InRealm;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablestream-state>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub enum ReadableStreamState {
    #[default]
    Readable,
    Closed,
    Errored,
}

#[derive(JSTraceable, MallocSizeOf)]
/// <https://streams.spec.whatwg.org/#readablestream-controller>
#[crown::unrooted_must_root_lint::must_root]
pub enum ControllerType {
    Byte(Dom<ReadableByteStreamController>),
    Default(Dom<ReadableStreamDefaultController>),
}

#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestream-controller>
    controller: ControllerType,

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    stored_error: DomRefCell<Option<Error>>,

    /// <https://streams.spec.whatwg.org/#readablestream-disturbed>
    disturbed: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestream-reader>
    /// TODO: ReadableStreamBYOBReader
    reader: MutNullableDom<ReadableStreamDefaultReader>,

    /// <https://streams.spec.whatwg.org/#readablestream-state>
    state: DomRefCell<ReadableStreamState>,
}

impl ReadableStream {
    #[allow(non_snake_case)]
    /// <https://streams.spec.whatwg.org/#rs-constructor>
    pub fn Constructor(
        cx: SafeJSContext,
        _global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
        underlying_source: Option<*mut JSObject>,
        _strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<Self>> {
        // Step 1
        rooted!(in(*cx) let underlying_source_obj = underlying_source.unwrap_or(ptr::null_mut()));
        // Step 2
        let _underlying_source_dict = if !underlying_source_obj.is_null() {
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
        // TODO
        Err(Error::NotFound)
    }

    fn new_inherited(controller: Controller) -> ReadableStream {
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
            stored_error: DomRefCell::new(None),
            disturbed: Default::default(),
            reader: MutNullableDom::new(None),
            state: DomRefCell::new(ReadableStreamState::Readable),
        }
    }

    fn new(global: &GlobalScope, controller: Controller) -> DomRoot<ReadableStream> {
        reflect_dom_object(Box::new(ReadableStream::new_inherited(controller)), global)
    }

    /// Used from RustCodegen.py
    /// TODO: remove here and from codegen, to be replaced by Constructor.
    #[allow(unsafe_code)]
    pub unsafe fn from_js(
        _cx: SafeJSContext,
        _obj: *mut JSObject,
        _realm: InRealm,
    ) -> Result<DomRoot<ReadableStream>, ()> {
        Err(())
    }

    /// Build a stream backed by a Rust source that has already been read into memory.
    pub fn new_from_bytes(global: &GlobalScope, bytes: Vec<u8>) -> DomRoot<ReadableStream> {
        let stream = ReadableStream::new_with_external_underlying_source(
            global,
            UnderlyingSource::Memory(bytes.len()),
        );
        stream.enqueue_native(bytes);
        stream.close_native();
        stream
    }

    /// Build a stream backed by a Rust underlying source.
    /// Note: external sources are always paired with a default controller for now.
    #[allow(unsafe_code)]
    pub fn new_with_external_underlying_source(
        global: &GlobalScope,
        source: UnderlyingSource,
    ) -> DomRoot<ReadableStream> {
        assert!(source.is_native());
        let controller = ReadableStreamDefaultController::new(global, Rc::new(source));
        let stream = ReadableStream::new(
            global,
            Controller::ReadableStreamDefaultController(controller.clone()),
        );
        controller.set_stream(&stream);

        stream
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
        self.reader.get().unwrap().add_read_request(read_request);
    }

    /// Get a pointer to the underlying JS object.
    pub fn get_js_stream(&self) -> NonNull<JSObject> {
        NonNull::new(*self.reflector().get_jsobject())
            .expect("Couldn't get a non-null pointer to JS stream object.")
    }

    pub fn enqueue_native(&self, bytes: Vec<u8>) {
        match self.controller {
            ControllerType::Default(ref controller) => controller.enqueue_chunk(bytes),
            _ => unreachable!(
                "Enqueueing chunk to a stream from Rust on other than default controller"
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error_native(&self, _error: Error) {
        *self.state.borrow_mut() = ReadableStreamState::Errored;
        match self.controller {
            ControllerType::Default(ref controller) => controller.error(),
            _ => unreachable!("Native closing a stream with a non-default controller"),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close_native(&self) {
        match self.controller {
            ControllerType::Default(ref controller) => controller.close(),
            _ => unreachable!("Native closing a stream with a non-default controller"),
        }
    }

    /// Does the stream have all data in memory?
    pub fn in_memory(&self) -> bool {
        match self.controller {
            ControllerType::Default(ref controller) => controller.in_memory(),
            _ => unreachable!(
                "Checking if source is in memory for a stream with a non-default controller"
            ),
        }
    }

    /// Return bytes for synchronous use, if the stream has all data in memory.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        match self.controller {
            ControllerType::Default(ref controller) => controller.get_in_memory_bytes(),
            _ => unreachable!("Getting in-memory bytes for a stream with a non-default controller"),
        }
    }

    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-reader>
    pub fn acquire_reader(&self) -> Result<(), ()> {
        if self.is_locked() {
            return Err(());
        }
        let global = self.global();
        self.reader
            .set(Some(&*ReadableStreamDefaultReader::new(&*global, self)));
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub fn read_a_chunk(&self) -> Rc<Promise> {
        let Some(reader) = self.reader.get() else {
            panic!("Attempt to read stream chunk without having acquired a reader.");
        };

        reader.Read()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-release>
    pub fn stop_reading(&self) {
        let Some(reader) = self.reader.get() else {
            panic!("Attempt to read stream chunk without having acquired a reader.");
        };

        reader.ReleaseLock();

        self.reader.set(None);
    }

    pub fn is_locked(&self) -> bool {
        self.reader.get().is_some()
    }

    pub fn is_disturbed(&self) -> bool {
        self.disturbed.get()
    }

    pub fn is_closed(&self) -> bool {
        matches!(*self.state.borrow(), ReadableStreamState::Closed)
    }

    pub fn is_errored(&self) -> bool {
        matches!(*self.state.borrow(), ReadableStreamState::Errored)
    }

    pub fn is_readable(&self) -> bool {
        matches!(*self.state.borrow(), ReadableStreamState::Readable)
    }

    pub fn has_default_reader(&self) -> bool {
        if let Some(_reader) = self.reader.get() {
            false
        } else {
            // TODO: check type of reader.
            true
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub fn get_num_read_requests(&self) -> usize {
        assert!(self.has_default_reader());
        let reader = self
            .reader
            .get()
            .expect("Stream must have a reader when get num read requests is called into.");
        reader.get_num_read_requests()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn fulfill_read_request(&self, chunk: Vec<u8>, done: bool) {
        assert!(self.has_default_reader());
        let reader = self
            .reader
            .get()
            .expect("Stream must have a reader when a read request is fulfilled.");
        let request = reader.remove_read_request();

        if done {
            request.chunk_steps(chunk);
        }
        // TODO: else, close steps.
    }
}

impl ReadableStreamMethods for ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-locked>
    fn Locked(&self) -> bool {
        // TODO
        false
    }

    /// <https://streams.spec.whatwg.org/#rs-cancel>
    fn Cancel(&self, _cx: SafeJSContext, _reason: SafeHandleValue) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }

    /// <https://streams.spec.whatwg.org/#rs-get-reader>
    fn GetReader(
        &self,
        _options: &ReadableStreamGetReaderOptions,
    ) -> Fallible<ReadableStreamReader> {
        // TODO
        Err(Error::NotFound)
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
