/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLFramebufferBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

#[dom_struct]
pub struct WebGLFramebuffer {
    webgl_object: WebGLObject,
    id: u32,
}

impl WebGLFramebuffer {
    fn new_inherited(id: u32) -> WebGLFramebuffer {
        WebGLFramebuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Root<WebGLFramebuffer> {
        reflect_dom_object(box WebGLFramebuffer::new_inherited(id), global, WebGLFramebufferBinding::Wrap)
    }
}

pub trait WebGLFramebufferHelpers {
    fn get_id(self) -> u32;
}

impl<'a> WebGLFramebufferHelpers for &'a WebGLFramebuffer {
    fn get_id(self) -> u32 {
        self.id
    }
}
