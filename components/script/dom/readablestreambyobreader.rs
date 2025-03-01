/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![allow(dead_code)]

use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::{ArrayBufferView, ArrayBufferViewU8};

use super::bindings::buffer_source::{BufferSource, HeapBufferSource};
use super::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderReadOptions;
use super::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamReadResult;
use super::bindings::reflector::reflect_dom_object;
use super::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#read-into-request>
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    Read(#[ignore_malloc_size_of = "Rc is hard"] Rc<Promise>),
}

impl ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-chunk-steps>
    pub fn chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>, can_gc: CanGc) {
        // chunk steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → false ]».
        match self {
            ReadIntoRequest::Read(promise) => {
                promise.resolve_native(
                    &ReadableStreamReadResult {
                        done: Some(false),
                        value: chunk,
                    },
                    can_gc,
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-close-steps%E2%91%A0>
    pub fn close_steps(&self, chunk: Option<RootedTraceableBox<Heap<JSVal>>>, can_gc: CanGc) {
        // close steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → true ]».
        match self {
            ReadIntoRequest::Read(promise) => match chunk {
                Some(chunk) => promise.resolve_native(
                    &ReadableStreamReadResult {
                        done: Some(true),
                        value: chunk,
                    },
                    can_gc,
                ),
                None => promise.resolve_native(&(), can_gc),
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-error-steps>
    pub(crate) fn error_steps(&self, e: SafeHandleValue, can_gc: CanGc) {
        // error steps, given e
        // Reject promise with e.
        match self {
            ReadIntoRequest::Read(promise) => promise.reject_native(&e, can_gc),
        }
    }
}

/// <https://streams.spec.whatwg.org/#readablestreambyobreader>
#[dom_struct]
pub(crate) struct ReadableStreamBYOBReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    stream: MutNullableDom<ReadableStream>,

    read_into_requests: DomRefCell<VecDeque<ReadIntoRequest>>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: DomRefCell<Rc<Promise>>,
}

impl ReadableStreamBYOBReader {
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStreamBYOBReader> {
        reflect_dom_object_with_proto(
            Box::new(ReadableStreamBYOBReader::new_inherited(global, can_gc)),
            global,
            proto,
            can_gc,
        )
    }

    fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamBYOBReader {
        ReadableStreamBYOBReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            read_into_requests: DomRefCell::new(Default::default()),
            closed_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<ReadableStreamBYOBReader> {
        reflect_dom_object(
            Box::new(Self::new_inherited(global, can_gc)),
            global,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-byob-reader>
    pub(crate) fn set_up(
        &self,
        stream: &ReadableStream,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // If ! IsReadableStreamLocked(stream) is true, throw a TypeError exception.
        if stream.is_locked() {
            return Err(Error::Type("stream is locked".to_owned()));
        }

        // If stream.[[controller]] does not implement ReadableByteStreamController, throw a TypeError exception.
        if !stream.has_byte_controller() {
            return Err(Error::Type(
                "stream controller is not a byte stream controller".to_owned(),
            ));
        }

        // Perform ! ReadableStreamReaderGenericInitialize(reader, stream).
        self.generic_initialize(global, stream, can_gc);

        // Set reader.[[readIntoRequests]] to a new empty list.
        self.read_into_requests.borrow_mut().clear();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreambyobreaderrelease>
    pub(crate) fn release(&self, can_gc: CanGc) -> Fallible<()> {
        // Perform ! ReadableStreamReaderGenericRelease(reader).
        self.generic_release(can_gc)?;
        // Let e be a new TypeError exception.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        Error::Type("Reader is released".to_owned()).to_jsval(
            cx,
            &self.global(),
            error.handle_mut(),
        );

        // Perform ! ReadableStreamBYOBReaderErrorReadIntoRequests(reader, e).
        self.error_read_into_requests(error.handle(), can_gc);
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreambyobreadererrorreadintorequests>
    fn error_read_into_requests(&self, rval: SafeHandleValue, can_gc: CanGc) {
        // Let readRequests be reader.[[readRequests]].
        let mut read_into_requests = self.take_read_into_requests();

        // Set reader.[[readIntoRequests]] to a new empty list.
        for request in read_into_requests.drain(0..) {
            // Perform readIntoRequest’s error steps, given e.
            request.error_steps(rval, can_gc);
        }
    }

    fn take_read_into_requests(&self) -> VecDeque<ReadIntoRequest> {
        mem::take(&mut *self.read_into_requests.borrow_mut())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-into-request>
    pub(crate) fn add_read_into_request(&self, read_request: &ReadIntoRequest) {
        self.read_into_requests
            .borrow_mut()
            .push_back(read_request.clone());
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    pub(crate) fn close(&self, can_gc: CanGc) {
        // If reader is not undefined and reader implements ReadableStreamBYOBReader,
        // Let readIntoRequests be reader.[[readIntoRequests]].
        let mut read_into_requests = self.take_read_into_requests();
        // Set reader.[[readIntoRequests]] to an empty list.
        // Perform readIntoRequest’s close steps, given undefined.
        for request in read_into_requests.drain(0..) {
            // Perform readIntoRequest’s close steps, given undefined.
            request.close_steps(None, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-byob-reader-read>
    pub(crate) fn read(
        &self,
        view: HeapBufferSource<ArrayBufferViewU8>,
        options: &ReadableStreamBYOBReaderReadOptions,
        read_into_request: &ReadIntoRequest,
        can_gc: CanGc,
    ) {
        // Let stream be reader.[[stream]].

        // Assert: stream is not undefined.
        assert!(self.stream.get().is_some());

        let stream = self.stream.get().unwrap();

        // Set stream.[[disturbed]] to true.
        stream.set_is_disturbed(true);
        // If stream.[[state]] is "errored", perform readIntoRequest’s error steps given stream.[[storedError]].
        if stream.is_errored() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            read_into_request.error_steps(error.handle(), can_gc);
        } else {
            // Otherwise,
            // perform ! ReadableByteStreamControllerPullInto(stream.[[controller]], view, min, readIntoRequest).
            stream.perform_pull_into_steps(read_into_request, view, options, can_gc);
        }
    }
}

impl ReadableStreamBYOBReaderMethods<crate::DomTypeHolder> for ReadableStreamBYOBReader {
    /// <https://streams.spec.whatwg.org/#byob-reader-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &ReadableStream,
    ) -> Fallible<DomRoot<Self>> {
        let reader = Self::new_with_proto(global, proto, can_gc);

        // Perform ? SetUpReadableStreamBYOBReader(this, stream).
        Self::set_up(&reader, stream, global, can_gc)?;

        Ok(reader)
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    #[allow(unsafe_code)]
    fn Read(
        &self,
        view: CustomAutoRooterGuard<ArrayBufferView>,
        options: &ReadableStreamBYOBReaderReadOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let view = HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(
            Heap::boxed(unsafe { *view.underlying_object() }),
        ));

        // Let promise be a new promise.
        let promise = Promise::new(&self.global(), can_gc);

        let cx = GlobalScope::get_cx();
        // If view.[[ByteLength]] is 0, return a promise rejected with a TypeError exception.
        if view.byte_length() == 0 {
            promise.reject_error(Error::Type("view byte length is 0".to_owned()), can_gc);
            return promise;
        }
        // If view.[[ViewedArrayBuffer]].[[ArrayBufferByteLength]] is 0,
        // return a promise rejected with a TypeError exception.
        if view.viewed_buffer_array_byte_length(cx) == 0 {
            promise.reject_error(
                Error::Type("viewed buffer byte length is 0".to_owned()),
                can_gc,
            );
            return promise;
        }

        // If ! IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is true,
        // return a promise rejected with a TypeError exception.
        if view.is_detached_buffer(cx) {
            promise.reject_error(Error::Type("view is detached".to_owned()), can_gc);
            return promise;
        }

        // If options["min"] is 0, return a promise rejected with a TypeError exception.
        if options.min == 0 {
            promise.reject_error(Error::Type("min is 0".to_owned()), can_gc);
            return promise;
        }

        // If view has a [[TypedArrayName]] internal slot,
        if view.has_typed_array_name() {
            // If options["min"] > view.[[ArrayLength]], return a promise rejected with a RangeError exception.
            if options.min > (view.array_length() as u64) {
                promise.reject_error(
                    Error::Type("min is greater than array length".to_owned()),
                    can_gc,
                );
                return promise;
            }
        } else {
            // Otherwise (i.e., it is a DataView),
            // If options["min"] > view.[[ByteLength]], return a promise rejected with a RangeError exception.
            if options.min > (view.byte_length() as u64) {
                promise.reject_error(
                    Error::Type("min is greater than byte length".to_owned()),
                    can_gc,
                );
                return promise;
            }
        }

        // If this.[[stream]] is undefined, return a promise rejected with a TypeError exception.
        if self.stream.get().is_none() {
            promise.reject_error(
                Error::Type("min is greater than byte length".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Let readIntoRequest be a new read-into request with the following items:
        //
        // chunk steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → false ]».
        //
        // close steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → true ]».
        //
        // error steps, given e
        // Reject promise with e
        let read_into_request = ReadIntoRequest::Read(promise.clone());

        // Perform ! ReadableStreamBYOBReaderRead(this, view, options["min"], readIntoRequest).
        self.read(view, options, &read_into_request, can_gc);

        // Return promise.
        promise
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-release-lock>
    fn ReleaseLock(&self, can_gc: CanGc) -> Fallible<()> {
        if self.stream.get().is_none() {
            // If this.[[stream]] is undefined, return.
            return Ok(());
        }

        // Perform !ReadableStreamBYOBReaderRelease(this).
        self.release(can_gc)
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        self.cancel(&self.global(), reason, can_gc)
    }
}

impl ReadableStreamGenericReader for ReadableStreamBYOBReader {
    fn get_closed_promise(&self) -> Rc<Promise> {
        self.closed_promise.borrow().clone()
    }

    fn set_closed_promise(&self, promise: Rc<Promise>) {
        *self.closed_promise.borrow_mut() = promise;
    }

    fn set_stream(&self, stream: Option<&ReadableStream>) {
        self.stream.set(stream);
    }

    fn get_stream(&self) -> Option<DomRoot<ReadableStream>> {
        self.stream.get()
    }

    fn as_byob_reader(&self) -> Option<&ReadableStreamBYOBReader> {
        Some(self)
    }
}
