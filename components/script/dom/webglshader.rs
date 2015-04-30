/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLShaderBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct WebGLShader {
    reflector_: Reflector,
    id: u32,
}

impl WebGLShader {
    fn new_inherited(id: u32) -> WebGLShader {
        WebGLShader {
            reflector_: Reflector::new(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Temporary<WebGLShader> {
        reflect_dom_object(box WebGLShader::new_inherited(id), global, WebGLShaderBinding::Wrap)
    }
}

pub trait WebGLShaderHelpers {
    fn get_id(&self) -> u32;
}

impl<'a> WebGLShaderHelpers for JSRef<'a, WebGLShader> {
    fn get_id(&self) -> u32 {
        self.id
    }
}

