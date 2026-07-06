/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::MutableHandleValue;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use webgpu_traits::ShaderCompilationInfo;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCompilationInfoMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::GPUCompilationMessage;

#[dom_struct]
pub(crate) struct GPUCompilationInfo {
    reflector_: Reflector,
    // currently we only get one message from wgpu
    msg: Vec<Dom<GPUCompilationMessage>>,
}

impl GPUCompilationInfo {
    pub(crate) fn new_inherited(msg: Vec<DomRoot<GPUCompilationMessage>>) -> Self {
        Self {
            reflector_: Reflector::new(),
            msg: msg.into_iter().map(|event| event.as_traced()).collect(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        msg: Vec<DomRoot<GPUCompilationMessage>>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_cx(Box::new(Self::new_inherited(msg)), global, None, cx)
    }

    pub(crate) fn from(
        cx: &mut JSContext,
        global: &GlobalScope,
        error: Option<ShaderCompilationInfo>,
    ) -> DomRoot<Self> {
        let msg = error
            .map(|error| vec![GPUCompilationMessage::from(cx, global, error)])
            .unwrap_or_default();
        Self::new(cx, global, msg)
    }
}

impl GPUCompilationInfoMethods<crate::DomTypeHolder> for GPUCompilationInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages>
    fn Messages(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        let messages: Vec<DomRoot<GPUCompilationMessage>> =
            self.msg.iter().map(|msg| msg.as_rooted()).collect();
        to_frozen_array(cx, messages.as_slice(), retval)
    }
}
