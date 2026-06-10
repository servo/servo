/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use script_bindings::script_runtime::CanGc;
use webgpu_traits::ShaderCompilationInfo;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCompilationInfoMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::GPUCompilationMessage;
use crate::script_runtime::JSContext;

#[dom_struct]
pub(crate) struct GPUCompilationInfo {
    reflector_: Reflector,
    // currently we only get one message from wgpu
    msg: Vec<DomRoot<GPUCompilationMessage>>,
}

impl GPUCompilationInfo {
    pub(crate) fn new_inherited(msg: Vec<DomRoot<GPUCompilationMessage>>) -> Self {
        Self {
            reflector_: Reflector::new(),
            msg,
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        msg: Vec<DomRoot<GPUCompilationMessage>>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_cx(Box::new(Self::new_inherited(msg)), global, None, cx)
    }

    pub(crate) fn from(
        cx: &mut js::context::JSContext,
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
    fn Messages(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(self.msg.as_slice(), cx, retval, can_gc)
    }
}
