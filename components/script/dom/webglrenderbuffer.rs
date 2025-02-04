/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;

use canvas_traits::webgl::{
    webgl_channel, GlType, InternalFormatIntVec, WebGLCommand, WebGLError, WebGLRenderbufferId,
    WebGLResult, WebGLVersion,
};
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::EXTColorBufferHalfFloatBinding::EXTColorBufferHalfFloatConstants;
use crate::dom::bindings::codegen::Bindings::WEBGLColorBufferFloatBinding::WEBGLColorBufferFloatConstants;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WebGLRenderbuffer {
    webgl_object: WebGLObject,
    #[no_trace]
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
            id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            internal_format: Cell::new(None),
            size: Cell::new(None),
            is_initialized: Cell::new(false),
            attached_framebuffer: Default::default(),
        }
    }

    pub(crate) fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateRenderbuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLRenderbuffer::new(context, id))
    }

    pub(crate) fn new(context: &WebGLRenderingContext, id: WebGLRenderbufferId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLRenderbuffer::new_inherited(context, id)),
            &*context.global(),
            CanGc::note(),
        )
    }
}

impl WebGLRenderbuffer {
    pub(crate) fn id(&self) -> WebGLRenderbufferId {
        self.id
    }

    pub(crate) fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    pub(crate) fn internal_format(&self) -> u32 {
        self.internal_format.get().unwrap_or(constants::RGBA4)
    }

    pub(crate) fn mark_initialized(&self) {
        self.is_initialized.set(true);
    }

    pub(crate) fn is_initialized(&self) -> bool {
        self.is_initialized.get()
    }

    pub(crate) fn bind(&self, target: u32) {
        self.ever_bound.set(true);
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindRenderbuffer(target, Some(self.id)));
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);

            let context = self.upcast::<WebGLObject>().context();

            /*
            If a renderbuffer object is deleted while its image is attached to one or more
            attachment points in a currently bound framebuffer object, then it is as if
            FramebufferRenderbuffer had been called, with a renderbuffer of zero, for each
            attachment point to which this image was attached in that framebuffer object.
            In other words,the renderbuffer image is first detached from all attachment points
            in that frame-buffer object.
            - GLES 3.0, 4.4.2.3, "Attaching Renderbuffer Images to a Framebuffer"
            */
            if let Some(fb) = context.get_draw_framebuffer_slot().get() {
                let _ = fb.detach_renderbuffer(self);
            }
            if let Some(fb) = context.get_read_framebuffer_slot().get() {
                let _ = fb.detach_renderbuffer(self);
            }

            let cmd = WebGLCommand::DeleteRenderbuffer(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub(crate) fn ever_bound(&self) -> bool {
        self.ever_bound.get()
    }

    pub(crate) fn storage(
        &self,
        api_type: GlType,
        sample_count: i32,
        internal_format: u32,
        width: i32,
        height: i32,
    ) -> WebGLResult<()> {
        let is_gles = api_type == GlType::Gles;
        let webgl_version = self.upcast().context().webgl_version();

        // Validate the internal_format, and save it for completeness
        // validation.
        let actual_format = match internal_format {
            constants::RGBA4 | constants::DEPTH_COMPONENT16 | constants::STENCIL_INDEX8 => {
                internal_format
            },
            constants::R8 |
            constants::R8UI |
            constants::R8I |
            constants::R16UI |
            constants::R16I |
            constants::R32UI |
            constants::R32I |
            constants::RG8 |
            constants::RG8UI |
            constants::RG8I |
            constants::RG16UI |
            constants::RG16I |
            constants::RG32UI |
            constants::RG32I |
            constants::RGB8 |
            constants::RGBA8 |
            constants::SRGB8_ALPHA8 |
            constants::RGB10_A2 |
            constants::RGBA8UI |
            constants::RGBA8I |
            constants::RGB10_A2UI |
            constants::RGBA16UI |
            constants::RGBA16I |
            constants::RGBA32I |
            constants::RGBA32UI |
            constants::DEPTH_COMPONENT24 |
            constants::DEPTH_COMPONENT32F |
            constants::DEPTH24_STENCIL8 |
            constants::DEPTH32F_STENCIL8 => match webgl_version {
                WebGLVersion::WebGL1 => return Err(WebGLError::InvalidEnum),
                _ => internal_format,
            },
            // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.8
            constants::DEPTH_STENCIL => constants::DEPTH24_STENCIL8,
            constants::RGB5_A1 => {
                // 16-bit RGBA formats are not supported on desktop GL.
                if is_gles {
                    constants::RGB5_A1
                } else {
                    constants::RGBA8
                }
            },
            constants::RGB565 => {
                // RGB565 is not supported on desktop GL.
                if is_gles {
                    constants::RGB565
                } else {
                    constants::RGB8
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

        if webgl_version != WebGLVersion::WebGL1 {
            let (sender, receiver) = webgl_channel().unwrap();
            self.upcast::<WebGLObject>().context().send_command(
                WebGLCommand::GetInternalFormatIntVec(
                    constants::RENDERBUFFER,
                    internal_format,
                    InternalFormatIntVec::Samples,
                    sender,
                ),
            );
            let samples = receiver.recv().unwrap();
            if sample_count < 0 || sample_count > samples.first().cloned().unwrap_or(0) {
                return Err(WebGLError::InvalidOperation);
            }
        }

        self.internal_format.set(Some(internal_format));
        self.is_initialized.set(false);

        if let Some(fb) = self.attached_framebuffer.get() {
            fb.update_status();
        }

        let command = match sample_count {
            0 => WebGLCommand::RenderbufferStorage(
                constants::RENDERBUFFER,
                actual_format,
                width,
                height,
            ),
            _ => WebGLCommand::RenderbufferStorageMultisample(
                constants::RENDERBUFFER,
                sample_count,
                actual_format,
                width,
                height,
            ),
        };
        self.upcast::<WebGLObject>().context().send_command(command);

        self.size.set(Some((width, height)));
        Ok(())
    }

    pub(crate) fn attach_to_framebuffer(&self, fb: &WebGLFramebuffer) {
        self.attached_framebuffer.set(Some(fb));
    }

    pub(crate) fn detach_from_framebuffer(&self) {
        self.attached_framebuffer.set(None);
    }
}

impl Drop for WebGLRenderbuffer {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}
