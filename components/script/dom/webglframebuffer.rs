/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unused_imports)]

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use std::cell::Cell;

use canvas_traits::webgl::{
    webgl_channel, WebGLCommand, WebGLError, WebGLFramebufferBindingRequest, WebGLFramebufferId,
    WebGLRenderbufferId, WebGLResult, WebGLTextureId, WebGLVersion,
};
use dom_struct::dom_struct;
use euclid::Size2D;
#[cfg(feature = "webxr")]
use webxr_api::Viewport;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglrenderingcontext::{Operation, WebGLRenderingContext};
use crate::dom::webgltexture::WebGLTexture;
#[cfg(feature = "webxr")]
use crate::dom::xrsession::XRSession;
use crate::script_runtime::CanGc;

pub(crate) enum CompleteForRendering {
    Complete,
    Incomplete,
    MissingColorAttachment,
}

fn log2(n: u32) -> u32 {
    31 - n.leading_zeros()
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
enum WebGLFramebufferAttachment {
    Renderbuffer(Dom<WebGLRenderbuffer>),
    Texture {
        texture: Dom<WebGLTexture>,
        level: i32,
    },
}

impl WebGLFramebufferAttachment {
    fn needs_initialization(&self) -> bool {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref r) => !r.is_initialized(),
            WebGLFramebufferAttachment::Texture { .. } => false,
        }
    }

    fn mark_initialized(&self) {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref r) => r.mark_initialized(),
            WebGLFramebufferAttachment::Texture { .. } => (),
        }
    }

    fn root(&self) -> WebGLFramebufferAttachmentRoot {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref rb) => {
                WebGLFramebufferAttachmentRoot::Renderbuffer(DomRoot::from_ref(rb))
            },
            WebGLFramebufferAttachment::Texture { ref texture, .. } => {
                WebGLFramebufferAttachmentRoot::Texture(DomRoot::from_ref(texture))
            },
        }
    }

    fn detach(&self) {
        match self {
            WebGLFramebufferAttachment::Renderbuffer(rb) => rb.detach_from_framebuffer(),
            WebGLFramebufferAttachment::Texture { ref texture, .. } => {
                texture.detach_from_framebuffer()
            },
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum WebGLFramebufferAttachmentRoot {
    Renderbuffer(DomRoot<WebGLRenderbuffer>),
    Texture(DomRoot<WebGLTexture>),
}

#[dom_struct]
pub(crate) struct WebGLFramebuffer {
    webgl_object: WebGLObject,
    #[no_trace]
    webgl_version: WebGLVersion,
    #[no_trace]
    id: WebGLFramebufferId,
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    size: Cell<Option<(i32, i32)>>,
    status: Cell<u32>,
    // The attachment points for textures and renderbuffers on this
    // FBO.
    colors: Vec<DomRefCell<Option<WebGLFramebufferAttachment>>>,
    depth: DomRefCell<Option<WebGLFramebufferAttachment>>,
    stencil: DomRefCell<Option<WebGLFramebufferAttachment>>,
    depthstencil: DomRefCell<Option<WebGLFramebufferAttachment>>,
    color_read_buffer: DomRefCell<u32>,
    color_draw_buffers: DomRefCell<Vec<u32>>,
    is_initialized: Cell<bool>,
    // Framebuffers for XR keep a reference to the XR session.
    // https://github.com/immersive-web/webxr/issues/856
    #[cfg(feature = "webxr")]
    xr_session: MutNullableDom<XRSession>,
}

impl WebGLFramebuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLFramebufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            webgl_version: context.webgl_version(),
            id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            size: Cell::new(None),
            status: Cell::new(constants::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT),
            colors: vec![DomRefCell::new(None); context.limits().max_color_attachments as usize],
            depth: DomRefCell::new(None),
            stencil: DomRefCell::new(None),
            depthstencil: DomRefCell::new(None),
            color_read_buffer: DomRefCell::new(constants::COLOR_ATTACHMENT0),
            color_draw_buffers: DomRefCell::new(vec![constants::COLOR_ATTACHMENT0]),
            is_initialized: Cell::new(false),
            #[cfg(feature = "webxr")]
            xr_session: Default::default(),
        }
    }

    pub(crate) fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateFramebuffer(sender));
        let id = receiver.recv().unwrap()?;
        let framebuffer = WebGLFramebuffer::new(context, id, CanGc::note());
        Some(framebuffer)
    }

    // TODO: depth, stencil and alpha
    // https://github.com/servo/servo/issues/24498
    #[cfg(feature = "webxr")]
    pub(crate) fn maybe_new_webxr(
        session: &XRSession,
        context: &WebGLRenderingContext,
        size: Size2D<i32, Viewport>,
    ) -> Option<DomRoot<Self>> {
        let framebuffer = Self::maybe_new(context)?;
        framebuffer.size.set(Some((size.width, size.height)));
        framebuffer.status.set(constants::FRAMEBUFFER_COMPLETE);
        framebuffer.xr_session.set(Some(session));
        Some(framebuffer)
    }

    pub(crate) fn new(
        context: &WebGLRenderingContext,
        id: WebGLFramebufferId,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLFramebuffer::new_inherited(context, id)),
            &*context.global(),
            can_gc,
        )
    }
}

