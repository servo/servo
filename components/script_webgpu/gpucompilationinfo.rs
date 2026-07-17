/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::MutableHandleValue;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUCompilationInfoMethods, GPUCompilationInfoWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_wrap};
use script_bindings::utils::to_frozen_array;
use webgpu_traits::ShaderCompilationInfo;

use crate::JSTraceable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::gpucompilationmessage::GPUCompilationMessage;

#[dom_struct]
pub struct GPUCompilationInfo<D: DomTypes> {
    reflector_: Reflector,
    // currently we only get one message from wgpu
    msg: Vec<Dom<GPUCompilationMessage<D>>>,
}

impl<D> GPUCompilationInfo<D>
where
    D: DomTypes<
            GPUCompilationInfo = GPUCompilationInfo<D>,
            GPUCompilationMessage = GPUCompilationMessage<D>,
        >,
{
    pub(crate) fn new_inherited(msg: Vec<DomRoot<GPUCompilationMessage<D>>>) -> Self {
        Self {
            reflector_: Reflector::new(),
            msg: msg.into_iter().map(|event| event.as_traced()).collect(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        msg: Vec<DomRoot<GPUCompilationMessage<D>>>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(Self::new_inherited(msg)),
            global,
            None,
            cx,
            GPUCompilationInfoWrap::<D>,
        )
    }

    pub fn from(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        error: Option<ShaderCompilationInfo>,
    ) -> DomRoot<Self> {
        let msg = error
            .map(|error| vec![GPUCompilationMessage::from(cx, global, error)])
            .unwrap_or_default();
        Self::new(cx, global, msg)
    }
}

impl<D: DomTypes> GPUCompilationInfoMethods<D> for GPUCompilationInfo<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages>
    fn Messages(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        let messages: Vec<DomRoot<GPUCompilationMessage<D>>> =
            self.msg.iter().map(|msg| msg.as_rooted()).collect();
        to_frozen_array(cx, messages.as_slice(), retval)
    }
}
