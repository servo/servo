/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerInit;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgl_validations::types::TexImageTarget;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use dom_struct::dom_struct;
use js::rust::CustomAutoRooter;
use std::convert::TryInto;
use webxr_api::Views;

#[dom_struct]
pub struct XRWebGLLayer {
    xrlayer: XRLayer,
    antialias: bool,
    depth: bool,
    stencil: bool,
    alpha: bool,
    context: Dom<WebGLRenderingContext>,
    session: Dom<XRSession>,
    framebuffer: Dom<WebGLFramebuffer>,
}

impl XRWebGLLayer {
    pub fn new_inherited(
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: &WebGLFramebuffer,
    ) -> XRWebGLLayer {
        XRWebGLLayer {
            xrlayer: XRLayer::new_inherited(),
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            alpha: init.alpha,
            context: Dom::from_ref(context),
            session: Dom::from_ref(session),
            framebuffer: Dom::from_ref(framebuffer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: &WebGLFramebuffer,
    ) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited(
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
    pub fn Constructor(
        global: &Window,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
    ) -> Fallible<DomRoot<Self>> {
        let cx = global.get_cx();
        let old_fbo = context.bound_framebuffer();
        let old_texture = context
            .textures()
            .active_texture_for_image_target(TexImageTarget::Texture2D);

        // Step 8.2. "Initialize layer’s framebuffer to a new opaque framebuffer created with context."
        let framebuffer = context.CreateFramebuffer().ok_or(Error::Operation)?;

        // Step 8.3. "Allocate and initialize resources compatible with session’s XR device,
        // including GPU accessible memory buffers, as required to support the compositing of layer."

        // Create a new texture with size given by the session's recommended resolution
        let texture = context.CreateTexture().ok_or(Error::Operation)?;
        let resolution = session.with_session(|s| s.recommended_framebuffer_resolution());
        let mut pixels = CustomAutoRooter::new(None);
        context.BindTexture(constants::TEXTURE_2D, Some(&texture));
        let sc = context.TexImage2D(
            constants::TEXTURE_2D,
            0,
            constants::RGBA,
            resolution.width,
            resolution.height,
            0,
            constants::RGBA,
            constants::UNSIGNED_BYTE,
            pixels.root(cx),
        );

        // Bind the new texture to the framebuffer
        context.BindFramebuffer(constants::FRAMEBUFFER, Some(&framebuffer));
        context.FramebufferTexture2D(
            constants::FRAMEBUFFER,
            constants::COLOR_ATTACHMENT0,
            constants::TEXTURE_2D,
            Some(&texture),
            0,
        );

        // Restore the WebGL state while complaining about global mutable state
        context.BindTexture(constants::TEXTURE_2D, old_texture.as_ref().map(|t| &**t));
        context.BindFramebuffer(constants::FRAMEBUFFER, old_fbo.as_ref().map(|f| &**f));

        // Step 8.4: "If layer’s resources were unable to be created for any reason,
        // throw an OperationError and abort these steps."
        sc.or(Err(Error::Operation))?;

        // Step 9. "Return layer."
        Ok(XRWebGLLayer::new(
            &global.global(),
            session,
            context,
            init,
            &framebuffer,
        ))
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
    fn Framebuffer(&self) -> DomRoot<WebGLFramebuffer> {
        DomRoot::from_ref(&self.framebuffer)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferwidth
    fn FramebufferWidth(&self) -> u32 {
        self.framebuffer
            .size()
            .unwrap_or((0, 0))
            .0
            .try_into()
            .unwrap_or(0)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferheight
    fn FramebufferHeight(&self) -> u32 {
        self.framebuffer
            .size()
            .unwrap_or((0, 0))
            .1
            .try_into()
            .unwrap_or(0)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport
    fn GetViewport(&self, view: &XRView) -> Option<DomRoot<XRViewport>> {
        if self.session != view.session() {
            return None;
        }

        let views = self.session.with_session(|s| s.views().clone());

        let viewport = match (view.Eye(), views) {
            (XREye::None, Views::Mono(view)) => view.viewport,
            (XREye::Left, Views::Stereo(view, _)) => view.viewport,
            (XREye::Right, Views::Stereo(_, view)) => view.viewport,
            // The spec doesn't really say what to do in this case
            // https://github.com/immersive-web/webxr/issues/769
            _ => return None,
        };

        Some(XRViewport::new(&self.global(), viewport))
    }
}
