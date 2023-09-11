/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom_struct::dom_struct;

use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;

#[dom_struct]
pub struct WebGLObject {
    reflector_: Reflector,
    context: Dom<WebGLRenderingContext>,
}

impl WebGLObject {
    pub fn new_inherited(context: &WebGLRenderingContext) -> WebGLObject {
        WebGLObject {
            reflector_: Reflector::new(),
            context: Dom::from_ref(context),
        }
    }

    pub fn context(&self) -> &WebGLRenderingContext {
        &self.context
    }
}
