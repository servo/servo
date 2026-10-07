/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::JSContext;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUCompilationMessageMethods, GPUCompilationMessageType, GPUCompilationMessageWrap,
};
use script_bindings::domstring::TracedDOMString;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{CompilationMessage, Utf16SourceLocation};

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::gpuconvert::WebGPUConvert as _;
use crate::traits::Equivalence;

#[dom_struct]
pub struct GPUCompilationMessage<D: DomTypes> {
    reflector_: Reflector,
    message: TracedDOMString,
    mtype: GPUCompilationMessageType,
    line_num: u64,
    line_pos: u64,
    offset: u64,
    length: u64,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D: Equivalence> GPUCompilationMessage<D> {
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
            message: message.traced(),
            mtype,
            line_num,
            line_pos,
            offset,
            length,
            phantom: PhantomData,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        message: DOMString,
        mtype: GPUCompilationMessageType,
        line_num: u64,
        line_pos: u64,
        offset: u64,
        length: u64,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            cx,
            Box::new(Self::new_inherited(
                message, mtype, line_num, line_pos, offset, length,
            )),
            global,
            GPUCompilationMessageWrap::<D>,
        )
    }

    pub(crate) fn from(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        info: CompilationMessage,
    ) -> DomRoot<Self> {
        let location = info.location.unwrap_or(Utf16SourceLocation {
            line_number: 0,
            offset: 0,
            length: 0,
            line_position: 0,
        });
        GPUCompilationMessage::new(
            cx,
            global,
            info.message.into(),
            info.message_type.convert(),
            location.line_number as u64,
            location.line_position as u64,
            location.offset as u64,
            location.length as u64,
        )
    }
}

impl<D: DomTypes> GPUCompilationMessageMethods<D> for GPUCompilationMessage<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpucompilationmessage-message>
    fn Message(&self) -> DOMString {
        self.message.to_owned().root()
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
