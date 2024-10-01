/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{IsPromiseObject, JSObject};
use js::jsval::UndefinedValue;
use js::rust::IntoHandle;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestreamdefaultcontroller::ReadableStreamDefaultController;
use crate::js::conversions::ToJSValConvertible;

/// <https://streams.spec.whatwg.org/#underlying-source-api>
/// The `Js` variant corresponds to
/// the JavaScript object representing the underlying source.
/// The other variants are native sources in Rust.
#[derive(JSTraceable)]
pub enum UnderlyingSourceType {
    /// Facilitate partial integration with sources
    /// that are currently read into memory.
    Memory(usize),
    /// A blob as underlying source, with a known total size.
    Blob(usize),
    /// A fetch response as underlying source.
    FetchResponse,
    /// A JS object as underlying source.
    Js(JsUnderlyingSource),
}

impl UnderlyingSourceType {
    /// Is the source backed by a Rust native source?
    pub fn is_native(&self) -> bool {
        match self {
            UnderlyingSourceType::Memory(_) |
            UnderlyingSourceType::Blob(_) |
            UnderlyingSourceType::FetchResponse => true,
            _ => false,
        }
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        matches!(self, UnderlyingSourceType::Memory(_))
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
    fn new_inherited(underlying_source_type: UnderlyingSourceType) -> UnderlyingSourceContainer {
        UnderlyingSourceContainer {
            reflector_: Reflector::new(),
            underlying_source_type: underlying_source_type,
        }
    }

    pub fn new(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
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
        )
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-pull>
    #[allow(unsafe_code)]
    pub fn call_pull_algorithm(&self, controller: Controller) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = &self.underlying_source_type {
            if let Some(pull) = &source.pull {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
                unsafe {
                    source.to_jsobject(*cx, this_object.handle_mut());
                }
                let this_handle = this_object.handle();
                let promise = pull
                    .Call_(&this_handle, controller, ExceptionHandling::Report)
                    .expect("Pull algorithm call failed");
                return Some(promise);
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
    pub fn call_start_algorithm(&self, controller: Controller) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = &self.underlying_source_type {
            if let Some(start) = &source.start {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
                unsafe {
                    source.to_jsobject(*cx, this_object.handle_mut());
                }
                let this_handle = this_object.handle();
                rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
                let result = start
                    .Call_(&this_handle, controller, ExceptionHandling::Report)
                    .expect("Start algorithm call failed");
                let is_promise = unsafe {
                    result_object.set(result.to_object());
                    IsPromiseObject(result_object.handle().into_handle())
                };
                // Let startPromise be a promise resolved with startResult.
                // from #set-up-readable-stream-default-controller.
                let promise = if is_promise {
                    let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                    promise
                } else {
                    let global = self.global();
                    let promise = Promise::new(&*global);
                    promise.resolve_native(&result);
                    promise
                };
                return Some(promise);
            }
        }
        None
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source_type.in_memory()
    }
}
