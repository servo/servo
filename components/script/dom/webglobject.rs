/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGLObjectBinding::WebGLObjectMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::str::USVString;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;

#[dom_struct]
pub(crate) struct WebGLObject {
    reflector_: Reflector,
    context: Dom<WebGLRenderingContext>,
    label: DomRefCell<USVString>,
}

impl WebGLObject {
    pub(crate) fn new_inherited(context: &WebGLRenderingContext) -> WebGLObject {
        WebGLObject {
            reflector_: Reflector::new(),
            context: Dom::from_ref(context),
            label: DomRefCell::new(USVString::default()),
        }
    }

    pub(crate) fn context(&self) -> &WebGLRenderingContext {
        &self.context
    }
}

impl WebGLObjectMethods<crate::DomTypeHolder> for WebGLObject {
    /// <https://registry.khronos.org/webgl/specs/latest/1.0/#5.3>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://registry.khronos.org/webgl/specs/latest/1.0/#5.3>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
