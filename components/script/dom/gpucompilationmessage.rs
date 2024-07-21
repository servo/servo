/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)] // this file is stub as wgpu does not provide info

use dom_struct::dom_struct;
use webgpu::ShaderCompilationInfo;

use super::bindings::codegen::Bindings::WebGPUBinding::{
    GPUCompilationMessageMethods, GPUCompilationMessageType,
};
use super::bindings::root::DomRoot;
use super::types::GlobalScope;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::test::DOMString;

#[dom_struct]
pub struct GPUCompilationMessage {
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

    pub fn new(
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
        )
    }

    pub fn from(global: &GlobalScope, info: ShaderCompilationInfo) -> DomRoot<Self> {
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

impl GPUCompilationMessageMethods for GPUCompilationMessage {
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
