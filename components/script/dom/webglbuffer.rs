/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLBufferBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::reflect_dom_object;
use dom::webglobject::WebGLObject;

#[dom_struct]
pub struct WebGLBuffer {
    webgl_object: WebGLObject,
    id: u32,
}

impl WebGLBuffer {
    fn new_inherited(id: u32) -> WebGLBuffer {
        WebGLBuffer {
            webgl_object: WebGLObject::new_inherited(),
            id: id,
        }
    }

    pub fn new(global: GlobalRef, id: u32) -> Temporary<WebGLBuffer> {
        reflect_dom_object(box WebGLBuffer::new_inherited(id), global, WebGLBufferBinding::Wrap)
    }
}

pub trait WebGLBufferHelpers {
    fn get_id(&self) -> u32;
}

impl<'a> WebGLBufferHelpers for JSRef<'a, WebGLBuffer> {
    fn get_id(&self) -> u32 {
        self.id
    }
}
