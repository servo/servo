/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLTextureBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

#[dom_struct]
pub struct WebGLTexture {
    webgl_object: WebGLObject,
    id: u32,
}

impl WebGLTexture {
    fn new_inherited(id: u32) -> WebGLTexture {
        WebGLTexture {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Root<WebGLTexture> {
        reflect_dom_object(box WebGLTexture::new_inherited(id), global, WebGLTextureBinding::Wrap)
    }
}

pub trait WebGLTextureHelpers {
    fn get_id(self) -> u32;
}

impl<'a> WebGLTextureHelpers for &'a WebGLTexture {
    fn get_id(self) -> u32 {
        self.id
    }
}
