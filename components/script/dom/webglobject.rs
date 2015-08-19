/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLObjectBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct WebGLObject {
    reflector_: Reflector,
}

impl WebGLObject {
    pub fn new_inherited() -> WebGLObject {
        WebGLObject {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<WebGLObject> {
        reflect_dom_object(box WebGLObject::new_inherited(), global, WebGLObjectBinding::Wrap)
    }
}
