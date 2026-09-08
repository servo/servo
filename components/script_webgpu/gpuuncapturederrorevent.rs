/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUUncapturedErrorEventInit, GPUUncapturedErrorEventMethods, GPUUncapturedErrorEventWrap,
};
use script_bindings::reflector::reflect_dom_object_with_proto_and_wrap;
use stylo_atoms::Atom;

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::gpuerror::GPUError;
use crate::traits::{Equivalence, WebGPUEventTrait};
use crate::{DomObject, JSTraceable};

#[dom_struct(special)]
pub struct GPUUncapturedErrorEvent<D: DomTypes> {
    event: D::Event,
    #[ignore_malloc_size_of = "Because it is non-owning"]
    gpu_error: Dom<GPUError<D>>,
}

impl<D> GPUUncapturedErrorEvent<D>
where
    D: Equivalence,
    D::Event: WebGPUEventTrait,
{
    fn new_inherited(init: &GPUUncapturedErrorEventInit<D>) -> Self {
        Self {
            gpu_error: Dom::from_ref(&init.error),
            event: D::Event::new_inherited(),
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        event_type: Atom,
        init: &GPUUncapturedErrorEventInit<D>,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, None, event_type, init)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        event_type: Atom,
        init: &GPUUncapturedErrorEventInit<D>,
    ) -> DomRoot<Self> {
        let event = reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(GPUUncapturedErrorEvent::new_inherited(init)),
            global,
            proto,
            cx,
            GPUUncapturedErrorEventWrap::<D>,
        );
        event
            .event
            .init_event(event_type, init.parent.bubbles, init.parent.cancelable);
        event
    }
}

impl<D> GPUUncapturedErrorEventMethods<D> for GPUUncapturedErrorEvent<D>
where
    D: Equivalence,
    D::Event: WebGPUEventTrait,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuuncapturederrorevent-gpuuncapturederrorevent>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        event_type: DOMString,
        init: &GPUUncapturedErrorEventInit<D>,
    ) -> DomRoot<Self> {
        GPUUncapturedErrorEvent::new_with_proto(cx, global, proto, event_type.into(), init)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuuncapturederrorevent-error>
    fn Error(&self) -> DomRoot<GPUError<D>> {
        DomRoot::from_ref(&self.gpu_error)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
