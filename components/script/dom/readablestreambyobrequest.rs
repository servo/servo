/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::jsapi::Heap;
use js::typedarray::{ArrayBufferView, ArrayBufferViewU8};

use super::bindings::buffer_source::{BufferSource, HeapBufferSource};
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBRequestBinding::ReadableStreamBYOBRequestMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::types::GlobalScope;
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
    fn GetView(&self, _cx: SafeJSContext) -> Option<js::typedarray::ArrayBufferView> {
        // Return this.[[view]].
        self.view.buffer_to_option()
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond>
    fn Respond(&self, bytes_written: u64) -> Fallible<()> {
        let cx = GlobalScope::get_cx();

        // If this.[[controller]] is undefined, throw a TypeError exception.
        let controller = if let Some(controller) = self.controller.get() {
            controller
        } else {
            return Err(Error::Type("controller is undefined".to_owned()));
        };

        // If ! IsDetachedBuffer(this.[[view]].[[ArrayBuffer]]) is true, throw a TypeError exception.
        if self.view.is_detached_buffer(cx) {
            return Err(Error::Type("buffer is detached".to_owned()));
        }

        // Assert: this.[[view]].[[ByteLength]] > 0.
        assert!(self.view.byte_length() > 0);

        // Assert: this.[[view]].[[ViewedArrayBuffer]].[[ByteLength]] > 0.
        assert!(self.view.viewed_buffer_array_byte_length(cx) > 0);

        // Perform ? ReadableByteStreamControllerRespond(this.[[controller]], bytesWritten).
        controller.respond(bytes_written)
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond-with-new-view>
    #[allow(unsafe_code)]
    fn RespondWithNewView(&self, view: CustomAutoRooterGuard<ArrayBufferView>) -> Fallible<()> {
        let view = HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(
            Heap::boxed(unsafe { *view.underlying_object() }),
        ));

        // If this.[[controller]] is undefined, throw a TypeError exception.
        let controller = if let Some(controller) = self.controller.get() {
            controller
        } else {
            return Err(Error::Type("controller is undefined".to_owned()));
        };

        // If ! IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
        if self.view.is_detached_buffer(GlobalScope::get_cx()) {
            return Err(Error::Type("buffer is detached".to_owned()));
        }

        // Return ? ReadableByteStreamControllerRespondWithNewView(this.[[controller]], view).
        controller.respond_with_new_view(view)
    }
}
