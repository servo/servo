/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUCompilationInfoMethods;
use super::bindings::root::Dom;
use super::types::GPUCompilationMessage;
use crate::dom::bindings::reflector::Reflector;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct GPUCompilationInfo {
    reflector_: Reflector,
    msg: Dom<GPUCompilationMessage>,
}

// TODO: wgpu does not expose right fields right now
impl GPUCompilationInfoMethods for GPUCompilationInfo {
    /// https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages
    fn Messages(&self, _cx: JSContext) -> JSVal {
        todo!()
    }
}
