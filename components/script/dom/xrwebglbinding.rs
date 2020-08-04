/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::UnionTypes::WebGLRenderingContextOrWebGL2RenderingContext;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRWebGLBinding {
    reflector: Reflector,
    session: Dom<XRSession>,
    context: Dom<WebGLRenderingContext>,
}

impl XRWebGLBinding {
    pub fn new_inherited(session: &XRSession, context: &WebGLRenderingContext) -> XRWebGLBinding {
        XRWebGLBinding {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            context: Dom::from_ref(context),
        }
    }

    pub fn new(
        global: &Window,
        session: &XRSession,
        context: &WebGLRenderingContext,
    ) -> DomRoot<XRWebGLBinding> {
        reflect_dom_object(
            Box::new(XRWebGLBinding::new_inherited(session, context)),
            global,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &Window,
        session: &XRSession,
        context: WebGLRenderingContextOrWebGL2RenderingContext,
    ) -> DomRoot<XRWebGLBinding> {
        let context = match context {
            WebGLRenderingContextOrWebGL2RenderingContext::WebGLRenderingContext(ctx) => ctx,
            WebGLRenderingContextOrWebGL2RenderingContext::WebGL2RenderingContext(ctx) => {
                ctx.base_context()
            },
        };
        XRWebGLBinding::new(global, session, &context)
    }
}
