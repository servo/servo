/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{Handle as SafeHandle, HandleObject, HandleValue as SafeHandleValue, IntoHandle};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::codegen::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::defaultteeunderlyingsource::DefaultTeeUnderlyingSource;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::promise::Promise;
use crate::dom::transformstream::TransformStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#underlying-source-api>
/// The `Js` variant corresponds to
/// the JavaScript object representing the underlying source.
/// The other variants are native sources in Rust.
#[derive(JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum UnderlyingSourceType {
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
    Tee(Dom<DefaultTeeUnderlyingSource>),
    /// Transfer, with the port used in some of the algorithms.
    Transfer(Dom<MessagePort>),
    /// A struct representing a JS object as underlying source,
    /// and the actual JS object for use as `thisArg` in callbacks.
    /// This is used for the `TransformStream` API.
    Transform(Dom<TransformStream>, Rc<Promise>),
}

impl UnderlyingSourceType {
    /// Is the source backed by a Rust native source?
    pub(crate) fn is_native(&self) -> bool {
        matches!(
            self,
            UnderlyingSourceType::Memory(_) |
                UnderlyingSourceType::Blob(_) |
                UnderlyingSourceType::FetchResponse |
                UnderlyingSourceType::Transfer(_)
        )
    }

    /// Does the source have all data in memory?
    pub(crate) fn in_memory(&self) -> bool {
        matches!(self, UnderlyingSourceType::Memory(_))
    }
}

/// Wrapper around the underlying source.
#[dom_struct]
pub(crate) struct UnderlyingSourceContainer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "JsUnderlyingSource implemented in SM."]
    underlying_source_type: UnderlyingSourceType,
}

impl UnderlyingSourceContainer {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(underlying_source_type: UnderlyingSourceType) -> UnderlyingSourceContainer {
        UnderlyingSourceContainer {
            reflector_: Reflector::new(),
            underlying_source_type,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
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
    pub(crate) fn set_underlying_source_this_object(&self, object: HandleObject) {
        if let UnderlyingSourceType::Js(_source, this_obj) = &self.underlying_source_type {
            this_obj.set(*object);
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-cancel>
    #[allow(unsafe_code)]
    pub(crate) fn call_cancel_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match &self.underlying_source_type {
            UnderlyingSourceType::Js(source, this_obj) => {
                if let Some(algo) = &source.cancel {
                    let result = unsafe {
                        algo.Call_(
                            &SafeHandle::from_raw(this_obj.handle()),
                            Some(reason),
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                    };
                    return Some(result);
                }
                None
            },
            UnderlyingSourceType::Tee(tee_underlying_source) => {
                // Call the cancel algorithm for the appropriate branch.
                tee_underlying_source.cancel_algorithm(cx, global, reason, can_gc)
            },
            UnderlyingSourceType::Transform(stream, _) => {
                // Return ! TransformStreamDefaultSourceCancelAlgorithm(stream, reason).
                Some(stream.transform_stream_default_source_cancel(cx, global, reason, can_gc))
            },
            UnderlyingSourceType::Transfer(port) => {
                // Let cancelAlgorithm be the following steps, taking a reason argument:
                // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable

                // Let result be PackAndPostMessageHandlingError(port, "error", reason).
                let result = port.pack_and_post_message_handling_error("error", reason, can_gc);

                // Disentangle port.
                self.global().disentangle_port(port, can_gc);

                let promise = Promise::new(&self.global(), can_gc);

                // If result is an abrupt completion,
                if let Err(error) = result {
                    // Return a promise rejected with result.[[Value]].
                    promise.reject_error(error, can_gc);
                } else {
                    // Otherwise, return a promise resolved with undefined.
                    promise.resolve_native(&(), can_gc);
                }
                Some(Ok(promise))
            },
            _ => None,
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-pull>
    #[allow(unsafe_code)]
    pub(crate) fn call_pull_algorithm(
        &self,
        controller: Controller,
        _global: &GlobalScope,
        can_gc: CanGc,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match &self.underlying_source_type {
            UnderlyingSourceType::Js(source, this_obj) => {
                if let Some(algo) = &source.pull {
                    let result = unsafe {
                        algo.Call_(
                            &SafeHandle::from_raw(this_obj.handle()),
                            controller,
                            ExceptionHandling::Rethrow,
                            can_gc,
                        )
                    };
                    return Some(result);
                }
                None
            },
            UnderlyingSourceType::Tee(tee_underlying_source) => {
                // Call the pull algorithm for the appropriate branch.
                Some(Ok(tee_underlying_source.pull_algorithm(can_gc)))
            },
            UnderlyingSourceType::Transfer(port) => {
                // Let pullAlgorithm be the following steps:
                // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable

                let cx = GlobalScope::get_cx();

                // Perform ! PackAndPostMessage(port, "pull", undefined).
                rooted!(in(*cx) let mut value = UndefinedValue());
                port.pack_and_post_message("pull", value.handle(), can_gc)
                    .expect("Sending pull should not fail.");

                // Return a promise resolved with undefined.
                let promise = Promise::new(&self.global(), can_gc);
                promise.resolve_native(&(), can_gc);
                Some(Ok(promise))
            },
            // Note: other source type have no pull steps for now.
            UnderlyingSourceType::Transform(stream, _) => {
                // Return ! TransformStreamDefaultSourcePullAlgorithm(stream).
                Some(stream.transform_stream_default_source_pull(&self.global(), can_gc))
            },
            _ => None,
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
    ///
    /// Note: The algorithm can return any value, including a promise,
    /// we always transform the result into a promise for convenience,
    /// and it is also how to spec deals with the situation.
    /// see "Let startPromise be a promise resolved with startResult."
    /// at <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    #[allow(unsafe_code)]
    pub(crate) fn call_start_algorithm(
        &self,
        controller: Controller,
        can_gc: CanGc,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match &self.underlying_source_type {
            UnderlyingSourceType::Js(source, this_obj) => {
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
                            can_gc,
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
                        promise.resolve_native(&result.get(), can_gc);
                        promise
                    };
                    return Some(Ok(promise));
                }
                None
            },
            UnderlyingSourceType::Tee(_) => {
                // Let startAlgorithm be an algorithm that returns undefined.
                None
            },
            UnderlyingSourceType::Transfer(_) => {
                // Let startAlgorithm be an algorithm that returns undefined.
                // from <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable
                None
            },
            UnderlyingSourceType::Transform(_, start_promise) => {
                // Let startAlgorithm be an algorithm that returns startPromise.
                Some(Ok(start_promise.clone()))
            },
            _ => None,
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-autoallocatechunksize>
    pub(crate) fn auto_allocate_chunk_size(&self) -> Option<u64> {
        match &self.underlying_source_type {
            UnderlyingSourceType::Js(source, _) => source.autoAllocateChunkSize,
            _ => None,
        }
    }

    /// Does the source have all data in memory?
    pub(crate) fn in_memory(&self) -> bool {
        self.underlying_source_type.in_memory()
    }
}
