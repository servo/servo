/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::typedarray::{ArrayBufferView, ArrayBufferViewU8};

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::root::MutNullableDom;
use super::types::ReadableByteStreamController;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBRequestBinding::ReadableStreamBYOBRequestMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablestreambyobrequest>
#[dom_struct]
pub(crate) struct ReadableStreamBYOBRequest {
    reflector_: Reflector,
    controller: MutNullableDom<ReadableByteStreamController>,
    #[ignore_malloc_size_of = "mozjs"]
    view: HeapBufferSource<ArrayBufferViewU8>,
}

impl ReadableStreamBYOBRequestMethods<crate::DomTypeHolder> for ReadableStreamBYOBRequest {
    /// <https://streams.spec.whatwg.org/#rs-byob-request-view>
    fn GetView(&self, _cx: SafeJSContext) -> Option<ArrayBufferView> {
        self.view.get_buffer().ok()
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond>
    fn Respond(&self, _bytes_written: u64) -> Fallible<()> {
        // If this.[[controller]] is undefined, throw a TypeError exception.
        assert!(self.controller.get().is_some());

        // If ! IsDetachedBuffer(this.[[view]].[[ArrayBuffer]]) is true, throw a TypeError exception.
        assert!(!self.view.is_detached_buffer());

        // Assert: this.[[view]].[[ByteLength]] > 0.
        assert!(self.view.byte_length() > 0);

        // Assert: this.[[view]].[[ViewedArrayBuffer]].[[ByteLength]] > 0.
        assert!(self.view.viewed_buffer_byte_length() > 0);

        // Perform ? ReadableByteStreamControllerRespond(this.[[controller]], bytesWritten).

        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond-with-new-view>
    fn RespondWithNewView(
        &self,
        view: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}
