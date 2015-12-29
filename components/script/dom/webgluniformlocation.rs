/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLUniformLocationBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct WebGLUniformLocation {
    reflector_: Reflector,
    id: i32,
    program_id: u32,
}

impl WebGLUniformLocation {
    fn new_inherited(id: i32, program_id: u32) -> WebGLUniformLocation {
        WebGLUniformLocation {
            reflector_: Reflector::new(),
            id: id,
            program_id: program_id,
        }
    }

    pub fn new(global: GlobalRef, id: i32, program_id: u32) -> Root<WebGLUniformLocation> {
        reflect_dom_object(
            box WebGLUniformLocation::new_inherited(id, program_id), global, WebGLUniformLocationBinding::Wrap)
    }
}


impl WebGLUniformLocation {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn program_id(&self) -> u32 {
        self.program_id
    }
}
