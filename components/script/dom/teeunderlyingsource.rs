/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue as SafeHandleValue;

use super::bindings::root::MutNullableDom;
use super::types::{ReadableStream, ReadableStreamDefaultReader};
use crate::dom::bindings::import::module::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::dom::teereadrequest::TeeReadRequest;
use crate::script_runtime::CanGc;

pub enum TeeCancelAlgorithm {
    Cancel1Algorithm,
    Cancel2Algorithm,
}

#[dom_struct]
pub struct TeeUnderlyingSource {
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
    reason_1: Box<Heap<JSVal>>,
    #[ignore_malloc_size_of = "Rc"]
    reason_2: Box<Heap<JSVal>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
    #[ignore_malloc_size_of = "TeeCancelAlgorithm"]
    #[no_trace]
    tee_cancel_algorithm: TeeCancelAlgorithm,
}

impl TeeUnderlyingSource {
    #[allow(clippy::too_many_arguments)]
    #[allow(crown::unrooted_must_root)]
    pub fn new(
        reader: Dom<ReadableStreamDefaultReader>,
        stream: Dom<ReadableStream>,
        branch_1: MutNullableDom<ReadableStream>,
        branch_2: MutNullableDom<ReadableStream>,
        reading: Rc<Cell<bool>>,
        read_again: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        clone_for_branch_2: Rc<Cell<bool>>,
        reason_1: SafeHandleValue,
        reason_2: SafeHandleValue,
        cancel_promise: Rc<Promise>,
        tee_cancel_algorithm: TeeCancelAlgorithm,
    ) -> TeeUnderlyingSource {
        TeeUnderlyingSource {
            reflector_: Reflector::new(),
            reader,
            stream,
            branch_1,
            branch_2,
            reading,
            read_again,
            canceled_1,
            canceled_2,
            clone_for_branch_2,
            reason_1: Heap::boxed(reason_1.get()),
            reason_2: Heap::boxed(reason_2.get()),
            cancel_promise,
            tee_cancel_algorithm,
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let pullAlgorithm be the following steps:
    pub fn pull_algorithm(&self) -> Option<Result<Rc<Promise>, Error>> {
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
        let tee_read_request = reflect_dom_object(
            Box::new(TeeReadRequest::new(
                self.stream.clone(),
                self.branch_1.clone(),
                self.branch_2.clone(),
                self.reading.clone(),
                self.read_again.clone(),
                self.canceled_1.clone(),
                self.canceled_2.clone(),
                self.clone_for_branch_2.clone(),
                self.cancel_promise.clone(),
                Dom::from_ref(self),
            )),
            &*self.stream.global(),
        );

        let read_request = ReadRequest::Tee {
            tee_read_request: Dom::from_ref(&tee_read_request),
        };

        // Perform ! ReadableStreamDefaultReaderRead(reader, readRequest).
        self.reader.read(read_request);

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
    pub fn cancel_algorithm(
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
                    // Let compositeReason be ! CreateArrayFromList(« reason_1, reason_2 »).

                    // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
                    let _cancel_result = self.stream.cancel(reason, can_gc);
                    // Resolve cancelPromise with cancelResult.
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
                    // Let compositeReason be ! CreateArrayFromList(« reason_1, reason_2 »).

                    // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
                    let _cancel_result = self.stream.cancel(reason, can_gc);
                    // Resolve cancelPromise with cancelResult.
                }
                // Return cancelPromise.
                Some(Ok(self.cancel_promise.clone()))
            },
        }
    }
}
