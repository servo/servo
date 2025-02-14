/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleValue as SafeHandleValue;
use js::typedarray::ArrayBufferViewU8;

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderReadOptions;
use super::readablestreambyobreader::ReadIntoRequest;
use super::types::ReadableStreamBYOBRequest;
use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
#[dom_struct]
pub(crate) struct ReadableByteStreamController {
    reflector_: Reflector,
    stream: MutNullableDom<ReadableStream>,
}

impl ReadableByteStreamController {
    pub(crate) fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-pull-into>
    pub(crate) fn perform_pull_into(
        &self,
        _read_into_request: &ReadIntoRequest,
        _view: HeapBufferSource<ArrayBufferViewU8>,
        _options: &ReadableStreamBYOBReaderReadOptions,
        _can_gc: CanGc,
    ) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond>
    pub(crate) fn respond(&self, _bytes_written: u64) -> Fallible<()> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-with-new-view>
    pub(crate) fn respond_with_new_view(
        &self,
        _view: HeapBufferSource<ArrayBufferViewU8>,
    ) -> Fallible<()> {
        todo!()
    }
}

impl ReadableByteStreamControllerMethods<crate::DomTypeHolder> for ReadableByteStreamController {
    /// <https://streams.spec.whatwg.org/#rbs-controller-byob-request>
    fn GetByobRequest(&self) -> Fallible<Option<DomRoot<ReadableStreamBYOBRequest>>> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // TODO
        None
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-close>
    fn Close(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-enqueue>
    fn Enqueue(
        &self,
        _chunk: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-error>
    fn Error(&self, _cx: SafeJSContext, _e: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}
