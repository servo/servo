/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::ArrayBufferView;

use super::bindings::cell::DomRefCell;
use super::bindings::root::MutNullableDom;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readablestreambyobreader>
#[dom_struct]
pub(crate) struct ReadableStreamBYOBReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    stream: MutNullableDom<ReadableStream>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: DomRefCell<Rc<Promise>>,
}

impl ReadableStreamBYOBReader {
    pub(crate) fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamBYOBReader {
        ReadableStreamBYOBReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            closed_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
    }

    fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<ReadableStreamBYOBReader> {
        reflect_dom_object(
            Box::new(ReadableStreamBYOBReader::new_inherited(global, can_gc)),
            global,
            can_gc,
        )
    }
}

impl ReadableStreamBYOBReaderMethods<crate::DomTypeHolder> for ReadableStreamBYOBReader {
    /// <https://streams.spec.whatwg.org/#byob-reader-constructor>
    fn Constructor(
        _global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
        _can_gc: CanGc,
        _stream: &ReadableStream,
    ) -> Fallible<DomRoot<Self>> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    fn Read(&self, _view: CustomAutoRooterGuard<ArrayBufferView>, can_gc: CanGc) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global(), can_gc)
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-release-lock>
    fn ReleaseLock(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self, can_gc: CanGc) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global(), can_gc)
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, _reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global(), can_gc)
    }
}

impl ReadableStreamGenericReader for ReadableStreamBYOBReader {
    fn get_closed_promise(&self) -> Rc<Promise> {
        self.closed_promise.borrow().clone()
    }

    fn set_closed_promise(&self, promise: Rc<Promise>) {
        *self.closed_promise.borrow_mut() = promise;
    }

    fn set_stream(&self, stream: Option<&ReadableStream>) {
        self.stream.set(stream);
    }

    fn get_stream(&self) -> Option<DomRoot<ReadableStream>> {
        self.stream.get()
    }

    fn as_byob_reader(&self) -> Option<&ReadableStreamBYOBReader> {
        Some(self)
    }
}
