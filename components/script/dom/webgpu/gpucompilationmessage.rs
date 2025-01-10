/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)] // this file is stub as wgpu does not provide info

use dom_struct::dom_struct;
use webgpu::ShaderCompilationInfo;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCompilationMessageMethods, GPUCompilationMessageType,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::types::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUCompilationMessage {
    reflector_: Reflector,
    // #[ignore_malloc_size_of = "defined in wgpu-types"]
    message: DOMString,
    mtype: GPUCompilationMessageType,
    line_num: u64,
    line_pos: u64,
    offset: u64,
    length: u64,
}

impl GPUCompilationMessage {
    fn new_inherited(
        message: DOMString,
        mtype: GPUCompilationMessageType,
        line_num: u64,
        line_pos: u64,
        offset: u64,
        length: u64,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
            mtype,
            line_num,
            line_pos,
            offset,
            length,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        message: DOMString,
        mtype: GPUCompilationMessageType,
        line_num: u64,
        line_pos: u64,
        offset: u64,
        length: u64,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(
                message, mtype, line_num, line_pos, offset, length,
            )),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn from(global: &GlobalScope, info: ShaderCompilationInfo) -> DomRoot<Self> {
        GPUCompilationMessage::new(
            global,
            info.message.into(),
            GPUCompilationMessageType::Error,
            info.line_number,
            info.line_pos,
            info.offset,
            info.length,
        )
    }
}

impl GPUCompilationMessageMethods<crate::DomTypeHolder> for GPUCompilationMessage {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-message>
    fn Message(&self) -> DOMString {
        self.message.to_owned()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-type>
    fn Type(&self) -> GPUCompilationMessageType {
        self.mtype
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-linenum>
    fn LineNum(&self) -> u64 {
        self.line_num
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-linepos>
    fn LinePos(&self) -> u64 {
        self.line_pos
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-offset>
    fn Offset(&self) -> u64 {
        self.offset
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-length>
    fn Length(&self) -> u64 {
        self.length
    }
}
