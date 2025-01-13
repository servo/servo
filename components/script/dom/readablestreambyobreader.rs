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
use js::typedarray::ArrayBufferView;

use super::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#read-into-request>
#[derive(JSTraceable)]
pub enum ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    Read(Rc<Promise>),
}

impl ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#read-into-request-chunk-steps>
    pub fn chunk_steps(&self, _chunk: RootedTraceableBox<Heap<JSVal>>) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#read-into-request-close-steps>
    pub fn close_steps(&self, _chunk: Option<RootedTraceableBox<Heap<JSVal>>>) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#read-into-request-error-steps>
    pub(crate) fn error_steps(&self, _e: SafeHandleValue) {
        todo!()
    }
}

/// <https://streams.spec.whatwg.org/#readablestreambyobreader>
#[dom_struct]
pub struct ReadableStreamBYOBReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    stream: MutNullableDom<ReadableStream>,

    #[ignore_malloc_size_of = "Rc is hard"]
    read_into_requests: DomRefCell<VecDeque<ReadIntoRequest>>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: DomRefCell<Rc<Promise>>,
}

impl ReadableStreamBYOBReader {
    /// <https://streams.spec.whatwg.org/#byob-reader-constructor>
    #[allow(non_snake_case)]
    pub(crate) fn Constructor(
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

    pub(crate) fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamBYOBReader {
        ReadableStreamBYOBReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            read_into_requests: DomRefCell::new(Default::default()),
            closed_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
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
        self.generic_initialize(global, stream, can_gc)?;

        // Set reader.[[readIntoRequests]] to a new empty list.
        self.read_into_requests.borrow_mut().clear();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreambyobreaderrelease>
    #[allow(unsafe_code)]
    pub(crate) fn release(&self) -> Fallible<()> {
        // Perform ! ReadableStreamReaderGenericRelease(reader).
        self.generic_release()?;
        // Let e be a new TypeError exception.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        unsafe {
            Error::Type("Reader is released".to_owned()).to_jsval(
                *cx,
                &self.global(),
                error.handle_mut(),
            )
        };

        // Perform ! ReadableStreamBYOBReaderErrorReadIntoRequests(reader, e).
        self.error_read_into_requests(error.handle());
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreambyobreadererrorreadintorequests>
    #[allow(crown::unrooted_must_root)]
    fn error_read_into_requests(&self, rval: SafeHandleValue) {
        // Let readRequests be reader.[[readRequests]].
        let mut read_into_requests = self.take_read_into_requests();

        // Set reader.[[readIntoRequests]] to a new empty list.
        for request in read_into_requests.drain(0..) {
            // Perform readIntoRequest’s error steps, given e.
            request.error_steps(rval);
        }
    }

    #[allow(crown::unrooted_must_root)]
    fn take_read_into_requests(&self) -> VecDeque<ReadIntoRequest> {
        mem::take(&mut *self.read_into_requests.borrow_mut())
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
        ReadableStreamBYOBReader::Constructor(global, proto, can_gc, stream)
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    fn Read(&self, _view: CustomAutoRooterGuard<ArrayBufferView>, can_gc: CanGc) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global(), can_gc)
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-release-lock>
    fn ReleaseLock(&self) -> Fallible<()> {
        if self.stream.get().is_none() {
            // If this.[[stream]] is undefined, return.
            return Ok(());
        }

        // Perform !ReadableStreamBYOBReaderRelease(this).
        self.release()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        self.cancel(&self.reflector_.global(), reason, can_gc)
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
