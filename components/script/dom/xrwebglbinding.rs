/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRWebGLBindingBinding::XRWebGLBindingBinding::XRWebGLBindingMethods;
use crate::dom::bindings::codegen::UnionTypes::WebGLRenderingContextOrWebGL2RenderingContext as RootedWebGLRenderingContextOrWebGL2RenderingContext;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrwebglsubimage::XRWebGLSubImage;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRWebGLBinding {
    reflector: Reflector,
    session: Dom<XRSession>,
    context: WebGLRenderingContextOrWebGL2RenderingContext,
}

// TODO: Should this live somewhere else?
#[unrooted_must_root_lint::must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum WebGLRenderingContextOrWebGL2RenderingContext {
    WebGLRenderingContext(Dom<WebGLRenderingContext>),
    WebGL2RenderingContext(Dom<WebGL2RenderingContext>),
}

impl WebGLRenderingContextOrWebGL2RenderingContext {
    #[allow(unrooted_must_root)]
    fn from_ref(
        context: &RootedWebGLRenderingContextOrWebGL2RenderingContext,
    ) -> WebGLRenderingContextOrWebGL2RenderingContext {
        match context {
            RootedWebGLRenderingContextOrWebGL2RenderingContext::WebGLRenderingContext(
                ref context,
            ) => WebGLRenderingContextOrWebGL2RenderingContext::WebGLRenderingContext(
                Dom::from_ref(context),
            ),
            RootedWebGLRenderingContextOrWebGL2RenderingContext::WebGL2RenderingContext(
                ref context,
            ) => WebGLRenderingContextOrWebGL2RenderingContext::WebGL2RenderingContext(
                Dom::from_ref(context),
            ),
        }
    }
}

impl XRWebGLBindingMethods for XRWebGLBinding {
    /// https://immersive-web.github.io/layers/#dom-xrwebglbinding-getsubimage
    fn GetSubImage(&self, _layer: &XRLayer, _frame: &XRFrame) -> Option<DomRoot<XRWebGLSubImage>> {
        // TODO: Implement this
        None
    }

    /// https://immersive-web.github.io/layers/#dom-xrwebglbinding-getviewsubimage
    fn GetViewSubImage(
        &self,
        _layer: &XRLayer,
        _view: &XRView,
    ) -> Option<DomRoot<XRWebGLSubImage>> {
        // TODO: Implement this
        None
    }
}

impl XRWebGLBinding {
    pub fn new_inherited(
        session: &XRSession,
        context: &WebGLRenderingContextOrWebGL2RenderingContext,
    ) -> XRWebGLBinding {
        XRWebGLBinding {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            context: context.clone(),
        }
    }

    pub fn new(
        global: &Window,
        session: &XRSession,
        context: &WebGLRenderingContextOrWebGL2RenderingContext,
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
        context: RootedWebGLRenderingContextOrWebGL2RenderingContext,
    ) -> DomRoot<XRWebGLBinding> {
        XRWebGLBinding::new(
            global,
            session,
            &WebGLRenderingContextOrWebGL2RenderingContext::from_ref(&context),
        )
    }
}
