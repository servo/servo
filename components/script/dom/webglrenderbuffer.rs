/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use crate::dom::bindings::codegen::Bindings::EXTColorBufferHalfFloatBinding::EXTColorBufferHalfFloatConstants;
use crate::dom::bindings::codegen::Bindings::WEBGLColorBufferFloatBinding::WEBGLColorBufferFloatConstants;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants;
use crate::dom::bindings::codegen::Bindings::WebGLRenderbufferBinding;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use canvas_traits::webgl::{
    webgl_channel, GlType, WebGLCommand, WebGLError, WebGLRenderbufferId, WebGLResult,
};
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WebGLRenderbuffer {
    webgl_object: WebGLObject,
    id: WebGLRenderbufferId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    size: Cell<Option<(i32, i32)>>,
    internal_format: Cell<Option<u32>>,
    is_initialized: Cell<bool>,
    attached_framebuffer: MutNullableDom<WebGLFramebuffer>,
}

impl WebGLRenderbuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLRenderbufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            internal_format: Cell::new(None),
            size: Cell::new(None),
            is_initialized: Cell::new(false),
            attached_framebuffer: Default::default(),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateRenderbuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLRenderbuffer::new(context, id))
    }

    pub fn new(context: &WebGLRenderingContext, id: WebGLRenderbufferId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLRenderbuffer::new_inherited(context, id)),
            &*context.global(),
            WebGLRenderbufferBinding::Wrap,
        )
    }
}

impl WebGLRenderbuffer {
    pub fn id(&self) -> WebGLRenderbufferId {
        self.id
    }

    pub fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    pub fn internal_format(&self) -> u32 {
        self.internal_format.get().unwrap_or(constants::RGBA4)
    }

    pub fn mark_initialized(&self) {
        self.is_initialized.set(true);
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.get()
    }

    pub fn bind(&self, target: u32) {
        self.ever_bound.set(true);
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindRenderbuffer(target, Some(self.id)));
    }

    pub fn delete(&self, fallible: bool) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);

            /*
            If a renderbuffer object is deleted while its image is attached to the currently
            bound framebuffer, then it is as if FramebufferRenderbuffer had been called, with
            a renderbuffer of 0, for each attachment point to which this image was attached
            in the currently bound framebuffer.
            - GLES 2.0, 4.4.3, "Attaching Renderbuffer Images to a Framebuffer"
             */
            let currently_bound_framebuffer =
                self.upcast::<WebGLObject>().context().bound_framebuffer();
            if let Some(fb) = currently_bound_framebuffer {
                let _ = fb.detach_renderbuffer(self);
            }

            let context = self.upcast::<WebGLObject>().context();
            let cmd = WebGLCommand::DeleteRenderbuffer(self.id);
            if fallible {
                context.send_command_ignored(cmd);
            } else {
                context.send_command(cmd);
            }
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn ever_bound(&self) -> bool {
        self.ever_bound.get()
    }

    pub fn storage(
        &self,
        api_type: GlType,
        internal_format: u32,
        width: i32,
        height: i32,
    ) -> WebGLResult<()> {
        let is_gles = api_type == GlType::Gles;

        // Validate the internal_format, and save it for completeness
        // validation.
        let actual_format = match internal_format {
            constants::RGBA4 | constants::DEPTH_COMPONENT16 | constants::STENCIL_INDEX8 => {
                internal_format
            },
            // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.8
            constants::DEPTH_STENCIL => WebGL2RenderingContextConstants::DEPTH24_STENCIL8,
            constants::RGB5_A1 => {
                // 16-bit RGBA formats are not supported on desktop GL.
                if is_gles {
                    constants::RGB5_A1
                } else {
                    WebGL2RenderingContextConstants::RGBA8
                }
            },
            constants::RGB565 => {
                // RGB565 is not supported on desktop GL.
                if is_gles {
                    constants::RGB565
                } else {
                    WebGL2RenderingContextConstants::RGB8
                }
            },
            EXTColorBufferHalfFloatConstants::RGBA16F_EXT |
            EXTColorBufferHalfFloatConstants::RGB16F_EXT => {
                if !self
                    .upcast()
                    .context()
                    .extension_manager()
                    .is_half_float_buffer_renderable()
                {
                    return Err(WebGLError::InvalidEnum);
                }
                internal_format
            },
            WEBGLColorBufferFloatConstants::RGBA32F_EXT => {
                if !self
                    .upcast()
                    .context()
                    .extension_manager()
                    .is_float_buffer_renderable()
                {
                    return Err(WebGLError::InvalidEnum);
                }
                internal_format
            },
            _ => return Err(WebGLError::InvalidEnum),
        };

        self.internal_format.set(Some(internal_format));
        self.is_initialized.set(false);

        if let Some(fb) = self.attached_framebuffer.get() {
            fb.update_status();
        }

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::RenderbufferStorage(
                constants::RENDERBUFFER,
                actual_format,
                width,
                height,
            ));

        self.size.set(Some((width, height)));

        Ok(())
    }

    pub fn attach_to_framebuffer(&self, fb: &WebGLFramebuffer) {
        self.attached_framebuffer.set(Some(fb));
    }

    pub fn detach_from_framebuffer(&self) {
        self.attached_framebuffer.set(None);
    }
}

impl Drop for WebGLRenderbuffer {
    fn drop(&mut self) {
        self.delete(true);
    }
}
