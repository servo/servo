/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom::bindings::reflector::Reflector;
use dom::bindings::root::Dom;
use dom::webglrenderingcontext::WebGLRenderingContext;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct WebGLObject<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    context: Dom<WebGLRenderingContext<TH>>,
}

impl<TH: TypeHolderTrait> WebGLObject<TH> {
    pub fn new_inherited(context: &WebGLRenderingContext<TH>) -> WebGLObject<TH> {
        WebGLObject {
            reflector_: Reflector::new(),
            context: Dom::from_ref(context),
        }
    }

    pub fn context(&self) -> &WebGLRenderingContext<TH> {
        &self.context
    }
}
