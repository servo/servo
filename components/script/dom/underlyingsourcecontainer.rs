/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{HandleObject, JSObject, Value};
use js::jsval::UndefinedValue;
use js::rust::wrappers::JS_GetProperty;

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

#[dom_struct]
pub struct UnderlyingSourceContainer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Rc is hard"]
    underlying_source_type: Rc<UnderlyingSourceType>,
}

impl UnderlyingSourceContainer {
    fn new_inherited(underlying_source_type: UnderlyingSourceType) -> UnderlyingSourceContainer {
        UnderlyingSourceContainer {
            reflector_: Reflector::new(),
            underlying_source_type: Rc::new(underlying_source_type),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
    ) -> DomRoot<UnderlyingSourceContainer> {
        // Setting the prototype of the underlying source dict on the
        // `UnderlyingSourceContainer` for later use in Call_.
        // TODO: is this a good idea?
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut constructor = UndefinedValue());
        rooted!(in(*cx) let mut prototype = UndefinedValue());
        rooted!(in(*cx) let mut constructor_obj = constructor.to_object());
        rooted!(in(*cx) let mut prototype_obj = prototype.to_object());
        let prototype = if let UnderlyingSourceType::Js(ref js_source) = underlying_source_type {
            unsafe {
                js_source.to_jsval(*cx, constructor.handle_mut());
                if !JS_GetProperty(
                    *GlobalScope::get_cx(),
                    constructor_obj.handle(),
                    b"prototype\0".as_ptr() as *const _,
                    prototype.handle_mut(),
                ) {
                    None
                } else if !prototype.get().is_object() {
                    None
                } else {
                    Some(prototype_obj.handle())
                }
            }
        } else {
            None
        };
        reflect_dom_object_with_proto(
            Box::new(UnderlyingSourceContainer::new_inherited(
                underlying_source_type,
            )),
            global,
            prototype,
        )
    }

    pub fn call_pull_algorithm(
        &self,
        controller: &ReadableStreamDefaultController,
    ) -> Option<Rc<Promise>> {
        if let UnderlyingSourceType::Js(source) = self.underlying_source_type.as_ref() {
            let global = self.global();
            let promise = if let Some(pull) = source.pull.as_ref() {
                // Note: this calls the pull callback with self as "this".
                // Should we find a way to pass the JsUnderlyingSource as "this"?
                pull.Call_(
                    &*self,
                    Controller::ReadableStreamDefaultController(DomRoot::from_ref(controller)),
                    ExceptionHandling::Report,
                )
                .expect("Pull algorithm call failed")
            } else {
                let promise = Promise::new(&*global);
                promise.resolve_native(&());
                promise
            };
            return Some(promise);
        }
        // Note: other source type have no pull steps for now.
        None
    }

    /// Does the source have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source_type.in_memory()
    }
}
