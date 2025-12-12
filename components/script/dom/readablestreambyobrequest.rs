/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::typedarray::{ArrayBufferView, ArrayBufferViewU8};
use script_bindings::trace::RootedTraceableBox;

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::cell::DomRefCell;
use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBRequestBinding::ReadableStreamBYOBRequestMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::types::GlobalScope;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readablestreambyobrequest>
#[dom_struct]
pub(crate) struct ReadableStreamBYOBRequest {
    reflector_: Reflector,
    controller: MutNullableDom<ReadableByteStreamController>,
    #[ignore_malloc_size_of = "mozjs"]
    view: DomRefCell<HeapBufferSource<ArrayBufferViewU8>>,
}

impl ReadableStreamBYOBRequest {
    fn new_inherited() -> ReadableStreamBYOBRequest {
        ReadableStreamBYOBRequest {
            reflector_: Reflector::new(),
            controller: MutNullableDom::new(None),
            view: DomRefCell::new(HeapBufferSource::<ArrayBufferViewU8>::default()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<ReadableStreamBYOBRequest> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, can_gc)
    }

    pub(crate) fn set_controller(&self, controller: Option<&ReadableByteStreamController>) {
        self.controller.set(controller);
    }

    pub(crate) fn set_view(&self, view: Option<HeapBufferSource<ArrayBufferViewU8>>) {
        match view {
            Some(view) => {
                *self.view.borrow_mut() = view;
            },
            None => {
                *self.view.borrow_mut() = HeapBufferSource::<ArrayBufferViewU8>::default();
            },
        }
    }

    pub(crate) fn get_view(&self) -> HeapBufferSource<ArrayBufferViewU8> {
        self.view.borrow().clone()
    }
}

impl ReadableStreamBYOBRequestMethods<crate::DomTypeHolder> for ReadableStreamBYOBRequest {
    /// <https://streams.spec.whatwg.org/#rs-byob-request-view>
    fn GetView(
        &self,
        _cx: SafeJSContext,
    ) -> Option<RootedTraceableBox<js::typedarray::HeapArrayBufferView>> {
        // Return this.[[view]].
        self.view.borrow().typed_array_to_option()
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond>
    fn Respond(&self, bytes_written: u64, can_gc: CanGc) -> Fallible<()> {
        let cx = GlobalScope::get_cx();

        // If this.[[controller]] is undefined, throw a TypeError exception.
        let controller = if let Some(controller) = self.controller.get() {
            controller
        } else {
            return Err(Error::Type("controller is undefined".to_owned()));
        };

        {
            let view = self.view.borrow();
            // If ! IsDetachedBuffer(this.[[view]].[[ArrayBuffer]]) is true, throw a TypeError exception.
            if view.get_array_buffer_view_buffer(cx).is_detached_buffer(cx) {
                return Err(Error::Type("buffer is detached".to_owned()));
            }

            // Assert: this.[[view]].[[ByteLength]] > 0.
            assert!(view.byte_length() > 0);

            // Assert: this.[[view]].[[ViewedArrayBuffer]].[[ByteLength]] > 0.
            assert!(view.viewed_buffer_array_byte_length(cx) > 0);
        }

        // Perform ? ReadableByteStreamControllerRespond(this.[[controller]], bytesWritten).
        controller.respond(cx, bytes_written, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond-with-new-view>
    fn RespondWithNewView(
        &self,
        view: CustomAutoRooterGuard<ArrayBufferView>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        let view = HeapBufferSource::<ArrayBufferViewU8>::from_view(view);

        // If this.[[controller]] is undefined, throw a TypeError exception.
        let controller = if let Some(controller) = self.controller.get() {
            controller
        } else {
            return Err(Error::Type("controller is undefined".to_owned()));
        };

        // If ! IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
        if view.is_detached_buffer(cx) {
            return Err(Error::Type("buffer is detached".to_owned()));
        }

        // Return ? ReadableByteStreamControllerRespondWithNewView(this.[[controller]], view).
        controller.respond_with_new_view(cx, view, can_gc)
    }
}
