/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLContextId, WebGLProgramId};
use dom_struct::dom_struct;

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLUniformLocation {
    reflector_: Reflector,
    id: i32,
    #[no_trace]
    context_id: WebGLContextId,
    #[no_trace]
    program_id: WebGLProgramId,
    link_generation: u64,
    size: Option<i32>,
    type_: u32,
}

impl WebGLUniformLocation {
    fn new_inherited(
        id: i32,
        context_id: WebGLContextId,
        program_id: WebGLProgramId,
        link_generation: u64,
        size: Option<i32>,
        type_: u32,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            id,
            context_id,
            program_id,
            link_generation,
            size,
            type_,
        }
    }

    pub(crate) fn new(
        window: &Window,
        id: i32,
        context_id: WebGLContextId,
        program_id: WebGLProgramId,
        link_generation: u64,
        size: Option<i32>,
        type_: u32,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(
                id,
                context_id,
                program_id,
                link_generation,
                size,
                type_,
            )),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn id(&self) -> i32 {
        self.id
    }

    pub(crate) fn program_id(&self) -> WebGLProgramId {
        self.program_id
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.context_id
    }

    pub(crate) fn link_generation(&self) -> u64 {
        self.link_generation
    }

    pub(crate) fn size(&self) -> Option<i32> {
        self.size
    }

    pub(crate) fn type_(&self) -> u32 {
        self.type_
    }
}
