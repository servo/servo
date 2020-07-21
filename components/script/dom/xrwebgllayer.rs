/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextBinding::WebGL2RenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerInit;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLRenderingContext;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use canvas_traits::webgl::WebGLContextId;
use canvas_traits::webgl::WebGLCommand;
use canvas_traits::webgl::WebGLTextureId;
use dom_struct::dom_struct;
use euclid::{Rect, Size2D};
use std::convert::TryInto;
use webxr_api::ContextId as WebXRContextId;
use webxr_api::LayerId;
use webxr_api::LayerInit;
use webxr_api::Viewport;

#[derive(JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub enum RenderingContext {
    WebGL1(Dom<WebGLRenderingContext>),
    WebGL2(Dom<WebGL2RenderingContext>),
}

impl RenderingContext {
    fn context_id(&self) -> WebGLContextId {
        match self {
            RenderingContext::WebGL1(ref ctx) => ctx.context_id(),
            RenderingContext::WebGL2(ref ctx) => ctx.base_context().context_id(),
        }
    }
}

impl<'a> From<&'a XRWebGLLayerInit> for LayerInit {
    fn from(init: &'a XRWebGLLayerInit) -> LayerInit {
        LayerInit::WebGLLayer {
            alpha: init.alpha,
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            framebuffer_scale_factor: *init.framebufferScaleFactor as f32,
            ignore_depth_values: init.ignoreDepthValues,
        }
    }
}

#[dom_struct]
pub struct XRWebGLLayer {
    reflector_: Reflector,
    antialias: bool,
    depth: bool,
    stencil: bool,
    alpha: bool,
    ignore_depth_values: bool,
    context: RenderingContext,
    session: Dom<XRSession>,
    /// If none, this is an inline session (the composition disabled flag is true)
    framebuffer: Option<Dom<WebGLFramebuffer>>,
    /// If none, this is an inline session (the composition disabled flag is true)
    #[ignore_malloc_size_of = "Layer ids don't heap-allocate"]
    layer_id: Option<LayerId>,
}

