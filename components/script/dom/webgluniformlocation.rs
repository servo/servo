/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLUniformLocationBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

#[dom_struct]
pub struct WebGLUniformLocation {
    webgl_object: WebGLObject,
    id: u32,
}

impl WebGLUniformLocation {
    fn new_inherited(id: u32) -> WebGLUniformLocation {
        WebGLUniformLocation {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Temporary<WebGLUniformLocation> {
        reflect_dom_object(box WebGLUniformLocation::new_inherited(id), global, WebGLUniformLocationBinding::Wrap)
    }
}

pub trait WebGLUniformLocationHelpers {
    fn get_id(&self) -> u32;
}

impl<'a> WebGLUniformLocationHelpers for JSRef<'a, WebGLUniformLocation> {
    fn get_id(&self) -> u32 {
        self.id
    }
}
