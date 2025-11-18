/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{WebGLCommand, WebGLContextId, WebGLMsgSender};
// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use dom_struct::dom_struct;
use script_bindings::root::DomRoot;
use script_bindings::weakref::WeakRef;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGLObjectBinding::WebGLObjectMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::str::USVString;
use crate::dom::webgl::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webglrenderingcontext::{Operation, capture_webgl_backtrace};

#[dom_struct]
pub(crate) struct WebGLObject {
    reflector_: Reflector,
    #[no_trace]
    webgl_sender: WebGLMsgSender,
    context: WeakRef<WebGLRenderingContext>,
    label: DomRefCell<USVString>,
}

impl WebGLObject {
    pub(crate) fn new_inherited(context: &WebGLRenderingContext) -> WebGLObject {
        WebGLObject {
            reflector_: Reflector::new(),
            webgl_sender: context.sender().clone(),
            context: WeakRef::new(context),
            label: DomRefCell::new(USVString::default()),
        }
    }

    /// Get the [`WebGLRenderingContext`] that created this object.
    ///
    /// If `None` is returned the [`WebGLRenderingContext`] has already been garbage collected.
    pub(crate) fn context(&self) -> Option<DomRoot<WebGLRenderingContext>> {
        self.context.root()
    }

    #[inline]
    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.webgl_sender.context_id()
    }

    #[inline]
    pub(crate) fn send_with_fallibility(&self, command: WebGLCommand, fallibility: Operation) {
        let result = self.webgl_sender.send(command, capture_webgl_backtrace());
        if matches!(fallibility, Operation::Infallible) {
            result.expect("Operation failed");
        }
    }

    #[inline]
    pub(crate) fn send_command(&self, command: WebGLCommand) {
        self.send_with_fallibility(command, Operation::Infallible);
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
