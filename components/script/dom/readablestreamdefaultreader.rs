/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
#[dom_struct]
pub struct ReadableStreamDefaultReader {
    reflector_: Reflector,
}

impl ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-constructor>
    #[allow(non_snake_case)]
    pub fn Constructor(
        _global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
        _stream: DomRoot<ReadableStream>,
    ) -> Fallible<DomRoot<Self>> {
        // TODO
        Err(Error::NotFound)
    }

    fn new_inherited() -> ReadableStreamDefaultReader {
        ReadableStreamDefaultReader {
            reflector_: Reflector::new(),
        }
    }

    fn new(global: &GlobalScope) -> DomRoot<ReadableStreamDefaultReader> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultReader::new_inherited()),
            global,
        )
    }
}

impl ReadableStreamDefaultReaderMethods for ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    fn Read(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    fn ReleaseLock(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, _reason: SafeHandleValue) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }
}
