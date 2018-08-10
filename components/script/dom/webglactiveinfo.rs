/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::codegen::Bindings::WebGLActiveInfoBinding;
use dom::bindings::codegen::Bindings::WebGLActiveInfoBinding::WebGLActiveInfoMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct WebGLActiveInfo<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    size: i32,
    // NOTE: `ty` stands for `type`, which is a reserved keyword
    ty: u32,
    name: DOMString,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> WebGLActiveInfo<TH> {
    fn new_inherited(size: i32, ty: u32, name: DOMString) -> WebGLActiveInfo<TH> {
        WebGLActiveInfo {
            reflector_: Reflector::new(),
            size: size,
            ty: ty,
            name: name,
            _p: Default::default(),
        }
    }

    pub fn new(window: &Window<TH>, size: i32, ty: u32, name: DOMString) -> DomRoot<WebGLActiveInfo<TH>> {
        reflect_dom_object(
            Box::new(WebGLActiveInfo::new_inherited(size, ty, name)),
            window,
            WebGLActiveInfoBinding::Wrap
        )
    }
}

impl<TH: TypeHolderTrait> WebGLActiveInfoMethods for WebGLActiveInfo<TH> {
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
