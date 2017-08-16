/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLUniformLocationBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::window::Window;
use dom_struct::dom_struct;
use webrender_api::WebGLProgramId;

#[dom_struct]
pub struct WebGLUniformLocation {
    reflector_: Reflector,
    id: i32,
    program_id: WebGLProgramId,
}

impl WebGLUniformLocation {
    fn new_inherited(id: i32,
                     program_id: WebGLProgramId)
                     -> WebGLUniformLocation {
        WebGLUniformLocation {
            reflector_: Reflector::new(),
            id: id,
            program_id: program_id,
        }
    }

    pub fn new(window: &Window,
               id: i32,
               program_id: WebGLProgramId)
               -> Root<WebGLUniformLocation> {
        reflect_dom_object(box WebGLUniformLocation::new_inherited(id, program_id),
                           window,
                           WebGLUniformLocationBinding::Wrap)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn program_id(&self) -> WebGLProgramId {
        self.program_id
    }
}
