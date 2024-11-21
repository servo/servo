/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{IsPromiseObject, JSObject, JS_NewObject};
use js::jsval::JSVal;
use js::rust::{HandleValue as SafeHandleValue, IntoHandle};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

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
            underlying_source_type,
        }
    }

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

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-cancel>
    #[allow(unsafe_code)]
    pub fn call_cancel_algorithm(&self, reason: SafeHandleValue) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = &self.underlying_source_type {
            if let Some(pull) = &source.cancel {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
                // TODO: move this into `bindings`.
                unsafe {
                    let obj = JS_NewObject(*cx, ptr::null_mut());
                    assert!(!obj.is_null());
                    this_object.set(obj);
                    source.to_jsobject(*cx, this_object.handle_mut());
                }
                let this_handle = this_object.handle();
                return pull
                    .Call_(&this_handle, Some(reason), ExceptionHandling::Report)
                    .ok();
            }
        }
        None
    }

    /// <https://streams.spec.whatwg.org/#dom-underlyingsource-pull>
    #[allow(unsafe_code)]
    pub fn call_pull_algorithm(&self, controller: Controller) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = &self.underlying_source_type {
            if let Some(pull) = &source.pull {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
                // TODO: move this into `bindings`.
                unsafe {
                    let obj = JS_NewObject(*cx, ptr::null_mut());
                    assert!(!obj.is_null());
                    this_object.set(obj);
                    source.to_jsobject(*cx, this_object.handle_mut());
                }
                let this_handle = this_object.handle();
                return pull
                    .Call_(&this_handle, controller, ExceptionHandling::Report)
                    .ok();
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
    ) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = &self.underlying_source_type {
            if let Some(start) = &source.start {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
                // TODO: move this into `bindings`.
                unsafe {
                    let obj = JS_NewObject(*cx, ptr::null_mut());
                    assert!(!obj.is_null());
                    this_object.set(obj);
                    source.to_jsobject(*cx, this_object.handle_mut());
                }
                let this_handle = this_object.handle();
                rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
                rooted!(in(*cx) let mut result: JSVal);

                if start
                    .Call_(
                        &this_handle,
                        controller,
                        result.handle_mut(),
                        ExceptionHandling::Report,
                    )
                    .is_err()
                {
                    return None;
                }
                let is_promise = unsafe {
                    if result.is_object() {
                        result_object.set(result.to_object());
                        IsPromiseObject(result_object.handle().into_handle())
                    } else {
                        false
                    }
                };
                // Let startPromise be a promise resolved with startResult.
                // from #set-up-readable-stream-default-controller.
                let promise = if is_promise {
                    let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                    promise
                } else {
                    let promise = Promise::new(&self.global(), can_gc);
                    promise.resolve_native(&result.get());
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
