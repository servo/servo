/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::{HandleValueArray, Heap, NewArrayObject, Value};
use js::jsval::ObjectValue;
use js::rust::HandleValue as SafeHandleValue;
use js::typedarray::ArrayBufferViewU8;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use super::byteteereadintorequest::ByteTeeReadIntoRequest;
use super::readablestream::ReaderType;
use super::readablestreambyobreader::ReadIntoRequest;
use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::stream::byteteereadrequest::ByteTeeReadRequest;
use crate::dom::stream::readablestreamdefaultreader::ReadRequest;
use crate::dom::types::ReadableStream;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ByteTeeCancelAlgorithm {
    Cancel1Algorithm,
    Cancel2Algorithm,
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum ByteTeePullAlgorithm {
    Pull1Algorithm,
    Pull2Algorithm,
}

#[dom_struct]
/// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
pub(crate) struct ByteTeeUnderlyingSource {
    reflector_: Reflector,
    #[conditional_malloc_size_of]
    reader: Rc<RefCell<ReaderType>>,
    stream: Dom<ReadableStream>,
    branch_1: MutNullableDom<ReadableStream>,
    branch_2: MutNullableDom<ReadableStream>,
    #[conditional_malloc_size_of]
    read_again_for_branch_1: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    read_again_for_branch_2: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    reading: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    canceled_1: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    canceled_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Mozjs"]
    reason_1: Rc<Heap<Value>>,
    #[ignore_malloc_size_of = "Mozjs"]
    reason_2: Rc<Heap<Value>>,
    #[conditional_malloc_size_of]
    cancel_promise: Rc<Promise>,
    #[conditional_malloc_size_of]
    reader_version: Rc<Cell<u64>>,
    tee_cancel_algorithm: ByteTeeCancelAlgorithm,
    byte_tee_pull_algorithm: ByteTeePullAlgorithm,
}

impl ByteTeeUnderlyingSource {
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(
        reader: Rc<RefCell<ReaderType>>,
        stream: &ReadableStream,
        reading: Rc<Cell<bool>>,
        read_again_for_branch_1: Rc<Cell<bool>>,
        read_again_for_branch_2: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        reason_1: Rc<Heap<Value>>,
        reason_2: Rc<Heap<Value>>,
        cancel_promise: Rc<Promise>,
        reader_version: Rc<Cell<u64>>,
        tee_cancel_algorithm: ByteTeeCancelAlgorithm,
        byte_tee_pull_algorithm: ByteTeePullAlgorithm,
        can_gc: CanGc,
    ) -> DomRoot<ByteTeeUnderlyingSource> {
        reflect_dom_object(
            Box::new(ByteTeeUnderlyingSource {
                reflector_: Reflector::new(),
                reader,
                stream: Dom::from_ref(stream),
                branch_1: MutNullableDom::new(None),
                branch_2: MutNullableDom::new(None),
                read_again_for_branch_1,
                read_again_for_branch_2,
                reading,
                canceled_1,
                canceled_2,
                reason_1,
                reason_2,
                cancel_promise,
                reader_version,
                tee_cancel_algorithm,
                byte_tee_pull_algorithm,
            }),
            &*stream.global(),
            can_gc,
        )
    }

    pub(crate) fn set_branch_1(&self, stream: &ReadableStream) {
        self.branch_1.set(Some(stream));
    }

    pub(crate) fn set_branch_2(&self, stream: &ReadableStream) {
        self.branch_2.set(Some(stream));
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn forward_reader_error(
        &self,
        cx: &mut JSContext,
        this_reader: Rc<RefCell<ReaderType>>,
    ) {
        let this_reader = this_reader.borrow_mut();
        match &*this_reader {
            ReaderType::Default(reader) => {
                let expected_version = self.reader_version.get();
                // Upon rejection of thisReader.[[closedPromise]] with reason r,
                reader
                    .get()
                    .expect("Reader should be set.")
                    .byte_tee_append_native_handler_to_closed_promise(
                        cx,
                        &self.branch_1.get().expect("Branch 1 should be set."),
                        &self.branch_2.get().expect("Branch 2 should be set."),
                        self.canceled_1.clone(),
                        self.canceled_2.clone(),
                        self.cancel_promise.clone(),
                        self.reader_version.clone(),
                        expected_version,
                    );
            },
            ReaderType::BYOB(reader) => {
                let expected_version = self.reader_version.get();
                // Upon rejection of thisReader.[[closedPromise]] with reason r,
                reader
                    .get()
                    .expect("Reader should be set.")
                    .byte_tee_append_native_handler_to_closed_promise(
                        cx,
                        &self.branch_1.get().expect("Branch 1 should be set."),
                        &self.branch_2.get().expect("Branch 2 should be set."),
                        self.canceled_1.clone(),
                        self.canceled_2.clone(),
                        self.cancel_promise.clone(),
                        self.reader_version.clone(),
                        expected_version,
                    );
            },
        }
    }

    fn pull_with_default_reader(&self, cx: &mut JSContext, global: &GlobalScope) -> Fallible<()> {
        let mut reader = self.reader.borrow_mut();
        match &*reader {
            ReaderType::BYOB(byte_reader) => {
                // Assert: readIntoRequests is empty.
                assert!(
                    byte_reader
                        .get()
                        .expect("Reader should be set.")
                        .get_num_read_into_requests() ==
                        0
                );

                // Release BYOB reader.
                byte_reader
                    .get()
                    .expect("Reader should be set.")
                    .release(CanGc::from_cx(cx))?;

                // Acquire default reader.
                let default_reader = self
                    .stream
                    .acquire_default_reader(CanGc::from_cx(cx))
                    .expect("AcquireReadableStreamDefaultReader should not fail");

                *reader = ReaderType::Default(MutNullableDom::new(Some(&default_reader)));
                self.reader_version
                    .set(self.reader_version.get().wrapping_add(1));
                drop(reader);

                // Attach error forwarding for the new reader.
                self.forward_reader_error(cx, self.reader.clone());

                // IMPORTANT: now actually perform the pull we were asked to do.
                return self.pull_with_default_reader(cx, global);
            },
            ReaderType::Default(reader) => {
                let byte_tee_read_request = ByteTeeReadRequest::new(
                    &self.branch_1.get().expect("Branch 1 should be set."),
                    &self.branch_2.get().expect("Branch 2 should be set."),
                    &self.stream,
                    self.read_again_for_branch_1.clone(),
                    self.read_again_for_branch_2.clone(),
                    self.reading.clone(),
                    self.canceled_1.clone(),
                    self.canceled_2.clone(),
                    self.cancel_promise.clone(),
                    self,
                    global,
                    CanGc::from_cx(cx),
                );

                let read_request = ReadRequest::ByteTee {
                    byte_tee_read_request: Dom::from_ref(&byte_tee_read_request),
                };

                reader
                    .get()
                    .expect("Reader should be set.")
                    .read(cx, &read_request);
            },
        }

        Ok(())
    }

    fn pull_with_byob_reader(
        &self,
        cx: &mut JSContext,
        view: &HeapBufferSource<ArrayBufferViewU8>,
        for_branch2: bool,
        global: &GlobalScope,
    ) {
        let mut reader = self.reader.borrow_mut();
        match &*reader {
            ReaderType::BYOB(reader) => {
                // Let byobBranch be branch2 if forBranch2 is true, and branch1 otherwise.
                let byob_branch = if for_branch2 {
                    self.branch_2.get().expect("Branch 2 should be set.")
                } else {
                    self.branch_1.get().expect("Branch 1 should be set.")
                };

                // let otherBranch be branch2 if forBranch2 is false, and branch1 otherwise.
                let other_branch = if for_branch2 {
                    self.branch_1.get().expect("Branch 1 should be set.")
                } else {
                    self.branch_2.get().expect("Branch 2 should be set.")
                };

                // Let readIntoRequest be a read-into request with the following items:
                let byte_tee_read_into_request = ByteTeeReadIntoRequest::new(
                    for_branch2,
                    &byob_branch,
                    &other_branch,
                    &self.stream,
                    self.read_again_for_branch_1.clone(),
                    self.read_again_for_branch_2.clone(),
                    self.reading.clone(),
                    self.canceled_1.clone(),
                    self.canceled_2.clone(),
                    self.cancel_promise.clone(),
                    self,
                    global,
                    CanGc::from_cx(cx),
                );

                let read_into_request = ReadIntoRequest::ByteTee {
                    byte_tee_read_into_request: Dom::from_ref(&byte_tee_read_into_request),
                };

                // Perform ! ReadableStreamBYOBReaderRead(reader, view, 1, readIntoRequest).
                reader
                    .get()
                    .expect("Reader should be set.")
                    .read(cx, view, 1, &read_into_request);
            },
            ReaderType::Default(default_reader) => {
                // If reader implements ReadableStreamDefaultReader,
                // Assert: reader.[[readRequests]] is empty.
                assert!(
                    default_reader
                        .get()
                        .expect("Reader should be set.")
                        .get_num_read_requests() ==
                        0
                );

                // Perform ! ReadableStreamDefaultReaderRelease(reader).
                default_reader
                    .get()
                    .expect("Reader should be set.")
                    .release(cx)
                    .expect("Release should be successful.");

                // Set reader to ! AcquireReadableStreamBYOBReader(stream).
                let byob_reader = self
                    .stream
                    .acquire_byob_reader(CanGc::from_cx(cx))
                    .expect("Reader should be set.");

                *reader = ReaderType::BYOB(MutNullableDom::new(Some(&byob_reader)));
                self.reader_version
                    .set(self.reader_version.get().wrapping_add(1));

                drop(reader);

                // Perform forwardReaderError, given reader.
                self.forward_reader_error(cx, self.reader.clone());

                // Retry the pull using the BYOB reader we just acquired.
                self.pull_with_byob_reader(cx, view, for_branch2, global);
            },
        }
    }

    /// Let pullAlgorithm be the following steps:
    pub(crate) fn pull_algorithm(
        &self,
        cx: &mut JSContext,
        byte_tee_pull_algorithm: Option<ByteTeePullAlgorithm>,
    ) -> Rc<Promise> {
        let pull_algorithm =
            byte_tee_pull_algorithm.unwrap_or(self.byte_tee_pull_algorithm.clone());

        match pull_algorithm {
            ByteTeePullAlgorithm::Pull1Algorithm => {
                // If reading is true,
                if self.reading.get() {
                    // Set readAgainForBranch1 to true.
                    self.read_again_for_branch_1.set(true);
                    // Return a promise resolved with undefined.
                    return Promise::new_resolved(
                        &self.stream.global(),
                        cx.into(),
                        (),
                        CanGc::from_cx(cx),
                    );
                }

                // Set reading to true.
                self.reading.set(true);

                // Let byobRequest be ! ReadableByteStreamControllerGetBYOBRequest(branch1.[[controller]]).
                let byob_branch_controller = self
                    .branch_1
                    .get()
                    .expect("Branch 1 should be set.")
                    .get_byte_controller();
                let byob_request = byob_branch_controller
                    .get_byob_request(cx)
                    .expect("Byob request should be set.");

                match byob_request {
                    // If byobRequest is null, perform pullWithDefaultReader.
                    None => {
                        self.pull_with_default_reader(cx, &self.stream.global())
                            .expect("Pull with default reader should be successful.");
                    },
                    Some(request) => {
                        // Otherwise, perform pullWithBYOBReader, given byobRequest.[[view]] and false.
                        let view = request.get_view();

                        self.pull_with_byob_reader(cx, &view, false, &self.stream.global());
                    },
                }

                // Return a promise resolved with undefined.
                Promise::new_resolved(&self.stream.global(), cx.into(), (), CanGc::from_cx(cx))
            },
            ByteTeePullAlgorithm::Pull2Algorithm => {
                // If reading is true,
                if self.reading.get() {
                    // Set readAgainForBranch2 to true.
                    self.read_again_for_branch_2.set(true);

                    // Return a promise resolved with undefined.
                    return Promise::new_resolved(
                        &self.stream.global(),
                        cx.into(),
                        (),
                        CanGc::from_cx(cx),
                    );
                }

                // Set reading to true.
                self.reading.set(true);

                // Let byobRequest be ! ReadableByteStreamControllerGetBYOBRequest(branch2.[[controller]]).
                let byob_branch_controller = self
                    .branch_2
                    .get()
                    .expect("Branch 2 should be set.")
                    .get_byte_controller();
                let byob_request = byob_branch_controller
                    .get_byob_request(cx)
                    .expect("Byob request should be set.");

                match byob_request {
                    None => {
                        self.pull_with_default_reader(cx, &self.stream.global())
                            .expect("Pull with default reader should be successful.");
                    },
                    Some(request) => {
                        // Otherwise, perform pullWithBYOBReader, given byobRequest.[[view]] and true.
                        let view = request.get_view();

                        self.pull_with_byob_reader(cx, &view, true, &self.stream.global());
                    },
                }

                // Return a promise resolved with undefined.
                Promise::new_resolved(&self.stream.global(), cx.into(), (), CanGc::from_cx(cx))
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let cancel1Algorithm be the following steps, taking a reason argument
    /// and
    /// Let cancel2Algorithm be the following steps, taking a reason argument
    pub(crate) fn cancel_algorithm(
        &self,
        cx: &mut JSContext,
        reason: SafeHandleValue,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match self.tee_cancel_algorithm {
            ByteTeeCancelAlgorithm::Cancel1Algorithm => {
                // Set canceled1 to true.
                self.canceled_1.set(true);

                // Set reason1 to reason.
                self.reason_1.set(reason.get());

                // If canceled2 is true,
                if self.canceled_2.get() {
                    self.resolve_cancel_promise(cx);
                }

                // Return cancelPromise.
                Some(Ok(self.cancel_promise.clone()))
            },
            ByteTeeCancelAlgorithm::Cancel2Algorithm => {
                // Set canceled_2 to true.
                self.canceled_2.set(true);

                // Set reason_2 to reason.
                self.reason_2.set(reason.get());

                // If canceled_1 is true,
                if self.canceled_1.get() {
                    self.resolve_cancel_promise(cx);
                }
                // Return cancelPromise.
                Some(Ok(self.cancel_promise.clone()))
            },
        }
    }

    #[expect(unsafe_code)]
    fn resolve_cancel_promise(&self, cx: &mut JSContext) {
        // Let compositeReason be ! CreateArrayFromList(« reason_1, reason_2 »).
        rooted_vec!(let mut reasons_values);
        reasons_values.push(self.reason_1.get());
        reasons_values.push(self.reason_2.get());

        let reasons_values_array = HandleValueArray::from(&reasons_values);
        rooted!(&in(cx) let reasons = unsafe { NewArrayObject(cx.raw_cx(), &reasons_values_array) });
        rooted!(&in(cx) let reasons_value = ObjectValue(reasons.get()));

        // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
        let cancel_result = self
            .stream
            .cancel(cx, &self.stream.global(), reasons_value.handle());

        // Resolve cancelPromise with cancelResult.
        self.cancel_promise
            .resolve_native(&cancel_result, CanGc::from_cx(cx));
    }
}
