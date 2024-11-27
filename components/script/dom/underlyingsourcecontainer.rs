/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{Handle as SafeHandle, HandleObject, HandleValue as SafeHandleValue, IntoHandle};

use super::types::{ReadableStream, ReadableStreamDefaultReader};
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::import::module::Error;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestreamdefaultreader::{ReadRequest, TeeReadRequest};
use crate::script_runtime::CanGc;

/// <https://streams.spec.whatwg.org/#underlying-source-api>
/// The `Js` variant corresponds to
/// the JavaScript object representing the underlying source.
/// The other variants are native sources in Rust.
#[derive(JSTraceable)]
#[crown::unrooted_must_root_lint::must_root]
pub enum UnderlyingSourceType {
    /// Facilitate partial integration with sources
    /// that are currently read into memory.
    Memory(usize),
    /// A blob as underlying source, with a known total size.
    Blob(usize),
    /// A fetch response as underlying source.
    FetchResponse,
    /// A struct representing a JS object as underlying source,
    /// and the actual JS object for use as `thisArg` in callbacks.
    Js(JsUnderlyingSource, Heap<*mut JSObject>),
    /// Tee
    Tee {
        tee_underlyin_source: TeeUnderlyingSource,
    },
}

impl UnderlyingSourceType {
    /// Is the source backed by a Rust native source?
    pub fn is_native(&self) -> bool {
        matches!(
            self,
            UnderlyingSourceType::Memory(_)
                | UnderlyingSourceType::Blob(_)
                | UnderlyingSourceType::FetchResponse
        )
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        matches!(self, UnderlyingSourceType::Memory(_))
    }
}

pub enum TeeCancelAlgorithm {
    Cancel1Algorithm,
    Cancel2Algorithm,
}

#[dom_struct]
pub struct TeeUnderlyingSource {
    reader: DomRoot<ReadableStreamDefaultReader>,
    stream: Dom<ReadableStream>,
    branch1: Option<DomRoot<ReadableStream>>,
    branch2: Option<DomRoot<ReadableStream>>,
    #[ignore_malloc_size_of = "Rc"]
    reading: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    read_again: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    clone_for_branch2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    reason1: Rc<Heap<*mut JSObject>>,
    #[ignore_malloc_size_of = "Rc"]
    reason2: Rc<Heap<*mut JSObject>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
    #[ignore_malloc_size_of = "TeeCancelAlgorithm"]
    #[no_trace]
    tee_cancel_algorithm: TeeCancelAlgorithm,
}

