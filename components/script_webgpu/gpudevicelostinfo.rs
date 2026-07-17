/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::JSContext;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUDeviceLostInfoMethods, GPUDeviceLostInfoWrap, GPUDeviceLostReason,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx_and_wrap};

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;

#[dom_struct]
pub struct GPUDeviceLostInfo<D: DomTypes> {
    reflector_: Reflector,
    message: DOMString,
    reason: GPUDeviceLostReason,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUDeviceLostInfo<D>
where
    D: DomTypes<GPUDeviceLostInfo = GPUDeviceLostInfo<D>>,
{
    fn new_inherited(message: DOMString, reason: GPUDeviceLostReason) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
            reason,
            phantom: PhantomData,
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        message: DOMString,
        reason: GPUDeviceLostReason,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx_and_wrap::<D, _, _>(
            Box::new(GPUDeviceLostInfo::new_inherited(message, reason)),
            global,
            cx,
            GPUDeviceLostInfoWrap::<D>,
        )
    }
}

impl<D: DomTypes> GPUDeviceLostInfoMethods<D> for GPUDeviceLostInfo<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevicelostinfo-message>
    fn Message(&self) -> DOMString {
        self.message.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevicelostinfo-reason>
    fn Reason(&self) -> GPUDeviceLostReason {
        self.reason
    }
}
