/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::WebGLProgramId;
use dom::bindings::codegen::Bindings::WebGLUniformLocationBinding;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct WebGLUniformLocation<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    id: i32,
    program_id: WebGLProgramId,
    link_generation: u64,
    size: Option<i32>,
    type_: u32,
}

impl<TH: TypeHolderTrait> WebGLUniformLocation<TH> {
    fn new_inherited(
        id: i32,
        program_id: WebGLProgramId,
        link_generation: u64,
        size: Option<i32>,
        type_: u32,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            id,
            program_id,
            link_generation,
            size,
            type_,
        }
    }

    pub fn new(
        window: &Window<TH>,
        id: i32,
        program_id: WebGLProgramId,
        link_generation: u64,
        size: Option<i32>,
        type_: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(id, program_id, link_generation, size, type_)),
            window,
            WebGLUniformLocationBinding::Wrap,
        )
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn program_id(&self) -> WebGLProgramId {
        self.program_id
    }

    pub fn link_generation(&self) -> u64 {
        self.link_generation
    }

    pub fn size(&self) -> Option<i32> {
        self.size
    }

    pub fn type_(&self) -> u32 {
        self.type_
    }
}
