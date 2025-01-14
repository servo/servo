/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::WebGLActiveInfoBinding::WebGLActiveInfoMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLActiveInfo {
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
            size,
            ty,
            name,
        }
    }

    pub(crate) fn new(
        window: &Window,
        size: i32,
        ty: u32,
        name: DOMString,
    ) -> DomRoot<WebGLActiveInfo> {
        reflect_dom_object(
            Box::new(WebGLActiveInfo::new_inherited(size, ty, name)),
            window,
            CanGc::note(),
        )
    }
}

impl WebGLActiveInfoMethods<crate::DomTypeHolder> for WebGLActiveInfo {
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
