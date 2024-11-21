/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;

use canvas_traits::webgl::{WebGLCommand, WebGLContextId, WebGLTextureId};
use dom_struct::dom_struct;
use euclid::{Rect, Size2D};
use js::rust::HandleObject;
use webxr_api::{ContextId as WebXRContextId, LayerId, LayerInit, Viewport};

use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::{
    XRWebGLLayerInit, XRWebGLLayerMethods, XRWebGLRenderingContext,
};
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use crate::script_runtime::CanGc;

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
pub(crate) struct XRWebGLLayer {
    xr_layer: XRLayer,
    antialias: bool,
    depth: bool,
    stencil: bool,
    alpha: bool,
    ignore_depth_values: bool,
    /// If none, this is an inline session (the composition disabled flag is true)
    framebuffer: Option<Dom<WebGLFramebuffer>>,
}

impl XRWebGLLayer {
    pub(crate) fn new_inherited(
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
        layer_id: Option<LayerId>,
    ) -> XRWebGLLayer {
        XRWebGLLayer {
            xr_layer: XRLayer::new_inherited(session, context, layer_id),
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            alpha: init.alpha,
            ignore_depth_values: init.ignoreDepthValues,
            framebuffer: framebuffer.map(Dom::from_ref),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: Option<&WebGLFramebuffer>,
        layer_id: Option<LayerId>,
        can_gc: CanGc,
    ) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object_with_proto(
            Box::new(XRWebGLLayer::new_inherited(
                session,
                context,
                init,
                framebuffer,
                layer_id,
            )),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn layer_id(&self) -> Option<LayerId> {
        self.xr_layer.layer_id()
    }

    pub(crate) fn context_id(&self) -> WebGLContextId {
        self.xr_layer.context_id()
    }

    pub(crate) fn session(&self) -> &XRSession {
        self.xr_layer.session()
    }

    pub(crate) fn size(&self) -> Size2D<u32, Viewport> {
        if let Some(framebuffer) = self.framebuffer.as_ref() {
            let size = framebuffer.size().unwrap_or((0, 0));
            Size2D::new(
                size.0.try_into().unwrap_or(0),
                size.1.try_into().unwrap_or(0),
            )
        } else {
            let size = match self.context().Canvas() {
                HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => canvas.get_size(),
                HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => {
                    let size = canvas.get_size();
                    Size2D::new(
                        size.width.try_into().unwrap_or(0),
                        size.height.try_into().unwrap_or(0),
                    )
                },
            };
            Size2D::from_untyped(size)
        }
    }

    fn texture_target(&self) -> u32 {
        if cfg!(target_os = "macos") {
            glow::TEXTURE_RECTANGLE
        } else {
            glow::TEXTURE_2D
        }
    }

    pub(crate) fn begin_frame(&self, frame: &XRFrame) -> Option<()> {
        debug!("XRWebGLLayer begin frame");
        let framebuffer = self.framebuffer.as_ref()?;
        let context = framebuffer.upcast::<WebGLObject>().context();
        let sub_images = frame.get_sub_images(self.layer_id()?)?;
        let session = self.session();
        // TODO: Cache this texture
        let color_texture_id = WebGLTextureId::new(sub_images.sub_image.as_ref()?.color_texture?);
        let color_texture = WebGLTexture::new_webxr(context, color_texture_id, session);
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
            let depth_stencil_texture_id = WebGLTextureId::new(id);
            let depth_stencil_texture =
                WebGLTexture::new_webxr(context, depth_stencil_texture_id, session);
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

    pub(crate) fn end_frame(&self, _frame: &XRFrame) -> Option<()> {
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

    pub(crate) fn context(&self) -> &WebGLRenderingContext {
        self.xr_layer.context()
    }
}

impl XRWebGLLayerMethods<crate::DomTypeHolder> for XRWebGLLayer {
    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-xrwebgllayer>
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        session: &XRSession,
        context: XRWebGLRenderingContext,
        init: &XRWebGLLayerInit,
    ) -> Fallible<DomRoot<Self>> {
        let context = match context {
            XRWebGLRenderingContext::WebGLRenderingContext(ctx) => ctx,
            XRWebGLRenderingContext::WebGL2RenderingContext(ctx) => ctx.base_context(),
        };

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
        context.Finish();

        // Step 10. "Return layer."
        Ok(XRWebGLLayer::new(
            &global.global(),
            proto,
            session,
            &context,
            init,
            framebuffer.as_deref(),
            layer_id,
            can_gc,
        ))
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrwebgllayer-getnativeframebufferscalefactor>
    fn GetNativeFramebufferScaleFactor(_window: &Window, session: &XRSession) -> Finite<f64> {
        let value: f64 = if session.is_ended() { 0.0 } else { 1.0 };
        Finite::wrap(value)
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-antialias>
    fn Antialias(&self) -> bool {
        self.antialias
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-ignoredepthvalues>
    fn IgnoreDepthValues(&self) -> bool {
        self.ignore_depth_values
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrwebgllayer-fixedfoveation>
    fn GetFixedFoveation(&self) -> Option<Finite<f32>> {
        // Fixed foveation is only available on Quest/Pico headset runtimes
        None
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrwebgllayer-fixedfoveation>
    fn SetFixedFoveation(&self, _value: Option<Finite<f32>>) {
        // no-op until fixed foveation is supported
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebuffer>
    fn GetFramebuffer(&self) -> Option<DomRoot<WebGLFramebuffer>> {
        self.framebuffer.as_ref().map(|x| DomRoot::from_ref(&**x))
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferwidth>
    fn FramebufferWidth(&self) -> u32 {
        self.size().width
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferheight>
    fn FramebufferHeight(&self) -> u32 {
        self.size().height
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport>
    fn GetViewport(&self, view: &XRView) -> Option<DomRoot<XRViewport>> {
        if self.session() != view.session() {
            return None;
        }

        let index = view.viewport_index();

        let viewport = self.session().with_session(|s| {
            // Inline sessions
            if s.viewports().is_empty() {
                Rect::from_size(self.size().to_i32())
            } else {
                s.viewports()[index]
            }
        });

        // NOTE: According to spec, viewport sizes should be recalculated here if the
        // requested viewport scale has changed. However, existing browser implementations
        // don't seem to do this for stereoscopic immersive sessions.
        // Revisit if Servo gets support for handheld AR/VR via ARCore/ARKit

        Some(XRViewport::new(&self.global(), viewport))
    }
}