impl XRWebGLLayer {
    pub fn new_inherited(
        session: &XRSession,
        context: XRWebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
        layer_id: Option<LayerId>,
    ) -> XRWebGLLayer {
        XRWebGLLayer {
            reflector_: Reflector::new(),
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            alpha: init.alpha,
            ignore_depth_values: init.ignoreDepthValues,
            layer_id,
            context: match context {
                XRWebGLRenderingContext::WebGLRenderingContext(ctx) => {
                    RenderingContext::WebGL1(Dom::from_ref(&*ctx))
                },
                XRWebGLRenderingContext::WebGL2RenderingContext(ctx) => {
                    RenderingContext::WebGL2(Dom::from_ref(&*ctx))
                },
            },
            session: Dom::from_ref(session),
            framebuffer: framebuffer.map(Dom::from_ref),
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        context: XRWebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
        layer_id: Option<LayerId>,
    ) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited(
                session,
                context,
                init,
                framebuffer,
                layer_id,
            )),
            global,
        )
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-xrwebgllayer
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &Window,
        session: &XRSession,
        context: XRWebGLRenderingContext,
        init: &XRWebGLLayerInit,
    ) -> Fallible<DomRoot<Self>> {
        // Step 2
        if session.is_ended() {
            return Err(Error::InvalidState);
        }
        // XXXManishearth step 3: throw error if context is lost
        // XXXManishearth step 4: check XR compat flag for immersive sessions

        let (framebuffer, layer_id) = if session.is_immersive() {
            // Step 9.2. "Initialize layer’s framebuffer to a new opaque framebuffer created with context."
            let size = session
                .with_session(|session| session.recommended_framebuffer_resolution())
                .ok_or(Error::Operation)?;
            let framebuffer = WebGLFramebuffer::maybe_new_webxr(session, &context, size)
                .ok_or(Error::Operation)?;

            // Step 9.3. "Allocate and initialize resources compatible with session’s XR device,
            // including GPU accessible memory buffers, as required to support the compositing of layer."
            let context_id = WebXRContextId::from(context.context_id());
            let layer_init = LayerInit::from(init);
            let layer_id = session
                .with_session(|session| session.create_layer(context_id, layer_init))
                .map_err(|_| Error::Operation)?;

            // Step 9.4: "If layer’s resources were unable to be created for any reason,
            // throw an OperationError and abort these steps."
            (Some(framebuffer), Some(layer_id))
        } else {
            (None, None)
        };

        // Ensure that we finish setting up this layer before continuing.
        match context {
            XRWebGLRenderingContext::WebGLRenderingContext(ref ctx) => ctx.Finish(),
            XRWebGLRenderingContext::WebGL2RenderingContext(ref ctx) => ctx.Finish(),
        }

        // Step 10. "Return layer."
        Ok(XRWebGLLayer::new(
            &global.global(),
            session,
            context,
            init,
            framebuffer.as_deref(),
            layer_id,
        ))
    }

    pub fn layer_id(&self) -> Option<LayerId> {
        self.layer_id
    }

    pub fn context_id(&self) -> WebGLContextId {
        self.context.context_id()
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }

    pub fn size(&self) -> Size2D<u32, Viewport> {
        if let Some(framebuffer) = self.framebuffer.as_ref() {
            let size = framebuffer.size().unwrap_or((0, 0));
            Size2D::new(
                size.0.try_into().unwrap_or(0),
                size.1.try_into().unwrap_or(0),
            )
        } else {
            let size = match self.context {
                RenderingContext::WebGL1(ref ctx) => ctx.Canvas().get_size(),
                RenderingContext::WebGL2(ref ctx) => ctx.base_context().Canvas().get_size(),
            };
            Size2D::from_untyped(size)
        }
    }

    fn texture_target(&self) -> u32 {
        if cfg!(target_os = "macos") {
            sparkle::gl::TEXTURE_RECTANGLE
        } else {
            sparkle::gl::TEXTURE_2D
        }
    }

    pub fn begin_frame(&self, frame: &XRFrame) -> Option<()> {
        debug!("XRWebGLLayer begin frame");
        let framebuffer = self.framebuffer.as_ref()?;
        let context = framebuffer.upcast::<WebGLObject>().context();
        let sub_images = frame.get_sub_images(self.layer_id?)?;
        // TODO: Cache this texture
        let color_texture_id =
            WebGLTextureId::maybe_new(sub_images.sub_image.as_ref()?.color_texture)?;
        let color_texture = WebGLTexture::new(context, color_texture_id);
        let target = self.texture_target();

        // Save the current bindings
        let saved_framebuffer = context.get_draw_framebuffer_slot().get();
        let saved_framebuffer_target = framebuffer.target();
        let saved_texture_id = context
            .textures()
            .active_texture_slot(target, context.webgl_version())
            .ok()
            .and_then(|slot| slot.get().map(|texture| texture.id()));

        // We have to pick a framebuffer target.
        // If there is a draw framebuffer, we use its target,
        // otherwise we just use DRAW_FRAMEBUFFER.
        let framebuffer_target = saved_framebuffer
            .as_ref()
            .and_then(|fb| fb.target())
            .unwrap_or(constants::DRAW_FRAMEBUFFER);

        // Update the attachments
        context.send_command(WebGLCommand::BindTexture(target, Some(color_texture_id)));
        framebuffer.bind(framebuffer_target);
        framebuffer
            .texture2d_even_if_opaque(
                constants::COLOR_ATTACHMENT0,
                self.texture_target(),
                Some(&color_texture),
                0,
            )
            .ok()?;
        if let Some(id) = sub_images.sub_image.as_ref()?.depth_stencil_texture {
            // TODO: Cache this texture
            let depth_stencil_texture_id = WebGLTextureId::maybe_new(id)?;
            let depth_stencil_texture = WebGLTexture::new(context, depth_stencil_texture_id);
            framebuffer
                .texture2d_even_if_opaque(
                    constants::DEPTH_STENCIL_ATTACHMENT,
                    constants::TEXTURE_2D,
                    Some(&depth_stencil_texture),
                    0,
                )
                .ok()?;
        }

        // Restore the old bindings
        context.send_command(WebGLCommand::BindTexture(target, saved_texture_id));
        if let Some(framebuffer_target) = saved_framebuffer_target {
            framebuffer.bind(framebuffer_target);
        }
        if let Some(framebuffer) = saved_framebuffer {
            framebuffer.bind(framebuffer_target);
        }
        Some(())
    }

    pub fn end_frame(&self, _frame: &XRFrame) -> Option<()> {
        debug!("XRWebGLLayer end frame");
        // TODO: invalidate the old texture
        let framebuffer = self.framebuffer.as_ref()?;
        // TODO: rebind the current bindings
        framebuffer.bind(constants::FRAMEBUFFER);
        framebuffer
            .texture2d_even_if_opaque(constants::COLOR_ATTACHMENT0, self.texture_target(), None, 0)
            .ok()?;
        framebuffer
            .texture2d_even_if_opaque(
                constants::DEPTH_STENCIL_ATTACHMENT,
                constants::DEPTH_STENCIL_ATTACHMENT,
                None,
                0,
            )
            .ok()?;
        framebuffer.upcast::<WebGLObject>().context().Flush();
        Some(())
    }

    pub(crate) fn context(&self) -> XRWebGLRenderingContext {
        match self.context {
            RenderingContext::WebGL1(ref ctx) => {
                XRWebGLRenderingContext::WebGLRenderingContext(DomRoot::from_ref(&**ctx))
            },
            RenderingContext::WebGL2(ref ctx) => {
                XRWebGLRenderingContext::WebGL2RenderingContext(DomRoot::from_ref(&**ctx))
            },
        }
    }
}

impl XRWebGLLayerMethods for XRWebGLLayer {
    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-antialias
    fn Antialias(&self) -> bool {
        self.antialias
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-ignoredepthvalues
    fn IgnoreDepthValues(&self) -> bool {
        self.ignore_depth_values
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

        let index = view.viewport_index();

        let viewport = self.session.with_session(|s| {
            // Inline sssions
            if s.viewports().is_empty() {
                Rect::from_size(self.size().to_i32())
            } else {
                s.viewports()[index]
            }
        });

        Some(XRViewport::new(&self.global(), viewport))
    }
}
