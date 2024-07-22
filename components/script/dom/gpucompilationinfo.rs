/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;
use webgpu::ShaderCompilationInfo;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUCompilationInfoMethods;
use super::bindings::import::module::DomRoot;
use super::bindings::reflector::reflect_dom_object_with_proto;
use super::bindings::utils::to_frozen_array;
use super::types::GPUCompilationMessage;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct GPUCompilationInfo {
    reflector_: Reflector,
    // currently we only get one message from wgpu
    msg: Vec<DomRoot<GPUCompilationMessage>>,
}

impl GPUCompilationInfo {
    pub fn new_inherited(msg: Vec<DomRoot<GPUCompilationMessage>>) -> Self {
        Self {
            reflector_: Reflector::new(),
            msg,
        }
    }

    #[allow(dead_code)]
    pub fn new(global: &GlobalScope, msg: Vec<DomRoot<GPUCompilationMessage>>) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(Self::new_inherited(msg)), global, None)
    }

    pub fn from(global: &GlobalScope, error: Option<ShaderCompilationInfo>) -> DomRoot<Self> {
        Self::new(
            global,
            if let Some(error) = error {
                vec![GPUCompilationMessage::from(global, error)]
            } else {
                Vec::new()
            },
        )
    }
}

impl GPUCompilationInfoMethods for GPUCompilationInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages>
    fn Messages(&self, cx: JSContext) -> JSVal {
        to_frozen_array(self.msg.as_slice(), cx)
    }
}
