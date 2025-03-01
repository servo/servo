/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;
use webgpu::ShaderCompilationInfo;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCompilationInfoMethods;
use crate::dom::bindings::import::module::DomRoot;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::GPUCompilationMessage;
use crate::script_runtime::{CanGc, JSContext};

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

    #[allow(dead_code)]
    pub(crate) fn new(
        global: &GlobalScope,
        msg: Vec<DomRoot<GPUCompilationMessage>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(Self::new_inherited(msg)), global, None, can_gc)
    }

    pub(crate) fn from(
        global: &GlobalScope,
        error: Option<ShaderCompilationInfo>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new(
            global,
            if let Some(error) = error {
                vec![GPUCompilationMessage::from(global, error, can_gc)]
            } else {
                Vec::new()
            },
            can_gc,
        )
    }
}

impl GPUCompilationInfoMethods<crate::DomTypeHolder> for GPUCompilationInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages>
    fn Messages(&self, cx: JSContext, retval: MutableHandleValue) {
        to_frozen_array(self.msg.as_slice(), cx, retval)
    }
}