impl TeeUnderlyingSource {
    pub fn new(
        reader: DomRoot<ReadableStreamDefaultReader>,
        stream: Dom<ReadableStream>,
        branch1: Option<DomRoot<ReadableStream>>,
        branch2: Option<DomRoot<ReadableStream>>,
        reading: Rc<Cell<bool>>,
        read_again: Rc<Cell<bool>>,
        canceled1: Rc<Cell<bool>>,
        canceled2: Rc<Cell<bool>>,
        clone_for_branch2: Rc<Cell<bool>>,
        reason1: Rc<Heap<*mut JSObject>>,
        reason2: Rc<Heap<*mut JSObject>>,
        cancel_promise: Rc<Promise>,
        tee_cancel_algorithm: TeeCancelAlgorithm,
    ) -> TeeUnderlyingSource {
        TeeUnderlyingSource {
            reader,
            stream,
            branch1,
            branch2,
            reading,
            read_again,
            canceled1,
            canceled2,
            clone_for_branch2,
            reason1,
            reason2,
            cancel_promise,
            tee_cancel_algorithm,
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let pullAlgorithm be the following steps:
    pub fn pull_algorithm(&self) -> Option<Rc<Promise>> {
        // If reading is true,
        if self.reading.get() {
            // Set readAgain to true.
            self.read_again.set(true);
            // Return a promise resolved with undefined.
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            return Some(Promise::new_resolved(&self.stream.global(), cx, rval.handle()).unwrap());
        }

        // Set reading to true.
        self.reading.set(true);

        // Let readRequest be a read request with the following items:
        let read_request = ReadRequest::Tee {
            tee_read_request: TeeReadRequest::new(
                self.stream.clone(),
                self.branch1.clone(),
                self.branch2.clone(),
                self.reading.clone(),
                self.read_again.clone(),
                self.canceled1.clone(),
                self.canceled2.clone(),
                self.clone_for_branch2.clone(),
                self.cancel_promise.clone(),
                Dom::from_ref(self),
            ),
        };

        self.reader.append_native_handler_to_closed_promise(
            self.branch1.clone(),
            self.branch2.clone(),
            self.canceled1.clone(),
            self.canceled2.clone(),
            self.cancel_promise.clone(),
        );

        // Perform ! ReadableStreamDefaultReaderRead(reader, readRequest).
        self.reader.read(read_request);

        // Return a promise resolved with undefined.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        match Promise::new_resolved(&self.stream.global(), GlobalScope::get_cx(), rval.handle()) {
            Ok(promise) => Some(promise),
            Err(_) => None,
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Let cancel1Algorithm be the following steps, taking a reason argument
    /// and
    /// Let cancel2Algorithm be the following steps, taking a reason argument
    fn cancel_algorithm(&self, reason: SafeHandleValue) -> Option<Rc<Promise>> {
        match self.tee_cancel_algorithm {
            TeeCancelAlgorithm::Cancel1Algorithm => {
                // Set canceled1 to true.
                self.canceled1.set(true);
                // Set reason1 to reason.
                self.reason1.set(reason.to_object());
                // If canceled2 is true,
                if self.canceled2.get() {
                    // Let compositeReason be ! CreateArrayFromList(« reason1, reason2 »).

                    // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
                    let _cancel_result = self.stream.cancel(reason, CanGc::note());
                    // Resolve cancelPromise with cancelResult.
                }
                // Return cancelPromise.
                return Some(self.cancel_promise.clone());
            },
            TeeCancelAlgorithm::Cancel2Algorithm => {
                // Set canceled2 to true.
                self.canceled2.set(true);
                // Set reason2 to reason.
                self.reason2.set(reason.to_object());
                // If canceled1 is true,
                if self.canceled1.get() {
                    // Let compositeReason be ! CreateArrayFromList(« reason1, reason2 »).

                    // Let cancelResult be ! ReadableStreamCancel(stream, compositeReason).
                    let _cancel_result = self.stream.cancel(reason, CanGc::note());
                    // Resolve cancelPromise with cancelResult.
                }
                // Return cancelPromise.
                return Some(self.cancel_promise.clone());
            },
        };
    }
}

/// Wrapper around the underlying source.
/// Useful because `Call_` requires the "this object" to impl DomObject.
#[dom_struct]
pub struct UnderlyingSourceContainer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "JsUnderlyingSource implemented in SM."]
    underlying_source_type: UnderlyingSourceType,
}

impl UnderlyingSourceContainer {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(underlying_source_type: UnderlyingSourceType) -> UnderlyingSourceContainer {
        UnderlyingSourceContainer {
            reflector_: Reflector::new(),
            underlying_source_type,
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> DomRoot<UnderlyingSourceContainer> {
        // TODO: setting the underlying source dict as the prototype of the
        // `UnderlyingSourceContainer`, as it is later used as the "this" in Call_.
        // Is this a good idea?
        reflect_dom_object_with_proto(
            Box::new(UnderlyingSourceContainer::new_inherited(
                underlying_source_type,
            )),
            global,
            None,
            can_gc,
        )
    }

    /// Setting the JS object after the heap has settled down.
    pub fn set_underlying_source_this_object(&self, object: HandleObject) {
        if let UnderlyingSourceType::Js(_source, this_obj) = &self.underlying_source_type {
            this_obj.set(*object);
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-cancel>
    #[allow(unsafe_code)]
    pub fn call_cancel_algorithm(
        &self,
        reason: SafeHandleValue,
    ) -> Option<Result<Rc<Promise>, Error>> {
        if let UnderlyingSourceType::Js(source, this_obj) = &self.underlying_source_type {
            if let Some(algo) = &source.cancel {
                let result = unsafe {
                    algo.Call_(
                        &SafeHandle::from_raw(this_obj.handle()),
                        Some(reason),
                        ExceptionHandling::Rethrow,
                    )
                };
                return Some(result);
            }
        }
        None
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-pull>
    #[allow(unsafe_code)]
    pub fn call_pull_algorithm(
        &self,
        controller: Controller,
    ) -> Option<Result<Rc<Promise>, Error>> {
        if let UnderlyingSourceType::Js(source, this_obj) = &self.underlying_source_type {
            if let Some(algo) = &source.pull {
                let result = unsafe {
                    algo.Call_(
                        &SafeHandle::from_raw(this_obj.handle()),
                        controller,
                        ExceptionHandling::Rethrow,
                    )
                };
                return Some(result);
            }
        }
        // Note: other source type have no pull steps for now.
        None
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
    ///
    /// Note: The algorithm can return any value, including a promise,
    /// we always transform the result into a promise for convenience,
    /// and it is also how to spec deals with the situation.
    /// see "Let startPromise be a promise resolved with startResult."
    /// at <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    #[allow(unsafe_code)]
    pub fn call_start_algorithm(
        &self,
        controller: Controller,
        can_gc: CanGc,
    ) -> Option<Result<Rc<Promise>, Error>> {
        if let UnderlyingSourceType::Js(source, this_obj) = &self.underlying_source_type {
            if let Some(start) = &source.start {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
                rooted!(in(*cx) let mut result: JSVal);
                unsafe {
                    if let Err(error) = start.Call_(
                        &SafeHandle::from_raw(this_obj.handle()),
                        controller,
                        result.handle_mut(),
                        ExceptionHandling::Rethrow,
                    ) {
                        return Some(Err(error));
                    }
                }
                let is_promise = unsafe {
                    if result.is_object() {
                        result_object.set(result.to_object());
                        IsPromiseObject(result_object.handle().into_handle())
                    } else {
                        false
                    }
                };
                let promise = if is_promise {
                    let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                    promise
                } else {
                    let promise = Promise::new(&self.global(), can_gc);
                    promise.resolve_native(&result.get());
                    promise
                };
                return Some(Ok(promise));
            }
        }
        None
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source_type.in_memory()
    }
}