impl WebGLFramebuffer {
    pub(crate) fn id(&self) -> WebGLFramebufferId {
        self.id
    }

    #[cfg(feature = "webxr")]
    fn is_in_xr_session(&self) -> bool {
        self.xr_session.get().is_some()
    }

    #[cfg(not(feature = "webxr"))]
    fn is_in_xr_session(&self) -> bool {
        false
    }

    pub(crate) fn validate_transparent(&self) -> WebGLResult<()> {
        if self.is_in_xr_session() {
            Err(WebGLError::InvalidOperation)
        } else {
            Ok(())
        }
    }

    pub(crate) fn bind(&self, target: u32) {
        if !self.is_in_xr_session() {
            // Update the framebuffer status on binding.  It may have
            // changed if its attachments were resized or deleted while
            // we've been unbound.
            self.update_status();
        }

        self.target.set(Some(target));
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindFramebuffer(
                target,
                WebGLFramebufferBindingRequest::Explicit(self.id),
            ));
    }

    pub(crate) fn delete(&self, operation_fallibility: Operation) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let context = self.upcast::<WebGLObject>().context();
            let cmd = WebGLCommand::DeleteFramebuffer(self.id);
            match operation_fallibility {
                Operation::Fallible => context.send_command_ignored(cmd),
                Operation::Infallible => context.send_command(cmd),
            }
        }
    }

    pub(crate) fn is_deleted(&self) -> bool {
        // TODO: if a framebuffer has an attachment which is invalid due to
        // being outside a webxr rAF, should this make the framebuffer invalid?
        // https://github.com/immersive-web/layers/issues/196
        self.is_deleted.get()
    }

    pub(crate) fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    pub(crate) fn get_attachment_formats(
        &self,
    ) -> WebGLResult<(Option<u32>, Option<u32>, Option<u32>)> {
        if self.check_status() != constants::FRAMEBUFFER_COMPLETE {
            return Err(WebGLError::InvalidFramebufferOperation);
        }
        let color = match self.attachment(constants::COLOR_ATTACHMENT0) {
            Some(WebGLFramebufferAttachmentRoot::Renderbuffer(rb)) => Some(rb.internal_format()),
            _ => None,
        };
        let depth = match self.attachment(constants::DEPTH_ATTACHMENT) {
            Some(WebGLFramebufferAttachmentRoot::Renderbuffer(rb)) => Some(rb.internal_format()),
            _ => None,
        };
        let stencil = match self.attachment(constants::STENCIL_ATTACHMENT) {
            Some(WebGLFramebufferAttachmentRoot::Renderbuffer(rb)) => Some(rb.internal_format()),
            _ => None,
        };
        Ok((color, depth, stencil))
    }

    fn check_attachment_constraints<'a>(
        &self,
        attachment: &Option<WebGLFramebufferAttachment>,
        mut constraints: impl Iterator<Item = &'a u32>,
        fb_size: &mut Option<(i32, i32)>,
    ) -> Result<(), u32> {
        // Get the size of this attachment.
        let (format, size) = match attachment {
            Some(WebGLFramebufferAttachment::Renderbuffer(ref att_rb)) => {
                (Some(att_rb.internal_format()), att_rb.size())
            },
            Some(WebGLFramebufferAttachment::Texture {
                texture: ref att_tex,
                level,
            }) => match att_tex.image_info_at_face(0, *level as u32) {
                Some(info) => (
                    Some(info.internal_format().as_gl_constant()),
                    Some((info.width() as i32, info.height() as i32)),
                ),
                None => return Err(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT),
            },
            None => (None, None),
        };

        // Make sure that, if we've found any other attachment,
        // that the size matches.
        if size.is_some() {
            if fb_size.is_some() && size != *fb_size {
                return Err(constants::FRAMEBUFFER_INCOMPLETE_DIMENSIONS);
            } else {
                *fb_size = size;
            }
        }

        if let Some(format) = format {
            if constraints.all(|c| *c != format) {
                return Err(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT);
            }
        }

        Ok(())
    }

    pub(crate) fn update_status(&self) {
        let z = self.depth.borrow();
        let s = self.stencil.borrow();
        let zs = self.depthstencil.borrow();
        let has_z = z.is_some();
        let has_s = s.is_some();
        let has_zs = zs.is_some();

        let is_supported = match self.webgl_version {
            // From the WebGL 1.0 spec, 6.6 ("Framebuffer Object Attachments"):
            //
            //    "In the WebGL API, it is an error to concurrently attach
            //     renderbuffers to the following combinations of
            //     attachment points:
            //
            //     DEPTH_ATTACHMENT + DEPTH_STENCIL_ATTACHMENT
            //     STENCIL_ATTACHMENT + DEPTH_STENCIL_ATTACHMENT
            //     DEPTH_ATTACHMENT + STENCIL_ATTACHMENT
            //
            //     If any of the constraints above are violated, then:
            //
            //     checkFramebufferStatus must return FRAMEBUFFER_UNSUPPORTED."
            WebGLVersion::WebGL1 => !(has_zs && (has_z || has_s)) && !(has_z && has_s),

            // In WebGL 2.0, DEPTH_STENCIL_ATTACHMENT is considered an alias for
            // DEPTH_ATTACHMENT + STENCIL_ATTACHMENT, i.e., the same image is attached to both DEPTH_ATTACHMENT
            // and STENCIL_ATTACHMENT, overwriting the original images attached to the two attachment points.
            // If different images are bound to the depth and stencil attachment points, checkFramebufferStatus
            // returns FRAMEBUFFER_UNSUPPORTED, and getFramebufferAttachmentParameter with attachment of
            // DEPTH_STENCIL_ATTACHMENT generates an INVALID_OPERATION error.
            // -- WebGL 2.0 spec, 4.1.5 Framebuffer Object Attachments
            WebGLVersion::WebGL2 => {
                use WebGLFramebufferAttachment::{Renderbuffer, Texture};
                match (&*z, &*s) {
                    (Some(Renderbuffer(a)), Some(Renderbuffer(b))) => a.id() == b.id(),
                    (Some(Texture { texture: a, .. }), Some(Texture { texture: b, .. })) => {
                        a.id() == b.id()
                    },
                    _ => !has_z || !has_s,
                }
            },
        };
        if !is_supported {
            return self.status.set(constants::FRAMEBUFFER_UNSUPPORTED);
        }

        let mut fb_size = None;

        let attachments = [&*z, &*s, &*zs];
        let webgl1_attachment_constraints = &[
            &[
                constants::DEPTH_COMPONENT16,
                constants::DEPTH_COMPONENT24,
                constants::DEPTH_COMPONENT32F,
                constants::DEPTH24_STENCIL8,
                constants::DEPTH32F_STENCIL8,
            ][..],
            &[
                constants::STENCIL_INDEX8,
                constants::DEPTH24_STENCIL8,
                constants::DEPTH32F_STENCIL8,
            ][..],
            &[constants::DEPTH_STENCIL][..],
        ];
        let webgl2_attachment_constraints = &[
            &[constants::DEPTH_STENCIL][..],
            &[constants::DEPTH_STENCIL][..],
            &[][..],
        ];
        let empty_attachment_constrains = &[&[][..], &[][..], &[][..]];
        let extra_attachment_constraints = match self.webgl_version {
            WebGLVersion::WebGL1 => empty_attachment_constrains,
            WebGLVersion::WebGL2 => webgl2_attachment_constraints,
        };
        let attachment_constraints = webgl1_attachment_constraints
            .iter()
            .zip(extra_attachment_constraints.iter())
            .map(|(a, b)| a.iter().chain(b.iter()));

        for (attachment, constraints) in attachments.iter().zip(attachment_constraints) {
            if let Err(errnum) =
                self.check_attachment_constraints(attachment, constraints, &mut fb_size)
            {
                return self.status.set(errnum);
            }
        }

        let webgl1_color_constraints = &[
            constants::RGB,
            constants::RGB565,
            constants::RGB5_A1,
            constants::RGBA,
            constants::RGBA4,
        ][..];
        let webgl2_color_constraints = &[
            constants::ALPHA,
            constants::LUMINANCE,
            constants::LUMINANCE_ALPHA,
            constants::R11F_G11F_B10F,
            constants::R16F,
            constants::R16I,
            constants::R16UI,
            constants::R32F,
            constants::R32I,
            constants::R32UI,
            constants::R8,
            constants::R8_SNORM,
            constants::R8I,
            constants::R8UI,
            constants::RG16F,
            constants::RG16I,
            constants::RG16UI,
            constants::RG32F,
            constants::RG32I,
            constants::RG32UI,
            constants::RG8,
            constants::RG8_SNORM,
            constants::RG8I,
            constants::RG8UI,
            constants::RGB10_A2,
            constants::RGB10_A2UI,
            constants::RGB16F,
            constants::RGB16I,
            constants::RGB16UI,
            constants::RGB32F,
            constants::RGB32I,
            constants::RGB32UI,
            constants::RGB8,
            constants::RGB8_SNORM,
            constants::RGB8I,
            constants::RGB8UI,
            constants::RGB9_E5,
            constants::RGBA16F,
            constants::RGBA16I,
            constants::RGBA16UI,
            constants::RGBA32F,
            constants::RGBA32I,
            constants::RGBA32UI,
            constants::RGBA8,
            constants::RGBA8_SNORM,
            constants::RGBA8I,
            constants::RGBA8UI,
            constants::SRGB8,
            constants::SRGB8_ALPHA8,
        ][..];
        let empty_color_constrains = &[][..];
        let extra_color_constraints = match self.webgl_version {
            WebGLVersion::WebGL1 => empty_color_constrains,
            WebGLVersion::WebGL2 => webgl2_color_constraints,
        };
        let color_constraints = webgl1_color_constraints
            .iter()
            .chain(extra_color_constraints.iter());

        let has_c = self.colors.iter().any(|att| att.borrow().is_some());
        for attachment in self.colors.iter() {
            let attachment = attachment.borrow();
            let constraints = color_constraints.clone();
            if let Err(errnum) =
                self.check_attachment_constraints(&attachment, constraints, &mut fb_size)
            {
                return self.status.set(errnum);
            }
        }

        self.size.set(fb_size);

        if has_c || has_z || has_zs || has_s {
            if self.size.get().is_some_and(|(w, h)| w != 0 && h != 0) {
                self.status.set(constants::FRAMEBUFFER_COMPLETE);
            } else {
                self.status
                    .set(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT);
            }
        } else {
            self.status
                .set(constants::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT);
        }
    }

    pub(crate) fn check_status(&self) -> u32 {
        // For opaque framebuffers, check to see if the XR session is currently processing an rAF
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        #[cfg(feature = "webxr")]
        if let Some(xr_session) = self.xr_session.get() {
            return if xr_session.is_outside_raf() {
                constants::FRAMEBUFFER_UNSUPPORTED
            } else {
                constants::FRAMEBUFFER_COMPLETE
            };
        }

        self.status.get()
        // TODO: if a framebuffer has an attachment which is invalid due to
        // being outside a webxr rAF, should this make the framebuffer incomplete?
        // https://github.com/immersive-web/layers/issues/196
    }

    pub(crate) fn check_status_for_rendering(&self) -> CompleteForRendering {
        let result = self.check_status();
        if result != constants::FRAMEBUFFER_COMPLETE {
            return CompleteForRendering::Incomplete;
        }

        // XR framebuffers are complete inside an rAF
        // https://github.com/immersive-web/webxr/issues/854
        #[cfg(feature = "webxr")]
        if self.xr_session.get().is_some() {
            return CompleteForRendering::Complete;
        }

        if self.colors.iter().all(|att| att.borrow().is_none()) {
            return CompleteForRendering::MissingColorAttachment;
        }

        if !self.is_initialized.get() {
            let attachments = [
                (&self.depth, constants::DEPTH_BUFFER_BIT),
                (&self.stencil, constants::STENCIL_BUFFER_BIT),
                (
                    &self.depthstencil,
                    constants::DEPTH_BUFFER_BIT | constants::STENCIL_BUFFER_BIT,
                ),
            ];
            let mut clear_bits = 0;
            for &(attachment, bits) in &attachments {
                if let Some(ref att) = *attachment.borrow() {
                    if att.needs_initialization() {
                        att.mark_initialized();
                        clear_bits |= bits;
                    }
                }
            }
            for attachment in self.colors.iter() {
                if let Some(ref att) = *attachment.borrow() {
                    if att.needs_initialization() {
                        att.mark_initialized();
                        clear_bits |= constants::COLOR_BUFFER_BIT;
                    }
                }
            }
            self.upcast::<WebGLObject>()
                .context()
                .initialize_framebuffer(clear_bits);
            self.is_initialized.set(true);
        }

        // TODO: if a framebuffer has an attachment which is invalid due to
        // being outside a webxr rAF, should this make the framebuffer incomplete?
        // https://github.com/immersive-web/layers/issues/196

        CompleteForRendering::Complete
    }

    pub(crate) fn renderbuffer(
        &self,
        attachment: u32,
        rb: Option<&WebGLRenderbuffer>,
    ) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let binding = self
            .attachment_binding(attachment)
            .ok_or(WebGLError::InvalidEnum)?;

        let rb_id = match rb {
            Some(rb) => {
                if !rb.ever_bound() {
                    return Err(WebGLError::InvalidOperation);
                }
                *binding.borrow_mut() =
                    Some(WebGLFramebufferAttachment::Renderbuffer(Dom::from_ref(rb)));
                rb.attach_to_framebuffer(self);
                Some(rb.id())
            },

            _ => None,
        };

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::FramebufferRenderbuffer(
                self.target.get().unwrap(),
                attachment,
                constants::RENDERBUFFER,
                rb_id,
            ));

        if rb.is_none() {
            self.detach_binding(binding, attachment)?;
        }

        self.update_status();
        self.is_initialized.set(false);
        Ok(())
    }

    fn detach_binding(
        &self,
        binding: &DomRefCell<Option<WebGLFramebufferAttachment>>,
        attachment: u32,
    ) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        if let Some(att) = &*binding.borrow() {
            att.detach();
        }
        *binding.borrow_mut() = None;
        if INTERESTING_ATTACHMENT_POINTS.contains(&attachment) {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    fn attachment_binding(
        &self,
        attachment: u32,
    ) -> Option<&DomRefCell<Option<WebGLFramebufferAttachment>>> {
        match attachment {
            constants::COLOR_ATTACHMENT0..=constants::COLOR_ATTACHMENT15 => {
                let idx = attachment - constants::COLOR_ATTACHMENT0;
                self.colors.get(idx as usize)
            },
            constants::DEPTH_ATTACHMENT => Some(&self.depth),
            constants::STENCIL_ATTACHMENT => Some(&self.stencil),
            constants::DEPTH_STENCIL_ATTACHMENT => Some(&self.depthstencil),
            _ => None,
        }
    }

    fn reattach_depth_stencil(&self) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let reattach = |attachment: &WebGLFramebufferAttachment, attachment_point| {
            let context = self.upcast::<WebGLObject>().context();
            match *attachment {
                WebGLFramebufferAttachment::Renderbuffer(ref rb) => {
                    rb.attach_to_framebuffer(self);
                    context.send_command(WebGLCommand::FramebufferRenderbuffer(
                        self.target.get().unwrap(),
                        attachment_point,
                        constants::RENDERBUFFER,
                        Some(rb.id()),
                    ));
                },
                WebGLFramebufferAttachment::Texture { ref texture, level } => {
                    texture.attach_to_framebuffer(self);
                    context.send_command(WebGLCommand::FramebufferTexture2D(
                        self.target.get().unwrap(),
                        attachment_point,
                        texture.target().expect("missing texture target"),
                        Some(texture.id()),
                        level,
                    ));
                },
            }
        };

        // Since the DEPTH_STENCIL attachment causes both the DEPTH and STENCIL
        // attachments to be overwritten, we need to ensure that we reattach
        // the DEPTH and STENCIL attachments when any of those attachments
        // is cleared.
        if let Some(ref depth) = *self.depth.borrow() {
            reattach(depth, constants::DEPTH_ATTACHMENT);
        }
        if let Some(ref stencil) = *self.stencil.borrow() {
            reattach(stencil, constants::STENCIL_ATTACHMENT);
        }
        if let Some(ref depth_stencil) = *self.depthstencil.borrow() {
            reattach(depth_stencil, constants::DEPTH_STENCIL_ATTACHMENT);
        }
        Ok(())
    }

    pub(crate) fn attachment(&self, attachment: u32) -> Option<WebGLFramebufferAttachmentRoot> {
        let binding = self.attachment_binding(attachment)?;
        binding
            .borrow()
            .as_ref()
            .map(WebGLFramebufferAttachment::root)
    }

    pub(crate) fn texture2d(
        &self,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;
        if let Some(texture) = texture {
            //     "If texture is not zero, then texture must either
            //      name an existing texture object with an target of
            //      textarget, or texture must name an existing cube
            //      map texture and textarget must be one of:
            //      TEXTURE_CUBE_MAP_POSITIVE_X,
            //      TEXTURE_CUBE_MAP_POSITIVE_Y,
            //      TEXTURE_CUBE_MAP_POSITIVE_Z,
            //      TEXTURE_CUBE_MAP_NEGATIVE_X,
            //      TEXTURE_CUBE_MAP_NEGATIVE_Y, or
            //      TEXTURE_CUBE_MAP_NEGATIVE_Z. Otherwise,
            //      INVALID_OPERATION is generated."
            let is_cube = match textarget {
                constants::TEXTURE_2D => false,

                constants::TEXTURE_CUBE_MAP_POSITIVE_X => true,
                constants::TEXTURE_CUBE_MAP_POSITIVE_Y => true,
                constants::TEXTURE_CUBE_MAP_POSITIVE_Z => true,
                constants::TEXTURE_CUBE_MAP_NEGATIVE_X => true,
                constants::TEXTURE_CUBE_MAP_NEGATIVE_Y => true,
                constants::TEXTURE_CUBE_MAP_NEGATIVE_Z => true,

                _ => return Err(WebGLError::InvalidEnum),
            };

            match texture.target() {
                Some(constants::TEXTURE_CUBE_MAP) if is_cube => {},
                Some(_) if !is_cube => {},
                _ => return Err(WebGLError::InvalidOperation),
            }

            let context = self.upcast::<WebGLObject>().context();
            let max_tex_size = if is_cube {
                context.limits().max_cube_map_tex_size
            } else {
                context.limits().max_tex_size
            };
            if level < 0 || level as u32 > log2(max_tex_size) {
                return Err(WebGLError::InvalidValue);
            }
        }
        self.texture2d_even_if_opaque(attachment, textarget, texture, level)
    }

    pub(crate) fn texture2d_even_if_opaque(
        &self,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) -> WebGLResult<()> {
        let binding = self
            .attachment_binding(attachment)
            .ok_or(WebGLError::InvalidEnum)?;

        let tex_id = match texture {
            // Note, from the GLES 2.0.25 spec, page 113:
            //      "If texture is zero, then textarget and level are ignored."
            Some(texture) => {
                *binding.borrow_mut() = Some(WebGLFramebufferAttachment::Texture {
                    texture: Dom::from_ref(texture),
                    level,
                });
                texture.attach_to_framebuffer(self);

                Some(texture.id())
            },

            _ => None,
        };

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::FramebufferTexture2D(
                self.target.get().unwrap(),
                attachment,
                textarget,
                tex_id,
                level,
            ));

        if texture.is_none() {
            self.detach_binding(binding, attachment)?;
        }

        self.update_status();
        self.is_initialized.set(false);
        Ok(())
    }

    pub(crate) fn texture_layer(
        &self,
        attachment: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
        layer: i32,
    ) -> WebGLResult<()> {
        let binding = self
            .attachment_binding(attachment)
            .ok_or(WebGLError::InvalidEnum)?;

        let context = self.upcast::<WebGLObject>().context();

        let tex_id = match texture {
            Some(texture) => {
                let (max_level, max_layer) = match texture.target() {
                    Some(constants::TEXTURE_3D) => (
                        log2(context.limits().max_3d_texture_size),
                        context.limits().max_3d_texture_size - 1,
                    ),
                    Some(constants::TEXTURE_2D) => (
                        log2(context.limits().max_tex_size),
                        context.limits().max_array_texture_layers - 1,
                    ),
                    _ => return Err(WebGLError::InvalidOperation),
                };

                if level < 0 || level as u32 >= max_level {
                    return Err(WebGLError::InvalidValue);
                }
                if layer < 0 || layer as u32 >= max_layer {
                    return Err(WebGLError::InvalidValue);
                }

                *binding.borrow_mut() = Some(WebGLFramebufferAttachment::Texture {
                    texture: Dom::from_ref(texture),
                    level,
                });
                texture.attach_to_framebuffer(self);

                Some(texture.id())
            },
            _ => None,
        };

        context.send_command(WebGLCommand::FramebufferTextureLayer(
            self.target.get().unwrap(),
            attachment,
            tex_id,
            level,
            layer,
        ));
        Ok(())
    }

    fn with_matching_renderbuffers<F>(&self, rb: &WebGLRenderbuffer, mut closure: F)
    where
        F: FnMut(&DomRefCell<Option<WebGLFramebufferAttachment>>, u32),
    {
        let rb_id = rb.id();
        let attachments = [
            (&self.depth, constants::DEPTH_ATTACHMENT),
            (&self.stencil, constants::STENCIL_ATTACHMENT),
            (&self.depthstencil, constants::DEPTH_STENCIL_ATTACHMENT),
        ];

        fn has_matching_id(
            attachment: &DomRefCell<Option<WebGLFramebufferAttachment>>,
            target: &WebGLRenderbufferId,
        ) -> bool {
            match *attachment.borrow() {
                Some(WebGLFramebufferAttachment::Renderbuffer(ref att_rb)) => {
                    att_rb.id() == *target
                },
                _ => false,
            }
        }

        for (attachment, name) in &attachments {
            if has_matching_id(attachment, &rb_id) {
                closure(attachment, *name);
            }
        }

        for (idx, attachment) in self.colors.iter().enumerate() {
            if has_matching_id(attachment, &rb_id) {
                let name = constants::COLOR_ATTACHMENT0 + idx as u32;
                closure(attachment, name);
            }
        }
    }

    fn with_matching_textures<F>(&self, texture: &WebGLTexture, mut closure: F)
    where
        F: FnMut(&DomRefCell<Option<WebGLFramebufferAttachment>>, u32),
    {
        let tex_id = texture.id();
        let attachments = [
            (&self.depth, constants::DEPTH_ATTACHMENT),
            (&self.stencil, constants::STENCIL_ATTACHMENT),
            (&self.depthstencil, constants::DEPTH_STENCIL_ATTACHMENT),
        ];

        fn has_matching_id(
            attachment: &DomRefCell<Option<WebGLFramebufferAttachment>>,
            target: &WebGLTextureId,
        ) -> bool {
            matches!(*attachment.borrow(), Some(WebGLFramebufferAttachment::Texture {
                                     texture: ref att_texture,
                                     ..
                                }) if att_texture.id() == *target)
        }

        for (attachment, name) in &attachments {
            if has_matching_id(attachment, &tex_id) {
                closure(attachment, *name);
            }
        }

        for (idx, attachment) in self.colors.iter().enumerate() {
            if has_matching_id(attachment, &tex_id) {
                let name = constants::COLOR_ATTACHMENT0 + idx as u32;
                closure(attachment, name);
            }
        }
    }

    pub(crate) fn detach_renderbuffer(&self, rb: &WebGLRenderbuffer) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let mut depth_or_stencil_updated = false;
        self.with_matching_renderbuffers(rb, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            if let Some(att) = &*att.borrow() {
                att.detach();
            }
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    pub(crate) fn detach_texture(&self, texture: &WebGLTexture) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let mut depth_or_stencil_updated = false;
        self.with_matching_textures(texture, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            if let Some(att) = &*att.borrow() {
                att.detach();
            }
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    pub(crate) fn invalidate_renderbuffer(&self, rb: &WebGLRenderbuffer) {
        self.with_matching_renderbuffers(rb, |_att, _| {
            self.is_initialized.set(false);
            self.update_status();
        });
    }

    pub(crate) fn invalidate_texture(&self, texture: &WebGLTexture) {
        self.with_matching_textures(texture, |_att, _name| {
            self.update_status();
        });
    }

    pub(crate) fn set_read_buffer(&self, buffer: u32) -> WebGLResult<()> {
        let context = self.upcast::<WebGLObject>().context();

        match buffer {
            constants::NONE => {},
            _ if context.valid_color_attachment_enum(buffer) => {},
            _ => return Err(WebGLError::InvalidOperation),
        };

        *self.color_read_buffer.borrow_mut() = buffer;
        context.send_command(WebGLCommand::ReadBuffer(buffer));
        Ok(())
    }

    pub(crate) fn set_draw_buffers(&self, buffers: Vec<u32>) -> WebGLResult<()> {
        let context = self.upcast::<WebGLObject>().context();

        if buffers.len() > context.limits().max_draw_buffers as usize {
            return Err(WebGLError::InvalidValue);
        }

        let enums_valid = buffers
            .iter()
            .all(|&val| val == constants::NONE || context.valid_color_attachment_enum(val));
        if !enums_valid {
            return Err(WebGLError::InvalidEnum);
        }

        let values_valid = buffers.iter().enumerate().all(|(i, &val)| {
            val == constants::NONE || val == (constants::COLOR_ATTACHMENT0 + i as u32)
        });
        if !values_valid {
            return Err(WebGLError::InvalidOperation);
        }

        self.color_draw_buffers.borrow_mut().clone_from(&buffers);
        context.send_command(WebGLCommand::DrawBuffers(buffers));
        Ok(())
    }

    pub(crate) fn read_buffer(&self) -> u32 {
        *self.color_read_buffer.borrow()
    }

    pub(crate) fn draw_buffer_i(&self, index: usize) -> u32 {
        let buffers = &*self.color_draw_buffers.borrow();
        *buffers.get(index).unwrap_or(&constants::NONE)
    }

    pub(crate) fn target(&self) -> Option<u32> {
        self.target.get()
    }
}

impl Drop for WebGLFramebuffer {
    fn drop(&mut self) {
        self.delete(Operation::Fallible);
    }
}

static INTERESTING_ATTACHMENT_POINTS: &[u32] = &[
    constants::DEPTH_ATTACHMENT,
    constants::STENCIL_ATTACHMENT,
    constants::DEPTH_STENCIL_ATTACHMENT,
];
