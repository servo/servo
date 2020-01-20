/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerInit;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use canvas_traits::webgl::WebGLFramebufferId;
use dom_struct::dom_struct;
use euclid::{Point2D, Rect, Size2D};
use std::convert::TryInto;
use webxr_api::SwapChainId as WebXRSwapChainId;
use webxr_api::{Viewport, Views};

#[dom_struct]
pub struct XRWebGLLayer {
    reflector_: Reflector,
    antialias: bool,
    depth: bool,
    stencil: bool,
    alpha: bool,
    #[ignore_malloc_size_of = "ids don't malloc"]
    swap_chain_id: Option<WebXRSwapChainId>,
    context: Dom<WebGLRenderingContext>,
    session: Dom<XRSession>,
    /// If none, this is an inline session (the composition disabled flag is true)
    framebuffer: Option<Dom<WebGLFramebuffer>>,
}

impl XRWebGLLayer {
    pub fn new_inherited(
        swap_chain_id: Option<WebXRSwapChainId>,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
    ) -> XRWebGLLayer {
        XRWebGLLayer {
            reflector_: Reflector::new(),
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            alpha: init.alpha,
            swap_chain_id,
            context: Dom::from_ref(context),
            session: Dom::from_ref(session),
            framebuffer: framebuffer.map(Dom::from_ref),
        }
    }

    pub fn new(
        global: &GlobalScope,
        swap_chain_id: Option<WebXRSwapChainId>,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
    ) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited(
                swap_chain_id,
                session,
                context,
                init,
                framebuffer,
            )),
            global,
            XRWebGLLayerBinding::Wrap,
        )
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-xrwebgllayer
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &Window,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
    ) -> Fallible<DomRoot<Self>> {
        let framebuffer;
        // Step 2
        if session.is_ended() {
            return Err(Error::InvalidState);
        }
        // XXXManishearth step 3: throw error if context is lost
        // XXXManishearth step 4: check XR compat flag for immersive sessions

        // Step 9.2. "Initialize layer’s framebuffer to a new opaque framebuffer created with context."
        let (swap_chain_id, framebuffer) = if session.is_immersive() {
            let size = session.with_session(|session| session.recommended_framebuffer_resolution());
            let (swap_chain_id, fb) = WebGLFramebuffer::maybe_new_webxr(session, context, size)
                .ok_or(Error::Operation)?;
            framebuffer = fb;
            (Some(swap_chain_id), Some(&*framebuffer))
        } else {
            (None, None)
        };

        // Step 9.3. "Allocate and initialize resources compatible with session’s XR device,
        // including GPU accessible memory buffers, as required to support the compositing of layer."

        // Step 9.4: "If layer’s resources were unable to be created for any reason,
        // throw an OperationError and abort these steps."

        // Ensure that we finish setting up this layer before continuing.
        context.Finish();

        // Step 10. "Return layer."
        Ok(XRWebGLLayer::new(
            &global.global(),
            swap_chain_id,
            session,
            context,
            init,
            framebuffer,
        ))
    }

    pub fn swap_chain_id(&self) -> WebXRSwapChainId {
        self.swap_chain_id
            .expect("swap_chain_id must not be called for inline sessions")
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }

    pub fn swap_buffers(&self) {
        if let WebGLFramebufferId::Opaque(id) = self
            .framebuffer
            .as_ref()
            .expect("swap_buffers must not be called for inline sessions")
            .id()
        {
            self.context.swap_buffers(Some(id));
        }
    }

    pub fn size(&self) -> Size2D<u32, Viewport> {
        if let Some(framebuffer) = self.framebuffer.as_ref() {
            let size = framebuffer.size().unwrap_or((0, 0));
            Size2D::new(
                size.0.try_into().unwrap_or(0),
                size.1.try_into().unwrap_or(0),
            )
        } else {
            Size2D::from_untyped(self.context.Canvas().get_size())
        }
    }
}

impl XRWebGLLayerMethods for XRWebGLLayer {
    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-depth
    fn Depth(&self) -> bool {
        self.depth
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-stencil
    fn Stencil(&self) -> bool {
        self.stencil
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-antialias
    fn Antialias(&self) -> bool {
        self.antialias
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-alpha
    fn Alpha(&self) -> bool {
        self.alpha
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-context
    fn Context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&self.context)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebuffer
    fn GetFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        self.framebuffer.as_ref().map(|x| DomRoot::from_ref(&**x))
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferwidth
    fn FramebufferWidth(&self) -> u32 {
        self.size().width
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferheight
    fn FramebufferHeight(&self) -> u32 {
        self.size().height
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport
    fn GetViewport(&self, view: &XRView) -> Option<DomRoot<XRViewport>> {
        if self.session != view.session() {
            return None;
        }

        let views = self.session.with_session(|s| s.views().clone());

        let viewport = match (view.Eye(), views) {
            (XREye::None, Views::Inline) => {
                let origin = Point2D::new(0, 0);
                Rect::new(origin, self.size().cast())
            },
            (XREye::None, Views::Mono(view)) => view.viewport,
            (XREye::Left, Views::Stereo(view, _)) => view.viewport,
            (XREye::Right, Views::Stereo(_, view)) => view.viewport,
            _ => return None,
        };

        Some(XRViewport::new(&self.global(), viewport))
    }
}
