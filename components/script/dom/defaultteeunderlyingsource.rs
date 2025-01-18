/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{HandleValueArray, Heap, NewArrayObject, Value};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::HandleValue as SafeHandleValue;

use super::bindings::root::{DomRoot, MutNullableDom};
use super::types::{ReadableStream, ReadableStreamDefaultReader};
use crate::dom::bindings::import::module::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::defaultteereadrequest::DefaultTeeReadRequest;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum TeeCancelAlgorithm {
    Cancel1Algorithm,
    Cancel2Algorithm,
}

#[dom_struct]
/// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
pub(crate) struct DefaultTeeUnderlyingSource {
    reflector_: Reflector,
    reader: Dom<ReadableStreamDefaultReader>,
    stream: Dom<ReadableStream>,
    branch_1: MutNullableDom<ReadableStream>,
    branch_2: MutNullableDom<ReadableStream>,
    #[ignore_malloc_size_of = "Rc"]
    reading: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    read_again: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    clone_for_branch_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    #[allow(clippy::redundant_allocation)]
    reason_1: Rc<Box<Heap<Value>>>,
    #[ignore_malloc_size_of = "Rc"]
    #[allow(clippy::redundant_allocation)]
    reason_2: Rc<Box<Heap<Value>>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
    tee_cancel_algorithm: TeeCancelAlgorithm,
}

impl DefaultTeeUnderlyingSource {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_allocation)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        reader: &ReadableStreamDefaultReader,
        stream: &ReadableStream,
        reading: Rc<Cell<bool>>,
        read_again: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        clone_for_branch_2: Rc<Cell<bool>>,
        reason_1: Rc<Box<Heap<Value>>>,
        reason_2: Rc<Box<Heap<Value>>>,
        cancel_promise: Rc<Promise>,
        tee_cancel_algorithm: TeeCancelAlgorithm,
        can_gc: CanGc,
    ) -> DomRoot<DefaultTeeUnderlyingSource> {
        reflect_dom_object(
            Box::new(DefaultTeeUnderlyingSource {
                reflector_: Reflector::new(),
                reader: Dom::from_ref(reader),
                stream: Dom::from_ref(stream),
                branch_1: MutNullableDom::new(None),
                branch_2: MutNullableDom::new(None),
                reading,
                read_again,
                canceled_1,
                canceled_2,
                clone_for_branch_2,
                reason_1,
                reason_2,
                cancel_promise,
                tee_cancel_algorithm,
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

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let pullAlgorithm be the following steps:
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn pull_algorithm(&self, can_gc: CanGc) -> Option<Result<Rc<Promise>, Error>> {
        // If reading is true,
        if self.reading.get() {
            // Set readAgain to true.
            self.read_again.set(true);
            // Return a promise resolved with undefined.
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            return Some(Promise::new_resolved(
                &self.stream.global(),
                cx,
                rval.handle(),
            ));
        }

        // Set reading to true.
        self.reading.set(true);

        // Let readRequest be a read request with the following items:
        let tee_read_request = DefaultTeeReadRequest::new(
            &self.stream,
            &self.branch_1.get().expect("Branch 1 should be set."),
            &self.branch_2.get().expect("Branch 2 should be set."),
            self.reading.clone(),
            self.read_again.clone(),
            self.canceled_1.clone(),
            self.canceled_2.clone(),
            self.clone_for_branch_2.clone(),
            self.cancel_promise.clone(),
            self,
            can_gc,
        );

        // Rooting: the tee read request is rooted above.
        let read_request = ReadRequest::DefaultTee {
            tee_read_request: Dom::from_ref(&tee_read_request),
        };

        // Perform ! ReadableStreamDefaultReaderRead(reader, readRequest).
        self.reader.read(&read_request, can_gc);

        // Return a promise resolved with undefined.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        Some(Promise::new_resolved(
            &self.stream.global(),
            GlobalScope::get_cx(),
            rval.handle(),
        ))
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let cancel1Algorithm be the following steps, taking a reason argument
    /// and
    /// Let cancel2Algorithm be the following steps, taking a reason argument
    #[allow(unsafe_code)]
    pub(crate) fn cancel_algorithm(
        &self,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match self.tee_cancel_algorithm {
            TeeCancelAlgorithm::Cancel1Algorithm => {
                // Set canceled_1 to true.
                self.canceled_1.set(true);

                // Set reason_1 to reason.
                self.reason_1.set(reason.get());

                // If canceled_2 is true,
                if self.canceled_2.get() {
                    self.resolve_cancel_promise(can_gc);
                }
                // Return cancelPromise.
                Some(Ok(self.cancel_promise.clone()))
            },
            TeeCancelAlgorithm::Cancel2Algorithm => {
                // Set canceled_2 to true.
                self.canceled_2.set(true);

                // Set reason_2 to reason.
                self.reason_2.set(reason.get());

                // If canceled_1 is true,
                if self.canceled_1.get() {
                    self.resolve_cancel_promise(can_gc);
                }
                // Return cancelPromise.
                Some(Ok(self.cancel_promise.clone()))
            },
        }
    }

    #[allow(unsafe_code)]
    fn resolve_cancel_promise(&self, can_gc: CanGc) {
        // Let compositeReason be ! CreateArrayFromList(« reason_1, reason_2 »).
        let cx = GlobalScope::get_cx();
        rooted_vec!(let mut reasons_values);
        reasons_values.push(self.reason_1.get());
        reasons_values.push(self.reason_2.get());

        let reasons_values_array = HandleValueArray::from(&reasons_values);
        rooted!(in(*cx) let reasons = unsafe { NewArrayObject(*cx, &reasons_values_array) });
        rooted!(in(*cx) let reasons_value = ObjectValue(reasons.get()));

        // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
        let cancel_result = self.stream.cancel(reasons_value.handle(), can_gc);

        // Resolve cancelPromise with cancelResult.
        self.cancel_promise.resolve_native(&cancel_result);
    }
}
