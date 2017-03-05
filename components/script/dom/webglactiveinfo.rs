/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLActiveInfoBinding;
use dom::bindings::codegen::Bindings::WebGLActiveInfoBinding::WebGLActiveInfoMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct WebGLActiveInfo {
    reflector_: Reflector,
    size: i32,
    // NOTE: `ty` stands for `type`, which is a reserved keyword
    ty: u32,
    name: DOMString,
}

impl WebGLActiveInfo {
    fn new_inherited(size: i32, ty: u32, name: DOMString) -> WebGLActiveInfo {
        WebGLActiveInfo {
            reflector_: Reflector::new(),
            size: size,
            ty: ty,
            name: name,
        }
    }

    pub fn new(window: &Window, size: i32, ty: u32, name: DOMString) -> Root<WebGLActiveInfo> {
        reflect_dom_object(box WebGLActiveInfo::new_inherited(size, ty, name), window, WebGLActiveInfoBinding::Wrap)
    }
}

impl WebGLActiveInfoMethods for WebGLActiveInfo {
    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.11.1
    fn Size(&self) -> i32 {
        self.size
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.11.1
    fn Type(&self) -> u32 {
        self.ty
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.11.1
    fn Name(&self) -> DOMString {
        self.name.clone()
    }
}
