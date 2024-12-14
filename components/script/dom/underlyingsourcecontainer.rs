/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::JSVal;
use js::rust::{Handle as SafeHandle, HandleObject, HandleValue as SafeHandleValue, IntoHandle};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::import::module::Error;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::defaultteeunderlyingsource::DefaultTeeUnderlyingSource;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
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
    Tee(Dom<DefaultTeeUnderlyingSource>),
}

impl UnderlyingSourceType {
    /// Is the source backed by a Rust native source?
    pub fn is_native(&self) -> bool {
        matches!(
            self,
            UnderlyingSourceType::Memory(_) |
                UnderlyingSourceType::Blob(_) |
                UnderlyingSourceType::FetchResponse
        )
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        matches!(self, UnderlyingSourceType::Memory(_))
    }
}

/// Wrapper around the underlying source.
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
                        )
                    };
                    return Some(result);
                }
                None
            },
            UnderlyingSourceType::Tee(tee_underlyin_source) => {
                // Call the cancel algorithm for the appropriate branch.
                tee_underlyin_source.cancel_algorithm(reason, can_gc)
            },
            _ => None,
        }
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-pull>
    #[allow(unsafe_code)]
    pub fn call_pull_algorithm(
        &self,
        controller: Controller,
    ) -> Option<Result<Rc<Promise>, Error>> {
        match &self.underlying_source_type {
            UnderlyingSourceType::Js(source, this_obj) => {
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
                None
            },
            UnderlyingSourceType::Tee(tee_underlyin_source) => {
                // Call the pull algorithm for the appropriate branch.
                tee_underlyin_source.pull_algorithm()
            },
            // Note: other source type have no pull steps for now.
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
    pub fn call_start_algorithm(
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
                None
            },
            UnderlyingSourceType::Tee(_) => {
                // Let startAlgorithm be an algorithm that returns undefined.
                None
            },
            _ => None,
        }
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source_type.in_memory()
    }
}
